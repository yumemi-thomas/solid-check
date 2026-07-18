use std::io::Write as _;
use std::process::{Command, Stdio};
use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use solid_compiler_facts::CompilerOptions;
use solid_facts_backend::{
    CacheStats, CompilerSidecar, IncrementalSession, NativeIncrementalSession, SourceChange,
    SourceFile, TypeFactsProvider, TypeFactsSidecar, build_project,
};
use solid_facts_core::Generation;
use solid_ts_facts::ClosureRequest;
use solid_ts_facts::v3::{FileChange, Operation, Request, TYPE_FACTS_SCHEMA_V3};

fn decode_findings(output: &[u8]) -> Vec<serde_json::Value> {
    let snapshot: serde_json::Value = serde_json::from_slice(output).expect("decode snapshot");
    snapshot["findings"]
        .as_array()
        .expect("snapshot findings")
        .clone()
}

fn diagnostic_fixture(name: &str) -> Option<Vec<serde_json::Value>> {
    let typefacts = env::var("SOLID_TYPEFACTS_BIN").ok()?;
    let compiler = env::var("SOLID_COMPILER_FACTS_BIN").ok()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join(format!("internal/reactiveir/testdata/{name}/tsconfig.json"));
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", typefacts)
        .env("SOLID_COMPILER_FACTS_BIN", compiler)
        .args(["--format", "json", "--project"])
        .arg(project)
        .output()
        .expect("run Rust diagnostic CLI");
    assert!(
        output.status.success(),
        "fixture {name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(decode_findings(&output.stdout))
}

fn findings_for_rule<'a>(
    findings: &'a [serde_json::Value],
    rule: &str,
) -> Vec<&'a serde_json::Value> {
    findings
        .iter()
        .filter(|finding| finding["rule"] == rule)
        .collect()
}

fn assert_rule_findings<'a>(
    findings: &'a [serde_json::Value],
    rule: &str,
    expected: usize,
) -> Vec<&'a serde_json::Value> {
    let selected = findings_for_rule(findings, rule);
    assert_eq!(selected.len(), expected, "rule {rule}: {selected:#?}");
    for finding in &selected {
        assert!(
            finding["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("SC")),
            "rule {rule} must have a stable diagnostic code"
        );
        assert!(
            finding["message"]
                .as_str()
                .is_some_and(|message| !message.is_empty()),
            "rule {rule} must explain the violation"
        );
        assert!(
            finding["primaryLocation"]["path"]
                .as_str()
                .is_some_and(|path| path.ends_with(".ts") || path.ends_with(".tsx")),
            "rule {rule} must point to source"
        );
        assert!(
            finding["evidence"]
                .as_array()
                .is_some_and(|evidence| !evidence.is_empty()),
            "rule {rule} must carry evidence"
        );
    }
    selected
}

fn temporary_directory(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "solid-check-rust-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn rust_cli_with_arguments_does_not_wait_for_stdin_eof() {
    // A stdin pipe that never reaches EOF must not block argv invocations;
    // only argument-less calls read a JSON request from stdin.
    let mut child = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .arg("--help")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn Rust CLI with open stdin");
    let stdin = child.stdin.take().expect("keep stdin pipe open");
    let deadline = SystemTime::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll Rust CLI") {
            break status;
        }
        if SystemTime::now() > deadline {
            child.kill().unwrap();
            panic!("Rust CLI blocked on stdin EOF despite argv invocation");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    drop(stdin);
    assert!(status.success());
}

#[test]
fn argument_less_invocation_still_accepts_a_json_request_on_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn Rust CLI for stdin request");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            br#"{"projectId":"stdin-mode","generation":1,"sources":[],"typefactsExecutable":"","help":true}"#,
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr = {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Usage: solid-check-rust"),
        "stdout = {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[cfg(unix)]
#[test]
fn rust_cli_rejects_mismatched_typefacts_build_with_distinct_exit_code() {
    use std::os::unix::fs::PermissionsExt;

    let directory = temporary_directory("handshake-mismatch");
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
        .args([
            "--typefacts",
            &service.to_string_lossy(),
            "--project",
            &root
                .join("internal/reactiveir/testdata/tracer/tsconfig.json")
                .to_string_lossy(),
        ])
        .output()
        .expect("run Rust CLI with mismatched service");
    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("compatibility handshake failed"),
        "stderr = {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn exchanges_frozen_cbor_with_go_tsgo_service() {
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

#[test]
fn incremental_session_reuses_unchanged_file_facts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture.join("tsconfig.json").canonicalize().unwrap();
    let project_id = project.to_string_lossy().into_owned();
    let paths = [fixture.join("App.tsx"), fixture.join("source.ts")];
    let sources = paths
        .iter()
        .map(|path| SourceFile {
            path: path.canonicalize().unwrap().to_string_lossy().into_owned(),
            source: std::fs::read_to_string(path).unwrap(),
            compiler_options: CompilerOptions::default(),
        })
        .collect();
    let compiler = CompilerSidecar::spawn(&compiler, &[]).unwrap();
    let typescript =
        TypeFactsSidecar::spawn(&typefacts, &["-project".into(), project_id.clone()]).unwrap();
    let mut session = IncrementalSession::open(project_id, sources, compiler, typescript).unwrap();
    session.analyze().unwrap();
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 2,
            compiler_entries: 2
        }
    );
    let app = paths[0].canonicalize().unwrap();
    let source = format!("// edit\n{}", std::fs::read_to_string(&app).unwrap());
    session
        .update(vec![SourceChange {
            path: app.to_string_lossy().into_owned(),
            version: 1,
            source: Some(source),
            compiler_options: CompilerOptions::default(),
        }])
        .unwrap();
    assert_eq!(session.generation(), 2);
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 1,
            compiler_entries: 1
        }
    );
    session.analyze().unwrap();
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 2,
            compiler_entries: 2
        }
    );
}

