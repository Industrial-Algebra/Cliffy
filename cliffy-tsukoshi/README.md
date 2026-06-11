# cliffy-tsukoshi

Minimal geometric state management for JavaScript/TypeScript. Pure TypeScript, zero dependencies, works everywhere including React Native.

## Features

- **GeometricState** - Smooth state interpolation via `.blend()`
- **Rotor** - Rotation representation with SLERP interpolation
- **Transform** - Combined rotation + translation
- **Distributed Protocols** - CRDT, vector clocks, sync, consensus
- **Zero dependencies** - Pure TypeScript math
- **Universal** - Works in browser, Node.js, React Native, Deno

## Installation

```bash
npm install cliffy-tsukoshi
```

## Quick Start

```typescript
import { GeometricState, Rotor, Transform } from 'cliffy-tsukoshi';

// Create position state
const position = GeometricState.fromVector(100, 200, 0);

// Smooth interpolation (the key feature!)
const target = GeometricState.fromVector(300, 400, 0);
const smoothed = position.blend(target, 0.15); // Move 15% toward target

// Apply rotation (45 degrees in XY plane)
const rotation = Rotor.xy(Math.PI / 4);
const rotated = position.applyRotor(rotation);

// Full transform (rotation + translation)
const transform = Transform.new(
  Rotor.xz(Math.PI / 6),
  { x: 10, y: 20, z: 0 }
);
const transformed = position.applyTransform(transform);

// Extract values
const [x, y, z] = smoothed.asVector();
console.log(`Position: (${x}, ${y}, ${z})`);
```

## Core Concepts

### GeometricState

The main container for position/state data. Internally uses GA3 (3D Geometric Algebra) but exposes a simple vector interface.

```typescript
// Create states
const pos = GeometricState.fromVector(x, y, z);
const pos2D = GeometricState.fromVector2D(x, y);
const value = GeometricState.fromScalar(42);

// Smooth interpolation - the killer feature
// In a game loop: position = position.blend(target, 0.1)
const smoothed = current.blend(target, t);

// Arithmetic
const sum = a.add(b);
const diff = a.sub(b);
const scaled = a.scale(2);

// Extract values
const [x, y, z] = state.asVector();
const { x, y, z } = state.asVectorObject();
const scalar = state.asScalar();
```

### Rotor (Rotations)

Rotors represent rotations without gimbal lock. They support smooth interpolation via SLERP.

```typescript
// Create rotations
const r1 = Rotor.xy(angle);  // Around Z axis
const r2 = Rotor.xz(angle);  // Around Y axis
const r3 = Rotor.yz(angle);  // Around X axis
const r4 = Rotor.fromAxisAngle(ax, ay, az, angle);

// Apply to state
const rotated = position.applyRotor(rotation);

// Compose rotations
const combined = r1.then(r2);  // r1 first, then r2

// Interpolate (SLERP)
const halfway = r1.slerpTo(r2, 0.5);
```

### Transform (Rotation + Translation)

Combines rotation and translation into a single operation.

```typescript
// Create transforms
const t = Transform.new(rotor, { x: 10, y: 20, z: 0 });
const tRotate = Transform.rotation(rotor);
const tMove = Transform.fromTranslation({ x: 10, y: 0, z: 0 });

// Apply
const transformed = position.applyTransform(t);

// Compose
const combined = t1.then(t2);

// Interpolate
const halfway = t1.interpolateTo(t2, 0.5);
```

### ReactiveState

A mutable container with subscriptions for reactive updates.

```typescript
import { reactiveState, GeometricState } from 'cliffy-tsukoshi';

const position = reactiveState(GeometricState.fromVector(0, 0, 0));

// Subscribe to changes
position.subscribe((state) => {
  const [x, y, z] = state.asVector();
  updateUI(x, y, z);
});

// Update (triggers subscribers)
position.set(GeometricState.fromVector(100, 200, 0));

// Smooth update
position.blendTo(target, 0.1);
```

## Common Patterns

### Smooth Following (Game Loop)

