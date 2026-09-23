//! Explicit migration operators: inspection never seals; export never executes.
use super::{Result, hex, particle};
use neuron_engine::{GraphPort, RuntimePort, legacy};
use neuron_model::Reader;
use neuron_node::{
    ArchiveGraph, Graph, Rune,
    archive::{MAX_INSPECTION_BYTES, MAX_INSPECTION_ROWS},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn existing_parent(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut parent = absolute.as_path();
    while !parent.try_exists()? {
        parent = parent
            .parent()
            .ok_or("destination has no existing ancestor")?;
    }
    Ok(std::fs::canonicalize(parent)?)
}
fn sizing(logical: u64, destination: Option<&Path>) -> Result<Value> {
    let planning = logical
        .checked_mul(4)
        .and_then(|n| n.checked_add(64 * 1024 * 1024))
        .ok_or("size overflow")?;
    let filesystem = destination.map(existing_parent).transpose()?;
    #[cfg(unix)]
    let available = filesystem
        .as_ref()
        .map(|p| {
            let stat = rustix::fs::statvfs(p)?;
            stat.f_bavail
                .checked_mul(stat.f_frsize)
                .ok_or(rustix::io::Errno::OVERFLOW)
        })
        .transpose()?;
    #[cfg(not(unix))]
    let available: Option<u64> = None;
    Ok(json!({"logical_bytes":logical,"planning_bytes":planning,
        "planning_rule":"4x-logical-plus-64MiB/1","filesystem":filesystem,
        "available_bytes":available,"planning_headroom_met":available.map(|n|n>=planning),
        "capacity_reserved":false,"physical_size_exact":false}))
}
pub(super) fn inspect(
    source: &Path,
    destination: Option<&Path>,
    max_rows: u64,
    max_bytes: u64,
) -> Result<Value> {
    let graph = ArchiveGraph::open(source)?;
    inspect_opened(&graph, source, destination, max_rows, max_bytes)
}
fn inspect_opened(
    graph: &ArchiveGraph,
    source: &Path,
    destination: Option<&Path>,
    max_rows: u64,
    max_bytes: u64,
) -> Result<Value> {
    let summary = graph.inspect(max_rows, max_bytes)?;
    let mut origins = Vec::new();
    for (origin, _) in graph.sources() {
        let view = legacy::inspect(graph, origin)?;
        let mut reader = Reader::new(graph, 100_000);
        let live = view
            .live
            .as_ref()
            .map(|live| -> Result<Value> {
                let checkpoint = reader.artifact(live.checkpoint)?;
                let incompatibility = Rune
                    .validate_checkpoint(&checkpoint, live.limit)
                    .err()
                    .map(|e| e.to_string());
                Ok(
                    json!({"invocation":hex(live.event),"context":live.context.map(hex),
                "checkpoint":hex(live.checkpoint),"checkpoint_compatible":incompatibility.is_none(),
                "incompatibility":incompatibility,"charged":live.charged,"reserved":live.reserved,
                "used":live.used,"limit":live.limit,"pending":live.pending.as_ref().map(|p|json!({
                    "operation":hex(p.id),"attempt":p.attempt.map(hex),"outcome":p.outcome.map(hex),
                    "stage":p.stage,"unknown":p.stage=="attempt-recorded"&&p.outcome.is_none()}))}),
                )
            })
            .transpose()?;
        origins.push(json!({"origin":hex(origin),"head":{"index":view.head.index,"commit":hex(view.head.commit)},
            "lifecycle":view.snapshot.lifecycle.name(),"source":hex(view.source),
            "state":hex(view.snapshot.application_state),"allowance":view.allowance,
            "allowed_acts":view.allowed_acts,"charged":view.charged,"live":live}));
    }
    let seal = graph.seal().map(|s| {
        json!({"target":hex(s.target),"nonce":hex(s.nonce),
        "source_transaction":hex(s.prior),"manifest":hex(s.manifest)})
    });
    Ok(
        json!({"schema":"neuron/legacy-source-inspection/1","source":std::fs::canonicalize(source)?,
        "validated":true,"source_sealed":seal.is_some(),"seal":seal,"last_transaction":graph.last_transaction().map(hex),
        "digest":hex(summary.digest),"digest_profile":"bbg/application-archive-rows/1",
        "rows":summary.rows,"logical_bytes":summary.bytes,
        "tables":{"content":summary.tables[0],"claims":summary.tables[1],"requests":summary.tables[2],"history":summary.tables[3],"heads":summary.tables[4]},
        "limits":{"rows":max_rows,"bytes":max_bytes},"sizing":sizing(summary.bytes,destination)?,
        "origins":origins,"dispatch":false}),
    )
}
pub(super) fn export(
    source: &Path,
    destination: &Path,
    target: &str,
    nonce: &str,
    pages: usize,
) -> Result<Value> {
    if pages == 0 || pages > 4096 {
        return Err("pages must be in 1..=4096".into());
    }
    let target_id = particle(target)?;
    let nonce_id = particle(nonce)?;
    let source_path = std::fs::canonicalize(source)?;
    let destination_path = if destination.try_exists()? {
        std::fs::canonicalize(destination)?
    } else {
        std::fs::canonicalize(
            destination
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )?
        .join(destination.file_name().ok_or("invalid destination")?)
    };
    if source_path.starts_with(&destination_path) || destination_path.starts_with(&source_path) {
        return Err("source and destination must be disjoint directories".into());
    }
    // Validation and seal share the same exclusive source handle. No writer can
    // change the inspected heads or archive bytes between these boundaries.
    let archive = ArchiveGraph::open(source)?;
    let report = inspect_opened(
        &archive,
        source,
        Some(destination),
        MAX_INSPECTION_ROWS,
        MAX_INSPECTION_BYTES,
    )?;
    if archive.sources().iter().any(|(id, _)| *id == target_id) {
        return Err("target subject collides with a legacy origin".into());
    }
    if archive
        .seal()
        .is_some_and(|s| s.target != target_id || s.nonce != nonce_id)
    {
        return Err("source is sealed for another target or transfer nonce".into());
    }
    // A missing/locked/invalid destination must not retire a fresh source.
    let graph = Graph::open(destination)?;
    if archive.seal().is_none() {
        for (origin, _) in archive.sources() {
            if graph.head(origin)?.is_some() {
                return Err("destination already owns a legacy origin namespace".into());
            }
        }
    }
    let transfer = archive.seal_for_transfer(target_id, nonce_id)?;
    let progress = transfer.stage(&graph.0, pages)?;
    Ok(
        json!({"schema":"neuron/legacy-export/1","source":source_path,"destination":destination_path,
        "target":hex(target_id),"nonce":hex(nonce_id),"manifest":hex(progress.manifest),
        "rows":progress.rows,"logical_bytes":progress.bytes,"complete":progress.complete,
        "source_sealed":true,"exported_namespaces_state":"inert-staged-archive","dispatch":false,
        "preflight":report}),
    )
}
