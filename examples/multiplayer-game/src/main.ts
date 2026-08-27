/**
 * Cliffy Multiplayer Game Demo
 *
 * Demonstrates:
 * - Player positions as GeometricState
 * - Smooth interpolation using .blend() (SLERP/LERP)
 * - Simulated network latency and state sync
 * - Collision detection via geometric distance
 * - High-frequency state updates with FRP Behaviors
 */

import init, {
  behavior,
  GeometricState,
  Rotor,
  generateNodeId,
} from '@industrialalgebra/cliffy-core';

// =============================================================================
// Types
// =============================================================================

interface Player {
  id: string;
  name: string;
  color: string;
  position: GeometricState;
  targetPosition: GeometricState;
  rotation: number;
  isLocal: boolean;
  lastUpdate: number;
  velocity: { x: number; y: number };
}

interface GameState {
  players: Map<string, Player>;
  localPlayerId: string;
  canvasWidth: number;
  canvasHeight: number;
  simulatedLatency: number;
  interpolationFactor: number;
  showTrails: boolean;
  trails: Array<{ x: number; y: number; color: string; age: number }>;
}

// =============================================================================
// Game State
// =============================================================================

const state: GameState = {
  players: new Map(),
  localPlayerId: '',
  canvasWidth: 600,
  canvasHeight: 450,
  simulatedLatency: 50,
  interpolationFactor: 0.15,
  showTrails: true,
  trails: [],
};

// FRP Behaviors for reactive updates (initialized after WASM init)
let fpsCounter: ReturnType<typeof behavior<number>>;
let frameCount = 0;
let lastFpsUpdate = Date.now();

// Player colors
const PLAYER_COLORS = ['#ff6b6b', '#4ecdc4', '#ffe66d', '#a855f7'];

// =============================================================================
// Player Management
// =============================================================================

function createPlayer(id: string, name: string, isLocal: boolean, colorIndex: number): Player {
  const x = 100 + Math.random() * (state.canvasWidth - 200);
  const y = 100 + Math.random() * (state.canvasHeight - 200);

  return {
    id,
    name,
    color: PLAYER_COLORS[colorIndex % PLAYER_COLORS.length],
    position: GeometricState.fromVector(x, y, 0),
    targetPosition: GeometricState.fromVector(x, y, 0),
    rotation: 0,
    isLocal,
    lastUpdate: Date.now(),
    velocity: { x: 0, y: 0 },
  };
}

function initializePlayers(): void {
  state.players.clear();

  // Create local player
  state.localPlayerId = generateNodeId();
  const localPlayer = createPlayer(state.localPlayerId, 'You', true, 0);
  state.players.set(state.localPlayerId, localPlayer);

  // Create simulated remote players
  for (let i = 1; i <= 3; i++) {
    const id = generateNodeId();
    const player = createPlayer(id, `Player ${i + 1}`, false, i);
    state.players.set(id, player);
  }
}

// =============================================================================
// Movement & Physics
// =============================================================================

function moveLocalPlayer(dx: number, dy: number): void {
  const player = state.players.get(state.localPlayerId);
  if (!player) return;

  const currentPos = player.position.asVector();
  let newX = currentPos[0] + dx;
  let newY = currentPos[1] + dy;

  // Boundary collision
  newX = Math.max(15, Math.min(state.canvasWidth - 15, newX));
  newY = Math.max(15, Math.min(state.canvasHeight - 15, newY));

  // Player collision detection using geometric distance
  for (const [id, other] of state.players) {
    if (id === state.localPlayerId) continue;

    const otherPos = other.position.asVector();
    const distX = newX - otherPos[0];
    const distY = newY - otherPos[1];
    const distance = Math.sqrt(distX * distX + distY * distY);

    if (distance < 30) {
      // Push back
      const pushX = (distX / distance) * (30 - distance);
      const pushY = (distY / distance) * (30 - distance);
      newX += pushX;
      newY += pushY;
    }
  }

  // Update target position (simulates network update)
  player.targetPosition = GeometricState.fromVector(newX, newY, 0);
  player.velocity = { x: dx, y: dy };
  player.lastUpdate = Date.now();

  // Calculate rotation based on movement direction
  if (dx !== 0 || dy !== 0) {
    player.rotation = Math.atan2(dy, dx);
  }
}