```typescript
function gameLoop() {
  // Smooth follow: move 10% toward target each frame
  playerPosition = playerPosition.blend(targetPosition, 0.1);

  // Smooth rotation
  playerRotation = playerRotation.slerpTo(targetRotation, 0.1);

  render();
  requestAnimationFrame(gameLoop);
}
```

### 2D Game Movement

```typescript
const position = GeometricState.fromVector2D(100, 100);
const velocity = GeometricState.fromVector2D(5, 0);

function update() {
  // Move
  position = position.add(velocity);

  // Rotate velocity (turn)
  const turn = Rotor.xy(turnAngle);
  velocity = velocity.applyRotor(turn);
}
```

### React Native Integration

```typescript
import { useRef, useEffect } from 'react';
import { Animated } from 'react-native';
import { GeometricState } from 'cliffy-tsukoshi';

function useGeometricAnimation(target: GeometricState) {
  const position = useRef(GeometricState.zero());
  const animX = useRef(new Animated.Value(0)).current;
  const animY = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    const interval = setInterval(() => {
      position.current = position.current.blend(target, 0.1);
      const [x, y] = position.current.asVector2D();
      animX.setValue(x);
      animY.setValue(y);
    }, 16);

    return () => clearInterval(interval);
  }, [target]);

  return { x: animX, y: animY };
}
```

## More Examples

### Camera Following with Damping

```typescript
class SmoothCamera {
  position: GeometricState;
  zoom: GeometricState;

  constructor() {
    this.position = GeometricState.fromVector2D(0, 0);
    this.zoom = GeometricState.fromScalar(1);
  }

  follow(target: GeometricState, deltaTime: number) {
    // Smooth follow with frame-rate independent damping
    const smoothing = 1 - Math.pow(0.001, deltaTime);
    this.position = this.position.blend(target, smoothing);
  }

  zoomTo(level: number, speed: number = 0.1) {
    const targetZoom = GeometricState.fromScalar(level);
    this.zoom = this.zoom.blend(targetZoom, speed);
  }

  getViewMatrix(): { x: number; y: number; scale: number } {
    const [x, y] = this.position.asVector2D();
    return { x: -x, y: -y, scale: this.zoom.asScalar() };
  }
}
```

### Touch/Joystick Input

```typescript
class JoystickController {
  private input = GeometricState.zero();
  private smoothedInput = GeometricState.zero();

  // Called on touch/mouse move
  setInput(dx: number, dy: number) {
    // Clamp to unit circle
    const mag = Math.sqrt(dx * dx + dy * dy);
    if (mag > 1) {
      dx /= mag;
      dy /= mag;
    }
    this.input = GeometricState.fromVector2D(dx, dy);
  }

  // Called on touch/mouse up
  release() {
    this.input = GeometricState.zero();
  }

  // Called every frame
  update(): [number, number] {
    // Smooth the input to avoid jerky movement
    this.smoothedInput = this.smoothedInput.blend(this.input, 0.2);
    return this.smoothedInput.asVector2D();
  }
}
```

### Particle System

```typescript
interface Particle {
  position: GeometricState;
  velocity: GeometricState;
  life: number;
}

class ParticleEmitter {
  particles: Particle[] = [];
  origin: GeometricState;

  constructor(x: number, y: number) {
    this.origin = GeometricState.fromVector2D(x, y);
  }

  emit(count: number) {
    for (let i = 0; i < count; i++) {
      const angle = Math.random() * Math.PI * 2;
      const speed = 50 + Math.random() * 100;

      this.particles.push({
        position: this.origin.clone(),
        velocity: GeometricState.fromVector2D(
          Math.cos(angle) * speed,
          Math.sin(angle) * speed
        ),
        life: 1.0,
      });
    }
  }

  update(dt: number) {
    const gravity = GeometricState.fromVector2D(0, 200);
    const drag = 0.98;

    this.particles = this.particles.filter(p => {
      // Apply physics
      p.velocity = p.velocity.add(gravity.scale(dt)).scale(drag);
      p.position = p.position.add(p.velocity.scale(dt));
      p.life -= dt;

      return p.life > 0;
    });
  }
}
```

### Animated Sprite with Rotation

