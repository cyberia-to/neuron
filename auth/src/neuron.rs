//! Existing native H(pubkey) statement envelope. See specs/neuron-auth.md.
use crate::{Error, SigningKey, claim, cosmos};
const TAG: &[u8] = b"cyber:neuron:authority:v1:";
fn message(statement: [u8; 32]) -> Vec<u8> {
    let mut bytes = TAG.to_vec();
    bytes.extend(statement);
    bytes
}
pub fn sign(
    key: &SigningKey,
    subject: [u8; 32],
    statement: [u8; 32],
) -> Result<Vec<u8>, Error> {
    let public = cosmos::compressed(key.verifying_key());
    if claim::neuron_of(&public) != subject {
        return Err(Error::Key("key does not own requested subject".into()));
    }
    let address = cosmos::address(&public, "neuron")?;
    let signature = claim::sign_arbitrary(key, &address, &message(statement));
    let mut proof = b"NSIG1".to_vec();
    proof.extend(public);
    proof.extend(signature);
    Ok(proof)
}
/// Compatibility for existing native-profile consumers. New callers use `sign`.
pub use sign as sign_statement;

pub fn verify_statement(subject: [u8; 32], statement: [u8; 32], evidence: &[u8]) -> bool {
    if evidence.len() != 102 || &evidence[..5] != b"NSIG1" {
        return false;
    }
    let Ok(public) = <&[u8; 33]>::try_from(&evidence[5..38]) else {
        return false;
    };
    let Ok(signature) = <&[u8; 64]>::try_from(&evidence[38..]) else {
        return false;
    };
    if claim::neuron_of(public) != subject {
        return false;
    }
    let Ok(address) = cosmos::address(public, "neuron") else {
        return false;
    };
    claim::verify_arbitrary(public, &address, &message(statement), signature)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_subject_message_domain_and_exact_envelope_are_bound() {
        let key = SigningKey::from_bytes((&[7; 32]).into()).unwrap();
        let public = cosmos::compressed(key.verifying_key());
        let id = claim::neuron_of(&public);
        let proof = sign(&key, id, [8; 32]).unwrap();
        assert!(verify_statement(id, [8; 32], &proof));
        assert!(!verify_statement(id, [9; 32], &proof));
        assert!(!verify_statement([9; 32], [8; 32], &proof));
        assert!(sign(&key, [9; 32], [8; 32]).is_err());
        for n in [0, 4, 5, 38, 101] {
            assert!(!verify_statement(id, [8; 32], &proof[..n]));
        }
        let mut trailing = proof.clone();
        trailing.push(0);
        assert!(!verify_statement(id, [8; 32], &trailing));
        for offset in [0, 5, 38, 101] {
            let mut changed = proof.clone();
            changed[offset] ^= 1;
            assert!(!verify_statement(id, [8; 32], &changed));
        }
        let address = cosmos::address(&public, "neuron").unwrap();
        let raw_signature = claim::sign_arbitrary(&key, &address, &[8; 32]);
        let mut raw = b"NSIG1".to_vec();
        raw.extend(public);
        raw.extend(raw_signature);
        assert!(!verify_statement(id, [8; 32], &raw));
    }
}
