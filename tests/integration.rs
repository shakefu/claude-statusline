use std::process::Command;

fn run_binary(input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-statusline"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        // Clear VIRTUAL_ENV so it doesn't leak into tests
        .env_remove("VIRTUAL_ENV")
        .spawn()
        .expect("failed to spawn binary");

    use std::io::Write;
    if !input.is_empty() {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    // Close stdin by dropping it
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ── Input spec ─────────────────────────────────────────────────────────

#[test]
fn empty_stdin_exits_0_prints_nothing() {
    let (code, stdout, stderr) = run_binary("");
    assert_eq!(code, 0);
    assert!(stdout.is_empty(), "expected no stdout, got: {stdout:?}");
    assert!(stderr.is_empty(), "expected no stderr, got: {stderr:?}");
}

#[test]
fn malformed_json_exits_0_prints_nothing() {
    let (code, stdout, stderr) = run_binary("{not json!}");
    assert_eq!(code, 0);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

// ── Output spec: no trailing newline ───────────────────────────────────

#[test]
fn output_has_no_trailing_newline() {
    let json = r#"{"model":{"display_name":"Test"}}"#;
    let (code, stdout, _) = run_binary(json);
    assert_eq!(code, 0);
    assert!(
        !stdout.ends_with('\n'),
        "output must not end with newline: {stdout:?}"
    );
}

// ── Output spec: nothing on stderr ─────────────────────────────────────

#[test]
fn no_stderr_output() {
    let json = r#"{"context_window":{"total_input_tokens":1000,"context_window_size":200000},"workspace":{"repo":{"owner":"acme","name":"widgets"}},"model":{"display_name":"Claude Sonnet 4"}}"#;
    let (code, _, stderr) = run_binary(json);
    assert_eq!(code, 0);
    assert!(stderr.is_empty(), "expected no stderr, got: {stderr:?}");
}

// ── Full integration: exact byte output ────────────────────────────────

#[test]
fn full_json_exact_output() {
    let json = r#"{"context_window":{"total_input_tokens":50000,"context_window_size":200000},"workspace":{"repo":{"owner":"acme","name":"widgets"}},"model":{"display_name":"Claude Sonnet 4"}}"#;
    let (code, stdout, _) = run_binary(json);
    assert_eq!(code, 0);

    // The output should contain these segments in order:
    // 1. Token segment: cyan "50.0k/200.0k" + reset + space
    // 2. Directory segment: blue "acme/widgets" + reset
    // 3. Git branch segment: depends on actual git state (running in a git repo)
    // 4. Venv segment: empty (VIRTUAL_ENV cleared)
    // 5. Model segment: cyan " [Claude Sonnet 4]" + reset

    // Check token segment at start
    assert!(
        stdout.starts_with("\x1b[0;36m50.0k/200.0k\x1b[0m "),
        "token segment mismatch: {stdout:?}"
    );

    // Check directory segment follows
    assert!(
        stdout.contains("\x1b[0;34macme/widgets\x1b[0m"),
        "directory segment mismatch: {stdout:?}"
    );

    // Check model segment at end
    assert!(
        stdout.ends_with("\x1b[0;36m [Claude Sonnet 4]\x1b[0m"),
        "model segment mismatch: {stdout:?}"
    );
}

// ── Token color thresholds ─────────────────────────────────────────────

#[test]
fn tokens_under_60k_cyan() {
    let json = r#"{"context_window":{"total_input_tokens":59999,"context_window_size":200000}}"#;
    let (_, stdout, _) = run_binary(json);
    assert!(
        stdout.starts_with("\x1b[0;36m"),
        "expected cyan for <60k: {stdout:?}"
    );
}

#[test]
fn tokens_at_60k_yellow() {
    let json = r#"{"context_window":{"total_input_tokens":60000,"context_window_size":200000}}"#;
    let (_, stdout, _) = run_binary(json);
    assert!(
        stdout.starts_with("\x1b[0;33m"),
        "expected yellow for >=60k: {stdout:?}"
    );
}

#[test]
fn tokens_at_100k_red() {
    let json = r#"{"context_window":{"total_input_tokens":100000,"context_window_size":200000}}"#;
    let (_, stdout, _) = run_binary(json);
    assert!(
        stdout.starts_with("\x1b[0;31m"),
        "expected red for >=100k: {stdout:?}"
    );
}

// ── Missing optional fields produce correct fallbacks ──────────────────

#[test]
fn no_context_window_no_token_segment() {
    let json = r#"{"workspace":{"repo":{"owner":"a","name":"b"}},"model":{"display_name":"M"}}"#;
    let (_, stdout, _) = run_binary(json);
    // Should start directly with the directory segment (blue), not token segment
    assert!(
        stdout.starts_with("\x1b[0;34m"),
        "expected directory (blue) at start when no tokens: {stdout:?}"
    );
}

#[test]
fn workspace_fallback_to_current_dir_basename() {
    let json = r#"{"workspace":{"current_dir":"/home/user/myproject"}}"#;
    let (_, stdout, _) = run_binary(json);
    assert!(
        stdout.contains("\x1b[0;34mmyproject\x1b[0m"),
        "expected basename of current_dir: {stdout:?}"
    );
}

#[test]
fn minimal_json_empty_object() {
    let json = r#"{}"#;
    let (code, stdout, stderr) = run_binary(json);
    assert_eq!(code, 0);
    // Should have at least the directory segment (even if empty)
    assert!(
        stdout.contains("\x1b[0;34m\x1b[0m"),
        "expected empty directory segment: {stdout:?}"
    );
    assert!(stderr.is_empty());
}
