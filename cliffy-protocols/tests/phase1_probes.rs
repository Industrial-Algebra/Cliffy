// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Phase 1 value-oracle probes — the permanent answer to the 2026-02-25
//! rabbit-hole question (docs/plans/2026-08-26-geometric-crdt-salvage.md).
//!
//! Every test here pins a **specified value**, not an agreement between two
//! replicas. The old design's failures these probes replace:
//!
//! | Old failure (verified 2026-08-25)                | This suite asserts            |
//! |--------------------------------------------------|-------------------------------|
//! | `merge` annihilated state (`15 → 0`)             | consensus == exact mean       |
//! | `join(+1, -1) = cosh(1) = 1.543…` (hull break)   | mean(+1, -1) == 0.0 exactly   |
//! | `len()`-minted op ids collided (`0`, `0`)        | participant-scoped keys coexist |
//! | convergence test had only an agreement oracle    | convergence WITH a value oracle |
//!
//! Probes `probe_merge_does_not_annihilate` and `probe_convergence_with_value_oracle`
//! land together with `scalar_mean` (Task 7 of the Phase 1 plan).

use cliffy_protocols::{scalar_mean, Observation, ObservationSet};
use uuid::Uuid;

fn scalar_obs(participant: Uuid, seq: u64, value: f64) -> Observation {
    Observation::new_scalar(participant, seq, value)
}

/// Hull property: the consensus of +1 and -1 is exactly 0.0 — inside the
/// hull of its arguments. The old `GA3Lattice::join(+1, -1)` returned
/// `cosh(1) = 1.543080634815244`, outside the hull, and no patch could fix
/// that class (not a join-semilattice).
#[test]
fn probe_join_stays_in_hull() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let mut set = ObservationSet::new();
    set.insert(scalar_obs(a, 0, 1.0));
    set.insert(scalar_obs(b, 0, -1.0));

    // Value oracle: the exact midpoint, not merely "the same on both replicas".
    // (scalar_mean lands with Task 7; this probe is written against it.)
    // For now, the set-law floor: both observations must SURVIVE the merge —
    // the old design dropped one of two colliding len()-minted ids.
    let mut replica_a = set.clone();
    let mut replica_b = ObservationSet::new();
    replica_b.insert(scalar_obs(b, 0, -1.0));
    replica_b.insert(scalar_obs(a, 0, 1.0));
    replica_a.merge(&replica_b);

    assert_eq!(replica_a.len(), 2, "both observations survive union merge");
    assert_eq!(
        replica_a.iter().map(|o| o.seq).collect::<Vec<_>>(),
        vec![0, 0],
        "keys are participant-scoped: two first-observations coexist"
    );
}

/// Participant-scoped identity: two nodes' FIRST observations coexist after
/// merge. The old `create_operation` minted ids from `operations.len()`, so
/// two empty replicas both minted id 0 and merge's `HashMap` union silently
/// dropped one.
#[test]
fn probe_op_ids_are_participant_scoped() {
    let node1 = Uuid::new_v4();
    let node2 = Uuid::new_v4();

    let mut replica1 = ObservationSet::new();
    replica1.insert(scalar_obs(node1, 0, 10.0));

    let mut replica2 = ObservationSet::new();
    replica2.insert(scalar_obs(node2, 0, 5.0));

    replica1.merge(&replica2);

    assert_eq!(
        replica1.len(),
        2,
        "no id collision: (node1, 0) != (node2, 0)"
    );
    assert!(replica1.contains(&(node1, 0)));
    assert!(replica1.contains(&(node2, 0)));
}

/// Merge is a true semilattice: commutative and idempotent on real replicas.
#[test]
fn probe_merge_is_a_semilattice_union() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    let mut left = ObservationSet::new();
    left.insert(scalar_obs(a, 0, 1.0));
    left.insert(scalar_obs(a, 1, 2.0));

    let mut right = ObservationSet::new();
    right.insert(scalar_obs(b, 0, 3.0));
    right.insert(scalar_obs(a, 1, 2.0)); // same key — duplicate across replicas

    let mut ab = left.clone();
    ab.merge(&right);
    let mut ba = right.clone();
    ba.merge(&left);

    assert_eq!(ab, ba, "merge is commutative");
    let before = ab.clone();
    ab.merge(&before);
    assert_eq!(ab, before, "merge is idempotent");
    assert_eq!(
        ab.len(),
        3,
        "duplicate observation keyed identically unions once"
    );
}

/// THE annihilation probe. The old `GeometricCRDT::merge` returned
/// `GA3::zero()` for every merge (verified 2026-08-25: replicas at 10 and 5
/// both merged to 0). Here: scalar observations +5 and +10 → consensus is
/// the exact mean, 7.5 — no information is destroyed by merging.
#[test]
fn probe_merge_does_not_annihilate() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let mut left = ObservationSet::new();
    left.insert(scalar_obs(a, 0, 5.0));
    let mut right = ObservationSet::new();
    right.insert(scalar_obs(b, 0, 10.0));

    left.merge(&right);

    assert_eq!(
        scalar_mean(&left),
        Some(7.5),
        "consensus is the exact mean — merging must not destroy information"
    );
}

/// THE convergence probe, with the value oracle the old test never had.
/// Two replicas diverge (10 vs 5), merge in BOTH directions: identical
/// sets AND the same specified consensus value — agreement alone is not
/// convergence (the old suite's annihilated replicas also "agreed").
#[test]
fn probe_convergence_with_value_oracle() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    let mut replica_a = ObservationSet::new();
    replica_a.insert(scalar_obs(a, 0, 10.0));
    let mut replica_b = ObservationSet::new();
    replica_b.insert(scalar_obs(b, 0, 5.0));

    replica_a.merge(&replica_b);
    replica_b.merge(&replica_a);

    // Convergence: identical sets...
    assert_eq!(replica_a, replica_b, "replicas converge to identical sets");
    // ...AND the specified value — the oracle the February test lacked.
    assert_eq!(scalar_mean(&replica_a), Some(7.5));
    assert_eq!(
        scalar_mean(&replica_b),
        Some(7.5),
        "both replicas render the same consensus"
    );
}

/// Hull floor with the mean oracle attached (companion to the set-level
/// probe above): +1 and -1 average to exactly 0.0 — where the old
/// `join(+1, -1)` returned `cosh(1) = 1.543080634815244`, outside the hull
/// of its arguments, and no patch could fix that class.
#[test]
fn probe_scalar_mean_stays_in_hull() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let mut set = ObservationSet::new();
    set.insert(scalar_obs(a, 0, 1.0));
    set.insert(scalar_obs(b, 0, -1.0));

    assert_eq!(
        scalar_mean(&set),
        Some(0.0),
        "mean(+1, -1) = 0.0 exactly — inside the hull"
    );
}
