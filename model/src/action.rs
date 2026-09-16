//! Public bound action data. Neither decoding nor possession grants authority.
use crate::{
    Particle,
    identity::{ActionContext, IdentityError},
};
pub const MAX_ACTION_PAYLOAD: usize = 8 * 1024 * 1024;
pub const MAX_ENVELOPE_BYTES: usize = 8 * 1024 * 1024;
pub const ACTION_DOMAIN: &str = "cyb/robot-action/1";
pub const ENVELOPE_SCHEMA: &str = "neuron/signed-action/1";

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct ActionRequest {
    pub request: Particle,
    pub attachment: Particle,
    pub context: ActionContext,
    pub kind: String,
    pub payload: Vec<u8>,
}
impl ActionRequest {
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.context.validate()?;
        if self.kind.is_empty()
            || self.kind.len() > 128
            || !self.kind.bytes().all(|b| b.is_ascii_graphic())
            || self.payload.len() > MAX_ACTION_PAYLOAD
        {
            return Err(IdentityError::Limit);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct SignedAction {
    pub schema: String,
    pub action: ActionRequest,
    pub evidence: Vec<u8>,
}
impl SignedAction {
    pub fn new(action: ActionRequest, evidence: Vec<u8>) -> Result<Self, IdentityError> {
        let envelope = Self {
            schema: ENVELOPE_SCHEMA.into(),
            action,
            evidence,
        };
        envelope.validate()?;
        Ok(envelope)
    }
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.action.validate()?;
        if self.schema != ENVELOPE_SCHEMA
            || self.evidence.len() != 102
            || &self.evidence[..5] != b"NSIG1"
        {
            return Err(IdentityError::InvalidReference);
        }
        Ok(())
    }
    #[cfg(feature = "serde")]
    pub fn encode(&self) -> Result<Vec<u8>, IdentityError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| IdentityError::InvalidReference)?;
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(IdentityError::Limit);
        }
        Ok(bytes)
    }
    #[cfg(feature = "serde")]
    pub fn decode(bytes: &[u8]) -> Result<Self, IdentityError> {
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(IdentityError::Limit);
        }
        let envelope: Self =
            serde_json::from_slice(bytes).map_err(|_| IdentityError::InvalidReference)?;
        if envelope.encode()? != bytes {
            return Err(IdentityError::InvalidReference);
        }
        Ok(envelope)
    }
}
#[cfg(feature = "serde")]
pub fn statement_bytes(action: &ActionRequest) -> Result<Vec<u8>, IdentityError> {
    action.validate()?;
    serde_json::to_vec(&(ACTION_DOMAIN, action)).map_err(|_| IdentityError::InvalidReference)
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use crate::identity::{NetworkRef, SubjectRef};
    fn action() -> ActionRequest {
        ActionRequest {
            request: [1; 32],
            attachment: [2; 32],
            context: ActionContext {
                subject: SubjectRef::Native([3; 32]),
                network: NetworkRef::Native([4; 32]),
                binding_revision: 5,
                prog: None,
                invocation: None,
                policy: [6; 32],
                grant: [7; 32],
            },
            kind: "native/signal".into(),
            payload: vec![8, 9],
        }
    }
    #[test]
    fn network_envelope_has_one_bounded_encoding_and_no_implicit_authority() {
        let mut evidence = b"NSIG1".to_vec();
        evidence.extend([0; 97]);
        let envelope = SignedAction::new(action(), evidence).unwrap();
        // Structural parsing accepts a well-formed but unverified proof; caller must verify mudra.
        let bytes = envelope.encode().unwrap();
        assert_eq!(SignedAction::decode(&bytes).unwrap(), envelope);
        let mut spaces = bytes.clone();
        spaces.push(b' ');
        assert!(SignedAction::decode(&spaces).is_err());
        let mut unknown = serde_json::to_value(&envelope).unwrap();
        unknown["action"]["context"]["unknown"] = true.into();
        assert!(SignedAction::decode(&serde_json::to_vec(&unknown).unwrap()).is_err());
        let statement = statement_bytes(&envelope.action).unwrap();
        let mut different = envelope.action.clone();
        different.context.network = NetworkRef::Native([5; 32]);
        assert_ne!(statement_bytes(&different).unwrap(), statement);
        different.context.prog = Some([1; 32]);
        assert!(different.validate().is_err());
        different = envelope.action;
        different.kind = "two words".into();
        assert!(different.validate().is_err());
        different.kind = "native/signal".into();
        different.payload = vec![0; MAX_ACTION_PAYLOAD + 1];
        assert!(different.validate().is_err());
    }
}
