---
tags: mudra, spec
crystal-type: spec
crystal-domain: comp
---

# bridge-proof — verifying a claim in the stack (Phase 2)

## goal

Phase 1 verifies a migration [[bridge]] claim natively (Rust `k256`). A verifier
is trusted to run `claim::verify` honestly. Phase 2 removes that trust: re-express
the verification — secp256k1 ECDSA + sha256 + ripemd160 — as a computation over
the Goldilocks field, run it on nox, and emit a **zheng proof**. Then a light
client checks the migration with no trusted operator.

The Phase-1 Rust is the reference; the Phase-2 gadget must match it bit-for-bit.

## the substrate (what the stack gives us)

nox reduces over **Goldilocks** `p = 2^64 − 2^32 + 1` only. Its patterns:
`add`/`sub`/`mul`/`inv` (field, tags 5–8), `xor`/`and`/`not`/`shl` (32-bit words,
tags 11–14), `eq`/`lt`, `hash` (Poseidon2). A run emits a `VecTrace` (rows of 16
Goldilocks columns); `zheng::commit(trace, …)` folds the rows by pattern and
produces the proof. secp256k1 lives in a **different, 256-bit field**
`q = 2^256 − 2^32 − 977`, so it must be *emulated* as limbs of Goldilocks. rune
and nox can't express it directly — Phase 2 is a Rust gadget over `nebu::Goldilocks`
whose every operation is one nox pattern.

## limb representation (the load-bearing choice)

A secp256k1 field element is **16 limbs of 16 bits**, little-endian, each limb a
Goldilocks element in `[0, 2^16)`. This is chosen so nothing ever wraps the
native field:

- a limb product is `< 2^32`
- a schoolbook column sums ≤ 16 such products: `< 16·2^32 = 2^36 ≪ p ≈ 2^64`

So every limb multiply and every column add is **exact inside one Goldilocks
element** — i.e. a legal `mul`/`add` nox step. Carry extraction (`digit = v & 0xFFFF`,
`carry = v >> 16`) is a **range-check** gadget in-circuit (`and`/`shl` + a bound
proof); in the reference it is a native shift.

Reduction mod `q` uses the Solinas structure `2^256 ≡ 2^32 + 977 (mod q)`: a
512-bit product folds by multiplying its high half by the small constant
`R = 2^32 + 977` and adding the low half, twice, then a final conditional
subtract of `q`. Every step is limb multiply/add — Goldilocks-native.

## milestones

| # | milestone | proves | status |
|---|-----------|--------|--------|
| 2a | secp256k1 base field `Fp` over Goldilocks limbs (add/sub/mul/inv/sqrt) | field arithmetic is exact and Goldilocks-native | **DONE** |
| 2b | secp256k1 curve: point add/double, scalar mul, decompress (Jacobian) | EC group law over the limb field | **DONE** |
| 2c | ECDSA verify equation: `R.x == r` from `(r,s,z,Q)` | the signature check itself | **DONE** |
| 2d | sha256 + ripemd160 as bit-op circuits (nox `and`/`xor`/`not`/`shl`) | address derivation + ADR-036 digest | **DONE** |
| 2e | nox program → `VecTrace` → `zheng::commit` → verify (pipeline) | a real, checkable zheng proof | **pipeline DONE; full-claim assembly remains** |

2a–2d express the full claim check (ECDSA + both hashes) in Goldilocks-limb /
word arithmetic — the reference gadget, every operation a nox pattern *by
construction*. Validated against `k256`, `num-bigint`, `sha2`, `ripemd`, and a
real Phase-1 claim (in-stack verify agrees with native on 40+ random signatures).

2e closes the proving *mechanism*: `proof::prove` (feature `prove`) builds a nox
program, runs `nox::reduce` to a `VecTrace`, `zheng::commit`s it, and
`zheng::verify`s the proof — demonstrated on the Goldilocks `a·b + c` that every
limb decomposes into, with a soundness check (understated `focus_bound`
rejected). **What remains:** emitting the *full* `verify_claim` as one nox
program is an arithmetization effort — the same proven operations at scale
(~10⁷ rows), a compiler/tracing task, not a new capability.

Each milestone is validated against the Phase-1 native path on shared inputs.

## milestone 2a — the base field

`proof::field::Fe` — an element of `𝔽_q`, `q = 2^256 − 2^32 − 977`:

```
Fe::from_bytes(&[u8;32]) -> Fe        // big-endian, reduced mod q
Fe::to_bytes(&self)      -> [u8;32]
Fe::add / sub / mul / sqr / inv       // field ops, all via Goldilocks limbs
Fe::ZERO / ONE / is_zero / ==
```

`mul` accumulates its 16×16 schoolbook columns with `nebu::Goldilocks`
operations, asserts each column `< Goldilocks::P` (the no-wraparound soundness
condition), carry-normalizes to a 512-bit limb vector, then Solinas-reduces mod
`q`. Verified against `num-bigint` (`BigUint`) modular arithmetic on thousands of
pseudo-random inputs plus edge cases (`0, 1, q−1`), and `a·a⁻¹ = 1`.

## constraint budget (estimate)

ECDSA verify is ~1 scalar-mul-heavy double-scalar multiplication: ~256 point ops,
each ~16 field muls, each 256 limb muls → order 10^6–10^7 Goldilocks steps, plus
sha256/ripemd160 bit circuits. ~10–20M trace rows per claim — a one-time,
per-neuron cost paid at mainnet migration, not at the spacepussy rehearsal.
