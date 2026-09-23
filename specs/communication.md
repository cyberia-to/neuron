---
title: communication
tags: neuron, prog, radio, tape, spec
status: accepted
spec-version: "0.3"
---
# Communication

A message identifies an actual subject and destination independently of the
transport endpoint and addressed work. Robot names resolve attached identities;
prog/task/operation IDs correlate execution under a neuron. Radio transports
negotiated dialects and Tape frames them. A transport identity or a displayed
name supplies neither current neuron authority nor a new signing subject.

## Supported boundaries

| Profile | Contract |
|---|---|
| Native external action | [Action envelope](action-envelope.md): exact signed request, explicit network/destination, retained receipt |
| Local task/tool | [Execution](execution.md) and [worker dispatch](worker-dispatch.md): captured context, durable attempt, one-use claim |
| Native history mirror | Cyb GraphSession/NativeMirror: bounded source-qualified pages and retained observation cursor |
| Foreign network | Its own address, domain, wire format and verifier; control requires an implemented adapter |
| Navigation | [Typed routes](navigation.md): resolve/display only; attachment or execution is a separate action |

These contracts define actual wire bytes. The semantic requirements below guide
additional delivery/subscription profiles without inventing a universal envelope
already supported by every transport. A radio carrier must negotiate its action
dialect and receiver verifier before it can carry executable requests.

## Authentication, identity and receipts

Every consequential request binds subject, network/profile, destination, action
kind, exact content, nonce and relevant work/causation. Changing content under an
existing request identity conflicts. A retry preserves the original request and
operation identities and addresses the original destination; UI selection changes
cannot redirect it. Receipt lookup may remain read-only after revocation.

Authentication covers the complete supported envelope. Deserializing a payload,
resolving a name or receiving a transport acknowledgement cannot admit an event.
The receiver independently validates the supported subject profile and authority.
An unsupported profile returns an explicit error before dispatch.

| Observation | What it establishes |
|---|---|
| Transport received | Bytes reached the stated ingress boundary |
| Durable ingress | That host retained the exact request under its storage contract |
| Admitted | The owning subject accepted the bounded work once |
| Committed | A selected state transition and its content are retained |
| External accepted | The named endpoint accepted the exact action |
| Completed | A correlated result/failure is retained under the executor contract |
| Final | The named consensus/economic finality verifier accepted the evidence |

A caller reports the actual stage. A local response or HTTP success cannot be
presented as a stronger network/economic claim.

## Read, write and trade

Read names subject/graph partition, selected head or requested snapshot, scope,
evidence and freshness. Responses identify the head actually used. Stronger proof
requests fail explicitly when unavailable; a weaker observation requires the
caller's explicit profile. State-changing computation retains the foreign answer,
head and evidence as input rather than reading an unpinned latest value on replay.

Write submits an authenticated action/fact to the receiving subject or governed
service. The receiver owns admission and state transition. A copied artifact or
remote write request cannot overwrite its authoritative root. In-process calls
preserve the same authority and correlation obligations even when serialization
is unnecessary.

Trade uses a named economic protocol that defines locking, matching, conservation,
expiry, delivery and settlement. Books, issuers, services and shards have the roles
in the [domain ladder](../../cyber/specs/domain-ladder.md). An unsupported trade
protocol fails closed. Generic message delivery supplies no cross-subject atomic
commit or automatic foreign consensus trust.

## Delivery and uncertainty

The owning outbox retains bounded requests and their original identities. Exact
retry within retained deduplication scope returns the original receipt. Missing
or expired lookup evidence yields Unknown/HistoryUnavailable; it does not grant
permission to execute again. Responses correlate network, request, operation and
attempt. An unresolved earlier publication cannot be silently skipped by advancing
a cursor. The native outbox advances only through its accepted prefix.

Deadline semantics name a supported clock. Expiry before any possible dispatch
may be a definite rejection. Expiry after a recorded attempt/possible external
acceptance requires reconciliation. Unknown effects retain obligations; only a
supported executor's exact lookup/idempotency contract can justify a resend.

## Subscription requirements

A profile advertising durable subscriptions binds source, selector, starting
head/position, requested evidence, retention and bounded delivery policy. Each
page identifies its scanned prefix and next cursor. Nonmatching positions may
advance a scan watermark only after complete validation of that prefix. A
consumer retains its result/progress before acknowledgement, and resumes from
that committed position after a crash.

Snapshot-and-follow must close the race between snapshot and subsequent history.
Slow consumers receive bounded backpressure or explicit ResyncRequired with an
available boundary. Silent loss while advancing a durable cursor is forbidden.
A displayed transient token stream advertises its narrower retention semantics.

## Privacy and discovery

Discovery returns an identity plus resolution provenance and serving endpoint.
Access policy applies before exporting private content, selectors, receipts and
routing metadata. A local/private subject may remain addressable without public
registration. Endpoint movement preserves subject identity and requires explicit
profile/network continuity before a pending request can be sent there.
