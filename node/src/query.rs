//! Read-only execution relations. Schemas: specs/query-projection.md.
use inf_source::LocalSource;
use inf_value::Value;
use neuron_engine::{Error, GraphPort, Neuron};
use neuron_model::{Lifecycle, NeuronId, Particle};
fn reference(id: Option<Particle>) -> Value {
    id.map(Value::hash).unwrap_or(Value::Null)
}
fn lifecycle(value: Lifecycle) -> &'static str {
    match value {
        Lifecycle::Installed => "installed",
        Lifecycle::Active => "active",
        Lifecycle::Paused => "paused",
        Lifecycle::Retiring => "retiring",
        Lifecycle::Retired => "retired",
    }
}
pub fn read<G: GraphPort>(graph: G, subject: NeuronId) -> Result<LocalSource, Error> {
    let view = Neuron::new(graph, crate::Rune).inspect(subject)?;
    let state = &view.state;
    let scope = vec![
        Value::neuron(subject),
        Value::hash(state.network),
        Value::hash(view.head.commit),
        Value::Word(view.head.index),
    ];
    let row = |fields: Vec<Value>| {
        let mut row = scope.clone();
        row.extend(fields);
        row
    };
    let mut source = LocalSource::new();
    let mut add = |name: &str, fields: &[&str], rows| {
        let mut columns = vec!["neuron", "network", "commit", "revision"];
        columns.extend(fields);
        source.add(name, &columns, rows);
    };
    add(
        "neuron_runtime",
        &[
            "state_root",
            "policy",
            "epoch",
            "limit",
            "charged",
            "held",
            "writer_generation",
            "worker",
        ],
        vec![row(vec![
            Value::hash(view.root),
            Value::hash(state.policy),
            Value::Word(state.epoch),
            Value::Word(state.limit),
            Value::Word(state.charged),
            Value::Word(state.held),
            Value::Word(state.writer_generation),
            reference(state.worker),
        ])],
    );
    add(
        "progs",
        &[
            "prog",
            "source",
            "state",
            "state_revision",
            "lifecycle",
            "step_limit",
            "max_inflight",
        ],
        state
            .progs
            .iter()
            .map(|(id, p)| {
                row(vec![
                    Value::hash(*id),
                    Value::hash(p.source),
                    Value::hash(p.state),
                    Value::Word(p.revision),
                    Value::str(lifecycle(p.lifecycle)),
                    Value::Word(p.step_limit),
                    Value::Word(p.max_inflight),
                ])
            })
            .collect(),
    );
    add(
        "invocations",
        &[
            "invocation",
            "prog",
            "code",
            "input",
            "context",
            "base_revision",
            "status",
            "limit",
            "charged",
            "reserved",
            "used",
            "parent",
            "result",
            "fault",
        ],
        state
            .invocations
            .iter()
            .map(|(id, i)| {
                row(vec![
                    Value::hash(*id),
                    Value::hash(i.prog),
                    Value::hash(i.code),
                    Value::hash(i.input),
                    reference(i.context),
                    Value::Word(i.base_revision),
                    Value::str(i.status.name()),
                    Value::Word(i.limit),
                    Value::Word(i.charged),
                    Value::Word(i.reserved),
                    Value::Word(i.used),
                    reference(i.parent),
                    reference(i.result),
                    reference(i.fault),
                ])
            })
            .collect(),
    );
    add(
        "operations",
        &[
            "invocation",
            "prog",
            "operation",
            "record",
            "tag",
            "arguments",
            "attempt",
            "outcome",
            "stage",
            "failed",
        ],
        state
            .invocations
            .iter()
            .filter_map(|(id, i)| {
                i.pending.as_ref().map(|p| {
                    row(vec![
                        Value::hash(*id),
                        Value::hash(i.prog),
                        Value::hash(p.id),
                        Value::hash(p.record),
                        Value::Word(p.tag),
                        Value::hash(p.arguments),
                        reference(p.attempt),
                        reference(p.outcome),
                        Value::Word(p.stage),
                        Value::Bool(p.failed),
                    ])
                })
            })
            .collect(),
    );
    add(
        "legacy_origins",
        &["origin", "prog"],
        state
            .imports
            .iter()
            .map(|(origin, prog)| row(vec![Value::hash(*origin), Value::hash(*prog)]))
            .collect(),
    );
    Ok(source)
}
