use crate::{Builder, Error, Particle, Reader, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Head {
    pub index: u64,
    pub commit: Particle,
}

pub enum Value<'a> {
    Atom(u64),
    Uint(u64),
    Ref(Particle),
    Optional(Option<Particle>),
    Text(&'a str),
    Refs(&'a [Particle]),
    Raw(Particle),
}
impl Builder {
    pub fn nonce(&mut self, bytes: [u8; 32]) -> Result<Particle, Error> {
        let words = bytes
            .chunks_exact(4)
            .map(|v| {
                self.atom(u64::from(u32::from_le_bytes(
                    v.try_into().map_err(|_| Error::InvalidData)?,
                )))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.fields(&words)
    }
    pub fn values(&mut self, name: &str, values: &[Value<'_>]) -> Result<Particle, Error> {
        let mut fields = Vec::new();
        for value in values {
            fields.push(match value {
                Value::Atom(n) => self.atom(*n)?,
                Value::Uint(n) => self.uint(*n)?,
                Value::Ref(id) => self.reference(*id)?,
                Value::Text(s) => self.text(s)?,
                Value::Optional(id) => {
                    let node = id.map(|id| self.reference(id)).transpose()?;
                    self.optional(node)?
                }
                Value::Refs(ids) => {
                    let refs = ids
                        .iter()
                        .map(|id| self.reference(*id))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.list(&refs)?
                }
                Value::Raw(node) => *node,
            });
        }
        self.record(name, &fields)
    }
    pub fn head(&mut self, head: Head) -> Result<Particle, Error> {
        self.values(
            "cell/head/1",
            &[Value::Uint(head.index), Value::Ref(head.commit)],
        )
    }
    pub fn artifact(&mut self, bytes: Vec<u8>, media: &str) -> Result<Particle, Error> {
        let len = bytes.len() as u64;
        let content = self.blob(bytes)?;
        let codec = self.blob(b"hemera/bytes/0.3".to_vec())?;
        self.values(
            "cell/artifact/1",
            &[
                Value::Ref(content),
                Value::Ref(codec),
                Value::Uint(len),
                Value::Text(media),
            ],
        )
    }
    pub fn variant(&mut self, name: &str, fields: &[Value<'_>]) -> Result<Particle, Error> {
        self.values(&format!("cell/{name}/1"), fields)
    }
    pub fn empty_map(&mut self) -> Result<Particle, Error> {
        self.map(&Default::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Installed,
    Active,
    Paused,
    Retiring,
    Retired,
}
impl Lifecycle {
    pub fn name(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Retiring => "retiring",
            Self::Retired => "retired",
        }
    }
    pub fn encode(self, b: &mut Builder) -> Result<Particle, Error> {
        b.variant(&format!("lifecycle/{}", self.name()), &[])
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub definition: Particle,
    pub application_state: Particle,
    pub lifecycle: Lifecycle,
    pub authority_policy: Particle,
    pub profile: Particle,
    pub epoch: u64,
    pub inbox: Particle,
    pub continuations: Particle,
    pub outbox: Particle,
    pub subscriptions: Particle,
    pub management: Particle,
}
impl Snapshot {
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        use Value::*;
        let lifecycle = self.lifecycle.encode(b)?;
        b.values(
            "cell/snapshot/1",
            &[
                Ref(self.definition),
                Ref(self.application_state),
                Raw(lifecycle),
                Ref(self.authority_policy),
                Ref(self.profile),
                Uint(self.epoch),
                Ref(self.inbox),
                Ref(self.continuations),
                Ref(self.outbox),
                Ref(self.subscriptions),
                Ref(self.management),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let f = r.record(id, "cell/snapshot/1", 11)?;
        let lifecycle = [
            Lifecycle::Installed,
            Lifecycle::Active,
            Lifecycle::Paused,
            Lifecycle::Retiring,
            Lifecycle::Retired,
        ]
        .into_iter()
        .find(|v| v.encode(&mut Builder::new()).ok() == Some(f[2]))
        .ok_or(Error::UnsupportedSchema)?;
        Ok(Self {
            definition: r.reference(f[0])?,
            application_state: r.reference(f[1])?,
            lifecycle,
            authority_policy: r.reference(f[3])?,
            profile: r.reference(f[4])?,
            epoch: r.uint(f[5])?,
            inbox: r.reference(f[6])?,
            continuations: r.reference(f[7])?,
            outbox: r.reference(f[8])?,
            subscriptions: r.reference(f[9])?,
            management: r.reference(f[10])?,
        })
    }
}

impl<S: Source> Reader<'_, S> {
    pub fn optional_ref(&mut self, id: Particle) -> Result<Option<Particle>, Error> {
        self.optional(id)?.map(|v| self.reference(v)).transpose()
    }
    pub fn head(&mut self, id: Particle) -> Result<Head, Error> {
        let f = self.record(id, "cell/head/1", 2)?;
        Ok(Head {
            index: self.uint(f[0])?,
            commit: self.reference(f[1])?,
        })
    }
    pub fn artifact(&mut self, id: Particle) -> Result<Vec<u8>, Error> {
        let f = self.record(id, "cell/artifact/1", 4)?;
        let content = self.reference(f[0])?;
        let codec = self.reference(f[1])?;
        if codec != *hemera::hash(b"hemera/bytes/0.3").as_bytes() {
            return Err(Error::UnsupportedSchema);
        }
        let expected = self.uint(f[2])?;
        let content = self.content(content)?;
        if !content.blob || content.bytes.len() as u64 != expected {
            return Err(Error::InvalidData);
        }
        Ok(content.bytes)
    }
    pub fn map(&mut self, id: Particle, limit: usize) -> Result<Vec<(Particle, Particle)>, Error> {
        let mut out = Vec::new();
        for entry in self.list(id, limit)? {
            let (k, v) = self.pair(entry)?;
            let key = self.reference(k)?;
            let value = self.reference(v)?;
            if out.last().is_some_and(|(prior, _)| prior >= &key) {
                return Err(Error::InvalidData);
            }
            out.push((key, value));
        }
        Ok(out)
    }
}
