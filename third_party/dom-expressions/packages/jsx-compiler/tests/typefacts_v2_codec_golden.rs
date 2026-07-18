use std::{fs, path::PathBuf};

use ciborium::value::Value;

fn assert_golden_round_trips(file: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/phase1")
        .join(file);
    let encoded = fs::read(path).expect("read TypeFacts v2 golden");
    let value: Value =
        ciborium::from_reader(encoded.as_slice()).expect("decode deterministic CBOR");
    let Value::Map(entries) = &value else {
        panic!("TypeFacts v2 message must be a map");
    };
    assert!(entries.iter().any(|(key, value)| {
        matches!((key, value), (Value::Text(key), Value::Integer(value)) if key == "schema" && i128::from(*value) == 2)
    }));
    let mut repeated = Vec::new();
    ciborium::into_writer(&value, &mut repeated).expect("re-encode deterministic CBOR");
    assert_eq!(repeated, encoded);
}

#[test]
fn typefacts_v2_request_golden_decodes_and_reencodes_identically() {
    assert_golden_round_trips("typefacts-v2-request-golden.cbor");
}

#[test]
fn typefacts_v2_response_golden_decodes_and_reencodes_identically() {
    assert_golden_round_trips("typefacts-v2-golden.cbor");
}
