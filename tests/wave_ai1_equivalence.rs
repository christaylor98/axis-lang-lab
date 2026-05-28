// Wave AI1 Core IR Equivalence Tests
//
// These tests verify that AI-1 (RPN/postfix) surface produces
// semantically equivalent Core IR to Semantic-Surface-0.
//
// Test strategy:
// 1. Compile a Semantic-Surface-0 program to Core IR
// 2. Compile an AI-1 program (semantically equivalent) to Core IR
// 3. Assert Core IR structural equivalence

use axis_lang_lab::ir::core_ir::CoreBundle;
use axis_lang_lab::pipeline::{run_pipeline, PipelineConfig};
use axis_lang_lab::registry::Registry;
use std::path::PathBuf;

fn run_surface_0(source_file: &str) -> CoreBundle {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-0-config/semantic-surface-0-ast.yaml"),
        source_file: PathBuf::from(source_file),
        entry_rule: None,
        normalize_spec: PathBuf::from("axis-surface-0-config/semantic-surface-0-normalize.yaml"),
        registry: Registry::new(),
        parser_mode: None, // grammar-based parsing
        hook_registry: None,
    };

    let artifacts = run_pipeline(&config).expect("Surface-0 pipeline should succeed");

    artifacts.core_ir
}

fn run_ai1(source_file: &str) -> CoreBundle {
    let config = PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-ai1-config/ai1-rpn-ast.yaml"),
        source_file: PathBuf::from(source_file),
        entry_rule: None,
        normalize_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-normalize.yaml"),
        registry: Registry::new(),
        parser_mode: Some("postfix".to_string()), // postfix parsing for AI-1
        hook_registry: None,
    };

    let artifacts = run_pipeline(&config).expect("AI-1 pipeline should succeed");

    artifacts.core_ir
}

fn assert_core_ir_equivalent(surface0_ir: &CoreBundle, ai1_ir: &CoreBundle) {
    // Structural equality check (Core IR should be deterministic)
    assert_eq!(
        format!("{:?}", surface0_ir),
        format!("{:?}", ai1_ir),
        "Core IR from Surface-0 and AI-1 must be structurally equivalent"
    );
}

#[test]
fn test_equivalence_int_literal() {
    // Surface-0: 42
    // AI-1: 42
    let s0_ir = run_surface_0("examples/surface-0/s0-simple-int.ax0");
    let ai1_ir = run_ai1("examples/ai1/ai1-simple-int.ai1");

    assert_core_ir_equivalent(&s0_ir, &ai1_ir);
}

#[test]
fn test_equivalence_if_expression() {
    // Surface-0: if true then 1 else 0
    // AI-1: true 1 0 if
    let s0_ir = run_surface_0("examples/surface-0/s0-if.ax0");
    let ai1_ir = run_ai1("examples/ai1/ai1-if.ai1");

    assert_core_ir_equivalent(&s0_ir, &ai1_ir);
}

#[test]
#[ignore = "AppExpr schema cannot handle variable-length application chains with current grammar - terminals interleaved with nodes"]
fn test_equivalence_app_chain() {
    // Surface-0: let f = fn x => fn y => x in f(1)(2)
    // AI-1: x lam y lam x f 1 app 2 app let f
    let s0_ir = run_surface_0("examples/surface-0/semantic-s0-test.ax0");
    let ai1_ir = run_ai1("examples/ai1/ai1-app-chain.ai1");

    assert_core_ir_equivalent(&s0_ir, &ai1_ir);
}