```typescript
class AnimatedSprite {
  position: GeometricState;
  rotation: Rotor;
  targetRotation: Rotor;

  constructor(x: number, y: number) {
    this.position = GeometricState.fromVector2D(x, y);
    this.rotation = Rotor.identity();
    this.targetRotation = Rotor.identity();
  }

  lookAt(targetX: number, targetY: number) {
    const [x, y] = this.position.asVector2D();
    const angle = Math.atan2(targetY - y, targetX - x);
    this.targetRotation = Rotor.xy(angle);
  }

  update() {
    // Smooth rotation toward target
    this.rotation = this.rotation.slerpTo(this.targetRotation, 0.15);
  }

  getAngle(): number {
    return this.rotation.angle();
  }
}
```

### Physics Body with Velocity Damping

```typescript
class PhysicsBody {
  position: GeometricState;
  velocity: GeometricState;

  readonly mass: number;
  readonly drag: number;

  constructor(x: number, y: number, mass = 1, drag = 0.02) {
    this.position = GeometricState.fromVector2D(x, y);
    this.velocity = GeometricState.zero();
    this.mass = mass;
    this.drag = drag;
  }

  applyForce(fx: number, fy: number) {
    const acceleration = GeometricState.fromVector2D(fx / this.mass, fy / this.mass);
    this.velocity = this.velocity.add(acceleration);
  }

  applyImpulse(ix: number, iy: number) {
    this.velocity = this.velocity.add(GeometricState.fromVector2D(ix, iy));
  }

  update(dt: number) {
    // Apply drag
    this.velocity = this.velocity.scale(1 - this.drag);

    // Update position
    this.position = this.position.add(this.velocity.scale(dt));

    // Stop if very slow
    if (this.velocity.magnitude() < 0.01) {
      this.velocity = GeometricState.zero();
    }
  }
}
```

### Path Following

```typescript
class PathFollower {
  private waypoints: GeometricState[];
  private currentIndex = 0;
  private position: GeometricState;

  constructor(waypoints: Array<[number, number]>) {
    this.waypoints = waypoints.map(([x, y]) => GeometricState.fromVector2D(x, y));
    this.position = this.waypoints[0].clone();
  }

  update(speed: number = 0.05): [number, number] {
    if (this.waypoints.length === 0) {
      return this.position.asVector2D();
    }

    const target = this.waypoints[this.currentIndex];
    this.position = this.position.blend(target, speed);

    // Check if reached waypoint
    if (this.position.distance(target) < 1) {
      this.currentIndex = (this.currentIndex + 1) % this.waypoints.length;
    }

    return this.position.asVector2D();
  }

  getProgress(): number {
    return this.currentIndex / this.waypoints.length;
  }
}
```

### UI Progress/Health Bar

```typescript
class SmoothProgressBar {
  private displayValue: GeometricState;
  private actualValue: number;

  constructor(initial: number = 1) {
    this.actualValue = initial;
    this.displayValue = GeometricState.fromScalar(initial);
  }

  setValue(value: number) {
    this.actualValue = Math.max(0, Math.min(1, value));
  }

  update(): number {
    const target = GeometricState.fromScalar(this.actualValue);
    this.displayValue = this.displayValue.blend(target, 0.1);
    return this.displayValue.asScalar();
  }

  // For damage flash effects
  flash(amount: number) {
    // Instantly show damage, then smooth back
    this.displayValue = GeometricState.fromScalar(
      this.displayValue.asScalar() - amount
    );
  }
}
```

### Pan and Zoom Controls