#[test]
fn native_incremental_session_reuses_oxc_and_solid_facts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture.join("tsconfig.json").canonicalize().unwrap();
    let project_id = project.to_string_lossy().into_owned();
    let paths = [fixture.join("App.tsx"), fixture.join("source.ts")];
    let sources = paths
        .iter()
        .map(|path| SourceFile {
            path: path.canonicalize().unwrap().to_string_lossy().into_owned(),
            source: fs::read_to_string(path).unwrap(),
            compiler_options: CompilerOptions::default(),
        })
        .collect();
    let typescript =
        TypeFactsSidecar::spawn(&typefacts, &["-project".into(), project_id.clone()]).unwrap();
    let mut session = NativeIncrementalSession::open(project_id, sources, typescript).unwrap();
    session.analyze().unwrap();
    let first_timings = session.last_build_timings();
    assert!(first_timings.exchange.server_materialized);
    assert!(!first_timings.exchange.roundtrip.is_zero());
    assert!(!first_timings.exchange.response_decode.is_zero());
    assert!(first_timings.exchange.response_bytes > 0);
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 2,
            compiler_entries: 2
        }
    );
    // A no-op analysis must reuse both native fact domains without growing
    // either cache.
    session.analyze().unwrap();
    assert!(!session.last_build_timings().exchange.server_materialized);
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 2,
            compiler_entries: 2
        }
    );
    let app = paths[0].canonicalize().unwrap();
    // One edit exchange: the update and the analysis of the new generation
    // travel as a single session call; the changed file's cache entries are
    // invalidated and repopulated within it.
    let before_recovery = session
        .edit(
            vec![SourceChange {
                path: app.to_string_lossy().into_owned(),
                version: 1,
                source: Some(format!("// edit\n{}", fs::read_to_string(&app).unwrap())),
                compiler_options: CompilerOptions::default(),
            }],
            None,
        )
        .unwrap();
    assert_eq!(session.generation(), 2);
    assert_eq!(
        session.cache_stats(),
        CacheStats {
            ast_entries: 2,
            compiler_entries: 2
        }
    );
    session.recover_typefacts().unwrap();
    let after_recovery = session.analyze().unwrap();
    assert_eq!(
        serde_json::to_vec(&after_recovery).unwrap(),
        serde_json::to_vec(&before_recovery).unwrap(),
        "restarted TypeFacts service must replay the retained generation exactly"
    );
}

#[test]
fn pipelined_open_matches_sequential_sources_and_analyzes() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture.join("tsconfig.json").canonicalize().unwrap();
    let project_id = project.to_string_lossy().into_owned();

    // The sequential baseline: fetch sources, then open, in two round trips.
    let mut sequential =
        TypeFactsSidecar::spawn(&typefacts, &["-project".into(), project_id.clone()]).unwrap();
    let expected = sequential.configured_sources(&project_id, 1).unwrap();

    // The reordered handshake lets the client pipeline open+sources before the
    // program build completes; the pipelined path must resolve the same source
    // set and produce facts.
    let pipelined =
        TypeFactsSidecar::spawn(&typefacts, &["-project".into(), project_id.clone()]).unwrap();
    let (mut session, sources) =
        NativeIncrementalSession::open_pipelined(project_id, pipelined).unwrap();
    assert_eq!(
        sources
            .iter()
            .map(|source| (source.path.clone(), source.source.clone()))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|source| (source.path.clone(), source.source.clone()))
            .collect::<Vec<_>>(),
        "pipelined open+sources must resolve the same configured source set"
    );
    session.analyze().unwrap();
    assert_eq!(session.generation(), 1);
}

#[test]
fn rust_cli_reports_expected_write_scope_rule_counts() {
    let Some(findings) = diagnostic_fixture("write-scope") else {
        return;
    };
    let writes = findings_for_rule(&findings, "reactive-write-in-owned-scope").len();
    let actions = findings_for_rule(&findings, "action-called-in-owned-scope").len();
    assert_eq!((writes, actions), (12, 2));
    assert!(findings.iter().all(|finding| {
        finding["primaryLocation"]["path"]
            .as_str()
            .is_some_and(|path| path.ends_with(".tsx"))
            && finding["message"].as_str().is_some_and(|message| {
                message.contains("owned scope") || message.contains("action")
            })
    }));
}

