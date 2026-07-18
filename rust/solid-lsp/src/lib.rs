use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use serde_json::{Value, json};
use solid_compiler_facts::CompilerOptions;
use solid_facts_backend::{
    NativeIncrementalSession, Snapshot, SnapshotFinding, SourceChange, SourceFile, SourceLocation,
    TypeFactsCancellation, TypeFactsSidecar, analyze_project,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LspError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("backend: {0}")]
    Backend(#[from] solid_facts_backend::BackendError),
    #[error("reactive IR: {0}")]
    Ir(#[from] solid_reactive_ir::BuildError),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("method not found")]
    MethodNotFound,
    #[error("{0}")]
    RequestFailed(String),
    #[error("{0}")]
    Protocol(String),
}

struct Document {
    version: i64,
}

struct Server<W> {
    project: PathBuf,
    contract_paths: Vec<String>,
    session: Option<NativeIncrementalSession>,
    cancellation: TypeFactsCancellation,
    jobs: Option<mpsc::Sender<AnalysisJob>>,
    events: Option<mpsc::Sender<Event>>,
    current_cancel: Option<Arc<AtomicBool>>,
    analysis_sequence: u64,
    completed_sequence: u64,
    pending_snapshots: Vec<Value>,
    pending_barriers: Vec<Value>,
    sources: HashMap<String, SourceFile>,
    documents: HashMap<String, Document>,
    wire_versions: HashMap<String, u64>,
    snapshot: Snapshot,
    published: HashMap<String, String>,
    writer: W,
}

struct AnalysisJob {
    sequence: u64,
    changes: Vec<SourceChange>,
    sources: Vec<SourceFile>,
    project: PathBuf,
    contract_paths: Vec<String>,
    cancelled: Arc<AtomicBool>,
}

enum Event {
    Frame(Vec<u8>),
    InputError(String),
    Eof,
    Analysis {
        sequence: u64,
        result: Result<Snapshot, String>,
        cancelled: bool,
    },
}

pub fn serve(
    project: &Path,
    typefacts_executable: &str,
    input: impl Read + Send + 'static,
    output: impl Write,
) -> Result<(), LspError> {
    serve_with_contracts(project, typefacts_executable, &[], input, output)
}

pub fn serve_with_contracts(
    project: &Path,
    typefacts_executable: &str,
    contract_paths: &[String],
    input: impl Read + Send + 'static,
    output: impl Write,
) -> Result<(), LspError> {
    let project = project.canonicalize()?;
    let project_id = project.to_string_lossy().into_owned();
    let typescript = TypeFactsSidecar::spawn(
        typefacts_executable,
        &["-project".into(), project_id.clone()],
    )?;
    let (mut session, configured) =
        NativeIncrementalSession::open_pipelined(project_id, typescript)?;
    let sources = configured
        .iter()
        .cloned()
        .map(|source| (source.path.clone(), source))
        .collect();
    let cancellation = session.cancellation_handle();
    let facts = session.analyze()?;
    let snapshot = analyze_project(&project, &configured, &facts, contract_paths)?.snapshot;
    let mut server = Server {
        project,
        contract_paths: contract_paths.to_vec(),
        session: Some(session),
        cancellation,
        jobs: None,
        events: None,
        current_cancel: None,
        analysis_sequence: 0,
        completed_sequence: 0,
        pending_snapshots: vec![],
        pending_barriers: vec![],
        sources,
        documents: HashMap::new(),
        wire_versions: HashMap::new(),
        snapshot,
        published: HashMap::new(),
        writer: output,
    };
    server.run(input)
}

impl<W: Write> Server<W> {
    fn run(&mut self, input: impl Read + Send + 'static) -> Result<(), LspError> {
        let (event_sender, event_receiver) = mpsc::channel();
        let reader_sender = event_sender.clone();
        std::thread::spawn(move || {
            let mut input = BufReader::new(input);
            loop {
                match read_frame(&mut input) {
                    Ok(Some(frame)) => {
                        if reader_sender.send(Event::Frame(frame)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => {
                        let _ = reader_sender.send(Event::Eof);
                        break;
                    }
                    Err(error) => {
                        let _ = reader_sender.send(Event::InputError(error.to_string()));
                        break;
                    }
                }
            }
        });
        let (job_sender, job_receiver) = mpsc::channel::<AnalysisJob>();
        let worker_sender = event_sender.clone();
        let mut session = self
            .session
            .take()
            .expect("analysis session is started exactly once");
        std::thread::spawn(move || {
            for job in job_receiver {
                let result = (|| {
                    let facts = session
                        .edit(job.changes.clone(), Some(&job.cancelled))
                        .map_err(|error| error.to_string())?;
                    analyze_project(&job.project, &job.sources, &facts, &job.contract_paths)
                        .map(|analysis| analysis.snapshot)
                        .map_err(|error| error.to_string())
                })();
                let cancelled = job.cancelled.load(Ordering::Acquire)
                    || result
                        .as_ref()
                        .is_err_and(|error| error == "analysis cancelled");
                if worker_sender
                    .send(Event::Analysis {
                        sequence: job.sequence,
                        result,
                        cancelled,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        self.jobs = Some(job_sender);
        self.events = Some(event_sender);
        let mut input_eof = false;
        while let Ok(event) = event_receiver.recv() {
            match event {
                Event::Frame(payload) => {
                    if self.handle_message(&payload)? {
                        return Ok(());
                    }
                }
                Event::InputError(error) => return Err(LspError::Protocol(error)),
                Event::Eof => {
                    input_eof = true;
                    if self.completed_sequence >= self.analysis_sequence {
                        return Ok(());
                    }
                }
                Event::Analysis {
                    sequence,
                    result,
                    cancelled,
                } => self.complete_analysis(sequence, result, cancelled)?,
            }
            if input_eof && self.completed_sequence >= self.analysis_sequence {
                return Ok(());
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, payload: &[u8]) -> Result<bool, LspError> {
        let message: Value = match serde_json::from_slice(payload) {
            Ok(message) => message,
            Err(error) => {
                self.respond(Value::Null, None, Some((-32700, error.to_string())))?;
                return Ok(false);
            }
        };
        let id = message.get("id").cloned();
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        if method == "$/cancelRequest" {
            if let Some(cancelled_id) = params.get("id") {
                self.cancel_snapshot(cancelled_id)?;
            }
            return Ok(false);
        }
        if method == "solid/checkSnapshot"
            && self.completed_sequence < self.analysis_sequence
            && let Some(id) = id
        {
            self.pending_snapshots.push(id);
            return Ok(false);
        }
        if method == "phase0/barrier"
            && self.completed_sequence < self.analysis_sequence
            && let Some(id) = id
        {
            self.pending_barriers.push(id);
            return Ok(false);
        }
        match self.dispatch(method, params) {
            Ok((result, exit)) => {
                if let Some(id) = id {
                    self.respond(id, Some(result), None)?;
                }
                Ok(exit)
            }
            Err(error) => {
                if let Some(id) = id {
                    let code = match &error {
                        LspError::InvalidParams(_) => -32602,
                        LspError::MethodNotFound => -32601,
                        LspError::RequestFailed(_) => -32001,
                        _ => -32603,
                    };
                    self.respond(id, None, Some((code, error.to_string())))?;
                }
                Ok(false)
            }
        }
    }

    fn dispatch(&mut self, method: &str, params: Value) -> Result<(Value, bool), LspError> {
        match method {
            "initialize" => Ok((
                json!({
                    "capabilities": {
                        "positionEncoding": "utf-16",
                        "textDocumentSync": {"openClose": true, "change": 1},
                        "codeActionProvider": true,
                        "experimental": {
                            "solid/checkSnapshot": true,
                            "solid/explainFinding": true
                        }
                    },
                    "serverInfo": {"name": "solid-checkd"}
                }),
                false,
            )),
            "initialized" => {
                self.publish_diagnostics(true)?;
                Ok((Value::Null, false))
            }
            "shutdown" => Ok((Value::Null, false)),
            "exit" => Ok((Value::Null, true)),
            "$/cancelRequest" | "phase0/barrier" => Ok((Value::Null, false)),
            "textDocument/didOpen" => {
                let document = params
                    .get("textDocument")
                    .ok_or_else(|| LspError::InvalidParams("missing textDocument".into()))?;
                let uri = string_field(document, "uri")?;
                let path = uri_to_path(uri)?;
                let version = integer_field(document, "version")?;
                let text = string_field(document, "text")?;
                self.documents.insert(path.clone(), Document { version });
                self.update(path, text.into(), version)?;
                Ok((Value::Null, false))
            }
            "textDocument/didChange" => {
                let document = params
                    .get("textDocument")
                    .ok_or_else(|| LspError::InvalidParams("missing textDocument".into()))?;
                let uri = string_field(document, "uri")?;
                let path = uri_to_path(uri)?;
                let version = integer_field(document, "version")?;
                if self
                    .documents
                    .get(&path)
                    .is_some_and(|document| version <= document.version)
                {
                    return Ok((Value::Null, false));
                }
                let changes = params
                    .get("contentChanges")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        LspError::InvalidParams("contentChanges must be an array".into())
                    })?;
                if changes.len() != 1 {
                    return Err(LspError::InvalidParams(
                        "solid-checkd requires exactly one full-document content change".into(),
                    ));
                }
                let text = string_field(&changes[0], "text")?;
                self.documents.insert(path.clone(), Document { version });
                self.update(path, text.into(), version)?;
                Ok((Value::Null, false))
            }
            "textDocument/didClose" => {
                let document = params
                    .get("textDocument")
                    .ok_or_else(|| LspError::InvalidParams("missing textDocument".into()))?;
                let path = uri_to_path(string_field(document, "uri")?)?;
                self.documents.remove(&path);
                match fs::read_to_string(&path) {
                    Ok(source) => self.update(path, source, 0)?,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        self.delete(path)?
                    }
                    Err(error) => return Err(error.into()),
                }
                Ok((Value::Null, false))
            }
            "textDocument/diagnostic" => {
                let uri = params
                    .pointer("/textDocument/uri")
                    .and_then(Value::as_str)
                    .ok_or_else(|| LspError::InvalidParams("missing textDocument.uri".into()))?;
                Ok((
                    json!({"kind": "full", "items": self.diagnostics_for_path(&uri_to_path(uri)?)}),
                    false,
                ))
            }
            "solid/checkSnapshot" => Ok((self.snapshot_json(), false)),
            "solid/explainFinding" => {
                let id = params
                    .get("id")
                    .or_else(|| params.get("findingId"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| LspError::InvalidParams("missing finding id".into()))?;
                let finding = self
                    .snapshot
                    .findings
                    .iter()
                    .find(|finding| finding.id == id)
                    .ok_or_else(|| {
                        LspError::RequestFailed(format!("finding {id:?} was not found"))
                    })?;
                Ok((serde_json::to_value(finding)?, false))
            }
            "textDocument/codeAction" => {
                let uri = params
                    .pointer("/textDocument/uri")
                    .and_then(Value::as_str)
                    .ok_or_else(|| LspError::InvalidParams("missing textDocument.uri".into()))?;
                Ok((Value::Array(self.code_actions(uri)?), false))
            }
            _ => Err(LspError::MethodNotFound),
        }
    }

    fn update(
        &mut self,
        path: String,
        source: String,
        editor_version: i64,
    ) -> Result<(), LspError> {
        let version = self.next_wire_version(&path, editor_version);
        let compiler_options = self
            .sources
            .get(&path)
            .map(|source| source.compiler_options.clone())
            .unwrap_or_default();
        let change = SourceChange {
            path: path.clone(),
            version,
            source: Some(source.clone()),
            compiler_options: compiler_options.clone(),
        };
        self.sources.insert(
            path.clone(),
            SourceFile {
                path: path.clone(),
                source,
                compiler_options,
            },
        );
        self.schedule_analysis(vec![change])
    }

    fn delete(&mut self, path: String) -> Result<(), LspError> {
        let version = self.next_wire_version(&path, 0);
        let change = SourceChange {
            path: path.clone(),
            version,
            source: None,
            compiler_options: CompilerOptions::default(),
        };
        self.sources.remove(&path);
        self.schedule_analysis(vec![change])
    }

    fn next_wire_version(&mut self, path: &str, editor_version: i64) -> u64 {
        let current = self.wire_versions.get(path).copied().unwrap_or(0);
        let requested = u64::try_from(editor_version).unwrap_or(0);
        let next = requested.max(current.saturating_add(1));
        self.wire_versions.insert(path.into(), next);
        next
    }

    fn ordered_sources(&self) -> Vec<SourceFile> {
        let mut sources = self.sources.values().cloned().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.path.cmp(&right.path));
        sources
    }

    fn schedule_analysis(&mut self, changes: Vec<SourceChange>) -> Result<(), LspError> {
        if let Some(cancelled) = self.current_cancel.take() {
            cancelled.store(true, Ordering::Release);
            self.cancellation.cancel_active()?;
        }
        self.analysis_sequence = self.analysis_sequence.saturating_add(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let job = AnalysisJob {
            sequence: self.analysis_sequence,
            changes,
            sources: self.ordered_sources(),
            project: self.project.clone(),
            contract_paths: self.contract_paths.clone(),
            cancelled: Arc::clone(&cancelled),
        };
        self.current_cancel = Some(cancelled);
        self.jobs
            .as_ref()
            .ok_or_else(|| LspError::Protocol("analysis worker is not running".into()))?
            .send(job)
            .map_err(|_| LspError::Protocol("analysis worker stopped".into()))
    }

    fn complete_analysis(
        &mut self,
        sequence: u64,
        result: Result<Snapshot, String>,
        cancelled: bool,
    ) -> Result<(), LspError> {
        self.completed_sequence = self.completed_sequence.max(sequence);
        if sequence != self.analysis_sequence {
            return Ok(());
        }
        self.current_cancel = None;
        match result {
            Ok(snapshot) => {
                self.snapshot = snapshot;
                self.publish_diagnostics(false)?;
                let result = self.snapshot_json();
                for id in std::mem::take(&mut self.pending_snapshots) {
                    self.respond(id, Some(result.clone()), None)?;
                }
                for id in std::mem::take(&mut self.pending_barriers) {
                    self.respond(id, Some(Value::Null), None)?;
                }
            }
            Err(error) => {
                let code = if cancelled { -32800 } else { -32603 };
                for id in std::mem::take(&mut self.pending_snapshots) {
                    self.respond(id, None, Some((code, error.clone())))?;
                }
                for id in std::mem::take(&mut self.pending_barriers) {
                    self.respond(id, None, Some((code, error.clone())))?;
                }
            }
        }
        Ok(())
    }

    fn cancel_snapshot(&mut self, id: &Value) -> Result<(), LspError> {
        if let Some(index) = self
            .pending_snapshots
            .iter()
            .position(|pending| pending == id)
        {
            let id = self.pending_snapshots.remove(index);
            self.respond(id, None, Some((-32800, "request cancelled".into())))?;
            return Ok(());
        }
        if let Some(index) = self
            .pending_barriers
            .iter()
            .position(|pending| pending == id)
        {
            let id = self.pending_barriers.remove(index);
            self.respond(id, None, Some((-32800, "request cancelled".into())))?;
        }
        Ok(())
    }

    fn diagnostics_for_path(&self, path: &str) -> Vec<Value> {
        let mut diagnostics = self
            .snapshot
            .findings
            .iter()
            .filter(|finding| clean_path(&finding.primary_location.path) == clean_path(path))
            .map(|finding| diagnostic_json(finding, &self.sources))
            .collect::<Vec<_>>();
        diagnostics.sort_by(|left, right| {
            let key = |diagnostic: &Value| {
                (
                    diagnostic
                        .pointer("/range/start/line")
                        .and_then(Value::as_u64),
                    diagnostic
                        .pointer("/range/start/character")
                        .and_then(Value::as_u64),
                    diagnostic
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                )
            };
            key(left).cmp(&key(right))
        });
        diagnostics.dedup();
        diagnostics
    }

    fn publish_diagnostics(&mut self, force: bool) -> Result<(), LspError> {
        let mut paths = self
            .snapshot
            .findings
            .iter()
            .map(|finding| clean_path(&finding.primary_location.path))
            .collect::<BTreeSet<_>>();
        if !force {
            paths.extend(self.published.keys().cloned());
            paths.extend(self.documents.keys().map(|path| clean_path(path)));
        }
        for path in paths {
            let diagnostics = self.diagnostics_for_path(&path);
            let fingerprint = serde_json::to_string(&diagnostics)?;
            if !force
                && self
                    .published
                    .get(&path)
                    .is_some_and(|previous| previous == &fingerprint)
            {
                continue;
            }
            let mut params = json!({
                "uri": path_to_uri(&path),
                "diagnostics": diagnostics
            });
            if let Some(document) = self.documents.get(&path) {
                params["version"] = document.version.into();
            }
            self.notify("textDocument/publishDiagnostics", params)?;
            self.published.insert(path, fingerprint);
        }
        Ok(())
    }

    fn snapshot_json(&self) -> Value {
        serde_json::to_value(&self.snapshot).expect("snapshot serialization cannot fail")
    }

    fn code_actions(&self, uri: &str) -> Result<Vec<Value>, LspError> {
        let path = uri_to_path(uri)?;
        let mut actions = Vec::new();
        for finding in self
            .snapshot
            .findings
            .iter()
            .filter(|finding| clean_path(&finding.primary_location.path) == clean_path(&path))
        {
            for fix in &finding.fixes {
                let mut changes = serde_json::Map::new();
                for edit in &fix.edits {
                    let uri = path_to_uri(&edit.location.path);
                    let edits = changes.entry(uri).or_insert_with(|| json!([]));
                    edits
                        .as_array_mut()
                        .expect("workspace edits are arrays")
                        .push(json!({
                            "range": location_range(&edit.location, &self.sources),
                            "newText": edit.new_text
                        }));
                }
                actions.push(json!({
                    "title": fix.message,
                    "kind": "quickfix",
                    "isPreferred": fix.applicability == "safe",
                    "edit": {"changes": changes},
                    "data": {"findingId": finding.id}
                }));
            }
        }
        Ok(actions)
    }

    fn respond(
        &mut self,
        id: Value,
        result: Option<Value>,
        error: Option<(i64, String)>,
    ) -> Result<(), LspError> {
        let message = if let Some((code, message)) = error {
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
        } else {
            json!({"jsonrpc": "2.0", "id": id, "result": result.unwrap_or(Value::Null)})
        };
        write_frame(&mut self.writer, &message)
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        write_frame(
            &mut self.writer,
            &json!({"jsonrpc": "2.0", "method": method, "params": params}),
        )
    }
}

fn diagnostic_json(finding: &SnapshotFinding, sources: &HashMap<String, SourceFile>) -> Value {
    let severity = if finding.severity == "error" { 1 } else { 2 };
    let mut related_information = finding
        .related_locations
        .iter()
        .filter(|location| !location.path.contains("://"))
        .map(|location| {
            json!({
                "location": {
                    "uri": path_to_uri(&location.path),
                    "range": location_range(location, sources)
                },
                "message": format!("related evidence for {}", finding.id)
            })
        })
        .collect::<Vec<_>>();
    related_information.extend(
        finding
            .evidence
            .iter()
            .filter_map(|evidence| {
                evidence
                    .location
                    .as_ref()
                    .map(|location| (evidence, location))
            })
            .filter(|(_, location)| !location.path.contains("://"))
            .map(|(evidence, location)| {
                json!({
                    "location": {
                        "uri": path_to_uri(&location.path),
                        "range": location_range(location, sources)
                    },
                    "message": evidence.message
                })
            }),
    );
    json!({
        "range": location_range(&finding.primary_location, sources),
        "severity": severity,
        "code": finding.id,
        "source": "solid-check",
        "message": finding.message,
        "data": {
            "findingId": finding.id,
            "rule": finding.rule,
            "kind": finding.kind
        },
        "relatedInformation": related_information
    })
}

fn location_range(location: &SourceLocation, sources: &HashMap<String, SourceFile>) -> Value {
    let start = byte_position(&location.path, location.start_byte, sources);
    let end = byte_position(&location.path, location.end_byte, sources);
    json!({
        "start": {"line": start.0, "character": start.1},
        "end": {"line": end.0, "character": end.1}
    })
}

fn byte_position(
    path: &str,
    start_byte: u64,
    sources: &HashMap<String, SourceFile>,
) -> (usize, usize) {
    let Some(source) = sources.get(path).map(|source| source.source.as_str()) else {
        return (0, 0);
    };
    let mut offset = usize::try_from(start_byte)
        .unwrap_or(usize::MAX)
        .min(source.len());
    while !source.is_char_boundary(offset) {
        offset = offset.saturating_sub(1);
    }
    let prefix = &source[..offset];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    (
        prefix.bytes().filter(|byte| *byte == b'\n').count(),
        source[line_start..offset].encode_utf16().count(),
    )
}

fn read_frame(input: &mut impl BufRead) -> Result<Option<Vec<u8>>, LspError> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|error| LspError::Protocol(error.to_string()))?,
            );
        }
    }
    let length =
        content_length.ok_or_else(|| LspError::Protocol("missing Content-Length".into()))?;
    let mut payload = vec![0; length];
    input.read_exact(&mut payload)?;
    Ok(Some(payload))
}

fn write_frame(output: &mut impl Write, message: &Value) -> Result<(), LspError> {
    let payload = serde_json::to_vec(message)?;
    write!(output, "Content-Length: {}\r\n\r\n", payload.len())?;
    output.write_all(&payload)?;
    output.flush()?;
    Ok(())
}

fn string_field<'a>(value: &'a Value, name: &str) -> Result<&'a str, LspError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| LspError::InvalidParams(format!("missing {name}")))
}

fn integer_field(value: &Value, name: &str) -> Result<i64, LspError> {
    value
        .get(name)
        .and_then(Value::as_i64)
        .ok_or_else(|| LspError::InvalidParams(format!("missing {name}")))
}

fn clean_path(path: &str) -> String {
    let path = Path::new(path);
    if let Ok(canonical) = path.canonicalize() {
        return canonical.to_string_lossy().into_owned();
    }
    let mut cleaned = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                cleaned.pop();
            }
            component => cleaned.push(component.as_os_str()),
        }
    }
    cleaned.to_string_lossy().into_owned()
}

