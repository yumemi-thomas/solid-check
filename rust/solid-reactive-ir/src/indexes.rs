//! Read-optimized project indexes used by every analysis stage.
//!
//! This module hides AST and TypeScript table layout from rule discovery. The
//! builder asks semantic questions here instead of repeatedly scanning facts.

use std::{collections::HashMap, sync::Arc};

use solid_facts::{FileFacts, ProjectFacts};
use solid_facts_core::Span;
use solid_ts_facts::{EntityFact, FactTable, FileFact, Location, SymbolFact};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EntitySymbols {
    pub(super) by_path: HashMap<String, HashMap<(u64, u64), String>>,
}

impl EntitySymbols {
    pub(super) fn get(&self, location: &Location) -> Option<&String> {
        self.by_path
            .get(location.path.as_str())
            .and_then(|entities| entities.get(&(location.start_byte, location.end_byte)))
    }
}

pub(super) struct ProjectIndexes<'a> {
    pub(super) files_by_path: HashMap<&'a str, &'a FileFacts>,
    pub(super) ast_files_by_path: HashMap<&'a str, &'a CachedAstFileIndex>,
    typescript: &'a FactTable,
    pub(super) symbols_by_id: HashMap<&'a str, &'a SymbolFact>,
}

impl<'a> ProjectIndexes<'a> {
    pub(super) fn new(
        facts: &'a ProjectFacts,
        ast_indexes: &'a HashMap<String, CachedAstFileIndex>,
    ) -> Self {
        let files_by_path = facts
            .files
            .iter()
            .map(|file| (file.path.as_str(), file))
            .collect();
        let ast_files_by_path = facts
            .files
            .iter()
            .filter_map(|file| {
                ast_indexes
                    .get(file.path.as_str())
                    .map(|index| (file.path.as_str(), index))
            })
            .collect();
        let symbols_by_id = facts
            .typescript
            .symbols
            .iter()
            .map(|symbol| (symbol.id.as_str(), symbol))
            .collect();
        Self {
            files_by_path,
            ast_files_by_path,
            typescript: &facts.typescript,
            symbols_by_id,
        }
    }

    pub(super) fn typescript_file(&self, path: &str) -> Option<&'a FileFact> {
        self.typescript
            .files
            .binary_search_by(|file| file.path.as_str().cmp(path))
            .ok()
            .map(|index| &self.typescript.files[index])
    }

    pub(super) fn entities_for_path(&self, path: &str) -> &'a [EntityFact] {
        let start = self
            .typescript
            .entities
            .partition_point(|entity| entity.location.path.as_str() < path);
        let end = self
            .typescript
            .entities
            .partition_point(|entity| entity.location.path.as_str() <= path);
        &self.typescript.entities[start..end]
    }
}

pub(super) struct CachedAstFileIndex {
    pub(super) ast: Arc<solid_ast_facts::AstFacts>,
    calls_by_span: HashMap<Span, usize>,
    calls_by_callee: HashMap<Span, Vec<usize>>,
    direct_calls_by_callee: HashMap<Span, usize>,
    functions_by_span: HashMap<Span, usize>,
}

impl CachedAstFileIndex {
    pub(super) fn new(file: &FileFacts) -> Self {
        let mut calls_by_span = HashMap::new();
        let mut calls_by_callee = HashMap::<Span, Vec<_>>::new();
        let mut direct_calls_by_callee = HashMap::new();
        for (index, call) in file.ast.calls.iter().enumerate() {
            calls_by_span.entry(call.span).or_insert(index);
            calls_by_callee.entry(call.callee).or_default().push(index);
            if call.direct_callee {
                direct_calls_by_callee.entry(call.callee).or_insert(index);
            }
        }
        let mut functions_by_span = HashMap::new();
        for (index, function) in file.ast.functions.iter().enumerate() {
            functions_by_span.entry(function.span).or_insert(index);
        }
        Self {
            ast: file.ast.clone(),
            calls_by_span,
            calls_by_callee,
            direct_calls_by_callee,
            functions_by_span,
        }
    }

    fn call(&self, index: usize) -> &solid_ast_facts::CallFact {
        &self.ast.calls[index]
    }

    fn function(&self, index: usize) -> &solid_ast_facts::FunctionFact {
        &self.ast.functions[index]
    }

    pub(super) fn call_by_span(&self, span: Span) -> Option<&solid_ast_facts::CallFact> {
        self.calls_by_span.get(&span).map(|index| self.call(*index))
    }

    pub(super) fn direct_call_by_callee(&self, span: Span) -> Option<&solid_ast_facts::CallFact> {
        self.direct_calls_by_callee
            .get(&span)
            .map(|index| self.call(*index))
    }

    pub(super) fn calls_by_callee(
        &self,
        span: Span,
    ) -> impl Iterator<Item = &solid_ast_facts::CallFact> {
        self.calls_by_callee
            .get(&span)
            .into_iter()
            .flatten()
            .map(|index| self.call(*index))
    }

    pub(super) fn function_by_span(&self, span: Span) -> Option<&solid_ast_facts::FunctionFact> {
        self.functions_by_span
            .get(&span)
            .map(|index| self.function(*index))
    }
}
