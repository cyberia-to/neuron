---
title: cell data
tags: cell, soft3, spec
status: draft
spec-version: "0.2"
---
# data and identity

Cell uses soft3's data, pair, field and particle vocabulary. Structural
hashing/encoding is supplied by hemera and the stack data codec. Cell specifies
application schemas and their canonical values; tape supplies outer framing.

## canonical values

A typed record is pair(schema_particle, fields). A schema particle identifies an
immutable schema manifest binding the complete shape, field order, refinements,
semantic contract and dependency particles. Human names such as cell/commit/1
resolve to that manifest; hashing a name alone does not bind its meaning.
Fields are a right-nested list in the order specified by the owning contract,
terminated by atom zero. This nominal wrapper is ordinary stack data.
Refinement checking preserves the underlying data identity. Transport framing
and application schema names create no new kernel data variants or hash capacity.

| Logical value | Canonical representation |
|---|---|
| bool | atom 0 or 1 |
| uint | pair(high32, low32), both atoms in [0, 2^32); range [0, 2^64) |
| nonce | eight u32 atoms as a fixed list; each decodes four consecutive nonce bytes in little-endian order |
| particle | the stack's canonical 32-byte particle as four field atoms |
| bytes | pair(uint byte_count, list of byte atoms in [0, 256)) |
| text | bytes of valid UTF-8, preserving exact code points |
| optional T | pair(0, 0) for absent; pair(1, value) for present |
| list T | pair(uint count, right-nested list of values ending at atom 0) |
| map K,V | list of pairs sorted by canonical key order with unique keys |
| named variant | pair(schema particle for that variant, ordered field list) |

Text is never case-folded or Unicode-normalized during identity calculation.
Human name resolution may have its own normalization contract.
Particle order is unsigned lexicographic order of canonical particle bytes.
Text/map keys use unsigned lexicographic UTF-8 order; numeric keys use unsigned
numeric order. Sets use sorted unique lists. Semantically ordered lists preserve
order. Counts, bounds and terminators MUST match exactly.
A decoder rejects extra fields, invalid fields, unknown mandatory schemas,
duplicate map keys, unsorted sets, overflow and excessive allocation requests.

This is the canonical logical shape. Physical transfer MAY use a negotiated
lossless stack encoding, whose decoded value MUST have the same particle.
Implementations MUST use the selected hemera structural-identity suite; hashing
an arbitrary serialization or a debug string is not a substitute.
Byte artifact references use the stack's content encoding and carry its codec
identifier. Codec compatibility is a release gate in integration.md.

## schemas

The schema prefix is cell/ and initial schema names end in /1. A release pins
the manifests and their transitive dependencies as one schema suite. The base
manifest encoding is supplied by the stack schema contract; manifests reference
dependencies and omit their own identifier, allowing an acyclic bootstrap.
Compatibility requires identical manifest identities, rather than equal names.
These drafts reserve names; production manifest identities and codec fixtures
are an integration prerequisite. No compatible wire release exists yet.

Schema field order and meaning are defined by the owning contract:

| Names | Contract |
|---|---|
| definition, entry, birth, snapshot, event, commit, head, inbox-entry, outbox-entry, management-entry | model.md |
| artifact, time-point, resource-contract | this document |
| operation, attempt, outcome, continuation | execution.md |
| authority-context | authority.md; runtime-local opaque representation |
| envelope, receipt, cursor, subscription | communication.md |
| evidence, profile, finality | evidence.md |
| upgrade, relocation, backup | evolution.md |
| query, page, host-info, error | api.md; local API representation |

Application context, task, skill and budget-accounting manifests belong to their
owning organ/protocol suites. Their use as particle references adds no dependency
from cell model to soma's implementation. Persistent-index schemas likewise
belong to the selected stack data/storage suite.

An extension defines its own schema particle and acceptance contract. Unknown
required schemas fail admission. Opaque archival forwarding MAY preserve unknown
content, marked unverified; it cannot cause execution or an authoritative update.

## common values

Artifact = (content, codec, byte_length, media_type).
content and codec are particles, byte_length is uint, media_type is UTF-8 text.
Verified possession means the content bytes match codec, length and particle.
An artifact manifest itself is ordinary content-addressed data.

TimePoint = (clock, value, evidence).
clock is a particle naming the clock semantics; value is uint in that clock's
units; evidence is optional particle. Clocks compare only under the same declared
semantics or a verified conversion. Local wall-clock timestamps are observations.
Commit order never derives from wall-clock sorting.

ResourceContract = (limits, yield_interval, overflow_policy).
limits is a sorted map from metric particle to uint limit. Metrics required by
the baseline: compute_steps, memory_bytes, input_bytes, output_bytes,
pending_operations, queued_events, graph_read_bytes, graph_write_bytes,
graph_hops, graph_results. Each graph request also has an explicit deadline.
yield_interval counts runtime compute steps.
overflow_policy is reject or bounded_wait; bounded_wait includes a deadline.
Admission intersects definition limits with host policy and delegated budgets.
A runtime unable to enforce a required metric fails activation.

Metric manifests define units, scope and accounting. Integer counters use checked
arithmetic; money uses a named asset and atomic units. Agent budgets additionally
bind provider token/cost limits and child reservations as specified in agent.md.
Energy observations state their measurement method and uncertainty; an estimate
cannot be advertised as a physically enforced energy ceiling.

## derived identities

All formulas below mean the particle of a canonical nominal record:

- CellId = particle(Birth).
- EventId = particle(Event).
- CommitId = particle(Commit).
- OperationId = particle(cell/operation-id/1:
  CellId, base CommitId, triggering EventId, ordinal).
- AttemptId = particle(cell/attempt-id/1: OperationId, attempt_number, epoch).

An operation's ordinal is its zero-based order within the proposal at that base
head. Proposed work losing a head comparison cannot be dispatched. Retrying an
already committed operation preserves OperationId; each permitted new attempt
has a distinct AttemptId. This avoids a cycle between a commit and its operations.

Protocol chain-position hashes retain their upstream meaning. A duplicate check
uses the full referenced content plus cell/position binding.

## versions and limits

Schema changes create a new schema particle. Definition/runtime ABI revisions,
state migrations and wire codec revisions are independently declared.
The 0.2 draft changes context/query fields, resource reporting and schema binding
before any wire release.
Implementations MUST pin manifests rather than treating all /1 aliases as equal.
An implementation publishes supported schemas/codecs and concrete limits.
Oversized or unsupported input is rejected before allocating its declared size.
Required records and receipts remain inspectable when executable code is unavailable.

Every codec release must provide positive and negative fixtures: field order,
absent versus empty, byte boundaries, UTF-8, sorted collections and malformed
lengths. Hash vectors are produced with the selected upstream codec/suite and
verified independently before compatibility is advertised.
