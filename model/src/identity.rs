//! Public identity records and bounded attachment projections. Crypto stays in mudra.
use crate::{NeuronId, Particle};
use std::collections::BTreeMap;

pub const MAX_DOMAIN_BYTES: usize = 128;
pub const MAX_ADDRESS_BYTES: usize = 256;
pub const MAX_ATTACHMENTS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    InvalidReference,
    Limit,
    Missing,
    Conflict,
    Denied,
}
impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "neuron identity: {self:?}")
    }
}
impl std::error::Error for IdentityError {}

fn domain_valid(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= MAX_DOMAIN_BYTES
        && domain
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.:/".contains(&c))
}

/// An observed reference proves neither control nor cross-domain equivalence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubjectRef {
    Native(NeuronId),
    Foreign { domain: String, address: Vec<u8> },
}
impl SubjectRef {
    pub fn validate(&self) -> Result<(), IdentityError> {
        match self {
            Self::Native(_) => Ok(()),
            Self::Foreign { domain, address }
                if domain_valid(domain)
                    && !address.is_empty()
                    && address.len() <= MAX_ADDRESS_BYTES =>
            {
                Ok(())
            }
            _ => Err(IdentityError::InvalidReference),
        }
    }
    pub fn native(&self) -> Option<NeuronId> {
        match self {
            Self::Native(id) => Some(*id),
            Self::Foreign { .. } => None,
        }
    }
}

/// Destination network, separate from subject derivation or transport endpoints.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NetworkRef {
    Native(Particle),
    Foreign { domain: String, network: Vec<u8> },
}
impl NetworkRef {
    pub fn validate(&self) -> Result<(), IdentityError> {
        match self {
            Self::Native(_) => Ok(()),
            Self::Foreign { domain, network }
                if domain_valid(domain)
                    && !network.is_empty()
                    && network.len() <= MAX_ADDRESS_BYTES =>
            {
                Ok(())
            }
            _ => Err(IdentityError::InvalidReference),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Access {
    Observe,
    Control,
    Delegated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BindingState {
    Attached,
    Suspended,
    Detached,
    Revoked,
}

/// Public binding evidence; accepting this record is a host/ward responsibility.
/// Deserialization alone never establishes a verified signing capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Binding {
    pub subject: SubjectRef,
    pub network: NetworkRef,
    pub revision: u64,
    pub access: Access,
    pub state: BindingState,
    pub policy: Particle,
    pub evidence: Option<Particle>,
}
impl Binding {
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.subject.validate()?;
        self.network.validate()?;
        if self.access != Access::Observe && self.evidence.is_none() {
            return Err(IdentityError::Denied);
        }
        Ok(())
    }
}

/// Immutable admission selection. Current permissions are checked at each dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionContext {
    pub subject: SubjectRef,
    pub network: NetworkRef,
    pub binding_revision: u64,
    pub prog: Option<Particle>,
    pub invocation: Option<Particle>,
    pub policy: Particle,
    pub grant: Particle,
}
impl ActionContext {
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.subject.validate()?;
        self.network.validate()?;
        if self.prog.is_some() != self.invocation.is_some() {
            return Err(IdentityError::InvalidReference);
        }
        Ok(())
    }
}

/// Rebuildable projection of admitted graph bindings; no database or ambient signer.
#[derive(Debug, Default)]
pub struct Attachments {
    bindings: BTreeMap<(SubjectRef, NetworkRef), Binding>,
}
impl Attachments {
    pub fn get(&self, subject: &SubjectRef, network: &NetworkRef) -> Option<&Binding> {
        self.bindings.get(&(subject.clone(), network.clone()))
    }
    pub fn iter(&self) -> impl Iterator<Item = &Binding> {
        self.bindings.values()
    }
    /// Apply a verified graph transition. Host verifies evidence before calling.
    pub fn apply(&mut self, expected: Option<u64>, next: Binding) -> Result<(), IdentityError> {
        next.validate()?;
        let key = (next.subject.clone(), next.network.clone());
        let prior = self.bindings.get(&key);
        if prior == Some(&next) {
            return Ok(());
        }
        if prior.map(|p| p.revision) != expected
            || next.revision
                != match expected {
                    None => 0,
                    Some(n) => n.checked_add(1).ok_or(IdentityError::Limit)?,
                }
        {
            return Err(IdentityError::Conflict);
        }
        if prior.is_none() && self.bindings.len() >= MAX_ATTACHMENTS {
            return Err(IdentityError::Limit);
        }
        self.bindings.insert(key, next);
        Ok(())
    }
    /// Structural/current-binding check, followed by the host's live ward decision.
    pub fn authorize_context(&self, context: &ActionContext) -> Result<&Binding, IdentityError> {
        context.validate()?;
        let binding = self
            .get(&context.subject, &context.network)
            .ok_or(IdentityError::Missing)?;
        if binding.access == Access::Observe
            || binding.state != BindingState::Attached
            || binding.revision != context.binding_revision
            || binding.policy != context.policy
        {
            return Err(IdentityError::Denied);
        }
        Ok(binding)
    }
}

/// Navigation addresses data/views without manufacturing another subject.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Destination {
    Neuron(SubjectRef),
    Prog { subject: SubjectRef, prog: Particle },
    Particle(Particle),
    View(String),
}
impl Destination {
    pub fn validate(&self) -> Result<(), IdentityError> {
        match self {
            Self::Neuron(subject) | Self::Prog { subject, .. } => subject.validate(),
            Self::Particle(_) => Ok(()),
            Self::View(name) if domain_valid(name) => Ok(()),
            Self::View(_) => Err(IdentityError::InvalidReference),
        }
    }
}