function simulateRemotePlayers(): void {
  for (const [id, player] of state.players) {
    if (player.isLocal) continue;

    // Random movement for AI players
    if (Math.random() < 0.02) {
      const angle = Math.random() * Math.PI * 2;
      const speed = 2 + Math.random() * 3;
      player.velocity = {
        x: Math.cos(angle) * speed,
        y: Math.sin(angle) * speed,
      };
      player.rotation = angle;
    }

    // Apply velocity with boundary checking
    const currentPos = player.targetPosition.asVector();
    let newX = currentPos[0] + player.velocity.x;
    let newY = currentPos[1] + player.velocity.y;

    // Bounce off walls
    if (newX < 15 || newX > state.canvasWidth - 15) {
      player.velocity.x *= -1;
      newX = Math.max(15, Math.min(state.canvasWidth - 15, newX));
    }
    if (newY < 15 || newY > state.canvasHeight - 15) {
      player.velocity.y *= -1;
      newY = Math.max(15, Math.min(state.canvasHeight - 15, newY));
    }

    // Friction
    player.velocity.x *= 0.98;
    player.velocity.y *= 0.98;

    player.targetPosition = GeometricState.fromVector(newX, newY, 0);
    player.lastUpdate = Date.now();
  }
}

function interpolatePositions(): void {
  const t = state.interpolationFactor;

  for (const player of state.players.values()) {
    // Use .blend() for smooth interpolation
    player.position = player.position.blend(player.targetPosition, t);

    // Add trail
    if (state.showTrails) {
      const pos = player.position.asVector();
      const vel = player.velocity;
      if (Math.abs(vel.x) > 0.5 || Math.abs(vel.y) > 0.5) {
        state.trails.push({
          x: pos[0],
          y: pos[1],
          color: player.color,
          age: 0,
        });
      }
    }
  }

  // Age and remove old trails
  state.trails = state.trails
    .map((t) => ({ ...t, age: t.age + 1 }))
    .filter((t) => t.age < 20);
}

// =============================================================================
// Input Handling
// =============================================================================

const keysPressed = new Set<string>();

function setupInput(canvas: HTMLElement): void {
  document.addEventListener('keydown', (e) => {
    keysPressed.add(e.key.toLowerCase());
  });

  document.addEventListener('keyup', (e) => {
    keysPressed.delete(e.key.toLowerCase());
  });

  canvas.addEventListener('click', (e) => {
    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * state.canvasWidth;
    const y = ((e.clientY - rect.top) / rect.height) * state.canvasHeight;

    const player = state.players.get(state.localPlayerId);
    if (player) {
      player.targetPosition = GeometricState.fromVector(x, y, 0);
      const currentPos = player.position.asVector();
      player.rotation = Math.atan2(y - currentPos[1], x - currentPos[0]);
    }
  });
}

function processInput(): void {
  const speed = 5;
  let dx = 0;
  let dy = 0;

  if (keysPressed.has('w') || keysPressed.has('arrowup')) dy -= speed;
  if (keysPressed.has('s') || keysPressed.has('arrowdown')) dy += speed;
  if (keysPressed.has('a') || keysPressed.has('arrowleft')) dx -= speed;
  if (keysPressed.has('d') || keysPressed.has('arrowright')) dx += speed;

  if (dx !== 0 || dy !== 0) {
    // Normalize diagonal movement
    if (dx !== 0 && dy !== 0) {
      dx *= 0.707;
      dy *= 0.707;
    }
    moveLocalPlayer(dx, dy);
  }
}

// =============================================================================
// Rendering
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

