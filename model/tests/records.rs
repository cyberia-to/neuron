use cell_model::{Builder, Error, Head, Lifecycle, Particle, Reader, Snapshot, Value};
use std::collections::BTreeMap;

fn particle(byte: u8) -> Particle {
    [byte; 32]
}

fn sample_snapshot(lifecycle: Lifecycle) -> Snapshot {
    Snapshot {
        definition: particle(1),
        application_state: particle(2),
        lifecycle,
        authority_policy: particle(3),
        profile: particle(4),
        epoch: 99,
        inbox: particle(5),
        continuations: particle(6),
        outbox: particle(7),
        subscriptions: particle(8),
        management: particle(9),
    }
}

#[test]
fn head_round_trips_index_and_commit() {
    let mut b = Builder::new();
    let head = Head {
        index: 42,
        commit: particle(7),
    };
    let id = b.head(head).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.head(id).unwrap(), head);
}

#[test]
fn nonce_round_trips_through_fields() {
    let mut b = Builder::new();
    let mut bytes = [0u8; 32];
    for (i, v) in bytes.iter_mut().enumerate() {
        *v = i as u8;
    }
    let id = b.nonce(bytes).unwrap();
    let mut r = Reader::new(&b, 100);
    let words = r.fields(id, 8).unwrap();
    assert_eq!(words.len(), 8);
    let mut decoded = [0u8; 32];
    for (i, w) in words.into_iter().enumerate() {
        let value = r.atom(w).unwrap();
        decoded[i * 4..i * 4 + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
    assert_eq!(decoded, bytes);
}

#[test]
fn lifecycle_names() {
    assert_eq!(Lifecycle::Installed.name(), "installed");
    assert_eq!(Lifecycle::Active.name(), "active");
    assert_eq!(Lifecycle::Paused.name(), "paused");
    assert_eq!(Lifecycle::Retiring.name(), "retiring");
    assert_eq!(Lifecycle::Retired.name(), "retired");
}

#[test]
fn lifecycle_variants_encode_to_distinct_and_deterministic_particles() {
    let variants = [
        Lifecycle::Installed,
        Lifecycle::Active,
        Lifecycle::Paused,
        Lifecycle::Retiring,
        Lifecycle::Retired,
    ];
    let ids: Vec<Particle> = variants
        .iter()
        .map(|v| v.encode(&mut Builder::new()).unwrap())
        .collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "{} vs {}", variants[i].name(), variants[j].name());
        }
    }
    // a fresh Builder must encode the same variant to the same particle:
    // Snapshot::read's lifecycle lookup depends on this determinism.
    assert_eq!(ids[0], Lifecycle::Installed.encode(&mut Builder::new()).unwrap());
}

#[test]
fn snapshot_round_trips_every_lifecycle_variant() {
    for lifecycle in [
        Lifecycle::Installed,
        Lifecycle::Active,
        Lifecycle::Paused,
        Lifecycle::Retiring,
        Lifecycle::Retired,
    ] {
        let mut b = Builder::new();
        let snap = sample_snapshot(lifecycle);
        let id = snap.encode(&mut b).unwrap();
        let mut r = Reader::new(&b, 100);
        let decoded = Snapshot::read(&mut r, id).unwrap();
        assert_eq!(decoded.definition, snap.definition);
        assert_eq!(decoded.application_state, snap.application_state);
        assert_eq!(decoded.lifecycle, snap.lifecycle);
        assert_eq!(decoded.authority_policy, snap.authority_policy);
        assert_eq!(decoded.profile, snap.profile);
        assert_eq!(decoded.epoch, snap.epoch);
        assert_eq!(decoded.inbox, snap.inbox);
        assert_eq!(decoded.continuations, snap.continuations);
        assert_eq!(decoded.outbox, snap.outbox);
        assert_eq!(decoded.subscriptions, snap.subscriptions);
        assert_eq!(decoded.management, snap.management);
    }
}

#[test]
fn snapshot_read_rejects_a_non_snapshot_schema() {
    let mut b = Builder::new();
    let not_a_snapshot = b.record("test/not-a-snapshot/1", &[]).unwrap();
    let mut r = Reader::new(&b, 100);
    assert!(matches!(
        Snapshot::read(&mut r, not_a_snapshot),
        Err(Error::UnsupportedSchema)
    ));
}

#[test]
fn artifact_round_trips_bytes_and_media() {
    let mut b = Builder::new();
    let id = b.artifact(b"hello world".to_vec(), "text/plain").unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.artifact(id).unwrap(), b"hello world".to_vec());
}

