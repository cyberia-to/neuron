---
title: cell model
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# model

A cell is an addressable owner of bounded state with explicit admission,
transition, authority, lifecycle and finality rules. A runtime-cell supplies
behavior through a loaded definition and grows an ability of cyb.

## entities and names

| Entity | Meaning |
|---|---|
| Definition | Immutable release particle describing behavior and dependencies |
| Instance | State/history owner identified by its birth particle, CellId |
| Replica | A host's retained copy at a declared cell head and evidence level |
| Host | Environment placing instances and connecting stack capabilities |
| Neuron | Principal that authors or authorizes changes |
| Name | Resolvable graph binding to a typed target |
| Endpoint | Transport location of a host |

A host MUST distinguish locally executable instances from observed replicas.
Immutable particles may be shared across cells. Mutation authority belongs to
the instance's governing policy. Neither possession of bytes nor physical
containment grants that authority.

A name target MUST declare definition, instance or host kind. An instance
resolution returns CellId plus the resolution's provenance and freshness.
Code publication, instance naming and endpoint discovery are independently
versioned operations. A caller pins the resolved release for an invocation.

## definition

Definition fields, in canonical order:

| Field | Type / meaning |
|---|---|
| revision | uint; 1 for this model |
| runtime | particle identifying evaluator semantics and ABI |
| code | artifact reference |
| state_schema | particle identifying application state representation |
| entries | sorted list of Entry records |
| required_artifacts | sorted artifact references |
| requested_authority | particle of ward's declarative request document |
| resource_contract | particle of ResourceContract |
| checkpoint_schema | optional particle of runtime checkpoint representation |
| presentation | optional artifact reference for prysm view binding |
| extension_requirements | sorted schema/protocol particles |

Entry = (name: text, input_schema: particle, output_schema: particle,
behavior: event | read | migration | view). Entry names are unique.
Entries are sorted by the exact UTF-8 bytes of name. An event entry proposes
transitions; read/view entries operate on a fixed
snapshot and cannot mutate cell state. A migration entry has evolution's rules.

Publication is a separately authenticated record binding author and definition.
The release identity covers its executable dependencies. Runtime code loading
MUST resolve pinned dependencies; a floating name cannot substitute code during
an invocation. Definition requests constrain the grant negotiation, not grant it.

## birth and initial state

Birth = (revision, nonce, initial_authority, definition, profile,
initial_snapshot, lineage).

- nonce is a fresh 256-bit value, persisted before retrying birth publication;
- initial_authority is a particle of the governing policy;
- profile names a complete admission/finality/evidence contract;
- lineage is absent or (source CellId, source Head, purpose particle);
- initial_snapshot is a Snapshot particle with lifecycle Installed and epoch 0.

CellId is the particle of canonical Birth. Authentication binds that complete
particle separately, avoiding a circular identifier/signature dependency.
The initial Head is (index 0, commit CellId). The initial Snapshot MUST agree
with Birth's definition, policy and profile. CellId remains stable thereafter.
Ephemeral test instances also use a birth; their receipts state volatile storage.

## snapshot

Snapshot = (definition, application_state, lifecycle, authority_policy, profile, epoch,
inbox, continuations, outbox, subscriptions, management).

All fields except lifecycle and epoch are particle references. Empty collections
use canonical empty values with their collection schema. A collection is a
sorted key/value map; values are typed record particles.
management holds ordered upgrade/relocation/retirement requests and their state.
Snapshot identity covers all resumable execution bookkeeping.
The birth schema omits CellId from initial data so identity is acyclic.
Later application data may refer to its containing CellId.

The collection values are:

- InboxEntry = (event, status, invocation, last_index), where status is admitted,
  suspended, completed or cancelled and invocation is an optional EventId.
- OutboxEntry = (operation, stage, attempts, outcome, consumed_by), where attempts
  is an ordered list of Attempt references; outcome and consumed_by are optional
  Outcome and invocation references. Stages are defined in execution.md.
- Continuations map continuation particles to their records; subscriptions map
  subscription particles to (record, last_cursor, status).
- ManagementEntry = (request, record, status, last_index); status is pending,
  blocked, completed or rejected. record is an Upgrade/Relocation/management
  Event reference. last_index identifies the commit index applying this status.

Values refer to preceding records or independent content. They MUST NOT contain
the containing CommitId as a prerequisite for calculating that same commit.
Event/operation deduplication remains available through history after completed
entries leave the live Snapshot; retention boundaries are explicit.

Application state is immutable data identified by a particle and interpreted by
state_schema. External graph reads bind a snapshot/head and evidence; they become
recorded inputs or witnesses if they affect a transition.

## events and commits

Event = (origin, nonce, destination, entry, payload, causation,
observed_at, deadline, authority_reference).

origin is an authenticated principal; nonce is unique within that origin.
destination is CellId; entry is text; payload is a particle.
causation is a sorted list of event/operation/commit references with explicit kind.
observed_at and deadline use TimePoint from data.md and may be absent.
EventId is its canonical particle. Same (origin, nonce) with different content
is a conflict. An origin persists and reuses its nonce on retry.

Commit = (cell, index, previous, before, after, events, operations,
authority_decision, executor_epoch, evidence_requirements).

previous is the previous commit particle. before/after are Snapshot particles.
events and operations are ordered lists of record particles, including management
and outcome records. Every commit has at least one event. index advances by one,
with overflow rejected. CommitId is its canonical particle.
Authentication/evidence envelopes bind CommitId and remain outside its identity.

A Head is (index: uint, commit: particle). A committed head identifies exactly
one Snapshot. Competing content at the same index is a conflict even when
transport, signing neuron or an upstream chain-position hash is identical.
An application transition, act outcome or management event uses this same path.

## cell and protocol ordering

Each instance has a total order of authoritative commits selected by its profile.
Different cells have independent Head counters. Existing protocol SignalChains
retain their per-neuron ordering; a single writer coordinator allocates their
step/previous values and commits through cybergraph.

Cell commit index MUST NOT be substituted for a SignalChain step. One graph
Signal MAY publish several references according to graph atomicity and cost
rules. Application schema tags stay in particle data; token fields retain their
protocol economic meanings.

Replicas of a filtered cell history need cell predecessor closure and evidence.
They cannot feed a sparse selection into a full per-neuron append API and claim
a complete SignalChain. Full-chain verification requires the missing records
or the upstream protocol's authenticated range proof.

Independently settling network cells MUST declare their network protocol,
writer scope and finality adapter. The baseline preserves existing chain formats;
new domain-scoped chains require a versioned change in their owning protocols.

## invariants

- One CellId has one birth and one selected authoritative history per profile.
- Every authoritative commit binds its previous head, state and admitted events.
- Every mutation is admitted under the governing policy at the relevant head.
- The state and required resumption data of a durable head are recoverable.
- Replica verification and possession never confer execution authority.
- No cell directly overwrites another's state; the receiver admits requests.
