// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! secp256k1 group law over the emulated base field (milestone 2b).
//!
//! Points are Jacobian `(X, Y, Z)` representing affine `(X/Z², Y/Z³)`, so the
//! only inversion is one final conversion to affine — every add and double is
//! pure [`Fe`] arithmetic, hence Goldilocks-native. secp256k1 has `a = 0`, which
//! shortens the doubling formula. The point at infinity is `Z = 0`.
//!
//! Formulas: `dbl-2009-l` (a = 0) and `add-2007-bl`, from the EFD, with the
//! standard equal/negation dispatch so `add` is total.

use super::field::Fe;

/// The generator `G`, big-endian affine `x` then `y`.
const GX: [u8; 32] = [
    0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B, 0x07,
    0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8, 0x17, 0x98,
];
const GY: [u8; 32] = [
    0x48, 0x3A, 0xDA, 0x77, 0x26, 0xA3, 0xC4, 0x65, 0x5D, 0xA4, 0xFB, 0xFC, 0x0E, 0x11, 0x08, 0xA8,
    0xFD, 0x17, 0xB4, 0x48, 0xA6, 0x85, 0x54, 0x19, 0x9C, 0x47, 0xD0, 0x8F, 0xFB, 0x10, 0xD4, 0xB8,
];

/// A point on secp256k1 in Jacobian coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: Fe,
    y: Fe,
    z: Fe,
}

impl Point {
    /// The point at infinity (`Z = 0`).
    pub const INFINITY: Point = Point { x: Fe::ONE, y: Fe::ONE, z: Fe::ZERO };

    /// The generator `G`.
    pub fn generator() -> Point {
        Point::from_affine(Fe::from_bytes(&GX), Fe::from_bytes(&GY))
    }

    /// Lift an affine `(x, y)` into Jacobian coordinates (`Z = 1`).
    pub fn from_affine(x: Fe, y: Fe) -> Point {
        Point { x, y, z: Fe::ONE }
    }

    /// Decompress a SEC1 compressed public key (33 bytes: `0x02|0x03 ‖ x`).
    /// Recovers `y = √(x³ + 7)` with the parity the prefix selects. `None` if
    /// the tag is wrong or `x` is not a valid curve x-coordinate.
    pub fn from_sec1(bytes: &[u8; 33]) -> Option<Point> {
        let tag = bytes[0];
        if tag != 0x02 && tag != 0x03 {
            return None;
        }
        let mut xb = [0u8; 32];
        xb.copy_from_slice(&bytes[1..33]);
        let x = Fe::from_bytes(&xb);
        // y² = x³ + 7  (secp256k1: a = 0, b = 7)
        let mut seven_b = [0u8; 32];
        seven_b[31] = 7;
        let rhs = x.sqr().mul(&x).add(&Fe::from_bytes(&seven_b));
        let mut y = rhs.sqrt()?;
        if y.parity() != (tag & 1) {
            y = y.neg();
        }
        Some(Point::from_affine(x, y))
    }

    /// Whether this is the point at infinity.
    pub fn is_infinity(&self) -> bool {
        self.z.is_zero()
    }

    /// Convert to affine `(x, y)`; `None` for the point at infinity. This is the
    /// single field inversion in a whole scalar multiplication.
    pub fn to_affine(&self) -> Option<(Fe, Fe)> {
        if self.is_infinity() {
            return None;
        }
        let zinv = self.z.inv();
        let zinv2 = zinv.sqr();
        let zinv3 = zinv2.mul(&zinv);
        Some((self.x.mul(&zinv2), self.y.mul(&zinv3)))
    }

    /// Point doubling (`2·self`), secp256k1 `a = 0` (`dbl-2009-l`).
    pub fn double(&self) -> Point {
        if self.is_infinity() {
            return Point::INFINITY;
        }
        let a = self.x.sqr();
        let b = self.y.sqr();
        let c = b.sqr();
        // d = 2·((X+B)² − A − C)
        let t = self.x.add(&b).sqr().sub(&a).sub(&c);
        let d = t.add(&t);
        let e = a.add(&a).add(&a); // 3·A
        let f = e.sqr();
        let x3 = f.sub(&d).sub(&d); // F − 2D
        let eight_c = c.add(&c).add(&c).add(&c).add(&c).add(&c).add(&c).add(&c);
        let y3 = e.mul(&d.sub(&x3)).sub(&eight_c); // E·(D − X3) − 8C
        let z3 = self.y.mul(&self.z);
        let z3 = z3.add(&z3); // 2·Y·Z
        Point { x: x3, y: y3, z: z3 }
    }