#[test]
fn rust_cli_reports_expected_leaf_owner_rule_counts() {
    let Some(findings) = diagnostic_fixture("leaf-owner") else {
        return;
    };
    for (rule, expected) in [
        ("cleanup-in-forbidden-scope", 3),
        ("primitive-in-leaf-owner", 3),
        ("flush-in-forbidden-scope", 2),
        ("invalid-cleanup-return", 6),
    ] {
        assert_rule_findings(&findings, rule, expected);
    }
}

#[test]
fn rust_cli_reports_expected_static_api_rule_counts() {
    let Some(findings) = diagnostic_fixture("static-api") else {
        return;
    };
    for (rule, expected) in [
        ("missing-effect-function", 2),
        ("sync-node-received-async", 6),
        ("invalid-refresh-target", 2),
        ("invalid-affects-target", 2),
        ("reactive-write-in-owned-scope", 1),
    ] {
        assert_rule_findings(&findings, rule, expected);
    }
}

#[test]
fn rust_cli_reports_expected_directive_rule_counts() {
    let Some(findings) = diagnostic_fixture("directive-phases") else {
        return;
    };
    for (rule, expected) in [
        ("reactive-write-in-owned-scope", 1),
        ("primitive-in-directive-application", 3),
    ] {
        assert_rule_findings(&findings, rule, expected);
    }
}

#[test]
fn rust_cli_reports_expected_owner_presence_rule_counts() {
    let Some(findings) = diagnostic_fixture("owner-presence") else {
        return;
    };
    for (rule, expected) in [
        ("no-owner-effect", 7),
        ("no-owner-cleanup", 2),
        ("no-owner-boundary", 3),
        ("settled-cleanup-unowned", 2),
    ] {
        assert_rule_findings(&findings, rule, expected);
    }
}

#[test]
fn rust_cli_reports_expected_async_boundary_rule_counts() {
    let Some(findings) = diagnostic_fixture("async-boundary") else {
        return;
    };
    for (rule, expected) in [
        ("pending-async-untracked-read", 1),
        ("pending-async-forbidden-scope", 1),
        ("async-outside-loading-boundary", 7),
    ] {
        assert_rule_findings(&findings, rule, expected);
    }
}

#[test]
fn rust_cli_reports_expected_interprocedural_primary_locations() {
    for (fixture, expected_count, message) in [
        ("interprocedural", 1, "readCount"),
        ("callback-forwarding", 1, "invoke"),
        ("polymorphic", 2, "readGeneric"),
        ("recursive", 1, "readA"),
        ("returned-closure", 1, "readCount"),
        ("store-flow", 1, "\"state.count\""),
    ] {
        let Some(findings) = diagnostic_fixture(fixture) else {
            return;
        };
        let strict = findings_for_rule(&findings, "strict-read-untracked");
        assert_eq!(
            strict.len(),
            expected_count,
            "fixture {fixture}: {findings:#?}"
        );
        assert!(
            strict
                .iter()
                .any(|finding| finding["message"].as_str().unwrap().contains(message)),
            "fixture {fixture}: {strict:#?}"
        );
        assert!(
            strict
                .iter()
                .all(|finding| finding["primaryLocation"]["path"]
                    .as_str()
                    .unwrap()
                    .ends_with("App.tsx")),
            "fixture {fixture}: {strict:#?}"
        );
    }
}

#[test]
fn rust_cli_reports_expected_control_flow_and_effect_phase_counts() {
    for (fixture, expected) in [("control-flow", 0), ("execution-phases", 1)] {
        let Some(findings) = diagnostic_fixture(fixture) else {
            return;
        };
        assert_eq!(
            findings_for_rule(&findings, "strict-read-untracked").len(),
            expected,
            "fixture {fixture}: {findings:#?}"
        );
    }
}

