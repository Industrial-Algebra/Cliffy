/**
 * Deterministic geometric projections over an ObservationSet — a faithful
 * port of cliffy-protocols/src/projection.rs (the "render is geometric"
 * half of the salvage).
 *
 * Projections are pure functions of the set: equal sets ⇒ bit-identical
 * values, on every replica, in every merge order, in Rust AND here. All
 * folds run over the set's canonical iteration order, sequentially — float
 * summation order is part of the contract.
 */
import {
  Observation,
  ObservationSet,
  RotorObservation,
  VectorObservation,
} from './observation.js';
import { dominantEigenvalueIndex, jacobiEigen4, type Mat4 } from './eigen.js';

function rotorNorm(r: RotorObservation): number {
  return Math.sqrt(r.w * r.w + r.x * r.x + r.y * r.y + r.z * r.z);
}

function rotorDot(a: RotorObservation, b: RotorObservation): number {
  return a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z;
}

/**
 * Arithmetic mean of every Scalar payload, in canonical order.
 * null when the set holds no scalar observations — boring state takes the
 * boring mean; empty takes null (never fabricate).
 */
export function scalarMean(set: ObservationSet): number | null {
  let sum = 0;
  let count = 0;
  for (const observation of set.entries()) {
    if ('Scalar' in observation.payload) {
      sum += observation.payload.Scalar;
      count += 1;
    }
  }
  return count === 0 ? null : sum / count;
}

/**
 * Componentwise mean of every Vector payload, in canonical order.
 * null when the set holds no vector observations.
 */
export function vectorMean(set: ObservationSet): VectorObservation | null {
  let x = 0;
  let y = 0;
  let z = 0;
  let count = 0;
  for (const observation of set.entries()) {
    if ('Vector' in observation.payload) {
      x += observation.payload.Vector.x;
      y += observation.payload.Vector.y;
      z += observation.payload.Vector.z;
      count += 1;
    }
  }
  return count === 0 ? null : { x: x / count, y: y / count, z: z / count };
}

/**
 * Deterministic rotor consensus: the Markley chordal-L₂ eigen-mean
 * (Markley et al. 2007) — dominant eigenvector of M = Σ wᵢ qᵢ qᵢᵀ over
 * hemisphere-canonicalized unit quaternions, computed with the in-house
 * cyclic Jacobi (eigen.ts). The result is sign-canonicalized (w ≥ 0) so
 * equal sets produce bit-identical rotors on every replica.
 *
 * null when the set holds no usable rotor observations (none present, or
 * all degenerate below the 1e-12 normalization floor — skipped, never
 * fabricated).
 *
 * # Weights
 *
 * The weights closure MUST be a pure function of set-determined observation
 * metadata (never local trust tables), or replicas rendering the same set
 * diverge — the determinism contract is the caller's to keep here.
 */
export function rotorConsensus(set: ObservationSet): RotorObservation | null {
  return rotorConsensusWithWeights(set, () => 1);
}

export function rotorConsensusWithWeights(
  set: ObservationSet,
  weights: (observation: Observation) => number,
): RotorObservation | null {
  // Single canonical-order pass: hemisphere-canonicalize against the FIRST
  // usable rotor (set-determined reference, never a local convention),
  // pair each with its weight, and accumulate M = Σ wᵢ qᵢ qᵢᵀ sequentially —
  // the fold order IS the determinism contract.
  const m: Mat4 = [
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
  ];
  let reference: RotorObservation | null = null;
  let usable = 0;
  for (const observation of set.entries()) {
    if (!('Rotor' in observation.payload)) continue;
    const rotor = observation.payload.Rotor;
    const norm = rotorNorm(rotor);
    if (norm < 1e-12) continue; // degenerate: skip, never fabricate
    const unit: RotorObservation = {
      w: rotor.w / norm,
      x: rotor.x / norm,
      y: rotor.y / norm,
      z: rotor.z / norm,
    };
    if (reference === null) reference = unit;
    const aligned =
      rotorDot(reference, unit) < 0
        ? { w: -unit.w, x: -unit.x, y: -unit.y, z: -unit.z }
        : unit;
    const weight = weights(observation);
    const qc = [aligned.w, aligned.x, aligned.y, aligned.z];
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        m[i][j] += weight * qc[i] * qc[j];
      }
    }
    usable += 1;
  }
  if (usable === 0) return null;

  // Dominant eigenvector, deterministic tie-break, sign canonicalization.
  const [eigenvalues, eigenvectors] = jacobiEigen4(m);
  const dominant = dominantEigenvalueIndex(eigenvalues);
  let result: RotorObservation = {
    w: eigenvectors[0][dominant],
    x: eigenvectors[1][dominant],
    y: eigenvectors[2][dominant],
    z: eigenvectors[3][dominant],
  };
  const norm = rotorNorm(result);
  if (norm < 1e-12) return null;
  result = {
    w: result.w / norm,
    x: result.x / norm,
    y: result.y / norm,
    z: result.z / norm,
  };
  if (result.w < 0 || (result.w === 0 && result.x + result.y + result.z < 0)) {
    result = { w: -result.w, x: -result.x, y: -result.y, z: -result.z };
  }
  return result;
}
