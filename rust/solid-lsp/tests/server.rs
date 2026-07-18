use std::{env, fs, path::PathBuf};

use serde_json::{Value, json};

fn frame(message: Value) -> Vec<u8> {
    let payload = serde_json::to_vec(&message).unwrap();
    let mut result = format!("Content-Length: {}\r\n\r\n", payload.len()).into_bytes();
    result.extend(payload);
    result
}

fn decode_frames(mut bytes: &[u8]) -> Vec<Value> {
    let mut messages = Vec::new();
    while !bytes.is_empty() {
        let boundary = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("LSP header boundary");
        let header = std::str::from_utf8(&bytes[..boundary]).unwrap();
        let length = header
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length:"))
            .expect("Content-Length")
            .trim()
            .parse::<usize>()
            .unwrap();
        bytes = &bytes[boundary + 4..];
        messages.push(serde_json::from_slice(&bytes[..length]).unwrap());
        bytes = &bytes[length..];
    }
    messages
}

#[test]
fn initializes_publishes_native_diagnostics_and_shuts_down() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("internal/reactiveir/testdata/tracer/tsconfig.json");
    let source = root
        .join("internal/reactiveir/testdata/tracer/App.tsx")
        .canonicalize()
        .unwrap();
    let source_uri = format!("file://{}", source.to_string_lossy());

    let mut input = Vec::new();
    for message in [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"capabilities": {}}
        }),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/diagnostic",
            "params": {"textDocument": {"uri": source_uri}}
        }),
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ] {
        input.extend(frame(message));
    }

    let mut output = Vec::new();
    solid_lsp::serve(
        &project,
        &typefacts,
        std::io::Cursor::new(input),
        &mut output,
    )
    .unwrap();
    let messages = decode_frames(&output);

    assert_eq!(
        messages[0].pointer("/result/capabilities/positionEncoding"),
        Some(&json!("utf-16"))
    );
    let publication = messages
        .iter()
        .find(|message| {
            message.get("method") == Some(&json!("textDocument/publishDiagnostics"))
                && message.pointer("/params/uri") == Some(&json!(source_uri))
        })
        .unwrap_or_else(|| panic!("diagnostic publication for tracer fixture: {messages:#?}"));
    let diagnostic = publication
        .pointer("/params/diagnostics/0")
        .expect("tracer diagnostic");
    assert_eq!(diagnostic.get("code"), Some(&json!("SC1001")));
    assert_eq!(
        diagnostic.pointer("/data/rule"),
        Some(&json!("strict-read-untracked"))
    );
    assert!(
        diagnostic
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .is_some()
    );

    let pull = messages
        .iter()
        .find(|message| message.get("id") == Some(&json!(2)))
        .expect("pull diagnostic response");
    assert_eq!(pull.pointer("/result/kind"), Some(&json!("full")));
    assert_eq!(pull.pointer("/result/items/0/code"), Some(&json!("SC1001")));
    assert_eq!(
        messages
            .iter()
            .find(|message| message.get("id") == Some(&json!(3)))
            .and_then(|message| message.get("result")),
        Some(&Value::Null)
    );
}

