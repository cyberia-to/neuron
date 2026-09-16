---
title: history
tags: neuron, cybergraph, bbg, spec
status: accepted
spec-version: "0.3"
---
# History and persistence

Cybergraph owns application history and its write interface. BBG supplies shared
storage, transaction boundaries and retention. Neuron validates typed execution
records and publication. Cyb Log renders this history; Tape frames transport or
legacy projections. A second per-organ canonical database is unnecessary.

## Records and order

Activation, prog state, invocations, commits, attempts, outcomes, checkpoints,
management and migration records are immutable content-addressed records. The
neuron namespace's selected head points at the current root. Previous/before/
after/event references preserve the execution chain and its content closure.
[Data](data.md) defines exact codecs and suite identities.

An activation particle identifies a record, while NeuronId identifies its
subject. Runtime commit indices, Foculus SignalChain steps, worker generations
and network finality are separate orderings. Application records in a shared BBG
database do not automatically become public Signals or consensus commitments.
A network publication uses that network's authenticated action/admission contract.

Task context, child lineage, observations and learning proposals use application
namespaces over this same storage. Indexes and views are rebuildable projections
with explicit scope, schema and source head. An opaque transcript alone cannot
provide the causal task/context records required by the agent contract.

## Atomic publication

The native GraphPort accepts namespace, stable request identity, expected head,
candidate head, immutable content and optional admission claim. `commit_once`
provides fresh-publication semantics: resolving an old receipt cannot become a
new effect permit. `resolve` returns an existing request's original head.

The adapter must:

1. Validate canonical content, record closure, subject signature, network, policy,
   epoch, predecessor and bounded state/resource changes.
2. Hold current host authorization through the publication boundary.
3. Compare the expected head and atomically retain content, request/nonce claim,
   history position and new head in BBG.
4. Satisfy the backend's durability barrier before returning durable success.
5. Dispatch or notify only from the selected, sufficiently durable result.

Validation that may reject state application precedes irreversible mutation or
runs inside an abortable transaction. Rejected insertion leaves ordering at the
predecessor. The complete selected commit or the unchanged predecessor is the
recovery result; a half-advanced authoritative projection is invalid.

A lost commit reply is resolved by request identity. `CommitUnknown` stops
work that depends on knowing the selected state. Staged unreferenced content may
survive an abort as cache data; acknowledged referenced bytes must remain
recoverable. Persistence batching must preserve each receipt's own boundary.

## Database ownership and compatibility

The CLI's default directory is `bbg`, backed by the configured shared BBG database.
A host may pass its existing Database to the application adapter. The database's
exclusive process ownership and CAS coordinate all local views. Separate adapters
to the same owner share coordination; cloning a view does not create a writer.

An implicit old `cell.redb` blocks opening an empty default store. Explicit store
selection and the feature-gated backend conversion preserve the old file. Semantic
[legacy import](migration.md) then maps old namespaces to authenticated subjects
and progs, retaining their bytes, claims and evidence. Source fences and monotonic
reader generations stop old binaries from continuing the retired histories.
Separate-store transfer seals the source before bounded copying and uses an
explicit transfer reader; staging remains inert until validated activation.

GraphSession imports the old cyb Signal log through its own strict coordinator,
retains the old source, and shares native graph/history and local-credit storage.
TextArchive imports old text with exact source/hash/ordinal provenance. These
product formats have distinct migration contracts; a legacy author label never
becomes a newly authenticated NeuronId merely by import.

## Durability and evidence

A production durable profile requires recoverable content and backend error
propagation. A test/volatile backend is explicitly selected and must not silently
replace failed persistent storage. Local durability, local authority, a host
observation, remote endpoint acceptance and verified consensus finality establish
different facts. [Evidence](evidence.md) defines their interpretation.

The current local native profile independently verifies its authenticated
application commits. It does not manufacture consensus certificates. NativeMirror
retains source network, cursor and observation provenance; mirror acceptance is
not a proof that all remote consensus rules were applied.

## Reading, replay and cursors

Reads identify the actual selected head and supported scope. Bounded history
pages use committed positions. A gap, corrupt record, missing artifact or
unsupported reader is an explicit error, never an empty history or proof of
absence. Replay validates retained identities and predecessor relationships
without repeating old external effects. A checkpoint accelerates replay only
when its relationship to selected history and required content is verified.

Admission and outcome claims remain resolvable after terminal invocation archive.
Exact duplicates return the existing result; changed content under the same
request conflicts. Local ingress receipt, neuron admission and external completion
remain distinct. A host observation cannot advance a foreign authoritative head.

A composing subscription must bind snapshot/head and scan cursor, retain consumer
progress before acknowledgement and expose backpressure/gaps. Current native
HTTP mirror/outbox cursors use their specific bounded page contracts; the engine
GraphPort does not advertise a generic live subscription service.

## Retention, export and privacy

A resumable closure includes code/runtime/checkpoint schemas; state and context
artifacts; live input, operation, attempt, outcome and consumption records; task/
child ownership; charged/reserved resources; policy/evidence; and a verified
history boundary. Legacy origins additionally retain original records and claims.
A self-contained backup includes the required bytes. A thin backup declares its
external availability dependencies; a particle hash alone cannot recreate data.

Archive removes eligible terminal work from bounded live maps while preserving
history and deduplication claims. Pending effects and unsettled children pin their
closure. Any future history compaction must declare a verified retained boundary
and preserve outstanding obligations before deleting data. A learned summary
retains source references and labels missing/pruned sources explicitly.

Export follows access and encryption policy, including metadata, causation and
routing references. Private notes and vault records remain encrypted under their
own contracts; execution grants are rebound from current policy after restore.
Physical deletion changes local availability and cannot retract existing replicas.
