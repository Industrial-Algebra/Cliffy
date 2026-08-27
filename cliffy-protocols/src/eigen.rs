// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Deterministic 4×4 symmetric eigensolve (cyclic Jacobi) — the linear
//! algebra core of the Markley rotor-consensus projection.
//!
//! Determinism contract: fixed sweep order `(0,1) (0,2) (0,3) (1,2) (1,3)
//! (2,3)`, 50-sweep cap, off-diagonal Frobenius tolerance 1e-15, pure f64
//! arithmetic — equal inputs ⇒ bit-identical outputs. Eigenvalue *order* is
//! the caller's concern: dominant-eigenvalue selection uses a lowest-index
//! tie-break so degenerate spectra resolve deterministically.
//!
//! No external linalg dependency by design: ~80 auditable lines, fully
//! oracle-tested below (the salvage plan's "boring floor" philosophy
//! applied to linear algebra).
#![allow(clippy::indexing_slicing)]
// All indexing here is over fixed-size [[f64; 4]; 4] arrays with loop
// bounds of literal 4 — the array length is in the type and the indices
// are structurally bounded; .get() ceremony would obscure the math.
#![allow(clippy::many_single_char_names)]
// p, q, c, s, t are the Jacobi-rotation literature's own notation
// (plane indices, cosine, sine, tangent); renaming them would divorce
// the code from every textbook presentation of the algorithm.
#![allow(clippy::needless_range_loop)]
// The (p, q) sweep in jacobi_eigen_4 enumerates rotation-plane pairs —
// indices are the domain; iterator gymnastics would obscure the fixed
// cyclic order the determinism contract promises.

