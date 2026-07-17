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
                "unknownOption": false
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
    let repeated: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(repeated, response);

    let unicode_source = "const emoji = '😀';\r\nconst view = <div>{東京()}</div>;";
    let unicode_hash = format!("sha256:{:x}", Sha256::digest(unicode_source.as_bytes()));
    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/東京.tsx",
            "source": unicode_source,
            "sourceHash": unicode_hash,
            "compilerOptions": {
                "moduleName": "dom",
                "generate": "dom",
                "hydratable": true,
                "dev": true
            }
        })
    )
    .unwrap();
    let unicode: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(unicode["ok"], true);
    assert_eq!(
        unicode["executionMap"]["trackedRegions"][0]["span"]["start"],
        unicode_source.find("東京()").unwrap()
    );

    let control_source = "const view = <For each={items()}>{item => <span>{item()}</span>}</For>;";
    let control_hash = format!("sha256:{:x}", Sha256::digest(control_source.as_bytes()));
    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": control_source,
            "sourceHash": control_hash,
            "compilerOptions": {
                "moduleName": "dom",
                "generate": "dom",
                "builtIns": ["For"]
            }
        })
    )
    .unwrap();
    let control: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(control["ok"], true);
    assert!(control["executionMap"]["callbackRoles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|role| role["role"] == "render"));

    let untracked_source = "const view = <div title={label}>{/*@static*/ count()}</div>;";
    let untracked_hash = format!("sha256:{:x}", Sha256::digest(untracked_source.as_bytes()));
    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": untracked_source,
            "sourceHash": untracked_hash,
            "compilerOptions": { "moduleName": "dom", "generate": "dom" }
        })
    )
    .unwrap();
    let untracked: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(untracked["ok"], true);
    assert_eq!(untracked["executionMap"]["trackedRegions"], json!([]));
    let label_start = untracked_source.find("label").unwrap();
    let count_start = untracked_source.find("count()").unwrap();
    assert_eq!(
        untracked["executionMap"]["untrackedRegions"],
        json!([
            {
                "span": { "start": label_start, "end": label_start + "label".len() },
                "reason": "jsx-attribute"
            },
            {
                "span": { "start": count_start, "end": count_start + "count()".len() },
                "reason": "jsx-child"
            }
        ])
    );
    assert_eq!(
        untracked["executionMap"]["jsxOperations"],
        json!([{
            "span": { "start": count_start, "end": count_start + "count()".len() },
            "kind": "jsx-expression"
        }])
    );

    let static_props_source = "const view = <Comp note={label}>{0}</Comp>;";
    let static_props_hash = format!(
        "sha256:{:x}",
        Sha256::digest(static_props_source.as_bytes())
    );
    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": static_props_source,
            "sourceHash": static_props_hash,
            "compilerOptions": { "moduleName": "dom", "generate": "dom" }
        })
    )
    .unwrap();
    let static_props: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(static_props["ok"], true);
    assert_eq!(static_props["executionMap"]["callbackRoles"], json!([]));
    let note_start = static_props_source.find("label").unwrap();
    let child_start = static_props_source.find("{0}").unwrap() + 1;
    assert_eq!(
        static_props["executionMap"]["untrackedRegions"],
        json!([
            {
                "span": { "start": note_start, "end": note_start + "label".len() },
                "reason": "component-getter"
            },
            {
                "span": { "start": child_start, "end": child_start + 1 },
                "reason": "component-getter"
            }
        ])
    );

    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "/workspace/App.tsx",
            "source": source,
            "sourceHash": source_hash,
            "compilerOptions": { "moduleName": "dom", "generate": "ssr" }
        })
    )
    .unwrap();
    let unsupported_mode: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(unsupported_mode["ok"], false);
    assert!(unsupported_mode["error"]["message"]
        .as_str()
        .unwrap()
        .contains("DOM output only"));

    writeln!(
        input,
        "{}",
        json!({
            "compilerFactsProtocol": 1,
            "path": "",
            "source": source,
            "sourceHash": source_hash,
            "compilerOptions": { "moduleName": "dom", "generate": "dom" }
        })
    )
    .unwrap();
    let missing_path: Value = serde_json::from_str(&output.next().unwrap().unwrap()).unwrap();
    assert_eq!(missing_path["ok"], false);
    assert!(missing_path["error"]["message"]
        .as_str()
        .unwrap()
        .contains("path"));

    drop(input);
    assert!(child.wait().unwrap().success());
}
