---
title: evolution
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# evolution

Evolution preserves authenticated lineage, state ownership and authority.
Management requests are events ordered with application transitions.

## upgrade

Upgrade = (request, target_definition, migration_entry, target_profile,
expected_head, requested_grants, deadline).

request is EventId; target_definition is a particle; migration_entry is optional
text naming a migration entry in that definition; target_profile is optional
Profile particle. requested_grants is a ward document particle.
expected_head and deadline constrain admission.

The host stages and verifies new code and dependencies before quiescing.
Quiescence waits for a bounded execution boundary and records the checkpoint.
Pending external attempts must be definitely resolved/cancelled or explicitly
supported by a migration adapter preserving their identities and semantics.
An unknown consequential attempt without such an adapter blocks upgrade.

Migration reads the old Snapshot and writes a candidate new application state
and checkpoint mapping. It runs with no consequential external acts. Its result
must preserve live inbox/outbox/subscription obligations, or explicitly settle
them under governing policy. It validates target state/checkpoint schemas.

One conditional graph commit changes definition, state, continuation mappings,
grants and optional profile together. Required new authority is negotiated before
activation. The activation proof/decision binds both versions and the resulting
state. The old definition remains active until that commit is authoritative.

Failure leaves the prior selected definition/state recoverable. Recovery of a
lost activation reply uses the upgrade request and committed head.
Returning to older code uses a new authorized forward transition and migration;
external outcomes remain in history.

## relocation and execution fencing

Relocation = (request, source_host, destination_host, base_head, next_epoch,
checkpoint, artifact_manifest, authority_transfer, deadline).

Hosts have authenticated identities. A placement authority grants at most one
consequential executor for an instance and epoch, unless its profile explicitly
specifies a multi-executor protocol. An in-process mutex is sufficient only for
one process with exclusive durable-store ownership.

The baseline supports graceful transfer:

1. Stop new dispatch on the source and reach a resumable boundary.
2. Resolve/cancel external attempts, retaining any supported pending operations.
3. Persist the checkpoint and complete artifact closure.
4. Record a release of the source execution epoch at the governing authority.
5. Transfer and verify data at the destination.
6. Commit a higher epoch, new placement and rebound grants.
7. Activate the destination; the source remains detached.

The new epoch must be checked by the authority/act executor that can actually
prevent old dispatch. The effect boundary verifies it for every attempt.
Already accepted external operations must be drained or reconciled before
exclusive execution is transferred.

If the old host is unreachable, forced takeover requires an independently
available fencing authority/protocol and compatible external executors.
Without it, recovery can inspect/compute speculatively but cannot resume
consequential effects under the same sole-executor claim.
An independent fork receives a new CellId and explicit new authorization.
A local-clock lease alone cannot establish safety across a partition.

## backup and restore

Backup = (cell, head, snapshot, history_boundary, artifacts, evidence,
privacy_policy, placement_status).

artifacts is a complete sorted manifest of required content; history_boundary
identifies the verified prefix/checkpoint from which the backup reconstructs.
placement_status states whether the old executor was detached, fenced, or unknown.
Evidence binds the head/snapshot to the selected history.

A self-contained backup includes every required artifact byte. A thin backup
declares its external availability dependencies and is labelled accordingly.
Raw secrets are exported only through vault's explicit export/recovery contract.
Secret references and grants are rebound after restore; backup possession grants
no execution authority.

Restore verifies content identities, history, schemas, evidence and live policy.
Missing content yields MissingArtifact with the unresolved manifest.
Historical inspection may proceed over available verified data. Execution waits
for a complete resumable closure and valid placement authority.

## fork, split and merge

A fork creates a new Birth with lineage pointing at the source CellId and Head.
It obtains separate grants and naming. Source operations are historical references;
their pending effects are not inherited as new dispatchable work.

Split/merge are explicit extension protocols. Their certificate must bind:

- all source heads and ownership scopes;
- daughter/merged births and lineage;
- complete state and artifact allocation;
- pending operations, subscriptions and authorities;
- routing cutover and the final authority of retired source instances;
- conservation/exclusivity requirements of the domain.

Every affected authority must admit the protocol. One source partition cannot
claim another's state by copying particles. The protocol must define reads and
writes during cutover and recovery after interruption.
Absent a supported protocol, split/merge returns UnsupportedProtocol.
Spectral partition selection supplies a candidate cut, while the protocol owns
its safe application.

## compatibility

Definition, runtime ABI, state schema, checkpoint schema, profile and data codec
versions are checked independently. Migration declares the specific accepted
source versions. A host retains readers for archived records it claims to serve.
Unknown runtime code may be inspected as content without being executable.
