# experimental local conformance — 2026-09-11

This matrix covers the implemented private local profile, not a general 0.2
conformance claim. Passed rows refer only to the named executable scenario.
Unimplemented includes partially tested contracts whose full gate is still open.
The C-series definitions remain in [conformance](../specs/conformance.md).

| ID | Status | Evidence or remaining gate |
|---|---|---|
| C01 | passed | node/runtime: two births, distinct IDs and isolated state |
| C02 | unimplemented | Replication, selection and remote replay |
| C03 | unimplemented | Upgrade/relocation |
| C04 | unimplemented | Independent fork API and grant handling |
| C05 | unimplemented | Local canonical values tested; cross-host/codec vectors pending |
| C06 | unimplemented | Bounded malformed data tests exist; complete schema refinement suite pending |
| C07 | passed | Model forged content and cybergraph modified content rejected |
| C08 | unimplemented | Implemented local transitions tested; exhaustive lifecycle matrix pending |
| C09 | unimplemented | Cyb surface attach/detach |
| C10 | passed | BBG simultaneous writers select exactly one conditional head; dispatch follows accepted head |
| C11 | unimplemented | Per-neuron public writer coordination |
| C12 | unimplemented | Sparse SignalChain replication |
| C13 | passed | Store, node and CLI duplicate/conflicting requests; cross-cell nonce claim |
| C14 | passed | Rune direct/sequential/nested/reactive authority regression; machine never owns grants |
| C15 | unimplemented | Delegated grants and children |
| C16 | unimplemented | Ward revocation races |
| C17 | passed | BBG reopen and CLI new-process recovery of state, content and attempted outbox |
| C18 | unimplemented | Local rollback tested; public chain/state transaction still open |
| C19 | passed | Injected loss of committed admission receipt resolves without second execution |
| C20 | unimplemented | Typed failure paths exist; complete I/O/barrier injection pending |
| C21 | unimplemented | Corrupt content refuses reads; quarantine/tail recovery not implemented |
| C22 | passed | Attempt restart is unknown, repeated take/cancel rejected, manual correlated reconciliation |
| C23 | passed | Sequential outcome consumption retained once; duplicate/conflicting outcomes tested |
| C24 | unimplemented | Tool results retained; model/time/random witness replay profile pending |
| C25 | unimplemented | Same-host new-process restore tested; cross-host compatibility matrix pending |
| C26 | unimplemented | Step/checkpoint bounds tested; full baseline resource metrics pending |
| C27 | unimplemented | Streaming cursors |
| C28 | unimplemented | Radio adapter |
| C29 | unimplemented | Snapshot-and-follow |
| C30 | unimplemented | Subscriber backpressure/resync |
| C31 | unimplemented | Full evidence suite |
| C32 | passed | CLI receipts label LocalDurable/LocalAuthority; no consensus/proof claim |
| C33 | unimplemented | Distributed authority freshness and partition handling |
| C34 | unimplemented | Upgrade with pending attempts |
| C35 | unimplemented | Schema/definition migration |
| C36 | unimplemented | Relocation fencing |
| C37 | unimplemented | Export/backup closure API |
| C38 | unimplemented | Private database boundary only; public disclosure/privacy suite pending |
| C39 | unimplemented | Unknown local profiles rejected; general profile admission API pending |
| C40 | unimplemented | Legacy graph/tape import |
| C41 | unimplemented | CLI uses shared engine; cyb adapter pending |
| C42 | inapplicable | This profile retains all history; compaction is unsupported |
| C43 | passed | CLI source creation/submit immediately parse, lower and interpret locally |
| C44 | passed | Context pinned into event and checkpoint; preserved on process restart |
| C45 | unimplemented | Soma now/soul steering |
| C46 | unimplemented | Immutable local manifests checked; full substitution/refinement suite pending |
| C47 | unimplemented | B-tree bounded ranges exist; large-history stress matrix pending |
| C48 | unimplemented | Lost local slice settlement tested; shared parent/child reservations pending |
| C49 | unimplemented | Durable child joins |
| C50 | unimplemented | Provider failover |
| C51 | unimplemented | Context compression |
| C52 | unimplemented | Learning evaluation/promotion |
| C53 | unimplemented | Workspace effect conflict recovery |
| C54 | unimplemented | Delivery identity/retry |
| C55 | unimplemented | Agent transfer closure |
| C56 | passed | Runtime recreated per slice; process exit releases residency while graph retains checkpoint |
| C57 | inapplicable | Current implementation uses individual Immediate transactions, without batching |
| C58 | unimplemented | Soma/tool/skill evidence semantics |

All A-series comparative agent gates are unimplemented. The current counter and
manual tool executor are foundational integration scenarios, not agent benchmarks.
