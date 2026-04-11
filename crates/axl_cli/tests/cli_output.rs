use std::path::PathBuf;
use std::process::Command;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/invalid_a11y.tsx")
        .canonicalize()
        .expect("fixture path should resolve")
}

#[test]
fn text_output_contains_expected_rules() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path())
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
        .arg(fixture_path())
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
        .arg(fixture_path())
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
