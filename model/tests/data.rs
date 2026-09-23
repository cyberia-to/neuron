use cell_model::{Builder, Content, Error, Reader, Source};

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

// `nox::encode::particle_of`'s pair case canonicalizes each 32-byte half's
// four Goldilocks limbs before hashing (`digest_from_bytes`), so a 64-byte
// pair node with an out-of-range limb (>= p) would otherwise hash the same
// as its canonical form: a second, malleable encoding of the same particle.
// `Content::validate` guards against this ahead of the id check itself
// (data.rs's canonical-limb loop runs before `particle_of` is even called),
// so a forged pair carrying it is rejected regardless of what id it claims.
// This branch had zero test coverage before this: nothing confirmed the
// guard actually fires, nor that a genuinely canonical limb (p - 1) passes
// it cleanly through to a real id match.
#[test]
fn non_canonical_pair_limb_is_rejected_ahead_of_the_id_check() {
    const GOLDILOCKS_P: u64 = 0xffff_ffff_0000_0001;

    let mut bytes = [0u8; 64];
    bytes[0..8].copy_from_slice(&GOLDILOCKS_P.to_le_bytes());
    let forged = Content {
        id: [7u8; 32],
        bytes: bytes.to_vec(),
        blob: false,
    };
    assert_eq!(forged.validate(), Err(Error::InvalidData));
}

#[test]
fn max_canonical_pair_limb_validates_against_its_real_id() {
    const GOLDILOCKS_P: u64 = 0xffff_ffff_0000_0001;

    let mut bytes = [0u8; 64];
    bytes[0..8].copy_from_slice(&(GOLDILOCKS_P - 1).to_le_bytes());
    let id = nox::encode::particle_of(&bytes).unwrap();
    let canonical = Content {
        id,
        bytes: bytes.to_vec(),
        blob: false,
    };
    assert_eq!(canonical.validate(), Ok(()));
}
