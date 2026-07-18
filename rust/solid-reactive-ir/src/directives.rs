//! Interprocedural discovery of reactive primitives created during directive
//! application.

use super::*;

pub(super) struct DirectiveCreationCollector<'a> {
    facts: &'a ProjectFacts,
    entities: &'a EntitySymbols,
    symbol_names: &'a HashMap<String, String>,
    visiting: HashSet<(String, Span)>,
    creations: &'a mut Vec<PrimitiveCreation>,
    seen: &'a mut HashSet<(String, u64, u64)>,
}

impl<'a> DirectiveCreationCollector<'a> {
    pub(super) fn new(
        facts: &'a ProjectFacts,
        entities: &'a EntitySymbols,
        symbol_names: &'a HashMap<String, String>,
        creations: &'a mut Vec<PrimitiveCreation>,
        seen: &'a mut HashSet<(String, u64, u64)>,
    ) -> Self {
        Self {
            facts,
            entities,
            symbol_names,
            visiting: HashSet::new(),
            creations,
            seen,
        }
    }

    pub(super) fn collect_returned(
        &mut self,
        file: &solid_facts::FileFacts,
        function: &solid_ast_facts::FunctionFact,
    ) {
        let key = (file.path.to_string(), function.span);
        if !self.visiting.insert(key.clone()) {
            return;
        }
        for returned in function
            .expression_return
            .iter()
            .chain(file.ast.returns.iter().filter(|returned| {
                containing_ast_function(&file.ast.functions, returned.span)
                    .is_some_and(|owner| owner.span == function.span)
            }))
        {
            match returned.value {
                solid_ast_facts::ReturnValueKind::Function => {
                    if let Some(returned_function) = file
                        .ast
                        .functions
                        .iter()
                        .find(|candidate| candidate.span == returned.span)
                    {
                        self.collect_function(file, returned_function);
                    }
                }
                solid_ast_facts::ReturnValueKind::Call => {
                    if let Some(callee) = returned.callee
                        && let Some((target_file, target)) =
                            function_called_at(self.facts, file, callee, self.entities)
                    {
                        self.collect_returned(target_file, target);
                    }
                }
                _ => {}
            }
        }
        self.visiting.remove(&key);
    }

    fn collect_function(
        &mut self,
        file: &solid_facts::FileFacts,
        function: &solid_ast_facts::FunctionFact,
    ) {
        let key = (file.path.to_string(), function.span);
        if !self.visiting.insert(key.clone()) {
            return;
        }
        for call in file.ast.calls.iter().filter(|call| {
            containing_ast_function(&file.ast.functions, call.span)
                .is_some_and(|owner| owner.span == function.span)
        }) {
            if let Some(primitive) = primitive_name(
                file.path.as_str(),
                call.callee,
                call.static_callee.as_deref(),
                self.entities,
                self.symbol_names,
            )
            .filter(|primitive| is_created_primitive(primitive))
            {
                push_directive_creation(
                    self.creations,
                    self.seen,
                    primitive,
                    file.path.as_str(),
                    call.callee,
                    true,
                );
            } else if let Some((target_file, target)) =
                function_called_at(self.facts, file, call.callee, self.entities)
            {
                self.collect_function(target_file, target);
            }
        }
        self.visiting.remove(&key);
    }
}

pub(super) fn is_created_primitive(primitive: &str) -> bool {
    matches!(
        primitive,
        "createSignal"
            | "createMemo"
            | "createStore"
            | "createProjection"
            | "createOptimistic"
            | "createOptimisticStore"
            | "createEffect"
            | "createRenderEffect"
            | "createTrackedEffect"
            | "createReaction"
            | "createRoot"
            | "createOwner"
    )
}

pub(super) fn push_directive_creation(
    creations: &mut Vec<PrimitiveCreation>,
    seen: &mut HashSet<(String, u64, u64)>,
    primitive: String,
    path: &str,
    span: Span,
    returned_closure: bool,
) {
    let location = location(path, span);
    if seen.insert((
        location.path.clone(),
        location.start_byte,
        location.end_byte,
    )) {
        creations.push(PrimitiveCreation {
            primitive,
            location,
            returned_closure,
        });
    }
}
