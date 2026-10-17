// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! neuron-auth — who acts, and how that is proven.
//!
//! Moved here from the `cyber-mudra` crate on 2026-10-07: identity is a field
//! of the neuron, so its authority code lives with the subject. mudra keeps the
//! confidentiality specifications (seal / stealth / veil / quorum).
//!
//! The implemented native secp256k1 profile identifies a neuron by
//! `Hemera(compressed public key)`. [`neuron`] owns its bounded NSIG1 statement
//! authentication; [`domain`] supplies separately domain-derived keys.
//! The proof-native/hash-preimage profile (`specs/proof-authority.md`) and
//! mudra's seal/stealth/veil/quorum retain their own specifications and are not
//! implemented by these APIs.
//!
//! **Phase 1 — the legacy-key bridge.** To migrate an existing Cosmos-SDK
//! network (spacepussy first), we must let a holder prove control of the
//! secp256k1 key they already have and bind it to a native neuron. That is the
//! one place classical signatures are structurally required. This crate
//! currently implements that bridge:
//!
//! - [`spell`]  — BIP-39 words → derived spell material → BIP-32/44 secp256k1 key (coin type 118),
//!   behind the `bridge` feature (on by default)
//! - [`cosmos`] — compressed pubkey → `ripemd160(sha256(pk))` → bech32 address
//! - [`claim`] — ADR-036 sign/verify of a `legacy address → native neuron` binding
//!
//! Verification here is native (in-process). Phase 2 re-expresses the same
//! secp256k1 + sha256 + ripemd160 verification as a nox program producing a
//! zheng proof, making the binding trustless. The Phase-1 code is the reference
//! implementation Phase 2 must match.
//!
//! **A second, permanent use of secp256k1.** [`domain`] derives a *fresh*
//! identity — no legacy account, no BIP-32 — for contexts where secp256k1
//! is chosen not to migrate anything, but because it's what a browser
//! already speaks natively (a wallet extension, or a few KB of JS). This
//! is not phase-1-bridge code that phase 2 retires: browsers stay
//! secp256k1-native regardless of what the rest of cyber's identity model
//! becomes.

pub mod claim;
pub use neuron_id::NeuronId;
pub mod cosmos;
pub mod domain;
pub mod neuron;
#[cfg(feature = "prove")]
pub mod proof;
#[cfg(feature = "bridge")]
pub mod spell;

pub use claim::Claim;

/// The secp256k1 signing key type used across neuron-auth. Depended on directly
/// (not through bip32's re-export) so it stays available with `bridge` off;
/// version-pinned to match bip32 0.5's own `k256` pin, so both resolve to
/// the same crate when `bridge` is on.
pub use k256::ecdsa::SigningKey;

/// Errors from the legacy-key bridge.
#[derive(Debug)]
pub enum Error {
    /// The BIP-39 spell could not be parsed.
    Spell(String),
    /// HD derivation (BIP-32/44) failed for the given path.
    Derive(String),
    /// Address encoding failed (bad HRP or bech32 error).
    Bech32(String),
    /// A key or signature was malformed.
    Key(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Spell(m) => write!(f, "invalid spell: {m}"),
            Error::Derive(m) => write!(f, "HD derivation failed: {m}"),
            Error::Bech32(m) => write!(f, "address encoding failed: {m}"),
            Error::Key(m) => write!(f, "malformed key or signature: {m}"),
        }
    }
}

impl std::error::Error for Error {}
