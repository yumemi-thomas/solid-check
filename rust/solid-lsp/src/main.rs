use std::{env, path::PathBuf};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("solid-checkd-rust: {error}");
            std::process::exit(if is_handshake_failure(error.as_ref()) {
                3
            } else {
                2
            });
        }
    }
}

fn is_handshake_failure(mut error: &(dyn std::error::Error + 'static)) -> bool {
    loop {
        if error
            .downcast_ref::<solid_facts_backend::BackendError>()
            .is_some_and(|error| matches!(error, solid_facts_backend::BackendError::Handshake(_)))
        {
            return true;
        }
        let Some(source) = error.source() else {
            return false;
        };
        error = source;
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = PathBuf::from("tsconfig.json");
    let mut typefacts = solid_facts_backend::default_typefacts_executable();
    let mut contract_paths = Vec::new();
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--project" => {
                index += 1;
                project = arguments
                    .get(index)
                    .ok_or("--project requires a value")?
                    .into();
            }
            "--typefacts" => {
                index += 1;
                typefacts = arguments
                    .get(index)
                    .ok_or("--typefacts requires a value")?
                    .clone();
            }
            "--contract" => {
                index += 1;
                contract_paths.push(
                    arguments
                        .get(index)
                        .ok_or("--contract requires a value")?
                        .clone(),
                );
            }
            "-h" | "--help" => {
                println!(
                    "Usage: solid-checkd-rust [OPTIONS]\n\n\
                     --project <PATH>    TypeScript project (default: tsconfig.json)\n\
                     --typefacts <PATH>  TypeFacts service executable\n\
                     --contract <PATH>   Package contract override (repeatable)"
                );
                return Ok(());
            }
            argument => return Err(format!("unknown option {argument:?}").into()),
        }
        index += 1;
    }
    solid_lsp::serve_with_contracts(
        &project,
        &typefacts,
        &contract_paths,
        std::io::stdin(),
        std::io::stdout(),
    )?;
    Ok(())
}
