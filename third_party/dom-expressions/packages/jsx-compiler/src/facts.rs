use oxc_span::Span as OxcSpan;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) const COMPILER_FACTS_PROTOCOL: u32 = 1;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Span {
    start: u32,
    end: u32,
}

impl From<OxcSpan> for Span {
    fn from(span: OxcSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionRegion {
    span: Span,
    reason: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CallbackRole {
    span: Span,
    role: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OwnershipRegion {
    span: Span,
    kind: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsxOperation {
    span: Span,
    kind: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionMap {
    compiler_facts_protocol: u32,
    source_hash: String,
    tracked_regions: Vec<ExecutionRegion>,
    untracked_regions: Vec<ExecutionRegion>,
    ownership_regions: Vec<OwnershipRegion>,
    callback_roles: Vec<CallbackRole>,
    jsx_operations: Vec<JsxOperation>,
}

#[derive(Default)]
pub(crate) struct FactRecorder {
    enabled: bool,
    tracked_regions: Vec<ExecutionRegion>,
    untracked_regions: Vec<ExecutionRegion>,
    callback_roles: Vec<CallbackRole>,
    jsx_operations: Vec<JsxOperation>,
}

impl FactRecorder {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub(crate) fn tracked(&mut self, span: OxcSpan, reason: &'static str, operation: &'static str) {
        if !self.enabled {
            return;
        }
        self.tracked_regions.push(ExecutionRegion {
            span: span.into(),
            reason,
        });
        self.jsx_operations.push(JsxOperation {
            span: span.into(),
            kind: operation,
        });
    }

    /// Records that the transform decided an expression does NOT track: it
    /// renders once (template inlining, unwrapped insert, static component
    /// prop) with no effect wrapper or getter around it.
    pub(crate) fn untracked(&mut self, span: OxcSpan, reason: &'static str) {
        if !self.enabled {
            return;
        }
        self.untracked_regions.push(ExecutionRegion {
            span: span.into(),
            reason,
        });
    }

    pub(crate) fn callback(&mut self, span: OxcSpan, role: &'static str, operation: &'static str) {
        if !self.enabled {
            return;
        }
        self.callback_roles.push(CallbackRole {
            span: span.into(),
            role,
        });
        self.jsx_operations.push(JsxOperation {
            span: span.into(),
            kind: operation,
        });
    }

    pub(crate) fn operation(&mut self, span: OxcSpan, kind: &'static str) {
        if !self.enabled {
            return;
        }
        self.jsx_operations.push(JsxOperation {
            span: span.into(),
            kind,
        });
    }

    pub(crate) fn finish(&self, source: &str) -> Option<String> {
        self.enabled.then(|| {
            let mut tracked_regions = self.tracked_regions.clone();
            tracked_regions.sort_by(|left, right| {
                (left.span.start, left.span.end, left.reason).cmp(&(
                    right.span.start,
                    right.span.end,
                    right.reason,
                ))
            });
            let mut untracked_regions = self.untracked_regions.clone();
            untracked_regions.sort_by(|left, right| {
                (left.span.start, left.span.end, left.reason).cmp(&(
                    right.span.start,
                    right.span.end,
                    right.reason,
                ))
            });
            let mut callback_roles = self.callback_roles.clone();
            callback_roles.sort_by(|left, right| {
                (left.span.start, left.span.end, left.role).cmp(&(
                    right.span.start,
                    right.span.end,
                    right.role,
                ))
            });
            let mut jsx_operations = self.jsx_operations.clone();
            jsx_operations.sort_by(|left, right| {
                (left.span.start, left.span.end, left.kind).cmp(&(
                    right.span.start,
                    right.span.end,
                    right.kind,
                ))
            });
            let execution_map = ExecutionMap {
                compiler_facts_protocol: COMPILER_FACTS_PROTOCOL,
                source_hash: format!("sha256:{:x}", Sha256::digest(source.as_bytes())),
                tracked_regions,
                untracked_regions,
                ownership_regions: Vec::new(),
                callback_roles,
                jsx_operations,
            };
            serde_json::to_string(&execution_map).expect("ExecutionMap is serializable")
        })
    }
}
