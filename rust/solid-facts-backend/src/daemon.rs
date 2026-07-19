//! Per-project daemon holding the retained `NativeIncrementalSession` behind
//! a Unix socket, so repeat CLI checks reuse the warm session instead of
//! rebuilding the TypeScript program and demand closure from scratch.
//!
//! Opt-in: clients use it only when `SOLID_CHECK_DAEMON=1`. The socket path is
//! derived from the canonical project id. Before every answer the daemon
//! resynchronizes with the filesystem: a changed tsconfig, a changed source
//! directory (file created, deleted, or renamed), or an unreadable known file
//! rebuilds the whole session; changed file contents become incremental
//! overlay updates. The response body is byte-identical to one-shot output.
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

use super::{Request, snapshot_emission};
use crate::daemon_cache::{CachedAnswer, CachedSnapshot, ContractFile};

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
        && !request.check_contracts
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

fn handle(state: &mut State, request: &Request, stream: UnixStream) -> Result<(), Box<dyn Error>> {
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
    let body =
        snapshot_emission::emit("json", &request.project_id, &analysis.snapshot, false)?.output;
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
) -> Result<Option<CachedSnapshot>, Box<dyn Error>> {
    let Some(cached) = &state.last else {
        return Ok(None);
    };
    let current = contract_files(state, &cached.modules, &check.contract_paths)?;
    Ok(cached.snapshot_if_current(state.session.generation(), &check.contract_paths, &current))
}

/// The current on-disk contract inputs: package manifests and discovered
/// contracts for the module set plus explicit overrides, each with its
/// content hash, sorted.
fn contract_files(
    state: &State,
    modules: &[String],
    explicit: &[String],
) -> Result<Vec<ContractFile>, Box<dyn Error>> {
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
    let snapshot: solid_facts_backend::Snapshot = serde_json::from_slice(&body)?;
    if snapshot.status != header.status {
        return Err("daemon response status does not match snapshot".into());
    }
    let emission = snapshot_emission::emit(
        &request.format,
        &request.project_id,
        &snapshot,
        request.certify,
    )?;
    io::stdout().write_all(&emission.output)?;
    Ok(emission.exit_code)
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
