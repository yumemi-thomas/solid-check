mod rules;

use serde::{Deserialize, Serialize};
use solid_reactive_ir::{ExecutionRole, Program};
use solid_ts_facts::Location;
use std::time::{Duration, Instant};

pub use rules::{Rule, RuleMetadata};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceStep {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub rule: String,
    pub kind: String,
    pub message: String,
    pub severity: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub analysis_context: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub subject_kind: String,
    pub primary_location: Location,
    pub related_locations: Vec<Location>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<solid_reactive_ir::Fix>,
}

impl Finding {
    fn new(rule: Rule, message: String, primary_location: Location) -> Self {
        let metadata = rule.metadata();
        Self {
            id: metadata.code.into(),
            rule: metadata.name.into(),
            kind: if metadata.uncertifiable {
                "uncertifiable".into()
            } else {
                "violation".into()
            },
            message,
            severity: metadata.severity.into(),
            analysis_context: String::new(),
            subject_kind: String::new(),
            primary_location,
            related_locations: vec![],
            evidence: vec![],
            fixes: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SolveTimings {
    pub total: Duration,
    pub finding_construction: Duration,
    pub final_ordering: Duration,
}

#[must_use]
pub fn solve(program: &Program) -> Vec<Finding> {
    solve_measured(program).0
}

#[must_use]
pub fn solve_measured(program: &Program) -> (Vec<Finding>, SolveTimings) {
    let total_started = Instant::now();
    let construction_started = Instant::now();
    let mut findings = program
        .reads
        .iter()
        .filter(|read| {
            matches!(
                read.execution,
                ExecutionRole::UntrackedRendering | ExecutionRole::EffectApply
            )
        })
        .map(|read| Finding {
            analysis_context: read.context.clone(),
            subject_kind: read.kind.clone(),
            related_locations: strict_read_related_locations(read),
            evidence: strict_read_evidence(read),
            ..Finding::new(
                Rule::StrictReadUntracked,
                strict_read_message(read),
                read.location.clone(),
            )
        })
        .collect::<Vec<_>>();
    findings.extend(
        program
            .writes
            .iter()
            .filter(|write| !write.allowed_by_option && !allowed_write_role(write.execution))
            .map(|write| {
                let context = if write.context.is_empty() {
                    "owned scope"
                } else {
                    &write.context
                };
                let (operation, provenance) = if write.setter.starts_with("refresh(") {
                    (
                        "refresh()".to_owned(),
                        "the refresh target is a proven Solid source accessor or store".to_owned(),
                    )
                } else {
                    (
                        format!("signal setter {:?}", write.setter),
                        format!(
                            "{:?} is the setter returned by createSignal or createStore",
                            write.setter
                        ),
                    )
                };
                Finding {
                    analysis_context: context.into(),
                    related_locations: vec![write.declaration.clone()],
                    evidence: vec![
                        EvidenceStep {
                            message: provenance,
                            location: Some(write.declaration.clone()),
                        },
                        EvidenceStep {
                            message: "the call executes in an owned scope with no allowed write role"
                                .into(),
                            location: Some(write.location.clone()),
                        },
                    ],
                    ..Finding::new(
                        Rule::ReactiveWriteInOwnedScope,
                        format!(
                            "{operation} is called inside owned scope {context}; move the write to an event handler, action, onSettled, tracked effect, or untracked callback"
                        ),
                        write.location.clone(),
                    )
                }
            }),
    );
    findings.extend(program.leaf_operations.iter().map(|operation| {
        let (rule, message) = match operation.primitive.as_str() {
            "onCleanup" => (
                Rule::CleanupInForbiddenScope,
                format!(
                    "onCleanup cannot be used inside {}; return a cleanup function instead",
                    operation.owner
                ),
            ),
            "flush" => (
                Rule::FlushInForbiddenScope,
                format!(
                    "flush cannot be called inside {} because the leaf owner is not reentrant",
                    operation.owner
                ),
            ),
            _ => (
                Rule::PrimitiveInLeafOwner,
                format!(
                    "cannot create reactive primitive {} inside leaf owner {}",
                    operation.primitive, operation.owner
                ),
            ),
        };
        Finding {
            evidence: vec![EvidenceStep {
                message: format!(
                    "the call is lexically contained by the {} callback",
                    operation.owner
                ),
                location: Some(operation.location.clone()),
            }],
            fixes: operation.fix.clone().into_iter().collect(),
            ..Finding::new(rule, message, operation.location.clone())
        }
    }));
    findings.extend(
        program
            .invalid_cleanup_returns
            .iter()
            .map(|invalid| Finding {
                evidence: vec![EvidenceStep {
                    message: "the callback statically returns a non-function value, including an implicit Promise from an async callback".into(),
                    location: Some(invalid.location.clone()),
                }],
                ..Finding::new(
                    Rule::InvalidCleanupReturn,
                    format!(
                        "{} callback returns a non-function cleanup value; return a cleanup function or undefined",
                        invalid.primitive
                    ),
                    invalid.location.clone(),
                )
            }),
    );
    findings.extend(
        program
            .unresolved_cleanup_returns
            .iter()
            .map(|unresolved| Finding {
                evidence: vec![EvidenceStep {
                    message: format!(
                        "cannot prove that {} callback returns only a cleanup function or undefined",
                        unresolved.primitive
                    ),
                    location: Some(unresolved.location.clone()),
                }],
                ..Finding::new(
                    Rule::CleanupReturnUnresolved,
                    format!(
                        "cannot prove that {} callback returns only a cleanup function or undefined",
                        unresolved.primitive
                    ),
                    unresolved.location.clone(),
                )
            }),
    );
    findings.extend(program.static_violations.iter().map(|violation| {
        let rule = Rule::from_identity(&violation.id, &violation.rule).unwrap_or_else(|| {
            panic!(
                "diagnostic identity is missing from the rule catalog: {} [{}]",
                violation.id, violation.rule
            )
        });
        Finding {
            analysis_context: violation.analysis_context.clone(),
            evidence: vec![EvidenceStep {
                message: if violation.rule == "component-props-destructure" {
                    "the destructuring pattern is bound to proven component props".into()
                } else if violation.rule == "package-contract-export-missing" {
                    "the imported package has a contract, but this export has no effect summary"
                        .into()
                } else {
                    "the invalid API shape is statically present at this call".into()
                },
                location: Some(violation.location.clone()),
            }],
            fixes: violation.fixes.clone(),
            ..Finding::new(rule, violation.message.clone(), violation.location.clone())
        }
    }));
    findings.extend(program.directive_creations.iter().map(|creation| Finding {
        evidence: vec![EvidenceStep {
            message: if creation.returned_closure {
                "the primitive is created inside the callback returned to a compiler-recognized ref application".into()
            } else {
                "the primitive is created inside a compiler-recognized ref application callback".into()
            },
            location: Some(creation.location.clone()),
        }],
        ..Finding::new(
            Rule::PrimitiveInDirectiveApplication,
            format!(
                "cannot create reactive primitive {} in a directive application callback; create it during directive setup",
                creation.primitive
            ),
            creation.location.clone(),
        )
    }));
    findings.extend(program.missing_owners.iter().filter_map(|requirement| {
        if !requirement.report {
            return None;
        }
        let (rule, message) = match requirement.operation.as_str() {
            "cleanup" => (
                Rule::NoOwnerCleanup,
                "onCleanup called without a reactive owner will never run",
            ),
            "boundary" => (
                Rule::NoOwnerBoundary,
                "boundary created without a reactive owner will never be disposed",
            ),
            "settled-cleanup" => (
                Rule::SettledCleanupUnowned,
                "onSettled returns a cleanup in an unowned or children-forbidden scope, so the cleanup cannot be honored",
            ),
            _ => (
                Rule::NoOwnerEffect,
                "effect created without a reactive owner will never be disposed",
            ),
        };
        let uncertain = requirement.uncertain;
        Some(Finding {
            kind: if uncertain {
                "uncertifiable".into()
            } else {
                "violation".into()
            },
            severity: if uncertain {
                "error".into()
            } else {
                rule.metadata().severity.into()
            },
            evidence: vec![EvidenceStep {
                message: "no containing component, computation, or root owner dominates this operation".into(),
                location: Some(requirement.location.clone()),
            }],
            ..Finding::new(
                rule,
                if uncertain {
                    format!(
                        "{message}; caller ownership for this exported function cannot be proven inside the project"
                    )
                } else {
                    message.into()
                },
                requirement.location.clone(),
            )
        })
    }));
    findings.extend(program.async_reads.iter().filter_map(|read| {
        let (rule, message) = if let Some(owner) = &read.leaf_owner {
            (
                Rule::PendingAsyncForbiddenScope,
                format!(
                    "pending async accessor {:?} is read inside {}, which cannot suspend",
                    read.accessor, owner
                ),
            )
        } else if read.execution == ExecutionRole::UntrackedRendering {
            (
                Rule::PendingAsyncUntrackedRead,
                format!(
                    "pending async accessor {:?} is read outside a tracking scope",
                    read.accessor
                ),
            )
        } else if read.execution == ExecutionRole::TrackedJsx && !read.under_loading {
            (
                Rule::AsyncOutsideLoadingBoundary,
                format!(
                    "async accessor {:?} is rendered without a dominating Loading boundary",
                    read.accessor
                ),
            )
        } else {
            return None;
        };
        Some(Finding {
            related_locations: vec![read.declaration.clone()],
            evidence: vec![
                EvidenceStep {
                    message: "the accessor is returned by an async computation".into(),
                    location: Some(read.declaration.clone()),
                },
                EvidenceStep {
                    message: message.clone(),
                    location: Some(read.location.clone()),
                },
            ],
            ..Finding::new(rule, message, read.location.clone())
        })
    }));
    findings.extend(
        program
            .actions
            .iter()
            .filter(|action| !allowed_write_role(action.execution))
            .map(|action| Finding {
                evidence: vec![EvidenceStep {
                    message: "invoking an action starts a write transaction while an owner is active"
                        .into(),
                    location: Some(action.location.clone()),
                }],
                ..Finding::new(
                    Rule::ActionCalledInOwnedScope,
                    format!(
                        "action {:?} is called inside owned scope {}; invoke it from an event, effect callback, onSettled, or another imperative scope",
                        action.action, action.context
                    ),
                    action.location.clone(),
                )
            }),
    );
    let finding_construction = construction_started.elapsed();
    let ordering_started = Instant::now();
    findings.sort_by(|left, right| {
        (
            &left.primary_location.path,
            left.primary_location.start_byte,
            &left.id,
            &left.message,
        )
            .cmp(&(
                &right.primary_location.path,
                right.primary_location.start_byte,
                &right.id,
                &right.message,
            ))
    });
    findings.dedup();
    let final_ordering = ordering_started.elapsed();
    (
        findings,
        SolveTimings {
            total: total_started.elapsed(),
            finding_construction,
            final_ordering,
        },
    )
}

const fn allowed_write_role(role: ExecutionRole) -> bool {
    matches!(
        role,
        ExecutionRole::EventCallback
            | ExecutionRole::DeferredCallback
            | ExecutionRole::EffectApply
            | ExecutionRole::DirectiveApply
    )
}

fn strict_read_message(read: &solid_reactive_ir::ReactiveRead) -> String {
    let context = if read.context.is_empty() {
        "rendering function"
    } else {
        &read.context
    };
    if read.via.is_empty() {
        format!(
            "{} {:?} is read directly in {context} and will not update; move the read into tracked JSX, a memo, or an effect compute function",
            reactive_value_label(&read.kind),
            read.accessor
        )
    } else {
        format!(
            "{} {:?} is read through {} in {context} and will not update; move the call into tracked JSX, a memo, or an effect compute function",
            reactive_value_label(&read.kind),
            read.accessor,
            read.via
        )
    }
}

fn reactive_value_label(kind: &str) -> &'static str {
    match kind {
        "store-path" => "reactive store path",
        "component-props" => "component prop",
        _ => "reactive accessor",
    }
}

fn strict_read_evidence(read: &solid_reactive_ir::ReactiveRead) -> Vec<EvidenceStep> {
    let mut evidence = vec![EvidenceStep {
        message: format!(
            "{:?} is a {}",
            read.accessor,
            reactive_value_label(&read.kind)
        ),
        location: Some(read.declaration.clone()),
    }];
    if let Some(origin) = &read.origin {
        let origin_context = if read.origin_context.is_empty() {
            &read.via
        } else {
            &read.origin_context
        };
        evidence.push(EvidenceStep {
            message: format!(
                "{origin_context} reads the {}",
                reactive_value_label(&read.kind)
            ),
            location: Some(origin.clone()),
        });
        evidence.push(EvidenceStep {
            message: format!(
                "the call to {} propagates that read into {}",
                read.via,
                if read.context.is_empty() {
                    "rendering function"
                } else {
                    &read.context
                }
            ),
            location: Some(read.location.clone()),
        });
        evidence.push(EvidenceStep {
            message: "the call is outside every compiler-tracked JSX region and deferred callback"
                .into(),
            location: Some(read.location.clone()),
        });
    } else {
        evidence.push(EvidenceStep {
            message: "the cross-file reference resolves to that accessor declaration".into(),
            location: Some(read.location.clone()),
        });
        evidence.push(EvidenceStep {
            message: "the read is outside every compiler-tracked JSX region and deferred callback"
                .into(),
            location: Some(read.location.clone()),
        });
    }
    evidence
}

fn strict_read_related_locations(
    read: &solid_reactive_ir::ReactiveRead,
) -> Vec<solid_ts_facts::Location> {
    let mut locations = vec![read.declaration.clone()];
    if let Some(origin) = &read.origin {
        locations.push(origin.clone());
    }
    locations
}