#[test]
fn rust_cli_consumes_discovered_package_contracts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (fixture, rule, message) in [
        ("package-consumer", "strict-read-untracked", "readCount"),
        (
            "package-return-consumer",
            "strict-read-untracked",
            "created count",
        ),
        (
            "package-callback-consumer",
            "strict-read-untracked",
            "runInline",
        ),
        (
            "package-store-consumer",
            "strict-read-untracked",
            "state.value",
        ),
        (
            "package-unknown-export",
            "package-contract-export-missing",
            "unknownPrimitive",
        ),
        ("bundled-solid-consumer", "strict-read-untracked", "doubled"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
            .env("SOLID_TYPEFACTS_BIN", &typefacts)
            .env("SOLID_COMPILER_FACTS_BIN", &compiler)
            .args([
                "--format",
                "json",
                "--project",
                &root
                    .join(format!(
                        "internal/reactiveir/testdata/{fixture}/tsconfig.json"
                    ))
                    .to_string_lossy(),
            ])
            .output()
            .expect("run Rust diagnostic CLI");
        assert!(
            output.status.success(),
            "fixture {fixture}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let findings = decode_findings(&output.stdout);
        assert_eq!(findings.len(), 1, "fixture {fixture}: {findings:#?}");
        assert_eq!(findings[0]["rule"], rule, "fixture {fixture}");
        assert!(
            findings[0]["message"]
                .as_str()
                .is_some_and(|finding| finding.contains(message)),
            "fixture {fixture}: {findings:#?}"
        );
    }
}

#[test]
fn rust_cli_covers_reactivity_v2_semantic_migration_matrix() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", typefacts)
        .env("SOLID_COMPILER_FACTS_BIN", compiler)
        .args([
            "--format",
            "json",
            "--project",
            &root
                .join("internal/engine/testdata/eslint-reactivity-v2/tsconfig.json")
                .to_string_lossy(),
        ])
        .output()
        .expect("run Rust diagnostic CLI");
    assert!(
        output.status.success(),
        "stderr = {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let findings = decode_findings(&output.stdout);
    let expected = [
        ("leaf-reexport-flush.tsx", "flush-in-forbidden-scope"),
        ("leaf-reexport-cleanup.tsx", "cleanup-in-forbidden-scope"),
        ("owned-reexport-memo.tsx", "reactive-write-in-owned-scope"),
        (
            "owned-reexport-refresh.tsx",
            "reactive-write-in-owned-scope",
        ),
        ("owned-reexport-action.tsx", "action-called-in-owned-scope"),
        ("effect-apply-parameter.tsx", "strict-read-untracked"),
        ("effect-apply-member.tsx", "strict-read-untracked"),
        ("after-await-member.tsx", "reactive-read-after-await"),
        ("after-await-parameter.tsx", "reactive-read-after-await"),
        ("after-await-namespace.tsx", "reactive-read-after-await"),
        (
            "after-await-callback-before-read.tsx",
            "reactive-read-after-await",
        ),
        (
            "after-await-named-callback.tsx",
            "reactive-read-after-await",
        ),
        (
            "after-await-aliased-callback.tsx",
            "reactive-read-after-await",
        ),
        (
            "after-await-same-expression.tsx",
            "reactive-read-after-await",
        ),
        ("after-await-both-branches.tsx", "reactive-read-after-await"),
        ("after-await-try-finally.tsx", "reactive-read-after-await"),
        (
            "imported-after-await-definition.ts",
            "reactive-read-after-await",
        ),
        ("component-props-read.tsx", "strict-read-untracked"),
        ("component-props-alias.tsx", "strict-read-untracked"),
        ("component-props-merge-alias.tsx", "strict-read-untracked"),
        (
            "component-reactive-early-return.tsx",
            "strict-read-untracked",
        ),
        (
            "component-reactive-conditional-return.tsx",
            "strict-read-untracked",
        ),
        (
            "component-props-parameter-destructure.tsx",
            "component-props-destructure",
        ),
        (
            "component-props-body-destructure.tsx",
            "component-props-destructure",
        ),
        (
            "derived-signal-in-effect.tsx",
            "reactive-write-in-owned-scope",
        ),
    ];
    for (file, rule) in expected {
        assert!(
            findings.iter().any(|finding| {
                finding["rule"] == rule
                    && finding["primaryLocation"]["path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with(file))
            }),
            "missing {file} / {rule}"
        );
    }
    let negatives = [
        ("effect-apply-plain-function.tsx", "strict-read-untracked"),
        ("effect-apply-structural-store.tsx", "strict-read-untracked"),
        (
            "after-await-plain-function.tsx",
            "reactive-read-after-await",
        ),
        (
            "after-await-local-accessor.tsx",
            "reactive-read-after-await",
        ),
        ("before-await-accessor.tsx", "reactive-read-after-await"),
        (
            "conditional-await-accessor.tsx",
            "reactive-read-after-await",
        ),
        (
            "nested-after-await-accessor.tsx",
            "reactive-read-after-await",
        ),
        ("loop-await-accessor.tsx", "reactive-read-after-await"),
        ("component-props-tracked.tsx", "strict-read-untracked"),
        ("noncomponent-object-read.ts", "strict-read-untracked"),
        (
            "noncomponent-object-destructure.ts",
            "component-props-destructure",
        ),
        ("component-props-passthrough.tsx", "strict-read-untracked"),
        ("component-props-local-merge.tsx", "strict-read-untracked"),
        (
            "component-props-unknown-callback.tsx",
            "strict-read-untracked",
        ),
        ("component-static-early-return.tsx", "strict-read-untracked"),
        (
            "signal-write-in-effect-apply.tsx",
            "reactive-write-in-owned-scope",
        ),
    ];
    for (file, rule) in negatives {
        assert!(
            !findings.iter().any(|finding| {
                finding["rule"] == rule
                    && finding["primaryLocation"]["path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with(file))
            }),
            "unexpected {file} / {rule}"
        );
    }
    assert_eq!(
        findings
            .iter()
            .filter(|finding| {
                finding["rule"] == "component-props-destructure"
                    && finding["primaryLocation"]["path"]
                        .as_str()
                        .is_some_and(|path| {
                            path.ends_with("component-props-parameter-complex-destructure.tsx")
                        })
            })
            .count(),
        3
    );
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
        state_token: String::new(),
        reset_state: false,
        removed_demand_paths: vec![],
        cancel_request_id: 0,
    }
}

#[test]
fn v3_updates_and_reanalyzes_a_retained_tsgo_project() {
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
        source: std::fs::read(&app).unwrap(),
        deleted: false,
    });
    service.lifecycle(update).unwrap();
    let analyzed = service
        .lifecycle(lifecycle_request(Operation::Analyze, project_id.clone(), 2))
        .unwrap();
    assert!(analyzed.table.is_some());
    assert!(
        service
            .lifecycle(lifecycle_request(Operation::Analyze, project_id, 1))
            .is_err()
    );
}

