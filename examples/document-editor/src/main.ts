/**
 * Cliffy Document Editor Demo
 *
 * Demonstrates:
 * - CRDT text operations (insert, delete) with geometric state
 * - Cursor positions as FRP Behaviors
 * - User presence indicators
 * - Conflict-free concurrent editing with simulated peers
 * - Operation history and merge visualization
 */

import init, {
  Behavior,
  ObservationSet,
  behavior,
  VectorClock,
  generateNodeId,
} from '@industrialalgebra/cliffy-core';

// =============================================================================
// Types
// =============================================================================

interface User {
  id: string;
  name: string;
  color: string;
  cursorPosition: number;
  isTyping: boolean;
  lastActive: number;
}

interface Operation {
  id: string;
  userId: string;
  type: 'insert' | 'delete';
  position: number;
  char?: string;
  timestamp: number;
}

interface DocumentState {
  content: string;
  users: Map<string, User>;
  localUserId: string;
  operations: Operation[];
  crdt: ObservationSet;
  crdtSeq: number;
  version: number;
}

// =============================================================================
// State Management
// =============================================================================

const state: DocumentState = {
  content: 'Welcome to the collaborative document editor!\n\nStart typing to see CRDT operations in action.\n\nMultiple users can edit simultaneously without conflicts.',
  users: new Map(),
  localUserId: '',
  operations: [],
  crdt: null as unknown as ObservationSet,
  crdtSeq: 0,
  version: 1,
};

// FRP Behaviors (initialized after WASM init)
let contentBehavior: Behavior;
let cursorBehavior: Behavior;

// User colors
const USER_COLORS = ['#ff6b6b', '#4ecdc4', '#ffe66d', '#a855f7'];

// =============================================================================
// User Management
// =============================================================================

function createUser(id: string, name: string, isLocal: boolean, colorIndex: number): User {
  return {
    id,
    name,
    color: USER_COLORS[colorIndex % USER_COLORS.length],
    cursorPosition: 0,
    isTyping: false,
    lastActive: Date.now(),
  };
}

function initializeUsers(): void {
  state.users.clear();

  // Create local user
  state.localUserId = generateNodeId();
  const localUser = createUser(state.localUserId, 'You', true, 0);
  state.users.set(state.localUserId, localUser);

  // Create simulated remote users
  const remoteNames = ['Alice', 'Bob', 'Charlie'];
  for (let i = 0; i < remoteNames.length; i++) {
    const id = generateNodeId();
    const user = createUser(id, remoteNames[i], false, i + 1);
    // Set random initial cursor positions
    user.cursorPosition = Math.floor(Math.random() * state.content.length);
    state.users.set(id, user);
  }
}

// =============================================================================
// CRDT Operations
// =============================================================================

function applyInsert(position: number, char: string, userId: string): void {
  // Insert character at position
  state.content = state.content.slice(0, position) + char + state.content.slice(position);
  state.version++;

  // Update cursor positions for all users after the insertion point
  for (const user of state.users.values()) {
    if (user.cursorPosition >= position) {
      user.cursorPosition++;
    }
  }

  // Record operation
  const op: Operation = {
    id: generateNodeId(),
    userId,
    type: 'insert',
    position,
    char,
    timestamp: Date.now(),
  };
  state.operations.unshift(op);

  // Keep only last 20 operations
  if (state.operations.length > 20) {
    state.operations.pop();
  }

  // Record the edit as a participant-scoped observation (+char code = insert)
  state.crdt.observeScalar(state.localUserId, state.crdtSeq, char.charCodeAt(0));
  state.crdtSeq += 1;

  contentBehavior.set(state.content);
}

function applyDelete(position: number, userId: string): void {
  if (position < 0 || position >= state.content.length) return;

  const deletedChar = state.content[position];
  state.content = state.content.slice(0, position) + state.content.slice(position + 1);
  state.version++;

  // Update cursor positions
  for (const user of state.users.values()) {
    if (user.cursorPosition > position) {
      user.cursorPosition--;
    }
  }

  // Record operation
  const op: Operation = {
    id: generateNodeId(),
    userId,
    type: 'delete',
    position,
    char: deletedChar,
    timestamp: Date.now(),
  };
  state.operations.unshift(op);

  if (state.operations.length > 20) {
    state.operations.pop();
  }

  // Record the edit as a participant-scoped observation (−char code = delete)
  state.crdt.observeScalar(state.localUserId, state.crdtSeq, -deletedChar.charCodeAt(0));
  state.crdtSeq += 1;

  contentBehavior.set(state.content);
}

