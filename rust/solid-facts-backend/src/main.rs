use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solid_facts_backend::{
    BackendError, CompilerSidecar, Snapshot, SourceFile, TypeFactsSidecar,
    analyze_project_measured_with, build_project, build_project_native_measured,
    bundled_solid_js_contract, default_typefacts_executable, read_package_contract,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    project_id: String,
    generation: u64,
    sources: Vec<SourceFile>,
    #[serde(default)]
    compiler_executable: String,
    #[serde(default)]
    compiler_args: Vec<String>,
    typefacts_executable: String,
    #[serde(default)]
    typefacts_args: Vec<String>,
    #[serde(default)]
    contract_paths: Vec<String>,
    #[serde(default = "json_format")]
    format: String,
    #[serde(default)]
    certify: bool,
    #[serde(default)]
    validate_contract_paths: Vec<String>,
    #[serde(default)]
    emit_contract: String,
    #[serde(default)]
    package_name: String,
    #[serde(default)]
    package_version: String,
    #[serde(default)]
    declaration_artifact: String,
    #[serde(default)]
    implementation_artifact: String,
    #[serde(default)]
    oxlint: bool,
    #[serde(default)]
    oxlint_args: Vec<String>,
    #[serde(default)]
    help: bool,
    #[serde(default)]
    serve: bool,
}

