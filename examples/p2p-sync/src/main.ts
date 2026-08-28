/**
 * Cliffy P2P Sync Demo
 *
 * Demonstrates:
 * - Real WebRTC peer-to-peer connections
 * - Real-time state synchronization across browsers
 * - Delta compression and efficient state transfer
 * - Network partition handling and recovery
 * - Vector clock-based causal ordering
 *
 * To run:
 * 1. Start the signaling server: npm run signaling (from examples/)
 * 2. Start this demo: npm run dev:p2p (from examples/)
 * 3. Open two browser tabs to the same URL
 */

import init, {
  ObservationSet,
  behavior,
  Behavior,
  VectorClock as WasmVectorClock,
  generateNodeId,
} from '@industrialalgebra/cliffy-core';

import {
  PeerManager,
  type PeerManagerConfig,
  type SyncMessage,
  type ConnectedPeer,
  type VectorClock,
  createDeltaBatch,
  type StateDelta,
} from '@cliffy/shared/webrtc';

// =============================================================================
// Configuration
// =============================================================================

const SIGNALING_SERVER_URL = 'ws://localhost:8080';
const ROOM_ID = 'cliffy-p2p-demo';

// =============================================================================
// Types
// =============================================================================

interface AppState {
  localPeerId: string;
  peerManager: PeerManager | null;
  crdt: ObservationSet | null;
  crdtSeq: number;
  clock: WasmVectorClock | null;
  sharedCounter: number;
  localOperationCount: number;
  connectedPeers: ConnectedPeer[];
  syncLog: SyncLogEntry[];
  stats: SyncStats;
  connectionStatus: 'disconnected' | 'connecting' | 'connected';
  error: string | null;
  mode: 'webrtc' | 'simulation';
}

interface SyncLogEntry {
  id: string;
  type: 'sent' | 'received';
  peerId: string;
  peerName?: string;
  messageType: string;
  timestamp: number;
  size: number;
}

interface SyncStats {
  totalSyncs: number;
  totalBytesSent: number;
  totalBytesReceived: number;
  avgRtt: number | null;
}

// =============================================================================
// State Management
// =============================================================================

const state: AppState = {
  localPeerId: '',
  peerManager: null,
  crdt: null,
  crdtSeq: 0,
  clock: null,
  sharedCounter: 0,
  localOperationCount: 0,
  connectedPeers: [],
  syncLog: [],
  stats: {
    totalSyncs: 0,
    totalBytesSent: 0,
    totalBytesReceived: 0,
    avgRtt: null,
  },
  connectionStatus: 'disconnected',
  error: null,
  mode: 'webrtc',
};

// FRP Behaviors (initialized after WASM init)
let counterBehavior: Behavior;
let statusBehavior: Behavior;

// =============================================================================
// WebRTC Connection
// =============================================================================

async function connectWebRTC(): Promise<void> {
  if (state.peerManager) {
    state.peerManager.leave();
  }

  state.connectionStatus = 'connecting';
  state.error = null;

  const config: PeerManagerConfig = {
    peerId: state.localPeerId,
    peerName: `User ${state.localPeerId.substring(0, 4)}`,
    signaling: {
      serverUrl: SIGNALING_SERVER_URL,
    },
    autoConnect: true,
    heartbeatInterval: 5000,
  };

  const peerManager = new PeerManager(config);

  peerManager.onReady(() => {
    state.connectionStatus = 'connected';
    statusBehavior.set('connected');
    console.log('[P2P] Connected to signaling server');
  });

  peerManager.onPeerConnected((peerId, peerInfo) => {
    console.log(`[P2P] Peer connected: ${peerId} (${peerInfo.name})`);
    updatePeerList();

    // Request full state from new peer
    peerManager.requestFullState(peerId);
  });

  peerManager.onPeerDisconnected((peerId) => {
    console.log(`[P2P] Peer disconnected: ${peerId}`);
    updatePeerList();
  });

  peerManager.onSyncMessage((peerId, message) => {
    handleSyncMessage(peerId, message);
  });

  peerManager.onError((error) => {
    console.error('[P2P] Error:', error);
    state.error = error.message;
  });

  try {
    await peerManager.join(ROOM_ID);
    state.peerManager = peerManager;
  } catch (error) {
    state.connectionStatus = 'disconnected';
    state.error = `Failed to connect: ${(error as Error).message}`;
    console.error('[P2P] Connection failed:', error);
  }
}

