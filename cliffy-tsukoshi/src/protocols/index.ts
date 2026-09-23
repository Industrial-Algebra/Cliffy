/**
 * Distributed protocols for cliffy-tsukoshi.
 *
 * The sound floor: `ObservationSet` (a grow-only set CRDT — merge is union)
 * plus deterministic geometric projections (`scalarMean`, `vectorMean`,
 * `rotorConsensus` — the Markley eigen-mean via an in-house Jacobi port).
 * Vector clocks, delta sync, storage, and P2P sync plumbing unchanged.
 *
 * REMOVED (Phase 2 WS0, 2026-09-20 — the cutover that reached the twin):
 * `GeometricCRDT`, `geometricMean`, `GA3Lattice`, `GeometricConsensus` —
 * the pre-salvage surface whose merge annihilated data, minted colliding
 * op ids, and joined by magnitude dominance (not a semilattice). The Rust
 * side deleted its twin in 0.4.0; see CHANGELOG for the failure modes.
 *
 * @example
 * ```typescript
 * import {
 *   ObservationSet,
 *   scalarObservation,
 *   scalarMean,
 *   rotorConsensus,
 * } from 'cliffy-tsukoshi/protocols';
 * ```
 *
 * @packageDocumentation
 */

// Vector Clock - causal ordering
export { VectorClock } from './vector-clock.js';

// The sound CRDT floor + deterministic projections (Phase 1, ported WS0)
export {
  ObservationSet,
  scalarObservation,
  vectorObservation,
  rotorObservation,
} from './observation.js';
export type {
  Observation,
  ObservationKey,
  ObservationPayload,
  RotorObservation,
  VectorObservation,
} from './observation.js';
export {
  scalarMean,
  vectorMean,
  rotorConsensus,
  rotorConsensusWithWeights,
} from './projection.js';
export { jacobiEigen4, dominantEigenvalueIndex } from './eigen.js';

// Lattice - the blessed componentwise floor
export {
  GeometricLattice,
  ComponentLattice,
  latticeJoin,
  latticeMeet,
} from './lattice.js';

// Delta - efficient state synchronization
export {
  StateDelta,
  DeltaEncoding,
  DeltaBatch,
  additiveDelta,
  multiplicativeDelta,
  compressedDelta,
  computeDelta,
  computeDeltaCompressed,
  applyDelta,
  applyAdditiveDelta,
  estimateDeltaSize,
  isApplicableTo,
  computeSavings,
} from './delta.js';

// Storage - persistence layer
export {
  Snapshot,
  StoredOperation,
  StorageStats,
  GeometricStore,
  MemoryStore,
  MemoryStoreConfig,
  RecoveryResult,
  recoverState,
} from './storage.js';

// Sync - P2P synchronization protocol
export {
  SyncMessage,
  SyncPayload,
  PeerInfo,
  PeerCapabilities,
  PeerConnectionState,
  PeerState,
  SyncConfig,
  SyncState,
  defaultCapabilities,
  createPeerState,
} from './sync.js';