fn json_format() -> String {
    "json".into()
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let started = Instant::now();
    // A JSON request arrives on stdin only when the caller passed no
    // arguments; argv invocations must not block waiting for stdin EOF.
    let mut encoded = String::new();
    if std::env::args().len() <= 1 && !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut encoded)?;
    }
    let mut request: Request = if encoded.trim().is_empty() {
        request_from_args()?
    } else {
        serde_json::from_str(&encoded)?
    };
    if request.help {
        print_help();
        return Ok(0);
    }
    if !request.validate_contract_paths.is_empty() {
        for path in &request.validate_contract_paths {
            read_package_contract(Path::new(path))?;
        }
        return Ok(0);
    }
    #[cfg(unix)]
    {
        if request.serve {
            return daemon::serve(&request);
        }
        if daemon::enabled() && daemon::eligible(&request) {
            match daemon::check(&request) {
                Ok(code) => return Ok(code),
                Err(error) => {
                    eprintln!("solid-check: daemon unavailable ({error}); running one-shot");
                }
            }
        }
    }
    #[cfg(not(unix))]
    if request.serve {
        return Err("--serve requires a Unix platform".into());
    }
    let diagnostics = env!("CARGO_BIN_NAME") == "solid-check-rust";
    let mut typescript =
        TypeFactsSidecar::spawn(&request.typefacts_executable, &request.typefacts_args)?;
    // The service now flushes its handshake before opening the TypeScript
    // program, so spawn returns as soon as the process is live rather than
    // after the program build. `sidecarSpawnNs` therefore measures process
    // startup plus handshake, no longer the program build (which has moved
    // into `sourcesFetchNs`, the first request that needs the built program).
    let sidecar_spawn_ns = started.elapsed().as_nanos();
    let mut sources_bytes = 0usize;
    let mut preloaded_bundled = None;
    if request.sources.is_empty() {
        // Pipeline open+sources ahead of the program build: both frames queue
        // in the service's ordered worker and are answered once the program is
        // built. While those responses are in flight, decode the bundled
        // solid-js contract — the only cold work that needs nothing from the
        // service — so it overlaps the build instead of running after facts.
        let pending_open = typescript.lifecycle_send(lifecycle_request(
            Operation::Open,
            &request.project_id,
            request.generation,
        ))?;
        let pending_sources = typescript.lifecycle_send(lifecycle_request(
            Operation::Sources,
            &request.project_id,
            request.generation,
        ))?;
        if diagnostics {
            preloaded_bundled = Some(bundled_solid_js_contract()?);
        }
        typescript.lifecycle_wait(pending_open)?;
        let response = typescript.lifecycle_wait(pending_sources)?;
        request.sources = decode_sources(response)?;
        sources_bytes = request.sources.iter().map(|s| s.source.len()).sum();
    }
    let source_setup_ns = started.elapsed().as_nanos();
    let (facts, native_timings) = if request.compiler_executable.is_empty() {
        let (facts, timings) = build_project_native_measured(
            request.project_id.clone(),
            request.generation,
            request.sources.clone(),
            &mut typescript,
        )?;
        (facts, Some(timings))
    } else {
        let mut compiler =
            CompilerSidecar::spawn(&request.compiler_executable, &request.compiler_args)?;
        (
            build_project(
                request.project_id.clone(),
                request.generation,
                request.sources.clone(),
                &mut compiler,
                &mut typescript,
            )?,
            None,
        )
    };
    let facts_complete_ns = started.elapsed().as_nanos();
    if diagnostics {
        let (analysis, diagnostic_timings) = analyze_project_measured_with(
            Path::new(&facts.project_id),
            &request.sources,
            &facts,
            &request.contract_paths,
            preloaded_bundled,
        )?;
        if !request.emit_contract.is_empty() {
            emit_package_contract(&request, &analysis.program)?;
            return Ok(0);
        }
        let snapshot = analysis.snapshot;
        if request.oxlint {
            return run_oxlint(&request.oxlint_args, &snapshot);
        }
        match request.format.as_str() {
            "json" => {
                let mut stdout = io::stdout().lock();
                stdout.write_all(&go_compatible_json(&snapshot, true)?)?;
                stdout.write_all(b"\n")?;
            }
            "text" => {
                println!("{}: {}", request.project_id, snapshot.status);
                for finding in &snapshot.findings {
                    println!("{} [{}] {}", finding.id, finding.kind, finding.message);
                }
            }
            format => return Err(format!("unsupported format {format:?}").into()),
        }
        if std::env::var_os("SOLID_CHECK_TIMINGS").is_some() {
            let (source_analysis_ns, type_facts_ns) = native_timings.map_or((0, 0), |timings| {
                (
                    timings.source_analysis.as_nanos(),
                    timings.type_facts.as_nanos(),
                )
            });
            eprintln!(
                "{}",
                serde_json::json!({
                    "sidecarSpawnNs": sidecar_spawn_ns,
                    "sourcesFetchNs": source_setup_ns.saturating_sub(sidecar_spawn_ns),
                    "sourcesBytes": sources_bytes,
                    "sourceSetupNs": source_setup_ns,
                    "sourceAnalysisNs": source_analysis_ns,
                    "typeFactsNs": type_facts_ns,
                    "factsTotalNs": facts_complete_ns.saturating_sub(source_setup_ns),
                    "irNs": diagnostic_timings.reactive_ir.as_nanos(),
                    "solveAndSnapshotNs": diagnostic_timings.solve_and_snapshot.as_nanos(),
                    "totalNs": started.elapsed().as_nanos(),
                })
            );
        }
        if request.certify && snapshot.status != "certified" {
            return Ok(1);
        }
    } else {
        serde_json::to_writer(io::stdout(), &facts)?;
    }
    Ok(0)
}

