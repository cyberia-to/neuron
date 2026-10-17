// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! secp256k1 scalar field `𝔽_n`, `n` the group order, emulated as Goldilocks
//! limbs (part of milestone 2c).
//!
//! Same 16×16-bit-limb representation as the base field, but `n` has no Solinas
//! shape, so reduction uses the general fold `2^256 ≡ M (mod n)` with the full
//! constant `M = 2^256 − n` (129 bits). Each fold is a limb schoolbook multiply
//! by `M` plus an add — Goldilocks-native like everything else. Shares the pure
//! limb helpers (`schoolbook`, `normalize_vec`, `add_into`) with the base field.

use nebu::field::Goldilocks;

use super::field::{add_into, normalize_vec, schoolbook};

const N: usize = 16;
const B: u32 = 16;

/// The group order `n`, little-endian 16-bit limbs (derived + checked in tests).
const ORDER: [u64; N] = [
    0x4141, 0xD036, 0x5E8C, 0xBFD2, 0xA03B, 0xAF48, 0xDCE6, 0xBAAE, 0xFFFE, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

/// `n − 2`, the Fermat inverse exponent.
const ORDER_MINUS_2: [u64; N] = [
    0x413F, 0xD036, 0x5E8C, 0xBFD2, 0xA03B, 0xAF48, 0xDCE6, 0xBAAE, 0xFFFE, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

/// The fold constant `M = 2^256 − n = 0x1_45512319_50B75FC4_402DA173_2FC9BEBF`
/// (129 bits → 9 limbs). `2^256 ≡ M (mod n)`.
const M: [u64; 9] = [
    0xBEBF, 0x2FC9, 0xA173, 0x402D, 0x5FC4, 0x50B7, 0x2319, 0x4551, 0x0001,
];

/// An element of the secp256k1 scalar field, stored reduced (`< n`).
#[derive(Clone, Copy, Debug)]
pub struct Scalar {
    limbs: [u64; N],
}

impl Scalar {
    /// The additive identity.
    pub const ZERO: Scalar = Scalar { limbs: [0; N] };

    /// The multiplicative identity.
    pub const ONE: Scalar = Scalar {
        limbs: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    /// Reduce a big-endian 32-byte integer mod `n`.
    pub fn from_bytes(be: &[u8; 32]) -> Scalar {
        let mut limbs = [0u64; N];
        for i in 0..N {
            let hi = be[30 - 2 * i] as u64;
            let lo = be[31 - 2 * i] as u64;
            limbs[i] = (hi << 8) | lo;
        }
        fold_reduce(&limbs)
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

    /// Whether this is zero.
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&l| l == 0)
    }

    /// Scalar multiplication mod `n`.
    pub fn mul(&self, other: &Scalar) -> Scalar {
        fold_reduce(&schoolbook(&self.limbs, &other.limbs))
    }

    /// Multiplicative inverse mod `n` via Fermat: `a^(n−2)`.
    pub fn inv(&self) -> Scalar {
        let mut result = Scalar::ONE;
        for i in (0..N).rev() {
            for bit in (0..B).rev() {
                result = result.mul(&result);
                if (ORDER_MINUS_2[i] >> bit) & 1 == 1 {
                    result = result.mul(self);
                }
            }
        }
        result
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Scalar) -> bool {
        self.limbs == other.limbs
    }
}
impl Eq for Scalar {}

// ── reduction ────────────────────────────────────────────────────────────────

/// `M · hi` as a normalized limb vector (general schoolbook by the 9-limb `M`).
fn mul_by_m(hi: &[u64]) -> Vec<u64> {
    let mut cols = vec![0u64; hi.len() + M.len()];
    for (i, &h) in hi.iter().enumerate() {
        let hg = Goldilocks::new(h);
        for (j, &m) in M.iter().enumerate() {
            let prod = hg * Goldilocks::new(m); // < 2^32
            let acc = Goldilocks::new(cols[i + j]) + prod;
            cols[i + j] = acc.as_u64();
        }
    }
    normalize_vec(&mut cols);
    cols
}

/// Reduce an arbitrary-width limb integer mod `n`.
///
/// Folds the part above `2^256` with `M = 2^256 − n` (`2^256 ≡ M mod n`) until
/// nothing remains above `2^256`, then conditionally subtracts `n`.
fn fold_reduce(input: &[u64]) -> Scalar {
    let mut acc: Vec<u64> = input.to_vec();
    normalize_vec(&mut acc);

    while acc.len() > N && acc[N..].iter().any(|&l| l != 0) {
        let hi = acc[N..].to_vec();
        let mut low: Vec<u64> = acc[..N].to_vec();
        add_into(&mut low, &mul_by_m(&hi));
        normalize_vec(&mut low);
        acc = low;
    }
    acc.resize(N, 0);

    let mut limbs = [0u64; N];
    limbs.copy_from_slice(&acc[..N]);
    while geq_order(&limbs) {
        limbs = sub_order(&limbs);
    }
    Scalar { limbs }
}

/// Whether a 16-limb value is `≥ n`.
fn geq_order(x: &[u64; N]) -> bool {
    for i in (0..N).rev() {
        if x[i] != ORDER[i] {
            return x[i] > ORDER[i];
        }
    }
    true
}

/// `x − n` for `x ≥ n`.
fn sub_order(x: &[u64; N]) -> [u64; N] {
    let mut out = [0u64; N];
    let mut borrow = 0i64;
    for i in 0..N {
        let mut v = x[i] as i64 - ORDER[i] as i64 - borrow;
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

    fn n_big() -> BigUint {
        BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .unwrap()
    }

    fn to_big(s: &Scalar) -> BigUint {
        BigUint::from_bytes_be(&s.to_bytes())
    }
    fn from_big(x: &BigUint) -> Scalar {
        let b = (x % n_big()).to_bytes_be();
        let mut be = [0u8; 32];
        be[32 - b.len()..].copy_from_slice(&b);
        Scalar::from_bytes(&be)
    }

    struct Rng(u64);
    impl Rng {
        fn bytes32(&mut self) -> [u8; 32] {
            let mut b = [0u8; 32];
            for chunk in b.chunks_mut(8) {
                let mut x = self.0;
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                self.0 = x;
                chunk.copy_from_slice(&x.to_le_bytes());
            }
            b
        }
    }

    #[test]
    fn order_constant_is_correct() {
        // n itself reduces to 0
        let b = n_big().to_bytes_be();
        let mut be = [0u8; 32];
        be[32 - b.len()..].copy_from_slice(&b);
        assert!(Scalar::from_bytes(&be).is_zero(), "n ≡ 0");
    }

    #[test]
    fn mul_matches_bigint() {
        let n = n_big();
        let mut rng = Rng(0xCAFEF00DD15EA5E5);
        for _ in 0..3000 {
            let a = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &n));
            let b = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &n));
            assert_eq!(to_big(&a.mul(&b)), (to_big(&a) * to_big(&b)) % &n);
        }
    }

    #[test]
    fn inverse_is_a_true_inverse() {
        let n = n_big();
        let mut rng = Rng(0x0123456789ABCDEF);
        for _ in 0..150 {
            let a = from_big(&(BigUint::from_bytes_be(&rng.bytes32()) % &n));
            if a.is_zero() {
                continue;
            }
            assert_eq!(a.mul(&a.inv()), Scalar::ONE, "a·a⁻¹ = 1 mod n");
        }
    }
}
