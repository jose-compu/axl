use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../tests/fixtures/{name}"))
        .canonicalize()
        .expect("fixture path should resolve")
}

#[test]
fn text_output_contains_expected_rules() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("text")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roles/abstract-role-usage"));
    assert!(stdout.contains("naming/empty-accessible-name"));
    assert!(stdout.contains("focus/tabindex-positive"));
}

#[test]
fn json_output_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("json")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("json should parse");
    let array = value.as_array().expect("top-level must be an array");
    assert_eq!(array.len(), 3);
}

#[test]
fn only_filter_limits_categories() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("text")
        .arg("--only")
        .arg("roles")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roles/abstract-role-usage"));
    assert!(!stdout.contains("naming/empty-accessible-name"));
    assert!(!stdout.contains("focus/tabindex-positive"));
}

#[test]
fn text_output_reports_zero_problems_for_valid_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("valid_a11y.tsx"))
        .arg("--format")
        .arg("text")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 problems (0 errors, 0 warnings)."));
}

#[test]
fn only_filter_accepts_multiple_categories() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("text")
        .arg("--only")
        .arg("roles,focus")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roles/abstract-role-usage"));
    assert!(stdout.contains("focus/tabindex-positive"));
    assert!(!stdout.contains("naming/empty-accessible-name"));
}
