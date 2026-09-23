#![allow(dead_code)]
use std::path::{Path, PathBuf};
use vault::local::{
    Signer,
    host::{Host, ReplicaConfig},
    owner::{Grant, Owner},
};
use vault::{Derivation, Entry, NeuronKeyRef, RequestId, SecretInput, SecretRef, Vault, VaultId};

pub const PASSWORD: &str = "synthetic-vault-unlock";
pub const ROOT: &str = "08080808080808080808080808080808";
pub const DOMAIN: &str = "example.test";

pub fn key() -> NeuronKeyRef {
    NeuronKeyRef {
        root: SecretRef([8; 16]),
        derivation: Derivation::Domain {
            domain: DOMAIN.into(),
            hrp: "bostrom".into(),
        },
    }
}
pub fn create(directory: &Path) -> PathBuf {
    let home = directory.join("vault");
    let host = Host::open(&home, true).unwrap();
    let mut state = host.state().unwrap();
    let ward = Owner {
        vault: VaultId(state.vault),
        context: state.context(),
        request: RequestId::random().unwrap(),
        grant: Grant::Create,
    };
    let mut vault = Vault::create(
        host.store.clone(),
        ward.vault,
        ward.request,
        ward.context,
        PASSWORD.as_bytes(),
        &[9; 32],
        &ward,
    )
    .unwrap();
    host.settle(vault.revision()).unwrap();
    let entry = Entry::new(
        key().root,
        "synthetic test".into(),
        "neuron".into(),
        ward.context.policy,
        false,
        SecretInput::generate_domain_root().unwrap(),
    )
    .unwrap();
    let ward = Owner {
        request: RequestId::random().unwrap(),
        grant: Grant::Put(entry.info().clone()),
        ..ward
    };
    vault
        .put(ward.context, ward.request, entry, None, &ward)
        .unwrap();
    host.settle(vault.revision()).unwrap();
    state = host.state().unwrap();
    for (name, id) in [("replica-a", 1), ("replica-b", 2)] {
        let path = directory.join(name);
        vault::local::io::private_dir(&path).unwrap();
        state.replicas.push(ReplicaConfig {
            id: [id; 32],
            path: path.canonicalize().unwrap(),
            failure_domain: name.into(),
        });
    }
    host.save(state).unwrap();
    home
}
pub fn open(home: &Path) -> Signer {
    Signer::open(home, key(), PASSWORD.as_bytes()).unwrap()
}