```typescript
class PanZoomController {
  offset: GeometricState;
  zoom: GeometricState;

  private targetOffset: GeometricState;
  private targetZoom: GeometricState;

  constructor() {
    this.offset = GeometricState.zero();
    this.targetOffset = GeometricState.zero();
    this.zoom = GeometricState.fromScalar(1);
    this.targetZoom = GeometricState.fromScalar(1);
  }

  pan(dx: number, dy: number) {
    const scale = this.zoom.asScalar();
    this.targetOffset = this.targetOffset.add(
      GeometricState.fromVector2D(dx / scale, dy / scale)
    );
  }

  zoomAt(factor: number, centerX: number, centerY: number) {
    const currentZoom = this.targetZoom.asScalar();
    const newZoom = Math.max(0.1, Math.min(10, currentZoom * factor));
    this.targetZoom = GeometricState.fromScalar(newZoom);

    // Zoom toward cursor position
    const [ox, oy] = this.targetOffset.asVector2D();
    const zoomRatio = newZoom / currentZoom;
    this.targetOffset = GeometricState.fromVector2D(
      centerX - (centerX - ox) * zoomRatio,
      centerY - (centerY - oy) * zoomRatio
    );
  }

  update() {
    this.offset = this.offset.blend(this.targetOffset, 0.15);
    this.zoom = this.zoom.blend(this.targetZoom, 0.15);
  }

  screenToWorld(screenX: number, screenY: number): [number, number] {
    const [ox, oy] = this.offset.asVector2D();
    const scale = this.zoom.asScalar();
    return [(screenX - ox) / scale, (screenY - oy) / scale];
  }
}
```

### React Hook for Smooth Values

```typescript
import { useState, useEffect, useRef } from 'react';
import { GeometricState } from 'cliffy-tsukoshi';

function useSmoothValue(target: number, smoothing: number = 0.1): number {
  const [display, setDisplay] = useState(target);
  const state = useRef(GeometricState.fromScalar(target));

  useEffect(() => {
    let animationId: number;
    const targetState = GeometricState.fromScalar(target);

    function animate() {
      state.current = state.current.blend(targetState, smoothing);
      setDisplay(state.current.asScalar());

      if (Math.abs(state.current.asScalar() - target) > 0.001) {
        animationId = requestAnimationFrame(animate);
      }
    }

    animationId = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animationId);
  }, [target, smoothing]);

  return display;
}

// Usage
function Counter({ value }: { value: number }) {
  const smoothValue = useSmoothValue(value);
  return <span>{Math.round(smoothValue)}</span>;
}
```

### Multiplayer State Interpolation

```typescript
interface NetworkState {
  position: GeometricState;
  timestamp: number;
}

class InterpolatedPlayer {
  private buffer: NetworkState[] = [];
  private renderPosition: GeometricState;

  // Render 100ms behind to allow interpolation
  private readonly INTERPOLATION_DELAY = 100;

  constructor(initialX: number, initialY: number) {
    this.renderPosition = GeometricState.fromVector2D(initialX, initialY);
  }

  // Called when network update arrives
  receiveState(x: number, y: number, serverTime: number) {
    this.buffer.push({
      position: GeometricState.fromVector2D(x, y),
      timestamp: serverTime,
    });

    // Keep only recent states
    const cutoff = serverTime - 1000;
    this.buffer = this.buffer.filter(s => s.timestamp > cutoff);
  }

  // Called every frame
  update(currentTime: number): [number, number] {
    const renderTime = currentTime - this.INTERPOLATION_DELAY;

    // Find states to interpolate between
    let before: NetworkState | null = null;
    let after: NetworkState | null = null;

    for (const state of this.buffer) {
      if (state.timestamp <= renderTime) {
        before = state;
      } else if (!after) {
        after = state;
      }
    }

    if (before && after) {
      // Interpolate between the two states
      const t = (renderTime - before.timestamp) / (after.timestamp - before.timestamp);
      this.renderPosition = before.position.blend(after.position, t);
    } else if (before) {
      // Extrapolate slightly
      this.renderPosition = this.renderPosition.blend(before.position, 0.1);
    }

    return this.renderPosition.asVector2D();
  }
}
```

## Why Geometric Algebra?

Under the hood, cliffy-tsukoshi uses GA3 (Clifford Algebra Cl(3,0)). This provides:

1. **Unified representation** - Scalars, vectors, rotations all in one type
2. **No gimbal lock** - Rotors avoid quaternion edge cases
3. **Composable** - Rotations compose correctly via geometric product
4. **Interpolation** - SLERP "just works" for smooth rotation blending

You don't need to understand GA to use the library - the API is designed around familiar vector operations.

## API Reference

### GeometricState

