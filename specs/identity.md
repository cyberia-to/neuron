---
title: neuron identity and authority bindings
tags: neuron, identity, spec
status: accepted
spec-version: "0.3"
---
# Identity and authority bindings

NeuronId is the existing native 32-byte identifier, shared through the
dependency-free neuron-id crate. BBG, tok and public graph APIs re-export this
same type. Code that names a subject requires no engine, database or VM.

## Subject reference

A subject is either Native(NeuronId) or Foreign(domain, address bytes). Foreign
domain is a nonempty bounded ASCII identifier; address is nonempty bounded bytes
in the native protocol representation. Destination network, derivation domain,
display formatting and transport address remain separate. References are safe
to observe without authority. Their codecs bind discriminant and lengths.

The supported native authentication profile is cyber-secp256k1-hemera-v1:
NeuronId = Hemera(SEC1 compressed public key). The 33-byte public key must decode
as a valid secp256k1 point; signatures use mudra's existing ADR-036 implementation.
The key-derived native ID must match the declared subject. Caller-supplied display
addresses do not establish that binding. Existing domain and bridge derivation
and signature bytes are unchanged. Other profiles require their own verified
adapter and cannot fall back to this profile.

## Attachment and binding

An attachment records subject, access mode, binding revision and available
network references under the robot's disclosure scope. Observe carries no signer.
Control and delegated access reference verified authority held by the host.
A binding carries subject, revision, network, policy and opaque vault key handle;
the graph may store its public evidence, never secret key material.

A binding revision changes monotonically. Detach/revoke disables future use
without deleting identity or earlier attempts. Reattach does not resurrect an
old grant. Changing the pubkey in H(pubkey) creates a different subject; retaining
the old identity requires the original key or a separately specified recovery
profile. Device/worker movement preserves the subject and admitted action bytes.

## Action authorization

An action pins subject, prog/invocation/native caller, binding revision, network,
payload, grant, executor contract and resources before dispatch. Ward validates
current grant/revocation and vault resolves the permitted key operation. The
signature authenticates the exact versioned action body and cannot be transplanted
to another subject, network, prog or operation. Persisted selection is not a grant.

Legacy cell birth hashes are origin references. Import establishes a new
authorized mapping to subject/prog state and does not claim the old local owner
signed historical records. No constructor authenticates an arbitrary NeuronId.

The [shared subject contract](../../soft3/specs/neuron.md) and
[cyb architecture](../../cyb/specs/architecture.md) define composition.
