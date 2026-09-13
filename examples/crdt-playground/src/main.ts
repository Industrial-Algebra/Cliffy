/**
 * Cliffy CRDT Playground — Phase 1 salvage showcase
 *
 * Demonstrates:
 * - ObservationSet from cliffy-protocols via WASM bindings (the sound floor)
 * - Participant-scoped observations: peers observe, sets union, nothing annihilates
 * - Deterministic projections: scalarMean consensus on every replica
 * - Live probe panel: the four value oracles that answer the February question
 *   (the old GeometricCRDT failed all four — see the salvage plan)
 */

import init, { ObservationSet, generateNodeId } from '@industrialalgebra/cliffy-core';

// =============================================================================
// UI State
// =============================================================================

interface PeerState {
  id: string;
  set: ObservationSet;
  seq: number;
  latest: number;
  displayName: string;
}

interface LogEntry {
  time: string;
  peer: string;
  action: string;
}

const state = {
  peers: new Map<string, PeerState>(),
  log: [] as LogEntry[],
  mergeResult: null as ObservationSet | null,
  initialized: false,
};

function addLogEntry(peer: string, action: string): void {
  const now = new Date();
  state.log.unshift({
    time: `${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`,
    peer,
    action,
  });
  if (state.log.length > 50) state.log.pop();
  render();
}

// =============================================================================
// Observation operations using real WASM bindings
// =============================================================================

function observe(peerId: string, value: number): void {
  const peer = state.peers.get(peerId);
  if (!peer) return;

  peer.set.observeScalar(peer.id, peer.seq, value);
  peer.seq += 1;
  peer.latest = value;
  addLogEntry(peerId, `Observed ${value} → set now holds ${peer.set.len()} observation(s)`);
}

function initializePeers(): void {
  state.peers.clear();

  const specs: Array<[string, string]> = [
    ['peer1', 'Peer 1'],
    ['peer2', 'Peer 2'],
    ['peer3', 'Peer 3'],
  ];
  for (const [key, displayName] of specs) {
    const id = generateNodeId();
    const set = new ObservationSet();
    set.observeScalar(id, 0, 10.0);
    state.peers.set(key, { id, set, seq: 1, latest: 10.0, displayName });
  }

  state.log = [];
  state.mergeResult = null;
  addLogEntry('system', 'Initialized 3 peers, each observing 10.0 (real WASM ObservationSet)');
}

function syncPeers(fromId: string, toId: string): void {
  const from = state.peers.get(fromId);
  const to = state.peers.get(toId);
  if (!from || !to) return;

  const changed = to.set.merge(from.set);
  addLogEntry(
    'system',
    `Synced ${fromId} → ${toId}: ${changed ? 'absorbed new observations' : 'already convergent'} (len = ${to.set.len()})`
  );
}

function mergeAll(): void {
  const peerList = Array.from(state.peers.values());
  if (peerList.length < 2) return;

  // Full-mesh union: every peer absorbs every other peer's observations.
  for (const target of peerList) {
    for (const source of peerList) {
      if (target !== source) {
        target.set.merge(source.set);
      }
    }
  }

  // Display copy: a fresh set with everything unioned in.
  const result = new ObservationSet();
  for (const peer of peerList) {
    result.merge(peer.set);
  }
  state.mergeResult = result;

  addLogEntry('system', `Merged all peers → ${result.len()} observations, consensus = ${result.scalarMean()?.toFixed(2)}`);
}

function reset(): void {
  initializePeers();
}

// =============================================================================
// The February probes — live value oracles (the salvage showcase)
// =============================================================================

interface ProbeResult {
  name: string;
  oldDesign: string;
  expectation: string;
  passed: boolean;
}

