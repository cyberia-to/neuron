#![cfg(feature = "records")]
use neuron_model::{
    Builder, Lifecycle, Reader,
    execution::{Invocation, NeuronState, PendingOperation, Prog, Status},
};
use std::collections::BTreeMap;
fn job() -> Invocation {
    Invocation {
        prog: [1; 32],
        code: [2; 32],
        input: [3; 32],
        context: None,
        base_revision: 0,
        checkpoint: Some([4; 32]),
        status: Status::Running,
        limit: 200,
        charged: 0,
        reserved: 0,
        used: 0,
        pending: None,
        result: None,
        fault: None,
        parent: None,
        delegated: 0,
        children: Vec::new(),
        epoch: 0,
        ordinal: 0,
    }
}
fn root() -> NeuronState {
    NeuronState {
        neuron: [8; 32],
        network: [9; 32],
        policy: [10; 32],
        epoch: 0,
        limit: 500,
        charged: 0,
        held: 400,
        progs: BTreeMap::from([(
            [1; 32],
            Prog {
                source: [2; 32],
                state: [3; 32],
                revision: 0,
                lifecycle: Lifecycle::Active,
                step_limit: 200,
                max_inflight: 2,
                allowed_acts: vec![6],
            },
        )]),
        invocations: BTreeMap::from([([5; 32], job()), ([6; 32], job())]),
        imports: BTreeMap::new(),
        cursor: 0,
        writer_generation: 0,
        worker: None,
    }
}
#[test]
fn budgets_and_parent_cycles_cannot_be_encoded_as_valid_state() {
    let root = root();
    let mut b = Builder::new();
    let id = root.encode(&mut b).unwrap();
    let restored = NeuronState::read(&mut Reader::new(&b, 100_000), id).unwrap();
    assert_eq!(restored.held, 400);
    let mut over = root.clone();
    over.charged = u64::MAX;
    assert!(over.encode(&mut b).is_err());
    let mut double = root.clone();
    double.held = 200;
    assert!(double.encode(&mut b).is_err());
    let mut cycle = root.clone();
    cycle.held = 0;
    let first = cycle.invocations.get_mut(&[5; 32]).unwrap();
    first.parent = Some([6; 32]);
    first.children = vec![[6; 32]];
    first.delegated = 200;
    let second = cycle.invocations.get_mut(&[6; 32]).unwrap();
    second.parent = Some([5; 32]);
    second.children = vec![[5; 32]];
    second.delegated = 200;
    assert!(cycle.encode(&mut b).is_err());
    let mut duplicate = root;
    duplicate.progs.get_mut(&[1; 32]).unwrap().allowed_acts = vec![6, 6];
    assert!(duplicate.encode(&mut b).is_err());
}
#[test]
fn unknown_effect_cannot_be_mislabeled_as_terminal_or_already_resolved() {
    let mut root = root();
    let job = root.invocations.get_mut(&[5; 32]).unwrap();
    job.status = Status::Waiting;
    job.pending = Some(PendingOperation {
        id: [11; 32],
        record: [12; 32],
        tag: 6,
        arguments: [3; 32],
        attempt: Some([13; 32]),
        outcome: None,
        stage: 1,
        value: None,
        failed: false,
        dispatch: None,
        attempt_record: None,
    });
    let mut b = Builder::new();
    let id = root.encode(&mut b).unwrap();
    let restored = NeuronState::read(&mut Reader::new(&b, 100_000), id).unwrap();
    assert_eq!(
        restored.invocations[&[5; 32]]
            .pending
            .as_ref()
            .unwrap()
            .stage,
        1
    );
    let mut changed = root.clone();
    let job = changed.invocations.get_mut(&[5; 32]).unwrap();
    job.status = Status::Completed;
    job.result = Some([14; 32]);
    assert!(changed.encode(&mut b).is_err());
    let pending = root
        .invocations
        .get_mut(&[5; 32])
        .unwrap()
        .pending
        .as_mut()
        .unwrap();
    pending.stage = 2;
    assert!(root.encode(&mut b).is_err());
}
