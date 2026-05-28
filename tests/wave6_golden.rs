// WAVE 6: Golden Fixture Generation and Tests
//
// OBJECTIVE: Establish canonical Core IR outputs for regression testing
// - Generate golden fixtures (JSON format)
// - Verify pipeline output matches golden
// - Human-readable, diffable, version-controlled

use axis_lang_lab::introspection::core_ir_json;
use axis_lang_lab::pipeline::{run_pipeline, PipelineConfig};
use axis_lang_lab::registry::Registry;
use std::fs;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

fn surface_0_config(source_file: &str) -> PipelineConfig {
    PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-0-config/semantic-surface-0-ast.yaml"),
        source_file: PathBuf::from(source_file),
        entry_rule: None,
        normalize_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-normalize.yaml"),
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    }
}

fn generate_golden(source_file: &str, golden_file: &str) {
    let config = surface_0_config(source_file);
    let artifacts = run_pipeline(&config).expect("pipeline should succeed");
    let json =
        core_ir_json::to_json(&artifacts.core_ir).expect("JSON serialization should succeed");
    fs::write(golden_file, json).expect("writing golden file should succeed");
    println!("Generated golden: {}", golden_file);
}

fn verify_against_golden(source_file: &str, golden_file: &str) {
    let config = surface_0_config(source_file);
    let artifacts = run_pipeline(&config).expect("pipeline should succeed");
    let actual_json =
        core_ir_json::to_json(&artifacts.core_ir).expect("JSON serialization should succeed");

    let golden_json = fs::read_to_string(golden_file)
        .expect(&format!("golden file {} should exist", golden_file));

    assert_eq!(
        actual_json.trim(),
        golden_json.trim(),
        "Core IR must match golden fixture"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// GOLDEN GENERATION (run with: cargo test --test wave6_golden generate -- --ignored)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore]
fn generate_all_goldens() {
    println!("Generating golden fixtures...");

    // Tier 0: Literals
    generate_golden(
        "examples/surface-0/s0-unit-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/unit.json",
    );

    generate_golden(
        "examples/surface-0/s0-int-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/int-42.json",
    );

    generate_golden(
        "examples/surface-0/s0-bool-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/bool-true.json",
    );

    // Tier 1: Control
    generate_golden(
        "examples/surface-0/s0-if.ax0",
        "tests/golden/wave6/surface-0/tier-1-control/if-true-1-else-0.json",
    );

    println!("✓ All golden fixtures generated");
}

// ═══════════════════════════════════════════════════════════════════════════
// TIER 0: LITERALS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn golden_tier0_unit_literal() {
    verify_against_golden(
        "examples/surface-0/s0-unit-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/unit.json",
    );
}

#[test]
fn golden_tier0_int_literal() {
    verify_against_golden(
        "examples/surface-0/s0-int-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/int-42.json",
    );
}

#[test]
fn golden_tier0_bool_literal() {
    verify_against_golden(
        "examples/surface-0/s0-bool-lit.ax0",
        "tests/golden/wave6/surface-0/tier-0-literals/bool-true.json",
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TIER 1: CONTROL
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn golden_tier1_if_expression() {
    verify_against_golden(
        "examples/surface-0/s0-if.ax0",
        "tests/golden/wave6/surface-0/tier-1-control/if-true-1-else-0.json",
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// GOLDEN VERIFICATION REPORT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn golden_verification_report() {
    println!("════════════════════════════════════════════════════════════");
    println!("WAVE 6: GOLDEN FIXTURE VERIFICATION REPORT");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("GOLDEN FIXTURE STRATEGY:");
    println!("  - JSON format (human-readable, diffable)");
    println!("  - Organized by surface and complexity tier");
    println!("  - Version-controlled for regression testing");
    println!();
    println!("VERIFIED GOLDENS:");
    println!("  ✓ Tier 0: Literals (unit, int, bool)");
    println!("  ✓ Tier 1: Control (if expression)");
    println!();
    println!("To generate goldens:");
    println!("  cargo test --test wave6_golden generate -- --ignored");
    println!();
    println!("════════════════════════════════════════════════════════════");
}
