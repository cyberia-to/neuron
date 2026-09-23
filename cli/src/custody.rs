use crate::Result;
use clap::Args;
use neuron_model::{NeuronId, Particle};
use neuron_node::{KeyVault, SigningVault, custody::LocalVault};
use std::path::PathBuf;

#[derive(Args)]
pub struct Selection {
    /// Explicit legacy raw-key custody (mutually exclusive with Vault).
    #[arg(long, global = true, conflicts_with = "vault_home")]
    pub key_file: Option<PathBuf>,
    /// Encrypted Vault directory; configure its replicas with the Vault CLI.
    #[arg(long, global = true, requires = "vault_root")]
    vault_home: Option<PathBuf>,
    /// Spell or domain-root entry ID (16-byte hex).
    #[arg(long, global = true, requires = "vault_home")]
    vault_root: Option<String>,
    /// Cosmos derivation path; default m/44'/118'/0'/0/0.
    #[arg(
        long,
        global = true,
        requires = "vault_home",
        conflicts_with = "vault_domain"
    )]
    vault_path: Option<String>,
    /// Derive a domain-root entry for this domain.
    #[arg(long, global = true, requires = "vault_home")]
    vault_domain: Option<String>,
    /// Display address prefix; does not change native signing bytes.
    #[arg(long, global = true, requires = "vault_home")]
    vault_hrp: Option<String>,
    /// Read Vault unlock input from an explicitly trusted pipe.
    #[arg(long, global = true, requires = "vault_home")]
    vault_secrets_stdin: bool,
}
impl Selection {
    pub fn open(&self) -> Result<Option<Custody>> {
        if let Some(home) = &self.vault_home {
            let root = self.vault_root.as_deref().ok_or("select --vault-root")?;
            let hrp = self.vault_hrp.clone().unwrap_or_else(|| "bostrom".into());
            let derivation = match &self.vault_domain {
                Some(domain) => vault::Derivation::Domain {
                    domain: domain.clone(),
                    hrp,
                },
                None => vault::Derivation::Cosmos {
                    path: self
                        .vault_path
                        .clone()
                        .unwrap_or_else(|| mudra::spell::COSMOS_PATH.into()),
                    hrp,
                },
            };
            let key = vault::NeuronKeyRef {
                root: vault::SecretRef(vault::local::io::array(root)?),
                derivation,
            };
            let signer = vault::local::Signer::prompt_open(home, key, self.vault_secrets_stdin)?;
            Ok(Some(Custody::Vault(Box::new(LocalVault::new(signer)))))
        } else {
            self.key_file
                .as_deref()
                .map(crate::key)
                .transpose()
                .map(|v| v.map(Custody::Legacy))
        }
    }
}

pub enum Custody {
    Legacy(KeyVault),
    Vault(Box<LocalVault>),
}
impl SigningVault for Custody {
    fn subject(&self) -> NeuronId {
        match self {
            Self::Legacy(v) => v.subject(),
            Self::Vault(v) => v.subject(),
        }
    }
    fn sign(
        &self,
        subject: NeuronId,
        statement: Particle,
    ) -> std::result::Result<Vec<u8>, neuron_engine::Error> {
        match self {
            Self::Legacy(v) => v.sign(subject, statement),
            Self::Vault(v) => v.sign(subject, statement),
        }
    }
}
