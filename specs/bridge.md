---
tags: mudra, spec
crystal-type: spec
crystal-domain: comp
---

# bridge — the legacy-key migration path

## why this exists

mudra's native identity is signature-free: `neuron = Hemera(secret)`, and
authentication is a zheng proof of hash preimage (see [[neuron/specs/proof-authority|proof-based authority]]). But an
existing Cosmos-SDK network — spacepussy first — holds accounts under
**secp256k1** keys. Migrating the graph by snapshot moves the neuron *records*,
yet leaves them orphaned: native auth is the preimage of a *new* secret nobody
has. The bridge returns control to the original holder.

This is the one place a classical signature is structurally required. Everywhere
else, proofs replace signatures; here we must accept the proof the holder can
already produce — a secp256k1 signature — exactly once, to bind their legacy key
to a native neuron.

## the two things migration needs

| step | mechanism | needs secp256k1 |
|------|-----------|-----------------|
| move the graph | bijective snapshot → replay → **state-root match** | no |
| return control | holder signs `legacy address → native neuron` | **yes** |

The snapshot is graded by the root match, not by signatures. The bridge covers
only the second step.

## pipeline

```
spell words ──BIP-39──▶ 64-byte material ──BIP-32/44 (m/44'/118'/0'/0/0)──▶ secp256k1 key
                                                                  │
                          compressed pubkey (33 B) ◀─────────────┘
                                   │
        ripemd160(sha256(pubkey))  │  = account id (20 B)
                                   ▼
               bech32(hrp, account_id)  = address   (hrp: pussy | bostrom)

        neuron = Hemera(compressed_pubkey)          (native identity, 32 B)

        claim  = ADR-036 sign(  "cyber:migrate:v1:" ‖ hex(neuron)  )
        verify = address derives from pubkey  ∧  signature covers the doc
```

Coin type **118** and path `m/44'/118'/0'/0/0` are the Cosmos standard; both
spacepussy (`pussy`) and bostrom (`bostrom`) use them.

## claim format

A claim is `(address, pubkey, neuron, signature)`:

- `address` — the legacy bech32 account the holder controls
- `pubkey` — 33-byte SEC1-compressed secp256k1 public key
- `neuron` — 32-byte native neuron id being bound (default `Hemera(pubkey)`;
  a holder re-keying to a post-quantum secret binds `Hemera(new_secret)`)
- `signature` — 64-byte ECDSA (low-S, RFC-6979 deterministic) over the ADR-036
  sign doc

The signed message uses **ADR-036** ("sign arbitrary data" — what a wallet's
`signArbitrary` produces), so a holder builds a claim from any standard wallet
with no on-chain transaction. The sign doc is the amino-JSON canonical form
(keys sorted, no whitespace):

```json
{"account_number":"0","chain_id":"","fee":{"amount":[],"gas":"0"},"memo":"","msgs":[{"type":"sign/MsgSignData","value":{"data":"<base64(message)>","signer":"<address>"}}],"sequence":"0"}
```

`verify` accepts a claim iff **both** hold:
1. `bech32(hrp, ripemd160(sha256(pubkey))) == address` — the pubkey owns the address
2. the signature verifies over the doc for this exact `neuron` — the holder
   authorized this binding and no other

Tampering with the neuron, the pubkey, or the chain (hrp) fails verification.

## trust model — phase 1 vs phase 2

**Phase 1 (this spec, implemented).** Verification is native: a verifier runs
`verify` in-process. Whoever ingests claims at migration is trusted to run it
honestly. Adequate for the spacepussy rehearsal.

**Phase 2 (planned).** Re-express `verify` — secp256k1 ECDSA + sha256 +
ripemd160 — as a nox program that emits a zheng proof. secp256k1's 256-bit
fields become 4×64-bit Goldilocks limbs with range checks; point add/double and
scalar-mul become gadgets; sha256/ripemd160 become bit-op circuits (nox has
`and`/`xor`/`not`/`shl`). Then the binding is trustless: a light client verifies
the migration with no trusted operator. Cost is ~10–20M constraints per
signature — a one-time per-neuron price paid at mainnet, not at the rehearsal.

The Phase-1 Rust code is the **reference implementation** the Phase-2 proof must
match bit-for-bit.

## surface

```
spell::derive(spell, passphrase)        -> [u8; 64]
spell::signing_key(material, path)           -> SigningKey
spell::cosmos_key(spell, passphrase)  -> SigningKey        // m/44'/118'/0'/0/0

cosmos::compressed(verifying_key)       -> [u8; 33]
cosmos::account_id(pubkey)              -> [u8; 20]          // ripemd160(sha256(pk))
cosmos::address(pubkey, hrp)           -> String            // bech32
cosmos::{PUSSY, BOSTROM}                                     // hrp constants

claim::neuron_of(pubkey)                -> [u8; 32]          // Hemera(pubkey)
claim::create(key, hrp, neuron)         -> Claim
claim::verify(claim, hrp)              -> bool
```

## known-answer

The all-`abandon` BIP-39 phrase, cross-checked against an independent pure-Python
secp256k1 BIP-32 derivation:

```
pubkey  024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62
pussy   pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm
```

Guarded by `tests/vectors.rs`.
