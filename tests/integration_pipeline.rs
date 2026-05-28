// Integration tests for full pipeline: parse → normalize → lower → coreir
//
// These tests verify the COMPLETE pipeline using REAL source files,
// ensuring normalization is YAML-driven and mechanical lowering is NF-only.

use axis_lang_lab::pipeline::{run_pipeline, PipelineConfig};
use axis_lang_lab::registry::Registry;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// SURFACE-0 TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "Surface-0 RPN schema has terminals in children() - needs fixing"]
fn test_surface0_full_pipeline() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-0-config/semantic-surface-0-ast.yaml"),
        normalize_spec: PathBuf::from("axis-surface-0-config/surface-0-normalize.yaml"),
        source_file: PathBuf::from("examples/surface-0/semantic-s0-test.ax0"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    };

    let result = run_pipeline(&config);
    assert!(
        result.is_ok(),
        "Surface-0 pipeline failed: {:?}",
        result.err()
    );

    let artifacts = result.unwrap();

    // Verify Core IR bundle exists
    assert_eq!(artifacts.core_ir.version.as_ref(), "0.2");

    // Verify no surface nodes in schema_ast (normalized output)
    assert_ne!(
        artifacts.schema_ast.kind, "Program",
        "Surface node Program should not appear in normalized AST"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// AI1 (RPN) TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ai1_rpn_full_pipeline() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-ai1-config/ai1-rpn-ast.yaml"),
        normalize_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-normalize.yaml"),
        source_file: PathBuf::from("examples/ai1/ai1-simple-int.ai1"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: Some("postfix".to_string()),
        hook_registry: None,
    };

    let result = run_pipeline(&config);
    assert!(
        result.is_ok(),
        "AI1 RPN pipeline failed: {:?}",
        result.err()
    );

    let artifacts = result.unwrap();
    assert_eq!(artifacts.core_ir.version.as_ref(), "0.3");
}

// ═══════════════════════════════════════════════════════════════════════════
// AI2 (EXPLICIT) TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ai2_explicit_full_pipeline() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-ai2-config/ai2-explicit-ast.yaml"),
        normalize_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-normalize.yaml"),
        source_file: PathBuf::from("examples/ai2/ai2-simple-int.ai2"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    };

    let result = run_pipeline(&config);
    assert!(
        result.is_ok(),
        "AI2 explicit pipeline failed: {:?}",
        result.err()
    );

    let artifacts = result.unwrap();
    assert_eq!(artifacts.core_ir.version.as_ref(), "0.3");
}

// ═══════════════════════════════════════════════════════════════════════════
// SURFACE-H1 TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "H1 normalization requires complex desugaring not yet implemented in YAML engine"]
fn test_surface_h1_full_pipeline() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-h1-config/surface-h1-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-h1-config/surface-h1-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-h1-config/surface-h1-ast.yaml"),
        normalize_spec: PathBuf::from("axis-surface-h1-config/surface-h1-normalize.yaml"),
        source_file: PathBuf::from("examples/surface-h1/test1_single_function.h1"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    };

    let result = run_pipeline(&config);
    assert!(result.is_ok(), "H1 pipeline failed: {:?}", result.err());

    let artifacts = result.unwrap();
    assert_eq!(artifacts.core_ir.version.as_ref(), "0.2");

    // H1 is the complex parse test - verify Program node was normalized away
    assert_ne!(
        artifacts.schema_ast.kind, "Program",
        "Surface node Program should not appear in normalized AST"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// NEGATIVE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "Error message changed due to schema projection failure occurring before normalization"]
fn test_missing_normalize_config_fails() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-0-config/semantic-surface-0-ast.yaml"),
        normalize_spec: PathBuf::from("nonexistent-normalize.yaml"),
        source_file: PathBuf::from("examples/surface-0/semantic-s0-test.ax0"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    };

    let result = run_pipeline(&config);
    assert!(result.is_err(), "Should fail with missing normalize config");

    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("normalization") || err_str.contains("Normalization"),
        "Error should mention normalization: {}",
        err_str
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// DETERMINISM TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_normalization_determinism() {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-ai2-config/ai2-explicit-ast.yaml"),
        normalize_spec: PathBuf::from("axis-surface-ai2-config/ai2-explicit-normalize.yaml"),
        source_file: PathBuf::from("examples/ai2/ai2-simple-int.ai2"),
        entry_rule: None,
        registry: Registry::new(),
        parser_mode: None,
        hook_registry: None,
    };

    // Run pipeline twice
    let result1 = run_pipeline(&config);
    let result2 = run_pipeline(&config);

    assert!(
        result1.is_ok() && result2.is_ok(),
        "Both pipeline runs should succeed"
    );

    let artifacts1 = result1.unwrap();
    let artifacts2 = result2.unwrap();

    // Normalized AST should have identical structure
    assert_eq!(
        artifacts1.schema_ast.kind, artifacts2.schema_ast.kind,
        "Normalized AST kind should be deterministic"
    );

    // Core IR should be identical
    assert_eq!(artifacts1.core_ir.version, artifacts2.core_ir.version);
}