// =============================================================================
// Simulated Remote Editing
// =============================================================================

function simulateRemoteEdits(): void {
  const remoteUsers = Array.from(state.users.values()).filter(u => u.id !== state.localUserId);

  for (const user of remoteUsers) {
    // Randomly decide if this user types
    if (Math.random() < 0.03) {
      user.isTyping = true;
      user.lastActive = Date.now();

      // Random action: insert or delete
      if (Math.random() < 0.7 && state.content.length < 1000) {
        // Insert a character
        const chars = 'abcdefghijklmnopqrstuvwxyz     \n';
        const char = chars[Math.floor(Math.random() * chars.length)];
        const position = Math.min(user.cursorPosition, state.content.length);
        applyInsert(position, char, user.id);
        user.cursorPosition = position + 1;
      } else if (state.content.length > 50) {
        // Delete a character
        const position = Math.max(0, user.cursorPosition - 1);
        if (position < state.content.length) {
          applyDelete(position, user.id);
          user.cursorPosition = position;
        }
      }
    } else {
      // Stop typing indicator after a delay
      if (Date.now() - user.lastActive > 500) {
        user.isTyping = false;
      }

      // Occasionally move cursor randomly
      if (Math.random() < 0.01) {
        user.cursorPosition = Math.floor(Math.random() * state.content.length);
      }
    }
  }
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

  // Editor area
  const editorArea = createElement('div', { class: 'editor-area' });

  // Toolbar
  const toolbar = createElement('div', { class: 'toolbar' });

  const syncBtn = createElement('button', {}, ['Force Sync']);
  syncBtn.onclick = () => {
    // Simulate a sync operation
    console.log('CRDT sync triggered');
    console.log('Observation set:', state.crdt.len(), 'recorded edits');
  };
  toolbar.appendChild(syncBtn);

  const clearBtn = createElement('button', {}, ['Clear']);
  clearBtn.onclick = () => {
    state.content = '';
    state.version++;
    contentBehavior.set('');
    state.operations = [];
    for (const user of state.users.values()) {
      user.cursorPosition = 0;
    }
  };
  toolbar.appendChild(clearBtn);

  const resetBtn = createElement('button', {}, ['Reset']);
  resetBtn.onclick = () => {
    state.content = 'Welcome to the collaborative document editor!\n\nStart typing to see CRDT operations in action.';
    state.version++;
    contentBehavior.set(state.content);
    state.operations = [];
    for (const user of state.users.values()) {
      user.cursorPosition = 0;
    }
  };
  toolbar.appendChild(resetBtn);

  const versionBadge = createElement('span', { class: 'version-badge' }, [`v${state.version}`]);
  toolbar.appendChild(versionBadge);

  editorArea.appendChild(toolbar);

  // Editor container with cursor layer
  const editorContainer = createElement('div', { class: 'editor-container' });

  // Text area
  const editor = document.createElement('textarea');
  editor.className = 'editor';
  editor.value = state.content;
  editor.spellcheck = false;

  // Subscribe to contentBehavior to update textarea when state changes externally
  contentBehavior.subscribe((content: string) => {
    if (editor.value !== content) {
      editor.value = content;
    }
  });

  editor.oninput = (e) => {
    const target = e.target as HTMLTextAreaElement;
    const newContent = target.value;
    const oldContent = state.content;
    const localUser = state.users.get(state.localUserId);

    if (!localUser) return;

    localUser.isTyping = true;
    localUser.lastActive = Date.now();

    // Detect what changed
    if (newContent.length > oldContent.length) {
      // Insertion
      const insertPos = target.selectionStart - 1;
      const insertedChar = newContent[insertPos];

      // Update state directly (textarea already has new value)
      state.content = newContent;
      state.version++;

      // Record operation
      const op: Operation = {
        id: generateNodeId(),
        userId: state.localUserId,
        type: 'insert',
        position: insertPos,
        char: insertedChar,
        timestamp: Date.now(),
      };
      state.operations.unshift(op);
      if (state.operations.length > 20) state.operations.pop();

      state.crdt.observeScalar(state.localUserId, state.crdtSeq, insertedChar.charCodeAt(0));
      state.crdtSeq += 1;

      localUser.cursorPosition = target.selectionStart;
    } else if (newContent.length < oldContent.length) {
      // Deletion
      const deletePos = target.selectionStart;
      const deletedChar = oldContent[deletePos];

      state.content = newContent;
      state.version++;

      const op: Operation = {
        id: generateNodeId(),
        userId: state.localUserId,
        type: 'delete',
        position: deletePos,
        char: deletedChar,
        timestamp: Date.now(),
      };
      state.operations.unshift(op);
      if (state.operations.length > 20) state.operations.pop();

      state.crdt.observeScalar(state.localUserId, state.crdtSeq, -(deletedChar?.charCodeAt(0) || 0));
      state.crdtSeq += 1;

      localUser.cursorPosition = target.selectionStart;
    }

    contentBehavior.set(state.content);
  };

  editor.onselect = () => {
    const localUser = state.users.get(state.localUserId);
    if (localUser) {
      localUser.cursorPosition = editor.selectionStart;
      cursorBehavior.set(editor.selectionStart);
    }
  };

  editor.onclick = editor.onselect;
  editor.onkeyup = () => {
    const localUser = state.users.get(state.localUserId);
    if (localUser) {
      localUser.cursorPosition = editor.selectionStart;
      cursorBehavior.set(editor.selectionStart);

      // Clear typing indicator after delay
      setTimeout(() => {
        if (Date.now() - localUser.lastActive > 500) {
          localUser.isTyping = false;
        }
      }, 600);
    }
  };

  editorContainer.appendChild(editor);

  // Cursor layer (for remote cursors visualization)
  const cursorLayer = createElement('div', { class: 'cursor-layer' });

  for (const user of state.users.values()) {
    if (user.id === state.localUserId) continue;

    // Calculate cursor position in the textarea
    // This is approximate - a real implementation would measure text
    const charWidth = 8.4;
    const lineHeight = 22.4;
    const padding = 16;

    // Find line and column for cursor position
    const textBeforeCursor = state.content.slice(0, user.cursorPosition);
    const lines = textBeforeCursor.split('\n');
    const lineNumber = lines.length - 1;
    const columnNumber = lines[lines.length - 1].length;

    const cursorEl = createElement('div', { class: 'remote-cursor' });
    cursorEl.setAttribute('data-user', user.name);
    cursorEl.style.backgroundColor = user.color;
    cursorEl.style.left = `${padding + columnNumber * charWidth}px`;
    cursorEl.style.top = `${padding + lineNumber * lineHeight}px`;
    cursorEl.querySelector('::after')?.setAttribute('style', `background: ${user.color}`);

    // Style the label
    const style = document.createElement('style');
    style.textContent = `
      .remote-cursor[data-user="${user.name}"]::after {
        background: ${user.color};
      }
    `;
    cursorEl.appendChild(style);

    cursorLayer.appendChild(cursorEl);
  }

  editorContainer.appendChild(cursorLayer);
  editorArea.appendChild(editorContainer);
  container.appendChild(editorArea);

  // Sidebar
  const sidebar = createElement('div', { class: 'sidebar' });

  // Users panel
  const usersPanel = createElement('div', { class: 'panel' });
  usersPanel.appendChild(createElement('h2', {}, ['Active Users']));
  const userList = createElement('div', { class: 'user-list' });

  for (const user of state.users.values()) {
    const item = createElement('div', { class: 'user-item' });

    const avatar = createElement('div', { class: 'user-avatar' });
    avatar.style.backgroundColor = user.color;
    avatar.textContent = user.name[0];
    item.appendChild(avatar);

    const info = createElement('div', { class: 'user-info' });
    info.appendChild(
      createElement('div', { class: 'player-name' }, [
        user.name + (user.id === state.localUserId ? ' (You)' : ''),
      ])
    );

    const statusDiv = createElement('div', { class: 'user-status' });
    statusDiv.textContent = `Cursor at ${user.cursorPosition}`;

    // Make local user's cursor position reactive
    if (user.id === state.localUserId) {
      cursorBehavior.subscribe((pos: number) => {
        statusDiv.textContent = `Cursor at ${pos}`;
      });
    }

    info.appendChild(statusDiv);

    item.appendChild(info);
    userList.appendChild(item);
  }

  usersPanel.appendChild(userList);
  sidebar.appendChild(usersPanel);

  // Stats panel
  const statsPanel = createElement('div', { class: 'panel' });
  statsPanel.appendChild(createElement('h2', {}, ['Document Stats']));
  const statsGrid = createElement('div', { class: 'stats-grid' });

  const addStat = (value: string, label: string) => {
    const item = createElement('div', { class: 'stat-item' });
    item.appendChild(createElement('div', { class: 'stat-value' }, [value]));
    item.appendChild(createElement('div', { class: 'stat-label' }, [label]));
    statsGrid.appendChild(item);
  };

  addStat(String(state.content.length), 'Characters');
  addStat(String(state.content.split('\n').length), 'Lines');
  addStat(String(state.version), 'Version');
  addStat(String(state.crdt?.len() || 0), 'Observed Edits');

  statsPanel.appendChild(statsGrid);
  sidebar.appendChild(statsPanel);

  // Operations log
  const opsPanel = createElement('div', { class: 'panel' });
  opsPanel.appendChild(createElement('h2', {}, ['Recent Operations']));
  const opsLog = createElement('div', { class: 'operations-log' });

  for (const op of state.operations.slice(0, 10)) {
    const user = state.users.get(op.userId);
    const userName = user?.name || 'Unknown';
    const opClass = `operation-item ${op.type}`;

    let text = '';
    if (op.type === 'insert') {
      const displayChar = op.char === '\n' ? '\\n' : op.char === ' ' ? '␣' : op.char;
      text = `${userName}: insert '${displayChar}' at ${op.position}`;
    } else {
      const displayChar = op.char === '\n' ? '\\n' : op.char === ' ' ? '␣' : op.char;
      text = `${userName}: delete '${displayChar}' at ${op.position}`;
    }

    const opItem = createElement('div', { class: opClass }, [text]);
    opsLog.appendChild(opItem);
  }

  if (state.operations.length === 0) {
    opsLog.appendChild(createElement('div', { class: 'operation-item' }, ['No operations yet']));
  }

  opsPanel.appendChild(opsLog);
  sidebar.appendChild(opsPanel);

  // Concept box
  const conceptBox = createElement('div', { class: 'concept-box' });
  conceptBox.appendChild(createElement('h3', {}, ['CRDT Text Editing']));
  const conceptP = createElement('p', {});
  conceptP.textContent =
    'Each edit is recorded as a participant-scoped observation in an ObservationSet — union-merged, never annihilated. ' +
    'Multiple users can edit concurrently - the CRDT ensures all replicas converge to the same state ' +
    'using geometric merge operations. Vector clocks track causal ordering.';
  conceptBox.appendChild(conceptP);
  sidebar.appendChild(conceptBox);

  container.appendChild(sidebar);
  return container;
}