| Method | Description |
|--------|-------------|
| `fromVector(x, y, z)` | Create from 3D coordinates |
| `fromVector2D(x, y)` | Create from 2D coordinates |
| `fromScalar(n)` | Create from scalar value |
| `blend(target, t)` | Interpolate toward target |
| `add(other)` | Vector addition |
| `sub(other)` | Vector subtraction |
| `scale(factor)` | Scalar multiplication |
| `applyRotor(r)` | Apply rotation |
| `applyTransform(t)` | Apply rotation + translation |
| `asVector()` | Extract as [x, y, z] |
| `asVector2D()` | Extract as [x, y] |
| `asScalar()` | Extract scalar value |
| `distance(other)` | Euclidean distance |
| `magnitude()` | Vector length |

### Rotor

| Method | Description |
|--------|-------------|
| `xy(angle)` | Rotation in XY plane (around Z) |
| `xz(angle)` | Rotation in XZ plane (around Y) |
| `yz(angle)` | Rotation in YZ plane (around X) |
| `fromAxisAngle(ax, ay, az, angle)` | From axis-angle |
| `identity()` | No rotation |
| `transform(v)` | Apply to multivector |
| `then(other)` | Compose rotations |
| `inverse()` | Reverse rotation |
| `slerp(t)` | Interpolate from identity |
| `slerpTo(other, t)` | Interpolate to other |
| `angle()` | Get rotation angle |

### Transform

| Method | Description |
|--------|-------------|
| `new(rotor, translation)` | Create from components |
| `identity()` | No transformation |
| `rotation(rotor)` | Pure rotation |
| `fromTranslation(t)` | Pure translation |
| `apply(v)` | Apply to multivector |
| `then(other)` | Compose transforms |
| `inverse()` | Reverse transform |
| `interpolateTo(other, t)` | Interpolate to other |

---

## Distributed Protocols

cliffy-tsukoshi includes a complete suite of distributed protocols for building collaborative applications. Import from `cliffy-tsukoshi/protocols` or directly from the main module.

```typescript
import { GeometricCRDT, VectorClock, SyncState } from 'cliffy-tsukoshi';
// or
import { GeometricCRDT, VectorClock } from 'cliffy-tsukoshi/protocols';
```

### VectorClock

Track causality in distributed systems without centralized coordination.

```typescript
import { VectorClock } from 'cliffy-tsukoshi';

const clock1 = new VectorClock();
const clock2 = new VectorClock();

// Each node increments its own entry
clock1.tick('node-1');
clock2.tick('node-2');

// These events are concurrent (neither happened before the other)
console.log(clock1.concurrent(clock2)); // true

// After syncing, merge clocks
clock1.update(clock2);
clock1.tick('node-1');

// Now clock1 happened after clock2
console.log(clock2.happensBefore(clock1)); // true
```

### GeometricCRDT

Conflict-free replicated data type using geometric algebra for merge operations.

```typescript
import { GeometricCRDT, OperationType, scalar } from 'cliffy-tsukoshi';

// Create CRDTs on two different nodes
const nodeId1 = crypto.randomUUID();
const nodeId2 = crypto.randomUUID();

const crdt1 = new GeometricCRDT(nodeId1, scalar(0));
const crdt2 = new GeometricCRDT(nodeId2, scalar(0));

// Each node makes independent updates
const op1 = crdt1.createOperation(scalar(5), OperationType.Addition);
crdt1.applyOperation(op1);

const op2 = crdt2.createOperation(scalar(3), OperationType.Addition);
crdt2.applyOperation(op2);

// Merge - order doesn't matter, result is always consistent
const merged1 = crdt1.merge(crdt2);
const merged2 = crdt2.merge(crdt1);

// Both produce the same state (8.0)
console.log(merged1.state[0] === merged2.state[0]); // true
```

### Lattice Operations

Join-semilattice operations for guaranteed convergence.

```typescript
import { GA3Lattice, ComponentLattice, latticeJoin, latticeMeet } from 'cliffy-tsukoshi';

// Magnitude-based lattice (larger magnitude wins)
const stateA = GA3Lattice.fromScalar(3);
const stateB = GA3Lattice.fromScalar(7);

const joined = stateA.join(stateB);
console.log(joined.dominates(stateA)); // true
console.log(joined.dominates(stateB)); // true

// Component-wise lattice (max of each coefficient)
const a = ComponentLattice.fromScalar(5);
const b = ComponentLattice.fromScalar(3);

const max = a.join(b);  // Takes max of each component
const min = a.meet(b);  // Takes min of each component
```

