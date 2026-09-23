/**
 * ObservationSet — the sound CRDT floor, ported from
 * cliffy-protocols/src/observation.rs (Phase 1; Rust original is canonical).
 *
 * The CRDT is the set; geometry is a deterministic projection (see
 * projection.ts). Merge is set union — associative, commutative,
 * idempotent by construction. Nothing here can annihilate, reorder, or
 * diverge.
 *
 * # Canonical order (the cross-runtime contract)
 *
 * Iteration order is ascending (participant_id, seq) — UUIDs compare
 * BYTE-wise in Rust's BTreeMap<(Uuid, u64), _>; the canonical hyphenated
 * lowercase string has the same order (hex digits at fixed positions), so
 * string comparison reproduces it exactly.
 *
 * # Duplicate-key semantics (mirrors the Rust original precisely)
 *
 * - insert:  LAST-wins (BTreeMap::insert replaces)
 * - merge:   FIRST-wins (entry.or_insert)
 * - fromWire: LAST-wins (documented deserialization behavior)
 *
 * For well-formed input (a participant assigns seq monotonically) the
 * distinction is unobservable — a re-delivered observation is identical.
 * Same-key-different-payload is malformed/Byzantine input outside the
 * contract; the deterministic rule above is all that is promised.
 */

/** Canonical identity: participant-scoped, globally unique without coordination. */
export type ObservationKey = readonly [participantId: string, seq: number];

export interface RotorObservation {
  /** Hamilton quaternion, right-handed: scalar (cosine) part. */
  w: number;
  x: number;
  y: number;
  z: number;
}

export interface VectorObservation {
  x: number;
  y: number;
  z: number;
}

export type ObservationPayload =
  | { Scalar: number }
  | { Vector: VectorObservation }
  | { Rotor: RotorObservation };

/**
 * One attributed observation. Field names match the Rust serde wire format
 * exactly, so JSON of the Rust shape IS an Observation (structural typing).
 */
export interface Observation {
  /** Observing participant — scopes the sequence number. */
  participant_id: string;
  /** Per-participant monotonic sequence (the participant assigns it). */
  seq: number;
  /** Causal context at observation time. */
  clock: { clocks: Record<string, number> };
  /** What was observed. */
  payload: ObservationPayload;
  /** Provenance grant (Phase 2 capability gating); null = ungated. */
  grant_ref: string | null;
}

export function scalarObservation(participantId: string, seq: number, value: number): Observation {
  return {
    participant_id: participantId,
    seq,
    clock: { clocks: {} },
    payload: { Scalar: value },
    grant_ref: null,
  };
}

export function vectorObservation(
  participantId: string,
  seq: number,
  v: VectorObservation,
): Observation {
  return {
    participant_id: participantId,
    seq,
    clock: { clocks: {} },
    payload: { Vector: v },
    grant_ref: null,
  };
}

export function rotorObservation(
  participantId: string,
  seq: number,
  r: RotorObservation,
): Observation {
  return {
    participant_id: participantId,
    seq,
    clock: { clocks: {} },
    payload: { Rotor: r },
    grant_ref: null,
  };
}

/** Ascending canonical order: participant UUID (string == byte order), then seq. */
function compareKeys(a: ObservationKey, b: ObservationKey): number {
  if (a[0] < b[0]) return -1;
  if (a[0] > b[0]) return 1;
  return a[1] - b[1];
}

/**
 * Grow-only set of attributed observations (G-Set semantics).
 *
 * Backed by a Map keyed `participant_id|seq`; iteration is ALWAYS in
 * canonical sorted order (the Map's insertion order is never exposed to
 * projections — the fold order IS the determinism contract).
 */
export class ObservationSet {
  private readonly observations = new Map<string, Observation>();

  /** Insert (last-wins on duplicate key, mirroring BTreeMap::insert). */
  insert(observation: Observation): boolean {
    const key = ObservationSet.keyString([observation.participant_id, observation.seq]);
    const isNew = !this.observations.has(key);
    this.observations.set(key, observation);
    return isNew;
  }

  /**
   * Union-merge: absorb every observation whose key is absent (first-wins).
   * Returns true if anything changed. This is the entire CRDT.
   */
  merge(other: ObservationSet): boolean {
    const before = this.observations.size;
    for (const [key, observation] of other.observations) {
      if (!this.observations.has(key)) {
        this.observations.set(key, observation);
      }
    }
    return this.observations.size !== before;
  }

  get size(): number {
    return this.observations.size;
  }

  get isEmpty(): boolean {
    return this.observations.size === 0;
  }

  contains(key: ObservationKey): boolean {
    return this.observations.has(ObservationSet.keyString(key));
  }

  /**
   * Iterate in canonical (sorted) key order — the ONLY order projections
   * may consume. Allocates a sorted index per call; projections are
   * read-time, and the allocation is the price of the contract.
   */
  *entries(): IterableIterator<Observation> {
    const sorted = [...this.observations.entries()].sort(([ka], [kb]) => {
      const [pa, sa] = ka.split('|');
      const [pb, sb] = kb.split('|');
      return compareKeys([pa, Number(sa)], [pb, Number(sb)]);
    });
    for (const [, observation] of sorted) {
      yield observation;
    }
  }

  /** Wire format: a JSON array of observations in canonical key order. */
  toWire(): Observation[] {
    return [...this.entries()];
  }

  /** Deserialize (last-wins on duplicate keys, mirroring the Rust side). */
  static fromWire(wire: Observation[]): ObservationSet {
    const set = new ObservationSet();
    for (const o of wire) {
      set.insert(o);
    }
    return set;
  }

  private static keyString([participantId, seq]: ObservationKey): string {
    return `${participantId}|${seq}`;
  }
}