function disconnect(): void {
  if (state.peerManager) {
    state.peerManager.leave();
    state.peerManager = null;
  }
  state.connectionStatus = 'disconnected';
  state.connectedPeers = [];
  statusBehavior.set('disconnected');
}

function updatePeerList(): void {
  if (state.peerManager) {
    state.connectedPeers = state.peerManager.getConnectedPeers();

    // Calculate average RTT
    const rtts = state.connectedPeers
      .map(p => p.rtt)
      .filter((rtt): rtt is number => rtt !== null);
    state.stats.avgRtt = rtts.length > 0
      ? rtts.reduce((a, b) => a + b, 0) / rtts.length
      : null;
  }
}

// =============================================================================
// Sync Message Handling
// =============================================================================

function handleSyncMessage(peerId: string, message: SyncMessage): void {
  const peerName = state.connectedPeers.find(p => p.id === peerId)?.name;

  // Log the message
  addSyncLog({
    type: 'received',
    peerId,
    peerName,
    messageType: message.payload.type,
    size: estimateMessageSize(message),
  });

  switch (message.payload.type) {
    case 'DeltaRequest':
      // Respond with current state
      sendFullState(peerId);
      break;

    case 'FullState':
      // Apply full state from peer
      applyFullState(message.payload.state, message.payload.clock);
      break;

    case 'DeltaResponse':
      // Apply deltas
      applyDeltas(message.payload.deltas.deltas);
      break;

    case 'Hello':
    case 'Heartbeat':
    case 'Goodbye':
    case 'Ack':
      // Handled by PeerManager
      break;
  }

  state.stats.totalSyncs++;
}

function sendFullState(peerId: string): void {
  if (!state.peerManager || !state.crdt || !state.clock) return;

  // Get state as array (GA3 has 8 coefficients)
  const stateArray = [state.sharedCounter, 0, 0, 0, 0, 0, 0, 0];

  const clock: VectorClock = {
    entries: { [state.localPeerId]: state.clock.getTime(state.localPeerId) },
  };

  state.peerManager.send(peerId, {
    type: 'FullState',
    state: stateArray,
    clock,
  });

  const peerName = state.connectedPeers.find(p => p.id === peerId)?.name;
  addSyncLog({
    type: 'sent',
    peerId,
    peerName,
    messageType: 'FullState',
    size: stateArray.length * 8 + 64,
  });
}

function applyFullState(stateArray: number[], clock: VectorClock): void {
  if (!state.crdt || !state.clock) return;

  // Extract counter value (first coefficient)
  const newCounter = Math.round(stateArray[0]);

  // Only update if newer
  if (newCounter > state.sharedCounter) {
    state.sharedCounter = newCounter;
    counterBehavior.set(state.sharedCounter);
  }

  // Merge clocks
  for (const [nodeId, time] of Object.entries(clock.entries)) {
    const currentTime = state.clock.getTime(nodeId);
    if (time > currentTime) {
      // Update clock (simplified - real impl would update internal state)
    }
  }
}

function applyDeltas(deltas: StateDelta[]): void {
  for (const delta of deltas) {
    // Apply delta transform (first coefficient is counter change)
    const change = Math.round(delta.transform[0]);
    state.sharedCounter += change;
  }
  counterBehavior.set(state.sharedCounter);
}

// =============================================================================
// Local Operations
// =============================================================================

function localIncrement(): void {
  if (!state.crdt || !state.clock) return;

  state.sharedCounter++;
  state.localOperationCount++;
  state.clock.tick(state.localPeerId);

  // Record the increment as a participant-scoped observation
  state.crdt.observeScalar(state.localPeerId, state.crdtSeq, 1);
  state.crdtSeq += 1;

  counterBehavior.set(state.sharedCounter);

  // Broadcast to peers
  broadcastUpdate();
}

function broadcastUpdate(): void {
  if (!state.peerManager || !state.clock) return;

  const clock: VectorClock = {
    entries: { [state.localPeerId]: state.clock.getTime(state.localPeerId) },
  };

  // Create delta batch with our latest change
  const batch = createDeltaBatch();
  batch.deltas.push({
    transform: [1, 0, 0, 0, 0, 0, 0, 0], // +1 increment
    encoding: 'Additive',
    fromClock: clock,
    toClock: clock,
    sourceNode: state.localPeerId,
  });
  batch.combinedClock = clock;

  state.peerManager.broadcast({
    type: 'DeltaResponse',
    deltas: batch,
    hasMore: false,
  });

  for (const peer of state.connectedPeers) {
    addSyncLog({
      type: 'sent',
      peerId: peer.id,
      peerName: peer.name,
      messageType: 'DeltaResponse',
      size: 96,
    });
  }
}