function runProbes(): ProbeResult[] {
  const a = generateNodeId();
  const b = generateNodeId();

  // Probe 1 — no annihilation: +5 and +10 merge; the mean is exactly 7.5.
  // (Old design: every merge returned zero.)
  const p1a = new ObservationSet();
  p1a.observeScalar(a, 0, 5.0);
  const p1b = new ObservationSet();
  p1b.observeScalar(b, 0, 10.0);
  p1a.merge(p1b);
  const probe1 = p1a.scalarMean() === 7.5 && p1a.len() === 2;

  // Probe 2 — hull: the mean of +1 and −1 is exactly 0.0.
  // (Old join: cosh(1) ≈ 1.543 — outside the hull of its arguments.)
  const p2 = new ObservationSet();
  p2.observeScalar(a, 0, 1.0);
  p2.observeScalar(b, 0, -1.0);
  const probe2 = p2.scalarMean() === 0.0;

  // Probe 3 — participant-scoped identity: two first observations coexist.
  // (Old design: len()-minted ids collided at 0; one was silently dropped.)
  const probe3 = p1a.len() === 2;

  // Probe 4 — convergence with a value oracle: diverge, merge both ways,
  // identical sets AND the specified mean on both replicas.
  const p4a = new ObservationSet();
  p4a.observeScalar(a, 0, 10.0);
  const p4b = new ObservationSet();
  p4b.observeScalar(b, 0, 5.0);
  p4a.merge(p4b);
  p4b.merge(p4a);
  const probe4 =
    p4a.len() === p4b.len() &&
    p4a.scalarMean() === 7.5 &&
    p4b.scalarMean() === 7.5;

  return [
    {
      name: 'Merge does not annihilate',
      oldDesign: 'old: every merge → 0',
      expectation: '+5/+10 → mean 7.5, both survive',
      passed: probe1,
    },
    {
      name: 'Consensus stays in the hull',
      oldDesign: 'old: join(+1,−1) = cosh(1)',
      expectation: 'mean(+1,−1) = 0.0 exactly',
      passed: probe2,
    },
    {
      name: 'Participant-scoped identity',
      oldDesign: 'old: ids collided at 0',
      expectation: 'first observations coexist',
      passed: probe3,
    },
    {
      name: 'Convergence with a value oracle',
      oldDesign: 'old: agreement-only oracle',
      expectation: 'both directions → identical, mean 7.5',
      passed: probe4,
    },
  ];
}

// =============================================================================
// Safe DOM Rendering Helpers
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

function checkConvergence(): boolean {
  const peers = Array.from(state.peers.values());
  if (peers.length === 0) return true;
  const firstLen = peers[0].set.len();
  const firstMean = peers[0].set.scalarMean();
  return peers.every(
    (p) => p.set.len() === firstLen && p.set.scalarMean() === firstMean
  );
}

function createPeerCard(key: string, peer: PeerState): HTMLElement {
  const card = createElement('div', { class: `peer-card ${key}` });

  // Header
  const header = createElement('div', { class: 'peer-header' });
  const name = createElement('span', { class: 'peer-name' }, [
    createElement('span', { class: 'peer-dot' }),
    peer.displayName,
  ]);
  const peerId = createElement('span', { class: 'peer-id' }, [peer.id.slice(0, 8) + '...']);
  header.appendChild(name);
  header.appendChild(peerId);
  card.appendChild(header);

  // State display
  const stateDisplay = createElement('div', { class: 'state-display' });
  stateDisplay.appendChild(
    createElement('div', { class: 'state-label' }, ['Latest Observation'])
  );
  stateDisplay.appendChild(
    createElement('div', { class: 'state-value' }, [peer.latest.toFixed(2)])
  );
  stateDisplay.appendChild(
    createElement('div', { class: 'vector-clock' }, [
      `Set: ${peer.set.len()} observation(s)`,
    ])
  );
  stateDisplay.appendChild(
    createElement('div', { class: 'op-count' }, [
      `Consensus (mean): ${peer.set.scalarMean()?.toFixed(2) ?? '—'}`,
    ])
  );
  card.appendChild(stateDisplay);

  // Operations — each button records an observation
  const ops = createElement('div', { class: 'operations' });

  const btn1 = createElement('button', {}, ['observe 15']);
  btn1.onclick = () => observe(key, 15);
  ops.appendChild(btn1);

  const btn2 = createElement('button', {}, ['observe −3']);
  btn2.onclick = () => observe(key, -3);
  ops.appendChild(btn2);

  const btn3 = createElement('button', {}, ['observe ×2']);
  btn3.onclick = () => observe(key, peer.latest * 2);
  ops.appendChild(btn3);

  card.appendChild(ops);

  return card;
}

