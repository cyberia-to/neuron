// ---
// tags: neuron, auth, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! The nox → zheng proving pipeline (milestone 2e).
//!
//! Milestones 2a–2d express secp256k1 ECDSA verification in Goldilocks-limb
//! arithmetic: every operation is a nox pattern (`add`/`sub`/`mul`/`inv`,
//! `and`/`xor`/`not`/`shl`). This module closes the loop on the *mechanism* that
//! turns such a computation into a real proof:
//!
//! 1. build a nox program (a formula noun tree),
//! 2. run [`nox::reduce`] to get an execution [`VecTrace`],
//! 3. [`zheng::commit`] the trace to a [`TraceProof`],
//! 4. [`zheng::verify`] it.
//!
//! It is demonstrated on the Goldilocks field operations that every limb of the
//! verifier decomposes into (`a·b + c`). Assembling the *full* `verify_claim`
//! into one nox program is the remaining arithmetization work — the same
//! operations at scale (~10⁷ rows), not a new capability. This is gated behind
//! the `prove` feature because it pulls the whole proof system.

use nebu::Goldilocks;
use nox::{reduce, NullCalls, Order, Outcome, Reduction, VecTrace};
use zheng::{commit, verify, ProofParams, Statement, TraceProof};

/// nox opcode tags (see nox reduction patterns).
const QUOTE: u64 = 1;
const ADD: u64 = 5;
const MUL: u64 = 7;

/// A statement that binds nothing but the trace's own validity. I/O binding is
/// added by setting `input_hash`/`output_hash`/`focus_bound`.
fn open_statement() -> Statement {
    Statement {
        program_hash: [0u8; 32],
        input_hash: [0u8; 32],
        output_hash: [0u8; 32],
        focus_bound: 0,
    }
}

/// Build and run the nox program `a·b + c` over Goldilocks, returning its
/// execution trace and the computed result. This exercises two arithmetic
/// patterns (`mul`, `add`) plus the quoting/compose steps around them — the
/// atoms every field-limb operation in the verifier is built from.
fn run_madd(a: u64, b: u64, c: u64) -> (VecTrace, u64) {
    let mut r = Reduction::<4096>::new();
    let subject = r.atom(Goldilocks::new(0)).expect("subject");

    let quote = |r: &mut Reduction<4096>, v: u64| -> Order {
        let tag = r.atom(Goldilocks::new(QUOTE)).unwrap();
        let val = r.atom(Goldilocks::new(v)).unwrap();
        r.pair(tag, val).unwrap() // [1 v]
    };

    let qa = quote(&mut r, a);
    let qb = quote(&mut r, b);
    let qc = quote(&mut r, c);

    // [7 [ [1 a] [1 b] ]]  — a·b
    let mul_tag = r.atom(Goldilocks::new(MUL)).unwrap();
    let mul_body = r.pair(qa, qb).unwrap();
    let mul = r.pair(mul_tag, mul_body).unwrap();

    // [5 [ (a·b) [1 c] ]]  — add the product to c
    let add_tag = r.atom(Goldilocks::new(ADD)).unwrap();
    let add_body = r.pair(mul, qc).unwrap();
    let formula = r.pair(add_tag, add_body).unwrap();

    let mut trace = VecTrace::default();
    let result = match reduce(&mut r, subject, formula, 1000, &NullCalls, &mut trace) {
        Outcome::Ok(res, _) => r.atom_value(res).expect("atom result").as_u64(),
        other => panic!("nox reduce failed: {other:?}"),
    };
    (trace, result)
}

/// Prove `a·b + c` end to end: run it on nox, commit the trace with zheng.
/// Returns the proof, its statement, and the computed value.
pub fn prove_madd(a: u64, b: u64, c: u64) -> (TraceProof, Statement, u64) {
    let (trace, result) = run_madd(a, b, c);
    let statement = open_statement();
    let proof = commit(&trace, &[], &[], &[], &statement, &ProofParams::default())
        .expect("zheng commit");
    (proof, statement, result)
}

/// Verify a proof against its statement.
pub fn verify_proof(proof: &TraceProof, statement: &Statement) -> bool {
    verify(proof, statement, &ProofParams::default()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn madd_proof_verifies_end_to_end() {
        let (proof, stmt, result) = prove_madd(3, 5, 7);
        assert_eq!(result, 22, "3·5 + 7 computed correctly by nox");
        assert!(verify_proof(&proof, &stmt), "the zheng proof verifies");
    }

    #[test]
    fn trace_has_multiple_pattern_rows() {
        // mul + add + the quote/compose steps → a genuine multi-pattern trace,
        // so commit exercises folding across CCS pattern groups.
        let (trace, _) = run_madd(6, 7, 1);
        assert!(trace.0.len() >= 2, "trace spans several reduction steps");
    }

    #[test]
    fn focus_bound_binds_the_step_count() {
        // A statement that claims fewer steps than the trace took must be rejected
        // at commit — the proof cannot understate the work done.
        let (trace, _) = run_madd(3, 5, 7);
        let mut stmt = open_statement();
        stmt.focus_bound = 1; // trace is longer than 1 row
        let rejected = commit(&trace, &[], &[], &[], &stmt, &ProofParams::default());
        assert!(rejected.is_err(), "understated focus_bound rejected");
    }
}