/// Cyclic Jacobi eigensolve for a symmetric 4×4 matrix.
///
/// Returns `(eigenvalues, eigenvectors)` where `eigenvalues[i]` pairs with
/// eigenvector **column** `i` of `eigenvectors`. The output is a
/// deterministic function of the input (see module contract).
#[must_use]
pub fn jacobi_eigen_4(m: [[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4]) {
    const MAX_SWEEPS: usize = 50;
    const TOLERANCE: f64 = 1e-15;

    let mut a = m;
    let mut v = identity_4();

    for _ in 0..MAX_SWEEPS {
        if off_diagonal_norm(&a) < TOLERANCE {
            break;
        }
        // Fixed cyclic order — never reorder, never skip.
        for p in 0..4 {
            for q in (p + 1)..4 {
                if a[p][q].abs() < f64::MIN_POSITIVE {
                    continue;
                }
                rotate_pair(&mut a, &mut v, p, q);
            }
        }
    }

    let eigenvalues = [a[0][0], a[1][1], a[2][2], a[3][3]];
    (eigenvalues, v)
}

/// Index of the maximum eigenvalue; ties resolve to the lowest index
/// (deterministic under degenerate spectra).
#[must_use]
pub fn dominant_eigenvalue_index(eigenvalues: &[f64; 4]) -> usize {
    let mut best = 0;
    for i in 1..4 {
        if eigenvalues[i] > eigenvalues[best] {
            best = i;
        }
    }
    best
}

fn identity_4() -> [[f64; 4]; 4] {
    let mut v = [[0.0; 4]; 4];
    for (i, row) in v.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    v
}

fn off_diagonal_norm(a: &[[f64; 4]; 4]) -> f64 {
    let sum: f64 = a
        .iter()
        .enumerate()
        .flat_map(|(p, row)| {
            row.iter()
                .enumerate()
                .filter(move |(q, _)| q > &p)
                .map(|(_, x)| x * x)
        })
        .sum();
    sum.sqrt()
}

/// One Jacobi rotation zeroing `a[p][q]` (classical formulation,
/// numerically stable tangent computation).
fn rotate_pair(a: &mut [[f64; 4]; 4], v: &mut [[f64; 4]; 4], p: usize, q: usize) {
    let theta_denom = 2.0 * a[p][q];
    let theta = (a[q][q] - a[p][p]) / theta_denom;
    let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
    let c = 1.0 / (t * t + 1.0).sqrt();
    let s = t * c;

    // A ← Jᵀ A J, with J the Givens rotation in the (p, q) plane.
    // Column pass, then row pass (columns first keeps the reads consistent
    // with classical formulations).
    for row in a.iter_mut() {
        let (akp, akq) = (row[p], row[q]);
        row[p] = c * akp - s * akq;
        row[q] = s * akp + c * akq;
    }
    for col in 0..4 {
        let (apk, aqk) = (a[p][col], a[q][col]);
        a[p][col] = c * apk - s * aqk;
        a[q][col] = s * apk + c * aqk;
    }

    // V ← V J (accumulate the eigenvector basis).
    for vrow in v.iter_mut() {
        let (vkp, vkq) = (vrow[p], vrow[q]);
        vrow[p] = c * vkp - s * vkq;
        vrow[q] = s * vkp + c * vkq;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    // Eigenvalue oracles assert analytic values within tight tolerances;
    // the determinism tests assert exact bit equality (that IS the point).

    use super::*;

    fn assert_close(a: f64, b: f64, msg: &str) {
        assert!((a - b).abs() < 1e-12, "{msg}: {a} vs {b}");
    }

    /// Diagonal input: already solved — eigenvalues on the diagonal, basis
    /// vectors unchanged.
    #[test]
    fn diagonal_matrix_is_a_fixed_point() {
        let (vals, vecs) = jacobi_eigen_4([
            [4.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 3.0],
        ]);
        for (i, expected) in [4.0, 1.0, 2.0, 3.0].iter().enumerate() {
            assert_close(vals[i], *expected, "eigenvalue");
        }
        for i in 0..4 {
            for j in 0..4 {
                assert_close(vecs[i][j], if i == j { 1.0 } else { 0.0 }, "identity basis");
            }
        }
    }

    /// Embedded 2×2: [[2,1],[1,2]] has eigenvalues 3 and 1.
    #[test]
    fn embedded_two_by_two() {
        let m = [
            [2.0, 1.0, 0.0, 0.0],
            [1.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 5.0, 0.0],
            [0.0, 0.0, 0.0, 7.0],
        ];
        let (vals, vecs) = jacobi_eigen_4(m);

        // Spectrum as a sorted set.
        let mut spectrum = vals;
        spectrum.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (got, expected) in spectrum.iter().zip([1.0, 3.0, 5.0, 7.0].iter()) {
            assert_close(*got, *expected, "spectrum");
        }

        // Eigendecomposition property: Vᵀ M V = diag(vals) for the RETURNED
        // pairing (not the sorted one).
        for i in 0..4 {
            for j in 0..4 {
                let mut mv_ij = 0.0;
                for k in 0..4 {
                    for l in 0..4 {
                        mv_ij += vecs[k][i] * m[k][l] * vecs[l][j];
                    }
                }
                let expected = if i == j { vals[i] } else { 0.0 };
                assert!(
                    (mv_ij - expected).abs() < 1e-10,
                    "VᵀMV[{i}][{j}] = {mv_ij}, expected {expected}"
                );
            }
        }
    }

    /// The Markley consistency case: a rank-1 outer product `q qᵀ` has
    /// dominant eigenvalue ‖q‖² with eigenvector parallel to `q`.
    #[test]
    fn rank_one_outer_product_dominant_vector_is_q() {
        let q = [0.5, 0.5, 0.5, 0.5]; // unit
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] = q[i] * q[j];
            }
        }
        let (vals, vecs) = jacobi_eigen_4(m);
        let dominant = dominant_eigenvalue_index(&vals);
        assert_close(vals[dominant], 1.0, "dominant eigenvalue = ‖q‖²");
        // Column `dominant` ∥ q (sign-free: check |dot| ≈ 1).
        let mut dot = 0.0;
        for i in 0..4 {
            dot += vecs[i][dominant] * q[i];
        }
        assert!(
            (dot.abs() - 1.0).abs() < 1e-10,
            "eigenvector ∥ q, |dot| = {}",
            dot.abs()
        );
    }

    /// Equal inputs ⇒ bit-identical outputs, including degenerate ties.
    #[test]
    fn deterministic_under_repetition_and_degeneracy() {
        // Degenerate spectrum: identity-like with a triple eigenvalue.
        let m = [
            [2.0, 0.3, -0.1, 0.2],
            [0.3, 2.0, 0.4, -0.5],
            [-0.1, 0.4, 2.0, 0.1],
            [0.2, -0.5, 0.1, 2.0],
        ];
        let (vals_a, vecs_a) = jacobi_eigen_4(m);
        let (vals_b, vecs_b) = jacobi_eigen_4(m);
        assert_eq!(vals_a, vals_b, "eigenvalues bit-identical");
        assert_eq!(vecs_a, vecs_b, "eigenvectors bit-identical");

        // Degenerate tie: lowest index wins.
        let tie = identity_4();
        let (vals, _) = jacobi_eigen_4(tie);
        assert_eq!(dominant_eigenvalue_index(&vals), 0, "tie → lowest index");
    }
}
