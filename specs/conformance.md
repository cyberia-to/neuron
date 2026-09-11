---
title: conformance
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# conformance

Conformance verifies behavior at component boundaries. A method stub, a trait
declaration or an unverified receipt does not satisfy a requirement.

## suite map

| ID | Contract / scenario | Required result |
|---|---|---|
| C01 | Same definition, two births | Distinct CellIds and isolated mutable histories |
| C02 | Same instance, two replicas | Same selected head/state after verified replay |
| C03 | Upgrade or graceful relocation | Stable CellId and explicit lineage/epoch change |
| C04 | Independent fork | New birth identity and separate grants; no inherited pending dispatch |
| C05 | Canonical values | Same semantic record has identical particle across supported codecs/hosts |
| C06 | Invalid encoding | Duplicate keys, unsorted sets, bad lengths/UTF-8/fields and unknown mandatory schemas rejected |
| C07 | Content binding | Altered code/input/state/operation/record fails identity or evidence verification |
| C08 | Lifecycle admission | Every state accepts only the operations allowed by lifecycle.md |
| C09 | View detach | Closing/reopening a surface leaves underlying lifecycle/history intact |
| C10 | Head race | At most one authoritative successor; losing proposal dispatches no act |
| C11 | Same neuron, several cells | Independent cell indices with valid coordinated upstream writer sequence |
| C12 | Sparse replication | Gap is explicit; sparse history cannot masquerade as a full SignalChain |
| C13 | Exact duplicate/conflict | Duplicate returns original receipt; changed content at same identity/position returns Conflict |
| C14 | Subject rewrite | Direct, sequential, nested and reactive acts retain the correct host authority |
| C15 | Delegation | Child cannot exceed parent rights, scope, epoch or budget |
| C16 | Revocation race | Enforcement at dispatch respects current grants; past external outcomes remain tracked |
| C17 | Crash after commit | Acknowledged head, content and outbox recover together |
| C18 | Rejected state insertion | Signal ordering and authoritative state remain at predecessor |
| C19 | Lost commit reply | Lookup resolves the same transaction; retry cannot create a second commit |
| C20 | Storage I/O failure | No false durable receipt or silent ephemeral fallback |
| C21 | Corrupt history | Quarantine at acknowledged corruption; only proven uncommitted tail may be discarded |
| C22 | Unknown external outcome | No blind retry; query/dedup/reconciliation follows executor contract |
| C23 | Duplicate outcome | Continuation consumes the retained result once |
| C24 | Replay with nondeterminism | Uses recorded model/time/random/I/O witnesses; repeats no completed effect |
| C25 | Checkpoint portability | Supported runtime restores on another compatible host without process pointers/grants |
| C26 | Resource exhaustion | Bounded yield/fault and no uncommitted effect escape |
| C27 | Streaming reconnect | Durable cursor never exceeds retained chunks; transient deltas are labelled |
| C28 | In-process vs radio | Same requests produce equivalent authority, admission and receipt semantics |
| C29 | Snapshot-and-follow | No missing committed event between snapshot and subscription |
| C30 | Slow subscriber | Bounded memory and explicit ResyncRequired with a verified recovery boundary |
| C31 | Evidence binding | Wrong cell/code/state/policy/epoch/clock evidence fails the gate |
| C32 | Durability vs finality | Local persistence never appears as stronger consensus or external-state verification |
| C33 | Lost authority/partition | Effects pause when fresh grants, fencing or required finality cannot be established |
| C34 | Upgrade with pending attempt | Unknown incompatible attempt blocks activation; old release remains recoverable |
| C35 | Migration failure | No mixed old/new schema, definition, grants or pending-work mapping |
| C36 | Relocation split brain | Only a fenced executor dispatches; unreachable unfenced predecessor blocks takeover |
| C37 | Backup closure | Restore succeeds from complete bytes; missing artifact is explicitly reported |
| C38 | Privacy | Public views/receipts/export omit inaccessible content and metadata |
| C39 | Unsupported profile | Activation/admission fails before any unsupported economic or consensus effect |
| C40 | Legacy import | Repeatable import reports malformed records, conflicts, missing predecessors/content |
| C41 | Headless parity | Same runtime instance is operable through CLI and cyb without separate business logic |
| C42 | Retention boundary | Compacted history is declared unavailable; pending obligations keep required artifacts |

## persistence fault matrix

Inject failure before validation, during content staging, before/after graph
transaction selection, at each disk persistence barrier, before receipt
publication, after receipt loss and during projection rebuild.

For each point record: selected head, retained bytes, receipt observed by caller,
outbox eligibility and restart result. The allowed outcomes are unchanged
predecessor, complete selected commit, or explicit unresolved status pending
recovery. Half-applied authoritative state and false success are failures.

For effects inject failure before attempt record, after it but before dispatch,
after external acceptance, during response recording and during continuation
consumption. Verify the same OperationId, explicit uncertainty, supported
reconciliation and single logical consumption.

## authority probe regression

The initial source review found:
base Subject(now=11, here=12, caps=13) yields Host caps=11 through run_cell,
and two sequential ordinary acts yield caps 13 then 12.
The integrated runtime/ward suite must instead show the same correct host-bound
grant across all paths, while forged subject caps never widen that grant.
This is a required regression test, not evidence of a current fix.

## model exploration

A bounded state-machine test explores lifecycle transitions, concurrent input,
grant changes, operation attempts, crashes and restarts. Assert:

- unique authoritative successor under each profile's selection rule;
- stable operation identity and one recorded logical result consumption;
- no dispatch from uncommitted or insufficiently final state;
- no authority growth through checkpoint, delegation or restore;
- reconstruction of the selected Snapshot from retained graph records.

Property tests cover canonical schemas, predecessor/dedup handling and recovery
boundaries. Protocol-specific tests additionally cover the advertised ledger,
knowledge or distributed-finality contracts.

## implementation report

For a concrete baseline, run a counter definition twice with distinct births.
Admit an increment event in the first instance and commit its resulting state.
The second instance remains at its initial value. A terminal and a cyb surface
reading the first instance at the same Head observe the same value. Restart the
host and verify the same heads/values from graph history, with log presenting
the admission and result records through a query.

Then let the first instance request a tool through ward. Stop the host after
the tool accepts the request and before its reply is recorded. Recovery must
show an unknown attempt, reconcile by OperationId, persist the actual outcome,
and resume the retained continuation once. This scenario combines the essential
boundaries before adding ledger or knowledge protocols.

A release report lists exact revisions of cell and companion crates, runtime,
profile, backend, codec and platform combinations exercised, plus unsupported
features. Each C identifier links to executable evidence or is explicitly
unimplemented. Documentation-only review verifies internal contracts and links;
it does not claim runtime conformance.