function renderGame(): HTMLElement {
  const container = createElement('div', { class: 'game-container' });

  // Game area
  const gameArea = createElement('div', { class: 'game-area' });
  const canvas = createElement('div', { class: 'game-canvas' });
  canvas.id = 'game-canvas';

  // FPS counter
  const fps = createElement('div', { class: 'fps-counter' }, [`${fpsCounter.sample()} FPS`]);
  canvas.appendChild(fps);

  // Latency indicator
  const latencyClass =
    state.simulatedLatency < 50 ? 'good' : state.simulatedLatency < 100 ? 'medium' : 'bad';
  const latency = createElement('div', { class: `latency-indicator ${latencyClass}` }, [
    `${state.simulatedLatency}ms`,
  ]);
  canvas.appendChild(latency);

  // Render trails
  for (const trail of state.trails) {
    const opacity = 1 - trail.age / 20;
    const size = 10 - (trail.age / 20) * 6;
    const trailEl = createElement('div', { class: 'player-trail' });
    trailEl.style.left = `${(trail.x / state.canvasWidth) * 100}%`;
    trailEl.style.top = `${(trail.y / state.canvasHeight) * 100}%`;
    trailEl.style.backgroundColor = trail.color;
    trailEl.style.opacity = String(opacity * 0.5);
    trailEl.style.width = `${size}px`;
    trailEl.style.height = `${size}px`;
    canvas.appendChild(trailEl);
  }

  // Render players
  for (const player of state.players.values()) {
    const pos = player.position.asVector();
    const x = (pos[0] / state.canvasWidth) * 100;
    const y = (pos[1] / state.canvasHeight) * 100;

    const isMoving =
      Math.abs(player.velocity.x) > 0.5 || Math.abs(player.velocity.y) > 0.5;
    const playerEl = createElement(
      'div',
      { class: `player ${player.isLocal ? 'local' : ''} ${isMoving ? 'moving' : ''}` },
      [player.isLocal ? 'Y' : player.name.charAt(player.name.length - 1)]
    );
    playerEl.style.left = `${x}%`;
    playerEl.style.top = `${y}%`;
    playerEl.style.backgroundColor = player.color;
    playerEl.style.color = player.color;
    playerEl.style.transform = `translate(-50%, -50%) rotate(${player.rotation + Math.PI / 2}rad)`;
    canvas.appendChild(playerEl);
  }

  gameArea.appendChild(canvas);
  container.appendChild(gameArea);

  // Sidebar
  const sidebar = createElement('div', { class: 'sidebar' });

  // Players panel
  const playersPanel = createElement('div', { class: 'panel' });
  playersPanel.appendChild(createElement('h2', {}, ['Players']));
  const playerList = createElement('div', { class: 'player-list' });

  for (const player of state.players.values()) {
    const pos = player.position.asVector();
    const item = createElement('div', { class: 'player-item' });

    const dot = createElement('div', { class: 'player-dot' });
    dot.style.backgroundColor = player.color;
    item.appendChild(dot);

    const info = createElement('div', { class: 'player-info' });
    info.appendChild(
      createElement('div', { class: 'player-name' }, [
        player.name + (player.isLocal ? ' (You)' : ''),
      ])
    );
    info.appendChild(
      createElement('div', { class: 'player-pos' }, [
        `(${pos[0].toFixed(0)}, ${pos[1].toFixed(0)})`,
      ])
    );
    item.appendChild(info);
    playerList.appendChild(item);
  }

  playersPanel.appendChild(playerList);
  sidebar.appendChild(playersPanel);

  // Stats panel
  const statsPanel = createElement('div', { class: 'panel' });
  statsPanel.appendChild(createElement('h2', {}, ['Network Stats']));
  const statsGrid = createElement('div', { class: 'stats-grid' });

  const addStat = (value: string, label: string) => {
    const item = createElement('div', { class: 'stat-item' });
    item.appendChild(createElement('div', { class: 'stat-value' }, [value]));
    item.appendChild(createElement('div', { class: 'stat-label' }, [label]));
    statsGrid.appendChild(item);
  };

  addStat(`${state.simulatedLatency}ms`, 'Latency');
  addStat(`${(state.interpolationFactor * 100).toFixed(0)}%`, 'Interp');
  addStat(String(state.players.size), 'Players');
  addStat(String(state.trails.length), 'Trails');

  statsPanel.appendChild(statsGrid);
  sidebar.appendChild(statsPanel);

  // Controls panel
  const controlsPanel = createElement('div', { class: 'panel' });
  controlsPanel.appendChild(createElement('h2', {}, ['Settings']));
  const controls = createElement('div', { class: 'controls' });

  // Latency slider
  const latencyRow = createElement('div', { class: 'control-row' });
  latencyRow.appendChild(createElement('label', {}, ['Simulated Latency']));
  const latencySlider = createElement('input', {
    type: 'range',
    min: '0',
    max: '200',
    value: String(state.simulatedLatency),
  }) as HTMLInputElement;
  latencySlider.oninput = () => {
    state.simulatedLatency = parseInt(latencySlider.value);
  };
  latencyRow.appendChild(latencySlider);
  latencyRow.appendChild(
    createElement('span', { class: 'value' }, [`${state.simulatedLatency}ms`])
  );
  controls.appendChild(latencyRow);

  // Interpolation slider
  const interpRow = createElement('div', { class: 'control-row' });
  interpRow.appendChild(createElement('label', {}, ['Interpolation']));
  const interpSlider = createElement('input', {
    type: 'range',
    min: '1',
    max: '50',
    value: String(state.interpolationFactor * 100),
  }) as HTMLInputElement;
  interpSlider.oninput = () => {
    state.interpolationFactor = parseInt(interpSlider.value) / 100;
  };
  interpRow.appendChild(interpSlider);
  interpRow.appendChild(
    createElement('span', { class: 'value' }, [`${(state.interpolationFactor * 100).toFixed(0)}%`])
  );
  controls.appendChild(interpRow);

  controlsPanel.appendChild(controls);

  // Instructions
  const instructions = createElement('div', { class: 'instructions' });
  instructions.appendChild(document.createTextNode('Use '));
  instructions.appendChild(createElement('kbd', {}, ['WASD']));
  instructions.appendChild(document.createTextNode(' or '));
  instructions.appendChild(createElement('kbd', {}, ['Arrow keys']));
  instructions.appendChild(document.createTextNode(' to move. Click to teleport.'));
  controlsPanel.appendChild(instructions);

  sidebar.appendChild(controlsPanel);

  // Concept box
  const conceptBox = createElement('div', { class: 'concept-box' });
  conceptBox.appendChild(createElement('h3', {}, ['Geometric Interpolation']));
  const conceptP = createElement('p', {});
  conceptP.textContent =
    'Player positions use GeometricState.blend() for smooth interpolation. ' +
    "This uses LERP for positions and SLERP for rotations, ensuring smooth movement even with network latency. " +
    "The .blend(target, t) method creates natural motion without jitter.";
  conceptBox.appendChild(conceptP);
  sidebar.appendChild(conceptBox);

  container.appendChild(sidebar);
  return container;
}

