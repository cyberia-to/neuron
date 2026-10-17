// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Provable claim verification (Phase 2) — see `specs/bridge-proof.md`.
//!
//! Re-expresses the secp256k1 + sha256 + ripemd160 verification of a migration
//! [`crate::claim`] as a computation over the Goldilocks field, so it can run on
//! nox and be proven by zheng — making the legacy-key binding trustless.
//!
//! secp256k1 lives in a 256-bit field; nox reduces over Goldilocks
//! (`p = 2^64 − 2^32 + 1`) only. So every secp256k1 value is emulated as 16
//! limbs of 16 bits, each a Goldilocks element, chosen so no limb product or
//! column sum ever wraps the native field — every step is a legal nox
//! `mul`/`add`.
//!
//! Milestone ladder: **2a base field (this module)** → 2b curve → 2c ECDSA →
//! 2d sha256/ripemd160 → 2e assemble into a nox program + zheng proof.

pub mod curve;
pub mod ecdsa;
pub mod field;
#[cfg(feature = "prove")]
pub mod prove;
pub mod ripemd160;
pub mod scalar;
pub mod sha256;
