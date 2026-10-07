// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! secp256k1 base field `𝔽_q`, `q = 2^256 − 2^32 − 977`, emulated as Goldilocks
//! limbs (milestone 2a).
//!
//! An element is 16 little-endian limbs of 16 bits. A limb is a Goldilocks
//! element in `[0, 2^16)`. The multiply builds its 16×16 schoolbook columns with
//! `nebu::Goldilocks` operations: a limb product is `< 2^32` and a column sums at
//! most 16 of them, `< 2^36`, so every intermediate is exact inside one
//! Goldilocks element (`p ≈ 2^64`) — i.e. a legal nox `mul`/`add` step. Carry
//! extraction is a native shift here; in-circuit it is a range-check gadget.
//!
//! Reduction mod `q` uses the Solinas identity `2^256 ≡ 2^32 + 977 (mod q)`.

use nebu::field::Goldilocks;

/// Number of 16-bit limbs in a field element (256 bits).
const N: usize = 16;
/// Limb radix: each limb holds this many bits.
const B: u32 = 16;
/// Limb mask (`2^16 − 1`).
const MASK: u64 = 0xFFFF;

/// secp256k1 base field prime `q`, little-endian 16-bit limbs.
/// `q = FFFFFFFF…FFFFFFFE FFFFFC2F`.
const Q: [u64; N] = [
    0xFC2F, 0xFFFF, 0xFFFE, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

/// `q − 2`, the Fermat inverse exponent (`q` with limb 0 lowered by 2).
const Q_MINUS_2: [u64; N] = [
    0xFC2D, 0xFFFF, 0xFFFE, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

/// `(q + 1) / 4` — the square-root exponent, since `q ≡ 3 (mod 4)`.
const Q_PLUS_1_DIV4: [u64; N] = [
    0xFF0C, 0xBFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0x3FFF,
];

/// The Solinas fold constant `R = 2^256 mod q = 2^32 + 977`, low-16-bit limbs.
/// `977 = 0x3D1` at limb 0, `2^32` at limb 2.
const R: [u64; 3] = [0x03D1, 0x0000, 0x0001];

/// An element of the secp256k1 base field, stored reduced (`< q`).
#[derive(Clone, Copy, Debug)]
pub struct Fe {
    /// 16 little-endian 16-bit limbs, each in `[0, 2^16)`.
    limbs: [u64; N],
}

impl Fe {
    /// The additive identity.
    pub const ZERO: Fe = Fe { limbs: [0; N] };

    /// The multiplicative identity.
    pub const ONE: Fe = Fe {
        limbs: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    /// Reduce a big-endian 32-byte integer into the field.
    pub fn from_bytes(be: &[u8; 32]) -> Fe {
        let mut limbs = [0u64; N];
        for i in 0..N {
            // limb i is bytes at big-endian positions 30−2i .. 32−2i
            let hi = be[30 - 2 * i] as u64;
            let lo = be[31 - 2 * i] as u64;
            limbs[i] = (hi << 8) | lo;
        }
        reduce_wide(&limbs)
    }

    /// The element as a big-endian 32-byte integer.
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..N {
            out[30 - 2 * i] = (self.limbs[i] >> 8) as u8;
            out[31 - 2 * i] = (self.limbs[i] & 0xFF) as u8;
        }
        out
    }

    /// Whether this is the additive identity.
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&l| l == 0)
    }

    /// Field addition.
    pub fn add(&self, other: &Fe) -> Fe {
        // sum of two reduced elements is < 2q < 2^257 → 17 limbs, then reduce.
        let mut wide = [0u64; N + 1];
        for i in 0..N {
            wide[i] = self.limbs[i] + other.limbs[i];
        }
        normalize(&mut wide);
        reduce_wide(&wide)
    }

    /// Field subtraction (`self − other`), via `self + (q − other)`.
    pub fn sub(&self, other: &Fe) -> Fe {
        let neg = sub_from_q(&other.limbs); // q − other ∈ [1, q]
        let mut wide = [0u64; N + 1];
        for i in 0..N {
            wide[i] = self.limbs[i] + neg[i];
        }
        normalize(&mut wide);
        reduce_wide(&wide)
    }

    /// Field multiplication.
    pub fn mul(&self, other: &Fe) -> Fe {
        let product = schoolbook(&self.limbs, &other.limbs); // 32 limbs, < 2^512
        reduce_wide(&product)
    }

    /// Field squaring.
    pub fn sqr(&self) -> Fe {
        self.mul(self)
    }

    /// Multiplicative inverse via Fermat: `a^(q−2)`. Inverse of zero is zero.
    pub fn inv(&self) -> Fe {
        self.pow(&Q_MINUS_2)
    }

    /// A square root, if one exists. `q ≡ 3 (mod 4)`, so a root is `a^((q+1)/4)`;
    /// returns `None` when `a` is a non-residue. Used to decompress a public key.
    pub fn sqrt(&self) -> Option<Fe> {
        let cand = self.pow(&Q_PLUS_1_DIV4);
        if cand.sqr() == *self { Some(cand) } else { None }
    }

    /// The least-significant bit of the canonical integer — the `y`-parity used
    /// in SEC1 point compression.
    pub fn parity(&self) -> u8 {
        (self.limbs[0] & 1) as u8
    }

    /// The additive inverse, `q − self` (0 maps to 0).
    pub fn neg(&self) -> Fe {
        Fe::ZERO.sub(self)
    }

    /// Exponentiation by a 256-bit exponent (little-endian 16-bit limbs),
    /// square-and-multiply from the most-significant bit.
    pub fn pow(&self, exp: &[u64; N]) -> Fe {
        let mut result = Fe::ONE;
        for i in (0..N).rev() {
            for bit in (0..B).rev() {
                result = result.sqr();
                if (exp[i] >> bit) & 1 == 1 {
                    result = result.mul(self);
                }
            }
        }
        result
    }
}

