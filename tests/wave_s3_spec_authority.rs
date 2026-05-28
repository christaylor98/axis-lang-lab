// Wave S3: Lowering Spec Authority Integration Tests
//
// OBJECTIVE: Verify that lowering spec is MANDATORY and enforced uniformly
// across debug and release builds.
//
// HARD CONSTRAINTS:
// - No fallback behavior
// - Missing spec MUST hard-fail
// - Invalid spec MUST hard-fail
// - Debug and release MUST behave identically

use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

fn get_binary_path() -> PathBuf {
    // Use debug binary by default, or release if CARGO_PROFILE=release
    let profile = std::env::var("CARGO_PROFILE").unwrap_or_else(|_| "debug".to_string());
    PathBuf::from(format!("target/{}/axis", profile))
}

fn ensure_binary_exists() -> PathBuf {
    let binary = get_binary_path();
    if !binary.exists() {
        panic!(
            "Binary not found at {:?}. Run: cargo build{}",
            binary,
            if std::env::var("CARGO_PROFILE").is_ok() {
                " --release"
            } else {
                ""
            }
        );
    }
    binary
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 1: Missing --lowering flag MUST fail
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_missing_lowering_flag_fails() {
    let binary = ensure_binary_exists();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        // NOTE: --lowering flag OMITTED
        .output()
        .expect("failed to execute binary");

    // MUST fail (non-zero exit code)
    assert!(
        !output.status.success(),
        "Expected failure when --lowering flag omitted, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention missing required argument
    assert!(
        stderr.contains("required") || stderr.contains("lowering"),
        "Error message should mention missing required --lowering flag. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 2: Non-existent lowering spec file MUST fail with spec-load error
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_missing_spec_file_fails() {
    let binary = ensure_binary_exists();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg("non-existent-spec.yaml") // File does not exist
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .output()
        .expect("failed to execute binary");

    // MUST fail
    assert!(
        !output.status.success(),
        "Expected failure when lowering spec file missing, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention file not found
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "Error message should mention file not found. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 3: Invalid lowering spec (bad YAML) MUST fail with parse error
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_invalid_yaml_spec_fails() {
    let binary = ensure_binary_exists();

    // Create invalid YAML spec
    let bad_spec_path = PathBuf::from("target/test_invalid_spec.yaml");
    fs::write(&bad_spec_path, "this is not valid yaml: ][}{").unwrap();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg(&bad_spec_path)
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .output()
        .expect("failed to execute binary");

    // Cleanup
    let _ = fs::remove_file(&bad_spec_path);

    // MUST fail
    assert!(
        !output.status.success(),
        "Expected failure when lowering spec has invalid YAML, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention parse error
    assert!(
        stderr.contains("parse") || stderr.contains("invalid"),
        "Error message should mention parse failure. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 4: Invalid lowering spec (wrong version) MUST fail with validation error
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_wrong_version_spec_fails() {
    let binary = ensure_binary_exists();

    // Create spec with wrong version
    let bad_spec_content = r#"
version: "99.0"
target_ir: "0.2"
rules: {}
"#;
    let bad_spec_path = PathBuf::from("target/test_wrong_version_spec.yaml");
    fs::write(&bad_spec_path, bad_spec_content).unwrap();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg(&bad_spec_path)
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .output()
        .expect("failed to execute binary");

    // Cleanup
    let _ = fs::remove_file(&bad_spec_path);

    // MUST fail
    assert!(
        !output.status.success(),
        "Expected failure when lowering spec has wrong version, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention version mismatch
    assert!(
        stderr.contains("version") || stderr.contains("unsupported"),
        "Error message should mention version mismatch. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 5: Invalid lowering spec (wrong target IR) MUST fail
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_wrong_target_ir_spec_fails() {
    let binary = ensure_binary_exists();

    // Create spec with wrong target IR
    let bad_spec_content = r#"
version: "0.1"
target_ir: "99.0"
rules: {}
"#;
    let bad_spec_path = PathBuf::from("target/test_wrong_ir_spec.yaml");
    fs::write(&bad_spec_path, bad_spec_content).unwrap();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg(&bad_spec_path)
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .output()
        .expect("failed to execute binary");

    // Cleanup
    let _ = fs::remove_file(&bad_spec_path);

    // MUST fail
    assert!(
        !output.status.success(),
        "Expected failure when lowering spec has wrong target IR, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention IR version mismatch
    assert!(
        stderr.contains("target IR")
            || stderr.contains("IR version")
            || stderr.contains("unsupported"),
        "Error message should mention target IR mismatch. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 6: Empty lowering spec (no rules) MUST fail
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_empty_spec_fails() {
    let binary = ensure_binary_exists();

    // Create spec with no rules
    let bad_spec_content = r#"
version: "0.1"
target_ir: "0.2"
rules: {}
"#;
    let bad_spec_path = PathBuf::from("target/test_empty_spec.yaml");
    fs::write(&bad_spec_path, bad_spec_content).unwrap();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg(&bad_spec_path)
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .output()
        .expect("failed to execute binary");

    // Cleanup
    let _ = fs::remove_file(&bad_spec_path);

    // MUST fail
    assert!(
        !output.status.success(),
        "Expected failure when lowering spec has no rules, but command succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);

    // MUST mention empty or no rules
    assert!(
        stderr.contains("no rules") || stderr.contains("empty"),
        "Error message should mention empty spec. Got: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 7: Valid spec with --inspect lowering MUST show spec metadata
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_inspect_lowering_shows_spec_metadata() {
    let binary = ensure_binary_exists();

    let output = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg("axis-surface-0-config/semantic-surface-0-lowering.yaml")
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .arg("--inspect")
        .arg("lowering")
        .output()
        .expect("failed to execute binary");

    // MUST succeed
    assert!(
        output.status.success(),
        "Expected success with valid spec and --inspect lowering. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // MUST show spec metadata
    assert!(
        stdout.contains("LOWERING SPEC INSPECTION") || stdout.contains("content_hash"),
        "Expected lowering spec inspection output with metadata. Got: {}",
        stdout
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 8: Valid spec MUST lower deterministically
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --test wave_s3_spec_authority -- --ignored
fn test_valid_spec_lowers_deterministically() {
    let binary = ensure_binary_exists();

    // Run twice with same inputs
    let output1 = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg("axis-surface-0-config/semantic-surface-0-lowering.yaml")
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .arg("--inspect")
        .arg("core-ir")
        .output()
        .expect("failed to execute binary");

    let output2 = Command::new(&binary)
        .arg("run")
        .arg("--lexer")
        .arg("axis-surface-0-config/semantic-surface-0-lexer.yaml")
        .arg("--parser")
        .arg("axis-surface-0-config/semantic-surface-0-parse.yaml")
        .arg("--schema")
        .arg("axis-surface-0-config/semantic-surface-0-ast.yaml")
        .arg("--lowering")
        .arg("axis-surface-0-config/semantic-surface-0-lowering.yaml")
        .arg("--file")
        .arg("examples/surface-0/semantic-s0-test.ax0")
        .arg("--inspect")
        .arg("core-ir")
        .output()
        .expect("failed to execute binary");

    // Both MUST succeed
    assert!(
        output1.status.success(),
        "First run failed. Stderr: {}",
        String::from_utf8_lossy(&output1.stderr)
    );
    assert!(
        output2.status.success(),
        "Second run failed. Stderr: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Outputs MUST be identical
    assert_eq!(
        output1.stdout, output2.stdout,
        "Lowering is not deterministic: outputs differ between runs"
    );
}
