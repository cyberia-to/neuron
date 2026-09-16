# Supported local authority profile

The local adapter uses the existing mudra secp256k1 `H(compressed_pubkey)`
profile. It does not reinterpret that ID as the proof-native `H(secret)` profile.
Vault owns the signing key behind an opaque host object. Ward holds a bounded
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
No seed, mnemonic, signing key or live grant handle is graph content.

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
