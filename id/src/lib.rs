#![no_std]
//! Native protocol identity bytes, shared below all runtime/storage adapters.
//!
//! Derivation and authentication belong to an explicitly supported profile.
//! This alias preserves the existing 32-byte wire and serde representation;
//! possession of an identifier is never evidence of signing authority.

/// Native neuron subject identifier. Programs and graph namespaces are data.
pub type NeuronId = [u8; 32];
