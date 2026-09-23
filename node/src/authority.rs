//! Local ward/vault adapter using mudra's existing native signing profile.
use mudra::{SigningKey, claim, cosmos};
use neuron_engine::{Action, Authority, Error};
use neuron_model::{NeuronId, Particle};
use std::{
    collections::BTreeSet,
    sync::{Arc, RwLock},
};

/// Opaque host custody; never serialized or exposed to program state.
pub trait SigningVault: Send + Sync {
    fn subject(&self) -> NeuronId;
    fn sign(&self, subject: NeuronId, statement: Particle) -> Result<Vec<u8>, Error>;
}
pub struct KeyVault {
    key: SigningKey,
}
impl KeyVault {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }
}
impl SigningVault for KeyVault {
    fn subject(&self) -> NeuronId {
        claim::neuron_of(&cosmos::compressed(self.key.verifying_key()))
    }
    fn sign(&self, subject: NeuronId, statement: Particle) -> Result<Vec<u8>, Error> {
        mudra::neuron::sign(&self.key, subject, statement).map_err(|_| Error::Denied)
    }
}
pub fn verify_evidence(subject: NeuronId, statement: Particle, evidence: &[u8]) -> bool {
    mudra::neuron::verify_statement(subject, statement, evidence)
}

#[derive(Clone, Debug)]
pub struct Grant {
    pub neuron: NeuronId,
    pub network: Particle,
    pub policy: Particle,
    pub epoch: u64,
    pub revision: u64,
    pub enabled: bool,
    pub acts: BTreeSet<u64>,
    pub progs: Option<BTreeSet<Particle>>,
}
#[derive(Clone)]
pub struct GrantHandle(Arc<RwLock<Grant>>);
impl GrantHandle {
    pub fn new(grant: Grant) -> Result<Self, Error> {
        if grant.acts.len() > 256 || grant.progs.as_ref().is_some_and(|p| p.len() > 256) {
            return Err(Error::Budget);
        }
        Ok(Self(Arc::new(RwLock::new(grant))))
    }
    pub fn get(&self) -> Result<Grant, Error> {
        self.0.read().map(|g| g.clone()).map_err(|_| Error::Denied)
    }
    pub fn replace(&self, expected: u64, next: Grant) -> Result<(), Error> {
        if next.acts.len() > 256 || next.progs.as_ref().is_some_and(|p| p.len() > 256) {
            return Err(Error::Budget);
        }
        let mut prior = self.0.write().map_err(|_| Error::Denied)?;
        if prior.revision != expected
            || expected.checked_add(1) != Some(next.revision)
            || prior.neuron != next.neuron
            || prior.network != next.network
            || next.epoch < prior.epoch
        {
            return Err(Error::Conflict);
        }
        *prior = next;
        Ok(())
    }
}
pub struct LocalAuthority<V> {
    vault: V,
    grant: GrantHandle,
}
impl<V: SigningVault> LocalAuthority<V> {
    pub fn new(vault: V, grant: GrantHandle) -> Result<Self, Error> {
        if vault.subject() != grant.get()?.neuron {
            return Err(Error::Denied);
        }
        Ok(Self { vault, grant })
    }
    pub fn subject(&self) -> NeuronId {
        self.vault.subject()
    }
}
impl<V: SigningVault> Authority for LocalAuthority<V> {
    fn authorize(&self, action: &Action<'_>) -> Result<Vec<u8>, Error> {
        let grant = self.grant.0.read().map_err(|_| Error::Denied)?;
        check(&grant, action)?;
        let evidence = self.vault.sign(action.neuron, action.statement)?;
        if !verify_evidence(action.neuron, action.statement, &evidence) {
            return Err(Error::Denied);
        }
        Ok(evidence)
    }
    fn with_current(
        &self,
        action: &Action<'_>,
        commit: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<(), Error> {
        let grant = self.grant.0.read().map_err(|_| Error::Denied)?;
        check(&grant, action)?;
        commit()
    }
}
fn check(grant: &Grant, action: &Action<'_>) -> Result<(), Error> {
    if !grant.enabled
        || action.neuron != grant.neuron
        || action.network != grant.network
        || action.policy != grant.policy
        || action.epoch != grant.epoch
        || action.act.is_some_and(|a| !grant.acts.contains(&a))
        || action.allowed_acts.iter().any(|a| !grant.acts.contains(a))
        || grant
            .progs
            .as_ref()
            .is_some_and(|p| action.prog.is_none_or(|id| !p.contains(&id)))
    {
        return Err(Error::Denied);
    }
    Ok(())
}
