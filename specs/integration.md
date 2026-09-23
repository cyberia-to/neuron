---
title: neuron integration
tags: neuron, soft3, spec
status: accepted
spec-version: "0.3"
---
# Integration and ownership

One repository preserves the former implementation's Git history and contains
several crates. It does not wrap an independently identified runtime component.

| Crate | Responsibility |
|---|---|
| neuron-id | Common native ID byte contract, no dependencies |
| neuron-model | Subject/binding/navigation; optional execution record codecs |
| neuron-engine | Admission, program/task state, budgets, effects and recovery through ports |
| neuron-rune | Existing Rune evaluator/checkpoint adapter |
| neuron-node | Graph publication validation, shared BBG adapter, ward/vault and worker composition |
| neuron-cli | Headless inspection, execution, migration and reconciliation |

Model without default features imports only the ID crate (plus explicit serde
when requested). BBG uses neuron-id, never engine/node. Identity clients remain
independent of GUI, storage and VM. Optional node/query projects execution data
into inf; the dependency points from the host to the query interface, not from
inf to its executor. GUI views belong to prysm/cyb, not a second neuron renderer.

## Owners

| Owner | Mechanism |
|---|---|
| cybergraph | Canonical graph history, native coordinator, application validation/publication/query and head CAS |
| bbg/storage | Atomic durable records, closures, claims, receipts and storage barriers |
| hemera / lens | Content/structural identity and authenticated commitments |
| foculus | Signal ordering and named consensus/finality profiles |
| tape / radio | Framing and transport; neither supplies ambient action authority |
| rune / nox / glia / wysm | Their declared evaluator/runtime/proof contracts |
| zheng | Execution proof generation/verification under a named supported profile |
| ward / vault / mudra | Current permission decisions / scoped secret operations / authentication primitives |
| body and workers | Device placement, resource enforcement/measurement and process supervision |
| name / state | Typed name resolution / external-state evidence and freshness |
| soma | Cognition, model/tool decisions, goals, child strategy and evaluated learning |
| soul / now / com | Configuration / contextual anchor / captured asks and steering |
| sigma | Asset and neuron-attachment presentation, monetary accounting |
| plan / sense | Standing orders / correlated conversations and delivery |
| log / time | History rendering / past-present-future presentation |
| brain / memory | Graph rendering / particle filesystem projection |
| cyb | Product assembly, bodies, interaction and common Host/Registry |

A storage WAL or compatibility signal log is a physical record format, not another
organ or independent history owner. GraphSession contains many subject chains;
its name is deliberately not Neuron. Views and task catalogs reuse the common
Database and reference original content.

## Composition contracts

- Graph acceptance validates before atomic publication; failures return definite
  rejection or an explicit unresolved commit. No success is inferred from memory.
- Content required by acknowledged state/checkpoints remains in the same durable
  closure; state-only sidecar writes are not sufficient.
- Rune host authority is independent of mutable subject data and survives nested,
  sequential and resumed calls. Runtime-declared caps are requests.
- Worker admission binds machine/environment/proof/network/executor, device/boot
  and generation. Current grants remain guarded through publication/handoff.
- Duplicate delivery compares complete original content. Gap, equivocation,
  invalid signature and missing source are distinguishable failures.
- NativeMirror/relay and application cursors advance only through durably accepted
  observations/receipts. Unknown effects are never skipped or automatically rerun.
- Soma captures versioned context, schemas, memory disclosure, model/provider and
  workspace. Tasks, child joins and schedules use retained neuron invocations.
- Private vault/note data retains its existing cipher/key derivation and explicit
  subject/network namespace. Public history never contains seed/key/plaintext notes.
- Every advertised runtime/proof/transport combination must declare actual limits,
  evidence and recovery; a trait stub or status label does not establish support.

## Migration sources and product paths

The two old cyb graph wrappers converge into one cyb-core GraphSession over the
existing native coordinator. Old graph.log/particles.jsonl files are strict
migration inputs; shared graph/archive APIs retain them and retire old append
entry points. The true-cyber client uses the same owner. Lost public-key provenance
in old analytics remains unresolved history, not a fabricated native identity.

Legacy birth/state/checkpoint/outbox data is translated by the
[migration contract](migration.md), with immutable old schema bytes, explicit
subject/prog mappings and source/target fences. Backend conversion is independent
of semantic import. A source is not deleted when target acceptance is uncertain.

Cyb terminal/Bevy and headless Soma use shared Host/Registry and agent composition.
Loaded Rune evaluation uses the existing evaluator and prysm chunks. Navigation
to a neuron, prog or page does not create an account or execute source. Historical
URI aliases resolve only through the bounded compatibility adapter.

A robot extension is a prog. Skills may be instructions/artifacts interpreted by
Soma; executable skills install code under an actual subject. Services can use
several programs; book/issuer and shard/validator rules remain domain protocols.
A new signing neuron requires independent authority/attribution, not merely an
independent process or stateful feature.

## Acceptance

Local durable execution, authenticated native delivery and the bounded local Soma
profile have separate owner tests and composition evidence. Distributed writers,
foreign control, consensus proofs and full provider/channel parity require their
own gates. Unsupported required profiles fail before effects. The
[conformance contract](conformance.md) defines behaviors; actual revisions,
feature sets, fault probes and consumer vectors belong to
[the audit](../../soft3/audit/neuron-cell/implementation.md).
