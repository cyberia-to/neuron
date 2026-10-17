// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Spells and hierarchical-deterministic keys.
//!
//! A holder's wallet is a BIP-39 spell. From it we derive the secp256k1 key
//! at the Cosmos derivation path so we can reproduce the *exact* on-chain
//! account a spacepussy neuron controls. This is how the bridge "computes real
//! neurons" from a spell.

use bip32::XPrv;

use crate::{Error, SigningKey};

/// The Cosmos-SDK derivation path — coin type **118**, account 0, external
/// chain, first address. spacepussy and bostrom both use this.
pub const COSMOS_PATH: &str = "m/44'/118'/0'/0/0";

/// Derive 64 bytes of spell material from words and an optional BIP-39 passphrase.
///
/// The passphrase (BIP-39 "25th word") is `""` for a standard wallet.
pub fn derive(spell: &str, passphrase: &str) -> Result<[u8; 64], Error> {
    let mn = bip39::Mnemonic::parse(spell).map_err(|e| Error::Spell(e.to_string()))?;
    Ok(mn.to_seed(passphrase))
}

/// Derive the secp256k1 signing key at `path` from derived spell material.
pub fn signing_key(spell: &[u8; 64], path: &str) -> Result<SigningKey, Error> {
    let path = path.parse().map_err(|e: bip32::Error| Error::Derive(e.to_string()))?;
    let xprv = XPrv::derive_from_path(spell, &path).map_err(|e| Error::Derive(e.to_string()))?;
    Ok(xprv.into())
}

/// Convenience: spell → signing key at the Cosmos path (coin type 118).
///
/// This is the one-call path from a holder's wallet phrase to the key that
/// controls their spacepussy account.
pub fn cosmos_key(spell: &str, passphrase: &str) -> Result<SigningKey, Error> {
    let material = derive(spell, passphrase)?;
    signing_key(&material, COSMOS_PATH)
}

/// Generate a fresh 24-word BIP-39 spell (256 bits of entropy). The phrase is
/// the sole key to the derived account and neuron — store it safely.
pub fn generate() -> Result<String, Error> {
    bip39::Mnemonic::generate(24)
        .map(|m| m.to_string())
        .map_err(|e| Error::Spell(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The canonical all-`abandon` BIP-39 test vector.
    const SPELL: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn derivation_is_deterministic() {
        let a = cosmos_key(SPELL, "").unwrap();
        let b = cosmos_key(SPELL, "").unwrap();
        assert_eq!(a.to_bytes(), b.to_bytes(), "same spell → same key");
    }

    #[test]
    fn passphrase_changes_the_key() {
        let a = cosmos_key(SPELL, "").unwrap();
        let b = cosmos_key(SPELL, "trezor").unwrap();
        assert_ne!(a.to_bytes(), b.to_bytes(), "the 25th word forks the wallet");
    }

    #[test]
    fn bad_spell_is_rejected() {
        assert!(cosmos_key("not a real spell phrase at all", "").is_err());
    }
}