fn request_from_args() -> Result<Request, Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .first()
        .is_some_and(|argument| argument == "oxlint")
    {
        return request_from_oxlint_args(&arguments[1..]);
    }
    let mut project = PathBuf::from("tsconfig.json");
    let mut compiler = std::env::var("SOLID_COMPILER_FACTS_BIN").unwrap_or_default();
    let mut typefacts = default_typefacts_executable();
    let mut contract_paths = Vec::new();
    let mut format = "text".to_owned();
    let mut certify = false;
    let mut validate_contract_paths = Vec::new();
    let mut emit_contract = String::new();
    let mut package_name = String::new();
    let mut package_version = String::new();
    let mut declaration_artifact = String::new();
    let mut implementation_artifact = String::new();
    let mut help = false;
    let mut serve = false;
    let mut args = arguments.into_iter();
    while let Some(argument) = args.next() {
        if let Some(value) = argument.strip_prefix("--project=") {
            project = PathBuf::from(value);
            continue;
        }
        if let Some(value) = argument.strip_prefix("--compiler=") {
            compiler = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--typefacts=") {
            typefacts = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--contract=") {
            contract_paths.push(value.into());
            continue;
        }
        if let Some(value) = argument.strip_prefix("--format=") {
            format = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--validate-contract=") {
            validate_contract_paths.push(value.into());
            continue;
        }
        if let Some(value) = argument.strip_prefix("--emit-contract=") {
            emit_contract = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--package-name=") {
            package_name = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--package-version=") {
            package_version = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--declaration-artifact=") {
            declaration_artifact = value.into();
            continue;
        }
        if let Some(value) = argument.strip_prefix("--implementation-artifact=") {
            implementation_artifact = value.into();
            continue;
        }
        match argument.as_str() {
            "--project" | "-project" => {
                project = PathBuf::from(args.next().ok_or("--project needs a path")?)
            }
            "--compiler" => compiler = args.next().ok_or("--compiler needs a path")?,
            "--typefacts" => typefacts = args.next().ok_or("--typefacts needs a path")?,
            "--contract" => contract_paths.push(args.next().ok_or("--contract needs a path")?),
            "--format" => format = args.next().ok_or("--format needs a value")?,
            "--certify" => certify = true,
            "--serve" => serve = true,
            "--help" | "-h" => help = true,
            "--validate-contract" => {
                validate_contract_paths.push(args.next().ok_or("--validate-contract needs a path")?)
            }
            "--emit-contract" => {
                emit_contract = args.next().ok_or("--emit-contract needs a path")?
            }
            "--package-name" => package_name = args.next().ok_or("--package-name needs a value")?,
            "--package-version" => {
                package_version = args.next().ok_or("--package-version needs a value")?
            }
            "--declaration-artifact" => {
                declaration_artifact = args.next().ok_or("--declaration-artifact needs a path")?
            }
            "--implementation-artifact" => {
                implementation_artifact = args
                    .next()
                    .ok_or("--implementation-artifact needs a path")?
            }
            unknown => return Err(format!("unknown argument {unknown:?}").into()),
        }
    }
    let project = if !help && validate_contract_paths.is_empty() {
        project.canonicalize()?
    } else {
        project
    };
    Ok(Request {
        project_id: project.to_string_lossy().into_owned(),
        generation: 1,
        sources: vec![],
        compiler_executable: compiler,
        compiler_args: vec![],
        typefacts_executable: typefacts,
        typefacts_args: vec!["-project".into(), project.to_string_lossy().into_owned()],
        contract_paths,
        format,
        certify,
        validate_contract_paths,
        emit_contract,
        package_name,
        package_version,
        declaration_artifact,
        implementation_artifact,
        oxlint: false,
        oxlint_args: vec![],
        help,
        serve,
    })
}

fn request_from_oxlint_args(arguments: &[String]) -> Result<Request, Box<dyn std::error::Error>> {
    let mut project = PathBuf::from("tsconfig.json");
    let mut compiler = std::env::var("SOLID_COMPILER_FACTS_BIN").unwrap_or_default();
    let mut typefacts = default_typefacts_executable();
    let mut contract_paths = Vec::new();
    let mut oxlint_args = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--" {
            oxlint_args.extend_from_slice(&arguments[index + 1..]);
            break;
        }
        if let Some(value) = argument.strip_prefix("--project=") {
            project = value.into();
        } else if let Some(value) = argument.strip_prefix("--contract=") {
            contract_paths.push(value.into());
        } else if let Some(value) = argument.strip_prefix("--typefacts=") {
            typefacts = value.into();
        } else if let Some(value) = argument.strip_prefix("--compiler=") {
            compiler = value.into();
        } else if matches!(
            argument.as_str(),
            "--project" | "-project" | "--contract" | "-contract" | "--typefacts" | "--compiler"
        ) {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| format!("{argument} needs a value"))?;
            match argument.as_str() {
                "--project" | "-project" => project = value.into(),
                "--contract" | "-contract" => contract_paths.push(value.clone()),
                "--typefacts" => typefacts = value.clone(),
                "--compiler" => compiler = value.clone(),
                _ => unreachable!(),
            }
        } else {
            oxlint_args.push(argument.clone());
        }
        index += 1;
    }
    let project = project.canonicalize()?;
    Ok(Request {
        project_id: project.to_string_lossy().into_owned(),
        generation: 1,
        sources: vec![],
        compiler_executable: compiler,
        compiler_args: vec![],
        typefacts_executable: typefacts,
        typefacts_args: vec!["-project".into(), project.to_string_lossy().into_owned()],
        contract_paths,
        format: "json".into(),
        certify: false,
        validate_contract_paths: vec![],
        emit_contract: String::new(),
        package_name: String::new(),
        package_version: String::new(),
        declaration_artifact: String::new(),
        implementation_artifact: String::new(),
        oxlint: true,
        oxlint_args,
        help: false,
        serve: false,
    })
}