function createVisualization(): HTMLElement {
  const peerStates = Array.from(state.peers.values());
  const values = peerStates.map((p) => p.set.scalarMean() ?? 0);
  const consensus = state.mergeResult?.scalarMean() ?? null;
  const maxState = Math.max(...values.map(Math.abs), Math.abs(consensus ?? 0), 1);

  const viz = createElement('div', { class: 'visualization' });
  viz.appendChild(createElement('div', { class: 'viz-axis x' }));
  viz.appendChild(createElement('div', { class: 'viz-axis y' }));

  const labelPlus = createElement('div', { class: 'viz-label' }, ['+']);
  labelPlus.style.left = '95%';
  labelPlus.style.top = '52%';
  viz.appendChild(labelPlus);

  const labelMinus = createElement('div', { class: 'viz-label' }, ['−']);
  labelMinus.style.left = '3%';
  labelMinus.style.top = '52%';
  viz.appendChild(labelMinus);

  const labelState = createElement('div', { class: 'viz-label' }, ['Consensus']);
  labelState.style.left = '52%';
  labelState.style.top = '5%';
  viz.appendChild(labelState);

  peerStates.forEach((peer, i) => {
    const x = 50 + ((values[i] ?? 0) / maxState) * 35;
    const y = 30 + i * 25;
    const point = createElement('div', { class: `viz-point peer${i + 1}` }, [`P${i + 1}`]);
    point.style.left = `${x}%`;
    point.style.top = `${y}%`;
    viz.appendChild(point);
  });

  if (consensus !== null) {
    const x = 50 + (consensus / maxState) * 35;
    const point = createElement('div', { class: 'viz-point merged' }, ['M']);
    point.style.left = `${x}%`;
    point.style.top = '80%';
    viz.appendChild(point);
  }

  return viz;
}

function createProbePanel(): HTMLElement {
  const section = createElement('div', { class: 'section' });
  section.appendChild(createElement('h2', {}, ['Value-Oracle Probes (live)']));
  const intro = createElement('p', {});
  intro.textContent =
    'Four checks run live against the WASM bindings on every render — the permanent ' +
    'answers to the 2026-02-25 rabbit-hole question. The old GeometricCRDT failed all four.';
  section.appendChild(intro);

  const grid = createElement('div', { class: 'peers-grid' });
  for (const probe of runProbes()) {
    const box = createElement('div', { class: 'concept-box' });
    const title = createElement(
      'h3',
      {},
      [`${probe.passed ? '✓' : '✗'} ${probe.name}`]
    );
    if (!probe.passed) title.style.color = '#e74c3c';
    box.appendChild(title);
    box.appendChild(createElement('div', { class: 'vector-clock' }, [probe.expectation]));
    box.appendChild(createElement('div', { class: 'op-count' }, [probe.oldDesign]));
    grid.appendChild(box);
  }
  section.appendChild(grid);
  return section;
}

function createLogSection(): HTMLElement {
  const log = createElement('div', { class: 'history-log' });

  if (state.log.length === 0) {
    const empty = createElement('div', {}, ['No observations yet...']);
    empty.style.color = 'var(--text-dim)';
    log.appendChild(empty);
  } else {
    for (const entry of state.log) {
      const row = createElement('div', { class: 'log-entry' });
      row.appendChild(createElement('span', { class: 'log-time' }, [entry.time]));
      row.appendChild(createElement('span', { class: `log-peer ${entry.peer}` }, [entry.peer]));
      row.appendChild(createElement('span', { class: 'log-action' }, [entry.action]));
      log.appendChild(row);
    }
  }

  return log;
}