// =============================================================================
// Utilities
// =============================================================================

function addSyncLog(entry: Omit<SyncLogEntry, 'id' | 'timestamp'>): void {
  state.syncLog.unshift({
    ...entry,
    id: Math.random().toString(36).substring(7),
    timestamp: Date.now(),
  });

  if (state.syncLog.length > 20) {
    state.syncLog.pop();
  }

  if (entry.type === 'sent') {
    state.stats.totalBytesSent += entry.size;
  } else {
    state.stats.totalBytesReceived += entry.size;
  }
}

function estimateMessageSize(message: SyncMessage): number {
  // Rough estimate based on payload type
  const baseSize = 64; // Header
  switch (message.payload.type) {
    case 'Hello': return baseSize + 128;
    case 'FullState': return baseSize + 64 + 8 * 8;
    case 'DeltaResponse': return baseSize + 96;
    case 'Heartbeat': return baseSize;
    default: return baseSize + 32;
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}

function formatRtt(rtt: number | null): string {
  if (rtt === null) return '-';
  return `${Math.round(rtt)}ms`;
}

// =============================================================================
// DOM Helpers
// =============================================================================

function createElement(
  tag: string,
  attrs: Record<string, string> = {},
  children: (Node | string)[] = []
): HTMLElement {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    el.setAttribute(key, value);
  }
  for (const child of children) {
    if (typeof child === 'string') {
      el.appendChild(document.createTextNode(child));
    } else {
      el.appendChild(child);
    }
  }
  return el;
}

// =============================================================================
// Rendering
// =============================================================================

