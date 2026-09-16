# Execution relations for inf

The optional `neuron-node/query` adapter reads the existing canonical neuron
snapshot through the normal bounded reader and exposes immutable inf relations.
It owns the execution schemas; inf's evaluator remains independent of neuron
engine, VM, storage, custody and signing. No new database or write path exists.
The returned source is read-only and `provable() == false`: a validated local
snapshot is not an independently supplied Lens opening or finality certificate.
Its `snapshot()` is None because a neuron commit index is not a network height.

Every row begins `(neuron, network, commit, revision)`: unchanged native NeuronId,
native execution-network ID, exact canonical commit content ID and neuron commit
index. IDs are inf Hash values and counters are Word(u64). The column schema
provides their meaning; a hash value alone is not proof of identity or control.
Foreign addresses retain separate domain/address/network columns in their own
adapters and cannot enter this native-only projection by being hashed.

| Relation | Remaining columns |
|---|---|
| `neuron_runtime` | state_root, policy, epoch, limit, charged, held, writer_generation, worker? |
| `progs` | prog, source, state, state_revision, lifecycle, step_limit, max_inflight |
| `invocations` | invocation, prog, code, input, context?, base_revision, status, limit, charged, reserved, used, parent?, result?, fault? |
| `operations` | invocation, prog, operation, record, tag, arguments, attempt?, outcome?, stage, failed |
| `legacy_origins` | origin, prog |

Names/status values are UTF-8 Bytes; optional references use Null. Stage retains
the versioned operation numeric code. Progs and invocations identify work and
state, not additional signing subjects. `invocations` is the execution-task
projection; Soma's goal/model/schedule metadata remains its own catalog. No secret
material, artifact plaintext or authority capability is returned. Rows are sorted
and deduplicated by inf's existing relation container. Limits come from validated
NeuronState (256 progs, 1024 invocations/operations, 4096 origins). The source is
an owned snapshot and never changes when its underlying graph advances.

This is a host API over a database the caller already possesses. An application
exposing it to another principal must authorize the read and choose the subject
explicitly. Query data never grants permission to dispatch, mutate or sign.
