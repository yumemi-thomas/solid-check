use std::{env, fs, path::PathBuf, process::Command};

use solid_facts_backend::{TypeFactsProvider, TypeFactsSidecar};
use solid_facts_core::Generation;
use solid_ts_facts::ClosureRequest;
use solid_ts_facts::v3::{FileChange, Operation, Request, TYPE_FACTS_SCHEMA_V3};

#[test]
fn timing_lines_are_valid_json() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .args(["--format", "json", "--project"])
        .arg(root.join("internal/reactiveir/testdata/tracer/tsconfig.json"))
        .args(["--typefacts", &typefacts])
        .env("SOLID_CHECK_TIMINGS", "1")
        .output()
        .expect("run Rust CLI with timings");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("timings are UTF-8");
    let timings = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid timing JSON"))
        .collect::<Vec<_>>();
    assert!(
        timings
            .iter()
            .any(|timing| timing["reactiveIrStage"].is_string()),
        "expected reactive IR stage timings: {timings:#?}"
    );
}

#[cfg(unix)]
#[test]
fn cli_rejects_a_mismatched_typefacts_build() {
    use std::os::unix::fs::PermissionsExt;

    let directory = env::temp_dir().join(format!("solid-check-handshake-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let service = directory.join("mismatched-typefacts");
    let handshake = solid_ts_facts::v3::Handshake {
        protocol: solid_ts_facts::v3::TYPE_FACTS_HANDSHAKE_PROTOCOL,
        schema_hash: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_SHA256.into(),
        build_id: "definitely-not-this-engine".into(),
    };
    let payload = solid_ts_facts::encode(&handshake).unwrap();
    let mut frame = u32::try_from(payload.len()).unwrap().to_le_bytes().to_vec();
    frame.extend(payload);
    let escaped = frame
        .iter()
        .map(|byte| format!("\\{byte:03o}"))
        .collect::<String>();
    fs::write(
        &service,
        format!("#!/bin/sh\nprintf '{escaped}'\ncat >/dev/null\n"),
    )
    .unwrap();
    fs::set_permissions(&service, fs::Permissions::from_mode(0o755)).unwrap();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .args(["--typefacts"])
        .arg(&service)
        .args(["--project"])
        .arg(root.join("internal/reactiveir/testdata/tracer/tsconfig.json"))
        .output()
        .expect("run Rust CLI with mismatched service");
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("compatibility handshake failed"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn frozen_cbor_exchanges_with_the_go_tsgo_service() {
    let executable = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("internal/reactiveir/testdata/tracer/tsconfig.json");
    let project_id = project.canonicalize().expect("canonical tsconfig");
    let args = vec!["-project".into(), project_id.to_string_lossy().into_owned()];
    let mut service = TypeFactsSidecar::spawn(&executable, &args).expect("spawn Go TS-Go service");
    let request = ClosureRequest::new(
        project_id.to_string_lossy(),
        Generation::new(1).unwrap(),
        vec![],
    )
    .unwrap();
    let response = service.closure(&request).expect("exchange TypeFacts v2");
    assert_eq!(response.project_id, request.project_id);
    assert!(!response.table.sources.is_empty());
}

fn lifecycle_request(operation: Operation, project_id: String, generation: u64) -> Request {
    Request {
        schema: TYPE_FACTS_SCHEMA_V3,
        request_id: 0,
        operation,
        project_id,
        generation,
        changes: vec![],
        structural_spans: vec![],
        compiler_spans: vec![],
        demands: vec![],
        compact_demands: None,
        state_token: String::new(),
        reset_state: false,
        removed_demand_paths: vec![],
        cancel_request_id: 0,
    }
}

#[test]
fn v3_updates_and_reanalyzes_a_retained_project() {
    let executable = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture.join("tsconfig.json").canonicalize().unwrap();
    let project_id = project.to_string_lossy().into_owned();
    let mut service =
        TypeFactsSidecar::spawn(&executable, &["-project".into(), project_id.clone()]).unwrap();
    let sources = service
        .lifecycle(lifecycle_request(Operation::Sources, project_id.clone(), 1))
        .unwrap()
        .sources;
    assert!(
        sources
            .iter()
            .any(|source| source.path.ends_with("App.tsx") && !source.source.is_empty())
    );
    service
        .lifecycle(lifecycle_request(Operation::Open, project_id.clone(), 1))
        .unwrap();
    let app = fixture.join("App.tsx").canonicalize().unwrap();
    let mut update = lifecycle_request(Operation::Update, project_id.clone(), 2);
    update.changes.push(FileChange {
        path: app.to_string_lossy().into_owned(),
        version: 1,
        source: fs::read(&app).unwrap(),
        deleted: false,
    });
    service.lifecycle(update).unwrap();
    assert!(
        service
            .lifecycle(lifecycle_request(Operation::Analyze, project_id.clone(), 2))
            .unwrap()
            .table
            .is_some()
    );
    // A stateful reset analyze answers with the compact full frame, and the
    // compact demand snapshot round-trips through the service.
    let mut reset = lifecycle_request(Operation::Analyze, project_id.clone(), 2);
    reset.reset_state = true;
    reset.compact_demands = Some(solid_ts_facts::v3::compact_demands(&[
        solid_ts_facts::v3::EntityDemand {
            r#async: false,
            symbol: true,
            location: solid_ts_facts::Location {
                path: app.to_string_lossy().into_owned(),
                start_byte: 0,
                end_byte: 1,
            },
            references: false,
            resolved_call: false,
            query_location: None,
            type_descriptor: false,
            structural_accessor: false,
        },
    ]));
    let stateful = service.lifecycle(reset).unwrap();
    assert_eq!(stateful.table_mode, "full");
    assert!(stateful.table.is_none());
    let compact = stateful.compact_table.expect("compact full frame");
    assert!(compact.expand().is_ok());
    assert!(
        service
            .lifecycle(lifecycle_request(Operation::Analyze, project_id, 1))
            .is_err()
    );
}