impl PartialEq for Fe {
    fn eq(&self, other: &Fe) -> bool {
        self.limbs == other.limbs
    }
}
impl Eq for Fe {}

// ── limb helpers ────────────────────────────────────────────────────────────

/// Carry-propagate a limb vector so every limb is a 16-bit digit. The final
/// carry rides into an extra high limb if present. In-circuit each split is a
/// range-check; here it is a native shift.
fn normalize(limbs: &mut [u64]) {
    let mut carry = 0u64;
    for limb in limbs.iter_mut() {
        let v = *limb + carry;
        *limb = v & MASK;
        carry = v >> B;
    }
    debug_assert_eq!(carry, 0, "normalize overflowed its limb vector");
}

/// The 16×16 schoolbook product, as 32 normalized 16-bit limbs.
///
/// Columns are accumulated with `nebu::Goldilocks` operations: each limb product
/// is `< 2^32` and each column sums ≤ 16 of them (`< 2^36`), so no column ever
/// reaches Goldilocks' modulus — the no-wraparound condition that makes every
/// step a valid nox `mul`/`add`.
pub(crate) fn schoolbook(a: &[u64; N], b: &[u64; N]) -> [u64; 2 * N] {
    let mut cols = [0u64; 2 * N];
    for i in 0..N {
        let ai = Goldilocks::new(a[i]);
        for j in 0..N {
            let prod = ai * Goldilocks::new(b[j]); // < 2^32, exact in Goldilocks
            let acc = Goldilocks::new(cols[i + j]) + prod; // column sum < 2^36
            cols[i + j] = acc.as_u64();
        }
    }
    normalize(&mut cols);
    cols
}