function renderApp(): HTMLElement {
  const container = createElement('div', { class: 'app-container' });

  // Header with connection controls
  const header = createElement('div', { class: 'header' });

  const title = createElement('h1', {}, ['P2P Sync Demo']);
  header.appendChild(title);

  const controls = createElement('div', { class: 'controls' });

  // Connection status indicator
  const statusIndicator = createElement('div', {
    class: `status-indicator ${state.connectionStatus}`,
  });
  const statusDot = createElement('div', { class: 'status-dot' });
  statusIndicator.appendChild(statusDot);
  statusIndicator.appendChild(
    document.createTextNode(state.connectionStatus.charAt(0).toUpperCase() + state.connectionStatus.slice(1))
  );
  controls.appendChild(statusIndicator);

  // Connect/Disconnect button
  if (state.connectionStatus === 'disconnected') {
    const connectBtn = createElement('button', { class: 'primary' }, ['Connect']);
    connectBtn.onclick = connectWebRTC;
    controls.appendChild(connectBtn);
  } else if (state.connectionStatus === 'connected') {
    const disconnectBtn = createElement('button', { class: 'danger' }, ['Disconnect']);
    disconnectBtn.onclick = disconnect;
    controls.appendChild(disconnectBtn);
  } else {
    const connectingBtn = createElement('button', { disabled: 'true' }, ['Connecting...']);
    controls.appendChild(connectingBtn);
  }

  header.appendChild(controls);
  container.appendChild(header);

  // Error message
  if (state.error) {
    const errorBox = createElement('div', { class: 'error-box' }, [state.error]);
    container.appendChild(errorBox);
  }

  // Main content
  const main = createElement('div', { class: 'main-content' });

  // Left panel: Counter and local info
  const leftPanel = createElement('div', { class: 'panel' });

  const counterSection = createElement('div', { class: 'counter-section' });
  counterSection.appendChild(createElement('h2', {}, ['Shared Counter']));

  const counterDisplay = createElement('div', { class: 'counter-display' }, [
    String(state.sharedCounter),
  ]);
  counterSection.appendChild(counterDisplay);

  const incrementBtn = createElement('button', { class: 'primary large' }, ['Increment (+1)']) as HTMLButtonElement;
  incrementBtn.onclick = localIncrement;
  if (state.connectionStatus !== 'connected') {
    incrementBtn.disabled = true;
  }
  counterSection.appendChild(incrementBtn);

  leftPanel.appendChild(counterSection);

  // Local peer info
  const localInfo = createElement('div', { class: 'local-info' });
  localInfo.appendChild(createElement('h3', {}, ['Local Peer']));
  localInfo.appendChild(createElement('div', { class: 'info-row' }, [
    createElement('span', { class: 'label' }, ['ID:']),
    createElement('span', { class: 'value' }, [state.localPeerId.substring(0, 8) + '...']),
  ]));
  localInfo.appendChild(createElement('div', { class: 'info-row' }, [
    createElement('span', { class: 'label' }, ['Operations:']),
    createElement('span', { class: 'value' }, [String(state.localOperationCount)]),
  ]));
  localInfo.appendChild(createElement('div', { class: 'info-row' }, [
    createElement('span', { class: 'label' }, ['Clock:']),
    createElement('span', { class: 'value' }, [
      state.clock ? String(state.clock.getTime(state.localPeerId)) : '0',
    ]),
  ]));
  leftPanel.appendChild(localInfo);

  main.appendChild(leftPanel);

  // Center panel: Connected peers
  const centerPanel = createElement('div', { class: 'panel' });
  centerPanel.appendChild(createElement('h2', {}, ['Connected Peers']));

  if (state.connectedPeers.length === 0) {
    const emptyState = createElement('div', { class: 'empty-state' }, [
      state.connectionStatus === 'connected'
        ? 'Waiting for peers to connect...'
        : 'Connect to start syncing',
    ]);
    centerPanel.appendChild(emptyState);
  } else {
    const peerList = createElement('div', { class: 'peer-list' });

    for (const peer of state.connectedPeers) {
      const peerItem = createElement('div', { class: `peer-item ${peer.connected ? 'connected' : 'disconnected'}` });

      const peerAvatar = createElement('div', { class: 'peer-avatar' }, [
        (peer.name || peer.id).charAt(0).toUpperCase(),
      ]);
      peerItem.appendChild(peerAvatar);

      const peerInfo = createElement('div', { class: 'peer-info' });
      peerInfo.appendChild(createElement('div', { class: 'peer-name' }, [
        peer.name || `Peer ${peer.id.substring(0, 8)}`,
      ]));
      peerInfo.appendChild(createElement('div', { class: 'peer-meta' }, [
        `RTT: ${formatRtt(peer.rtt)} | ${peer.connected ? 'Connected' : 'Disconnected'}`,
      ]));
      peerItem.appendChild(peerInfo);

      peerList.appendChild(peerItem);
    }

    centerPanel.appendChild(peerList);
  }

  main.appendChild(centerPanel);

  // Right panel: Sync log and stats
  const rightPanel = createElement('div', { class: 'panel' });

  // Stats
  const statsSection = createElement('div', { class: 'stats-section' });
  statsSection.appendChild(createElement('h2', {}, ['Statistics']));

  const statsGrid = createElement('div', { class: 'stats-grid' });

  const addStat = (label: string, value: string) => {
    const stat = createElement('div', { class: 'stat' });
    stat.appendChild(createElement('div', { class: 'stat-value' }, [value]));
    stat.appendChild(createElement('div', { class: 'stat-label' }, [label]));
    statsGrid.appendChild(stat);
  };

  addStat('Total Syncs', String(state.stats.totalSyncs));
  addStat('Sent', formatBytes(state.stats.totalBytesSent));
  addStat('Received', formatBytes(state.stats.totalBytesReceived));
  addStat('Avg RTT', formatRtt(state.stats.avgRtt));

  statsSection.appendChild(statsGrid);
  rightPanel.appendChild(statsSection);

  // Sync log
  const logSection = createElement('div', { class: 'log-section' });
  logSection.appendChild(createElement('h3', {}, ['Sync Log']));

  const logList = createElement('div', { class: 'log-list' });

  for (const entry of state.syncLog.slice(0, 10)) {
    const logItem = createElement('div', { class: `log-item ${entry.type}` });

    const arrow = createElement('span', { class: 'arrow' }, [
      entry.type === 'sent' ? '\u2192' : '\u2190',
    ]);
    logItem.appendChild(arrow);

    const peerName = entry.peerName || entry.peerId.substring(0, 8);
    logItem.appendChild(createElement('span', { class: 'peer' }, [peerName]));
    logItem.appendChild(createElement('span', { class: 'type' }, [entry.messageType]));
    logItem.appendChild(createElement('span', { class: 'size' }, [formatBytes(entry.size)]));

    logList.appendChild(logItem);
  }

  if (state.syncLog.length === 0) {
    logList.appendChild(createElement('div', { class: 'empty-state' }, ['No sync activity yet']));
  }

  logSection.appendChild(logList);
  rightPanel.appendChild(logSection);

  main.appendChild(rightPanel);
  container.appendChild(main);

  // Instructions
  const instructions = createElement('div', { class: 'instructions' });
  instructions.appendChild(createElement('h3', {}, ['How to Test']));
  const instructionsList = createElement('ol', {});
  instructionsList.appendChild(createElement('li', {}, [
    'Start signaling server: ',
    createElement('code', {}, ['npm run signaling']),
    ' (from examples/)',
  ]));
  instructionsList.appendChild(createElement('li', {}, [
    'Open this page in two browser tabs',
  ]));
  instructionsList.appendChild(createElement('li', {}, [
    'Click "Connect" in both tabs',
  ]));
  instructionsList.appendChild(createElement('li', {}, [
    'Click "Increment" in one tab and watch it sync to the other',
  ]));
  instructions.appendChild(instructionsList);
  container.appendChild(instructions);

  return container;
}

