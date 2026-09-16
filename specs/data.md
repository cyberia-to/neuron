---
title: neuron data and schemas
tags: neuron, soft3, spec
status: accepted
spec-version: "0.3"
---
# Data and identity

Neuron uses the existing nox atom/pair data codec and Hemera structural/content
identity. Tape supplies framing; application records add no kernel data variant
or new hash function. Raw binary encodings are defined by the checked-in suites
and their readers, not by Rust type names or debug formatting.

## Canonical values

| Value | Current logical encoding |
|---|---|
| uint | pair(high32, low32), checked u64 |
| nonce | fixed eight-u32 list, each word from four little-endian input bytes |
| ref | balanced pair tree of four little-endian u64 limbs of the 32-byte reference |
| text | bounded UTF-8 bytes represented by the suite's byte atoms |
| optional | pair(0,0) absent; pair(1,value) present |
| sequence | pair(uint count, right-nested fields ending in atom zero) |
| map | sorted unique reference-key/value pairs in a bounded sequence |
| record | pair(ref(schema manifest), ordered fields) |
| byte artifact | content hash plus codec, exact byte length and media type |

Numeric/status/boolean refinements are record-specific; integers are not silently
truncated. Text preserves exact UTF-8 bytes without case folding/normalization.
Map/reference ordering uses unsigned lexicographic ID bytes; sets reject duplicate
or unsorted values. Semantically ordered sequences retain their order. Decoders
reject extra/missing fields, noncanonical limbs, bad terminators, invalid UTF-8,
overflow and lengths beyond bounds before allocating declared sizes.

Data content validates with `nox::encode::particle_of`; Blob content validates
with unchanged Hemera bytes hashing. The same ID cannot refer to different codec
or bytes in one proposal. A framing conversion must decode to the same value/ID.
Foreign address bytes belong to their own domain-qualified reference contract;
arbitrary address strings are never hashed into native NeuronId by a reader.

## Suites and manifests

[Original suite](../model/schema-suite-v1.txt) is immutable legacy compatibility.
[Neuron suite](../model/schema-suite-neuron-v1.txt) defines current record field
order and validation. Its exact UTF-8 bytes participate in every schema identity:

```text
manifest(name) = pair(atom(0x53434831), fields(text(name), ref(blob(suite bytes))))
record(name, values...) = pair(ref(manifest(name)), fields(values...))
```

A name alone does not bind semantics. The reader selects the neuron suite for
`neuron/` names and the retained suite for historical records. New incompatible
fields/contracts require a new suite and explicit reader/migration; changing a
Rust module name never rewrites old schema particles, signatures or checkpoints.
Unknown required schemas fail admission. Archival forwarding may preserve opaque
content but cannot authorize execution or assert successful verification.

| Current record group | Semantic contract |
|---|---|
| activation/root/prog/event/commit/transition and ID records | [model](model.md), [runtime v1](runtime-v1.md) |
| operation/attempt/outcome/consumed/pending | [execution](execution.md) |
| authorization/authority statement | [local authority](local-authority.md) |
| worker/placement/dispatch | [worker dispatch](worker-dispatch.md) |
| import/migration and original claims | [migration](migration.md) |
| signed action envelope | [action envelope](action-envelope.md); separate bounded canonical JSON |
| robot, Soma context/task/schedule and private wallet/vault catalogs | owning cyb/Soma schemas, referenced as data |

The older Definition/Birth/Snapshot records are legacy reader inputs. New writers
use native activation and prog/invocation records; they do not create a new birth
subject. Exact suite text lists all auxiliary records, their ordered fields and
refinements; it is part of this specification.

## Derived IDs

All `particle(...)` formulas use canonical suite records:

- NeuronId: existing identity-profile derivation, never particle(activation).
- ProgId: particle(neuron/prog-id/1(subject, installation nonce)).
- Request key: particle(neuron/request-id/1(subject, nonce)).
- InvocationId: the admitted neuron/event/1 particle binding all input fields.
- CommitId: particle(neuron/commit/1(...)); activation is its initial head.
- OperationId: particle(neuron/operation-id/1(subject, prog, invocation, ordinal)).
- AttemptId: particle(neuron/attempt-id/1(operation, number, epoch)).

These work/data IDs create no signing authority. Retrying a committed operation
keeps its original identity. New attempts need the declared executor idempotency
or reconciliation contract and current grant. Legacy birth/commit/operation IDs
remain unchanged references under their old suite. Signal position hashes retain
their upstream meaning and cannot replace full-content duplicate comparison.

## Bounds and resource semantics

Current Builder content count is at most 131072; individual content/artifacts are
at most 8 MiB and text metadata is at most 4096 bytes. Readers additionally bound
node traversal, collections, runtime artifacts and per-operation input. Database
transaction/page limits can be stricter than the sum of individually valid
artifacts; a host must report that limit without acknowledging a partial record.
Runtime map/step bounds are in [runtime v1](runtime-v1.md); host/provider limits
remain explicit in their contracts. Checked cumulative accounting survives restart.

Time points identify their clock/units/evidence. Local wall time is an observation
and cannot reorder a committed chain. Resource metrics identify units, scope and
aggregation: steps, memory, input/output, queued work, graph accesses, provider
tokens and monetary costs are not interchangeable. A claimed hard limit requires
an enforcing adapter; estimates and cooperative deadlines disclose their scope.

Every codec release requires positive/negative vectors, including field order,
absent versus empty, UTF-8, canonical words, collection ordering and malformed
lengths. Migration verifies old fixtures without recalculating their identities.
Missing code does not prevent bounded inspection of retained records/receipts.