    /// Point addition (`self + other`), total (`add-2007-bl` with dispatch).
    pub fn add(&self, other: &Point) -> Point {
        if self.is_infinity() {
            return *other;
        }
        if other.is_infinity() {
            return *self;
        }
        let z1z1 = self.z.sqr();
        let z2z2 = other.z.sqr();
        let u1 = self.x.mul(&z2z2);
        let u2 = other.x.mul(&z1z1);
        let s1 = self.y.mul(&other.z).mul(&z2z2);
        let s2 = other.y.mul(&self.z).mul(&z1z1);
        let h = u2.sub(&u1);
        let rr = s2.sub(&s1);
        if h.is_zero() {
            // same x: either the same point (double) or negations (infinity)
            return if rr.is_zero() { self.double() } else { Point::INFINITY };
        }
        let i = h.add(&h).sqr(); // (2H)²
        let j = h.mul(&i);
        let r = rr.add(&rr); // 2·(S2 − S1)
        let v = u1.mul(&i);
        let x3 = r.sqr().sub(&j).sub(&v).sub(&v); // r² − J − 2V
        let two_s1j = s1.mul(&j).add(&s1.mul(&j)); // 2·S1·J
        let y3 = r.mul(&v.sub(&x3)).sub(&two_s1j); // r·(V − X3) − 2·S1·J
        let z3 = self.z.add(&other.z).sqr().sub(&z1z1).sub(&z2z2).mul(&h);
        Point { x: x3, y: y3, z: z3 }
    }

    /// Scalar multiplication `scalar · self`, MSB-first double-and-add. The
    /// scalar is a big-endian 256-bit integer. Not constant-time — verification
    /// inputs are public.
    pub fn scalar_mul(&self, scalar_be: &[u8; 32]) -> Point {
        let mut acc = Point::INFINITY;
        for &byte in scalar_be.iter() {
            for bit in (0..8).rev() {
                acc = acc.double();
                if (byte >> bit) & 1 == 1 {
                    acc = acc.add(self);
                }
            }
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    use k256::elliptic_curve::PrimeField;
    use k256::{ProjectivePoint, Scalar};
    use num_bigint::BigUint;

    fn n_big() -> BigUint {
        BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .unwrap()
    }

    /// A canonical scalar (< n) as 32 big-endian bytes, from an arbitrary seed.
    fn scalar_bytes(x: &BigUint) -> [u8; 32] {
        let r = x % n_big();
        let b = r.to_bytes_be();
        let mut out = [0u8; 32];
        out[32 - b.len()..].copy_from_slice(&b);
        out
    }

    /// k256's `k·G` affine coordinates, or `None` for the identity.
    fn k256_mul(scalar: &[u8; 32]) -> Option<([u8; 32], [u8; 32])> {
        let s = Scalar::from_repr((*scalar).into()).unwrap();
        let p = (ProjectivePoint::GENERATOR * s).to_affine();
        let ep = p.to_encoded_point(false);
        let x = ep.x()?;
        let y = ep.y()?;
        Some(((*x).into(), (*y).into()))
    }

    fn mine_mul(scalar: &[u8; 32]) -> Option<([u8; 32], [u8; 32])> {
        Point::generator().scalar_mul(scalar).to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes()))
    }

    struct Rng(u64);
    impl Rng {
        fn big(&mut self) -> BigUint {
            let mut b = [0u8; 32];
            for chunk in b.chunks_mut(8) {
                let mut x = self.0;
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                self.0 = x;
                chunk.copy_from_slice(&x.to_le_bytes());
            }
            BigUint::from_bytes_be(&b)
        }
    }

    #[test]
    fn generator_times_scalar_matches_k256() {
        let mut rng = Rng(0x243F6A8885A308D3);
        for _ in 0..40 {
            let s = scalar_bytes(&rng.big());
            if s == [0u8; 32] {
                continue;
            }
            assert_eq!(mine_mul(&s), k256_mul(&s), "k·G mismatch for {}", hex(&s));
        }
    }

    #[test]
    fn small_multiples_match_k256() {
        for k in 1u8..=8 {
            let mut s = [0u8; 32];
            s[31] = k;
            assert_eq!(mine_mul(&s), k256_mul(&s), "{k}·G");
        }
    }

    #[test]
    fn addition_is_consistent_with_scalar_mul() {
        // (a·G) + (b·G) == (a+b)·G
        let mut rng = Rng(0xB7E151628AED2A6A);
        for _ in 0..20 {
            let a = &rng.big() % n_big();
            let b = &rng.big() % n_big();
            let sa = scalar_bytes(&a);
            let sb = scalar_bytes(&b);
            let sum = scalar_bytes(&(&a + &b));
            let pa = Point::generator().scalar_mul(&sa);
            let pb = Point::generator().scalar_mul(&sb);
            let lhs = pa.add(&pb).to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes()));
            let rhs = mine_mul(&sum);
            assert_eq!(lhs, rhs, "(a+b)·G");
        }
    }

    #[test]
    fn infinity_and_negation_edges() {
        let g = Point::generator();
        // G + O = G
        assert_eq!(
            g.add(&Point::INFINITY).to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes())),
            g.to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes())),
        );
        // G + (−G) = O   (−G has the same x, negated y)
        let (gx, gy) = g.to_affine().unwrap();
        let neg_g = Point::from_affine(gx, Fe::ZERO.sub(&gy));
        assert!(g.add(&neg_g).is_infinity(), "G + (−G) = O");
        // G + G = 2G via both paths
        let two_g_add = g.add(&g).to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes()));
        let two_g_dbl = g.double().to_affine().map(|(x, y)| (x.to_bytes(), y.to_bytes()));
        assert_eq!(two_g_add, two_g_dbl);
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
