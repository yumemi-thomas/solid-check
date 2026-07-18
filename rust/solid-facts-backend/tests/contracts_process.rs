#[path = "support/process.rs"]
mod support;

use std::{env, fs, path::PathBuf, process::Command};

use support::{decode_findings, temporary_directory};

#[test]
fn cli_consumes_discovered_package_contracts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let compiler = match env::var("SOLID_COMPILER_FACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (fixture, rule, message) in [
        ("package-consumer", "strict-read-untracked", "readCount"),
        (
            "package-return-consumer",
            "strict-read-untracked",
            "created count",
        ),
        (
            "package-callback-consumer",
            "strict-read-untracked",
            "runInline",
        ),
        (
            "package-store-consumer",
            "strict-read-untracked",
            "state.value",
        ),
        (
            "package-unknown-export",
            "package-contract-export-missing",
            "unknownPrimitive",
        ),
        ("bundled-solid-consumer", "strict-read-untracked", "doubled"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
            .env("SOLID_TYPEFACTS_BIN", &typefacts)
            .env("SOLID_COMPILER_FACTS_BIN", &compiler)
            .args(["--format", "json", "--project"])
            .arg(root.join(format!(
                "internal/reactiveir/testdata/{fixture}/tsconfig.json"
            )))
            .output()
            .expect("run Rust diagnostic CLI");
        assert!(
            output.status.success(),
            "fixture {fixture}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let findings = decode_findings(&output.stdout);
        assert_eq!(findings.len(), 1, "fixture {fixture}: {findings:#?}");
        assert_eq!(findings[0]["rule"], rule, "fixture {fixture}");
        assert!(
            findings[0]["message"]
                .as_str()
                .is_some_and(|finding| finding.contains(message))
        );
    }
}

#[test]
fn cli_validates_a_contract_without_opening_a_project() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let contract = root.join(
        "internal/reactiveir/testdata/package-consumer/node_modules/reactive-package/solid-reactivity.json",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env_remove("SOLID_TYPEFACTS_BIN")
        .env_remove("SOLID_COMPILER_FACTS_BIN")
        .args(["--validate-contract"])
        .arg(contract)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_emits_and_revalidates_package_contracts() {
    let typefacts = match env::var("SOLID_TYPEFACTS_BIN") {
        Ok(value) => value,
        Err(_) => return,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = temporary_directory("emit-contract");
    let output = directory.join("solid-reactivity.json");
    let declaration = directory.join("index.d.ts");
    fs::write(
        &declaration,
        "export declare function createCount(): () => number;\n",
    )
    .unwrap();
    let producer = root.join("internal/reactiveir/testdata/package-return-producer/tsconfig.json");
    let result = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env("SOLID_TYPEFACTS_BIN", &typefacts)
        .args(["--project"])
        .arg(producer)
        .args(["--emit-contract"])
        .arg(&output)
        .args([
            "--package-name",
            "reactive-package",
            "--package-version",
            "1.0.0",
            "--declaration-artifact",
        ])
        .arg(&declaration)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let contract: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    for name in [
        "createCount",
        "createAliasedCount",
        "createArrowCount",
        "createMemoCount",
    ] {
        assert_eq!(contract["exports"][name]["returns"]["kind"], "accessor");
    }
    assert_eq!(
        contract["exports"]["createState"]["returns"]["kind"],
        "store-path"
    );
    assert_eq!(contract["exports"]["packageVersion"]["kind"], "value");
    assert_eq!(contract["artifacts"]["declaration"]["path"], "index.d.ts");

    let validate = Command::new(env!("CARGO_BIN_EXE_solid-check-rust"))
        .env_remove("SOLID_TYPEFACTS_BIN")
        .args(["--validate-contract"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        validate.status.success(),
        "{}",
        String::from_utf8_lossy(&validate.stderr)
    );
    fs::remove_dir_all(directory).unwrap();
}
