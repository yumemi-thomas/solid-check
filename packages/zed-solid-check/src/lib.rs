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
        let mut command = worktree.which("solid-checkd").ok_or_else(|| {
            "solid-checkd was not found in PATH; configure lsp.solid-check.binary.path".to_string()
        })?;
        let mut args = vec!["--project".to_string(), "tsconfig.json".to_string()];
        let mut env = worktree.shell_env();

        if let Some(binary) = settings.binary {
            if let Some(path) = binary.path {
                command = resolve_worktree_path(worktree, path);
            }
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
    use super::resolve_path;

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
}
