---
title: evidence and profiles
tags: neuron, proof, spec
status: accepted
spec-version: "0.3"
---
# Evidence and profiles

Evidence binds a particular claim to content, roots, verification semantics and
explicit assumptions. Neuron identity, prog identity, local execution and network
participation are distinct axes. A profile defines the evidence required at each
boundary; it does not create another subject for every governed state object.

## Claims and binding

| Claim | Establishes |
|---|---|
| Content integrity | Named bytes/data match their particle |
| Authorship | A supported principal authenticated the exact record |
| Inclusion | A record belongs to the specified authenticated root/history |
| Transition | The verifier accepted the computation over its bound inputs |
| Host observation | A named host reports the specified external result |
| Availability | Named holders retained declared content under a contract |
| Consensus finality | The protocol selected the specified history under its rules |

Evidence references the exact verifier/version, certificate, trust anchors,
subject/network, applicable prog/runtime/input/before/after/policy/epoch and clock
or freshness bounds. A proof for different code, subject, root or predicate fails
the gate. Unknown certificates may be archived as unverified data, but cannot
satisfy an effect requirement. Missing, malformed and expired evidence remain
distinct diagnostic conditions.

Witness bytes and assumptions must remain available under the retention contract.
A proof conditional on a model/host answer proves only the bound computation
under that assumption. A signature establishes authorship and still needs current
authorization. Anchoring a root proves its anchor relationship; other claims need
their own evidence. The state organ's external T0/T1/T3 grades retain that organ's
contract and cannot be inferred from local storage or execution.

## Native local profile

[Local authority](local-authority.md) binds H(compressed_pubkey) and NSIG1 to the
exact canonical action statement. The node verifies each native application
publication independently. The host also checks the current grant through commit
and dispatch. The database provides local durability and ordered application CAS.

This profile records trusted Rune/host execution and correlated observations.
It advertises local authority rather than a consensus certificate. Native HTTP
endpoint acceptance and mirror observations carry their actual source/profile.
Soma local inference retains the pinned model and observed result; it makes no
cryptographic claim that the model's answer is true.

Durable proposals or staged imports have no permission to dispatch before their
authoritative activation. Effect eligibility is the conjunction of the profile's
required selected state, durability and current authority. A stronger requested
finality gate requires a supported verifier. Incomplete/unverifiable evidence
leaves dependent effects undispatched.

## Domain profiles

The following roles refine service and protocol contracts, independently of
whether a neuron runs a prog:

| Role | Required boundary |
|---|---|
| Executing neuron | Current grant, bounded work, retained state and one-use effect handoff |
| Service | Client authentication, governance, delivery limits, declared local or consensus authority |
| Token book/issuer | Conservation, mint/burn/transfer rights, conditions, expiry and economic finality |
| Graph shard | Membership, completeness, availability, reconciliation and partition finality |
| Full/partial/light node | Storage/proof duties and verified tip under its participation mode |

Several replicas alone do not establish distributed finality. A token book does
not inherit an economic verifier from the neuron engine. A shard split requires
the protocol's explicit conservation/ownership and routing cutover rules. Oikos
and distributed profiles must implement their named contracts before advertising
those effects; the local runtime cannot silently stand in for them.

Each additional profile declares admission, finality, execution/read evidence,
retention, privacy, clock and required protocols. Activation rejects unavailable
mandatory semantics. A profile change is an authorized forward transition bound
by old and new governance; editing a manifest cannot change the trust contract.
These requirements do not add a generic Profile record to the immutable native
v1 schema suite.

## Freshness and failure

Trust-anchor updates require authenticated continuity under policy. A selected
head is a correctness claim; a sufficiently recent head also needs liveness and
clock evidence. Partitions may preserve an old valid proof while making a fresh
read/condition unavailable. Unavailable ancestors, clock uncertainty or missing
content cannot become successful verification by timeout or fallback.

A protocol permitting reversal must define an effect policy compatible with its
provisional state. Irreversible effects cannot rely on a weaker head than their
declared gate. Lost authority or disputed finality pauses dependent work while
retaining already observed outcomes and their original attribution.
