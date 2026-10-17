---
title: proof-based authority
tags: neuron, identity, spec
status: accepted
alias: signatureless identity, hash-based identity, identity primitive, proof authority
---
# Proof-based authority

The native target profile: authority is a private proof over a hash commitment, not a signature. Moved from `mudra/specs/identity.md` on 2026-10-07 — identity is a field of the neuron, so its contract lives with the subject; [identity](identity.md) binds subjects and keys today, [local authority](local-authority.md) is the implemented secp256k1 profile, this page is where both are headed.

Cyber uses private proofs of programmable authority. a default authority
commits to a secret with Hemera; Zheng proves that the owner satisfies the
policy for an exact action. proof-based authority and [[mudra]]'s encryption/key
agreement have separate roles.

## authority, subject and profiles

```text
authority_commitment = profile_hash(secret)
authorization = private_proof(policy(authority, action, state, witness) = accept)
```

the authority commitment is bound to a versioned policy. a stable neuron
subject and a rotatable authority key are separate objects: rotation updates
authorized policy state rather than silently renaming the subject.

native secret generation uses 256 bits of entropy. its KDF, hash domain,
encoding and output size belong to the selected profile. field width, digest
length and entropy are separate parameters.

existing identifiers and compatibility profiles retain their defined meaning.
this specification does not reinterpret a secp256k1-derived neuron ID as a
hash-preimage identity. migrations and recovery follow an explicit authorized
transition. see the [neuron identity contract](identity.md).

## exact authorization statement

the verified statement binds at least:

```text
protocol / cryptographic profile and version
network / genesis / execution environment
expected program and policy identity
canonical action, inputs and outputs
accepted state root and applicable policy revision
nonce / operation identity and validity interval
required public result
```

an execution proof must bind the expected relation and these values. proving
preimage knowledge without action binding is insufficient for message
authorization. a valid proof for one action cannot authorize another action,
network, root, program or policy revision.

the witness contains the secret, policy credentials and authenticated state
openings. private authority requires a zero-knowledge proof profile; a public
execution trace containing the secret cannot fulfill this contract.

## verification and replay

a verifier checks the expected profile/program, the proof, the action's domain
and current policy/state rules. replay and double-spend prevention use
authenticated nonce/nullifier state. durable admission and retries retain the
same operation identity; proof bytes do not define that identity.

knowledge soundness, statement binding and zero knowledge are requirements of
the selected proof construction. preimage resistance protects secret ownership;
collision resistance protects relevant commitments. transcript and recursive
verification assumptions also remain explicit. the full system is not reduced
to collision resistance alone.

## programmable policies

the same proof interface supports single owners, thresholds, delegation,
timelocks and recovery:

| policy | required relation |
|---|---|
| single owner | secret opens the authorized commitment |
| threshold | sufficient distinct authorized credentials |
| delegation | valid scope, revision, expiry and delegate authority |
| timelock | authority plus authenticated chain time/height |
| recovery | authorized recovery quorum and policy transition |

policy configuration and transitions are authenticated state. wall-clock
assertions from an untrusted host do not satisfy a chain timelock.
performance, proof sizes and recursive costs are measured for the selected
program/backend; they are not fixed by this abstract interface.

## private authority

a public permanent subject identifier links its actions. the private profile
proves membership and policy satisfaction against a committed authority set
while keeping the member, ownership opening and private values hidden.

nullifiers bind their intended scope and prevent replay without exposing the
authority commitment or discovery key. private output values and individual
contributions remain private witnesses. plaintext per-action weights cannot
be labeled hidden merely because authorship is hidden.

public roots and authorized aggregate releases have an explicit leakage
contract. exact aggregate changes can reveal an isolated contribution.
the enclosing BBG/graph protocol must analyze this disclosure together with
transaction timing and side information.

selective disclosure uses a purpose-bound proof or scoped credential.
it must not reveal root secrets, reusable viewing keys or derivation material
that exposes unrelated actions.

## encryption and recovery

- [seal](../../mudra/specs/seal.md) encapsulates a fresh secret for a published recipient key.
- [stealth](../../mudra/specs/stealth.md) derives pairwise secrets by a validated commutative
  action, including per-payment ephemeral delivery.
- [veil](../../mudra/specs/veil.md) supplies profile-specific private computation.
- [quorum](../../mudra/specs/quorum.md) supplies the selected threshold distribution protocol.

a sender knows a delivery shared secret. spending authority therefore uses an
independent owner secret and verifier-checkable policy binding. static NIKE
or a channel MAC alone does not provide transferable public authorization.

[Private recovery](../../mudra/specs/private-recovery.md) defines discovery, authenticated history
coverage, spend-state verification and key lifecycle. a spell reconstructs keys;
the recovery service must also preserve the required ciphertext history/state.

## protocol qualification

each deployed profile fixes action encodings, key derivation, hash/proof
parameters, privacy disclosures and classical/quantum resource assumptions.
the abstract choice of proof-based authority is independent of the measured
latency or current build status of an implementation.

source/version comparisons and measurements live in
[the architecture assessment](../../mudra/audit/signature-optimality.md).
