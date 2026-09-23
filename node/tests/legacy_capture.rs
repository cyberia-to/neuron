#[path = "support/legacy.rs"]
mod legacy;
mod support;
use neuron_engine::legacy::inspect;
use neuron_model::Reader;
use support::*;

#[test]
fn unmodified_v1_contents_heads_claims_and_unknown_attempt_remain_readable() {
    let dir = Directory::new();
    let graph = dir.graph();
    legacy::replay(&graph, legacy::CAPTURE).unwrap();
    let counter = inspect(&graph, legacy::named("counter")).unwrap();
    assert_eq!(
        Reader::new(&graph, 20_000)
            .artifact(counter.snapshot.application_state)
            .unwrap(),
        value("7")
    );
    assert!(counter.live.is_none());
    assert!(counter.charged > 0);
    // The generator's historical label means the admission receipt's commit.
    assert_eq!(
        neuron_engine::legacy::resolve(&graph, counter.origin, [2; 32])
            .unwrap()
            .unwrap()
            .commit,
        legacy::named("completed_request")
    );
    let tool = inspect(&graph, legacy::named("tool")).unwrap();
    let pending = tool.live.unwrap().pending.unwrap();
    assert_eq!(pending.id, legacy::named("operation"));
    assert_eq!(pending.stage, "attempt-recorded");
    let mut r = Reader::new(&graph, 20_000);
    let f = r
        .record(pending.attempt.unwrap(), "cell/attempt/1", 7)
        .unwrap();
    assert_eq!(r.reference(f[0]).unwrap(), legacy::named("attempt"));
    let reserved = inspect(&graph, legacy::named("reserved")).unwrap();
    assert_eq!(reserved.live.unwrap().reserved, 1000);
}
