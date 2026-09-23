//! Neuron subject/binding semantics and optional canonical execution records.
//!
//! Disable default features for identity-only consumers: no VM, hash, storage,
//! renderer or model-provider dependencies are then linked.
pub mod action;
pub mod identity;
pub mod navigation;
pub use identity::*;
pub type Particle = [u8; 32];
#[cfg(feature = "records")]
pub mod data;
#[cfg(feature = "records")]
pub mod execution;
#[cfg(feature = "records")]
pub mod records;
#[cfg(feature = "records")]
pub mod worker;
#[cfg(feature = "records")]
pub use data::{Builder, Content, Error, Reader, Source};
pub use neuron_id::NeuronId;
#[cfg(feature = "records")]
pub use records::*;
