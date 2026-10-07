// ---
// tags: neuron, auth, rust, example
// crystal-type: source
// crystal-domain: comp
// ---
//! End-to-end spacepussy migration bridge: a spell → its real `pussy1…`
//! account → a native neuron → a signed, verified claim binding the two.
//!
//!   cargo run --example migrate

fn main() {
    // A holder's wallet phrase (canonical BIP-39 test vector — never use for value).
    let spell =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // 1. spell → the exact secp256k1 key that controls their spacepussy account
    let key = neuron_auth::spell::cosmos_key(spell, "").expect("derive key");
    let pubkey = neuron_auth::cosmos::compressed(key.verifying_key());
    let address = neuron_auth::cosmos::address(&pubkey, neuron_auth::cosmos::PUSSY).expect("address");

    // 2. the deterministic native neuron this account maps to: Hemera(pubkey)
    let neuron = neuron_auth::claim::neuron_of(&pubkey);

    // 3. the holder signs a claim binding the legacy address → the native neuron
    let claim = neuron_auth::claim::create(&key, neuron_auth::cosmos::PUSSY, neuron).expect("claim");

    // 4. anyone verifies it — no trusted party, just the signature and the address
    let ok = neuron_auth::claim::verify(&claim, neuron_auth::cosmos::PUSSY);

    println!("spacepussy account : {address}");
    println!("pubkey (compressed): {}", hex(&pubkey));
    println!("native neuron      : {}", hex(&neuron));
    println!("signature          : {}…", &hex(&claim.signature)[..24]);
    println!("claim verifies     : {ok}");
    assert!(ok);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
