//! Rust-led orchestration of Oxc AST facts, Solid execution facts, and
//! TypeScript-Go semantic facts.

mod cache;
mod demand_plan;
mod diagnostics;
mod transport;

pub use cache::{CacheStats, FactsCache};
pub use diagnostics::{
    DiagnosticAnalysis, DiagnosticTimings, Metrics, PackageContractStatus, PackageSummary,
    Snapshot, SnapshotEvidence, SnapshotFinding, SnapshotFix, SnapshotTextEdit, SourceLocation,
    analysis_metrics, analyze_project, analyze_project_measured, analyze_project_measured_with,
    bundled_solid_js_contract, discovered_contract_paths, imported_package_roots,
    load_package_contracts, load_package_contracts_with, package_contract_statuses,
    package_contract_statuses_with, read_package_contract, snapshot, source_location,
};

#[must_use]
pub fn default_typefacts_executable() -> String {
    if let Some(value) = std::env::var_os("SOLID_TYPEFACTS_BIN")
        && !value.is_empty()
    {
        return value.to_string_lossy().into_owned();
    }
    let name = if cfg!(windows) {
        "solid-typefacts.exe"
    } else {
        "solid-typefacts"
    };
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        let sibling = directory.join(name);
        if sibling.is_file() {
            return sibling.to_string_lossy().into_owned();
        }
    }
    name.into()
}

use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, BufWriter, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use solid_compiler_facts::{AnalysisRequest, CompilerOptions, ExecutionMap, SidecarResponse};
use solid_facts::{FileFacts, ProjectFacts, TypeScriptChanges};
use solid_facts_core::{Generation, Span};
use solid_ts_facts::{ClosureRequest, ClosureResponse, FramedTransport};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub source: String,
    #[serde(default)]
    pub compiler_options: CompilerOptions,
}

pub trait CompilerFactsProvider {
    fn analyze(&mut self, request: &AnalysisRequest) -> Result<ExecutionMap, BackendError>;
}

pub struct SemanticDemandGroup<'a> {
    pub path: &'a str,
    pub demands: &'a [solid_ts_facts::v3::EntityDemand],
}

pub trait TypeFactsProvider {
    fn closure(&mut self, request: &ClosureRequest) -> Result<ClosureResponse, BackendError>;

    /// Whether the semantic path consumes compiler spans from `ClosureRequest`.
    ///
    /// Retained v3 sessions receive compiler structure through lifecycle
    /// updates and intentionally send no compiler spans with analyze requests.
    /// The v2/reference path still requires the canonical span table.
    fn semantic_requires_compiler_spans(&self) -> bool {
        true
    }

    fn semantic(
        &mut self,
        request: &ClosureRequest,
        _demands: Vec<solid_ts_facts::v3::EntityDemand>,
    ) -> Result<ClosureResponse, BackendError> {
        self.closure(request)
    }

