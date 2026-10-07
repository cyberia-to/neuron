# Supported local authority profile

The local adapter uses the existing mudra secp256k1 `H(compressed_pubkey)`
profile. It does not reinterpret that ID as the proof-native `H(secret)` profile.
Vault owns the signing key behind an opaque host object. `SigningVault::sign`
is the host port. With the `vault` feature, `custody::LocalVault` wraps
`vault::local::Signer`: encrypted Cybergraph/BBG custody, retained checkpoints,
OS-owner unlock, exact operations and two verified ciphertext replicas before
signature release. The local action grant stays held for this whole call; the
Vault owner adapter does not reacquire the action grant lock. This is a trusted
local operator profile, not authenticated remote Ward delegation. `KeyVault`
is the explicit raw-key compatibility/test adapter. No automatic fallback or
migration between these custody modes is permitted. Ward holds a bounded
grant for one subject/network/policy/epoch and declared act set. Watch-only uses
NoAuthority. A program's allowed-act list is a request, never a grant.

Every graph transition signs a canonical `neuron/authority-statement/1` that
binds subject, network, policy, epoch, optional prog/invocation/act, action kind
and proposed state/event digest. Dispatch additionally signs the immutable
operation. Verification derives the same native ID from the embedded public key.

Evidence bytes: ASCII `NSIG1`, compressed SEC1 key (33 bytes), ADR-036 signature
(64 bytes). The ADR-036 signer is the key's `neuron`-HRP bech32 address; signed
data is ASCII `cyber:neuron:authority:v1:` followed by the 32 statement bytes.
The existing mudra ADR-036 signing/verification functions define canonical JSON.
No spell, signing key or live grant handle is graph content.

The neuron-node graph adapter independently verifies each native publication:
the key-derived subject, network/policy/epoch, exact previous root and proposed
state/event digest, and the evidence signature. A fabricated Authority adapter or
changed state with an old signature cannot publish a native head. Legacy history
has its explicit unsigned import codec and retains that provenance; storage
fences prohibit successors after activation. This local verification does not
substitute for a future consensus/proof profile.

The host checks the current grant at every authorization. Grant replacement is
monotonic in revision; revocation takes effect before subsequent dispatch checks.
The enforcing executor must recheck the current grant and writer lease at its
dispatch linearization point. An earlier signed decision documents history; it
does not override current revocation. Unsupported remote/proof profiles fail closed.

## NSIG1 envelope

The exact bytes of the profile (moved from `mudra/specs/neuron-auth.md`, 2026-10-07; the code is `neuron-auth`'s `neuron::sign` / `verify_statement`).

This is the implemented H(compressed SEC1 public key) neuron profile used by the
local neuron authority adapter. It uses the existing Cosmos/ADR-036 primitives.
Proof-native/hash-preimage profiles retain their separate specification and
identities; this profile asserts neither post-quantum security nor network finality.

A statement is a 32-byte content commitment supplied by the owning data codec.
The message is the exact ASCII bytes `cyber:neuron:authority:v1:` followed by the
32 statement bytes. Its ADR-036 signer is the Cosmos address of the compressed
public key under the literal `neuron` HRP. The envelope is exactly 102 bytes:

| Offset | Length | Meaning |
|---|---|---|
| 0 | 5 | ASCII `NSIG1` |
| 5 | 33 | Compressed SEC1 secp256k1 public key |
| 38 | 64 | Existing ADR-036 compact ECDSA signature |

Verification requires exact length/prefix, valid curve key/signature, unchanged
`claim::neuron_of(public)` equal to the expected subject, and the ADR-036 signature
over that exact domain-separated message. It consumes no spell, storage, scheduler,
VM or inference dependency. `neuron::sign` rejects a subject different from the key's
native ID. Verification alone grants no permission; the host/network checks its
current scope and binds the canonical statement to the actual operation.

The signature format lives in the `neuron-auth` crate; node's existing
`verify_evidence` API delegates to it. Existing NSIG1 bytes, native IDs, ADR-036
signer strings and messages remain valid. This adds no new signing identity.

The canonical signing function is `neuron::sign`; `sign_statement` remains a
source-compatible alias for existing callers. Custody, permission and durable
release belong to Vault and its host, not this primitive.

