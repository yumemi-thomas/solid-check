use solid_facts_backend::Snapshot;

use crate::json_output;

pub(crate) struct Emission {
    pub(crate) output: Vec<u8>,
    pub(crate) exit_code: i32,
}

/// Render a diagnostic snapshot and decide its process result.
///
/// Callers only write the returned bytes; output formatting and certification
/// semantics stay local to this module.
pub(crate) fn emit(
    format: &str,
    project_id: &str,
    snapshot: &Snapshot,
    certify: bool,
) -> Result<Emission, Box<dyn std::error::Error>> {
    let output = match format {
        "json" => {
            let mut output = json_output::go_compatible(snapshot, true)?;
            output.push(b'\n');
            output
        }
        "text" => {
            let mut output = format!("{project_id}: {}\n", snapshot.status).into_bytes();
            for finding in &snapshot.findings {
                output.extend_from_slice(
                    format!("{} [{}] {}\n", finding.id, finding.kind, finding.message).as_bytes(),
                );
            }
            output
        }
        format => return Err(format!("unsupported format {format:?}").into()),
    };
    Ok(Emission {
        output,
        exit_code: i32::from(certify && snapshot.status != "certified"),
    })
}

#[cfg(test)]
mod tests {
    use solid_facts_backend::{Metrics, Snapshot};

    use super::emit;

    fn snapshot(status: &str) -> Snapshot {
        Snapshot {
            status: status.into(),
            findings: Vec::new(),
            package_summaries: Vec::new(),
            metrics: Metrics {
                files_analyzed: 1,
                functions_analyzed: 2,
                proof_obligations: 3,
                cached_summaries: 4,
                unresolved_obligations: 0,
            },
        }
    }

    #[test]
    fn text_and_json_share_certification_semantics() {
        for format in ["text", "json"] {
            assert_eq!(
                emit(format, "project", &snapshot("violation"), true)
                    .unwrap()
                    .exit_code,
                1
            );
            assert_eq!(
                emit(format, "project", &snapshot("certified"), true)
                    .unwrap()
                    .exit_code,
                0
            );
            assert_eq!(
                emit(format, "project", &snapshot("violation"), false)
                    .unwrap()
                    .exit_code,
                0
            );
        }
    }

    #[test]
    fn wire_snapshot_with_omitted_optional_fields_round_trips() {
        // The daemon client re-parses emitted snapshots; findings serialize
        // without their empty optional fields, so deserialization must
        // default them instead of failing (which silently downgraded every
        // daemon check with findings to a cold one-shot run).
        let wire = r#"{
            "status": "violation",
            "findings": [{
                "id": "SC1003",
                "rule": "strict-read-untracked",
                "kind": "violation",
                "severity": "error",
                "message": "reactive read outside tracked scope",
                "primaryLocation": {
                    "path": "App.tsx", "startByte": 1, "endByte": 2,
                    "line": 1, "column": 1
                }
            }],
            "packageSummaries": [{
                "name": "solid-js", "evidence": "reviewed", "exportsAnalyzed": 3
            }],
            "metrics": {
                "filesAnalyzed": 1, "functionsAnalyzed": 1,
                "proofObligations": 1, "cachedSummaries": 0,
                "unresolvedObligations": 0
            }
        }"#;
        let decoded: Snapshot = serde_json::from_str(wire).unwrap();
        assert_eq!(decoded.findings[0].subject_kind, "");
        assert!(decoded.findings[0].fixes.is_empty());
        let reemitted = emit("json", "project", &decoded, true).unwrap();
        assert_eq!(reemitted.exit_code, 1);
    }

    #[test]
    fn formats_snapshot_without_adapter_specific_logic() {
        let text = emit("text", "project", &snapshot("certified"), false).unwrap();
        assert_eq!(text.output, b"project: certified\n");

        let json = emit("json", "project", &snapshot("certified"), false).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&json.output).unwrap();
        assert_eq!(decoded["status"], "certified");
        assert!(json.output.ends_with(b"\n"));
    }
}
