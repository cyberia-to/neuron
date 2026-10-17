//! Frozen mudra 348c46195388ac009667144987511a0e33bdc859 outputs.
//! Capture source hashes are recorded by soft3's neuron convergence audit.
use neuron_auth::{claim,domain::DomainKey};
fn hex(b:&[u8])->String {b.iter().map(|b|format!("{b:02x}")).collect()}
#[test]
fn domain_addresses_native_ids_adr036_docs_and_signatures_match_legacy_bytes(){
    let rows:serde_json::Value=serde_json::from_str(include_str!("fixtures/domain-v1.json")).unwrap();
    for row in rows.as_array().unwrap(){
        let key=DomainKey::derive(&[7;32],row["domain"].as_str().unwrap(),row["hrp"].as_str().unwrap()).unwrap();
        let body=row["body"].as_str().unwrap().as_bytes();
        assert_eq!(key.bech32,row["address"]);assert_eq!(hex(&key.pubkey),row["pubkey_hex"]);
        assert_eq!(hex(&key.native),row["native_hex"]);
        assert_eq!(claim::adr036_doc(&key.bech32,body),row["adr036_doc"]);
        assert_eq!(hex(&claim::sign_arbitrary(key.signing_key(),&key.bech32,body)),row["signature_hex"]);
    }
}
