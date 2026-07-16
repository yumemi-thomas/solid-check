use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[test]
fn persistent_sidecar_returns_the_compiler_native_execution_map() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_solid-compiler-facts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start sidecar");
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap()).lines();

    writeln!(input, "not-json").unwrap();
    let invalid: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(invalid["ok"], false);
    assert_eq!(invalid["error"]["code"], "invalid-request");

    let source = "const view = <div>{count()}</div>;";
    let source_hash = format!("sha256:{:x}", Sha256::digest(source.as_bytes()));
    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": source,
            "sourceHash": "sha256:stale",
            "compilerOptions": { "moduleName": "dom", "generate": "dom" }
        })
    )
    .unwrap();
    let stale: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(stale["ok"], false);
    assert_eq!(stale["error"]["code"], "analysis-failed");
    assert!(stale["error"]["message"]
        .as_str()
        .unwrap()
        .contains("source hash"));

    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": source,
            "sourceHash": source_hash,
            "compilerOptions": {
                "moduleName": "dom",
                "generate": "dom",
                "effectWrapper": false
            }
        })
    )
    .unwrap();
    let unsupported: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(unsupported["ok"], false);
    assert_eq!(unsupported["error"]["code"], "invalid-request");
    assert!(unsupported["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown field"));

    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": source,
            "sourceHash": source_hash,
            "compilerOptions": { "moduleName": "dom", "generate": "dom" }
        })
    )
    .unwrap();
    let response: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["executionMap"]["sourceHash"], source_hash);
    assert_eq!(
        response["executionMap"]["trackedRegions"][0]["reason"],
        "jsx-child"
    );

    drop(input);
    assert!(child.wait().unwrap().success());
}
