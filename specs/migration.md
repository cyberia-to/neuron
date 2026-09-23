---
title: cell v1 to neuron migration
tags: neuron, migration, storage, spec
status: accepted
spec-version: "0.3"
---
# Cell v1 → neuron

The [operator guide](../docs/legacy-cutover.md) gives the retained-source inspect,
export, activation and reconciliation sequence. Exact supported CLI bounds and
the logical archive digest contract are owned by the
[CLI](cli.md) and [BBG transfer](../../bbg/specs/application-transfer.md) specs.

Migration preserves immutable bytes and maps legacy origin namespaces to an
authenticated neuron and distinct prog bindings. It is neither a type alias
from birth hash to public-key identity nor a rewrite of historical authorship.

## Inspection and manifest

An import manifest pins source store/profile, every legacy namespace and exact
head, target subject/network, authority evidence, installation mapping and content
hashes. Multiple legacy origins may target one subject; their states, histories,
nonce claims, policies and resource records remain distinct. Ambiguous mappings
are rejected. An absent/unverified target grants inspection/export only.

Inspection walks all required structural and explicit artifact references with
bounds, verifies canonical hashes/schema/profile, and reports missing/corrupt data.
It includes live input, continuation codec, pending operation/attempt/outcome,
consumption and resource reservation. Inspection dispatches no work.

## Durable activation and fencing

Imports operate through the existing shared BBG database and retain the original
history/content. Staging batches and a manifest are resumable, idempotent records.
Activation validates all source heads and the target head again under the same
exclusive writer transaction, then installs the target root and origin mappings.
Concurrent changes, incomplete content or authority mismatch leave the target
inactive and preserve source state.

Prepared authorization does not survive revocation as permission to activate.
The local ward holds its current-grant read guard over the final source/target
head transaction. Revocation either precedes activation (which rejects) or
follows its durable commit. The imported allowed-act union is checked again.
An Authority without an atomic current-grant guard cannot activate an import.
An already committed exact import retry remains a read and needs no fresh grant.

The old source namespaces become read-only at activation. This is enforced at
the BBG application writer boundary, not solely by the new CLI. Older binaries
must also fail normal open of an activated store: use a recognized storage
generation marker that old Database::validate_open rejects, and that the new
database recognizes explicitly. The existing application migration status slot
provides that compatibility boundary; neuron activation uses its own distinct
marker, not the ordinary redb-import "complete" marker.

Activation changes the writer generation and mapping atomically. New implementations
reject writes to fenced legacy namespaces even after reopening. A receipt retry
can still resolve historical success. Live physical database copying is not a
backup; use a quiescent store/export. A retained separate source cannot remain an
active effect dispatcher after cutover; its normal entry path is retired and
explicit legacy read/export remains available.

## Translation

Legacy code/state becomes a prog. Each live continuation becomes an invocation
with the exact input/context/code/checkpoint profile and outstanding budget.
Legacy event/operation/attempt identities remain valid correlation keys. Unknown
attempts stay unknown; resolved but unconsumed outcomes retain their single
consumer. Terminal receipts and admission nonce claims stay discoverable.

Old local-owner authority is recorded as legacy local provenance. A new supported
subject authorizes the mapping and future work; no signature is attributed to the
past. Changes of author/network are new authorized actions. Incompatible runtime
checkpoints remain suspended with a precise incompatibility error and retained
data, rather than being reset or treated as completed.

Charged resources remain charged; held/reserved work does not become free. Source
allowances map into the target accounting with checked aggregation. Insufficient
target budget prevents activation. All imported programs and invocations have
explicit lifecycle states and preserve independent responsibilities.

## Recovery and reconciliation

Before activation, failure leaves the old source authoritative and staging inert.
After activation, recovery resolves the committed manifest/receipt before writes.
Rerunning the same import returns the same mappings; changing source or mappings
under its request identity conflicts. Faults at every batch, validation and
activation boundary are recoverable without duplicate work.

After new records or external dispatch exist, downgrade is not a directory swap:
use forward recovery. Unknown external effects require executor receipts or
explicit reconciliation; do not claim general exactly-once for arbitrary APIs.
Archive/retention is a separate explicit policy; migration publishes no private
history to a network and deletes no source evidence.

The old redb→Fjall import remains a separate backend conversion preceding this
semantic migration where needed. The fixed legacy-v1.capture baseline from the
pre-migration engine is independent input for compatibility/recovery tests.

## Separate source store

The graph adapter uses BBG's [sealed application transfer](../../bbg/specs/application-transfer.md)
to import a separate legacy SSD store into the already-open shared Database.
Explicit source/target paths, target subject and a stable transfer key are
required. The source process lock must be available. Transfer first seals the
source, then reserves inert target namespaces and copies bounded pages with an
atomic cursor. Old content/history/requests/claims retain their bytes. Completion
validates the full archive, including historical records outside the live root.
Only then may the normal authenticated `import` activate the mapped progs.

The seal is durable source retirement: normal writers cannot reopen it. Recovery
uses the explicit transfer reader and the same manifest, which retains all source
data. Before semantic activation, cancellation leaves staging inert and source
read-only; it never silently resumes an old external-effect dispatcher.