fn path_to_uri(path: &str) -> String {
    let mut result = String::from("file://");
    let path = clean_path(path);
    for byte in path.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(*byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            result.push(char::from(*byte));
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}

fn uri_to_path(uri: &str) -> Result<String, LspError> {
    if uri.contains(['?', '#']) {
        return Err(LspError::InvalidParams(
            "document file URI must not contain a query or fragment".into(),
        ));
    }
    let remainder = uri
        .strip_prefix("file://")
        .ok_or_else(|| LspError::InvalidParams("only file URIs are supported".into()))?;
    let encoded = if remainder.starts_with('/') {
        remainder
    } else {
        let (authority, path) = remainder
            .split_once('/')
            .ok_or_else(|| LspError::InvalidParams("file URI has no absolute path".into()))?;
        if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
            return Err(LspError::InvalidParams(format!(
                "unsupported file URI authority {authority:?}"
            )));
        }
        &remainder[remainder.len() - path.len() - 1..]
    };
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let value = bytes
                .get(index + 1..index + 3)
                .and_then(|digits| std::str::from_utf8(digits).ok())
                .and_then(|digits| u8::from_str_radix(digits, 16).ok())
                .ok_or_else(|| LspError::InvalidParams("invalid percent-encoded URI".into()))?;
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded)
        .map(|path| clean_path(&path))
        .map_err(|error| LspError::InvalidParams(error.to_string()))
}
