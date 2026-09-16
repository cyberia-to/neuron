#![cfg(feature = "records")]
use neuron_model::{Builder, Content, Error, Reader, Source};

#[test]
fn full_width_uint_and_balanced_particle_round_trip() {
    let mut b = Builder::new();
    let uint = b.uint(u64::MAX).unwrap();
    let particle = b.blob(b"artifact".to_vec()).unwrap();
    let reference = b.reference(particle).unwrap();
    let mut r = Reader::new(&b, 100);
    assert_eq!(r.uint(uint).unwrap(), u64::MAX);
    assert_eq!(r.reference(reference).unwrap(), particle);
    assert!(b.atom(0xffff_ffff_0000_0001).is_err());
}

#[test]
fn malformed_collection_schema_and_forged_bytes_are_rejected() {
    let mut b = Builder::new();
    let one = b.atom(1).unwrap();
    let record = b.record("test/one/1", &[one]).unwrap();
    assert_eq!(
        Reader::new(&b, 100).record(record, "test/two/1", 1),
        Err(Error::UnsupportedSchema)
    );
    let count = b.uint(2).unwrap();
    let fields = b.fields(&[one]).unwrap();
    let list = b.pair(count, fields).unwrap();
    assert_eq!(Reader::new(&b, 100).list(list, 5), Err(Error::InvalidData));
    assert_eq!(Reader::new(&b, 0).atom(one), Err(Error::Limit));
    struct Forged(Content);
    impl Source for Forged {
        fn get(&self, _: &[u8; 32]) -> Result<Content, Error> {
            Ok(self.0.clone())
        }
    }
    let forged = Forged(Content {
        id: one,
        bytes: 2u64.to_le_bytes().to_vec(),
        blob: false,
    });
    assert_eq!(Reader::new(&forged, 100).atom(one), Err(Error::InvalidData));
}