#[test]
fn returns_standard_json_rpc_errors_and_ignores_unknown_notifications() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("internal/reactiveir/testdata/tracer/tsconfig.json");
    let mut input = b"Content-Length: 1\r\n\r\n{".to_vec();
    for message in [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({"jsonrpc": "2.0", "method": "future/notification"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "future/method"}),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/didChange",
            "params": {"contentChanges": "broken"}
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ] {
        input.extend(frame(message));
    }

    let mut output = Vec::new();
    solid_lsp::serve(
        &project,
        &typefacts,
        std::io::Cursor::new(input),
        &mut output,
    )
    .unwrap();
    let messages = decode_frames(&output);
    assert_eq!(messages.len(), 5, "{messages:#?}");
    let codes = messages
        .iter()
        .map(|message| {
            (
                message.get("id").cloned().unwrap_or(Value::Null),
                message.pointer("/error/code").and_then(Value::as_i64),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        vec![
            (Value::Null, Some(-32700)),
            (json!(1), None),
            (json!(2), Some(-32601)),
            (json!(3), Some(-32602)),
            (json!(4), None),
        ]
    );
}

#[test]
fn rejects_stale_versions_and_waits_for_the_accepted_edit() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("internal/reactiveir/testdata/tracer-corrected/tsconfig.json");
    let source = root
        .join("internal/reactiveir/testdata/tracer-corrected/App.tsx")
        .canonicalize()
        .unwrap();
    let uri = format!("file://{}", source.to_string_lossy());
    let unsafe_source =
        fs::read_to_string(root.join("internal/reactiveir/testdata/tracer/App.tsx")).unwrap();
    let safe_source = fs::read_to_string(&source).unwrap();
    let mut input = Vec::new();
    for message in [
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri,
                "languageId": "typescriptreact",
                "version": 5,
                "text": unsafe_source
            }}
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 4},
                "contentChanges": [{"text": safe_source}]
            }
        }),
        json!({"jsonrpc": "2.0", "id": 10, "method": "solid/checkSnapshot"}),
    ] {
        input.extend(frame(message));
    }

    let mut output = Vec::new();
    solid_lsp::serve(
        &project,
        &typefacts,
        std::io::Cursor::new(input),
        &mut output,
    )
    .unwrap();
    let messages = decode_frames(&output);
    let snapshot = messages
        .iter()
        .find(|message| message.get("id") == Some(&json!(10)))
        .expect("snapshot after accepted edit");
    assert_eq!(
        snapshot.pointer("/result/status"),
        Some(&json!("violation"))
    );
    assert_eq!(
        snapshot.pointer("/result/findings/0/id"),
        Some(&json!("SC1001"))
    );
    let publication = messages
        .iter()
        .find(|message| message.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .expect("diagnostics for accepted edit");
    assert_eq!(publication.pointer("/params/version"), Some(&json!(5)));
}

#[test]
fn superseding_edit_wins_and_pending_snapshot_can_be_cancelled() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("internal/reactiveir/testdata/tracer-corrected/tsconfig.json");
    let source = root
        .join("internal/reactiveir/testdata/tracer-corrected/App.tsx")
        .canonicalize()
        .unwrap();
    let uri = format!("file://{}", source.to_string_lossy());
    let unsafe_source =
        fs::read_to_string(root.join("internal/reactiveir/testdata/tracer/App.tsx")).unwrap();
    let safe_source = fs::read_to_string(&source).unwrap();
    let mut input = Vec::new();
    for message in [
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri,
                "languageId": "typescriptreact",
                "version": 1,
                "text": unsafe_source
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 20, "method": "solid/checkSnapshot"}),
        json!({
            "jsonrpc": "2.0",
            "method": "$/cancelRequest",
            "params": {"id": 20}
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": safe_source}]
            }
        }),
        json!({"jsonrpc": "2.0", "id": 21, "method": "solid/checkSnapshot"}),
    ] {
        input.extend(frame(message));
    }

    let mut output = Vec::new();
    solid_lsp::serve(
        &project,
        &typefacts,
        std::io::Cursor::new(input),
        &mut output,
    )
    .unwrap();
    let messages = decode_frames(&output);
    assert_eq!(
        messages
            .iter()
            .find(|message| message.get("id") == Some(&json!(20)))
            .and_then(|message| message.pointer("/error/code")),
        Some(&json!(-32800))
    );
    let snapshot = messages
        .iter()
        .find(|message| message.get("id") == Some(&json!(21)))
        .expect("snapshot after superseding edit");
    assert_eq!(
        snapshot.pointer("/result/status"),
        Some(&json!("certified"))
    );
    assert_eq!(snapshot.pointer("/result/findings"), Some(&json!([])));
    let final_publication = messages
        .iter()
        .rev()
        .find(|message| message.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .expect("clearing diagnostics for superseding edit");
    assert_eq!(
        final_publication.pointer("/params/version"),
        Some(&json!(2))
    );
    assert_eq!(
        final_publication.pointer("/params/diagnostics"),
        Some(&json!([]))
    );
}