function render(): void {
  const app = document.getElementById('app');
  if (!app) return;

  if (!state.initialized) {
    app.textContent = 'Initializing WASM...';
    return;
  }

  app.textContent = '';

  const isConverged = checkConvergence();
  const playground = createElement('div', { class: 'playground' });

  // === WASM Badge ===
  const badge = createElement('div', { class: 'wasm-badge' }, [
    '✓ Using real cliffy-protocols WASM bindings (Phase 1 sound floor)',
  ]);
  playground.appendChild(badge);

  // === Peers Section ===
  const peersSection = createElement('div', { class: 'section' });
  peersSection.appendChild(createElement('h2', {}, ['Distributed Peers']));
  const peersGrid = createElement('div', { class: 'peers-grid' });
  for (const [key, peer] of state.peers.entries()) {
    peersGrid.appendChild(createPeerCard(key, peer));
  }
  peersSection.appendChild(peersGrid);
  playground.appendChild(peersSection);

  // === Visualization Section ===
  const vizSection = createElement('div', { class: 'section' });
  vizSection.appendChild(createElement('h2', {}, ['Consensus Visualization']));
  vizSection.appendChild(createVisualization());

  const convergence = createElement(
    'div',
    { class: `convergence-indicator ${isConverged ? 'converged' : 'diverged'}` }
  );
  convergence.textContent = isConverged
    ? '✓ All peers convergent (equal sets — same length, same consensus)'
    : '⚠ Peers hold divergent observation sets';
  vizSection.appendChild(convergence);
  playground.appendChild(vizSection);

  // === Probe Panel ===
  playground.appendChild(createProbePanel());

  // === Sync & Merge Section ===
  const syncSection = createElement('div', { class: 'section' });
  syncSection.appendChild(createElement('h2', {}, ['Synchronization (union merge)']));

  const mergeSection = createElement('div', { class: 'merge-section' });
  const controls = createElement('div', { class: 'merge-controls' });

  const syncBtn1 = createElement('button', {}, ['Sync P1 → P2']);
  syncBtn1.onclick = () => syncPeers('peer1', 'peer2');
  controls.appendChild(syncBtn1);

  const syncBtn2 = createElement('button', {}, ['Sync P2 → P1']);
  syncBtn2.onclick = () => syncPeers('peer2', 'peer1');
  controls.appendChild(syncBtn2);

  const syncBtn3 = createElement('button', {}, ['Sync P2 → P3']);
  syncBtn3.onclick = () => syncPeers('peer2', 'peer3');
  controls.appendChild(syncBtn3);

  const syncBtn4 = createElement('button', {}, ['Sync P3 → P1']);
  syncBtn4.onclick = () => syncPeers('peer3', 'peer1');
  controls.appendChild(syncBtn4);

  const mergeBtn = createElement('button', { class: 'accent' }, ['Merge All (Converge)']);
  mergeBtn.onclick = () => mergeAll();
  controls.appendChild(mergeBtn);

  mergeSection.appendChild(controls);

  if (state.mergeResult) {
    const result = createElement('div', { class: 'merge-result' });
    result.appendChild(
      createElement('div', { class: 'state-label' }, ['Union of All Sets'])
    );
    result.appendChild(
      createElement('div', { class: 'state-value' }, [
        `${state.mergeResult.len()} obs → mean ${state.mergeResult.scalarMean()?.toFixed(2)}`,
      ])
    );
    mergeSection.appendChild(result);
  }

  syncSection.appendChild(mergeSection);

  const conceptBox1 = createElement('div', { class: 'concept-box' });
  conceptBox1.appendChild(createElement('h3', {}, ['How the Sound Floor Works']));
  const p1 = createElement('p', {});
  p1.textContent =
    'Each peer holds a grow-only ObservationSet. Observations are keyed by ' +
    '(participant_id, seq) — participant-scoped, so identities never collide. ' +
    'Merging is plain set union: associative, commutative, idempotent by ' +
    'construction. The consensus value is a deterministic projection (the ' +
    'arithmetic mean here; the Markley eigen-mean for orientations) — equal ' +
    'sets render bit-identical values on every replica. The merge is boring; ' +
    'the render is geometric.';
  conceptBox1.appendChild(p1);
  syncSection.appendChild(conceptBox1);
  playground.appendChild(syncSection);

  // === Log Section ===
  const logSection = createElement('div', { class: 'section' });
  logSection.appendChild(createElement('h2', {}, ['Observation Log']));
  logSection.appendChild(createLogSection());
  playground.appendChild(logSection);

  // === Reset Button ===
  const resetDiv = createElement('div', { class: 'reset-all' });
  const resetBtn = createElement('button', { class: 'primary' }, ['Reset All Peers']);
  resetBtn.onclick = () => reset();
  resetDiv.appendChild(resetBtn);
  playground.appendChild(resetDiv);

  app.appendChild(playground);
}

// =============================================================================
// Initialize
// =============================================================================

async function main() {
  await init();

  state.initialized = true;
  initializePeers();
  render();

  console.log('Cliffy CRDT Playground initialized (Phase 1 sound floor)');
  console.log('Available types: ObservationSet, VectorClock, generateNodeId');
}

main().catch((err) => {
  console.error('Failed to initialize:', err);
  const app = document.getElementById('app');
  if (app) {
    app.textContent = `Failed to initialize: ${err.message}`;
  }
});
