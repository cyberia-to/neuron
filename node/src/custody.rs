//! Local-operator Vault custody. LocalAuthority holds its action grant through sign.
use crate::SigningVault;
use neuron_engine::Error;
use neuron_model::{NeuronId, Particle};
use std::sync::Mutex;

pub struct LocalVault {
    subject: NeuronId,
    signer: Mutex<vault::local::Signer>,
}
impl LocalVault {
    pub fn new(signer: vault::local::Signer) -> Self {
        Self {
            subject: signer.subject(),
            signer: Mutex::new(signer),
        }
    }
}
impl SigningVault for LocalVault {
    fn subject(&self) -> NeuronId {
        self.subject
    }
    fn sign(&self, subject: NeuronId, statement: Particle) -> Result<Vec<u8>, Error> {
        if subject != self.subject {
            return Err(Error::Denied);
        }
        let mut signer = self.signer.lock().map_err(|_| Error::Denied)?;
        let request = vault::RequestId::random().map_err(|_| Error::Denied)?;
        signer.sign(request, statement).map_err(|_| Error::Denied)
    }
}