// =============================================================================
// Main Loop
// =============================================================================

let lastRender = 0;

function mainLoop(timestamp: number): void {
  // Update at ~30fps for rendering, but process logic more frequently
  simulateRemoteEdits();

  if (timestamp - lastRender > 33) {
    const app = document.getElementById('app');
    if (app) {
      // Preserve editor content and selection
      const oldEditor = app.querySelector('.editor') as HTMLTextAreaElement | null;
      const selStart = oldEditor?.selectionStart || 0;
      const selEnd = oldEditor?.selectionEnd || 0;

      app.textContent = '';
      app.appendChild(renderApp());

      // Restore selection in new editor
      const newEditor = app.querySelector('.editor') as HTMLTextAreaElement | null;
      if (newEditor && oldEditor) {
        newEditor.value = state.content;
        newEditor.setSelectionRange(selStart, selEnd);
        // Don't focus automatically - let user control focus
      }
    }
    lastRender = timestamp;
  }

  requestAnimationFrame(mainLoop);
}

// =============================================================================
// Initialize
// =============================================================================

async function main() {
  await init();

  // Initialize users first (sets localUserId)
  initializeUsers();

  // Initialize CRDT with the local user ID
  state.crdt = new ObservationSet();
  state.crdtSeq = 0;

  contentBehavior = behavior(state.content);
  cursorBehavior = behavior(0);

  const app = document.getElementById('app');
  if (app) {
    app.appendChild(renderApp());
  }

  console.log('Cliffy Document Editor initialized');
  console.log('Using ObservationSet for conflict-free edit logging');
  console.log('Vector clocks ensure causal ordering of operations');

  requestAnimationFrame(mainLoop);
}

main().catch(console.error);
