---
status: implemented
created: 2026-08-06
---
# absorb lytics's fresh-domain identity + generalize ADR-036 signing

## problem

lytics (`~/cyber/lytics`) independently implemented secp256k1 + bech32 +
ADR-036 signing for its visitor identity — and it landed almost
byte-identical to code already in mudra, without depending on it:

| lytics (`rs/event/src/*.rs`) | mudra (`src/*.rs`) | match |
|---|---|---|
| `keys::bech32_of_pubkey` | `cosmos::address` | same formula: `bech32(hrp, ripemd160(sha256(pubkey)))` |
| `sign::sign_doc` | `claim::adr036_doc` | identical field order, same amino-JSON shape |
| `Neuron.native` | `claim::neuron_of` | both `Hemera(pubkey)` |

this is real duplication mudra should absorb — it is the crate that owns
these conventions cyber-wide, per its own CLAUDE.md.

what is genuinely different, and must NOT be collapsed: **key
derivation**. mudra's `seed.rs` is BIP32/BIP39 at the Cosmos path
(`m/44'/118'/0'/0/0`) because the legacy bridge must reproduce an
*existing* spacepussy/cosmos account byte-for-byte — there is no freedom
there, the chain already recognizes that key. lytics has no legacy
account to reproduce: a visitor's identity is fresh, generated in a
browser, and needs to stay small in a wasm bundle (bip32's HMAC-SHA512
ladder was cut deliberately for size — see lytics's `specs/README.md`,
"identity pipeline"). lytics derives `d = Hemera(entropy ‖ 0x00 ‖
domain) mod n` directly — one hash, no bip32.

these are two legitimate derivation schemes for two different problems,
not one implementation and one bug. both belong in mudra as siblings.

a second thing worth stating explicitly: mudra's `lib.rs` currently
frames secp256k1 + ADR-036 as "Phase 1 — the legacy-key bridge," implying
it exists only to migrate old accounts and will eventually be superseded
by the post-quantum, signature-free `Hemera(secret)` model everywhere.
lytics's use is a *third* motivation for the same primitive, not a bridge
at all: secp256k1 is chosen because it is what a browser wallet
(`window.ethereum`/keplr) or a ~4KB JS library (noble-secp256k1) already
speaks natively — zero-to-minimal wasm cost, not migration compatibility.
mudra's module doc should say this, or a reader will wonder why a
"soon-obsolete bridge" primitive is also the permanent visitor-identity
scheme for a different project.

## changes

### 1. `src/domain.rs` (new) — fresh domain-scoped identity

- `domain_scalar(entropy: &[u8; 32], domain: &str) -> k256::Scalar` —
  moved from lytics `keys.rs`, unchanged formula.
- `DomainKey` (or similar) wrapping a `SigningKey` + its bech32 + native
  id, mirroring lytics's `Neuron` shape but named for mudra's domain
  (not "Neuron" specifically — mudra already exports a bare
  `SigningKey`; check for a naming collision with any future mudra
  neuron type before settling the name).
- reuses `cosmos::address`/`cosmos::account_id` for the bech32 encoding
  — no second bech32 implementation.
- reuses `claim::neuron_of` (or a common helper both `domain.rs` and
  `claim.rs` call) for `Hemera(pubkey)`.

### 2. `src/claim.rs` — generalize ADR-036 signing out of claim-specific code

`adr036_doc` is currently private and hardcoded to the "bind neuron"
message shape. split into:
- `pub fn adr036_doc(signer: &str, data: &[u8]) -> String` — the general
  sign-doc builder (already exists, just needs to become the shared
  primitive, not a claim.rs implementation detail).
- `pub fn sign_arbitrary(key: &SigningKey, signer: &str, data: &[u8]) -> [u8; 64]`
  and `pub fn verify_arbitrary(pubkey: &[u8; 33], signer: &str, data: &[u8], sig: &[u8; 64]) -> bool`
  — general sign/verify over the ADR-036 doc, extracted from `claim::create`/`claim::verify`'s
  inline logic.
- `claim::create`/`claim::verify` become thin callers of the general
  functions, passing `bind_message(&neuron)` as `data`. behavior
  unchanged — existing claim tests must still pass untouched.
- lytics's event-signing (`sign_body`/`verify` in its `sign.rs`) becomes
  a caller of `sign_arbitrary`/`verify_arbitrary` too, passing the
  canonical event body bytes as `data`.

### 3. `Cargo.toml` — make the legacy-bridge deps optional

`bip32`, `bip39` are currently unconditional dependencies. lytics-core
(the wasm tracker) cannot afford them — dropping bip32 was one of the
explicit size cuts that took the tracker from 172KB to ~26KB brotli.
gate them behind a feature (e.g. `bridge`, on by default for mudra's own
CLI/tests, off when a downstream crate like lytics-event depends on
`mudra` with `default-features = false`):

```toml
[features]
default = ["bridge"]
bridge = ["dep:bip32", "dep:bip39"]

[dependencies]
bip32 = { version = "0.5", optional = true }
bip39 = { version = "2", features = ["rand"], optional = true }
```

`seed.rs`/`claim::create`'s `SigningKey` type is currently sourced from
`bip32::secp256k1::ecdsa::SigningKey` (a re-export, to keep k256's
version locked to bip32's). with `bridge` off, that re-export vanishes —
`domain.rs` and the generalized `claim` signing functions need a
`SigningKey` that does not depend on `bip32` being present. resolve by
depending on `k256` directly (not through bip32's re-export) for the
parts that must work without `bridge`, and confirm the two `k256`
versions (bip32's pinned one, and a direct one) don't collide when both
are compiled in (mudra's own tests, `bridge` on, need both paths at
once).

### 4. lytics side

- `rs/event/Cargo.toml`: add `mudra = { path = "../../../mudra", default-features = false }`.
- `rs/event/src/keys.rs`: delete `domain_scalar`, `bech32_of_pubkey`; `Seed`/`Neuron`
  become thin wrappers over `mudra::domain`, keeping lytics's own
  `mnemonic` feature gate for the wordlist import/export path (that stays
  lytics-side — it is about *display*, not identity, and mudra's
  `bridge` feature is about a different mnemonic entirely, the Cosmos
  recovery one).
- `rs/event/src/sign.rs`: delete `sign_doc`; `sign_body`/`verify` call
  `mudra::claim::sign_arbitrary`/`verify_arbitrary`.
- every existing lytics-event test must still pass unchanged (they pin
  exact wire bytes / bech32 strings — this is a refactor, not a behavior
  change).

### 5. verify the constraint that matters

after the move, rebuild `lytics-core` (the wasm tracker) and re-measure
brotli size. the whole point of lytics's independent implementation was
avoiding bip32/serde_json in the wasm bundle — if depending on mudra
with `default-features = false` reintroduces any of that transitively,
this migration is a regression, not a cleanup, and needs to stop before
landing.

## what does NOT change

- `seed.rs`'s BIP32/Cosmos-path derivation — untouched, still the
  correct (only) way to recover an existing legacy account.
- `claim.rs`'s public behavior (`Claim::encode`/`decode`, `create`,
  `verify`) — same inputs, same outputs, same tests green.
- `cosmos.rs` — untouched, `domain.rs` calls into it rather than
  duplicating it.
- lytics's PoW, canonical encoding, event schema, ingest verification
  pipeline — none of this is mudra's concern, stays in `lytics-event`.
