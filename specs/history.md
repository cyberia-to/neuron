---
title: history
tags: cell, soft3, spec
status: draft
spec-version: "0.2"
---
# history and persistence

Cybergraph is the authoritative history and write interface. Cell supplies
typed lifecycle/transition records and coordinates their admission. BBG and its
storage backends implement durable graph state and content retention.
The cyb log organ reads and presents this history.

## representation

Births, events, definitions, snapshots, commits, attempts, outcomes, checkpoints
and management records are particles. Cyberlinks establish their provenance,
membership and causation through the existing graph protocol.

At minimum an authenticated commit publication links the instance particle to
the Commit particle. The commit's canonical fields carry previous, before/after,
event and operation references. Referenced records and their artifact closure
are retained under the same privacy and availability contract.
Cell history indexes derive from these records; index corruption is repaired by
replay from an authenticated checkpoint/history boundary.

Application-context bindings, task lineage and learning evidence use this same
publication path. Record provenance and bounded semantic projections are part
of the owning application's contract. An opaque transcript blob alone cannot
satisfy the agent profile's contextual, causal and retrieval requirements.
Index roots pin their schema and scope; indexes remain rebuildable projections.

A Signal's economic fields retain cybergraph semantics. Cell tags and operation
kinds are expressed in application particle schemas. Publishing cell records
obeys the selected network's fees/focus/admission rules.

The graph API exposes a generic application namespace with opaque record
validation and conditional head updates. Cybergraph MUST NOT import cell's
runtime or renderer. Cell's projection and validator register through that
interface. App namespace isolation also applies to queries and subscriptions.

## durable commit boundary

The required GraphPort transaction accepts:

~~~text
transaction_id, namespace, expected_head, candidate_commit,
referenced_content, derived_head_update, required_durability
~~~

It either establishes the complete commit and its recoverable content closure
or leaves the authoritative head unchanged. The successful decision, source
records, outbox changes and head projection share one recovery boundary.
A store may implement this with a durable transaction log and replayable indexes.

Required ordering:

1. Validate authentication, cell policy, predecessor, schemas and resource bounds.
2. Stage immutable content and verify its identities.
3. Compare the authoritative predecessor and epoch at commit time.
4. Atomically record the selected graph change, recoverable references and head.
5. Satisfy the requested persistence barrier.
6. Publish the durable receipt and notify subscribers.

Storage MAY batch commits and persistence barriers while preserving per-cell
head selection, receipt order and durable-before-dispatch rules. Pending model
tokens or transient view deltas need no individual durable graph transaction.

All fallible validation that could reject state application must precede its
irreversible mutation, or run in an abortable transaction. A partially advanced
SignalChain with a rejected BBG insertion fails this contract.

Staged content may survive an aborted transaction as unreferenced cache data.
Acknowledged content cannot disappear through garbage collection.
A lost reply after commit is resolved by transaction_id/CommitId lookup.
An uncertain I/O result returns CommitUnknown and stops dependent effects until
lookup/recovery determines the selected history.

## local database ownership

The local CLI uses a BBG database directory named bbg by default, with Fjall
selected by Cybergraph's local-storage feature. The same BBG Database owner
may be passed to the local graph adapter; it owns backend transactions and the
writer lock shared with other storage views. Cell continues to publish through
GraphPort and does not own a separate storage engine.

If the old default cell.redb exists, an invocation without --store must stop
with an explicit migration or store-selection diagnostic. It must not silently
create a fresh session. An explicit --store directory selects that session.
The optional legacy-redb-migration build feature supplies
cell migrate-redb SOURCE DESTINATION. This command runs before normal graph
opening, imports the source through Cybergraph/BBG, preserves the old file and
requires a fresh destination. Partial imports remain unavailable. Backend
format errors and migration failures propagate to the caller. Default builds
do not enable redb.

## durability and finality

Durability levels are Volatile and LocalDurable. Volatile is explicit opt-in for
tests/throwaway sessions. A durable instance never silently falls back to it.
LocalDurable requires crash-recoverable bytes, storage error propagation and
the backend's data/directory persistence barriers for newly created storage.

Finality is an independent value defined by evidence.md. Network replication,
authorship, a content hash and local fsync make different claims.
A receipt states both durability and finality. Consequential operations require
LocalDurable plus the profile's operation finality gate.
Replication acknowledgements identify which content and heads were retained;
their claimed durability must be part of the replica's contract.

## ingestion, replay and queries

Ingress first authenticates and checks content identity and limits.
Exact duplicate content returns the existing result. A missing predecessor
returns Gap with the needed range. Different content for a selected position
returns Conflict and retains evidence according to the governing protocol.

Replay verifies the retained history boundary, definitions, cell predecessor
chain, outcomes and required evidence. It reconstructs projections without
performing past acts. A checkpoint accelerates replay only when its state
commitment and relationship to selected history are verified.

Corruption before an acknowledged boundary causes quarantine. An uncommitted
partial trailing frame may be discarded after validation establishes that it
was never acknowledged. Storage APIs must expose enough information to make
that distinction; decoder errors cannot be silently treated as an empty graph.

Reads specify a Head or request the latest selected head. Results report the
actual head, evidence and any incompleteness. History cursors refer to committed
cell positions, not file byte offsets. Rebuilding an index preserves cursor
meaning; retention may return HistoryUnavailable with the available boundary.

Local host ingress and placement records also live in graph namespaces.
A receipt for such a record means Received; cell admission/finality is a separate
receipt stage. A host observation cannot advance a foreign authoritative head.

## content closure and retention

A resumable checkpoint MUST retain:

- its birth, definition, runtime/checkpoint schemas and needed code;
- its Snapshot and application-state closure;
- application-context manifests and the source artifacts required to resume;
- pending input, operation, attempt, result and continuation records;
- evidence and governing policy references needed for verification;
- the authenticated history/checkpoint boundary needed for reconstruction.

Referenced bytes are included, durably retained locally, or backed by a declared
availability policy accepted for that replica. A self-contained backup includes
the bytes. A particle identifier alone is insufficient to recreate lost content.

A cell declares retention as full_history or checkpointed(policy particle).
Full history keeps every acknowledged record and required content.
Checkpointed history may compact content only after a verified checkpoint and
the policy's retention/availability obligations are met. Historical queries then
state their retained boundary. Pending external obligations pin their artifacts.

Graph access control and encryption apply before export. Indexes, causation,
addresses and receipts can leak private context and inherit the relevant scope.
Physical deletion changes availability; already published facts remain in the
selected history and in replicas that retain them.

A learned summary or skill records source references and retained evidence.
Compaction preserves pending obligations, explicit user constraints and source
availability declarations. Retrieval reports inaccessible or pruned sources;
it MUST NOT fabricate support from a summary whose originals are unavailable.

## current implementation status

The required GraphPort is a target integration contract. Current Cybergraph
applies in-memory state; cyb_core::Cell appends the tape; ShardStore lacks an
error-returning commit. These need integration and stronger failure semantics.
The target introduces no independent log-organ database.