fn print_help() {
    println!(
        "Usage: solid-check-rust [OPTIONS]\n\
         \n\
         Options:\n\
           --project <PATH>             TypeScript project (default: tsconfig.json)\n\
           --format <text|json>         Output format (default: text)\n\
           --certify                    Exit 1 unless the project is certified\n\
           --contract <PATH>            Override/discover a package contract (repeatable)\n\
           --validate-contract <PATH>   Validate a contract and artifact hashes\n\
           --emit-contract <PATH>       Write a generated solid-reactivity.json contract\n\
           --package-name <NAME>        Package name used by --emit-contract\n\
           --package-version <VERSION>  Optional package version\n\
           --declaration-artifact <PATH> Hash a declaration artifact into the contract\n\
           --implementation-artifact <PATH> Hash an implementation artifact into the contract\n\
           --typefacts <PATH>           TypeFacts service executable\n\
           --compiler <PATH>            Use the compiler-facts sidecar instead of native Rust\n\
           --serve                      Run the retained per-project check daemon (Unix only);\n\
                                        clients use it when SOLID_CHECK_DAEMON=1\n\
           -h, --help                   Print help"
    );
}

fn run_oxlint(
    arguments: &[String],
    snapshot: &Snapshot,
) -> Result<i32, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "solid-check-oxlint-{}-{unique}.json",
        std::process::id()
    ));
    let mut encoded = go_compatible_json(snapshot, false)?;
    encoded.push(b'\n');
    fs::write(&path, encoded)?;
    let executable = std::env::var("OXLINT_BIN").unwrap_or_else(|_| "oxlint".into());
    let mut command = Command::new(executable);
    if !arguments.iter().any(|argument| {
        argument == "--format"
            || argument == "-f"
            || argument.starts_with("--format=")
            || argument.starts_with("-f=")
    }) {
        command.arg("--format=default");
    }
    let status = command
        .args(arguments)
        .env("SOLID_CHECK_SNAPSHOT_PATH", &path)
        .status();
    let _ = fs::remove_file(path);
    let status = status?;
    Ok(status.code().unwrap_or(2))
}

fn emit_package_contract(
    request: &Request,
    program: &solid_reactive_ir::Program,
) -> Result<(), Box<dyn std::error::Error>> {
    if request.package_name.is_empty() {
        return Err("--package-name is required with --emit-contract".into());
    }
    if let Some(unresolved) = program
        .static_violations
        .iter()
        .find(|violation| violation.id.starts_with("SC9"))
    {
        return Err(format!(
            "emit package contract: unresolved effect at {}:{}: {}",
            unresolved.location.path, unresolved.location.start_byte, unresolved.message
        )
        .into());
    }
    if let Some(unresolved) = program.unresolved_cleanup_returns.first() {
        return Err(format!(
            "emit package contract: unresolved cleanup return at {}:{}",
            unresolved.location.path, unresolved.location.start_byte
        )
        .into());
    }
    let output = Path::new(&request.emit_contract);
    let artifacts = solid_reactive_ir::ContractArtifacts {
        declaration: (!request.declaration_artifact.is_empty())
            .then(|| artifact_for_file(output, Path::new(&request.declaration_artifact)))
            .transpose()?,
        implementation: (!request.implementation_artifact.is_empty())
            .then(|| artifact_for_file(output, Path::new(&request.implementation_artifact)))
            .transpose()?,
    };
    let contract = solid_reactive_ir::PackageContract {
        schema_version: 1,
        package: solid_reactive_ir::ContractPackage {
            name: request.package_name.clone(),
            version: request.package_version.clone(),
        },
        compiler_facts_protocol: 1,
        artifacts,
        exports: (*program.contract_exports).clone(),
        evidence: solid_reactive_ir::ContractEvidence {
            kind: "generated".into(),
            generator: "solid-check".into(),
        },
        contract_hash: String::new(),
        source_path: String::new(),
    };
    contract.validate().map_err(|error| error.to_string())?;
    let mut encoded = go_compatible_json(&contract, true)?;
    encoded.push(b'\n');
    fs::write(output, encoded)?;
    Ok(())
}

