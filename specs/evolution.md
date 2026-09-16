---
title: evolution
tags: neuron, prog, migration, spec
status: accepted
spec-version: "0.3"
---
# Evolution

Evolution preserves subject identity, authenticated lineage, current authority
and outstanding obligations. Program revision, worker placement, node storage
mode and signing identity are independent changes. Management is ordered with
application transitions through the same authorized graph publication boundary.

## Program upgrade

Native `upgrade` validates replacement Rune source/state, stores their artifacts
and increments the prog revision under CAS. The caller supplies the replacement
state explicitly. A state-transforming migration must produce that candidate
under its own validated application contract; the engine does not pretend to run
an unspecified migration entry automatically.

Live invocations keep their admitted code, input, context and continuation.
Their old state revision prevents adoption over the replacement state. Already
attempted effects retain their exact identity, authority and observations. An
upgrade cannot erase unknown outcomes or restart them under changed code.
A host may pause and settle work before upgrade when the application requires a
clean cutover. Retiring/retired progs reject upgrade.

A future continuation migration must bind old/new runtime and checkpoint schemas,
validate complete mappings and preserve inbox/outbox, child, budget and result
obligations in one authorized transition. Unsupported mappings leave the original
work inspectable and suspended. New rights require a current ward decision;
changing code alone does not widen the allowed-act grant.

A failed candidate leaves selected state recoverable. Lost replies are resolved
against retained history before retry. Returning to older code is another forward
revision with explicit state; external outcomes remain in history.

## Placement and transfer

Workers execute bounded work under the subject's current authority and guarded
one-use attempt claim. Many workers/readers can serve one subject while the shared
local database orders its publications. A worker device/boot label is distinct
from NeuronId and does not create a new signer. Fresh generations prevent old
permits from becoming valid after worker replacement.

For the supported separate-store legacy transfer, [migration](migration.md) and
BBG's transfer contract require an available exclusive source lock, durable seal,
validated manifest, bounded copy cursor, complete content closure, current target
authority and final source/target fences. Staging is inert. The retained source
remains sealed after interruption; ordinary old/new writers cannot reopen it.

General live-neuron relocation requires a supported placement protocol that:

1. Stops new source dispatch at a bounded resumable boundary.
2. Retains or definitely settles all attempts, children and resource obligations.
3. Verifies checkpoint/artifact closure and the selected source head.
4. Fences old publication and physical dispatch at their enforcing boundaries.
5. Transfers and validates the data, then binds current destination custody/grants.
6. Activates the destination only after the cutover is authoritative.

An in-process lock covers only that process with exclusive database ownership.
An unreachable source needs an independently enforceable fencing protocol before
same-subject consequential execution can resume elsewhere. Copying files or using
a local-clock lease alone cannot establish that condition. Current sealed legacy
transfer is not a general partition-tolerant distributed relocation service.

## Backup and restore

A self-contained backup binds subject/network, selected head/history boundary,
complete required artifacts, evidence, privacy and source placement status. It
includes state/code/checkpoints, context, task/children, attempts/outcomes,
consumption, claims and charged/held budget. A thin backup declares the content
that depends on external availability. A rendered Log or seed alone is incomplete.

Use a quiescent backend snapshot or its supported export protocol. Live directory
copying has no atomicity guarantee. Vault/private-state material follows its own
explicit encrypted export/recovery contract. Keys and grants are not restored by
ordinary execution content. Current policy and writer/dispatch fencing are
verified again before restored work can act.

Missing or incompatible content is reported explicitly. Historical inspection
may proceed over verified available data, while execution waits for a complete
resumable closure. Model caches may be rebuilt only under the pinned adapter's
semantics; changing provider/model is an explicit new application request.

An execution checkpoint does not restore an external filesystem, message or
remote transaction. Compensation is a new authorized operation with observed
current revisions. It preserves later user edits and reports partial/conflicting
results under the workspace contract.

## Copy, fork, split and merge

Copying a prog creates another installation data ID under its chosen neuron.
It can isolate mutable state without new keys. A fork requiring independent
signing authority uses an explicitly created/attached neuron with separate grants
and source lineage. Pending effects are historical references, not newly
inherited dispatchable work. Merely copying a database cannot safely fork the
same subject's writer/effect authority.

Shard split/merge, service deployment and token-book migration belong to their
[domain roles](../../cyber/specs/domain-ladder.md). Their protocols must bind all
source heads, ownership allocation, content, pending economic/service obligations,
authorities, routing cutover, conservation and recovery. A spectral cut proposes
a partition; it does not itself authorize the cutover or create new neuron keys.
Every affected authority admits the corresponding protocol, and unavailable
semantics fail closed before a partially owned state can be published.

## Compatibility

Identity profile, data codec, runtime/checkpoint semantics, program revision,
application schema, authority and finality are versioned independently. Supported
migrations name exact source versions and retain readers for served history.
The original `cell/*/1` suite is immutable legacy data. The native neuron v1 suite
has separate identities. Unknown runtime artifacts can be inspected as content
without being executable; renaming source packages never rewrites signed bytes.
