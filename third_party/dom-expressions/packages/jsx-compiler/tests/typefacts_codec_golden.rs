use std::{fs, path::PathBuf};

use ciborium::value::Value;

#[test]
fn typefacts_v1_golden_decodes_and_reencodes_identically() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/phase1/typefacts-v1-golden.cbor");
    let encoded = fs::read(path).expect("read TypeFacts v1 golden");
    let value: Value =
        ciborium::from_reader(encoded.as_slice()).expect("decode deterministic CBOR");
    let Value::Map(entries) = &value else {
        panic!("TypeFacts response must be a map");
    };
    assert!(entries.iter().any(|(key, value)| {
        matches!((key, value), (Value::Text(key), Value::Integer(value)) if key == "schema" && i128::from(*value) == 1)
    }));
    let mut repeated = Vec::new();
    ciborium::into_writer(&value, &mut repeated).expect("re-encode deterministic CBOR");
    assert_eq!(repeated, encoded);
}