// =============================================================================
// Game Loop
// =============================================================================

let lastTime = 0;

function gameLoop(currentTime: number): void {
  const deltaTime = currentTime - lastTime;
  lastTime = currentTime;

  // Update FPS counter
  frameCount++;
  if (Date.now() - lastFpsUpdate > 1000) {
    fpsCounter.set(frameCount);
    frameCount = 0;
    lastFpsUpdate = Date.now();
  }

  // Process input and update game state
  processInput();
  simulateRemotePlayers();
  interpolatePositions();

  // Render
  const app = document.getElementById('app');
  if (app) {
    app.textContent = '';
    app.appendChild(renderGame());

    // Re-setup input for new canvas
    const canvas = document.getElementById('game-canvas');
    if (canvas) {
      setupInput(canvas);
    }
  }

  requestAnimationFrame(gameLoop);
}

// =============================================================================
// Initialize
// =============================================================================

async function main() {
  await init();

  // Initialize FRP Behaviors after WASM is ready
  fpsCounter = behavior(60);

  initializePlayers();

  const app = document.getElementById('app');
  if (app) {
    app.appendChild(renderGame());
    const canvas = document.getElementById('game-canvas');
    if (canvas) {
      setupInput(canvas);
    }
  }

  console.log('Cliffy Multiplayer Game initialized');
  console.log('Using GeometricState.blend() for smooth interpolation');

  requestAnimationFrame(gameLoop);
}

main().catch(console.error);