fn go_compatible_json<T: Serialize>(value: &T, pretty: bool) -> Result<Vec<u8>, serde_json::Error> {
    let encoded = if pretty {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };
    // Go's encoding/json escapes these code points by default. Keeping that
    // behavior makes Rust and Go snapshots/contracts byte-for-byte
    // interchangeable during the additive migration.
    Ok(encoded
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
        .into_bytes())
}

fn artifact_for_file(
    contract_path: &Path,
    artifact_path: &Path,
) -> Result<solid_reactive_ir::ContractArtifact, Box<dyn std::error::Error>> {
    let contract_directory = contract_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    let artifact = artifact_path.canonicalize()?;
    let relative = artifact
        .strip_prefix(&contract_directory)
        .map_err(|_| "package contract artifact must be a file inside the contract directory")?;
    if relative.as_os_str().is_empty() || !artifact.is_file() {
        return Err(
            "package contract artifact must be a file inside the contract directory".into(),
        );
    }
    let data = fs::read(&artifact)?;
    Ok(solid_reactive_ir::ContractArtifact {
        path: relative.to_string_lossy().replace('\\', "/"),
        hash: format!("sha256:{:x}", Sha256::digest(data)),
    })
}

use solid_ts_facts::v3::Operation;

/// A generation-scoped lifecycle request with an empty payload, used for the
/// open and sources handshakes the cold path pipelines ahead of the build.
fn lifecycle_request(
    operation: Operation,
    project_id: &str,
    generation: u64,
) -> solid_ts_facts::v3::Request {
    solid_ts_facts::v3::Request {
        schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
        request_id: 0,
        operation,
        project_id: project_id.into(),
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

fn decode_sources(
    response: solid_ts_facts::v3::Response,
) -> Result<Vec<SourceFile>, Box<dyn std::error::Error>> {
    response
        .sources
        .into_iter()
        .map(|source| {
            Ok(SourceFile {
                path: source.path,
                source: String::from_utf8(source.source)?,
                compiler_options: Default::default(),
            })
        })
        .collect()
}

/// A per-project daemon holding the retained `NativeIncrementalSession` behind
/// a Unix socket, so repeat CLI checks reuse the warm session instead of
/// rebuilding the TypeScript program and demand closure from scratch.
///
/// Opt-in: clients use it only when `SOLID_CHECK_DAEMON=1`. The socket path is
/// derived from the canonical project id. Before every answer the daemon
/// resynchronizes with the filesystem: a changed tsconfig, a changed source
/// directory (file created, deleted, or renamed), or an unreadable known file
/// rebuilds the whole session; changed file contents become incremental
/// overlay updates. The response body is byte-identical to one-shot output.
#[cfg(unix)]
mod daemon {
    use std::{
        collections::BTreeMap,
        error::Error,
        fs,
        io::{self, BufRead, BufReader, Read, Write},
        os::unix::net::{UnixListener, UnixStream},
        path::PathBuf,
        process::{Command, Stdio},
        time::{Duration, Instant, SystemTime},
    };

    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use solid_facts_backend::{
        NativeIncrementalSession, SourceChange, SourceFile, TypeFactsSidecar, analyze_project,
        discovered_contract_paths, imported_package_roots,
    };

    use super::{Request, go_compatible_json};

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CheckRequest {
        project_id: String,
        #[serde(default)]
        contract_paths: Vec<String>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CheckHeader {
        ok: bool,
        #[serde(default)]
        status: String,
        #[serde(default)]
        error: String,
    }

    pub fn enabled() -> bool {
        std::env::var("SOLID_CHECK_DAEMON").is_ok_and(|value| value == "1" || value == "true")
    }

    pub fn eligible(request: &Request) -> bool {
        request.sources.is_empty()
            && request.compiler_executable.is_empty()
            && request.emit_contract.is_empty()
            && !request.oxlint
            && matches!(request.format.as_str(), "json" | "text")
    }

    fn socket_path(project_id: &str) -> PathBuf {
        let digest = Sha256::digest(project_id.as_bytes());
        let mut name = String::from("solid-check-");
        for byte in &digest[..8] {
            name.push_str(&format!("{byte:02x}"));
        }
        std::env::temp_dir().join(format!("{name}.sock"))
    }

    fn idle_limit() -> Duration {
        let seconds = std::env::var("SOLID_CHECK_DAEMON_IDLE_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(600);
        Duration::from_secs(seconds.max(1))
    }

    struct State {
        project: PathBuf,
        session: NativeIncrementalSession,
        sources: BTreeMap<String, SourceFile>,
        hashes: BTreeMap<String, [u8; 32]>,
        dirs: BTreeMap<PathBuf, Option<SystemTime>>,
        tsconfig_hash: [u8; 32],
        last: Option<CachedAnswer>,
    }

    /// The previous answer plus everything needed to prove it still holds:
    /// the generation it was computed at, the requested contract overrides,
    /// the imported package roots (fixed for a generation), and the resolved
    /// contract files with their content hashes. Bundled contracts are
    /// compiled into the binary and need no validation.
    struct CachedAnswer {
        generation: u64,
        explicit: Vec<String>,
        modules: Vec<String>,
        contract_files: Vec<(PathBuf, [u8; 32])>,
        status: String,
        body: Vec<u8>,
    }

    enum Sync {
        Ready(Vec<SourceChange>),
        Rebuild,
    }

    impl State {
        fn open(request: &Request) -> Result<Self, Box<dyn Error>> {
            let typescript =
                TypeFactsSidecar::spawn(&request.typefacts_executable, &request.typefacts_args)?;
            let (session, configured) =
                NativeIncrementalSession::open_pipelined(request.project_id.clone(), typescript)?;
            let project = PathBuf::from(&request.project_id);
            let tsconfig_hash = hash_file(&project)?;
            let mut sources = BTreeMap::new();
            let mut hashes = BTreeMap::new();
            let mut dirs = BTreeMap::new();
            if let Some(parent) = project.parent() {
                dirs.insert(parent.to_path_buf(), directory_stamp(parent));
            }
            for source in configured {
                hashes.insert(source.path.clone(), content_hash(source.source.as_bytes()));
                if let Some(parent) = PathBuf::from(&source.path).parent() {
                    dirs.entry(parent.to_path_buf())
                        .or_insert_with(|| directory_stamp(parent));
                }
                sources.insert(source.path.clone(), source);
            }
            Ok(Self {
                project,
                session,
                sources,
                hashes,
                dirs,
                tsconfig_hash,
                last: None,
            })
        }

        /// Reconcile the retained session with the filesystem. Content edits
        /// to known files become overlay updates; anything that can change
        /// the project's file set demands a full rebuild.
        fn resync(&mut self) -> Result<Sync, Box<dyn Error>> {
            if hash_file(&self.project)? != self.tsconfig_hash {
                return Ok(Sync::Rebuild);
            }
            for (dir, recorded) in &self.dirs {
                if directory_stamp(dir) != *recorded {
                    return Ok(Sync::Rebuild);
                }
            }
            let mut changes = Vec::new();
            for (path, recorded) in &self.hashes {
                let Ok(bytes) = fs::read(path) else {
                    return Ok(Sync::Rebuild);
                };
                if content_hash(&bytes) != *recorded {
                    changes.push(SourceChange {
                        path: path.clone(),
                        version: self.session.generation() + 1,
                        source: Some(String::from_utf8(bytes)?),
                        compiler_options: Default::default(),
                    });
                }
            }
            for change in &changes {
                let Some(text) = &change.source else { continue };
                self.hashes
                    .insert(change.path.clone(), content_hash(text.as_bytes()));
                self.sources.insert(
                    change.path.clone(),
                    SourceFile {
                        path: change.path.clone(),
                        source: text.clone(),
                        compiler_options: Default::default(),
                    },
                );
            }
            Ok(Sync::Ready(changes))
        }
    }

    fn content_hash(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    fn hash_file(path: &std::path::Path) -> Result<[u8; 32], Box<dyn Error>> {
        Ok(content_hash(&fs::read(path)?))
    }

    fn directory_stamp(path: &std::path::Path) -> Option<SystemTime> {
        fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok())
    }

    pub fn serve(request: &Request) -> Result<i32, Box<dyn Error>> {
        let socket = socket_path(&request.project_id);
        if UnixStream::connect(&socket).is_ok() {
            return Ok(0); // a live daemon already serves this project
        }
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket)?;
        let mut state = State::open(request)?;
        // Blocking accept keeps request latency free of poll sleeps; a
        // watchdog thread ends the whole process after the idle limit.
        let idle = idle_limit();
        let last_activity = std::sync::Arc::new(std::sync::Mutex::new(Instant::now()));
        let watchdog_activity = std::sync::Arc::clone(&last_activity);
        let watchdog_socket = socket.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5).min(idle));
                let idle_for = watchdog_activity
                    .lock()
                    .map(|instant| instant.elapsed())
                    .unwrap_or(idle);
                if idle_for >= idle {
                    let _ = fs::remove_file(&watchdog_socket);
                    std::process::exit(0);
                }
            }
        });
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Ok(mut instant) = last_activity.lock() {
                        *instant = Instant::now();
                    }
                    if let Err(error) = handle(&mut state, request, stream) {
                        eprintln!("solid-check daemon: {error}");
                    }
                    if let Ok(mut instant) = last_activity.lock() {
                        *instant = Instant::now();
                    }
                }
                Err(error) => {
                    let _ = fs::remove_file(&socket);
                    return Err(error.into());
                }
            }
        }
    }

    fn handle(
        state: &mut State,
        request: &Request,
        stream: UnixStream,
    ) -> Result<(), Box<dyn Error>> {
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let check: CheckRequest = serde_json::from_str(&line)?;
        let mut stream = reader.into_inner();
        if check.project_id != request.project_id {
            return respond_error(&mut stream, "daemon serves a different project");
        }
        let outcome = answer(state, request, &check);
        match outcome {
            Ok((status, body)) => {
                let header = serde_json::to_vec(&CheckHeader {
                    ok: true,
                    status,
                    error: String::new(),
                })?;
                stream.write_all(&header)?;
                stream.write_all(b"\n")?;
                stream.write_all(&body)?;
                stream.flush()?;
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                let _ = respond_error(&mut stream, &message);
                Err(message.into())
            }
        }
    }

    fn respond_error(stream: &mut UnixStream, message: &str) -> Result<(), Box<dyn Error>> {
        let header = serde_json::to_vec(&CheckHeader {
            ok: false,
            status: String::new(),
            error: message.into(),
        })?;
        stream.write_all(&header)?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        Ok(())
    }

    fn answer(
        state: &mut State,
        request: &Request,
        check: &CheckRequest,
    ) -> Result<(String, Vec<u8>), Box<dyn Error>> {
        let changes = match state.resync()? {
            Sync::Rebuild => {
                *state = State::open(request)?;
                Vec::new()
            }
            Sync::Ready(changes) => changes,
        };
        if changes.is_empty()
            && let Some(cached) = cached_answer(state, check)?
        {
            return Ok(cached);
        }
        let facts = if changes.is_empty() {
            state.session.analyze()?
        } else {
            state.session.edit(changes, None)?
        };
        let sources = state.sources.values().cloned().collect::<Vec<_>>();
        let analysis = analyze_project(&state.project, &sources, &facts, &check.contract_paths)?;
        let mut body = go_compatible_json(&analysis.snapshot, true)?;
        body.push(b'\n');
        let status = analysis.snapshot.status.to_string();
        let modules = imported_package_roots(&facts);
        state.last = Some(CachedAnswer {
            generation: state.session.generation(),
            explicit: check.contract_paths.clone(),
            contract_files: contract_files(state, &modules, &check.contract_paths)?,
            modules,
            status: status.clone(),
            body: body.clone(),
        });
        Ok((status, body))
    }

    /// Return the cached snapshot only when its inputs still hold: same
    /// generation, same explicit contracts, the discovery walk resolves the
    /// same contract files, and their contents are unchanged.
    fn cached_answer(
        state: &State,
        check: &CheckRequest,
    ) -> Result<Option<(String, Vec<u8>)>, Box<dyn Error>> {
        let Some(cached) = &state.last else {
            return Ok(None);
        };
        if cached.generation != state.session.generation()
            || cached.explicit != check.contract_paths
        {
            return Ok(None);
        }
        let current = contract_files(state, &cached.modules, &check.contract_paths)?;
        if current.len() != cached.contract_files.len()
            || current
                .iter()
                .zip(&cached.contract_files)
                .any(|(now, then)| now != then)
        {
            return Ok(None);
        }
        Ok(Some((cached.status.clone(), cached.body.clone())))
    }

    /// The current on-disk contract inputs: discovered files for the module
    /// set plus explicit overrides, each with its content hash, sorted.
    fn contract_files(
        state: &State,
        modules: &[String],
        explicit: &[String],
    ) -> Result<Vec<(PathBuf, [u8; 32])>, Box<dyn Error>> {
        let directory = state
            .project
            .parent()
            .ok_or("tsconfig has no parent directory")?;
        let mut paths = discovered_contract_paths(directory, modules)?;
        paths.extend(explicit.iter().map(PathBuf::from));
        paths.sort();
        paths.dedup();
        let mut files = Vec::with_capacity(paths.len());
        for path in paths {
            files.push((path.clone(), hash_file(&path)?));
        }
        Ok(files)
    }

    pub fn check(request: &Request) -> Result<i32, Box<dyn Error>> {
        let socket = socket_path(&request.project_id);
        let stream = match UnixStream::connect(&socket) {
            Ok(stream) => stream,
            Err(_) => spawn_and_connect(request, &socket)?,
        };
        let payload = serde_json::to_vec(&CheckRequest {
            project_id: request.project_id.clone(),
            contract_paths: request.contract_paths.clone(),
        })?;
        let mut stream = stream;
        stream.write_all(&payload)?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        let mut reader = BufReader::new(stream);
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        let header: CheckHeader = serde_json::from_str(&header_line)?;
        if !header.ok {
            return Err(header.error.into());
        }
        let mut body = Vec::new();
        reader.read_to_end(&mut body)?;
        match request.format.as_str() {
            "json" => io::stdout().write_all(&body)?,
            "text" => print_text(&request.project_id, &body)?,
            format => return Err(format!("unsupported format {format:?}").into()),
        }
        if request.certify && header.status != "certified" {
            return Ok(1);
        }
        Ok(0)
    }

    fn print_text(project_id: &str, body: &[u8]) -> Result<(), Box<dyn Error>> {
        let snapshot: serde_json::Value = serde_json::from_slice(body)?;
        let status = snapshot["status"].as_str().unwrap_or_default();
        println!("{project_id}: {status}");
        if let Some(findings) = snapshot["findings"].as_array() {
            for finding in findings {
                println!(
                    "{} [{}] {}",
                    finding["id"].as_str().unwrap_or_default(),
                    finding["kind"].as_str().unwrap_or_default(),
                    finding["message"].as_str().unwrap_or_default()
                );
            }
        }
        Ok(())
    }

    fn spawn_and_connect(
        request: &Request,
        socket: &std::path::Path,
    ) -> Result<UnixStream, Box<dyn Error>> {
        let executable = std::env::current_exe()?;
        Command::new(executable)
            .arg("--serve")
            .arg("--project")
            .arg(&request.project_id)
            .arg("--typefacts")
            .arg(&request.typefacts_executable)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match UnixStream::connect(socket) {
                Ok(stream) => return Ok(stream),
                Err(error) => {
                    if Instant::now() >= deadline {
                        return Err(format!("daemon did not start: {error}").into());
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            let program = std::env::current_exe()
                .ok()
                .and_then(|path| {
                    path.file_stem()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| "solid-facts-backend".into());
            eprintln!("{program}: {error}");
            let exit_code = if error
                .downcast_ref::<BackendError>()
                .is_some_and(|error| matches!(error, BackendError::Handshake(_)))
            {
                3
            } else {
                2
            };
            std::process::exit(exit_code);
        }
    }
}
