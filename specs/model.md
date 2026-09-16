---
title: neuron model
tags: neuron, soft3, spec
status: accepted
spec-version: "0.3"
---
# Model

One neuron is one protocol subject. Its optional runtime installs programs and
maintains durable invocations under the same authority. A loaded program, view,
service endpoint or graph partition does not automatically acquire a key.

| Entity | Identity and role |
|---|---|
| Neuron | Native NeuronId or explicit foreign-domain reference; the acting subject |
| Robot | Name/configuration attaching neurons and devices; no mandatory root signer |
| Binding | Current subject/network/policy/custody/device association with revision |
| Prog | Data ID for an installation under a neuron; pinned source and mutable state revision |
| Invocation | Data ID for admitted work with its original input, context and allowance |
| Operation / attempt | Stable effect and dispatch-attempt records, not accounts |
| Checkpoint / artifact | Content-addressed continuation/data with declared codec |
| Worker / host / device | Execution placement and observation, independent of subject |
| GraphSession | Local graph containing many subject chains; no signing identity |
| Name / endpoint | Typed discovery/presentation and transport; neither proves control |

Subject/domain/network and native key behavior are normative in [identity](identity.md).
A native neuron can have several bindings across compatible networks. The current
runtime namespace pins one execution network; incompatible execution roots under
the same subject use an explicitly separate database/profile. There is no global
mutable current-neuron inside the library.

## Activation and program installation

Native activation commits `(neuron, snapshot, authorization)` at index zero in
the subject namespace. Its commit particle is data and is not the subject ID.
Authorization binds the actual existing key identity, network, policy and root.
Reactivation of the same subject cannot reset its budget or replace its history.
Observation-only bindings need no activation or program installation.

Prog ID derives from `(subject, installation nonce)` under the versioned schema.
Two installations of the same source have separate state, without separate keys.
The local Rune profile validates source/initial state and commits an Active prog
atomically. Its record contains source artifact, state artifact, revision,
lifecycle, step limit, max in-flight jobs and sorted allowed-act requests.
Requests do not grant those acts. Code and admitted dependencies are immutable;
resolving a floating model/source name at resume cannot replace their revision.

The generic release contract additionally declares evaluator/ABI, input/output
and checkpoint schema, required artifacts, requested capabilities and resource
limits. A profile must reject unsupported required extensions. A pure read/view
may evaluate against a retained snapshot without creating a persistent prog.
Publishing a release and attaching a signing subject remain separate actions.

## Runtime state

The exact current field order is [runtime suite v1](../model/schema-suite-neuron-v1.txt).
`neuron/root/1` contains subject, execution network, policy, authority epoch,
aggregate limit/charged/held, maps of progs, invocations and legacy origins,
a scheduling cursor, writer generation and optional worker descriptor.

Each invocation pins prog, admitted source, input, optional application context,
base state revision, checkpoint, status, limit/charged/reserved/used, pending
operation, result/fault, parent/children/delegated allowance, epoch and ordinal.
The current persisted statuses are running, waiting, joining, completed,
cancelled, failed and conflict. Queued/runnable and unknown-outcome are derived
conditions; they do not invent additional persisted status discriminants.

Maps are bounded and sorted with unique keys. Current bounds are 256 progs,
1024 invocations, 4096 origins, 256 direct children and depth 64. Terminal jobs can
leave the live map only after their claims/outcomes remain durable. Immutable
artifacts are shared; no duplicate private model/history store is introduced.
Optional future indexes must preserve ordering, closure and the declared suite.

## Admission, effects and commits

A request key derives from `(neuron, nonce)`. Admission content binds prog, input,
context, epoch, parent and allowance; its event particle is the invocation ID.
Exact retry resolves the original claim even after archive. Reusing a nonce with
different content or another prog conflicts. Management requests share the
namespace and cannot reinterpret an admission nonce as another operation kind.
Application context is inert data; the host supplies authority independently.

Each commit binds subject, monotonically increasing index, exact predecessor,
before/after roots, event, changed records and authorization. Publication compares
the subject head atomically. Competing proposals cannot both become authoritative.
Independent computations can run concurrently; adopting a result also requires
its admitted prog state revision. A stale result is retained as conflict, and an
external effect is not repeated to rebase it.

Operation ID binds subject/prog/invocation/ordinal. The operation fixes exact
arguments, network, policy/epoch and executor contract. An attempt is persisted
before physical dispatch; a crash can leave its outcome unknown. Result and
consumption claims remain addressable after the pending slot clears. The
[execution contract](execution.md) governs recovery and limits.

## Distinct orderings and profiles

Neuron commit index, prog state revision, invocation ordinal, worker generation,
SignalChain step, native coordinator position and network height have distinct
meanings. They MUST NOT be substituted for each other. Foculus/native coordinator
owns the public signal sequence; one runtime commit need not publish one Signal.
An execution commit alone establishes no network or economic finality.

A shard owns a region of graph state under its protocol; a book records a token's
obligations; a service has application governance. These are data/protocol roles,
not additional runtime subjects. Independent authority may require another
neuron, while independent state/lifetime only requires another prog/task.

A filtered replica retains predecessor closure and declared evidence. A sparse
selection cannot be fed into a complete-chain writer and called full history.
Historical unsigned rows and legacy origins preserve their original attribution;
[migration](migration.md) never turns a birth hash into verified key ownership.

Every acknowledged durable head must retain its state and required resumption
closure. Possessing, querying or rendering those bytes grants no dispatch rights.
A program retires independently of the neuron and of the robot attachment.
