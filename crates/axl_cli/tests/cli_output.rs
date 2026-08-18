use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../tests/fixtures/{name}"))
        .canonicalize()
        .expect("fixture path should resolve")
}

fn temp_fixture_copy(name: &str) -> PathBuf {
    let source = fixture_path(name);
    let path = std::env::temp_dir().join(format!(
        "axl_cli_fixture_{}_{}.tsx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ));
    fs::copy(&source, &path).expect("fixture copy should succeed");
    path
}

#[test]
fn text_output_contains_expected_rules() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("text")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roles/abstract-role-usage"));
    assert!(stdout.contains("naming/empty-accessible-name"));
    assert!(stdout.contains("focus/tabindex-positive"));
    assert!(stdout.contains("  error  "));
}

#[test]
fn json_output_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(fixture_path("invalid_a11y.tsx"))
        .arg("--format")
        .arg("json")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
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

    assert_eq!(output.status.code(), Some(1));
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

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roles/abstract-role-usage"));
    assert!(stdout.contains("focus/tabindex-positive"));
    assert!(!stdout.contains("naming/empty-accessible-name"));
}

#[test]
fn fix_flag_rewrites_positive_tabindex_and_reduces_problems() {
    let path = temp_fixture_copy("invalid_a11y.tsx");
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(&path)
        .arg("--format")
        .arg("text")
        .arg("--fix")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 problems (2 errors, 0 warnings)."));
    assert!(stdout.contains("Autofix applied 1 changes across 1 files."));

    let updated = fs::read_to_string(&path).expect("updated fixture should be readable");
    assert!(updated.contains("tabIndex={0}"));

    let _ = fs::remove_file(path);
}

#[test]
fn fix_flag_reports_no_changes_for_valid_fixture() {
    let path = temp_fixture_copy("valid_a11y.tsx");
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(&path)
        .arg("--format")
        .arg("text")
        .arg("--fix")
        .output()
        .expect("axl_cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 problems (0 errors, 0 warnings)."));
    assert!(stdout.contains("Autofix applied 0 changes across 0 files."));

    let _ = fs::remove_file(path);
}

#[test]
fn fix_flag_with_json_output_keeps_json_machine_readable() {
    let path = temp_fixture_copy("invalid_a11y.tsx");
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(&path)
        .arg("--format")
        .arg("json")
        .arg("--fix")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("json should parse");
    let array = value.as_array().expect("top-level must be an array");
    assert_eq!(array.len(), 2);

    let _ = fs::remove_file(path);
}

#[test]
fn fix_flag_with_multiple_inputs_reports_aggregate_summary() {
    let invalid = temp_fixture_copy("invalid_a11y.tsx");
    let valid = temp_fixture_copy("valid_a11y.tsx");
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(&invalid)
        .arg(&valid)
        .arg("--format")
        .arg("text")
        .arg("--fix")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 files checked. 2 problems (2 errors, 0 warnings)."));
    assert!(stdout.contains("Autofix applied 1 changes across 1 files."));

    let _ = fs::remove_file(invalid);
    let _ = fs::remove_file(valid);
}

#[test]
fn fix_dry_run_does_not_rewrite_files() {
    let path = temp_fixture_copy("invalid_a11y.tsx");
    let original = fs::read_to_string(&path).expect("original fixture should be readable");
    let output = Command::new(env!("CARGO_BIN_EXE_axl_cli"))
        .arg(&path)
        .arg("--format")
        .arg("text")
        .arg("--fix-dry-run")
        .output()
        .expect("axl_cli should run");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Autofix would apply 1 changes across 1 files."));
    assert!(stdout.contains("focus/tabindex-positive"));

    let unchanged = fs::read_to_string(&path).expect("fixture should remain readable");
    assert_eq!(unchanged, original);

    let _ = fs::remove_file(path);
}