### Delta Synchronization

Efficient state sync using minimal deltas instead of full state transfer.

```typescript
import {
  computeDelta,
  applyDelta,
  additiveDelta,
  DeltaBatch,
  VectorClock
} from 'cliffy-tsukoshi';
import { scalar } from 'cliffy-tsukoshi';

// Compute delta between states
const from = scalar(10);
const to = scalar(25);
const delta = computeDelta(from, to); // Results in scalar(15)

// Create a delta with causal metadata
const fromClock = new VectorClock();
const toClock = new VectorClock();
toClock.tick('node-1');

const stateDelta = additiveDelta(delta, fromClock, toClock, 'node-1');

// Apply delta to reconstruct state
const newState = applyDelta(from, stateDelta);
console.log(newState[0]); // 25

// Batch multiple deltas for efficiency
const batch = new DeltaBatch();
batch.push(stateDelta);
// ... add more deltas
const finalState = batch.applyTo(from);
```

### Storage & Recovery

Persist state with snapshots and operation logs.

```typescript
import { MemoryStore, recoverState, VectorClock, additiveDelta } from 'cliffy-tsukoshi';
import { scalar } from 'cliffy-tsukoshi';

// Create a store (in-memory for this example)
const store = new MemoryStore({
  maxSnapshots: 10,
  maxOperationsBeforeCompact: 100,
  autoCompact: true,
});

// Save initial snapshot
const state = scalar(100);
const clock = new VectorClock();
store.saveSnapshot(state, clock);

// Append operations
clock.tick('node-1');
const delta = additiveDelta(scalar(50), new VectorClock(), clock, 'node-1');
store.appendOperation(delta);

// Later: recover state from storage
const result = recoverState(store);
if (result) {
  console.log(result.state[0]); // 150
  console.log(result.operationsReplayed); // 1
}

// Check stats
const stats = store.stats();
console.log(`Snapshots: ${stats.snapshotCount}, Operations: ${stats.operationCount}`);
```

### Peer-to-Peer Sync

Protocol messages for P2P state synchronization.

```typescript
import { SyncState, PeerConnectionState, VectorClock } from 'cliffy-tsukoshi';

// Create sync state for this node
const nodeId = crypto.randomUUID();
const syncState = new SyncState(nodeId, {
  heartbeatInterval: 5000,
  peerTimeout: 30000,
});

// When a peer connects
const peerId = crypto.randomUUID();
syncState.registerPeer(peerId, new VectorClock());

// Create protocol messages
const hello = syncState.createHello('My App Node');
const heartbeat = syncState.createHeartbeat();
const deltaRequest = syncState.createDeltaRequest(new VectorClock());

// Send messages via your transport (WebRTC, WebSocket, etc.)
sendToPeer(peerId, JSON.stringify(hello));

// Handle incoming messages
function onMessageReceived(data: string) {
  const message = JSON.parse(data);
  const response = syncState.handleMessage(message);
  if (response) {
    sendToPeer(message.sender, JSON.stringify(response));
  }
}

// Maintenance: check for stale peers
const stalePeers = syncState.stalePeers();
for (const peerId of stalePeers) {
  // Attempt reconnection or remove
}
```

### Distributed Consensus

Geometric mean consensus for distributed agreement.

```typescript
import { GeometricConsensus, scalar } from 'cliffy-tsukoshi';

const nodeId = crypto.randomUUID();
const consensus = new GeometricConsensus(nodeId, scalar(0));

// Subscribe to outgoing messages
consensus.onMessage((message) => {
  // Broadcast to all peers
  broadcastToPeers(JSON.stringify(message));
});

// Propose a value
const round = consensus.propose(scalar(42));

// Receive proposals from other nodes
consensus.receiveProposal('other-node-id', scalar(38), round);
consensus.receiveProposal('another-node-id', scalar(45), round);

// Compute consensus from all proposals
const proposals = consensus.getProposals(round);
const consensusValue = consensus.geometricConsensus(proposals, 0.1);
console.log('Consensus:', consensusValue[0]); // ~41.67 (geometric mean)

// Vote on the consensus value
consensus.vote(round, true, consensusValue);

// Try to commit if we have majority
const committed = consensus.tryCommit(round, 3); // 3 participants
if (committed) {
  console.log('Round committed!', committed[0]);
}
```

