// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! RIPEMD-160 as a word-operation circuit (part of milestone 2d).
//!
//! The second half of the Cosmos account id `ripemd160(sha256(pubkey))`. Built
//! from the same 32-bit word operations as [`super::sha256`] — `and`/`xor`/`not`
//! and rotate (a range-decomposition) — each a nox pattern. Little-endian, dual
//! parallel lines, 80 rounds each. Reference uses native `u32`.

const H0: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

const KL: [u32; 5] = [0x00000000, 0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xA953FD4E];
const KR: [u32; 5] = [0x50A28BE6, 0x5C4DD124, 0x6D703EF3, 0x7A6D76E9, 0x00000000];

// message-word selection, left and right lines
const RL: [usize; 80] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0, 9, 5, 2,
    14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8, 12, 4, 13,
    3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15, 13,
];
const RR: [usize; 80] = [
    5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14, 15, 8, 12,
    4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11, 15, 0, 5,
    12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14, 0, 3, 9, 11,
];

// rotation amounts, left and right lines
const SL: [u32; 80] = [
    11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7, 12, 15, 9,
    11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14, 15, 14, 15,
    9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12, 13, 14, 11, 8, 5, 6,
];
const SR: [u32; 80] = [
    8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7, 7, 12, 7,
    6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11, 14, 14, 6,
    14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6, 5, 15, 13, 11, 11,
];

/// The round nonlinear function for round-group `round` (0..5).
fn f(round: usize, x: u32, y: u32, z: u32) -> u32 {
    match round {
        0 => x ^ y ^ z,
        1 => (x & y) | (!x & z),
        2 => (x | !y) ^ z,
        3 => (x & z) | (y & !z),
        _ => x ^ (y | !z),
    }
}

/// The RIPEMD-160 digest of `input`.
pub fn hash(input: &[u8]) -> [u8; 20] {
    let mut h = H0;

    // padding: 0x80, zeros, then the 64-bit little-endian bit length
    let bit_len = (input.len() as u64) * 8;
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    for block in msg.chunks_exact(64) {
        let mut x = [0u32; 16];
        for (i, word) in block.chunks_exact(4).enumerate() {
            x[i] = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        }

        let (mut al, mut bl, mut cl, mut dl, mut el) = (h[0], h[1], h[2], h[3], h[4]);
        let (mut ar, mut br, mut cr, mut dr, mut er) = (h[0], h[1], h[2], h[3], h[4]);

        for j in 0..80 {
            let round = j / 16;
            let tl = al
                .wrapping_add(f(round, bl, cl, dl))
                .wrapping_add(x[RL[j]])
                .wrapping_add(KL[round])
                .rotate_left(SL[j])
                .wrapping_add(el);
            al = el;
            el = dl;
            dl = cl.rotate_left(10);
            cl = bl;
            bl = tl;

            let tr = ar
                .wrapping_add(f(4 - round, br, cr, dr))
                .wrapping_add(x[RR[j]])
                .wrapping_add(KR[round])
                .rotate_left(SR[j])
                .wrapping_add(er);
            ar = er;
            er = dr;
            dr = cr.rotate_left(10);
            cr = br;
            br = tr;
        }

        let t = h[1].wrapping_add(cl).wrapping_add(dr);
        h[1] = h[2].wrapping_add(dl).wrapping_add(er);
        h[2] = h[3].wrapping_add(el).wrapping_add(ar);
        h[3] = h[4].wrapping_add(al).wrapping_add(br);
        h[4] = h[0].wrapping_add(bl).wrapping_add(cr);
        h[0] = t;
    }

    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ripemd::{Digest, Ripemd160};

    fn reference(input: &[u8]) -> [u8; 20] {
        Ripemd160::digest(input).into()
    }

    #[test]
    fn matches_ripemd_crate_on_edge_lengths() {
        for len in [0usize, 1, 3, 55, 56, 57, 63, 64, 65, 100, 128, 200] {
            let input: Vec<u8> = (0..len).map(|i| (i * 17 + 3) as u8).collect();
            assert_eq!(hash(&input), reference(&input), "ripemd160 mismatch at len {len}");
        }
    }

    #[test]
    fn matches_known_vector() {
        // "abc" → the canonical RIPEMD-160 test vector
        let expect: [u8; 20] = {
            let mut o = [0u8; 20];
            let s = "8eb208f7e05d987a9b044a8e98c6b087f15a0bfc";
            for (i, b) in o.iter_mut().enumerate() {
                *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
            }
            o
        };
        assert_eq!(hash(b"abc"), expect);
    }
}
