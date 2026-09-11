---
title: communication
tags: cell, soft3, spec
status: draft
spec-version: "0.2"
---
# communication

Cell-to-cell communication uses destination, operation, provenance and receipt
semantics shared by in-process dispatch and radio transport. Tape frames carry
the negotiated application dialect. Moving across a process boundary preserves
authorization and result meaning.

## envelope and authentication

Envelope = (revision, message_id, sender, destination, mode, content,
causation, deadline, requested_evidence).

message_id is a fresh origin-scoped nonce; sender is a principal identity.
destination is a named variant: instance(CellId), host(HostId), or
definition(DefinitionId) for inspection only.
mode is read, write, trade, receipt or control.
content is a particle of the corresponding typed request/result.
causation is a sorted list of typed references. deadline is optional TimePoint;
requested_evidence is a profile requirement particle.

Authentication is a separate envelope binding the full Envelope particle.
Repeating (sender, message_id) with changed content produces Conflict.
Retries preserve Envelope and Event/Operation identities.
An unauthenticated network payload never becomes an admitted cell event.

Transport receipt, local durable receipt, cell admission, commit and external
completion are distinct stages. A receipt's stage determines what was achieved.

Receipt = (request, stage, cell, head, result, durability, finality,
evidence, observed_at).

request references the complete Envelope/Event/Operation being acknowledged.
cell/head/result/evidence/observed_at may be absent where the stage permits.
Stages: Received, Admitted, Committed, Completed, Rejected, Unknown.
Committed requires cell/head, LocalDurable or explicit Volatile, and a finality
value. Completed requires a result and evidence/provenance appropriate to it.
Received promises only the stated host ingress durability.

## read, write and trade

Read queries a pinned cell head. It returns value, actual head, evidence and
freshness. A latest read names the selected head used. A stronger requested proof
that cannot be supplied returns EvidenceUnavailable or an explicitly requested
weaker result; downgrade is never implicit.

Write exports an authenticated fact or submits an Event to another cell.
The receiver validates provenance, authority, current policy and input schema.
Its own transition establishes the resulting state. An external writer cannot
overwrite the receiver's Snapshot.

Trade supplies conditions and receipts interpreted by an explicitly supported
economic protocol. Version 0.2 defines the routing/evidence boundary; the
economic protocol defines locking, matching, expiry and settlement.
A host without that protocol returns UnsupportedProtocol. The interface provides
no generic cross-cell atomic-commit promise.

Reads inside a state-changing computation pin the foreign head and record their
answer/evidence as witnesses. Later foreign updates arrive as new events.
A local call follows the same rules; shortcuts may avoid serialization while
preserving all validation and identities.

## delivery and duplicate handling

Delivery is at-least-once while the sender retains an unexpired pending request.
The receiver persists deduplication keys alongside admission/outcome state.
Within the declared retention horizon, an exact retry returns the existing
receipt and cannot create a second admitted operation.

Expired deduplication history returns HistoryUnavailable/Unknown; it does not
authorize blind re-execution. Gaps request missing predecessors or a verified
checkpoint. Conflicts retain both content identities for the governing policy.
Out-of-order replies are correlated by request/operation/attempt identity.

Deadlines use their declared clock semantics. If clock evidence is insufficient,
the deadline cannot be asserted satisfied. Expiry before dispatch yields a
definite rejection. Expiry after possible dispatch requires reconciliation.

## subscriptions and cursors

Subscription = (subscriber, source, selector, start, delivery_policy, expiry).
subscriber and source identify cells/principals. selector is a schema/query
particle; start is beginning, a Cursor, or a verified snapshot Head.
delivery_policy names acknowledgement, retention and bounded-buffer behavior.

Cursor = (cell, index, commit, selector).
A cursor is valid only against the selected history and selector.
A subscription delivers (previous_cursor, next_cursor, matching_records,
scanned_head, evidence). Nonmatching positions can advance the scan watermark.
A consumer persists its effects/result and cursor atomically before acknowledging.
Reconnect resumes at the committed cursor. Snapshot-and-follow binds the
snapshot head and subsequent cursor atomically to prevent missing updates.

Slow consumers have bounded buffers. Overflow returns ResyncRequired and the
available snapshot/history boundary. Silent dropping while advancing a durable
cursor is forbidden. A history gap has an explicit result.

## discovery and privacy

Name resolution returns identity and provenance. Endpoint discovery locates a
host serving that identity. Neither operation implies execution authority or
proof of a state transition. A private instance may remain locally addressable
without public registration.

A peer authenticates before receiving private references or content.
Export policy governs payloads, receipts, routing metadata and subscriptions.
Address changes preserve instance identity. Definitions are immutable and fetched
by particle once resolved. Unsupported dialects/codecs fail negotiation before
their content can be executed.