#[test]
fn rust_cli_emits_snapshot_text_and_certification_exit_codes() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tracer = root.join("internal/reactiveir/testdata/tracer/tsconfig.json");
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", &typefacts)
        .env("SOLID_COMPILER_FACTS_BIN", &compiler)
        .args(["--project", &tracer.to_string_lossy()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains(": violation\n"));
    assert!(text.contains("SC1001 [violation]"));

    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", &typefacts)
        .env("SOLID_COMPILER_FACTS_BIN", &compiler)
        .args([
            "--format",
            "json",
            "--certify",
            "--project",
            &tracer.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let snapshot: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot["status"], "violation");
    assert_eq!(snapshot["findings"][0]["primaryLocation"]["line"], 8);
    assert!(
        snapshot["findings"][0]["primaryLocation"]["column"]
            .as_u64()
            .is_some_and(|column| column > 0)
    );
    assert_eq!(snapshot["metrics"]["filesAnalyzed"], 1);

    let corrected = root.join("internal/reactiveir/testdata/tracer-corrected/tsconfig.json");
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", typefacts)
        .env("SOLID_COMPILER_FACTS_BIN", compiler)
        .args([
            "--format",
            "json",
            "--certify",
            "--project",
            &corrected.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot["status"], "certified");
    assert_eq!(snapshot["findings"].as_array().unwrap().len(), 0);
}

#[test]
fn rust_cli_validates_package_contract_without_opening_a_project() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let contract = root.join(
        "internal/reactiveir/testdata/package-consumer/node_modules/reactive-package/solid-reactivity.json",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env_remove("SOLID_TYPEFACTS_BIN")
        .env_remove("SOLID_COMPILER_FACTS_BIN")
        .args(["--validate-contract", &contract.to_string_lossy()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rust_cli_emits_and_consumes_package_contracts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = temporary_directory("emit-contract");
    let output = directory.join("solid-reactivity.json");
    let declaration = directory.join("index.d.ts");
    fs::write(
        &declaration,
        "export declare function createCount(): () => number;\n",
    )
    .unwrap();
    let producer = root.join("internal/reactiveir/testdata/package-return-producer/tsconfig.json");
    let result = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", &typefacts)
        .args([
            "--project",
            &producer.to_string_lossy(),
            "--emit-contract",
            &output.to_string_lossy(),
            "--package-name",
            "reactive-package",
            "--package-version",
            "1.0.0",
            "--declaration-artifact",
            &declaration.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let contract: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    for name in [
        "createCount",
        "createAliasedCount",
        "createArrowCount",
        "createMemoCount",
    ] {
        assert_eq!(contract["exports"][name]["returns"]["kind"], "accessor");
    }
    assert_eq!(
        contract["exports"]["createState"]["returns"]["kind"],
        "store-path"
    );
    assert_eq!(contract["exports"]["packageVersion"]["kind"], "value");
    assert_eq!(contract["artifacts"]["declaration"]["path"], "index.d.ts");

    let validate = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env_remove("SOLID_TYPEFACTS_BIN")
        .args(["--validate-contract", &output.to_string_lossy()])
        .output()
        .unwrap();
    assert!(
        validate.status.success(),
        "{}",
        String::from_utf8_lossy(&validate.stderr)
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn in_process_compiler_matches_the_sidecar_snapshot() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for fixture in ["tracer", "control-flow", "async-boundary"] {
        let project = root.join(format!(
            "internal/reactiveir/testdata/{fixture}/tsconfig.json"
        ));
        let native = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
            .env("SOLID_TYPEFACTS_BIN", &typefacts)
            .env_remove("SOLID_COMPILER_FACTS_BIN")
            .args(["--format", "json", "--project", &project.to_string_lossy()])
            .output()
            .unwrap();
        let sidecar = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
            .env("SOLID_TYPEFACTS_BIN", &typefacts)
            .env("SOLID_COMPILER_FACTS_BIN", &compiler)
            .args(["--format", "json", "--project", &project.to_string_lossy()])
            .output()
            .unwrap();
        assert!(
            native.status.success() && sidecar.status.success(),
            "fixture {fixture}: native={}, sidecar={}",
            String::from_utf8_lossy(&native.stderr),
            String::from_utf8_lossy(&sidecar.stderr)
        );
        let native: serde_json::Value = serde_json::from_slice(&native.stdout).unwrap();
        let sidecar: serde_json::Value = serde_json::from_slice(&sidecar.stdout).unwrap();
        assert_eq!(native, sidecar, "fixture {fixture}");
    }
}

#[test]
fn joins_real_oxc_compiler_and_tsgo_facts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture
        .join("tsconfig.json")
        .canonicalize()
        .expect("canonical tsconfig");
    let source_paths = [fixture.join("App.tsx"), fixture.join("source.ts")];
    let sources = source_paths
        .into_iter()
        .map(|path| SourceFile {
            path: path
                .canonicalize()
                .expect("canonical source")
                .to_string_lossy()
                .into_owned(),
            source: std::fs::read_to_string(path).expect("read source"),
            compiler_options: CompilerOptions::default(),
        })
        .collect();
    let mut compiler = CompilerSidecar::spawn(&compiler, &[]).expect("spawn Oxc compiler");
    let mut typescript = TypeFactsSidecar::spawn(
        &typefacts,
        &["-project".into(), project.to_string_lossy().into_owned()],
    )
    .expect("spawn TS-Go service");
    let facts = build_project(
        project.to_string_lossy(),
        1,
        sources,
        &mut compiler,
        &mut typescript,
    )
    .expect("join real facts");
    assert_eq!(facts.files.len(), 2);
    assert!(!facts.typescript.entities.is_empty());
    let program = solid_reactive_ir::build(&facts).expect("build Rust Reactive IR");
    let (incremental_program, incremental_timings) =
        solid_reactive_ir::IncrementalBuilder::default()
            .build(&facts)
            .expect("build incremental Rust Reactive IR");
    assert_eq!(
        incremental_program, program,
        "initial incremental fragments must match the fresh builder"
    );
    assert_eq!(incremental_timings.source_discovery_recomputed_files, 2);
    assert_eq!(incremental_timings.typed_accessor_recomputed_files, 2);
    let findings = solid_reactive_solver::solve(&program);
    assert_eq!(findings.len(), 1, "findings = {findings:#?}");
    assert_eq!(findings[0].rule, "strict-read-untracked");
    assert!(findings[0].primary_location.path.ends_with("App.tsx"));
}

fn tracer_fixture_session(typefacts_executable: &str) -> (NativeIncrementalSession, Vec<PathBuf>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("internal/reactiveir/testdata/tracer");
    let project = fixture.join("tsconfig.json").canonicalize().unwrap();
    let project_id = project.to_string_lossy().into_owned();
    let paths = vec![
        fixture.join("App.tsx").canonicalize().unwrap(),
        fixture.join("source.ts").canonicalize().unwrap(),
    ];
    let sources = paths
        .iter()
        .map(|path| SourceFile {
            path: path.to_string_lossy().into_owned(),
            source: fs::read_to_string(path).unwrap(),
            compiler_options: CompilerOptions::default(),
        })
        .collect();
    let typescript = TypeFactsSidecar::spawn(
        typefacts_executable,
        &["-project".into(), project_id.clone()],
    )
    .unwrap();
    let session = NativeIncrementalSession::open(project_id, sources, typescript).unwrap();
    (session, paths)
}

fn app_edit(paths: &[PathBuf]) -> SourceChange {
    SourceChange {
        path: paths[0].to_string_lossy().into_owned(),
        version: 1,
        source: Some(format!(
            "// edit exchange test\n{}",
            fs::read_to_string(&paths[0]).unwrap()
        )),
        compiler_options: CompilerOptions::default(),
    }
}

#[test]
fn incremental_reactive_ir_matches_fresh_after_an_edit() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (mut session, paths) = tracer_fixture_session(&typefacts);
    let first = session.analyze().unwrap();
    let mut incremental = solid_reactive_ir::IncrementalBuilder::default();
    incremental.build(&first).unwrap();

    let edited = session.edit(vec![app_edit(&paths)], None).unwrap();
    let fresh = solid_reactive_ir::build(&edited).unwrap();
    let (retained, timings) = incremental.build(&edited).unwrap();

    assert_eq!(retained, fresh);
    assert!(!timings.reused);
    assert!(timings.typescript_indexes_reused);
    assert!(!timings.reachability_reused);
    assert!(!timings.local_accesses_reused);
    assert!(!timings.interprocedural_reused);
    assert!(!timings.owner_fixed_point_reused);
    assert_eq!(
        timings.reachability_reused_files + timings.reachability_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.local_access_reused_files + timings.local_access_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.owner_reused_files + timings.owner_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.source_discovery_reused_files + timings.source_discovery_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.typed_accessor_reused_files + timings.typed_accessor_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.interprocedural_graph_reused_files,
        timings.source_discovery_reused_files
    );
    assert_eq!(
        timings.interprocedural_graph_reused_files + timings.interprocedural_graph_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(
        timings.interprocedural_result_reused_files
            + timings.interprocedural_result_recomputed_files,
        u64::try_from(edited.files.len()).unwrap()
    );
}

#[test]
fn incremental_contract_exports_drop_removed_names() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (mut session, paths) = tracer_fixture_session(&typefacts);
    let first = session.analyze().unwrap();
    let mut incremental = solid_reactive_ir::IncrementalBuilder::default();
    let (initial, _) = incremental.build(&first).unwrap();
    assert!(initial.contract_exports.contains_key("Bad"));

    let original = fs::read_to_string(&paths[0]).unwrap();
    let changed = original.replacen("export function Bad", "function Bad", 1);
    assert_ne!(changed, original);
    let edited = session
        .edit(
            vec![SourceChange {
                path: paths[0].to_string_lossy().into_owned(),
                version: 1,
                source: Some(changed),
                compiler_options: CompilerOptions::default(),
            }],
            None,
        )
        .unwrap();
    let fresh = solid_reactive_ir::build(&edited).unwrap();
    let (retained, _) = incremental.build(&edited).unwrap();

    assert_eq!(retained, fresh);
    assert!(!retained.contract_exports.contains_key("Bad"));
}

#[test]
fn incremental_contract_exports_refresh_changed_summaries() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (mut session, paths) = tracer_fixture_session(&typefacts);
    let first = session.analyze().unwrap();
    let mut incremental = solid_reactive_ir::IncrementalBuilder::default();
    let (initial, _) = incremental.build(&first).unwrap();
    let initial_good = initial.contract_exports.get("Good").cloned().unwrap();

    let original = fs::read_to_string(&paths[0]).unwrap();
    let changed = original.replacen(
        "return <div>{count()}</div>;",
        "return <div>static</div>;",
        1,
    );
    assert_ne!(changed, original);
    let edited = session
        .edit(
            vec![SourceChange {
                path: paths[0].to_string_lossy().into_owned(),
                version: 1,
                source: Some(changed),
                compiler_options: CompilerOptions::default(),
            }],
            None,
        )
        .unwrap();
    let fresh = solid_reactive_ir::build(&edited).unwrap();
    let (retained, _) = incremental.build(&edited).unwrap();

    assert_eq!(retained, fresh);
    assert_ne!(
        retained.contract_exports.get("Good"),
        Some(&initial_good),
        "a changed inferred function contract must invalidate its fragment"
    );
}

#[test]
fn incremental_reactive_ir_reuses_semantic_indexes_for_same_shape_body_edit() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (mut session, paths) = tracer_fixture_session(&typefacts);
    let first = session.analyze().unwrap();
    let mut incremental = solid_reactive_ir::IncrementalBuilder::default();
    incremental.build(&first).unwrap();

    let original = fs::read_to_string(&paths[1]).unwrap();
    let changed = original.replacen("createSignal(0)", "createSignal(1)", 1);
    assert_ne!(changed, original);
    let edited = session
        .edit(
            vec![SourceChange {
                path: paths[1].to_string_lossy().into_owned(),
                version: 1,
                source: Some(changed),
                compiler_options: CompilerOptions::default(),
            }],
            None,
        )
        .unwrap();
    let fresh = solid_reactive_ir::build(&edited).unwrap();
    let (retained, timings) = incremental.build(&edited).unwrap();

    assert_eq!(retained, fresh);
    assert!(timings.typescript_indexes_reused);
    assert!(timings.reachability_reused);
    assert!(timings.local_accesses_reused);
    assert_eq!(
        timings.local_access_reused_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(timings.local_access_recomputed_files, 0);
    assert!(timings.interprocedural_reused);
    assert!(timings.owner_fixed_point_reused);
    assert_eq!(
        timings.owner_reused_files,
        u64::try_from(edited.files.len()).unwrap()
    );
    assert_eq!(timings.owner_recomputed_files, 0);
}

#[test]
fn incremental_owner_fragments_match_fresh_owner_fixtures() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for fixture_name in ["owner-presence", "leaf-owner"] {
        let fixture = root.join(format!("internal/reactiveir/testdata/{fixture_name}"));
        let project = fixture.join("tsconfig.json").canonicalize().unwrap();
        let project_id = project.to_string_lossy().into_owned();
        let app = fixture.join("App.tsx").canonicalize().unwrap();
        let sources = vec![SourceFile {
            path: app.to_string_lossy().into_owned(),
            source: fs::read_to_string(&app).unwrap(),
            compiler_options: CompilerOptions::default(),
        }];
        let typescript =
            TypeFactsSidecar::spawn(&typefacts, &["-project".into(), project_id.clone()]).unwrap();
        let mut session = NativeIncrementalSession::open(project_id, sources, typescript).unwrap();
        let first = session.analyze().unwrap();
        let mut incremental = solid_reactive_ir::IncrementalBuilder::default();
        incremental.build(&first).unwrap();
        let edited = session
            .edit(
                vec![SourceChange {
                    path: app.to_string_lossy().into_owned(),
                    version: 1,
                    source: Some(format!(
                        "// owner fragment edit\n{}",
                        fs::read_to_string(&app).unwrap()
                    )),
                    compiler_options: CompilerOptions::default(),
                }],
                None,
            )
            .unwrap();
        let fresh = solid_reactive_ir::build(&edited).unwrap();
        let (retained, timings) = incremental.build(&edited).unwrap();
        assert_eq!(retained, fresh, "fixture {fixture_name}");
        assert_eq!(
            timings.owner_reused_files + timings.owner_recomputed_files,
            u64::try_from(edited.files.len()).unwrap(),
            "fixture {fixture_name}"
        );
    }
}

#[test]
fn cancelled_edit_before_any_send_leaves_the_session_consistent() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (mut session, paths) = tracer_fixture_session(&typefacts);
    session.analyze().unwrap();
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    let error = session
        .edit(vec![app_edit(&paths)], Some(&cancelled))
        .unwrap_err();
    assert!(
        matches!(error, solid_facts_backend::BackendError::Cancelled),
        "expected cancellation, got {error}"
    );
    assert_eq!(
        session.generation(),
        1,
        "nothing was sent, so the generation must not advance"
    );
    cancelled.store(false, std::sync::atomic::Ordering::Release);
    let edited = session
        .edit(vec![app_edit(&paths)], Some(&cancelled))
        .unwrap();
    assert_eq!(session.generation(), 2);
    let reanalyzed = session.analyze().unwrap();
    assert_eq!(
        serde_json::to_vec(&edited).unwrap(),
        serde_json::to_vec(&reanalyzed).unwrap(),
        "an edit exchange must answer exactly what a follow-up analysis answers"
    );
}

/// Wraps the real service in a script that arms one crash marker, so the
/// session under test observes a deterministic service death and the
/// restarted service (same wrapper, marker consumed) runs normally.
#[cfg(unix)]
fn crash_armed_service(
    typefacts_executable: &str,
    variable: &str,
    label: &str,
) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = temporary_directory(label);
    let marker = directory.join("crash-marker");
    fs::write(&marker, b"armed").unwrap();
    let wrapper = directory.join("solid-typefacts-crashing.sh");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\n{variable}={marker} exec {typefacts_executable} \"$@\"\n",
            marker = marker.display(),
        ),
    )
    .unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    (wrapper, marker)
}

