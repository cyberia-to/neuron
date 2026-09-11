---
title: evidence
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# evidence and profiles

Evidence is a verifiable statement about specified content at specified roots.
Its scope and assumptions remain explicit in storage, reads and receipts.

## evidence records

Evidence = (claim, subject, roots, verifier, artifact, assumptions, observed_at,
valid_until).

claim is a named variant: content, authorship, state_inclusion, transition,
host_observation, availability, or consensus_finality.
subject is the particle whose claim is being made.
roots is a sorted list of typed root/head references.
verifier identifies exact verification semantics; artifact holds the certificate.
assumptions is a sorted list of typed trust-anchor/model/host/clock references.
observed_at and valid_until are optional TimePoints with comparable semantics.

The verifier MUST bind the complete claim: cell identity, definition/runtime,
before/after state, admitted input/witnesses and policy where applicable.
A valid proof for different code, state, cell, epoch or predicate is rejected.
Unknown verifiers may be archived as unverified data; they cannot satisfy a gate.

Witness bytes, commitments and the assumptions required to interpret them must
remain available under the profile's retention contract.
Malformed evidence returns EvidenceInvalid; missing evidence returns
EvidenceUnavailable; insufficient freshness returns EvidenceExpired.

## distinct claims

| Claim | Establishes |
|---|---|
| Content integrity | Bytes/data correspond to a named particle |
| Authorship | A principal authenticated the exact record |
| Inclusion | The record belongs to an authenticated state/history |
| Transition | The declared computation maps its bound inputs to outputs |
| Host observation | A named host reports a value/action result |
| Availability | Named holders retained the declared content under a contract |
| Consensus finality | The protocol selected this history/root under its rules |

An execution proof conditional on model or external-host output records that
assumption. It establishes only the bound computation. Authority is checked
separately. A root anchored into another graph proves the anchor relationship;
further claims need their corresponding evidence.

The state organ's T0/T1/T3 labels apply to external-state answers under its own
contract. Cell stores that result and its evidence. It does not derive T0 from
local execution, local durability or a bare signature.

## finality

Finality = (rule, selected_head, certificate, assumptions).
rule is LocalAuthority or a protocol particle.
selected_head is a Head; certificate is an optional evidence particle.
LocalAuthority identifies the instance's governing local authority and its
fencing epoch as assumptions. Its receipt is explicitly local.
A consensus profile requires its protocol's verified certificate.

Durability and finality are independent. A host may retain durable proposals
before they are selected. Such proposals have no authoritative cell Head and
cannot dispatch effects. The graph may expose their status by proposal identity.

An operation's gate is the conjunction of the instance profile and any stronger
operation requirement. The engine MUST verify this gate immediately before
making an attempt dispatchable. Disputed finality quarantines dependent effects.
A protocol that admits post-selection reversal must define a compatible effect
policy; irreversible effects cannot rely on a weaker provisional head.

## profile record

Profile = (kind, admission_rule, finality_rule, execution_requirement,
read_requirement, retention, privacy, clock, required_protocols).

kind is runtime, building, ledger or knowledge. The remaining fields are
particles naming complete versioned contracts; required_protocols is a sorted
list. Activation verifies that every mandatory rule has a supported adapter.

| Profile | Mandatory boundary behavior |
|---|---|
| Runtime | One authorized execution placement/epoch; LocalDurable commits; explicit host assumptions permitted |
| Building | Service governance; per-client authentication; a named local-authority or consensus policy; bounded delivery and queries |
| Ledger | Economic transition/conservation verifier, authority and finality rules, expiry/freshness semantics for conditions |
| Knowledge | Region membership/admission rules, authenticated state, reconciliation/finality and availability contracts |

A building served by one authority must advertise that authority assumption.
Several replicas alone do not establish distributed finality.
A ledger profile is usable only with its named economic protocol. Oikos support
requires the matching/freshness protocols to be specified and verified.
Knowledge division requires a division protocol conforming to evolution.md.

Profile parameters can be changed only through an authorized upgrade bound by
both the old and new governance/finality requirements. Unsupported transitions
return UnsupportedProfileChange. Changing a manifest or host configuration alone
cannot silently alter the instance's trust contract.

## verification and liveness

Verification consumes explicit trust anchors. Anchor updates are authenticated
events with policy-defined continuity. Clock uncertainty, unavailable ancestors
or incomplete content cannot be turned into a successful proof result.

A selected head is a correctness claim; a recent head is also a liveness/freshness
claim. Partitions may preserve the former while preventing the latter.
A read/condition that needs freshness must reject an expired or unverifiable
answer even if an old execution proof still verifies.
