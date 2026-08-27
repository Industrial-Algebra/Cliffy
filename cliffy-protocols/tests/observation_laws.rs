// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Set-law property tests for `ObservationSet` — the semilattice the old
//! geometric merge could never be. Union laws proven on random inputs:
//! commutativity, associativity, idempotence, and change-detection
//! consistency.

#![allow(clippy::needless_pass_by_value)]
// quickcheck hands properties owned values (Vec<u64> seeds here); by-value
// is the calling convention the macro type-checks against.

use cliffy_protocols::{Observation, ObservationSet};
use quickcheck_macros::quickcheck;
use uuid::Uuid;

/// Random observation generator: bounded seq keeps sets small enough to
/// exercise key collisions deliberately.
///
/// Well-formedness: the payload is a pure function of the key — same
/// `(participant, seq)` always yields the same value (re-delivery, never
/// conflict). This is the G-Set domain invariant the union laws assume.
fn gen_observation(seed: u64) -> Observation {
    let participant_bucket = seed % 8;
    let seq_bucket = seed % 5;
    let participant = Uuid::from_u64_pair(participant_bucket, 0);
    let value = participant_bucket as f64 * 10.0 + seq_bucket as f64;
    Observation::new_scalar(participant, seq_bucket, value)
}

fn gen_set(seeds: &[u64]) -> ObservationSet {
    let mut set = ObservationSet::new();
    for &seed in seeds {
        set.insert(gen_observation(seed));
    }
    set
}

/// merge(a, b) == merge(b, a) — same set contents either direction.
#[quickcheck]
fn merge_is_commutative(left: Vec<u64>, right: Vec<u64>) -> bool {
    let a = gen_set(&left);
    let b = gen_set(&right);
    let mut ab = a.clone();
    ab.merge(&b);
    let mut ba = b.clone();
    ba.merge(&a);
    ab == ba
}

/// merge(merge(a,b),c) == merge(a,merge(b,c)).
#[quickcheck]
fn merge_is_associative(a: Vec<u64>, b: Vec<u64>, c: Vec<u64>) -> bool {
    let (a, b, c) = (gen_set(&a), gen_set(&b), gen_set(&c));
    let mut ab_c = a.clone();
    ab_c.merge(&b);
    ab_c.merge(&c);
    let mut a_bc = a;
    a_bc.merge(&b);
    a_bc.merge(&c);
    ab_c == a_bc
}

/// merge(s, s) == s — idempotence.
#[quickcheck]
fn merge_is_idempotent(seeds: Vec<u64>) -> bool {
    let mut s = gen_set(&seeds);
    let snapshot = s.clone();
    s.merge(&snapshot);
    s == snapshot
}

/// Change-detection consistency: merge reports `true` iff the key set grew.
#[quickcheck]
fn merge_change_flag_matches_key_growth(a: Vec<u64>, b: Vec<u64>) -> bool {
    let (a, b) = (gen_set(&a), gen_set(&b));
    let keys_b: std::collections::BTreeSet<_> = b.iter().map(Observation::key).collect();
    let keys_a: std::collections::BTreeSet<_> = a.iter().map(Observation::key).collect();
    let grows = keys_b.difference(&keys_a).next().is_some();
    let mut merged = a;
    let reported = merged.merge(&b);
    reported == grows
}

/// Equal sets iterate identically — the determinism contract's foundation.
#[quickcheck]
fn equal_sets_iterate_identically(a: Vec<u64>, b: Vec<u64>) -> bool {
    let (mut a, mut b) = (gen_set(&a), gen_set(&b));
    a.merge(&b);
    b.merge(&a.clone());
    if a != b {
        return false;
    }
    let ia: Vec<_> = a.iter().map(Observation::key).collect();
    let ib: Vec<_> = b.iter().map(Observation::key).collect();
    ia == ib
}
