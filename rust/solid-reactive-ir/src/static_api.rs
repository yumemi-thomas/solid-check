//! Static validation for Solid 2 API shapes and explicit refresh writes.

use super::*;

pub(super) struct StaticDirectiveFileResult {
    pub(super) violations: Vec<StaticViolation>,
    pub(super) writes: Vec<ReactiveWrite>,
    pub(super) write_action_obligations: Vec<(&'static str, String, u64, u64)>,
}

pub(super) struct StaticApiContext<'a> {
    pub(super) lookup: &'a SemanticLookup<'a>,
    pub(super) entities: &'a EntitySymbols,
    pub(super) symbol_names: &'a HashMap<String, String>,
    pub(super) source_kinds: &'a HashMap<String, ReactiveSourceKind>,
    pub(super) source_owned_write: &'a HashMap<String, bool>,
    pub(super) accessors: &'a HashMap<String, (String, Location)>,
    pub(super) reachable_calls: &'a HashMap<Location, usize>,
}

impl StaticApiContext<'_> {
    pub(super) fn check_file(&self, file: &FileFacts) -> StaticDirectiveFileResult {
        let mut result = StaticDirectiveFileResult {
            violations: Vec::new(),
            writes: Vec::new(),
            write_action_obligations: Vec::new(),
        };
        let allowed = allowed_callback_spans(file, self.entities, self.symbol_names);
        for call in &file.ast.calls {
            let Some(primitive) = primitive_name(
                file.path.as_str(),
                call.callee,
                call.static_callee.as_deref(),
                self.entities,
                self.symbol_names,
            ) else {
                continue;
            };
            if primitive == "createEffect"
                && (call.arguments.len() < 2
                    || call.arguments[1].value == solid_ast_facts::ArgumentValueKind::Undefined)
            {
                result.violations.push(StaticViolation {
                    id: "SC7001".into(),
                    rule: "missing-effect-function".into(),
                    message: "createEffect requires both a compute function and an effect function"
                        .into(),
                    location: location(file.path.as_str(), call.callee),
                    analysis_context: String::new(),
                    fixes: vec![],
                });
            }
            let options_index = match primitive.as_str() {
                "createMemo" | "createSignal" | "createOptimistic" => Some(1),
                "createStore"
                | "createProjection"
                | "createOptimisticStore"
                | "createEffect"
                | "createRenderEffect" => Some(2),
                "createTrackedEffect" => Some(1),
                _ => None,
            };
            if let Some(options_index) = options_index
                && call.arguments.get(options_index).is_some_and(|argument| {
                    argument
                        .boolean_properties
                        .iter()
                        .any(|property| property.name == "sync" && property.value)
                })
                && call
                    .arguments
                    .first()
                    .is_some_and(|argument| computation_is_async(self.lookup, file, argument.span))
            {
                result.violations.push(StaticViolation {
                    id: "SC7002".into(),
                    rule: "sync-node-received-async".into(),
                    message: format!(
                        "{primitive} uses sync: true but its computation can return a Promise or AsyncIterable"
                    ),
                    location: location(file.path.as_str(), call.callee),
                    analysis_context: String::new(),
                    fixes: vec![],
                });
            }
            if !matches!(primitive.as_str(), "refresh" | "affects") {
                continue;
            }
            let invalid_arity = call.arguments.is_empty()
                || primitive == "refresh" && call.arguments.len() != 1
                || primitive == "affects" && call.arguments.len() > 2;
            if invalid_arity {
                result.violations.push(StaticViolation {
                    id: "SC7003".into(),
                    rule: format!("invalid-{primitive}-target"),
                    message: format!(
                        "{primitive}() received an invalid number of target arguments"
                    ),
                    location: location(file.path.as_str(), call.callee),
                    analysis_context: String::new(),
                    fixes: vec![],
                });
                continue;
            }
            let target = &call.arguments[0];
            if target.value != solid_ast_facts::ArgumentValueKind::Identifier {
                result.violations.push(StaticViolation {
                    id: "SC7003".into(),
                    rule: format!("invalid-{primitive}-target"),
                    message: format!(
                        "{primitive}() expects the original Solid source accessor or store, not a wrapper, read value, or literal"
                    ),
                    location: location(file.path.as_str(), target.span),
                    analysis_context: String::new(),
                    fixes: vec![],
                });
                continue;
            }
            let target_location = location(file.path.as_str(), target.span);
            let Some(symbol) = self.entities.get(&target_location) else {
                result.violations.push(StaticViolation {
                    id: "SC9003".into(),
                    rule: format!("{primitive}-target-unresolved"),
                    message: format!(
                        "cannot prove that the target is a branded Solid source accepted by {primitive}"
                    ),
                    location: target_location,
                    analysis_context: String::new(),
                    fixes: vec![],
                });
                continue;
            };
            let Some(kind) = self.source_kinds.get(symbol).copied() else {
                result.violations.push(StaticViolation {
                    id: "SC9003".into(),
                    rule: format!("{primitive}-target-unresolved"),
                    message: format!(
                        "cannot prove that the target is a branded Solid source accepted by {primitive}"
                    ),
                    location: target_location,
                    analysis_context: String::new(),
                    fixes: vec![],
                });
                continue;
            };
            if primitive == "affects" {
                if kind == ReactiveSourceKind::Accessor && call.arguments.len() == 2 {
                    result.violations.push(StaticViolation {
                        id: "SC7004".into(),
                        rule: "affects-keys-on-accessor".into(),
                        message: "affects() keys are only valid on store targets".into(),
                        location: location(file.path.as_str(), call.callee),
                        analysis_context: String::new(),
                        fixes: vec![],
                    });
                }
                continue;
            }
            if file
                .ast
                .functions
                .iter()
                .any(|function| function.body.contains(call.span))
            {
                result.write_action_obligations.push((
                    "write",
                    file.path.to_string(),
                    u64::from(call.callee.start),
                    u64::from(call.callee.end),
                ));
            }
            let callee = location(file.path.as_str(), call.callee);
            let Some(multiplicity) = self.reachable_calls.get(&callee).copied() else {
                continue;
            };
            let Some((name, declaration)) = self.accessors.get(symbol) else {
                continue;
            };
            for _ in 0..multiplicity {
                result.writes.push(ReactiveWrite {
                    setter: format!("refresh({name})"),
                    location: location(
                        file.path.as_str(),
                        Span::new(
                            call.span.start,
                            call.arguments
                                .last()
                                .map_or(call.span.end, |argument| argument.span.end),
                        ),
                    ),
                    declaration: declaration.clone(),
                    execution: semantic_execution_role(
                        file,
                        call.callee,
                        &allowed,
                        self.entities,
                        self.symbol_names,
                    ),
                    allowed_by_option: self
                        .source_owned_write
                        .get(symbol)
                        .copied()
                        .unwrap_or(false),
                    context: analysis_context(file, call.span, self.entities, self.symbol_names),
                });
            }
        }
        result
    }
}
