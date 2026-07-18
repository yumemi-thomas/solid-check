use std::io::{self, BufRead, BufWriter, Write};
use std::time::Instant;

use dom_expressions_jsx_compiler::{analyze_execution_map, prelude::Either, TransformOptions};
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
    #[serde(default)]
    effect_wrapper: Option<String>,
    #[serde(default)]
    wrap_conditionals: Option<bool>,
    #[serde(default)]
    static_marker: Option<String>,
    #[serde(default)]
    built_ins: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessResponse {
    ok: bool,
    execution_map: Value,
    measurement: Measurement,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Measurement {
    computation_ns: u64,
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
    let started = Instant::now();
    match analyze(request) {
        Ok(execution_map) => serde_json::to_string(&SuccessResponse {
            ok: true,
            execution_map,
            measurement: Measurement {
                computation_ns: started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
            },
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
    if request.path.trim().is_empty() || request.path.contains('\0') {
        return Err("request path is required and must not contain NUL".to_owned());
    }
    if request.compiler_options.module_name.trim().is_empty() {
        return Err("compiler option moduleName is required".to_owned());
    }
    if request.compiler_options.generate != "dom" {
        return Err(format!(
            "compiler facts analysis supports DOM output only, got {:?}",
            request.compiler_options.generate
        ));
    }
    if request
        .compiler_options
        .effect_wrapper
        .as_deref()
        .is_some_and(|name| name.contains('\0'))
    {
        return Err("compiler option effectWrapper must not contain NUL".to_owned());
    }
    if request
        .compiler_options
        .static_marker
        .as_deref()
        .is_some_and(|marker| marker.contains('\0'))
    {
        return Err("compiler option staticMarker must not contain NUL".to_owned());
    }
    if request
        .compiler_options
        .built_ins
        .iter()
        .any(|name| name.trim().is_empty())
    {
        return Err("compiler option builtIns must not contain empty names".to_owned());
    }
    if request
        .compiler_options
        .built_ins
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err("compiler option builtIns must be sorted and unique".to_owned());
    }
    let effect_wrapper = request.compiler_options.effect_wrapper.map(|name| {
        if name.is_empty() {
            Either::A(false)
        } else {
            Either::B(name)
        }
    });
    let options = TransformOptions {
        filename: Some(request.path),
        module_name: Some(request.compiler_options.module_name),
        generate: Some(request.compiler_options.generate),
        hydratable: Some(request.compiler_options.hydratable),
        dev: Some(request.compiler_options.dev),
        effect_wrapper,
        wrap_conditionals: request.compiler_options.wrap_conditionals,
        static_marker: request.compiler_options.static_marker,
        built_ins: Some(request.compiler_options.built_ins),
        compiler_facts: Some(true),
        ..TransformOptions::default()
    };
    let execution_map =
        analyze_execution_map(&request.source, &options).map_err(|error| error.to_string())?;
    serde_json::from_str(&execution_map).map_err(|error| error.to_string())
}

fn serialize_error(code: &str, message: String) -> String {
    serde_json::to_string(&ErrorResponse {
        ok: false,
        error: ErrorBody { code, message },
    })
    .expect("error response is serializable")
}
