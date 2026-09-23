//! Decoder for a capture produced by the unmodified v1 binary before migration.
use neuron_engine::{Error, GraphPort};
use neuron_model::{Builder, Content, Head, Particle};
use neuron_model::{Lifecycle, Reader, Value::*};
use std::collections::BTreeMap;
pub const CAPTURE: &[u8] = include_bytes!("../fixtures/legacy-v1.capture");
pub fn named(name: &str) -> Particle {
    let value = include_str!("../fixtures/legacy-v1.txt")
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{name}=")))
        .unwrap();
    std::array::from_fn(|i| u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).unwrap())
}
struct Input<'a>(&'a [u8]);
impl Input<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], Error> {
        if n > self.0.len() {
            return Err(Error::Conflict);
        }
        let (left, right) = self.0.split_at(n);
        self.0 = right;
        Ok(left)
    }
    fn id(&mut self) -> Result<Particle, Error> {
        self.take(32)?.try_into().map_err(|_| Error::Conflict)
    }
    fn uint(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| Error::Conflict)?,
        ))
    }
    fn count(&mut self, max: u64) -> Result<usize, Error> {
        let n = self.uint()?;
        if n > max {
            return Err(Error::Budget);
        }
        Ok(n as usize)
    }
    fn flag(&mut self) -> Result<bool, Error> {
        match self.take(1)? {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(Error::Conflict),
        }
    }
    fn head(&mut self) -> Result<Head, Error> {
        Ok(Head {
            index: self.uint()?,
            commit: self.id()?,
        })
    }
}
pub fn replay(graph: &impl GraphPort, bytes: &[u8]) -> Result<(), Error> {
    let mut r = Input(bytes);
    if r.take(8)? != b"CELLCAP1" {
        return Err(Error::Unsupported);
    }
    let mut contents = BTreeMap::new();
    for _ in 0..r.count(131_072)? {
        let id = r.id()?;
        let blob = r.flag()?;
        let len = r.count(8 * 1024 * 1024)?;
        let content = Content {
            id,
            blob,
            bytes: r.take(len)?.to_vec(),
        };
        content.validate()?;
        if contents.insert(id, content).is_some() {
            return Err(Error::Conflict);
        }
    }
    for _ in 0..r.count(4096)? {
        let namespace = r.id()?;
        let request = r.id()?;
        let expected = if r.flag()? { Some(r.head()?) } else { None };
        let head = r.head()?;
        let mut b = Builder::new();
        for _ in 0..r.count(131_072)? {
            let id = r.id()?;
            b.content
                .insert(id, contents.get(&id).ok_or(Error::Missing)?.clone());
        }
        let claim = if r.flag()? {
            Some((r.id()?, r.id()?))
        } else {
            None
        };
        graph.commit(namespace, request, expected, head, b, claim)?;
    }
    if !r.0.is_empty() {
        return Err(Error::Conflict);
    }
    Ok(())
}

/// Test-only old-format successor, independent of the retired v1 engine.
#[allow(dead_code)]
pub fn advance(graph: &impl GraphPort, origin: Particle) -> Result<Head, Error> {
    let view = neuron_engine::legacy::inspect(graph, origin)?;
    let mut r = Reader::new(graph, 50_000);
    let fields = if view.head.index == 0 {
        r.record(view.head.commit, "cell/birth/1", 7)?
    } else {
        r.record(view.head.commit, "cell/commit/1", 10)?
    };
    let before = r.reference(fields[if view.head.index == 0 { 5 } else { 4 }])?;
    let mut snapshot = view.snapshot;
    snapshot.lifecycle = Lifecycle::Paused;
    let mut b = Builder::new();
    let after = snapshot.encode(&mut b)?;
    let owner = b.blob(b"cell/local-private-database-owner/1".to_vec())?;
    let nonce = b.nonce([91; 32])?;
    let payload = b.artifact(b"test legacy successor".to_vec(), "text/plain")?;
    let event = b.values(
        "cell/event/1",
        &[
            Ref(owner),
            Raw(nonce),
            Ref(origin),
            Text("$pause"),
            Ref(payload),
            Optional(None),
            Refs(&[]),
            Optional(None),
            Optional(None),
            Ref(snapshot.authority_policy),
        ],
    )?;
    let request = b.values("cell/request-id/1", &[Ref(owner), Raw(nonce)])?;
    let index = view.head.index + 1;
    let commit = b.values(
        "cell/commit/1",
        &[
            Ref(origin),
            Uint(index),
            Ref(view.head.commit),
            Ref(before),
            Ref(after),
            Refs(&[event]),
            Refs(&[]),
            Ref(snapshot.authority_policy),
            Uint(snapshot.epoch),
            Refs(&[]),
        ],
    )?;
    graph.commit(
        origin,
        request,
        Some(view.head),
        Head { index, commit },
        b,
        Some((request, event)),
    )
}
