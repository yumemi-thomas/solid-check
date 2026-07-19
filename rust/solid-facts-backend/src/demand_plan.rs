//! Plans the smallest complete TypeFacts query set required by downstream
//! analysis. Keeping this policy separate from transport orchestration makes
//! omissions testable before they become missing diagnostics.

use std::collections::HashMap;

use solid_facts::FileFacts;
use solid_ts_facts::v3::EntityDemand;

use crate::{
    BackendError, callee_property_location, structural_accessor_spans, typefacts_location,
};

pub(crate) fn plan(files: &[FileFacts]) -> Result<Vec<EntityDemand>, BackendError> {
    let mut demands = Vec::new();
    for file in files {
        plan_file(file, &mut demands)?;
    }
    stable_deduplicate(&mut demands);
    Ok(demands)
}

fn plan_file(file: &FileFacts, demands: &mut Vec<EntityDemand>) -> Result<(), BackendError> {
    let path = file.path.to_string();
    let structural_accessors = structural_accessor_spans(file);
    let mut symbol_spans = HashMap::new();
    let mut add_symbol = |span, references| {
        symbol_spans
            .entry(span)
            .and_modify(|current| *current |= references)
            .or_insert(references);
    };

    for import in &file.ast.imports {
        for binding in import
            .bindings
            .iter()
            .filter(|binding| !binding.local.name.is_empty())
        {
            add_symbol(binding.local.span, true);
        }
    }
    for binding in &file.ast.bindings {
        for name in &binding.names {
            add_symbol(name.span, true);
        }
        if let Some(initializer) = &binding.initializer_identifier {
            add_symbol(initializer.span, true);
        }
    }
    for function in &file.ast.functions {
        if let Some(name) = &function.name {
            add_symbol(name.span, true);
        }
        for name in function
            .parameters
            .iter()
            .flat_map(|parameter| &parameter.names)
        {
            add_symbol(name.span, true);
        }
    }
    for export in &file.ast.exports {
        for item in export.specifiers.iter().chain(&export.declarations) {
            add_symbol(item.local.span, true);
        }
    }
    for element in &file.ast.jsx_elements {
        add_symbol(element.name.span, false);
    }
    for returned in &file.ast.returns {
        if let Some(callee) = returned.callee {
            demands.push(demand(typefacts_location(&path, callee)).resolved_call());
        }
        if returned.value == solid_ast_facts::ReturnValueKind::Identifier {
            add_symbol(returned.span, false);
        }
    }
    for call in &file.ast.calls {
        for argument in &call.arguments {
            if argument.value == solid_ast_facts::ArgumentValueKind::Identifier {
                add_symbol(argument.span, false);
            }
        }
    }

    // Member objects are needed to connect reads such as `props.title` and
    // `state.value` to their declarations. Querying only the property token
    // loses that provenance.
    for member in &file.ast.members {
        add_symbol(member.object, false);
    }
    for spread in &file.ast.spreads {
        add_symbol(spread.argument, false);
    }
    for (span, references) in symbol_spans {
        let mut planned = demand(typefacts_location(&path, span)).symbol(references);
        planned.structural_accessor = structural_accessors.contains(&span);
        demands.push(planned);
    }

    for location in file.compiler_seed_locations()? {
        demands.push(demand(location).symbol(false));
    }
    for span in &file.ast.awaits {
        demands.push(demand(typefacts_location(&path, *span)).async_context());
    }

    // A representative call asks TS-Go for the containing function's async
    // status even when the function contains no await (for example an async
    // arrow that immediately returns a Promise).
    if let Some(call) = file.ast.calls.first() {
        demands.push(demand(typefacts_location(&path, call.span)).async_context());
    }
    for call in &file.ast.calls {
        let callee = typefacts_location(&path, call.callee);
        let query = callee_property_location(&file.source, &callee);
        let mut planned = demand(callee).symbol(false);
        planned.query_location = Some(query);
        planned.type_descriptor = call.arguments.is_empty();
        demands.push(planned);
    }
    Ok(())
}

fn demand(location: solid_ts_facts::Location) -> EntityDemand {
    EntityDemand {
        location,
        query_location: None,
        symbol: false,
        type_descriptor: false,
        resolved_call: false,
        references: false,
        r#async: false,
        structural_accessor: false,
    }
}

trait DemandFlags {
    fn symbol(self, references: bool) -> Self;
    fn resolved_call(self) -> Self;
    fn async_context(self) -> Self;
}

impl DemandFlags for EntityDemand {
    fn symbol(mut self, references: bool) -> Self {
        self.symbol = true;
        self.references = references;
        self
    }

    fn resolved_call(mut self) -> Self {
        self.resolved_call = true;
        self
    }

    fn async_context(mut self) -> Self {
        self.r#async = true;
        self
    }
}

fn stable_deduplicate(demands: &mut Vec<EntityDemand>) {
    demands.sort_by(|left, right| {
        (
            &left.location.path,
            left.location.start_byte,
            left.location.end_byte,
            left.query_location.as_ref().map(|value| value.start_byte),
            left.query_location.as_ref().map(|value| value.end_byte),
        )
            .cmp(&(
                &right.location.path,
                right.location.start_byte,
                right.location.end_byte,
                right.query_location.as_ref().map(|value| value.start_byte),
                right.query_location.as_ref().map(|value| value.end_byte),
            ))
    });
    demands.dedup();
}