/// `q − x` for a reduced `x` (`x < q`), as 16 normalized limbs (result in `[1, q]`).
fn sub_from_q(x: &[u64; N]) -> [u64; N] {
    let mut out = [0u64; N];
    let mut borrow = 0i64;
    for i in 0..N {
        let mut v = Q[i] as i64 - x[i] as i64 - borrow;
        if v < 0 {
            v += 1 << B;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = v as u64;
    }
    out
}

/// Reduce an arbitrary-width (≤ 32-limb) integer mod `q` into a field element.
///
/// Folds the part above `2^256` with the Solinas constant `R = 2^32 + 977`
/// (`2^256 ≡ R mod q`) until the value fits 16 limbs, then conditionally
/// subtracts `q`.
fn reduce_wide(input: &[u64]) -> Fe {
    let mut acc: Vec<u64> = input.to_vec();
    normalize_vec(&mut acc);

    // Fold the high half (limbs ≥ 16) down via R until nothing remains above 2^256.
    while acc.len() > N && acc[N..].iter().any(|&l| l != 0) {
        let hi = acc[N..].to_vec();
        let mut low: Vec<u64> = acc[..N].to_vec();
        let folded = mul_by_r(&hi); // R · hi
        add_into(&mut low, &folded);
        normalize_vec(&mut low);
        acc = low;
    }
    acc.resize(N, 0);

    // acc < 2^256 now; 2^256 − q = R < q, so acc < 2q → at most two subtractions.
    let mut limbs = [0u64; N];
    limbs.copy_from_slice(&acc[..N]);
    while geq_q(&limbs) {
        limbs = sub_q(&limbs);
    }
    Fe { limbs }
}

/// `R · hi`, where `R = 2^32 + 977`: `(hi << 2 limbs) + 977·hi`.
fn mul_by_r(hi: &[u64]) -> Vec<u64> {
    // 977·hi
    let mut scaled = vec![0u64; hi.len() + 1];
    for (i, &h) in hi.iter().enumerate() {
        let acc = Goldilocks::new(scaled[i]) + Goldilocks::new(h) * Goldilocks::new(R[0]);
        scaled[i] = acc.as_u64();
    }
    normalize_vec(&mut scaled);
    // hi << 2 limbs (× 2^32), using R's limb layout (limb 2 = 1)
    let mut shifted = vec![0u64; hi.len() + 2];
    for (i, &h) in hi.iter().enumerate() {
        shifted[i + 2] = h;
    }
    add_into(&mut scaled, &shifted);
    normalize_vec(&mut scaled);
    scaled
}

/// `dst += src` (limb-wise, dst grown as needed; caller normalizes).
pub(crate) fn add_into(dst: &mut Vec<u64>, src: &[u64]) {
    if dst.len() < src.len() {
        dst.resize(src.len(), 0);
    }
    for (i, &s) in src.iter().enumerate() {
        dst[i] += s;
    }
}

/// Carry-propagate a growable limb vector, extending it for any final carry.
pub(crate) fn normalize_vec(limbs: &mut Vec<u64>) {
    let mut carry = 0u64;
    for limb in limbs.iter_mut() {
        let v = *limb + carry;
        *limb = v & MASK;
        carry = v >> B;
    }
    while carry != 0 {
        limbs.push(carry & MASK);
        carry >>= B;
    }
}

/// Whether a 16-limb value is `≥ q`.
fn geq_q(x: &[u64; N]) -> bool {
    for i in (0..N).rev() {
        if x[i] != Q[i] {
            return x[i] > Q[i];
        }
    }
    true // equal to q
}

/// `x − q` for `x ≥ q`, as 16 limbs.
fn sub_q(x: &[u64; N]) -> [u64; N] {
    let mut out = [0u64; N];
    let mut borrow = 0i64;
    for i in 0..N {
        let mut v = x[i] as i64 - Q[i] as i64 - borrow;
        if v < 0 {
            v += 1 << B;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = v as u64;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    fn q_big() -> BigUint {
        (BigUint::from(1u8) << 256u32) - (BigUint::from(1u8) << 32u32) - BigUint::from(977u32)
    }

    fn to_big(fe: &Fe) -> BigUint {
        BigUint::from_bytes_be(&fe.to_bytes())
    }

    fn from_big(x: &BigUint) -> Fe {
        let bytes = x.to_bytes_be();
        let mut be = [0u8; 32];
        be[32 - bytes.len()..].copy_from_slice(&bytes);
        Fe::from_bytes(&be)
    }

    // Deterministic xorshift64 — no external rng, reproducible (determinism pass).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn bytes32(&mut self) -> [u8; 32] {
            let mut b = [0u8; 32];
            for chunk in b.chunks_mut(8) {
                chunk.copy_from_slice(&self.next().to_le_bytes());
            }
            b
        }
    }

    #[test]
    fn constants_reduce_correctly() {
        assert_eq!(to_big(&Fe::ZERO), BigUint::from(0u8));
        assert_eq!(to_big(&Fe::ONE), BigUint::from(1u8));
        // q itself reduces to 0
        let mut q_be = [0u8; 32];
        let qb = q_big().to_bytes_be();
        q_be[32 - qb.len()..].copy_from_slice(&qb);
        assert!(Fe::from_bytes(&q_be).is_zero(), "q ≡ 0");
    }

    #[test]
    fn bytes_round_trip_below_q() {
        let v = &q_big() - BigUint::from(12345u32);
        assert_eq!(to_big(&from_big(&v)), v);
    }

    #[test]
    fn add_sub_mul_match_bigint() {
        let q = q_big();
        let mut rng = Rng(0x9E3779B97F4A7C15);
        for _ in 0..4000 {
            let a = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &q));
            let b = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &q));
            let ba = to_big(&a);
            let bb = to_big(&b);
            assert_eq!(to_big(&a.add(&b)), (&ba + &bb) % &q, "add");
            assert_eq!(to_big(&a.sub(&b)), (&ba + &q - &bb) % &q, "sub");
            assert_eq!(to_big(&a.mul(&b)), (&ba * &bb) % &q, "mul");
        }
    }

    #[test]
    fn edge_values() {
        let q = q_big();
        let one = Fe::ONE;
        let qm1 = from_big(&(&q - BigUint::from(1u8)));
        // (q−1) + 1 = 0
        assert!(qm1.add(&one).is_zero());
        // (q−1)·(q−1) = 1  (since q−1 ≡ −1)
        assert_eq!(to_big(&qm1.mul(&qm1)), BigUint::from(1u8));
        // 0 − 1 = q − 1
        assert_eq!(Fe::ZERO.sub(&one), qm1);
    }

    #[test]
    fn inverse_is_a_true_inverse() {
        let mut rng = Rng(0xD1B54A32D192ED03);
        for _ in 0..200 {
            let q = q_big();
            let a = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &q));
            if a.is_zero() {
                continue;
            }
            assert_eq!(a.mul(&a.inv()), Fe::ONE, "a·a⁻¹ = 1");
        }
    }

    #[test]
    fn schoolbook_columns_stay_inside_goldilocks() {
        // The soundness condition for the nox mapping: no column reaches p.
        // Max column before normalize = 16·(2^16−1)^2 < 2^36 ≪ Goldilocks P.
        let max_col = 16u128 * (0xFFFFu128 * 0xFFFFu128);
        assert!(max_col < nebu::field::P as u128, "columns never wrap the native field");
    }
}
