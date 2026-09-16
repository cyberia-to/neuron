---
title: conformance
tags: neuron, prog, soft3, spec
status: accepted
spec-version: "0.3"
---
# conformance

Conformance verifies behavior at component boundaries. A method stub, a trait
declaration or an unverified receipt does not satisfy a requirement.

## suite map

| ID | Contract / scenario | Required result |
|---|---|---|
| C01 | Same code, two installations under one neuron | Distinct prog data IDs, isolated state and one subject identity |
| C02 | Same neuron, two supported readers/replicas | Same selected head/state after verified replay |
| C03 | Upgrade or supported fenced transfer | Stable NeuronId with explicit prog revision or placement generation |
| C04 | Independent authority fork | Explicit new neuron/custody/grants and lineage; no inherited pending dispatch |
| C05 | Canonical values | Same semantic record has identical particle across supported codecs/hosts |
| C06 | Invalid encoding | Duplicate keys, unsorted sets, bad lengths/UTF-8/fields and unknown mandatory schemas rejected |
| C07 | Content binding | Altered code/input/state/operation/record fails identity or evidence verification |
| C08 | Lifecycle admission | Every state accepts only the operations allowed by lifecycle.md |
| C09 | View detach | Closing/reopening a surface leaves underlying lifecycle/history intact |
| C10 | Head race | At most one authoritative successor; losing proposal dispatches no act |
| C11 | Same neuron, several progs | Independent prog revisions and waiting jobs; shared root CAS and coordinated upstream sequence |
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
| C31 | Evidence binding | Wrong subject/network/prog/code/state/policy/epoch/clock evidence fails the gate |
| C32 | Durability vs finality | Local persistence never appears as stronger consensus or external-state verification |
| C33 | Lost authority/partition | Effects pause when fresh grants, fencing or required finality cannot be established |
| C34 | Upgrade with pending attempt | Original attempt and code retained; no blind replay or old-state adoption over new revision |
| C35 | Migration failure | No mixed old/new schema, definition, grants or pending-work mapping |
| C36 | Relocation split brain | Only a fenced executor dispatches; unreachable unfenced predecessor blocks takeover |
| C37 | Backup closure | Restore succeeds from complete bytes; missing artifact is explicitly reported |
| C38 | Privacy | Public views/receipts/export omit inaccessible content and metadata |
| C39 | Unsupported profile | Activation/admission fails before any unsupported economic or consensus effect |
| C40 | Legacy import | Repeatable import reports malformed records, conflicts, missing predecessors/content |
| C41 | Headless parity | Same neuron/prog/task is operable through CLI and cyb without separate business logic |
| C42 | Retention boundary | Compacted history is declared unavailable; pending obligations keep required artifacts |
| C43 | Local source gate | Existing host evaluates without publication, remote consensus or ahead-of-time compilation |
| C44 | Context identity | Admission, yield and restore retain the same application binding; changes are explicit inputs |
| C45 | Agent: now/soul change mid-task | UI navigation does not mutate the task; steering is ordered and current revocations still apply |
| C46 | Schema substitution | Same human alias with different manifest content cannot pass as the same schema |
| C47 | Large history and sparse mutation | Bounded graph requests and changed index paths; no mandatory full-history scan per transition |
| C48 | Concurrent reservations and lost usage report | Parent and children cannot spend one reservation twice; crash before RunStep retains reserved exposure |
| C49 | Agent: durable child joins | Restart retains ownership/results; completion explicitly settles or detaches every child |
| C50 | Agent: provider failover | Model/config change is recorded and cannot widen disclosure, authority or budget |
| C51 | Agent: context compression | Explicit constraints and unresolved obligations remain accessible; missing sources are labelled |
| C52 | Agent: learning promotion | Candidate provenance/evaluation retained; promotion follows policy and grants no new authority |
| C53 | Agent: workspace conflict | Recovery/rollback preserves later user edits and reports partial/conflicting effects |
| C54 | Agent: delivery retry | Completed task result is resent by delivery identity without rerunning the task |
| C55 | Agent: transfer closure | Task, context, children, artifacts and budget state transfer with fencing and policy checks |
| C56 | Parked progs/invocations | Suspended work can release VM/model residency and resume from the declared closure |
| C57 | Batched persistence | Receipts/cursors/dispatch wait for their own durability boundary despite batching |
| C58 | Agent: evidence semantics | Tool observations/skill acceptance cannot appear as stronger verification or economic conviction |

Rows marked Agent apply to the agent composition profile. Other rows apply when
their local-host/runtime/storage capability is advertised. A release lists every
row as passed, failed, unimplemented or inapplicable with a contract-based reason.
Task competence and comparative outcomes use the A-series in evaluation.md.

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

Use a distinguishable Subject fixture, for example now=11, here=12 and caps=13.
Direct, sequential, nested and reactive act paths must deliver the same bound
host authority. Mutating any program-visible slot must not change that authority.
Run the actual Rune/host adapter regression together with the current publication
and dispatch revocation tests. Historical observations and measured coverage
belong in the release audit rather than being assumed from this fixture.

## model exploration

A bounded state-machine test explores lifecycle transitions, concurrent input,
grant changes, operation attempts, crashes and restarts. Assert:

- unique authoritative successor under each profile's selection rule;
- stable operation identity and one recorded logical result consumption;
- no dispatch from uncommitted or insufficiently final state;
- no authority growth through checkpoint, delegation or restore;
- reconstruction of the selected Snapshot from retained graph records.

Property tests cover canonical schemas, predecessor/dedup handling and recovery
boundaries. Protocol-specific tests additionally cover the advertised book,
shard or distributed-finality contracts.

## Release evidence

Run one code artifact as two progs under the same authenticated neuron. An
increment in the first changes only its state. Park one invocation on a tool
while another progresses. CLI and cyb read the same head/state after reopening.
Then use a second explicit neuron/network binding and verify that selection,
custody, signatures and device policy never leak across those scopes.

Stop after a tool accepts an attempt but before its response is recorded.
Recovery must expose Unknown, retain the same operation/attempt, reconcile only
with correlated evidence and consume the result once. Revocation before another
dispatch must deny it while preserving the old observation. Multi-origin import
must additionally retain each original state, reservation, unknown attempt,
receipt and source fence under distinct progs of the target subject.

A release report lists exact source revisions/features, runtime/profile/backend/
codec/platform combinations, reproduction commands and unsupported capabilities.
Each applicable C identifier links to executable evidence; an inapplicable row
names the absent advertised profile rather than treating documentation as a pass.
Agent competence remains subject to the separate A-series. Observed coverage is
recorded in [the implementation audit](../../soft3/audit/neuron-cell/implementation.md)
and [Soma composition evidence](../../soma/audit/neuron-composition.md).
