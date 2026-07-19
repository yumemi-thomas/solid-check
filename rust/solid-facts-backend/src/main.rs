mod daemon_cache;
mod json_output;
mod snapshot_emission;

use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use solid_facts_backend::{
    BackendError, CompilerSidecar, SourceFile, TypeFactsSidecar, analyze_project_measured_with,
    build_project, build_project_native_measured, bundled_solid_js_contract,
    default_typefacts_executable, package_contract_statuses, read_package_contract,
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
    check_contracts: bool,
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
    if diagnostics && request.check_contracts {
        let statuses = package_contract_statuses(
            Path::new(&facts.project_id),
            &facts,
            &request.contract_paths,
        )?;
        let missing = statuses
            .iter()
            .filter(|status| status.status == "missing")
            .collect::<Vec<_>>();
        match request.format.as_str() {
            "json" => {
                let report = serde_json::json!({
                    "packages": statuses,
                    "missing": missing.len(),
                });
                let mut stdout = io::stdout().lock();
                stdout.write_all(&json_output::go_compatible(&report, true)?)?;
                stdout.write_all(b"\n")?;
            }
            "text" => {
                for status in &statuses {
                    println!(
                        "{}: {} ({})",
                        status.name, status.status, status.contract_path
                    );
                }
                if statuses.is_empty() {
                    println!("No imported Solid packages need contracts.");
                }
            }
            format => return Err(format!("unsupported format {format:?}").into()),
        }
        return Ok(i32::from(!missing.is_empty()));
    }
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
        let emission = snapshot_emission::emit(
            &request.format,
            &request.project_id,
            &snapshot,
            request.certify,
        )?;
        io::stdout().write_all(&emission.output)?;
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
        return Ok(emission.exit_code);
    } else {
        serde_json::to_writer(io::stdout(), &facts)?;
    }
    Ok(0)
}

fn request_from_args() -> Result<Request, Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mut project = PathBuf::from("tsconfig.json");
    let mut compiler = std::env::var("SOLID_COMPILER_FACTS_BIN").unwrap_or_default();
    let mut typefacts = default_typefacts_executable();
    let mut contract_paths = Vec::new();
    let mut format = "text".to_owned();
    let mut certify = false;
    let mut check_contracts = false;
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
            "--check-contracts" => check_contracts = true,
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
        check_contracts,
        validate_contract_paths,
        emit_contract,
        package_name,
        package_version,
        declaration_artifact,
        implementation_artifact,
        help,
        serve,
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
           --check-contracts            Report imported Solid packages without contracts\n\
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
    let mut encoded = json_output::go_compatible(&contract, true)?;
    encoded.push(b'\n');
    fs::write(output, encoded)?;
    Ok(())
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
        compact_demands: None,
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
mod daemon;

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