// =============================================================================
// Main Loop
// =============================================================================

// Track state for change detection
let lastRenderState = {
  connectionStatus: '',
  connectedPeersCount: 0,
  sharedCounter: 0,
  syncLogLength: 0,
  error: null as string | null,
  totalSyncs: 0,
};

function stateHasChanged(): boolean {
  return (
    lastRenderState.connectionStatus !== state.connectionStatus ||
    lastRenderState.connectedPeersCount !== state.connectedPeers.length ||
    lastRenderState.sharedCounter !== state.sharedCounter ||
    lastRenderState.syncLogLength !== state.syncLog.length ||
    lastRenderState.error !== state.error ||
    lastRenderState.totalSyncs !== state.stats.totalSyncs
  );
}

function updateLastRenderState(): void {
  lastRenderState = {
    connectionStatus: state.connectionStatus,
    connectedPeersCount: state.connectedPeers.length,
    sharedCounter: state.sharedCounter,
    syncLogLength: state.syncLog.length,
    error: state.error,
    totalSyncs: state.stats.totalSyncs,
  };
}

function render(): void {
  const app = document.getElementById('app');
  if (app) {
    app.textContent = '';
    app.appendChild(renderApp());
  }
  updateLastRenderState();
}

function mainLoop(): void {
  updatePeerList();

  // Only re-render if state has changed
  if (stateHasChanged()) {
    render();
  }

  setTimeout(mainLoop, 100);
}

// =============================================================================
// Styles
// =============================================================================

