use std::io::{self, BufRead, BufWriter, Write};

use dom_expressions_jsx_compiler::{transform, TransformOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const COMPILER_FACTS_PROTOCOL: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AnalysisRequest {
    compiler_facts_protocol: u32,
    path: String,
    source: String,
    source_hash: String,
    compiler_options: CompilerOptions,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct CompilerOptions {
    module_name: String,
    generate: String,
    #[serde(default)]
    hydratable: bool,
    #[serde(default)]
    dev: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessResponse {
    ok: bool,
    execution_map: Value,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    ok: bool,
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

fn main() -> io::Result<()> {
    let input = io::stdin();
    let output = io::stdout();
    let mut writer = BufWriter::new(output.lock());
    for line in input.lock().lines() {
        let response = match line {
            Ok(line) => respond(&line),
            Err(error) => serialize_error("invalid-request", error.to_string()),
        };
        writer.write_all(response.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    Ok(())
}

fn respond(line: &str) -> String {
    let request: AnalysisRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(error) => return serialize_error("invalid-request", error.to_string()),
    };
    match analyze(request) {
        Ok(execution_map) => serde_json::to_string(&SuccessResponse {
            ok: true,
            execution_map,
        })
        .expect("success response is serializable"),
        Err(error) => serialize_error("analysis-failed", error),
    }
}

fn analyze(request: AnalysisRequest) -> Result<Value, String> {
    if request.compiler_facts_protocol != COMPILER_FACTS_PROTOCOL {
        return Err(format!(
            "compiler facts protocol {} is unsupported",
            request.compiler_facts_protocol
        ));
    }
    let source_hash = format!("sha256:{:x}", Sha256::digest(request.source.as_bytes()));
    if source_hash != request.source_hash {
        return Err("request source hash does not match exact source bytes".to_owned());
    }
    let result = transform(
        request.source,
        Some(TransformOptions {
            filename: Some(request.path),
            module_name: Some(request.compiler_options.module_name),
            generate: Some(request.compiler_options.generate),
            hydratable: Some(request.compiler_options.hydratable),
            dev: Some(request.compiler_options.dev),
            compiler_facts: Some(true),
            ..TransformOptions::default()
        }),
    )
    .map_err(|error| error.to_string())?;
    let execution_map = result
        .execution_map
        .ok_or_else(|| "compiler returned no ExecutionMap".to_owned())?;
    serde_json::from_str(&execution_map).map_err(|error| error.to_string())
}

fn serialize_error(code: &str, message: String) -> String {
    serde_json::to_string(&ErrorResponse {
        ok: false,
        error: ErrorBody { code, message },
    })
    .expect("error response is serializable")
}
