//! Explicit filesystem-owner staging followed by authenticated semantic import.
use crate::{Graph, graph_error};
pub use cybergraph::application::{Transfer, TransferProgress};
use neuron_engine::{Action, Authority, Error, Neuron, RuntimePort};
use neuron_model::Particle;

pub fn stage_legacy<R: RuntimePort, A: Authority>(
    agent: &Neuron<Graph, R, A>,
    neuron: Particle,
    source: &std::path::Path,
    transfer_key: Particle,
    pages: usize,
) -> Result<TransferProgress, Error> {
    if pages == 0 || pages > 4096 {
        return Err(Error::Budget);
    }
    let state = agent.inspect(neuron)?.state;
    agent.authority.authorize(&Action {
        neuron,
        network: state.network,
        policy: state.policy,
        epoch: state.epoch,
        prog: None,
        invocation: None,
        act: None,
        kind: "stage-import",
        statement: transfer_key,
        allowed_acts: &[],
    })?;
    let transfer = Transfer::open(source, neuron, transfer_key).map_err(graph_error)?;
    transfer.stage(&agent.graph.0, pages).map_err(graph_error)
}
