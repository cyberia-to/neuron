---
title: authority
tags: neuron, ward, vault, soft3, spec
status: accepted
spec-version: "0.3"
---
# Authority

Neuron is the subject that authenticates acts. Robot attaches explicit subjects,
networks, keys and devices. A prog/task ID identifies work under that subject and
has no separate signing key. Ward evaluates current rights; vault performs
permitted secret operations; the runtime carries the context through execution.

## Native publication linearization

The native neuron/prog profile signs the exact canonical statement at preparation
and rechecks current authority through the graph commit. `Authority::with_current`
is required for activation, every fresh publication, import and worker dispatch
claim. An adapter without this guarantee returns Unsupported. A signature made
before revocation cannot authorize a new publication after revocation.

The neuron-node adapter independently verifies subject derived from the key,
network/policy/epoch, exact predecessor, candidate state/event and signature.
[Local authority](local-authority.md) defines the unchanged H(compressed_pubkey)
and NSIG1 bytes. The host's current-grant guard and the storage verifier perform
different checks; both are required for the supported local profile.

An exact historical request lookup is read-only and returns its original receipt.
The engine's admission lookup validates the subject, nonce and admitted event
kind. A composing owner may recover its missing catalog receipt after admission
without admitting another child or obtaining another effect permit. Historical
receipts never supply current authority.

## Bound context

A runtime authority action binds NeuronId, network, policy, epoch, optional prog,
invocation and act, action kind, exact statement digest and requested act set.
Product bindings additionally capture attachment revision, supported identity
profile, device policy and key custody. Selection changes do not retarget an
already submitted task. A foreign/watch-only binding can observe within policy;
control requires an implemented proof-of-control and action profile.

Checkpoint/context records contain immutable references sufficient to request
rebinding. Their deserialization cannot restore live grants. Program-visible
capabilities are inspectable descriptors or opaque host-bound handles. A mutated
Rune noun, model response, nested call or forged returned value cannot enlarge
rights. Authentication identifies the request author; authorization decides
whether this exact act is allowed at the current boundary.

## Policy and disclosure

Soul records configuration and intended policy. Ward interprets it and controls
installation, activation, upgrade, management, delegation and effect dispatch.
An allowed-act list in a prog requests rights; activation receives at most the
intersection with available rights and current policy. New rights require a new
decision. Imported instructions and learning proposals remain application data.

An invocation pins the configuration and context that formed its intent while
ward checks current policy at each effect. Context expansion, model replacement,
provider failover and child delegation must respect disclosure scope. Reading a
source does not automatically authorize transmitting it to another recipient.
Learning promotion that changes executable code, capabilities or economic
conviction passes the corresponding owner policy and authorization boundary.

## Delegation, revocation and placement

Delegated acts, targets, expiry, further delegation and allowance are bounded by
the delegator's current rights. Same-subject child work has its own data identity
and allowance; it introduces no new principal. Cross-subject delivery authenticates
the actual sender and explicitly delegated rights through its supported profile.

Grant replacement is monotonic. Rebinding checks current policy and refuses
stale epochs, revoked attachments, missing custody or unsupported profiles.
[Worker dispatch](worker-dispatch.md) binds the selected worker generation and
holds current authority through the physical handoff. A device label describes
placement policy; it is not remote hardware attestation.

Revocation cannot undo an already accepted external effect. Its original attempt
and outcome remain tracked. Recording an observation or cancellation uses current
management scope without demanding the revoked tool capability again. Unknown
outcomes retain obligations until definite evidence or explicit reconciliation.
A multi-device reader or speculative worker gains no independent writer lease.

## World access and privacy

Graph writes, protected reads, output, file/process/network/device acts and runtime
calls pass their relevant host boundary. Sandboxed runtimes expose gated imports.
Native Rust code in the host process is trusted and has ambient process access;
a profile requiring confinement must enforce it through a supported sandbox or
separate process. A manifest or signature alone cannot provide confinement.

Secret references select vault operations. Mnemonics, keys and live handles stay
out of ordinary checkpoints, public history, views and receipts. Private return
values use explicit encrypted storage/channel and retention contracts. Backup
possession and successful decryption do not by themselves grant execution rights.

## Human decisions

A permission UI may propose allow, deny or pending under an owning host policy.
A durable pending request binds exact subject, revision, release, destination,
arguments and rights; waiting retains no mutable transaction or native stack.
Changed requests require new decisions. Denial is a definite result, and an
unavailable authority leaves the effect undispatched. The generic native engine
currently exposes allow/error through Authority; a UI's pending workflow is a
composing application contract rather than an additional built-in wire enum.
