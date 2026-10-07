// ---
// tags: neuron, auth, rust, test
// crystal-type: source
// crystal-domain: comp
// ---
//! Known-answer vectors for the legacy-key bridge, cross-checked against an
//! independent pure-Python BIP-32 secp256k1 derivation + BIP-173 bech32. If
//! these break, the spell→key→address pipeline diverged from real Cosmos.

#![cfg(feature = "bridge")]

#[cfg(feature = "prove")]
use neuron_auth::proof::ecdsa;
use neuron_auth::{claim, cosmos, spell};
#[cfg(feature = "prove")]
use sha2::{Digest, Sha256};

/// Canonical all-`abandon` BIP-39 phrase. Public test key — never holds value.
const SPELL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// Independently verified (pure-Python secp256k1 BIP-32, no shared crates):
const PUBKEY: &str = "024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62";
const PUSSY_ADDR: &str = "pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn abandon_spell_derives_the_verified_pussy_account() {
    let key = spell::cosmos_key(SPELL, "").unwrap();
    let pubkey = cosmos::compressed(key.verifying_key());
    assert_eq!(hex(&pubkey), PUBKEY, "HD-derived pubkey matches independent derivation");
    assert_eq!(cosmos::address(&pubkey, cosmos::PUSSY).unwrap(), PUSSY_ADDR);
}

#[test]
fn full_bridge_round_trips() {
    // spell → real account → native neuron → signed claim → verified
    let key = spell::cosmos_key(SPELL, "").unwrap();
    let pubkey = cosmos::compressed(key.verifying_key());
    let neuron = claim::neuron_of(&pubkey);
    let c = claim::create(&key, cosmos::PUSSY, neuron).unwrap();
    assert_eq!(c.address, PUSSY_ADDR);
    assert!(claim::verify(&c, cosmos::PUSSY));
}

#[test]
#[cfg(feature = "prove")]
fn in_stack_ecdsa_verifies_a_real_claim() {
    // The Phase-2 (Goldilocks-limb) ECDSA verifier must agree with the native
    // Phase-1 claim on a REAL claim. A claim signs ECDSA over sha256(ADR-036 doc),
    // so the digest z the equation checks is exactly that sha256.
    let key = spell::cosmos_key(SPELL, "").unwrap();
    let pubkey = cosmos::compressed(key.verifying_key());
    let neuron = claim::neuron_of(&pubkey);
    let c = claim::create(&key, cosmos::PUSSY, neuron).unwrap();

    let doc = claim::adr036_doc(&c.address, &claim::bind_message(&c.neuron));
    let z: [u8; 32] = Sha256::digest(doc.as_bytes()).into();
    let mut r = [0u8; 32];
    let mut s = [0u8; 32];
    r.copy_from_slice(&c.signature[..32]);
    s.copy_from_slice(&c.signature[32..]);

    assert!(ecdsa::verify(&c.pubkey, &r, &s, &z), "in-stack ECDSA agrees with the native claim");

    // and a forged neuron changes the doc → the same verifier rejects it
    let forged_doc = claim::adr036_doc(&c.address, &claim::bind_message(&[0xFFu8; 32]));
    let z_forged: [u8; 32] = Sha256::digest(forged_doc.as_bytes()).into();
    assert!(!ecdsa::verify(&c.pubkey, &r, &s, &z_forged), "forged binding rejected");
}