### Complete Example: Collaborative Counter

```typescript
import {
  GeometricCRDT,
  OperationType,
  SyncState,
  VectorClock,
  MemoryStore,
  additiveDelta,
  scalar,
} from 'cliffy-tsukoshi';

class CollaborativeCounter {
  private crdt: GeometricCRDT;
  private syncState: SyncState;
  private store: MemoryStore;
  private onUpdate: (value: number) => void;

  constructor(nodeId: string, onUpdate: (value: number) => void) {
    this.crdt = new GeometricCRDT(nodeId, scalar(0));
    this.syncState = new SyncState(nodeId);
    this.store = new MemoryStore();
    this.onUpdate = onUpdate;

    // Save initial state
    this.store.saveSnapshot(this.crdt.state, this.crdt.vectorClock);
  }

  increment(amount: number = 1): void {
    const op = this.crdt.createOperation(scalar(amount), OperationType.Addition);
    this.crdt.applyOperation(op);
    this.notifyUpdate();

    // Store the operation
    const delta = additiveDelta(
      scalar(amount),
      new VectorClock(),
      this.crdt.vectorClock,
      this.crdt.nodeId
    );
    this.store.appendOperation(delta);
  }

  getValue(): number {
    return this.crdt.state[0];
  }

  // Call when receiving state from another peer
  merge(otherCrdt: GeometricCRDT): void {
    this.crdt = this.crdt.merge(otherCrdt);
    this.notifyUpdate();
  }

  // Get state to send to peers
  getState(): GeometricCRDT {
    return this.crdt;
  }

  private notifyUpdate(): void {
    this.onUpdate(this.getValue());
  }
}

// Usage
const counter = new CollaborativeCounter('node-1', (value) => {
  console.log('Counter updated:', value);
});

counter.increment(5);
counter.increment(3);
console.log(counter.getValue()); // 8
```

## Protocol Reference

### VectorClock

| Method | Description |
|--------|-------------|
| `tick(nodeId)` | Increment clock for a node |
| `update(other)` | Merge with another clock |
| `happensBefore(other)` | Check causal ordering |
| `concurrent(other)` | Check if concurrent events |
| `merge(other)` | Create merged clock |
| `clone()` | Copy the clock |
| `toJSON()` / `fromJSON()` | Serialization |

### GeometricCRDT

| Method | Description |
|--------|-------------|
| `createOperation(transform, type)` | Create a new operation |
| `applyOperation(op)` | Apply an operation |
| `merge(other)` | Merge with another CRDT |
| `geometricJoin(other)` | Conflict resolution via magnitude |
| `toJSON()` / `fromJSON()` | Serialization |

### SyncState

| Method | Description |
|--------|-------------|
| `registerPeer(id, clock)` | Add a peer |
| `removePeer(id)` | Remove a peer |
| `createHello(name?)` | Create hello message |
| `createHeartbeat()` | Create heartbeat message |
| `createDeltaRequest(clock)` | Request deltas |
| `handleMessage(msg)` | Process incoming message |
| `stalePeers()` | Get list of stale peers |

### GeometricStore

| Method | Description |
|--------|-------------|
| `saveSnapshot(state, clock)` | Persist a snapshot |
| `loadLatestSnapshot()` | Load most recent snapshot |
| `appendOperation(delta)` | Add operation to log |
| `operationsSince(clock)` | Get operations after clock |
| `compact()` | Create snapshot from operations |
| `stats()` | Get storage statistics |
| `clear()` | Remove all data |

---

## License

MIT

## Part of Cliffy

cliffy-tsukoshi is the pure TypeScript extraction of geometric state management from [Cliffy](https://github.com/Industrial-Algebra/Cliffy), a framework for building collaborative applications using geometric algebra.
