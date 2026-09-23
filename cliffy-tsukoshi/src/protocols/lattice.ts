/**
 * Lattice-based conflict resolution.
 *
 * EPITAPH (2026-09-20, Phase 2 WS0): the magnitude-dominance `GA3Lattice`
 * was deleted with `crdt.ts` — its join was not a semilattice join
 * (`join(+1,−1)` produced cosh(1), outside the hull of its arguments; the
 * geometric-mean tie-break is provably non-associative over multivectors).
 * What survives is the sound floor the salvage blessed:
 *
 * - `latticeJoin` / `latticeMeet`: componentwise max/min over GA3
 *   coefficients — honest semilattice laws.
 * - `ComponentLattice`: the same laws as a class wrapper.
 *
 * For distributed state, use `ObservationSet` (merge = union) plus the
 * deterministic projections in `projection.ts` — the merge is boring; the
 * render is geometric.
 */

import type { GA3 } from '../ga3.js';
import {
  scalar,
  fromCoefficients,
  clone,
} from '../ga3.js';

const EPSILON = 1e-10;

/**
 * Interface for geometric lattice operations.
 */
export interface GeometricLattice<T> {
  /** Lattice join (least upper bound) - always converges */
  join(other: T): T;

  /** Check if this state dominates (is >= to) another */
  dominates(other: T): boolean;

  /** Compute the geometric distance/divergence from another state */
  divergence(other: T): number;

  /** Check if two states are equal in the lattice ordering */
  latticeEq(other: T): boolean;

  /** Compute the lattice meet (greatest lower bound) if it exists */
  meet(other: T): T | null;
}

/**
 * Component-wise lattice operations for multivectors.
 *
 * Unlike GA3Lattice which uses magnitude ordering, this provides
 * coefficient-by-coefficient join/meet operations.
 */
export class ComponentLattice implements GeometricLattice<ComponentLattice> {
  private inner: GA3;

  constructor(mv: GA3) {
    this.inner = mv;
  }

  /** Create from a scalar value. */
  static fromScalar(value: number): ComponentLattice {
    return new ComponentLattice(scalar(value));
  }

  /** Get the underlying multivector. */
  asMultivector(): GA3 {
    return this.inner;
  }

  /** Create a copy. */
  clone(): ComponentLattice {
    return new ComponentLattice(clone(this.inner));
  }

  /** Component-wise maximum (join). */
  join(other: ComponentLattice): ComponentLattice {
    const selfCoeffs = this.inner as number[];
    const otherCoeffs = other.inner as number[];
    const result = selfCoeffs.map((c, i) => Math.max(c, otherCoeffs[i]));
    return new ComponentLattice(fromCoefficients(result));
  }

  /** Dominates if every component is >= the corresponding component. */
  dominates(other: ComponentLattice): boolean {
    const selfCoeffs = this.inner as number[];
    const otherCoeffs = other.inner as number[];
    return selfCoeffs.every((c, i) => c >= otherCoeffs[i] - EPSILON);
  }

  /** L-infinity norm (max component difference). */
  divergence(other: ComponentLattice): number {
    const selfCoeffs = this.inner as number[];
    const otherCoeffs = other.inner as number[];
    return Math.max(...selfCoeffs.map((c, i) => Math.abs(c - otherCoeffs[i])));
  }

  /** Check if two states are equal in the lattice ordering. */
  latticeEq(other: ComponentLattice): boolean {
    return this.dominates(other) && other.dominates(this);
  }

  /** Component-wise minimum (meet). */
  meet(other: ComponentLattice): ComponentLattice {
    const selfCoeffs = this.inner as number[];
    const otherCoeffs = other.inner as number[];
    const result = selfCoeffs.map((c, i) => Math.min(c, otherCoeffs[i]));
    return new ComponentLattice(fromCoefficients(result));
  }

  /** Serialize to JSON. */
  toJSON(): number[] {
    return this.inner as number[];
  }

  /** Deserialize from JSON. */
  static fromJSON(data: number[]): ComponentLattice {
    return new ComponentLattice(fromCoefficients(data));
  }
}

/**
 * Compute component-wise lattice join of two multivectors.
 */
export function latticeJoin(a: GA3, b: GA3): GA3 {
  const aCoeffs = a as number[];
  const bCoeffs = b as number[];
  return fromCoefficients(aCoeffs.map((c, i) => Math.max(c, bCoeffs[i])));
}

/**
 * Compute component-wise lattice meet of two multivectors.
 */
export function latticeMeet(a: GA3, b: GA3): GA3 {
  const aCoeffs = a as number[];
  const bCoeffs = b as number[];
  return fromCoefficients(aCoeffs.map((c, i) => Math.min(c, bCoeffs[i])));
}