#[cfg(unix)]
#[test]
fn edit_recovers_when_the_service_dies_before_the_update_lands() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (wrapper, marker) = crash_armed_service(
        &typefacts,
        "SOLID_TYPEFACTS_CRASH_BEFORE_UPDATE",
        "crash-before-update",
    );
    // Arm the marker only after the session is warm, so the crash hits the
    // edit's update half: the update never lands, recovery replays the
    // pre-edit state, and the retry re-sends both halves.
    fs::remove_file(&marker).unwrap();
    let (mut session, paths) = tracer_fixture_session(&wrapper.to_string_lossy());
    session.analyze().unwrap();
    fs::write(&marker, b"armed").unwrap();
    let edited = session.edit(vec![app_edit(&paths)], None).unwrap();
    assert_eq!(session.generation(), 2);
    assert!(!marker.exists(), "the crash marker must have been consumed");
    let reanalyzed = session.analyze().unwrap();
    assert_eq!(
        serde_json::to_vec(&edited).unwrap(),
        serde_json::to_vec(&reanalyzed).unwrap(),
        "recovery must converge on the same facts"
    );
}

#[cfg(unix)]
#[test]
fn edit_recovers_when_the_service_dies_between_update_and_analyze() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let (wrapper, marker) = crash_armed_service(
        &typefacts,
        "SOLID_TYPEFACTS_CRASH_BEFORE_ANALYZE",
        "crash-before-analyze",
    );
    // Arm the marker only after the session is warm, so the crash hits the
    // edit's analyze half: the update has landed and the generation is
    // committed, recovery replays the committed generation, and the retry
    // re-sends only the analyze.
    fs::remove_file(&marker).unwrap();
    let (mut session, paths) = tracer_fixture_session(&wrapper.to_string_lossy());
    session.analyze().unwrap();
    fs::write(&marker, b"armed").unwrap();
    let edited = session.edit(vec![app_edit(&paths)], None).unwrap();
    assert_eq!(session.generation(), 2);
    assert!(!marker.exists(), "the crash marker must have been consumed");
    let reanalyzed = session.analyze().unwrap();
    assert_eq!(
        serde_json::to_vec(&edited).unwrap(),
        serde_json::to_vec(&reanalyzed).unwrap(),
        "recovery must converge on the same facts"
    );
}
