/**
 * cliffy-tsukoshi
 *
 * Minimal geometric state management library.
 * Pure TypeScript, zero dependencies, works everywhere.
 *
 * @example
 * ```typescript
 * import { GeometricState, Rotor, Transform } from 'cliffy-tsukoshi';
 *
 * // Create position state
 * const position = GeometricState.fromVector(100, 200, 0);
 *
 * // Smooth interpolation
 * const target = GeometricState.fromVector(300, 400, 0);
 * const smoothed = position.blend(target, 0.15);
 *
 * // Apply rotation
 * const rotation = Rotor.xy(Math.PI / 4);
 * const rotated = position.applyRotor(rotation);
 *
 * // Extract values
 * const [x, y, z] = smoothed.asVector();
 * ```
 *
 * @packageDocumentation
 */

// ==========================================================================
// Core Types
// ==========================================================================

export type { GA3 } from './ga3.js';

// ==========================================================================
// GA3 Functions (low-level, for advanced use)
// ==========================================================================

export {
  // Constants
  GA3_SCALAR,
  GA3_E1,
  GA3_E2,
  GA3_E12,
  GA3_E3,
  GA3_E13,
  GA3_E23,
  GA3_E123,
  // Constructors
  zero,
  scalar,
  vector,
  bivector,
  fromCoefficients,
  clone,
  // Operations
  add,
  sub,
  scale,
  negate,
  geometricProduct,
  reverse,
  magnitude,
  normalize,
  sandwich,
  lerp,
  // Extraction
  asVector,
  asBivector,
  equals,
} from './ga3.js';

// ==========================================================================
// Rotor (Rotations)
// ==========================================================================

export { Rotor } from './rotor.js';

// ==========================================================================
// Transform (Rotation + Translation)
// ==========================================================================

export {
  Transform,
  translation,
  zeroTranslation,
  addTranslation,
  subTranslation,
  scaleTranslation,
  negateTranslation,
  lerpTranslation,
  translationToVector,
} from './transform.js';

export type { Translation } from './transform.js';

// ==========================================================================
// GeometricState (Main API)
// ==========================================================================

export {
  GeometricState,
  ReactiveState,
  reactiveState,
} from './state.js';

export type { StateSubscriber, Subscription } from './state.js';

// ==========================================================================
// Convenience re-exports
// ==========================================================================

/**
 * Create a 2D position state.
 */
export function position2D(x: number, y: number): import('./state.js').GeometricState {
  return import('./state.js').then(m => m.GeometricState.fromVector2D(x, y)) as any;
}

/**
 * Create a 3D position state.
 */
export function position3D(x: number, y: number, z: number): import('./state.js').GeometricState {
  return import('./state.js').then(m => m.GeometricState.fromVector(x, y, z)) as any;
}

// Synchronous versions using direct imports
import { GeometricState } from './state.js';

/**
 * Create a 2D position state (synchronous).
 */
export function pos2(x: number, y: number): GeometricState {
  return GeometricState.fromVector2D(x, y);
}

/**
 * Create a 3D position state (synchronous).
 */
export function pos3(x: number, y: number, z: number): GeometricState {
  return GeometricState.fromVector(x, y, z);
}

/**
 * Create a scalar state.
 */
export function scalarState(value: number): GeometricState {
  return GeometricState.fromScalar(value);
}

// ==========================================================================
// Protocols (also available via 'cliffy-tsukoshi/protocols')
// ==========================================================================

export {
  // Vector Clock
  VectorClock,
  // The sound CRDT floor + deterministic projections
  ObservationSet,
  scalarObservation,
  vectorObservation,
  rotorObservation,
  scalarMean,
  vectorMean,
  rotorConsensus,
  // Lattice
  ComponentLattice,
  latticeJoin,
  latticeMeet,
  // Delta
  DeltaEncoding,
  DeltaBatch,
  computeDelta,
  applyDelta,
  applyAdditiveDelta,
  // Storage
  MemoryStore,
  recoverState,
  // Sync
  SyncState,
  PeerConnectionState,
} from './protocols/index.js';

export type {
  Observation,
  ObservationPayload,
  RotorObservation,
  VectorObservation,
  GeometricLattice,
  StateDelta,
  Snapshot,
  StoredOperation,
  StorageStats,
  GeometricStore,
  MemoryStoreConfig,
  RecoveryResult,
  SyncMessage,
  SyncPayload,
  PeerInfo,
  PeerCapabilities,
  PeerState,
  SyncConfig,
} from './protocols/index.js';
