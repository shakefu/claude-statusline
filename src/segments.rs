use crate::format_tokens::format_tokens;
use crate::input::StatusInput;
use crate::util::path_basename;

// ANSI color codes
const RED: &str = "\x1b[0;31m";
const YELLOW: &str = "\x1b[0;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";

/// Render the token usage segment: `{used}/{total}` colored by usage level,
/// followed by a reset and a trailing space.
/// Returns empty string when token counts are absent.
pub fn token_segment(input: &StatusInput) -> String {
    match input.token_usage {
        Some((used, total)) => {
            let color = if used >= 100_000 {
                RED
            } else if used >= 60_000 {
                YELLOW
            } else {
                CYAN
            };
            format!(
                "{}{}/{}{} ",
                color,
                format_tokens(used),
                format_tokens(total),
                RESET
            )
        }
        None => String::new(),
    }
}

/// Render the directory label colored blue.
pub fn directory_segment(input: &StatusInput) -> String {
    format!("{}{}{}", BLUE, input.directory_label, RESET)
}

/// Format a trimmed git branch name into the colored segment string.
/// Returns empty string if the branch name is empty.
pub fn format_git_branch(branch: &str) -> String {
    if branch.is_empty() {
        return String::new();
    }
    format!("{} ({}){}", GREEN, branch, RESET)
}

/// Render the git branch segment by running `git rev-parse --abbrev-ref HEAD`.
/// Returns empty string if git fails or returns empty output.
pub fn git_branch_segment() -> String {
    let output = std::process::Command::new("git")
        .args(["-c", "core.fsmonitor=false", "rev-parse", "--abbrev-ref", "HEAD"])
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let branch = String::from_utf8(o.stdout).unwrap_or_default();
            format_git_branch(branch.trim())
        }
        _ => String::new(),
    }
}

/// Format a VIRTUAL_ENV path into the colored segment string.
/// Returns empty string if the path is empty.
pub fn format_venv(venv_path: &str) -> String {
    if venv_path.is_empty() {
        return String::new();
    }
    let basename = path_basename(venv_path);
    format!("{} ({}) {}", YELLOW, basename, RESET)
}

/// Render the virtual environment segment by reading $VIRTUAL_ENV.
/// Returns empty string if the variable is unset or empty.
pub fn venv_segment() -> String {
    match std::env::var("VIRTUAL_ENV") {
        Ok(val) if !val.is_empty() => format_venv(&val),
        _ => String::new(),
    }
}

/// Render the model name segment: ` [{name}]` colored cyan.
/// Returns empty string when the model name is absent.
pub fn model_segment(input: &StatusInput) -> String {
    match &input.model_name {
        Some(name) => format!("{} [{}]{}", CYAN, name, RESET),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::StatusInput;

    // Helper to build a StatusInput for tests
    fn make_input(
        token_usage: Option<(u64, u64)>,
        directory_label: &str,
        model_name: Option<&str>,
    ) -> StatusInput {
        StatusInput {
            token_usage,
            directory_label: directory_label.to_string(),
            model_name: model_name.map(|s| s.to_string()),
        }
    }

    // ── token_segment ──────────────────────────────────────────────────

    #[test]
    fn token_absent_returns_empty() {
        let input = make_input(None, "", None);
        assert_eq!(token_segment(&input), "");
    }

    #[test]
    fn token_under_60k_cyan() {
        let input = make_input(Some((50_000, 200_000)), "", None);
        assert_eq!(
            token_segment(&input),
            "\x1b[0;36m50.0k/200.0k\x1b[0m "
        );
    }

    #[test]
    fn token_at_60k_yellow() {
        let input = make_input(Some((60_000, 200_000)), "", None);
        assert_eq!(
            token_segment(&input),
            "\x1b[0;33m60.0k/200.0k\x1b[0m "
        );
    }

    #[test]
    fn token_75k_yellow() {
        let input = make_input(Some((75_000, 200_000)), "", None);
        assert_eq!(
            token_segment(&input),
            "\x1b[0;33m75.0k/200.0k\x1b[0m "
        );
    }

    #[test]
    fn token_at_99999_yellow() {
        let input = make_input(Some((99_999, 200_000)), "", None);
        assert!(token_segment(&input).starts_with("\x1b[0;33m"));
    }

    #[test]
    fn token_at_100k_red() {
        let input = make_input(Some((100_000, 200_000)), "", None);
        assert_eq!(
            token_segment(&input),
            "\x1b[0;31m100.0k/200.0k\x1b[0m "
        );
    }

    #[test]
    fn token_150k_red() {
        let input = make_input(Some((150_000, 200_000)), "", None);
        assert_eq!(
            token_segment(&input),
            "\x1b[0;31m150.0k/200.0k\x1b[0m "
        );
    }

    #[test]
    fn token_segment_has_trailing_space() {
        let input = make_input(Some((1_000, 200_000)), "", None);
        let result = token_segment(&input);
        assert!(result.ends_with("\x1b[0m "), "expected trailing space after reset");
    }

    // ── directory_segment ──────────────────────────────────────────────

    #[test]
    fn directory_with_label() {
        let input = make_input(None, "owner/name", None);
        assert_eq!(
            directory_segment(&input),
            "\x1b[0;34mowner/name\x1b[0m"
        );
    }

    #[test]
    fn directory_empty_label() {
        let input = make_input(None, "", None);
        assert_eq!(
            directory_segment(&input),
            "\x1b[0;34m\x1b[0m"
        );
    }

    // ── format_git_branch (helper) ─────────────────────────────────────

    #[test]
    fn git_branch_non_empty() {
        assert_eq!(
            format_git_branch("main"),
            "\x1b[0;32m (main)\x1b[0m"
        );
    }

    #[test]
    fn git_branch_empty() {
        assert_eq!(format_git_branch(""), "");
    }

    #[test]
    fn git_branch_feature_branch() {
        assert_eq!(
            format_git_branch("feature/cool-stuff"),
            "\x1b[0;32m (feature/cool-stuff)\x1b[0m"
        );
    }

    // ── format_venv (helper) ───────────────────────────────────────────

    #[test]
    fn venv_with_path() {
        assert_eq!(
            format_venv("/home/user/.venvs/myenv"),
            "\x1b[0;33m (myenv) \x1b[0m"
        );
    }

    #[test]
    fn venv_empty_path() {
        assert_eq!(format_venv(""), "");
    }

    #[test]
    fn venv_trailing_slash() {
        assert_eq!(
            format_venv("/home/user/.venvs/myenv/"),
            "\x1b[0;33m (myenv) \x1b[0m"
        );
    }

    #[test]
    fn venv_simple_name() {
        assert_eq!(
            format_venv("venv"),
            "\x1b[0;33m (venv) \x1b[0m"
        );
    }

    // ── model_segment ──────────────────────────────────────────────────

    #[test]
    fn model_with_name() {
        let input = make_input(None, "", Some("Claude Sonnet 4"));
        assert_eq!(
            model_segment(&input),
            "\x1b[0;36m [Claude Sonnet 4]\x1b[0m"
        );
    }

    #[test]
    fn model_absent() {
        let input = make_input(None, "", None);
        assert_eq!(model_segment(&input), "");
    }
}
