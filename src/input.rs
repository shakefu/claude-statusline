use serde::Deserialize;

use crate::util::path_basename;

/// Parsed status input produced from the JSON read off stdin.
pub struct StatusInput {
    /// Used-token count and window-size, both present or both absent.
    pub token_usage: Option<(u64, u64)>,
    pub directory_label: String,
    pub model_name: Option<String>,
}

// ── Raw serde shapes (private) ──────────────────────────────────────────

#[derive(Deserialize)]
struct RawInput {
    context_window: Option<ContextWindow>,
    workspace: Option<Workspace>,
    model: Option<Model>,
}

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: Option<u64>,
    context_window_size: Option<u64>,
}

#[derive(Deserialize)]
struct Workspace {
    repo: Option<Repo>,
    current_dir: Option<String>,
}

#[derive(Deserialize)]
struct Repo {
    owner: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct Model {
    display_name: Option<String>,
}

// ── Public API ──────────────────────────────────────────────────────────

/// Parse a JSON string into a [`StatusInput`].
///
/// Returns `None` when the input is empty or when the JSON is malformed,
/// matching the spec requirement to exit silently in those cases.
pub fn parse_input(input: &str) -> Option<StatusInput> {
    if input.trim().is_empty() {
        return None;
    }

    let raw: RawInput = serde_json::from_str(input).ok()?;

    // Context window: both fields must be present to produce token counts.
    let token_usage = raw
        .context_window
        .and_then(|cw| Some((cw.total_input_tokens?, cw.context_window_size?)));

    // Workspace → directory label.
    let directory_label = match raw.workspace {
        Some(ws) => {
            match ws.repo {
                Some(Repo {
                    owner: Some(ref owner),
                    name: Some(ref name),
                }) if !owner.is_empty() && !name.is_empty() => {
                    format!("{}/{}", owner, name)
                }
                _ => {
                    // Fall back to basename of current_dir.
                    ws.current_dir
                        .as_deref()
                        .map(path_basename)
                        .unwrap_or("")
                        .to_string()
                }
            }
        }
        None => String::new(),
    };

    // Model display name.
    let model_name = raw.model.and_then(|m| m.display_name);

    Some(StatusInput {
        token_usage,
        directory_label,
        model_name,
    })
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Input (top-level) ───────────────────────────────────────────

    #[test]
    fn empty_input_returns_none() {
        assert!(parse_input("").is_none());
    }

    #[test]
    fn whitespace_only_input_returns_none() {
        assert!(parse_input("   \n\t  ").is_none());
    }

    #[test]
    fn malformed_json_returns_none() {
        assert!(parse_input("{not json}").is_none());
    }

    #[test]
    fn valid_json_all_fields() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": 1000,
                "context_window_size": 200000
            },
            "workspace": {
                "repo": { "owner": "acme", "name": "widgets" },
                "current_dir": "/home/user/widgets"
            },
            "model": {
                "display_name": "Claude Sonnet 4"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.token_usage, Some((1000, 200000)));
        assert_eq!(result.directory_label, "acme/widgets");
        assert_eq!(result.model_name.as_deref(), Some("Claude Sonnet 4"));
    }

    // ── Input.ContextWindow ─────────────────────────────────────────

    #[test]
    fn context_window_both_fields_present() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": 500,
                "context_window_size": 100000
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.token_usage, Some((500, 100000)));
    }

    #[test]
    fn context_window_missing_total_input_tokens() {
        let json = r#"{
            "context_window": {
                "context_window_size": 100000
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert!(result.token_usage.is_none());
    }

    #[test]
    fn context_window_missing_context_window_size() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": 500
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert!(result.token_usage.is_none());
    }

    #[test]
    fn context_window_null_fields() {
        let json = r#"{
            "context_window": {
                "total_input_tokens": null,
                "context_window_size": null
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert!(result.token_usage.is_none());
    }

    #[test]
    fn context_window_absent() {
        let json = r#"{}"#;
        let result = parse_input(json).unwrap();
        assert!(result.token_usage.is_none());
    }

    // ── Input.Workspace ─────────────────────────────────────────────

    #[test]
    fn workspace_repo_owner_and_name() {
        let json = r#"{
            "workspace": {
                "repo": { "owner": "acme", "name": "widgets" },
                "current_dir": "/somewhere"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "acme/widgets");
    }

    #[test]
    fn workspace_repo_missing_owner_falls_back_to_current_dir() {
        let json = r#"{
            "workspace": {
                "repo": { "name": "widgets" },
                "current_dir": "/home/user/project"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "project");
    }

    #[test]
    fn workspace_repo_missing_name_falls_back_to_current_dir() {
        let json = r#"{
            "workspace": {
                "repo": { "owner": "acme" },
                "current_dir": "/home/user/project"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "project");
    }

    #[test]
    fn workspace_no_repo_uses_current_dir_basename() {
        let json = r#"{
            "workspace": {
                "current_dir": "/home/user/project"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "project");
    }

    #[test]
    fn workspace_no_repo_no_current_dir() {
        let json = r#"{
            "workspace": {}
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "");
    }

    #[test]
    fn workspace_absent() {
        let json = r#"{}"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "");
    }

    #[test]
    fn current_dir_basename_extraction() {
        let json = r#"{
            "workspace": {
                "current_dir": "/home/user/deep/nested/project"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "project");
    }

    #[test]
    fn current_dir_root_path() {
        let json = r#"{
            "workspace": {
                "current_dir": "/"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "");
    }

    #[test]
    fn current_dir_trailing_slash() {
        let json = r#"{
            "workspace": {
                "current_dir": "/home/user/project/"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.directory_label, "project");
    }

    // ── Input.Model ─────────────────────────────────────────────────

    #[test]
    fn model_with_display_name() {
        let json = r#"{
            "model": {
                "display_name": "Claude Opus 4"
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert_eq!(result.model_name.as_deref(), Some("Claude Opus 4"));
    }

    #[test]
    fn model_absent() {
        let json = r#"{}"#;
        let result = parse_input(json).unwrap();
        assert!(result.model_name.is_none());
    }

    #[test]
    fn model_display_name_null() {
        let json = r#"{
            "model": {
                "display_name": null
            }
        }"#;
        let result = parse_input(json).unwrap();
        assert!(result.model_name.is_none());
    }
}
