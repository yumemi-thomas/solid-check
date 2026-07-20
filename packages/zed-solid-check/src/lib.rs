use std::path::{Path, PathBuf};
use zed::settings::LspSettings;
use zed_extension_api as zed;

struct SolidCheckExtension;

fn resolve_path(root: &str, value: String) -> String {
    let path = PathBuf::from(&value);
    if path.is_absolute() {
        return value;
    }
    Path::new(root).join(path).to_string_lossy().to_string()
}

fn resolve_worktree_path(worktree: &zed::Worktree, value: String) -> String {
    let root = worktree.root_path();
    resolve_path(&root, value)
}

fn select_command(
    configured: Option<String>,
    local_npm: Option<String>,
    path_command: Option<String>,
) -> Result<String, String> {
    configured.or(local_npm).or(path_command).ok_or_else(|| {
        "solid-checkd was not found; install solid-checker in the project, add solid-checkd to PATH, or configure lsp.solid-check.binary.path".to_string()
    })
}

fn has_solid_checker_dependency(package_json: &str) -> bool {
    let Ok(package) = zed::serde_json::from_str::<zed::serde_json::Value>(package_json) else {
        return false;
    };

    [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ]
    .into_iter()
    .any(|section| {
        package
            .get(section)
            .and_then(|dependencies| dependencies.get("solid-checker"))
            .is_some()
    })
}

fn local_npm_command(worktree: &zed::Worktree) -> Option<String> {
    let package_json = worktree.read_text_file("package.json").ok()?;
    if !has_solid_checker_dependency(&package_json) {
        return None;
    }

    let candidate = if cfg!(windows) {
        "node_modules/.bin/solid-checkd.cmd"
    } else {
        "node_modules/.bin/solid-checkd"
    };
    Some(resolve_worktree_path(worktree, candidate.to_string()))
}

fn default_project(has_app_config: bool) -> &'static str {
    if has_app_config {
        "tsconfig.app.json"
    } else {
        "tsconfig.json"
    }
}

impl zed::Extension for SolidCheckExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        let project = default_project(worktree.read_text_file("tsconfig.app.json").is_ok());
        let mut args = vec!["--project".to_string(), project.to_string()];
        let mut env = worktree.shell_env();
        let configured_command = settings
            .binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .map(|path| resolve_worktree_path(worktree, path));
        let command = select_command(
            configured_command,
            local_npm_command(worktree),
            worktree.which("solid-checkd"),
        )?;

        if let Some(binary) = settings.binary {
            if let Some(arguments) = binary.arguments {
                args = arguments;
            }
            for (key, value) in binary.env.unwrap_or_default() {
                let value = if key == "SOLID_COMPILER_FACTS_BIN" {
                    resolve_worktree_path(worktree, value)
                } else {
                    value
                };
                env.retain(|(existing, _)| existing != &key);
                env.push((key, value));
            }
        }

        Ok(zed::Command { command, args, env })
    }
}

zed::register_extension!(SolidCheckExtension);

#[cfg(test)]
mod tests {
    use super::{default_project, has_solid_checker_dependency, resolve_path, select_command};

    #[test]
    fn resolves_relative_paths_from_the_worktree_root() {
        assert_eq!(
            resolve_path("/workspace/app", "../../bin/solid-checkd".to_string()),
            "/workspace/app/../../bin/solid-checkd"
        );
    }

    #[test]
    fn preserves_absolute_paths() {
        assert_eq!(
            resolve_path("/workspace/app", "/tools/solid-checkd".to_string()),
            "/tools/solid-checkd"
        );
    }

    #[test]
    fn configured_binary_takes_precedence() {
        assert_eq!(
            select_command(
                Some("/configured/solid-checkd".to_string()),
                Some("/project/node_modules/.bin/solid-checkd".to_string()),
                Some("/path/solid-checkd".to_string()),
            ),
            Ok("/configured/solid-checkd".to_string())
        );
    }

    #[test]
    fn local_npm_binary_precedes_path() {
        assert_eq!(
            select_command(
                None,
                Some("/project/node_modules/.bin/solid-checkd".to_string()),
                Some("/path/solid-checkd".to_string()),
            ),
            Ok("/project/node_modules/.bin/solid-checkd".to_string())
        );
    }

    #[test]
    fn path_binary_is_the_final_fallback() {
        assert_eq!(
            select_command(None, None, Some("/path/solid-checkd".to_string())),
            Ok("/path/solid-checkd".to_string())
        );
    }

    #[test]
    fn missing_binary_explains_all_installation_options() {
        let error = select_command(None, None, None).expect_err("missing command");
        assert!(error.contains("install solid-checker"));
        assert!(error.contains("PATH"));
        assert!(error.contains("binary.path"));
    }

    #[test]
    fn detects_solid_checker_in_supported_dependency_sections() {
        for section in [
            "dependencies",
            "devDependencies",
            "optionalDependencies",
            "peerDependencies",
        ] {
            let package_json = format!(r#"{{"{section}":{{"solid-checker":"0.1.4"}}}}"#);
            assert!(has_solid_checker_dependency(&package_json), "{section}");
        }
    }

    #[test]
    fn ignores_missing_or_invalid_package_metadata() {
        assert!(!has_solid_checker_dependency(r#"{"devDependencies":{}}"#));
        assert!(!has_solid_checker_dependency("not json"));
    }

    #[test]
    fn vite_app_config_is_preferred_when_present() {
        assert_eq!(default_project(true), "tsconfig.app.json");
        assert_eq!(default_project(false), "tsconfig.json");
    }
}
