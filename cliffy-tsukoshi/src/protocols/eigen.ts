/**
 * Deterministic 4×4 symmetric eigensolve (cyclic Jacobi) — a faithful port
 * of cliffy-protocols/src/eigen.rs, which is the linear-algebra core of the
 * Markley rotor-consensus projection.
 *
 * Determinism contract (identical to the Rust original): fixed sweep order
 * (0,1) (0,2) (0,3) (1,2) (1,3) (2,3), 50-sweep cap, off-diagonal Frobenius
 * tolerance 1e-15, pure f64 arithmetic — equal inputs ⇒ bit-identical
 * outputs, in Rust AND here. Eigenvalue order is the caller's concern:
 * dominant selection uses a lowest-index tie-break.
 *
 * PORT NOTES (places where JS semantics differ from Rust and the port must
 * not drift):
 * - f64::MIN_POSITIVE === Number.MIN_VALUE (both are the smallest positive
 *   NORMAL double, 2^-1022 — NOT Number.EPSILON).
 * - Rust's f64::signum returns 1.0 for +0.0 and -1.0 for -0.0; JS's
 *   Math.sign returns 0/-0 for both. See rustSignum below.
 * - Math.sqrt and Rust's f64::sqrt are both the correctly-rounded IEEE 754
 *   square root — bit-identical for identical inputs.
 */

/** Rust f64::signum semantics: ±0 keep their sign bit; NaN → NaN. */
function rustSignum(x: number): number {
  if (Number.isNaN(x)) return NaN;
  if (x < 0 || Object.is(x, -0)) return -1;
  return 1; // x > 0 or +0
}

export type Mat4 = number[][]; // 4×4, row-major, fixed literal bounds

function identity4(): Mat4 {
  const v: Mat4 = [];
  for (let i = 0; i < 4; i++) {
    const row = [0, 0, 0, 0];
    row[i] = 1;
    v.push(row);
  }
  return v;
}

function offDiagonalNorm(a: Mat4): number {
  let sum = 0;
  for (let p = 0; p < 4; p++) {
    for (let q = p + 1; q < 4; q++) {
      sum += a[p][q] * a[p][q];
    }
  }
  return Math.sqrt(sum);
}

/**
 * One Jacobi rotation zeroing a[p][q] (classical formulation, numerically
 * stable tangent computation). Column pass, then row pass — the pass order
 * is part of the determinism contract (mirrors the Rust original exactly).
 */
function rotatePair(a: Mat4, v: Mat4, p: number, q: number): void {
  const theta = (a[q][q] - a[p][p]) / (2 * a[p][q]);
  const t = rustSignum(theta) / (Math.abs(theta) + Math.sqrt(theta * theta + 1));
  const c = 1 / Math.sqrt(t * t + 1);
  const s = t * c;

  // A ← Jᵀ A J — column pass first (reads consistent with the classical form).
  for (let row = 0; row < 4; row++) {
    const akp = a[row][p];
    const akq = a[row][q];
    a[row][p] = c * akp - s * akq;
    a[row][q] = s * akp + c * akq;
  }
  for (let col = 0; col < 4; col++) {
    const apk = a[p][col];
    const aqk = a[q][col];
    a[p][col] = c * apk - s * aqk;
    a[q][col] = s * apk + c * aqk;
  }

  // V ← V J (accumulate the eigenvector basis).
  for (let row = 0; row < 4; row++) {
    const vkp = v[row][p];
    const vkq = v[row][q];
    v[row][p] = c * vkp - s * vkq;
    v[row][q] = s * vkp + c * vkq;
  }
}

/**
 * Cyclic Jacobi eigensolve for a symmetric 4×4 matrix.
 *
 * Returns [eigenvalues, eigenvectors] where eigenvalues[i] pairs with
 * eigenvector COLUMN i of eigenvectors. A deterministic function of the
 * input (see module contract).
 */
export function jacobiEigen4(m: Mat4): [number[], Mat4] {
  const MAX_SWEEPS = 50;
  const TOLERANCE = 1e-15;

  const a: Mat4 = m.map((row) => [...row]);
  const v = identity4();

  for (let sweep = 0; sweep < MAX_SWEEPS; sweep++) {
    if (offDiagonalNorm(a) < TOLERANCE) break;
    // Fixed cyclic order — never reorder, never skip.
    for (let p = 0; p < 4; p++) {
      for (let q = p + 1; q < 4; q++) {
        if (Math.abs(a[p][q]) < Number.MIN_VALUE) continue;
        rotatePair(a, v, p, q);
      }
    }
  }

  return [[a[0][0], a[1][1], a[2][2], a[3][3]], v];
}

/**
 * Index of the maximum eigenvalue; ties resolve to the lowest index
 * (deterministic under degenerate spectra).
 */
export function dominantEigenvalueIndex(eigenvalues: number[]): number {
  let best = 0;
  for (let i = 1; i < 4; i++) {
    if (eigenvalues[i] > eigenvalues[best]) best = i;
  }
  return best;
}
