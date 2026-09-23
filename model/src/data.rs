use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};
pub type Particle = [u8; 32];
const MAX_NODES: usize = 131_072;
const CONTRACT: &str = include_str!("../schema-suite-v1.txt");

fn contract_id() -> Particle {
    static ID: OnceLock<Particle> = OnceLock::new();
    *ID.get_or_init(|| *hemera::hash(CONTRACT.as_bytes()).as_bytes())
}

fn schema_id(name: &str) -> Result<Particle, Error> {
    static IDS: OnceLock<Mutex<BTreeMap<String, Particle>>> = OnceLock::new();
    let mut ids = IDS
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| Error::InvalidData)?;
    if let Some(id) = ids.get(name) {
        return Ok(*id);
    }
    let id = Builder::new().schema(name)?;
    if ids.len() < 256 {
        ids.insert(name.to_owned(), id);
    }
    Ok(id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidData,
    UnsupportedSchema,
    Limit,
    Missing(Particle),
    Source(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cell data: {self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct Content {
    pub id: Particle,
    pub bytes: Vec<u8>,
    pub blob: bool,
}
impl Content {
    pub fn validate(&self) -> Result<(), Error> {
        if self.bytes.len() > 8 * 1024 * 1024 {
            return Err(Error::Limit);
        }
        if !self.blob && self.bytes.len() == 64 {
            for limb in self.bytes.chunks_exact(8) {
                if u64::from_le_bytes(limb.try_into().map_err(|_| Error::InvalidData)?)
                    >= 0xffff_ffff_0000_0001
                {
                    return Err(Error::InvalidData);
                }
            }
        }
        let id = if self.blob {
            *hemera::hash(&self.bytes).as_bytes()
        } else {
            nox::encode::particle_of(&self.bytes).map_err(|_| Error::InvalidData)?
        };
        if id != self.id {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
}
#[derive(Debug, Default)]
pub struct Builder {
    pub content: BTreeMap<Particle, Content>,
    schemas: BTreeMap<String, Particle>,
}
impl Builder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn blob(&mut self, bytes: Vec<u8>) -> Result<Particle, Error> {
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(Error::Limit);
        }
        self.insert(Content {
            id: *hemera::hash(&bytes).as_bytes(),
            bytes,
            blob: true,
        })
    }
    fn node(&mut self, bytes: Vec<u8>) -> Result<Particle, Error> {
        let id = nox::encode::particle_of(&bytes).map_err(|_| Error::InvalidData)?;
        self.insert(Content {
            id,
            bytes,
            blob: false,
        })
    }
    fn insert(&mut self, content: Content) -> Result<Particle, Error> {
        if !self.content.contains_key(&content.id) && self.content.len() >= MAX_NODES {
            return Err(Error::Limit);
        }
        let id = content.id;
        if let Some(prior) = self.content.get(&id) {
            if prior.blob != content.blob || prior.bytes != content.bytes {
                return Err(Error::InvalidData);
            }
        } else {
            self.content.insert(id, content);
        }
        Ok(id)
    }
    pub fn atom(&mut self, value: u64) -> Result<Particle, Error> {
        self.node(value.to_le_bytes().to_vec())
    }
    pub fn pair(&mut self, left: Particle, right: Particle) -> Result<Particle, Error> {
        self.node(nox::encode::encode_pair(&left, &right).to_vec())
    }
    pub fn uint(&mut self, value: u64) -> Result<Particle, Error> {
        let hi = self.atom(value >> 32)?;
        let lo = self.atom(value & 0xffff_ffff)?;
        self.pair(hi, lo)
    }
    pub fn reference(&mut self, id: Particle) -> Result<Particle, Error> {
        let mut fields = Vec::new();
        for bytes in id.chunks_exact(8) {
            fields.push(self.atom(u64::from_le_bytes(
                bytes.try_into().map_err(|_| Error::InvalidData)?,
            ))?);
        }
        let left = self.pair(fields[0], fields[1])?;
        let right = self.pair(fields[2], fields[3])?;
        self.pair(left, right)
    }
    pub fn fields(&mut self, values: &[Particle]) -> Result<Particle, Error> {
        let mut tail = self.atom(0)?;
        for value in values.iter().rev() {
            tail = self.pair(*value, tail)?;
        }
        Ok(tail)
    }
    pub fn list(&mut self, values: &[Particle]) -> Result<Particle, Error> {
        let count = self.uint(values.len() as u64)?;
        let fields = self.fields(values)?;
        self.pair(count, fields)
    }
    pub fn text(&mut self, value: &str) -> Result<Particle, Error> {
        if value.len() > 4096 {
            return Err(Error::Limit);
        }
        let values = value
            .bytes()
            .map(|v| self.atom(u64::from(v)))
            .collect::<Result<Vec<_>, _>>()?;
        self.list(&values)
    }
    pub fn optional(&mut self, value: Option<Particle>) -> Result<Particle, Error> {
        let tag = self.atom(u64::from(value.is_some()))?;
        let value = match value {
            Some(v) => v,
            None => self.atom(0)?,
        };
        self.pair(tag, value)
    }
    pub fn schema(&mut self, name: &str) -> Result<Particle, Error> {
        if let Some(id) = self.schemas.get(name) {
            return Ok(*id);
        }
        let contract = self.insert(Content {
            id: contract_id(),
            bytes: CONTRACT.as_bytes().to_vec(),
            blob: true,
        })?;
        let name_value = self.text(name)?;
        let contract_ref = self.reference(contract)?;
        let fields = self.fields(&[name_value, contract_ref])?;
        let marker = self.atom(0x53434831)?;
        let id = self.pair(marker, fields)?;
        self.schemas.insert(name.to_owned(), id);
        Ok(id)
    }
    pub fn record(&mut self, name: &str, fields: &[Particle]) -> Result<Particle, Error> {
        let schema = self.schema(name)?;
        let schema_ref = self.reference(schema)?;
        let fields = self.fields(fields)?;
        self.pair(schema_ref, fields)
    }
    pub fn map(&mut self, values: &BTreeMap<Particle, Particle>) -> Result<Particle, Error> {
        let entries = values
            .iter()
            .map(|(k, v)| {
                let key = self.reference(*k)?;
                let value = self.reference(*v)?;
                self.pair(key, value)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        self.list(&entries)
    }
}

pub trait Source {
    fn get(&self, id: &Particle) -> Result<Content, Error>;
}
impl Source for Builder {
    fn get(&self, id: &Particle) -> Result<Content, Error> {
        self.content.get(id).cloned().ok_or(Error::Missing(*id))
    }
}
pub struct Reader<'a, S: Source> {
    source: &'a S,
    remaining: usize,
}
impl<'a, S: Source> Reader<'a, S> {
    pub fn new(source: &'a S, limit: usize) -> Self {
        Self {
            source,
            remaining: limit,
        }
    }
    pub fn content(&mut self, id: Particle) -> Result<Content, Error> {
        self.remaining = self.remaining.checked_sub(1).ok_or(Error::Limit)?;
        let content = self.source.get(&id)?;
        if content.id != id {
            return Err(Error::InvalidData);
        }
        content.validate()?;
        Ok(content)
    }
    pub fn atom(&mut self, id: Particle) -> Result<u64, Error> {
        let content = self.content(id)?;
        if content.blob {
            return Err(Error::InvalidData);
        }
        Ok(u64::from_le_bytes(
            content
                .bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidData)?,
        ))
    }
    pub fn pair(&mut self, id: Particle) -> Result<(Particle, Particle), Error> {
        let content = self.content(id)?;
        if content.blob || content.bytes.len() != 64 {
            return Err(Error::InvalidData);
        }
        Ok((
            content.bytes[..32]
                .try_into()
                .map_err(|_| Error::InvalidData)?,
            content.bytes[32..]
                .try_into()
                .map_err(|_| Error::InvalidData)?,
        ))
    }
    pub fn uint(&mut self, id: Particle) -> Result<u64, Error> {
        let (hi, lo) = self.pair(id)?;
        let hi = self.atom(hi)?;
        let lo = self.atom(lo)?;
        if hi > u32::MAX as u64 || lo > u32::MAX as u64 {
            return Err(Error::InvalidData);
        }
        Ok((hi << 32) | lo)
    }
    pub fn reference(&mut self, id: Particle) -> Result<Particle, Error> {
        let (a, b) = self.pair(id)?;
        let (a, b1) = self.pair(a)?;
        let (c, d) = self.pair(b)?;
        let mut bytes = [0; 32];
        for (i, node) in [a, b1, c, d].into_iter().enumerate() {
            bytes[i * 8..i * 8 + 8].copy_from_slice(&self.atom(node)?.to_le_bytes());
        }
        Ok(bytes)
    }
    pub fn fields(&mut self, mut id: Particle, limit: usize) -> Result<Vec<Particle>, Error> {
        let zero = *hemera::tree::hash_leaf(&0u64.to_le_bytes(), 0, false).as_bytes();
        let mut values = Vec::new();
        while id != zero {
            if values.len() >= limit {
                return Err(Error::Limit);
            }
            let (h, t) = self.pair(id)?;
            values.push(h);
            id = t;
        }
        Ok(values)
    }
    pub fn list(&mut self, id: Particle, limit: usize) -> Result<Vec<Particle>, Error> {
        let (count, fields) = self.pair(id)?;
        let count = self.uint(count)?;
        if count > limit as u64 {
            return Err(Error::Limit);
        }
        let values = self.fields(fields, limit)?;
        if count != values.len() as u64 {
            return Err(Error::InvalidData);
        }
        Ok(values)
    }
    pub fn text(&mut self, id: Particle) -> Result<String, Error> {
        let fields = self.list(id, 4096)?;
        let bytes = fields
            .into_iter()
            .map(|id| u8::try_from(self.atom(id)?).map_err(|_| Error::InvalidData))
            .collect::<Result<Vec<_>, _>>()?;
        String::from_utf8(bytes).map_err(|_| Error::InvalidData)
    }
    pub fn optional(&mut self, id: Particle) -> Result<Option<Particle>, Error> {
        let (tag, value) = self.pair(id)?;
        match self.atom(tag)? {
            0 if self.atom(value)? == 0 => Ok(None),
            1 => Ok(Some(value)),
            _ => Err(Error::InvalidData),
        }
    }
    pub fn record(
        &mut self,
        id: Particle,
        name: &str,
        count: usize,
    ) -> Result<Vec<Particle>, Error> {
        let (schema, fields) = self.pair(id)?;
        if self.reference(schema)? != schema_id(name)? {
            return Err(Error::UnsupportedSchema);
        }
        let fields = self.fields(fields, count)?;
        if fields.len() != count {
            return Err(Error::InvalidData);
        }
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_and_uint_round_trip_including_values_above_u32() {
        let mut b = Builder::new();
        let small = b.atom(42).unwrap();
        let big = b.uint(0xdead_beef_1234_5678).unwrap();
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.atom(small).unwrap(), 42);
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.uint(big).unwrap(), 0xdead_beef_1234_5678);
    }

    #[test]
    fn pair_round_trips_its_two_components() {
        let mut b = Builder::new();
        let left = b.atom(1).unwrap();
        let right = b.atom(2).unwrap();
        let pair = b.pair(left, right).unwrap();
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.pair(pair).unwrap(), (left, right));
    }

    #[test]
    fn list_and_text_round_trip() {
        let mut b = Builder::new();
        let a = b.atom(10).unwrap();
        let c = b.atom(20).unwrap();
        let list = b.list(&[a, c]).unwrap();
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.list(list, 100).unwrap(), vec![a, c]);

        let mut b = Builder::new();
        let empty = b.list(&[]).unwrap();
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.list(empty, 100).unwrap(), Vec::<Particle>::new());

        let mut b = Builder::new();
        let text = b.text("hello").unwrap();
        let mut r = Reader::new(&b, 1000);
        assert_eq!(r.text(text).unwrap(), "hello");
    }

    #[test]
    fn list_over_limit_rejected_by_builder_and_reader() {
        let mut b = Builder::new();
        assert!(matches!(b.text(&"x".repeat(4097)), Err(Error::Limit)));

        let mut b = Builder::new();
        let a = b.atom(1).unwrap();
        let c = b.atom(2).unwrap();
        let list = b.list(&[a, c]).unwrap();
        let mut r = Reader::new(&b, 100);
        assert!(matches!(r.list(list, 1), Err(Error::Limit)));
    }

    #[test]
    fn optional_round_trips_some_and_none() {
        let mut b = Builder::new();
        let inner = b.atom(5).unwrap();
        let some = b.optional(Some(inner)).unwrap();
        let none = b.optional(None).unwrap();
        let mut r = Reader::new(&b, 100);
        assert_eq!(r.optional(some).unwrap(), Some(inner));
        assert_eq!(r.optional(none).unwrap(), None);
    }

    #[test]
    fn optional_rejects_an_invalid_tag() {
        let mut b = Builder::new();
        let value = b.atom(0).unwrap();
        let bad_tag = b.atom(2).unwrap();
        let malformed = b.pair(bad_tag, value).unwrap();
        let mut r = Reader::new(&b, 100);
        assert!(matches!(r.optional(malformed), Err(Error::InvalidData)));

        // tag says "present" (0) but the value slot is non-zero: also rejected.
        let mut b = Builder::new();
        let zero_tag = b.atom(0).unwrap();
        let nonzero = b.atom(9).unwrap();
        let malformed = b.pair(zero_tag, nonzero).unwrap();
        let mut r = Reader::new(&b, 100);
        assert!(matches!(r.optional(malformed), Err(Error::InvalidData)));
    }

    #[test]
    fn record_round_trips_and_rejects_the_wrong_schema_or_count() {
        let mut b = Builder::new();
        let f0 = b.atom(1).unwrap();
        let f1 = b.atom(2).unwrap();
        let record = b.record("cell/test-record/1", &[f0, f1]).unwrap();

        let mut r = Reader::new(&b, 100);
        assert_eq!(r.record(record, "cell/test-record/1", 2).unwrap(), vec![f0, f1]);

        let mut r = Reader::new(&b, 100);
        assert!(matches!(
            r.record(record, "cell/other-record/1", 2),
            Err(Error::UnsupportedSchema)
        ));

        // asking for more fields than the record actually carries: the field
        // chain hits its zero sentinel before `count`, so the length check
        // downstream of `fields()` catches it as invalid data.
        let mut r = Reader::new(&b, 100);
        assert!(matches!(
            r.record(record, "cell/test-record/1", 3),
            Err(Error::InvalidData)
        ));

        // asking for fewer fields than it carries: `fields()` itself hits its
        // own `limit` before reaching the zero sentinel.
        let mut r = Reader::new(&b, 100);
        assert!(matches!(
            r.record(record, "cell/test-record/1", 1),
            Err(Error::Limit)
        ));
    }

    #[test]
    fn reader_enforces_its_remaining_content_budget() {
        let mut b = Builder::new();
        let a = b.atom(1).unwrap();
        let mut r = Reader::new(&b, 0);
        assert!(matches!(r.atom(a), Err(Error::Limit)));

        let mut r = Reader::new(&b, 1);
        assert_eq!(r.atom(a).unwrap(), 1);
    }

    #[test]
    fn builder_insert_is_idempotent_for_identical_content() {
        let mut b = Builder::new();
        let first = b.atom(5).unwrap();
        let second = b.atom(5).unwrap();
        assert_eq!(first, second);
        assert_eq!(b.content.len(), 1);
    }

    #[test]
    fn builder_blob_and_text_enforce_their_size_limits() {
        let mut b = Builder::new();
        assert!(matches!(
            b.blob(vec![0u8; 8 * 1024 * 1024 + 1]),
            Err(Error::Limit)
        ));
        assert!(b.blob(vec![0u8; 8 * 1024 * 1024]).is_ok());
    }
}