#[test]
fn artifact_rejects_a_forged_codec() {
    let mut b = Builder::new();
    let content = b.blob(b"hello".to_vec()).unwrap();
    let wrong_codec = b.blob(b"not-the-real-codec".to_vec()).unwrap();
    let id = b
        .values(
            "cell/artifact/1",
            &[
                Value::Ref(content),
                Value::Ref(wrong_codec),
                Value::Uint(5),
                Value::Text("text/plain"),
            ],
        )
        .unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.artifact(id), Err(Error::UnsupportedSchema));
}

#[test]
fn artifact_rejects_a_declared_length_that_does_not_match_the_bytes() {
    let mut b = Builder::new();
    let content = b.blob(b"hello".to_vec()).unwrap();
    let codec = b.blob(b"hemera/bytes/0.3".to_vec()).unwrap();
    let id = b
        .values(
            "cell/artifact/1",
            &[
                Value::Ref(content),
                Value::Ref(codec),
                Value::Uint(999),
                Value::Text("text/plain"),
            ],
        )
        .unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.artifact(id), Err(Error::InvalidData));
}

#[test]
fn empty_map_round_trips() {
    let mut b = Builder::new();
    let id = b.empty_map().unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.map(id, 10).unwrap(), Vec::new());
}

#[test]
fn map_round_trips_sorted_entries() {
    let mut b = Builder::new();
    let mut entries = BTreeMap::new();
    entries.insert(particle(1), particle(10));
    entries.insert(particle(2), particle(20));
    let id = b.map(&entries).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(
        r.map(id, 10).unwrap(),
        vec![(particle(1), particle(10)), (particle(2), particle(20))]
    );
}

// `commit_dim` in a sibling repo (bbg) enforces this same "sorted map, no
// duplicate keys" contract at the same layer; this codec's own `Reader::map`
// had zero direct tests confirming it actually rejects, rather than
// silently accepting, a wire-supplied map that violates it.
#[test]
fn map_rejects_entries_out_of_ascending_key_order() {
    let mut b = Builder::new();
    let k1 = b.reference(particle(1)).unwrap();
    let v1 = b.reference(particle(10)).unwrap();
    let k2 = b.reference(particle(2)).unwrap();
    let v2 = b.reference(particle(20)).unwrap();
    let e1 = b.pair(k1, v1).unwrap();
    let e2 = b.pair(k2, v2).unwrap();
    let id = b.list(&[e2, e1]).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.map(id, 10), Err(Error::InvalidData));
}

#[test]
fn map_rejects_a_duplicate_key() {
    let mut b = Builder::new();
    let k = b.reference(particle(1)).unwrap();
    let v1 = b.reference(particle(10)).unwrap();
    let v2 = b.reference(particle(20)).unwrap();
    let e1 = b.pair(k, v1).unwrap();
    let e2 = b.pair(k, v2).unwrap();
    let id = b.list(&[e1, e2]).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.map(id, 10), Err(Error::InvalidData));
}

// `optional_ref` decodes its `Some` payload through `reference()`, so the
// node it is given must itself be `Builder::reference`-encoded (exactly
// what `Value::Optional`'s own branch in `values()` does before calling
// `Builder::optional`) — not a bare particle passed straight through.
#[test]
fn optional_ref_round_trips_some_and_none() {
    let mut b = Builder::new();
    let some_ref = b.reference(particle(5)).unwrap();
    let some_id = b.optional(Some(some_ref)).unwrap();
    let none_id = b.optional(None).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.optional_ref(some_id).unwrap(), Some(particle(5)));
    assert_eq!(r.optional_ref(none_id).unwrap(), None);
}

#[test]
fn values_covers_optional_refs_and_raw_branches() {
    let mut b = Builder::new();
    let raw_node = b.atom(2).unwrap();
    let id = b
        .values(
            "test/values/1",
            &[
                Value::Optional(Some(particle(9))),
                Value::Optional(None),
                Value::Refs(&[particle(1), particle(2)]),
                Value::Raw(raw_node),
            ],
        )
        .unwrap();
    let mut r = Reader::new(&b, 100);
    let fields = r.record(id, "test/values/1", 4).unwrap();
    assert_eq!(r.optional_ref(fields[0]).unwrap(), Some(particle(9)));
    assert_eq!(r.optional_ref(fields[1]).unwrap(), None);
    let refs = r.list(fields[2], 10).unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(r.reference(refs[0]).unwrap(), particle(1));
    assert_eq!(r.reference(refs[1]).unwrap(), particle(2));
    assert_eq!(fields[3], raw_node);
}