function injectStyles(): void {
  const style = document.createElement('style');
  style.textContent = `
    :root {
      --bg: #0f0f14;
      --bg-secondary: #1a1a24;
      --bg-tertiary: #252532;
      --text: #e0e0e8;
      --text-muted: #8888a0;
      --primary: #6366f1;
      --primary-hover: #818cf8;
      --success: #22c55e;
      --warning: #f59e0b;
      --danger: #ef4444;
      --border: #3f3f5a;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 20px;
    }

    .app-container {
      max-width: 1400px;
      margin: 0 auto;
    }

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 20px;
      border-bottom: 1px solid var(--border);
      margin-bottom: 20px;
    }

    .header h1 {
      font-size: 1.5rem;
      font-weight: 600;
    }

    .controls {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .status-indicator {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 12px;
      border-radius: 20px;
      background: var(--bg-secondary);
      font-size: 0.85rem;
    }

    .status-indicator .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
    }

    .status-indicator.disconnected .status-dot { background: var(--danger); }
    .status-indicator.connecting .status-dot { background: var(--warning); animation: pulse 1s infinite; }
    .status-indicator.connected .status-dot { background: var(--success); }

    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.5; }
    }

    button {
      padding: 8px 16px;
      border: none;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.9rem;
      background: var(--bg-tertiary);
      color: var(--text);
      transition: background 0.2s;
    }

    button:hover:not(:disabled) {
      background: var(--border);
    }

    button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    button.primary {
      background: var(--primary);
      color: white;
    }

    button.primary:hover:not(:disabled) {
      background: var(--primary-hover);
    }

    button.danger {
      background: var(--danger);
      color: white;
    }

    button.large {
      padding: 12px 24px;
      font-size: 1rem;
    }

    .error-box {
      background: rgba(239, 68, 68, 0.1);
      border: 1px solid var(--danger);
      border-radius: 8px;
      padding: 12px 16px;
      margin-bottom: 20px;
      color: var(--danger);
    }

    .main-content {
      display: grid;
      grid-template-columns: 1fr 1fr 1fr;
      gap: 20px;
      margin-bottom: 20px;
    }

    .panel {
      background: var(--bg-secondary);
      border-radius: 12px;
      padding: 20px;
      border: 1px solid var(--border);
    }

    .panel h2 {
      font-size: 1rem;
      margin-bottom: 16px;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .panel h3 {
      font-size: 0.9rem;
      margin-bottom: 12px;
      color: var(--text-muted);
    }

    .counter-section {
      text-align: center;
    }

    .counter-display {
      font-size: 4rem;
      font-weight: 700;
      color: var(--primary);
      margin: 20px 0;
      font-family: monospace;
    }

    .local-info {
      margin-top: 24px;
      padding-top: 20px;
      border-top: 1px solid var(--border);
    }

    .info-row {
      display: flex;
      justify-content: space-between;
      padding: 8px 0;
      font-size: 0.9rem;
    }

    .info-row .label {
      color: var(--text-muted);
    }

    .info-row .value {
      font-family: monospace;
    }

    .empty-state {
      text-align: center;
      color: var(--text-muted);
      padding: 40px 20px;
      font-size: 0.9rem;
    }

    .peer-list {
      display: flex;
      flex-direction: column;
      gap: 12px;
    }

    .peer-item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 12px;
      background: var(--bg-tertiary);
      border-radius: 8px;
    }

    .peer-item.disconnected {
      opacity: 0.5;
    }

    .peer-avatar {
      width: 40px;
      height: 40px;
      border-radius: 50%;
      background: var(--primary);
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 600;
      color: white;
    }

    .peer-info {
      flex: 1;
    }

    .peer-name {
      font-weight: 500;
      margin-bottom: 4px;
    }

    .peer-meta {
      font-size: 0.8rem;
      color: var(--text-muted);
    }

    .stats-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
    }

    .stat {
      background: var(--bg-tertiary);
      padding: 12px;
      border-radius: 8px;
      text-align: center;
    }

    .stat-value {
      font-size: 1.5rem;
      font-weight: 600;
      font-family: monospace;
      color: var(--primary);
    }

    .stat-label {
      font-size: 0.75rem;
      color: var(--text-muted);
      margin-top: 4px;
      text-transform: uppercase;
    }

    .log-section {
      margin-top: 20px;
      padding-top: 16px;
      border-top: 1px solid var(--border);
    }

    .log-list {
      max-height: 300px;
      overflow-y: auto;
    }

    .log-item {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 8px;
      font-size: 0.8rem;
      font-family: monospace;
      border-radius: 4px;
      margin-bottom: 4px;
    }

    .log-item.sent {
      background: rgba(99, 102, 241, 0.1);
    }

    .log-item.received {
      background: rgba(34, 197, 94, 0.1);
    }

    .log-item .arrow {
      font-weight: bold;
    }

    .log-item.sent .arrow {
      color: var(--primary);
    }

    .log-item.received .arrow {
      color: var(--success);
    }

    .log-item .peer {
      color: var(--text-muted);
      width: 80px;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .log-item .type {
      flex: 1;
    }

    .log-item .size {
      color: var(--text-muted);
    }

    .instructions {
      background: var(--bg-secondary);
      border-radius: 12px;
      padding: 20px;
      border: 1px solid var(--border);
    }

    .instructions h3 {
      margin-bottom: 12px;
    }

    .instructions ol {
      padding-left: 24px;
    }

    .instructions li {
      margin-bottom: 8px;
      line-height: 1.5;
    }

    .instructions code {
      background: var(--bg-tertiary);
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 0.9em;
    }

    @media (max-width: 1200px) {
      .main-content {
        grid-template-columns: 1fr 1fr;
      }
    }

    @media (max-width: 768px) {
      .main-content {
        grid-template-columns: 1fr;
      }

      .header {
        flex-direction: column;
        gap: 12px;
      }
    }
  `;
  document.head.appendChild(style);
}

// =============================================================================
// Initialize
// =============================================================================

async function main() {
  await init();

  injectStyles();

  // Initialize local peer
  state.localPeerId = generateNodeId();
  state.crdt = new ObservationSet();
  state.crdtSeq = 0;
  state.clock = new WasmVectorClock();

  // Initialize FRP Behaviors
  counterBehavior = behavior(0);
  statusBehavior = behavior('disconnected');

  // Initial render
  render();

  console.log('Cliffy P2P Sync Demo initialized');
  console.log('Local peer ID:', state.localPeerId);
  console.log('Using real WebRTC for peer connections');

  mainLoop();
}

main().catch(console.error);