    fn semantic_grouped(
        &mut self,
        request: &ClosureRequest,
        groups: &[SemanticDemandGroup<'_>],
    ) -> Result<ClosureResponse, BackendError> {
        self.semantic(
            request,
            groups
                .iter()
                .flat_map(|group| group.demands.iter().cloned())
                .collect(),
        )
    }

    fn semantic_response_requires_validation(&self) -> bool {
        true
    }

    fn validate_semantic_reuse(&mut self, _request: &ClosureRequest) -> Result<bool, BackendError> {
        Ok(false)
    }

    fn take_last_exchange_timings(&mut self) -> Option<TypeFactsExchangeTimings> {
        None
    }

    fn take_last_table_changes(&mut self) -> Option<TypeScriptChanges> {
        None
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("generation must be non-zero")]
    Generation,
    #[error("process error: {0}")]
    Process(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("AST facts error: {0}")]
    Ast(#[from] solid_ast_facts::AstFactsError),
    #[error("compiler facts error: {0}")]
    Compiler(#[from] solid_compiler_facts::CompilerFactsError),
    #[error("TypeFacts error: {0}")]
    TypeFacts(#[from] solid_ts_facts::TypeFactsError),
    #[error("fact join error: {0}")]
    Join(#[from] solid_facts::JoinError),
    #[error("compiler sidecar returned no execution map")]
    MissingExecutionMap,
    #[error("native Solid compiler facts error: {0}")]
    NativeCompiler(String),
    #[error("compiler sidecar {code}: {message}")]
    CompilerSidecar { code: String, message: String },
    #[error("TypeFacts service {code}: {message}")]
    TypeFactsService { code: String, message: String },
    #[error("reactive IR error: {0}")]
    ReactiveIr(#[from] solid_reactive_ir::BuildError),
    #[error("package contract error: {0}")]
    Contract(String),
    #[error("TypeFacts compatibility handshake failed: {0}")]
    Handshake(String),
    #[error("analysis cancelled")]
    Cancelled,
}

impl BackendError {
    #[must_use]
    pub fn is_typefacts_transport_failure(&self) -> bool {
        matches!(
            self,
            Self::Process(_) | Self::Io(_) | Self::TypeFacts(solid_ts_facts::TypeFactsError::Io(_))
        )
    }
}

#[derive(Default)]
pub struct NativeCompilerFacts;

impl CompilerFactsProvider for NativeCompilerFacts {
    fn analyze(&mut self, request: &AnalysisRequest) -> Result<ExecutionMap, BackendError> {
        use dom_expressions_jsx_compiler::{
            TransformOptions, analyze_execution_map, prelude::Either,
        };

        let effect_wrapper = request.compiler_options.effect_wrapper.clone().map(|name| {
            if name.is_empty() {
                Either::A(false)
            } else {
                Either::B(name)
            }
        });
        let options = TransformOptions {
            filename: Some(request.path.clone()),
            module_name: Some(request.compiler_options.module_name.clone()),
            generate: Some(request.compiler_options.generate.clone()),
            hydratable: Some(request.compiler_options.hydratable),
            dev: Some(request.compiler_options.dev),
            effect_wrapper,
            wrap_conditionals: request.compiler_options.wrap_conditionals,
            static_marker: request.compiler_options.static_marker.clone(),
            built_ins: Some(request.compiler_options.built_ins.clone()),
            compiler_facts: Some(true),
            ..TransformOptions::default()
        };
        let encoded = analyze_execution_map(&request.source, &options)
            .map_err(|error| BackendError::NativeCompiler(error.to_string()))?;
        ExecutionMap::from_json(&encoded, &request.source).map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub struct SourceChange {
    pub path: String,
    pub version: u64,
    pub source: Option<String>,
    pub compiler_options: CompilerOptions,
}

pub struct IncrementalSession {
    project_id: String,
    generation: u64,
    sources: HashMap<String, SourceFile>,
    cache: FactsCache,
    compiler: CompilerSidecar,
    typescript: TypeFactsSidecar,
}

const TYPEFACTS_RECOVERY_ATTEMPTS: u32 = 3;

/// Awaits an edit's pipelined update immediately before the analyze request
/// is written, so the analyze of the new generation is never sent ahead of
/// the update's acknowledgement, and a superseding edit observed at that
/// point skips the analyze entirely — the update half still lands.
struct PipelinedTypeFacts<'session> {
    sidecar: &'session mut TypeFactsSidecar,
    pending_update: Option<PendingLifecycle>,
    update_landed: bool,
    cancelled: Option<&'session std::sync::atomic::AtomicBool>,
}

impl PipelinedTypeFacts<'_> {
    fn await_update(&mut self) -> Result<(), BackendError> {
        if let Some(pending) = self.pending_update.take() {
            self.sidecar.lifecycle_wait(pending)?;
            self.update_landed = true;
            check_cancelled(self.cancelled)?;
        }
        Ok(())
    }
}

impl TypeFactsProvider for PipelinedTypeFacts<'_> {
    fn closure(&mut self, request: &ClosureRequest) -> Result<ClosureResponse, BackendError> {
        self.await_update()?;
        self.sidecar.closure(request)
    }

    fn semantic(
        &mut self,
        request: &ClosureRequest,
        demands: Vec<solid_ts_facts::v3::EntityDemand>,
    ) -> Result<ClosureResponse, BackendError> {
        self.await_update()?;
        self.sidecar.semantic(request, demands)
    }

    fn semantic_grouped(
        &mut self,
        request: &ClosureRequest,
        groups: &[SemanticDemandGroup<'_>],
    ) -> Result<ClosureResponse, BackendError> {
        self.await_update()?;
        self.sidecar.semantic_grouped(request, groups)
    }

    fn semantic_response_requires_validation(&self) -> bool {
        self.sidecar.semantic_response_requires_validation()
    }

    fn semantic_requires_compiler_spans(&self) -> bool {
        self.sidecar.semantic_requires_compiler_spans()
    }

    fn take_last_exchange_timings(&mut self) -> Option<TypeFactsExchangeTimings> {
        self.sidecar.take_last_exchange_timings()
    }

    fn take_last_table_changes(&mut self) -> Option<TypeScriptChanges> {
        self.sidecar.take_last_table_changes()
    }
}

/// A retained editor session with both Oxc and the Solid compiler running
/// in-process. TypeScript-Go is the only process boundary.
pub struct NativeIncrementalSession {
    project_id: String,
    generation: u64,
    sources: HashMap<String, SourceFile>,
    cache: FactsCache,
    last_facts: Option<Arc<ProjectFacts>>,
    typescript: TypeFactsSidecar,
    known_paths: HashSet<String>,
    last_build_timings: NativeBuildTimings,
}

impl NativeIncrementalSession {
    pub fn open(
        project_id: String,
        sources: Vec<SourceFile>,
        mut typescript: TypeFactsSidecar,
    ) -> Result<Self, BackendError> {
        open_typescript_project(&mut typescript, &project_id)?;
        Ok(Self::from_sources(project_id, sources, typescript))
    }

    /// Opens the project by pipelining `open` and `sources` (see
    /// [`TypeFactsSidecar::open_and_configured_sources`]) rather than fetching
    /// sources and opening in two sequential round trips. Returns the session
    /// together with the configured sources so callers can seed their own
    /// bookkeeping. The `open` is issued here, so callers must not open again.
    pub fn open_pipelined(
        project_id: String,
        mut typescript: TypeFactsSidecar,
    ) -> Result<(Self, Vec<SourceFile>), BackendError> {
        let sources = typescript.open_and_configured_sources(&project_id, 1)?;
        let session = Self::from_sources(project_id, sources.clone(), typescript);
        Ok((session, sources))
    }

    fn from_sources(
        project_id: String,
        sources: Vec<SourceFile>,
        typescript: TypeFactsSidecar,
    ) -> Self {
        Self {
            project_id,
            generation: 1,
            known_paths: sources.iter().map(|source| source.path.clone()).collect(),
            sources: sources
                .into_iter()
                .map(|source| (source.path.clone(), source))
                .collect(),
            cache: FactsCache::default(),
            last_facts: None,
            typescript,
            last_build_timings: NativeBuildTimings::default(),
        }
    }

    /// One edit exchange: the update that advances the generation always
    /// lands, and only the analyze half is cancellable. Local preparation
    /// (Oxc parse, native compiler facts, demand assembly) overlaps the
    /// update round trip; the analyze request is sent once the service has
    /// acknowledged the new generation. A transport failure restarts the
    /// service, replays local state, and retries exactly the half that
    /// failed.
    pub fn edit(
        &mut self,
        changes: Vec<SourceChange>,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Arc<ProjectFacts>, BackendError> {
        check_cancelled(cancelled)?;
        if changes.is_empty() {
            return self.analyze_with_recovery(cancelled);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BackendError::Generation)?;
        self.known_paths
            .extend(changes.iter().map(|change| change.path.clone()));
        let wire_changes = changes
            .iter()
            .map(|change| solid_ts_facts::v3::FileChange {
                path: change.path.clone(),
                version: change.version,
                source: change
                    .source
                    .as_deref()
                    .map_or_else(Vec::new, |source| source.as_bytes().to_vec()),
                deleted: change.source.is_none(),
            })
            .collect::<Vec<_>>();
        // Apply the overlay locally before anything is sent: demand assembly
        // reads these sources while the update round trip is in flight. The
        // displaced entries restore the overlay if the update never lands.
        let mut displaced = Vec::with_capacity(changes.len());
        for change in changes {
            displaced.push((change.path.clone(), self.sources.get(&change.path).cloned()));
            self.cache.invalidate_path(&change.path);
            if let Some(source) = change.source {
                self.sources.insert(
                    change.path.clone(),
                    SourceFile {
                        path: change.path,
                        source,
                        compiler_options: change.compiler_options,
                    },
                );
            } else {
                self.sources.remove(&change.path);
            }
        }
        let mut update_landed = false;
        let mut attempt = 0_u32;
        loop {
            let result = self.edit_attempt(
                next_generation,
                &wire_changes,
                &mut update_landed,
                cancelled,
            );
            if update_landed {
                self.generation = next_generation;
            }
            match result {
                Err(error)
                    if error.is_typefacts_transport_failure()
                        && attempt < TYPEFACTS_RECOVERY_ATTEMPTS =>
                {
                    std::thread::sleep(Duration::from_millis(25_u64 << attempt));
                    attempt += 1;
                    if let Err(recovery) = self.recover_typefacts() {
                        if !update_landed {
                            self.restore_overlay(displaced);
                        }
                        return Err(BackendError::Process(format!(
                            "{error}; recovery failed: {recovery}"
                        )));
                    }
                }
                Err(error) => {
                    if !update_landed {
                        self.restore_overlay(displaced);
                    }
                    return Err(error);
                }
                Ok(facts) => {
                    let facts = Arc::new(facts);
                    self.last_facts = Some(Arc::clone(&facts));
                    return Ok(facts);
                }
            }
        }
    }

    /// One attempt at the edit exchange. When the update has not landed yet,
    /// it is written first and awaited by the type-facts provider right
    /// before the analyze request goes out — or after the build returns, if
    /// the build never reached the semantic stage. A written update is
    /// always awaited: the service applies it regardless, so returning
    /// without committing the generation would desynchronize the session.
    fn edit_attempt(
        &mut self,
        next_generation: u64,
        wire_changes: &[solid_ts_facts::v3::FileChange],
        update_landed: &mut bool,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<ProjectFacts, BackendError> {
        let pending_update = if *update_landed {
            None
        } else {
            Some(
                self.typescript
                    .lifecycle_send(solid_ts_facts::v3::Request {
                        schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
                        request_id: 0,
                        operation: solid_ts_facts::v3::Operation::Update,
                        project_id: self.project_id.clone(),
                        generation: next_generation,
                        changes: wire_changes.to_vec(),
                        structural_spans: vec![],
                        compiler_spans: vec![],
                        demands: vec![],
                        compact_demands: None,
                        state_token: String::new(),
                        reset_state: false,
                        removed_demand_paths: vec![],
                        cancel_request_id: 0,
                    })?,
            )
        };
        let mut sources = self.sources.values().cloned().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.path.cmp(&right.path));
        let changed_paths = wire_changes
            .iter()
            .map(|change| change.path.clone())
            .collect::<HashSet<_>>();
        let retained = self
            .last_facts
            .as_deref()
            .filter(|facts| facts.generation.get() == self.generation)
            .map(|facts| RetainedFileFacts {
                previous: facts,
                changed_paths: &changed_paths,
            });
        let mut provider = PipelinedTypeFacts {
            sidecar: &mut self.typescript,
            pending_update,
            update_landed: false,
            cancelled,
        };
        let result = build_project_native_cached_measured_inner(
            self.project_id.clone(),
            next_generation,
            sources,
            &mut provider,
            &mut self.cache,
            cancelled,
            retained,
        );
        let leftover_update = provider.pending_update.take();
        if provider.update_landed {
            *update_landed = true;
        }
        if let Some(pending) = leftover_update {
            match self.typescript.lifecycle_wait(pending) {
                Ok(_) => *update_landed = true,
                // The build failed before the semantic stage and the update
                // also failed; the update failure is the root cause.
                Err(update_error) => return Err(update_error),
            }
        }
        result.map(|(facts, timings)| {
            self.last_build_timings = timings;
            facts
        })
    }

    fn restore_overlay(&mut self, displaced: Vec<(String, Option<SourceFile>)>) {
        for (path, previous) in displaced {
            self.cache.invalidate_path(&path);
            match previous {
                Some(source) => {
                    self.sources.insert(path, source);
                }
                None => {
                    self.sources.remove(&path);
                }
            }
        }
    }

    fn analyze_with_recovery(
        &mut self,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<Arc<ProjectFacts>, BackendError> {
        let mut attempt = 0_u32;
        loop {
            let result = match cancelled {
                Some(flag) => self.analyze_cancellable(flag),
                None => self.analyze(),
            };
            match result {
                Err(error)
                    if error.is_typefacts_transport_failure()
                        && attempt < TYPEFACTS_RECOVERY_ATTEMPTS =>
                {
                    std::thread::sleep(Duration::from_millis(25_u64 << attempt));
                    attempt += 1;
                    if let Err(recovery) = self.recover_typefacts() {
                        return Err(BackendError::Process(format!(
                            "{error}; recovery failed: {recovery}"
                        )));
                    }
                }
                other => return other,
            }
        }
    }

    pub fn analyze(&mut self) -> Result<Arc<ProjectFacts>, BackendError> {
        if let Some(facts) = self
            .last_facts
            .as_ref()
            .filter(|facts| facts.generation.get() == self.generation)
        {
            self.last_build_timings = NativeBuildTimings::default();
            return Ok(Arc::clone(facts));
        }
        let mut sources = self.sources.values().cloned().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.path.cmp(&right.path));
        let (facts, timings) = build_project_native_cached_measured(
            self.project_id.clone(),
            self.generation,
            sources,
            &mut self.typescript,
            &mut self.cache,
        )?;
        self.last_build_timings = timings;
        let facts = Arc::new(facts);
        self.last_facts = Some(Arc::clone(&facts));
        Ok(facts)
    }

    pub fn analyze_cancellable(
        &mut self,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<Arc<ProjectFacts>, BackendError> {
        check_cancelled(Some(cancelled))?;
        if let Some(facts) = self
            .last_facts
            .as_ref()
            .filter(|facts| facts.generation.get() == self.generation)
        {
            self.last_build_timings = NativeBuildTimings::default();
            return Ok(Arc::clone(facts));
        }
        let mut sources = self.sources.values().cloned().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.path.cmp(&right.path));
        let (facts, timings) = build_project_native_cached_measured_inner(
            self.project_id.clone(),
            self.generation,
            sources,
            &mut self.typescript,
            &mut self.cache,
            Some(cancelled),
            None,
        )?;
        self.last_build_timings = timings;
        let facts = Arc::new(facts);
        self.last_facts = Some(Arc::clone(&facts));
        Ok(facts)
    }

    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    #[must_use]
    pub const fn last_build_timings(&self) -> NativeBuildTimings {
        self.last_build_timings
    }

    #[must_use]
    pub fn cancellation_handle(&self) -> TypeFactsCancellation {
        self.typescript.cancellation_handle()
    }

    pub fn recover_typefacts(&mut self) -> Result<(), BackendError> {
        self.typescript.restart()?;
        open_typescript_project(&mut self.typescript, &self.project_id)?;
        if self.generation == 1 {
            return Ok(());
        }
        let mut changes = self
            .known_paths
            .iter()
            .map(|path| solid_ts_facts::v3::FileChange {
                path: path.clone(),
                version: self.generation,
                source: self
                    .sources
                    .get(path)
                    .map_or_else(Vec::new, |source| source.source.as_bytes().to_vec()),
                deleted: !self.sources.contains_key(path),
            })
            .collect::<Vec<_>>();
        changes.sort_by(|left, right| left.path.cmp(&right.path));
        for generation in 2..=self.generation {
            self.typescript.lifecycle(solid_ts_facts::v3::Request {
                schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
                request_id: 0,
                operation: solid_ts_facts::v3::Operation::Update,
                project_id: self.project_id.clone(),
                generation,
                changes: if generation == 2 {
                    changes.clone()
                } else {
                    vec![]
                },
                structural_spans: vec![],
                compiler_spans: vec![],
                demands: vec![],
                compact_demands: None,
                state_token: String::new(),
                reset_state: false,
                removed_demand_paths: vec![],
                cancel_request_id: 0,
            })?;
        }
        Ok(())
    }
}

impl IncrementalSession {
    pub fn open(
        project_id: String,
        sources: Vec<SourceFile>,
        compiler: CompilerSidecar,
        mut typescript: TypeFactsSidecar,
    ) -> Result<Self, BackendError> {
        open_typescript_project(&mut typescript, &project_id)?;
        Ok(Self {
            project_id,
            generation: 1,
            sources: sources
                .into_iter()
                .map(|source| (source.path.clone(), source))
                .collect(),
            cache: FactsCache::default(),
            compiler,
            typescript,
        })
    }

    pub fn update(&mut self, changes: Vec<SourceChange>) -> Result<Vec<String>, BackendError> {
        update_session(
            &self.project_id,
            &mut self.generation,
            &mut self.sources,
            &mut self.cache,
            &mut self.typescript,
            changes,
        )
    }

    pub fn analyze(&mut self) -> Result<ProjectFacts, BackendError> {
        let mut sources = self.sources.values().cloned().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.path.cmp(&right.path));
        build_project_cached(
            self.project_id.clone(),
            self.generation,
            sources,
            &mut self.compiler,
            &mut self.typescript,
            &mut self.cache,
        )
    }

    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

fn open_typescript_project(
    typescript: &mut TypeFactsSidecar,
    project_id: &str,
) -> Result<(), BackendError> {
    typescript.lifecycle(solid_ts_facts::v3::Request {
        schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
        request_id: 0,
        operation: solid_ts_facts::v3::Operation::Open,
        project_id: project_id.into(),
        generation: 1,
        changes: vec![],
        structural_spans: vec![],
        compiler_spans: vec![],
        demands: vec![],
        compact_demands: None,
        state_token: String::new(),
        reset_state: false,
        removed_demand_paths: vec![],
        cancel_request_id: 0,
    })?;
    Ok(())
}

fn update_session(
    project_id: &str,
    generation: &mut u64,
    sources: &mut HashMap<String, SourceFile>,
    cache: &mut FactsCache,
    typescript: &mut TypeFactsSidecar,
    changes: Vec<SourceChange>,
) -> Result<Vec<String>, BackendError> {
    let next_generation = generation.checked_add(1).ok_or(BackendError::Generation)?;
    let wire_changes = changes
        .iter()
        .map(|change| solid_ts_facts::v3::FileChange {
            path: change.path.clone(),
            version: change.version,
            source: change
                .source
                .as_deref()
                .map_or_else(Vec::new, |source| source.as_bytes().to_vec()),
            deleted: change.source.is_none(),
        })
        .collect();
    let response = typescript.lifecycle(solid_ts_facts::v3::Request {
        schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
        request_id: 0,
        operation: solid_ts_facts::v3::Operation::Update,
        project_id: project_id.into(),
        generation: next_generation,
        changes: wire_changes,
        structural_spans: vec![],
        compiler_spans: vec![],
        demands: vec![],
        compact_demands: None,
        state_token: String::new(),
        reset_state: false,
        removed_demand_paths: vec![],
        cancel_request_id: 0,
    })?;
    for change in changes {
        cache.invalidate_path(&change.path);
        if let Some(source) = change.source {
            sources.insert(
                change.path.clone(),
                SourceFile {
                    path: change.path,
                    source,
                    compiler_options: change.compiler_options,
                },
            );
        } else {
            sources.remove(&change.path);
        }
    }
    *generation = next_generation;
    Ok(response.affected)
}
pub fn build_project(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    compiler: &mut (impl CompilerFactsProvider + ?Sized),
    typescript: &mut impl TypeFactsProvider,
) -> Result<ProjectFacts, BackendError> {
    build_project_cached(
        project_id,
        generation,
        sources,
        compiler,
        typescript,
        &mut FactsCache::default(),
    )
}

pub fn build_project_native(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
) -> Result<ProjectFacts, BackendError> {
    build_project_native_measured(project_id, generation, sources, typescript)
        .map(|(facts, _)| facts)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NativeBuildTimings {
    pub source_analysis: Duration,
    pub source_files_reused: u64,
    pub source_files_recomputed: u64,
    pub ast_facts: Duration,
    pub compiler_facts: Duration,
    pub file_fact_assembly: Duration,
    pub type_facts: Duration,
    pub demand_assembly: Duration,
    pub request_assembly: Duration,
    pub semantic_demand_assembly: Duration,
    pub hydrate: Duration,
    pub join: Duration,
    pub exchange: TypeFactsExchangeTimings,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TypeFactsExchangeTimings {
    pub roundtrip: Duration,
    pub request_send: Duration,
    pub request_bytes: u64,
    pub response_decode: Duration,
    pub response_bytes: u64,
    pub server_request_decode: Duration,
    pub server_analyze: Duration,
    pub server_async: Duration,
    pub server_demand: Duration,
    pub server_assembly: Duration,
    pub server_sort: Duration,
    pub server_close_symbols: Duration,
    pub server_prepare: Duration,
    pub server_materialized: bool,
    pub server_retained_files: u64,
    pub server_recomputed_files: u64,
    pub server_non_durable_files: u64,
}

impl TypeFactsExchangeTimings {
    #[must_use]
    pub fn encode_and_transport(self) -> Duration {
        self.roundtrip.saturating_sub(
            self.request_send
                .saturating_add(self.response_decode)
                .saturating_add(self.server_request_decode)
                .saturating_add(self.server_analyze),
        )
    }
}

pub fn build_project_native_measured(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
) -> Result<(ProjectFacts, NativeBuildTimings), BackendError> {
    let project_id = project_id.into();
    let generation = Generation::new(generation).map_err(|_| BackendError::Generation)?;
    let source_files_recomputed = u64::try_from(sources.len()).unwrap_or(u64::MAX);
    let analysis_started = Instant::now();
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(sources.len().max(1));
    let chunk_size = sources.len().div_ceil(workers);
    let analyzed = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (chunk_index, chunk) in sources.chunks(chunk_size.max(1)).enumerate() {
            handles.push(scope.spawn(move || {
                let mut compiler = NativeCompilerFacts;
                chunk
                    .iter()
                    .enumerate()
                    .map(|(offset, file)| {
                        let ast = solid_ast_facts::extract(&file.path, &file.source)?;
                        let request = AnalysisRequest::new(
                            &file.path,
                            &file.source,
                            file.compiler_options.clone(),
                        );
                        let execution = compiler.analyze(&request)?;
                        execution.validate(&file.source)?;
                        Ok((
                            chunk_index * chunk_size + offset,
                            FileFacts::new(generation, &file.source, ast, execution)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, BackendError>>()
            }));
        }
        let mut analyzed = Vec::with_capacity(sources.len());
        for handle in handles {
            analyzed.extend(handle.join().expect("native facts worker panicked")?);
        }
        Ok::<_, BackendError>(analyzed)
    })?;
    let mut analyzed = analyzed;
    analyzed.sort_by_key(|(index, _)| *index);
    let mut files = Vec::with_capacity(analyzed.len());
    let mut seeds = Vec::new();
    for (_, file) in analyzed {
        seeds.extend(file.compiler_seed_locations()?);
        seeds.extend(file.structural_seed_locations());
        files.push(file);
    }
    let source_analysis = analysis_started.elapsed();
    let type_facts_started = Instant::now();
    let demand_started = Instant::now();
    let request_started = Instant::now();
    let request = ClosureRequest::new(project_id.clone(), generation, seeds)?;
    let request_assembly = request_started.elapsed();
    let semantic_demand_started = Instant::now();
    let demands = semantic_demands(&files)?;
    let semantic_demand_assembly = semantic_demand_started.elapsed();
    let demand_assembly = demand_started.elapsed();
    let response = typescript.semantic(&request, demands)?;
    let exchange = typescript.take_last_exchange_timings().unwrap_or_default();
    let table_changes = typescript.take_last_table_changes();
    if typescript.semantic_response_requires_validation() {
        response.validate_for(&request)?;
    }
    let type_facts = type_facts_started.elapsed();
    let mut table = response.table;
    let hydrate_started = Instant::now();
    hydrate_structural_file_facts(&mut table, &files);
    let hydrate = hydrate_started.elapsed();
    let join_started = Instant::now();
    let mut facts =
        ProjectFacts::join(generation, project_id, files, table).map_err(BackendError::from)?;
    facts.typescript_changes = table_changes;
    let join = join_started.elapsed();
    Ok((
        facts,
        NativeBuildTimings {
            source_analysis,
            source_files_reused: 0,
            source_files_recomputed,
            // This compatibility path intentionally fuses AST and compiler
            // extraction in the same parallel workers.
            ast_facts: source_analysis,
            compiler_facts: Duration::ZERO,
            file_fact_assembly: Duration::ZERO,
            type_facts,
            demand_assembly,
            request_assembly,
            semantic_demand_assembly,
            hydrate,
            join,
            exchange,
        },
    ))
}

pub fn build_project_native_cached(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
    cache: &mut FactsCache,
) -> Result<ProjectFacts, BackendError> {
    build_project_native_cached_measured(project_id, generation, sources, typescript, cache)
        .map(|(facts, _)| facts)
}

pub fn build_project_native_cached_measured(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
    cache: &mut FactsCache,
) -> Result<(ProjectFacts, NativeBuildTimings), BackendError> {
    build_project_native_cached_measured_inner(
        project_id, generation, sources, typescript, cache, None, None,
    )
}

pub fn build_project_native_cached_cancellable(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
    cache: &mut FactsCache,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<ProjectFacts, BackendError> {
    build_project_native_cached_measured_inner(
        project_id,
        generation,
        sources,
        typescript,
        cache,
        Some(cancelled),
        None,
    )
    .map(|(facts, _)| facts)
}

struct RetainedFileFacts<'a> {
    previous: &'a ProjectFacts,
    changed_paths: &'a HashSet<String>,
}

fn build_project_native_cached_measured_inner(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    typescript: &mut impl TypeFactsProvider,
    cache: &mut FactsCache,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    retained: Option<RetainedFileFacts<'_>>,
) -> Result<(ProjectFacts, NativeBuildTimings), BackendError> {
    let project_id = project_id.into();
    let generation = Generation::new(generation).map_err(|_| BackendError::Generation)?;
    check_cancelled(cancelled)?;
    let analysis_started = Instant::now();
    let mut files = std::iter::repeat_with(|| None)
        .take(sources.len())
        .collect::<Vec<Option<FileFacts>>>();
    let mut pending_indices = Vec::new();
    let mut pending_sources = Vec::new();
    let retained_by_path = retained.as_ref().map(|retained| {
        retained
            .previous
            .files
            .iter()
            .map(|file| (file.path.as_str(), file))
            .collect::<HashMap<_, _>>()
    });
    for (index, source) in sources.into_iter().enumerate() {
        let previous = retained_by_path
            .as_ref()
            .and_then(|files| files.get(source.path.as_str()).copied());
        if let Some(previous) = previous.filter(|_| {
            retained
                .as_ref()
                .is_some_and(|retained| !retained.changed_paths.contains(&source.path))
        }) {
            files[index] = Some(FileFacts {
                generation,
                path: previous.path.clone(),
                source_hash: previous.source_hash.clone(),
                source: source.source,
                ast: Arc::clone(&previous.ast),
                compiler: Arc::clone(&previous.compiler),
            });
        } else {
            pending_indices.push(index);
            pending_sources.push(source);
        }
    }
    let source_files_recomputed = u64::try_from(pending_sources.len()).unwrap_or(u64::MAX);
    let source_files_reused =
        u64::try_from(files.len().saturating_sub(pending_sources.len())).unwrap_or(u64::MAX);
    let ast_started = Instant::now();
    let prepared = prepare_ast_parallel(pending_sources, cache)?;
    let ast_facts = ast_started.elapsed();
    check_cancelled(cancelled)?;
    let compiler_started = Instant::now();
    let executions = prepare_native_compiler_parallel(&prepared, cache)?;
    let compiler_facts = compiler_started.elapsed();
    check_cancelled(cancelled)?;
    let assembly_started = Instant::now();
    for (((file, ast), execution), index) in
        prepared.into_iter().zip(executions).zip(pending_indices)
    {
        let facts = FileFacts::new(generation, &file.source, ast, execution)?;
        files[index] = Some(facts);
    }
    let files = files
        .into_iter()
        .map(|file| file.expect("every source was retained or recomputed"))
        .collect::<Vec<_>>();
    let mut seeds = Vec::new();
    if typescript.semantic_requires_compiler_spans() {
        for facts in &files {
            seeds.extend(facts.compiler_seed_locations()?);
            seeds.extend(facts.structural_seed_locations());
        }
    }
    let file_fact_assembly = assembly_started.elapsed();
    let source_analysis = analysis_started.elapsed();
    let type_facts_started = Instant::now();
    check_cancelled(cancelled)?;
    let cached_generation_matches = cache
        .semantic_table
        .as_ref()
        .is_some_and(|(cached_generation, _)| *cached_generation == generation.get());
    let reuse_request = ClosureRequest::new(project_id.clone(), generation, Vec::new())?;
    if cached_generation_matches && typescript.validate_semantic_reuse(&reuse_request)? {
        let exchange = typescript.take_last_exchange_timings().unwrap_or_default();
        check_cancelled(cancelled)?;
        let type_facts = type_facts_started.elapsed();
        let hydrate_started = Instant::now();
        let table = cache
            .semantic_table
            .as_ref()
            .expect("generation-matched semantic table")
            .1
            .clone();
        let hydrate = hydrate_started.elapsed();
        let join_started = Instant::now();
        let facts = ProjectFacts::join(generation, project_id, files, table)?;
        let join = join_started.elapsed();
        return Ok((
            facts,
            NativeBuildTimings {
                source_analysis,
                source_files_reused,
                source_files_recomputed,
                ast_facts,
                compiler_facts,
                file_fact_assembly,
                type_facts,
                demand_assembly: Duration::ZERO,
                request_assembly: Duration::ZERO,
                semantic_demand_assembly: Duration::ZERO,
                hydrate,
                join,
                exchange,
            },
        ));
    }
    let demand_started = Instant::now();
    let request_started = Instant::now();
    let request = ClosureRequest::new(project_id.clone(), generation, seeds)?;
    let request_assembly = request_started.elapsed();
    let semantic_demand_started = Instant::now();
    let demand_groups = semantic_demand_groups_cached(&files, cache)?;
    let semantic_demand_assembly = semantic_demand_started.elapsed();
    let demand_assembly = demand_started.elapsed();
    let response = typescript.semantic_grouped(&request, &demand_groups)?;
    let exchange = typescript.take_last_exchange_timings().unwrap_or_default();
    let table_changes = typescript.take_last_table_changes();
    check_cancelled(cancelled)?;
    if typescript.semantic_response_requires_validation() {
        response.validate_for(&request)?;
    }
    let type_facts = type_facts_started.elapsed();
    let mut table = response.table;
    let hydrate_started = Instant::now();
    hydrate_structural_file_facts_cached(&mut table, &files, cache);
    let hydrate = hydrate_started.elapsed();
    cache.semantic_table = Some((generation.get(), table.clone()));
    let join_started = Instant::now();
    let mut facts = ProjectFacts::join(generation, project_id, files, table)?;
    facts.typescript_changes = table_changes;
    let join = join_started.elapsed();
    Ok((
        facts,
        NativeBuildTimings {
            source_analysis,
            source_files_reused,
            source_files_recomputed,
            ast_facts,
            compiler_facts,
            file_fact_assembly,
            type_facts,
            demand_assembly,
            request_assembly,
            semantic_demand_assembly,
            hydrate,
            join,
            exchange,
        },
    ))
}

fn check_cancelled(cancelled: Option<&std::sync::atomic::AtomicBool>) -> Result<(), BackendError> {
    if cancelled.is_some_and(|cancelled| cancelled.load(std::sync::atomic::Ordering::Acquire)) {
        Err(BackendError::Cancelled)
    } else {
        Ok(())
    }
}

fn prepare_native_compiler_parallel(
    prepared: &[(SourceFile, Arc<solid_ast_facts::AstFacts>)],
    cache: &mut FactsCache,
) -> Result<Vec<Arc<ExecutionMap>>, BackendError> {
    let mut executions = vec![None; prepared.len()];
    let mut misses = Vec::new();
    for (index, (file, _)) in prepared.iter().enumerate() {
        let request = AnalysisRequest::new(&file.path, &file.source, file.compiler_options.clone());
        let key = compiler_cache_key(&request)?;
        if let Some(execution) = cache.compiler.get(&key) {
            executions[index] = Some(execution.clone());
        } else {
            misses.push((index, key, file));
        }
    }
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(misses.len().max(1));
    let chunk_size = misses.len().div_ceil(workers);
    let analyzed = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in misses.chunks(chunk_size.max(1)) {
            handles.push(scope.spawn(move || {
                let mut compiler = NativeCompilerFacts;
                chunk
                    .iter()
                    .map(|(index, key, file)| {
                        let request = AnalysisRequest::new(
                            &file.path,
                            &file.source,
                            file.compiler_options.clone(),
                        );
                        let execution = compiler.analyze(&request)?;
                        execution.validate(&file.source)?;
                        Ok((*index, key.clone(), execution))
                    })
                    .collect::<Result<Vec<_>, BackendError>>()
            }));
        }
        let mut analyzed = Vec::with_capacity(misses.len());
        for handle in handles {
            analyzed.extend(handle.join().expect("native compiler worker panicked")?);
        }
        Ok::<_, BackendError>(analyzed)
    })?;
    for (index, key, execution) in analyzed {
        let execution = Arc::new(execution);
        executions[index] = Some(Arc::clone(&execution));
        cache.compiler.insert(key, execution);
    }
    Ok(executions
        .into_iter()
        .map(|execution| execution.expect("every source was compiled or cached"))
        .collect())
}

pub fn build_project_cached(
    project_id: impl Into<String>,
    generation: u64,
    sources: Vec<SourceFile>,
    compiler: &mut (impl CompilerFactsProvider + ?Sized),
    typescript: &mut impl TypeFactsProvider,
    cache: &mut FactsCache,
) -> Result<ProjectFacts, BackendError> {
    let project_id = project_id.into();
    let generation = Generation::new(generation).map_err(|_| BackendError::Generation)?;
    let mut files = Vec::with_capacity(sources.len());
    let mut seeds = Vec::new();
    let prepared = prepare_ast_parallel(sources, cache)?;
    for (file, ast) in prepared {
        let request = AnalysisRequest::new(&file.path, &file.source, file.compiler_options);
        let compiler_key = compiler_cache_key(&request)?;
        let execution = if let Some(cached) = cache.compiler.get(&compiler_key) {
            cached.clone()
        } else {
            let execution = Arc::new(compiler.analyze(&request)?);
            cache.compiler.insert(compiler_key, Arc::clone(&execution));
            execution
        };
        execution.validate(&file.source)?;
        let facts = FileFacts::new(generation, &file.source, ast, execution)?;
        seeds.extend(facts.compiler_seed_locations()?);
        seeds.extend(facts.structural_seed_locations());
        files.push(facts);
    }
    let request = ClosureRequest::new(project_id.clone(), generation, seeds)?;
    let response = typescript.semantic(&request, semantic_demands_cached(&files, cache)?)?;
    if typescript.semantic_response_requires_validation() {
        response.validate_for(&request)?;
    }
    let mut table = response.table;
    hydrate_structural_file_facts_cached(&mut table, &files, cache);
    ProjectFacts::join(generation, project_id, files, table).map_err(Into::into)
}

fn semantic_demands(
    files: &[FileFacts],
) -> Result<Vec<solid_ts_facts::v3::EntityDemand>, BackendError> {
    demand_plan::plan(files)
}

fn structural_accessor_spans(file: &FileFacts) -> HashSet<Span> {
    let mut named_imports = HashMap::<&str, &str>::new();
    let mut namespace_imports = HashSet::<&str>::new();
    for import in &file.ast.imports {
        if !import.module.starts_with("solid-js") {
            continue;
        }
        for binding in &import.bindings {
            match binding.kind {
                solid_ast_facts::ImportKind::Named => {
                    let Some(local) = file.source_text(binding.local.span) else {
                        continue;
                    };
                    named_imports.insert(local, binding.imported.as_deref().unwrap_or(local));
                }
                solid_ast_facts::ImportKind::Namespace => {
                    if let Some(local) = file.source_text(binding.local.span) {
                        namespace_imports.insert(local);
                    }
                }
                _ => {}
            }
        }
    }
    let mut result = HashSet::new();
    for binding in &file.ast.bindings {
        let Some(initializer) = binding.call_initializer else {
            continue;
        };
        let Some(call) = file.ast.call_at(initializer) else {
            continue;
        };
        let Some(static_callee) = call.static_callee(&file.source) else {
            continue;
        };
        let primitive = if let Some(imported) = named_imports.get(static_callee) {
            Some(*imported)
        } else if let Some((namespace, property)) = static_callee.split_once('.')
            && namespace_imports.contains(namespace)
        {
            Some(property.rsplit('.').next().unwrap_or(property))
        } else {
            None
        };
        if !matches!(
            primitive,
            Some(
                "createSignal"
                    | "createMemo"
                    | "createStore"
                    | "createProjection"
                    | "createOptimistic"
                    | "createOptimisticStore"
            )
        ) {
            continue;
        }
        let source = if binding.shape == solid_ast_facts::BindingShape::Array {
            binding.array_slots.first().and_then(Option::as_ref)
        } else {
            binding.names.first()
        };
        if let Some(source) = source {
            result.insert(source.span);
        }
    }
    result
}

fn semantic_demands_cached(
    files: &[FileFacts],
    cache: &mut FactsCache,
) -> Result<Vec<solid_ts_facts::v3::EntityDemand>, BackendError> {
    let mut demands = Vec::new();
    let mut ordered_files = files.iter().collect::<Vec<_>>();
    ordered_files.sort_by(|left, right| left.path.cmp(&right.path));
    for file in ordered_files {
        let key = format!("{}\0{}", file.path, file.source_hash);
        let per_file = if let Some(cached) = cache.semantic_demands.get(&key) {
            cached
        } else {
            let generated = semantic_demands(std::slice::from_ref(file))?;
            cache.semantic_demands.insert(key.clone(), generated);
            cache
                .semantic_demands
                .get(&key)
                .expect("inserted semantic demand run")
        };
        demands.extend_from_slice(per_file);
    }
    Ok(demands)
}

fn semantic_demand_groups_cached<'a>(
    files: &'a [FileFacts],
    cache: &'a mut FactsCache,
) -> Result<Vec<SemanticDemandGroup<'a>>, BackendError> {
    let mut ordered_files = files.iter().collect::<Vec<_>>();
    ordered_files.sort_by(|left, right| left.path.cmp(&right.path));
    let keys = ordered_files
        .iter()
        .map(|file| format!("{}\0{}", file.path, file.source_hash))
        .collect::<Vec<_>>();
    for (file, key) in ordered_files.iter().zip(&keys) {
        if !cache.semantic_demands.contains_key(key) {
            cache
                .semantic_demands
                .insert(key.clone(), semantic_demands(std::slice::from_ref(*file))?);
        }
    }
    Ok(ordered_files
        .into_iter()
        .zip(keys)
        .map(|(file, key)| SemanticDemandGroup {
            path: file.path.as_str(),
            demands: cache
                .semantic_demands
                .get(&key)
                .expect("cached semantic demand run"),
        })
        .collect())
}

fn typefacts_location(path: &str, span: solid_facts_core::Span) -> solid_ts_facts::Location {
    solid_ts_facts::Location {
        path: path.into(),
        start_byte: u64::from(span.start),
        end_byte: u64::from(span.end),
    }
}

fn callee_property_location(
    source: &str,
    callee: &solid_ts_facts::Location,
) -> solid_ts_facts::Location {
    let Ok(start) = usize::try_from(callee.start_byte) else {
        return callee.clone();
    };
    let Ok(end) = usize::try_from(callee.end_byte) else {
        return callee.clone();
    };
    let Some(text) = source.as_bytes().get(start..end) else {
        return callee.clone();
    };
    let Some(dot) = text.iter().rposition(|byte| *byte == b'.') else {
        return callee.clone();
    };
    solid_ts_facts::Location {
        path: callee.path.clone(),
        start_byte: u64::try_from(start + dot + 1).unwrap_or(callee.start_byte),
        end_byte: callee.end_byte,
    }
}

fn hydrate_structural_file_facts(table: &mut solid_ts_facts::FactTable, files: &[FileFacts]) {
    let files_by_path = files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<HashMap<_, _>>();
    for target in Arc::make_mut(&mut table.files) {
        let Some(file) = files_by_path.get(target.path.as_str()).copied() else {
            continue;
        };
        target.functions = structural_functions(file);
    }
}

fn hydrate_structural_file_facts_cached(
    table: &mut solid_ts_facts::FactTable,
    files: &[FileFacts],
    cache: &mut FactsCache,
) {
    let files_by_path = files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<HashMap<_, _>>();
    for target in Arc::make_mut(&mut table.files) {
        let Some(file) = files_by_path.get(target.path.as_str()).copied() else {
            continue;
        };
        let key = format!("{}\0{}", file.path, file.source_hash);
        let functions = if let Some(cached) = cache.structural_functions.get(&key) {
            cached
        } else {
            cache
                .structural_functions
                .insert(key.clone(), structural_functions(file));
            cache
                .structural_functions
                .get(&key)
                .expect("inserted structural functions")
        };
        if target.functions != *functions {
            target.functions.clone_from(functions);
        }
    }
}

fn structural_functions(file: &FileFacts) -> Vec<solid_ts_facts::SourceFunction> {
    let mut result = Vec::new();
    for function in &file.ast.functions {
        let name = function.name.as_ref().or_else(|| {
            if function.kind != solid_ast_facts::FunctionKind::Arrow || function.expression_body {
                return None;
            }
            file.ast
                .bindings
                .iter()
                .filter(|binding| {
                    binding
                        .initializer
                        .is_some_and(|initializer| initializer.contains(function.span))
                })
                .min_by_key(|binding| {
                    binding
                        .initializer
                        .map_or(u32::MAX, |span| span.end - span.start)
                })
                .and_then(|binding| binding.names.first())
        });
        let Some(name) = name else {
            continue;
        };
        if function.kind == solid_ast_facts::FunctionKind::Expression
            || (function.kind == solid_ast_facts::FunctionKind::Arrow && function.expression_body)
        {
            continue;
        }
        let exported = file.ast.exports.iter().any(|export| {
            export.span.contains(function.span)
                && !file.ast.functions.iter().any(|candidate| {
                    candidate.span != function.span
                        && export.span.contains(candidate.span)
                        && candidate.span.contains(function.span)
                })
        });
        result.push(solid_ts_facts::SourceFunction {
            name: typefacts_location(file.path.as_str(), name.span),
            body: solid_ts_facts::Location {
                path: file.path.to_string(),
                start_byte: u64::from(function.body.start),
                // TS-Go reports a block body without the closing brace,
                // while Oxc's span includes it.
                end_byte: u64::from(function.body.end.saturating_sub(1)),
            },
            parameters: function
                .parameters
                .iter()
                .map(|parameter| typefacts_location(file.path.as_str(), parameter.pattern))
                .collect(),
            exported,
            r#async: function.r#async,
            arrow: function.kind == solid_ast_facts::FunctionKind::Arrow,
        });
    }
    result
}

fn prepare_ast_parallel(
    sources: Vec<SourceFile>,
    cache: &mut FactsCache,
) -> Result<Vec<(SourceFile, Arc<solid_ast_facts::AstFacts>)>, BackendError> {
    let mut misses = Vec::new();
    let mut prepared = vec![None; sources.len()];
    for (index, file) in sources.iter().enumerate() {
        let key = ast_cache_key(file);
        if let Some(ast) = cache.ast.get(&key) {
            prepared[index] = Some(ast.clone());
        } else {
            misses.push((index, key, file.path.as_str(), file.source.as_str()));
        }
    }
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(misses.len().max(1));
    let chunk_size = misses.len().div_ceil(workers);
    let parsed = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in misses.chunks(chunk_size.max(1)) {
            handles.push(scope.spawn(move || {
                chunk
                    .iter()
                    .map(|(index, key, path, source)| {
                        Ok((
                            *index,
                            key.clone(),
                            solid_ast_facts::extract(*path, source)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, solid_ast_facts::AstFactsError>>()
            }));
        }
        let mut parsed = Vec::new();
        for handle in handles {
            parsed.extend(handle.join().expect("Oxc worker panicked")?);
        }
        Ok::<_, solid_ast_facts::AstFactsError>(parsed)
    })?;
    for (index, key, ast) in parsed {
        let ast = Arc::new(ast);
        prepared[index] = Some(Arc::clone(&ast));
        cache.ast.insert(key, ast);
    }
    Ok(sources
        .into_iter()
        .zip(prepared)
        .map(|(file, ast)| (file, ast.expect("every source was parsed or cached")))
        .collect())
}

fn ast_cache_key(file: &SourceFile) -> String {
    format!(
        "{}\0{}",
        file.path,
        solid_facts_core::SourceHash::of(&file.source)
    )
}

fn compiler_cache_key(request: &AnalysisRequest) -> Result<String, BackendError> {
    Ok(format!(
        "{}\0{}\0{}",
        request.path,
        request.source_hash,
        serde_json::to_string(&request.compiler_options)?
    ))
}

pub struct CompilerSidecar {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl CompilerSidecar {
    pub fn spawn(executable: &str, args: &[String]) -> Result<Self, BackendError> {
        let mut child = Command::new(executable)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| BackendError::Process(error.to_string()))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| BackendError::Process("compiler stdin unavailable".into()))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| BackendError::Process("compiler stdout unavailable".into()))?;
        Ok(Self {
            child,
            input: BufWriter::new(input),
            output: BufReader::new(output),
        })
    }
}

impl CompilerFactsProvider for CompilerSidecar {
    fn analyze(&mut self, request: &AnalysisRequest) -> Result<ExecutionMap, BackendError> {
        serde_json::to_writer(&mut self.input, request)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            return Err(BackendError::Process(
                "compiler sidecar closed stdout".into(),
            ));
        }
        let response: SidecarResponse = serde_json::from_str(&line)?;
        if !response.ok {
            let error = response
                .error
                .ok_or_else(|| BackendError::Process("invalid compiler error response".into()))?;
            return Err(BackendError::CompilerSidecar {
                code: error.code,
                message: error.message,
            });
        }
        response
            .execution_map
            .ok_or(BackendError::MissingExecutionMap)
    }
}

impl Drop for CompilerSidecar {
    fn drop(&mut self) {
        let _ = self.input.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct TypeFactsSidecar {
    child: Child,
    writer: std::sync::Arc<std::sync::Mutex<BufWriter<ChildStdin>>>,
    pending: PendingResponses,
    next_request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
    active_request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
    project_id: String,
    executable: String,
    args: Vec<String>,
    reader: Option<std::thread::JoinHandle<()>>,
    last_exchange_timings: Option<TypeFactsExchangeTimings>,
    state_token: String,
    retained_demands: HashMap<String, Vec<solid_ts_facts::v3::EntityDemand>>,
    retained_table: Option<solid_ts_facts::FactTable>,
    last_table_changes: Option<TypeScriptChanges>,
}

// The reader thread decodes each frame once for routing and hands the decoded
// response to the waiting caller; re-decoding multi-megabyte fact tables at
// the call site was a measured double cost.
type PendingResponses = std::sync::Arc<
    std::sync::Mutex<
        HashMap<u64, std::sync::mpsc::SyncSender<Result<solid_ts_facts::v3::Response, String>>>,
    >,
>;

/// A lifecycle request that has been written but not yet awaited. Dropping it
/// abandons the response; the reader thread discards frames with no waiter.
pub struct PendingLifecycle {
    request_id: u64,
    cancellable: bool,
    sent_at: Instant,
    request_send: Duration,
    request_bytes: u64,
    receiver: std::sync::mpsc::Receiver<Result<solid_ts_facts::v3::Response, String>>,
}

#[derive(Clone)]
pub struct TypeFactsCancellation {
    writer: std::sync::Weak<std::sync::Mutex<BufWriter<ChildStdin>>>,
    next_request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
    active_request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
    project_id: String,
}

struct ProcessIo {
    input: ChildStdin,
    output: ChildStdout,
}

impl std::io::Read for ProcessIo {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.output.read(buffer)
    }
}

impl Write for ProcessIo {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.input.write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.input.flush()
    }
}

impl TypeFactsSidecar {
    pub fn spawn(executable: &str, args: &[String]) -> Result<Self, BackendError> {
        let mut child = Command::new(executable)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| BackendError::Process(error.to_string()))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| BackendError::Process("TypeFacts stdin unavailable".into()))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| BackendError::Process("TypeFacts stdout unavailable".into()))?;
        let transport = FramedTransport::new(ProcessIo { input, output });
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let handshake_thread = std::thread::spawn(move || {
            let mut transport = transport;
            let handshake = transport.receive::<solid_ts_facts::v3::Handshake>();
            let _ = sender.send((handshake, transport));
        });
        let (handshake, transport) = match receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(result) => {
                handshake_thread
                    .join()
                    .map_err(|_| BackendError::Handshake("startup reader panicked".into()))?;
                result
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = handshake_thread.join();
                return Err(BackendError::Handshake(
                    "service did not report compatibility within 5 seconds".into(),
                ));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = handshake_thread.join();
                return Err(BackendError::Handshake(
                    "startup reader disconnected".into(),
                ));
            }
        };
        let handshake = handshake
            .map_err(|error| BackendError::Handshake(format!("invalid startup frame: {error}")))?;
        let expected = (
            solid_ts_facts::v3::TYPE_FACTS_HANDSHAKE_PROTOCOL,
            solid_ts_facts::v3::TYPE_FACTS_SCHEMA_SHA256,
            solid_ts_facts::v3::TYPE_FACTS_BUILD_ID,
        );
        let actual = (
            handshake.protocol,
            handshake.schema_hash.as_str(),
            handshake.build_id.as_str(),
        );
        if actual != expected {
            let _ = child.kill();
            let _ = child.wait();
            return Err(BackendError::Handshake(format!(
                "expected protocol {}, schema {}, build {:?}; got protocol {}, schema {}, build {:?}",
                expected.0, expected.1, expected.2, actual.0, actual.1, actual.2
            )));
        }
        let ProcessIo { input, mut output } = transport.into_inner();
        let writer = std::sync::Arc::new(std::sync::Mutex::new(BufWriter::new(input)));
        let pending = PendingResponses::default();
        let reader_pending = std::sync::Arc::clone(&pending);
        let bad_frame_path = std::env::var_os("SOLID_TYPEFACTS_BAD_FRAME");
        let reader = std::thread::spawn(move || {
            loop {
                let payload = match transport::read_frame(&mut output) {
                    Ok(payload) => payload,
                    Err(error) => {
                        fail_pending_responses(&reader_pending, error.to_string());
                        break;
                    }
                };
                let decode_started = Instant::now();
                let response = match solid_ts_facts::decode_trusted::<solid_ts_facts::v3::Response>(
                    &payload,
                ) {
                    Ok(mut response) => {
                        response.client_decode_ns =
                            u64::try_from(decode_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                        response.client_response_bytes =
                            u64::try_from(payload.len()).unwrap_or(u64::MAX);
                        response
                    }
                    Err(error) => {
                        if let Some(path) = &bad_frame_path {
                            let _ = std::fs::write(path, &payload);
                        }
                        fail_pending_responses(&reader_pending, error.to_string());
                        break;
                    }
                };
                if let Ok(mut pending) = reader_pending.lock()
                    && let Some(sender) = pending.remove(&response.request_id)
                {
                    let _ = sender.send(Ok(response));
                }
            }
        });
        let next_request_id = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1));
        let active_request_id = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        Ok(Self {
            child,
            writer,
            pending,
            next_request_id,
            active_request_id,
            project_id: args
                .windows(2)
                .find(|arguments| arguments[0] == "-project")
                .map_or_else(String::new, |arguments| arguments[1].clone()),
            executable: executable.into(),
            args: args.to_vec(),
            reader: Some(reader),
            last_exchange_timings: None,
            state_token: String::new(),
            retained_demands: HashMap::new(),
            retained_table: None,
            last_table_changes: None,
        })
    }

    pub fn restart(&mut self) -> Result<(), BackendError> {
        let replacement = Self::spawn(&self.executable, &self.args)?;
        *self = replacement;
        Ok(())
    }

    #[must_use]
    pub fn cancellation_handle(&self) -> TypeFactsCancellation {
        TypeFactsCancellation {
            writer: std::sync::Arc::downgrade(&self.writer),
            next_request_id: std::sync::Arc::clone(&self.next_request_id),
            active_request_id: std::sync::Arc::clone(&self.active_request_id),
            project_id: self.project_id.clone(),
        }
    }

    pub fn lifecycle(
        &mut self,
        request: solid_ts_facts::v3::Request,
    ) -> Result<solid_ts_facts::v3::Response, BackendError> {
        let pending = self.lifecycle_send(request)?;
        self.lifecycle_wait(pending)
    }

    /// Writes a lifecycle request without awaiting its response, so a caller
    /// can overlap local work with the service round trip. The service
    /// processes generation-scoped requests in arrival order; responses are
    /// correlated by request identity and may be awaited in any order.
    pub fn lifecycle_send(
        &mut self,
        mut request: solid_ts_facts::v3::Request,
    ) -> Result<PendingLifecycle, BackendError> {
        request.request_id = self
            .next_request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let request_id = request.request_id;
        let sent_at = Instant::now();
        let cancellable = request.operation == solid_ts_facts::v3::Operation::Analyze;
        if cancellable {
            self.active_request_id
                .store(request_id, std::sync::atomic::Ordering::Release);
        }
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        self.pending
            .lock()
            .map_err(|_| BackendError::Process("TypeFacts pending map poisoned".into()))?
            .insert(request_id, sender);
        let request_bytes = match transport::write_frame(&self.writer, &request) {
            Ok(bytes) => bytes,
            Err(error) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&request_id);
                }
                if cancellable {
                    self.clear_active_request(request_id);
                }
                return Err(error);
            }
        };
        let request_send = sent_at.elapsed();
        Ok(PendingLifecycle {
            request_id,
            cancellable,
            sent_at,
            request_send,
            request_bytes: u64::try_from(request_bytes).unwrap_or(u64::MAX),
            receiver,
        })
    }

    pub fn lifecycle_wait(
        &self,
        pending: PendingLifecycle,
    ) -> Result<solid_ts_facts::v3::Response, BackendError> {
        let response = pending
            .receiver
            .recv()
            .map_err(|_| BackendError::Process("TypeFacts response channel closed".into()))
            .and_then(|received| received.map_err(BackendError::Process));
        if pending.cancellable {
            self.clear_active_request(pending.request_id);
        }
        let mut response = response?;
        response.client_roundtrip_ns =
            u64::try_from(pending.sent_at.elapsed().as_nanos()).unwrap_or(u64::MAX);
        response.client_request_send_ns =
            u64::try_from(pending.request_send.as_nanos()).unwrap_or(u64::MAX);
        response.client_request_bytes = pending.request_bytes;
        if response.request_id != pending.request_id {
            return Err(BackendError::Process(
                "TypeFacts response request identity mismatch".into(),
            ));
        }
        if !response.ok {
            let error = response.error.ok_or_else(|| {
                BackendError::Process("TypeFacts error response has no body".into())
            })?;
            return Err(BackendError::TypeFactsService {
                code: error.code,
                message: error.message,
            });
        }
        Ok(response)
    }

    fn clear_active_request(&self, request_id: u64) {
        self.active_request_id
            .compare_exchange(
                request_id,
                0,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .ok();
    }

    fn record_exchange_timings(&mut self, response: &solid_ts_facts::v3::Response) {
        let server = response.timings.unwrap_or_default();
        self.last_exchange_timings = Some(TypeFactsExchangeTimings {
            roundtrip: Duration::from_nanos(response.client_roundtrip_ns),
            request_send: Duration::from_nanos(response.client_request_send_ns),
            request_bytes: response.client_request_bytes,
            response_decode: Duration::from_nanos(response.client_decode_ns),
            response_bytes: response.client_response_bytes,
            server_request_decode: Duration::from_nanos(server.request_decode_ns),
            server_analyze: Duration::from_nanos(server.analyze_ns),
            server_async: Duration::from_nanos(server.r#async_ns),
            server_demand: Duration::from_nanos(server.demand_ns),
            server_assembly: Duration::from_nanos(server.assembly_ns),
            server_sort: Duration::from_nanos(server.sort_ns),
            server_close_symbols: Duration::from_nanos(server.close_symbols_ns),
            server_prepare: Duration::from_nanos(server.prepare_ns),
            server_materialized: server.materialized,
            server_retained_files: server.retained_files,
            server_recomputed_files: server.recomputed_files,
            server_non_durable_files: server.non_durable_files,
        });
    }

    pub fn configured_sources(
        &mut self,
        project_id: &str,
        generation: u64,
    ) -> Result<Vec<SourceFile>, BackendError> {
        let response = self.lifecycle(sources_request(project_id, generation))?;
        decode_source_files(response)
    }

    /// Pipelines the `open` and `sources` lifecycle requests: both frames are
    /// written before either response is awaited, so the service — which
    /// processes generation-scoped requests in arrival order after building
    /// the program — answers them back-to-back, collapsing two sequential
    /// cold-start round trips into one. Returns the configured sources; the
    /// open acknowledgement is validated and discarded.
    pub fn open_and_configured_sources(
        &mut self,
        project_id: &str,
        generation: u64,
    ) -> Result<Vec<SourceFile>, BackendError> {
        let pending_open = self.lifecycle_send(solid_ts_facts::v3::Request {
            schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
            request_id: 0,
            operation: solid_ts_facts::v3::Operation::Open,
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
        })?;
        let pending_sources = self.lifecycle_send(sources_request(project_id, generation))?;
        self.lifecycle_wait(pending_open)?;
        let response = self.lifecycle_wait(pending_sources)?;
        decode_source_files(response)
    }
}

fn sources_request(project_id: &str, generation: u64) -> solid_ts_facts::v3::Request {
    solid_ts_facts::v3::Request {
        schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
        request_id: 0,
        operation: solid_ts_facts::v3::Operation::Sources,
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

pub fn decode_source_files(
    response: solid_ts_facts::v3::Response,
) -> Result<Vec<SourceFile>, BackendError> {
    let arena = if response.source_arena.is_empty() {
        None
    } else {
        let bytes = std::fs::read(&response.source_arena).map_err(|error| {
            BackendError::Process(format!(
                "read TypeFacts source arena {:?}: {error}",
                response.source_arena
            ))
        })?;
        let _ = std::fs::remove_file(&response.source_arena);
        Some(bytes)
    };
    if let Some(arena) = arena.as_deref() {
        if response.source_lengths.len() != response.sources.len() {
            return Err(BackendError::Process(
                "TypeFacts source arena descriptor count mismatch".into(),
            ));
        }
        let mut offset = 0usize;
        let mut sources = Vec::with_capacity(response.sources.len());
        for (source, length) in response.sources.into_iter().zip(response.source_lengths) {
            let length = usize::try_from(length)
                .map_err(|_| BackendError::Process("source arena length overflow".into()))?;
            let end = offset
                .checked_add(length)
                .ok_or_else(|| BackendError::Process("source arena range overflow".into()))?;
            let bytes = arena.get(offset..end).ok_or_else(|| {
                BackendError::Process("source arena range is out of bounds".into())
            })?;
            offset = end;
            sources.push(decode_source_file(source, Some(bytes))?);
        }
        if offset != arena.len() {
            return Err(BackendError::Process(
                "TypeFacts source arena has trailing bytes".into(),
            ));
        }
        return Ok(sources);
    }
    response
        .sources
        .into_iter()
        .map(|source| decode_source_file(source, None))
        .collect()
}

fn decode_source_file(
    source: solid_ts_facts::v3::SourceFile,
    arena: Option<&[u8]>,
) -> Result<SourceFile, BackendError> {
    let bytes = if let Some(arena) = arena {
        arena.to_vec()
    } else if source.local {
        std::fs::read(&source.path).map_err(|error| {
            BackendError::Process(format!("read configured source {:?}: {error}", source.path))
        })?
    } else {
        source.source
    };
    Ok(SourceFile {
        path: source.path,
        source: String::from_utf8(bytes).map_err(|error| {
            BackendError::Process(format!("TypeFacts returned non-UTF-8 source: {error}"))
        })?,
        compiler_options: CompilerOptions::default(),
    })
}

impl TypeFactsSidecar {
    fn semantic_retained(
        &mut self,
        request: &ClosureRequest,
        demands: Vec<solid_ts_facts::v3::EntityDemand>,
        force_reset: bool,
    ) -> Result<ClosureResponse, BackendError> {
        let grouped = group_demands(&demands);
        let reset_state = force_reset || self.state_token.is_empty();
        let (wire_demands, removed_demand_paths) = if reset_state {
            (demands, Vec::new())
        } else {
            demand_delta(&self.retained_demands, &grouped)
        };
        let response = self.semantic_retained_exchange(
            request,
            wire_demands,
            removed_demand_paths,
            reset_state,
        )?;
        self.retained_demands = grouped;
        Ok(response)
    }

    fn semantic_retained_grouped(
        &mut self,
        request: &ClosureRequest,
        groups: &[SemanticDemandGroup<'_>],
        force_reset: bool,
    ) -> Result<ClosureResponse, BackendError> {
        let reset_state = force_reset || self.state_token.is_empty();
        let mut wire_demands = Vec::new();
        let mut present = HashSet::with_capacity(groups.len());
        for group in groups {
            present.insert(group.path);
            if reset_state
                || self
                    .retained_demands
                    .get(group.path)
                    .is_none_or(|previous| previous != group.demands)
            {
                wire_demands.extend_from_slice(group.demands);
            }
        }
        let mut removed_demand_paths = if reset_state {
            Vec::new()
        } else {
            self.retained_demands
                .keys()
                .filter(|path| !present.contains(path.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        };
        removed_demand_paths.sort();
        let response = self.semantic_retained_exchange(
            request,
            wire_demands,
            removed_demand_paths,
            reset_state,
        )?;
        if reset_state {
            self.retained_demands.clear();
        } else {
            self.retained_demands
                .retain(|path, _| present.contains(path.as_str()));
        }
        for group in groups {
            let changed = self
                .retained_demands
                .get(group.path)
                .is_none_or(|previous| previous != group.demands);
            if changed {
                self.retained_demands
                    .insert(group.path.to_owned(), group.demands.to_vec());
            }
        }
        Ok(response)
    }

    fn semantic_retained_exchange(
        &mut self,
        request: &ClosureRequest,
        wire_demands: Vec<solid_ts_facts::v3::EntityDemand>,
        removed_demand_paths: Vec<String>,
        reset_state: bool,
    ) -> Result<ClosureResponse, BackendError> {
        // Full reset snapshots go over compact; warm demand deltas stay plain.
        let (demands, compact_demands) = if reset_state && !wire_demands.is_empty() {
            (
                vec![],
                Some(solid_ts_facts::v3::compact_demands(&wire_demands)),
            )
        } else {
            (wire_demands, None)
        };
        let mut response = self.lifecycle(solid_ts_facts::v3::Request {
            schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
            request_id: 0,
            operation: solid_ts_facts::v3::Operation::Analyze,
            project_id: request.project_id.clone(),
            generation: request.generation,
            changes: vec![],
            structural_spans: vec![],
            compiler_spans: vec![],
            demands,
            compact_demands,
            state_token: if reset_state {
                String::new()
            } else {
                self.state_token.clone()
            },
            reset_state,
            removed_demand_paths,
            cancel_request_id: 0,
        })?;
        self.record_exchange_timings(&response);
        let reused = response.table_mode == "reuse";
        self.last_table_changes = Some(match response.table_mode.as_str() {
            "reuse" => TypeScriptChanges {
                unchanged: true,
                ..TypeScriptChanges::default()
            },
            "delta" => {
                let delta = response.table_delta.as_ref().ok_or_else(|| {
                    BackendError::Process("TypeFacts delta response has no delta".into())
                })?;
                let mut entity_paths = delta
                    .entity_files
                    .iter()
                    .map(|file| file.path.clone())
                    .chain(delta.removed_entity_paths.iter().cloned())
                    .collect::<Vec<_>>();
                let mut symbol_ids = delta
                    .symbols
                    .iter()
                    .map(|symbol| symbol.id.clone())
                    .chain(
                        delta
                            .symbol_reference_files
                            .iter()
                            .map(|references| references.id.clone()),
                    )
                    .chain(delta.removed_symbol_ids.iter().cloned())
                    .collect::<Vec<_>>();
                let mut file_paths = delta
                    .files
                    .iter()
                    .map(|file| file.path.clone())
                    .chain(delta.removed_file_paths.iter().cloned())
                    .collect::<Vec<_>>();
                entity_paths.sort();
                entity_paths.dedup();
                symbol_ids.sort();
                symbol_ids.dedup();
                file_paths.sort();
                file_paths.dedup();
                let unchanged =
                    entity_paths.is_empty() && symbol_ids.is_empty() && file_paths.is_empty();
                TypeScriptChanges {
                    unchanged,
                    entity_paths,
                    symbol_ids,
                    file_paths,
                }
            }
            _ => TypeScriptChanges::default(),
        });
        let table = match response.table_mode.as_str() {
            "full" => {
                if !response.packed_table.is_empty() {
                    solid_ts_facts::v3::decode_packed_fact_table(
                        &response.packed_table,
                        response.project_id.clone(),
                    )
                    .map_err(|error| {
                        BackendError::Process(format!("TypeFacts packed table invalid: {error}"))
                    })?
                } else {
                    match (response.table.take(), response.compact_table.take()) {
                        (Some(table), _) => table,
                        (None, Some(compact)) => compact.expand().map_err(|error| {
                            BackendError::Process(format!(
                                "TypeFacts compact table invalid: {error}"
                            ))
                        })?,
                        (None, None) => {
                            return Err(BackendError::Process(
                                "TypeFacts full response returned no table".into(),
                            ));
                        }
                    }
                }
            }
            "reuse" => {
                let mut table = self.retained_table.clone().ok_or_else(|| {
                    BackendError::Process("TypeFacts requested reuse without retained table".into())
                })?;
                table.generation = response.generation;
                table.project_id.clone_from(&response.project_id);
                table
            }
            "delta" => {
                let mut table = self.retained_table.clone().ok_or_else(|| {
                    BackendError::Process(
                        "TypeFacts returned a delta without retained table".into(),
                    )
                })?;
                apply_table_delta(
                    &mut table,
                    response.table_delta.as_ref().ok_or_else(|| {
                        BackendError::Process("TypeFacts delta response has no delta".into())
                    })?,
                )?;
                table
            }
            other => {
                return Err(BackendError::Process(format!(
                    "TypeFacts returned unsupported table mode {other:?}"
                )));
            }
        };
        let closure = ClosureResponse {
            schema: solid_ts_facts::TYPE_FACTS_SCHEMA,
            project_id: response.project_id,
            generation: response.generation,
            table,
        };
        closure.validate_for(request)?;
        if response.state_token.is_empty() {
            return Err(BackendError::Process(
                "TypeFacts retained response has no state token".into(),
            ));
        }
        self.state_token = response.state_token;
        if !reused {
            self.retained_table = Some(closure.table.clone());
        }
        Ok(closure)
    }
}

fn group_demands(
    demands: &[solid_ts_facts::v3::EntityDemand],
) -> HashMap<String, Vec<solid_ts_facts::v3::EntityDemand>> {
    let mut grouped: HashMap<String, Vec<_>> = HashMap::new();
    for demand in demands {
        grouped
            .entry(demand.location.path.clone())
            .or_default()
            .push(demand.clone());
    }
    grouped
}

fn demand_delta(
    previous: &HashMap<String, Vec<solid_ts_facts::v3::EntityDemand>>,
    next: &HashMap<String, Vec<solid_ts_facts::v3::EntityDemand>>,
) -> (Vec<solid_ts_facts::v3::EntityDemand>, Vec<String>) {
    let mut paths: Vec<_> = next.keys().collect();
    paths.sort();
    let mut changed = Vec::new();
    for path in paths {
        if previous.get(path) != next.get(path) {
            changed.extend(next[path].iter().cloned());
        }
    }
    let mut removed: Vec<_> = previous
        .keys()
        .filter(|path| !next.contains_key(*path))
        .cloned()
        .collect();
    removed.sort();
    (changed, removed)
}

fn apply_table_delta(
    table: &mut solid_ts_facts::FactTable,
    delta: &solid_ts_facts::v3::FactTableDelta,
) -> Result<(), BackendError> {
    let source_paths: HashSet<_> = delta
        .sources
        .iter()
        .map(|value| value.path.as_str())
        .collect();
    let removed_source_paths: HashSet<_> = delta
        .removed_source_paths
        .iter()
        .map(String::as_str)
        .collect();
    let sources = Arc::make_mut(&mut table.sources);
    sources.retain(|value| {
        !source_paths.contains(value.path.as_str())
            && !removed_source_paths.contains(value.path.as_str())
    });
    sources.extend(delta.sources.iter().cloned());
    sources.sort_by(|left, right| left.path.cmp(&right.path));

    let entity_paths: HashSet<_> = delta
        .entity_files
        .iter()
        .map(|value| value.path.as_str())
        .collect();
    let removed_entity_paths: HashSet<_> = delta
        .removed_entity_paths
        .iter()
        .map(String::as_str)
        .collect();
    let entities = Arc::make_mut(&mut table.entities);
    entities.retain(|value| {
        !entity_paths.contains(value.location.path.as_str())
            && !removed_entity_paths.contains(value.location.path.as_str())
    });
    for file in &delta.entity_files {
        entities.extend(file.entities.iter().cloned());
    }
    entities.sort_by(|left, right| {
        (
            &left.location.path,
            left.location.start_byte,
            left.location.end_byte,
        )
            .cmp(&(
                &right.location.path,
                right.location.start_byte,
                right.location.end_byte,
            ))
    });

    let symbol_ids: HashSet<_> = delta
        .symbols
        .iter()
        .map(|value| value.id.as_str())
        .collect();
    let removed_symbol_ids: HashSet<_> = delta
        .removed_symbol_ids
        .iter()
        .map(String::as_str)
        .collect();
    let symbols = Arc::make_mut(&mut table.symbols);
    symbols.retain(|value| {
        !symbol_ids.contains(value.id.as_str()) && !removed_symbol_ids.contains(value.id.as_str())
    });
    symbols.extend(delta.symbols.iter().cloned());
    symbols.sort_by(|left, right| left.id.cmp(&right.id));
    for replacement in &delta.symbol_reference_files {
        if replacement
            .references
            .iter()
            .any(|reference| reference.path != replacement.path)
        {
            return Err(BackendError::Process(format!(
                "TypeFacts reference delta for {:?} contains another path",
                replacement.path
            )));
        }
        if replacement.references.windows(2).any(|pair| {
            (pair[0].start_byte, pair[0].end_byte) > (pair[1].start_byte, pair[1].end_byte)
        }) {
            return Err(BackendError::Process(format!(
                "TypeFacts reference delta for {:?} is not ordered",
                replacement.id
            )));
        }
        let symbol_index = symbols
            .binary_search_by(|symbol| symbol.id.cmp(&replacement.id))
            .map_err(|_| {
                BackendError::Process(format!(
                    "TypeFacts reference delta names missing symbol {:?}",
                    replacement.id
                ))
            })?;
        let symbol = &mut symbols[symbol_index];
        let start = symbol
            .references
            .partition_point(|reference| reference.path < replacement.path);
        let end = symbol
            .references
            .partition_point(|reference| reference.path <= replacement.path);
        symbol
            .references
            .splice(start..end, replacement.references.iter().cloned());
    }

    let file_paths: HashSet<_> = delta
        .files
        .iter()
        .map(|value| value.path.as_str())
        .collect();
    let removed_file_paths: HashSet<_> = delta
        .removed_file_paths
        .iter()
        .map(String::as_str)
        .collect();
    let files = Arc::make_mut(&mut table.files);
    files.retain(|value| {
        !file_paths.contains(value.path.as_str())
            && !removed_file_paths.contains(value.path.as_str())
    });
    files.extend(delta.files.iter().cloned());
    files.sort_by(|left, right| left.path.cmp(&right.path));
    table.generation = delta.generation;
    Ok(())
}

impl TypeFactsProvider for TypeFactsSidecar {
    fn closure(&mut self, request: &ClosureRequest) -> Result<ClosureResponse, BackendError> {
        let response = self.lifecycle(solid_ts_facts::v3::Request {
            schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
            request_id: 0,
            operation: solid_ts_facts::v3::Operation::Analyze,
            project_id: request.project_id.clone(),
            generation: request.generation,
            changes: vec![],
            structural_spans: vec![],
            compiler_spans: request.compiler_spans.clone(),
            demands: vec![],
            compact_demands: None,
            state_token: String::new(),
            reset_state: false,
            removed_demand_paths: vec![],
            cancel_request_id: 0,
        })?;
        self.record_exchange_timings(&response);
        let closure = ClosureResponse {
            schema: solid_ts_facts::TYPE_FACTS_SCHEMA,
            project_id: response.project_id,
            generation: response.generation,
            table: response.table.ok_or_else(|| {
                BackendError::Process("TypeFacts analysis returned no table".into())
            })?,
        };
        closure.validate_for(request)?;
        Ok(closure)
    }

    fn semantic(
        &mut self,
        request: &ClosureRequest,
        demands: Vec<solid_ts_facts::v3::EntityDemand>,
    ) -> Result<ClosureResponse, BackendError> {
        if std::env::var_os("SOLID_TYPEFACTS_REFERENCE_V2").is_some() {
            return self.closure(request);
        }
        let retry_demands = demands.clone();
        match self.semantic_retained(request, demands, false) {
            Err(BackendError::TypeFactsService { code, .. }) if code == "state-mismatch" => {
                self.state_token.clear();
                self.retained_demands.clear();
                self.retained_table = None;
                self.semantic_retained(request, retry_demands, true)
            }
            result => result,
        }
    }

    fn semantic_requires_compiler_spans(&self) -> bool {
        std::env::var_os("SOLID_TYPEFACTS_REFERENCE_V2").is_some()
    }

    fn semantic_grouped(
        &mut self,
        request: &ClosureRequest,
        groups: &[SemanticDemandGroup<'_>],
    ) -> Result<ClosureResponse, BackendError> {
        if std::env::var_os("SOLID_TYPEFACTS_REFERENCE_V2").is_some() {
            return self.closure(request);
        }
        match self.semantic_retained_grouped(request, groups, false) {
            Err(BackendError::TypeFactsService { code, .. }) if code == "state-mismatch" => {
                self.state_token.clear();
                self.retained_demands.clear();
                self.retained_table = None;
                self.semantic_retained_grouped(request, groups, true)
            }
            result => result,
        }
    }

    fn semantic_response_requires_validation(&self) -> bool {
        false
    }

    fn validate_semantic_reuse(&mut self, request: &ClosureRequest) -> Result<bool, BackendError> {
        if self.state_token.is_empty() {
            return Ok(false);
        }
        let result = self.lifecycle(solid_ts_facts::v3::Request {
            schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
            request_id: 0,
            operation: solid_ts_facts::v3::Operation::Analyze,
            project_id: request.project_id.clone(),
            generation: request.generation,
            changes: vec![],
            structural_spans: vec![],
            compiler_spans: vec![],
            demands: vec![],
            compact_demands: None,
            state_token: self.state_token.clone(),
            reset_state: false,
            removed_demand_paths: vec![],
            cancel_request_id: 0,
        });
        match result {
            Ok(response) => {
                self.record_exchange_timings(&response);
                if response.table_mode == "reuse" && response.state_token == self.state_token {
                    Ok(true)
                } else {
                    self.state_token.clear();
                    self.retained_demands.clear();
                    self.retained_table = None;
                    Ok(false)
                }
            }
            Err(BackendError::TypeFactsService { code, .. }) if code == "state-mismatch" => {
                self.state_token.clear();
                self.retained_demands.clear();
                self.retained_table = None;
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    fn take_last_exchange_timings(&mut self) -> Option<TypeFactsExchangeTimings> {
        self.last_exchange_timings.take()
    }

    fn take_last_table_changes(&mut self) -> Option<TypeScriptChanges> {
        self.last_table_changes.take()
    }
}

impl Drop for TypeFactsSidecar {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl TypeFactsCancellation {
    pub fn cancel_active(&self) -> Result<bool, BackendError> {
        let target = self
            .active_request_id
            .load(std::sync::atomic::Ordering::Acquire);
        if target == 0 {
            return Ok(false);
        }
        let Some(writer) = self.writer.upgrade() else {
            return Ok(false);
        };
        let request_id = self
            .next_request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = transport::write_frame(
            &writer,
            &solid_ts_facts::v3::Request {
                schema: solid_ts_facts::v3::TYPE_FACTS_SCHEMA_V3,
                request_id,
                operation: solid_ts_facts::v3::Operation::Cancel,
                project_id: self.project_id.clone(),
                generation: 1,
                changes: vec![],
                structural_spans: vec![],
                compiler_spans: vec![],
                demands: vec![],
                compact_demands: None,
                state_token: String::new(),
                reset_state: false,
                removed_demand_paths: vec![],
                cancel_request_id: target,
            },
        )?;
        Ok(true)
    }
}

fn fail_pending_responses(pending: &PendingResponses, message: String) {
    if let Ok(mut pending) = pending.lock() {
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(message.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solid_compiler_facts::COMPILER_FACTS_PROTOCOL;
    use solid_facts_core::SourceHash;
    use solid_ts_facts::{FactTable, SourceDigest, TYPE_FACTS_SCHEMA};

    struct Compiler;
    impl CompilerFactsProvider for Compiler {
        fn analyze(&mut self, request: &AnalysisRequest) -> Result<ExecutionMap, BackendError> {
            Ok(ExecutionMap {
                compiler_facts_protocol: COMPILER_FACTS_PROTOCOL,
                source_hash: request.source_hash.clone(),
                tracked_regions: vec![],
                untracked_regions: vec![],
                ownership_regions: vec![],
                callback_roles: vec![],
                jsx_operations: vec![],
            })
        }
    }

    struct CountingCompiler(usize);
    impl CompilerFactsProvider for CountingCompiler {
        fn analyze(&mut self, request: &AnalysisRequest) -> Result<ExecutionMap, BackendError> {
            self.0 += 1;
            Compiler.analyze(request)
        }
    }

    struct Types {
        source: SourceDigest,
    }
    impl TypeFactsProvider for Types {
        fn closure(&mut self, request: &ClosureRequest) -> Result<ClosureResponse, BackendError> {
            Ok(ClosureResponse {
                schema: TYPE_FACTS_SCHEMA,
                project_id: request.project_id.clone(),
                generation: request.generation,
                table: FactTable {
                    schema: TYPE_FACTS_SCHEMA,
                    project_id: request.project_id.clone(),
                    generation: request.generation,
                    sources: vec![self.source.clone()].into(),
                    entities: vec![].into(),
                    symbols: vec![].into(),
                    files: vec![].into(),
                },
            })
        }
    }

    fn test_file_facts(path: &str, source: &str) -> FileFacts {
        let ast = solid_ast_facts::extract(path, source).unwrap();
        FileFacts::new(
            Generation::new(1).unwrap(),
            source,
            ast,
            ExecutionMap {
                compiler_facts_protocol: COMPILER_FACTS_PROTOCOL,
                source_hash: SourceHash::of(source),
                tracked_regions: vec![],
                untracked_regions: vec![],
                ownership_regions: vec![],
                callback_roles: vec![],
                jsx_operations: vec![],
            },
        )
        .unwrap()
    }

    #[test]
    fn hydrates_local_sources_and_preserves_inline_fallbacks() {
        let path = std::env::temp_dir().join(format!(
            "solid-checker-source-hydration-{}.ts",
            std::process::id()
        ));
        std::fs::write(&path, "export const local = 1;\n").unwrap();
        let local = decode_source_file(
            solid_ts_facts::v3::SourceFile {
                path: path.to_string_lossy().into_owned(),
                local: true,
                source: vec![],
            },
            None,
        )
        .unwrap();
        assert_eq!(local.source, "export const local = 1;\n");
        std::fs::remove_file(path).unwrap();

        let inline = decode_source_file(
            solid_ts_facts::v3::SourceFile {
                path: "/virtual/generated.ts".into(),
                local: false,
                source: b"export const generated = 2;\n".to_vec(),
            },
            None,
        )
        .unwrap();
        assert_eq!(inline.source, "export const generated = 2;\n");
    }

    #[test]
    fn retained_demands_and_indexed_hydration_match_fresh_results() {
        let files = vec![
            test_file_facts(
                "src/b.tsx",
                "export const B = () => <div>{createSignal(1)}</div>;",
            ),
            test_file_facts(
                "src/a.ts",
                "export function A(value: number) { return value; }",
            ),
        ];
        let fresh_demands = semantic_demands(&files).unwrap();
        let mut cache = FactsCache::default();
        let retained_demands = semantic_demands_cached(&files, &mut cache).unwrap();
        assert_eq!(retained_demands, fresh_demands);

        let mut fresh_table = FactTable {
            schema: TYPE_FACTS_SCHEMA,
            project_id: "project".into(),
            generation: 1,
            sources: vec![].into(),
            entities: vec![].into(),
            symbols: vec![].into(),
            files: files
                .iter()
                .rev()
                .map(|file| solid_ts_facts::FileFact {
                    path: file.path.to_string(),
                    calls: vec![],
                    bindings: vec![],
                    functions: vec![],
                    async_functions: vec![],
                })
                .collect::<Vec<_>>()
                .into(),
        };
        let mut retained_table = fresh_table.clone();
        hydrate_structural_file_facts(&mut fresh_table, &files);
        hydrate_structural_file_facts_cached(&mut retained_table, &files, &mut cache);
        assert_eq!(retained_table, fresh_table);
    }

    #[test]
    fn semantic_demand_plan_is_complete_for_downstream_consumers() {
        let file = test_file_facts(
            "src/component.tsx",
            "const value = createMemo(async () => 1); export function Card(props: { title: string }) { const key = 'title'; const copy = { ...props }; return <div>{props[key]}{copy.title}{value()}</div>; }",
        );
        let demands = semantic_demands(std::slice::from_ref(&file)).unwrap();

        for member in &file.ast.members {
            let location = typefacts_location(file.path.as_str(), member.object);
            assert!(
                demands
                    .iter()
                    .any(|demand| demand.symbol && demand.location == location),
                "member object {location:?} must retain symbol provenance"
            );
        }
        for spread in &file.ast.spreads {
            let location = typefacts_location(file.path.as_str(), spread.argument);
            assert!(
                demands
                    .iter()
                    .any(|demand| demand.symbol && demand.location == location),
                "spread argument {location:?} must retain symbol provenance"
            );
        }
        for call in &file.ast.calls {
            let location = typefacts_location(file.path.as_str(), call.callee);
            let demand = demands
                .iter()
                .find(|demand| demand.location == location && demand.query_location.is_some())
                .expect("every call callee needs a symbol/type query");
            assert!(demand.symbol);
            assert_eq!(demand.type_descriptor, call.arguments.is_empty());
        }
        assert!(
            demands
                .iter()
                .any(|demand| { demand.r#async && demand.location.path == file.path.as_str() })
        );
        assert!(
            demands.windows(2).all(|pair| pair[0] != pair[1]),
            "the transport plan must not contain duplicate queries"
        );
        let mut reversed = vec![
            file.clone(),
            test_file_facts("src/a.ts", "export const a = 1;"),
        ];
        let planned = semantic_demands(&reversed).unwrap();
        reversed.reverse();
        assert_eq!(
            planned,
            semantic_demands(&reversed).unwrap(),
            "query order must not depend on source traversal order"
        );
    }

    #[test]
    fn joins_all_three_fact_sources() {
        let source = "export const answer = 42;";
        let mut compiler = Compiler;
        let mut types = Types {
            source: SourceDigest {
                path: "src/a.ts".into(),
                sha256: SourceHash::of(source),
            },
        };
        let project = build_project(
            "project",
            1,
            vec![SourceFile {
                path: "src/a.ts".into(),
                source: source.into(),
                compiler_options: CompilerOptions::default(),
            }],
            &mut compiler,
            &mut types,
        )
        .unwrap();
        assert_eq!(project.files.len(), 1);
        assert_eq!(project.typescript.sources.as_slice(), &[types.source]);
    }

    #[test]
    fn reuses_ast_and_compiler_facts_by_source_identity() {
        let source = "export const answer = 42;";
        let input = SourceFile {
            path: "src/a.ts".into(),
            source: source.into(),
            compiler_options: CompilerOptions::default(),
        };
        let mut compiler = CountingCompiler(0);
        let mut types = Types {
            source: SourceDigest {
                path: "src/a.ts".into(),
                sha256: SourceHash::of(source),
            },
        };
        let mut cache = FactsCache::default();
        let projects = [1, 2].map(|generation| {
            build_project_cached(
                "project",
                generation,
                vec![input.clone()],
                &mut compiler,
                &mut types,
                &mut cache,
            )
            .unwrap()
        });
        assert_eq!(compiler.0, 1);
        assert!(Arc::ptr_eq(
            &projects[0].files[0].ast,
            &projects[1].files[0].ast
        ));
        assert!(Arc::ptr_eq(
            &projects[0].files[0].compiler,
            &projects[1].files[0].compiler
        ));
        assert_eq!(
            cache.stats(),
            CacheStats {
                ast_entries: 1,
                compiler_entries: 1
            }
        );
    }
}
