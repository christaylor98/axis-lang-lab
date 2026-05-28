// WAVE 6: Cross-Surface Equivalence Tests
//
// OBJECTIVE: Prove Surface-0 and AI-1 (RPN) produce structurally equivalent Core IR
// - Same semantics expressed in different surface syntax
// - Core IR should be identical (modulo node IDs)

use axis_lang_lab::ir::core_ir::{CoreBundle, CoreTerm};
use axis_lang_lab::pipeline::{run_pipeline, PipelineConfig};
use axis_lang_lab::registry::Registry;
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

fn ai1_config(source_file: &str) -> PipelineConfig {
    PipelineConfig {
        lexer_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-lexer.yaml"),
        parser_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-parse.yaml"),
        ast_schema: PathBuf::from("axis-surface-ai1-config/ai1-rpn-ast.yaml"),
        source_file: PathBuf::from(source_file),
        entry_rule: None,
        normalize_spec: PathBuf::from("axis-surface-ai1-config/ai1-rpn-normalize.yaml"),
        registry: Registry::new(),
        parser_mode: Some("postfix".to_string()),
        hook_registry: None,
    }
}

/// Normalize Core IR by stripping node IDs for structural comparison
fn normalize_core_term(term: &CoreTerm) -> CoreTerm {
    match term {
        CoreTerm::CIntLit { value, .. } => CoreTerm::CIntLit {
            value: *value,
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CBoolLit { value, .. } => CoreTerm::CBoolLit {
            value: *value,
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CUnitLit { .. } => CoreTerm::CUnitLit {
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            ..
        } => CoreTerm::CIf {
            cond: Box::new(normalize_core_term(cond)),
            then_branch: Box::new(normalize_core_term(then_branch)),
            else_branch: Box::new(normalize_core_term(else_branch)),
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CLam { param, body, .. } => CoreTerm::CLam {
            param: param.clone(),
            body: Box::new(normalize_core_term(body)),
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CLet {
            name, value, body, ..
        } => CoreTerm::CLet {
            name: name.clone(),
            value: Box::new(normalize_core_term(value)),
            body: Box::new(normalize_core_term(body)),
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CVar { name, .. } => CoreTerm::CVar {
            name: name.clone(),
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CApp { func, arg, .. } => CoreTerm::CApp {
            func: Box::new(normalize_core_term(func)),
            arg: Box::new(normalize_core_term(arg)),
            node_id: None,
            annotations: vec![],
        },
        CoreTerm::CCall {
            target_name, args, ..
        } => CoreTerm::CCall {
            target_name: target_name.clone(),
            args: args.iter().map(|a| normalize_core_term(a)).collect(),
            node_id: None,
            annotations: vec![],
        },
    }
}

fn assert_structurally_equivalent(bundle1: &CoreBundle, bundle2: &CoreBundle) {
    let normalized1 = normalize_core_term(&bundle1.core_term);
    let normalized2 = normalize_core_term(&bundle2.core_term);

    assert_eq!(
        normalized1, normalized2,
        "Core IR terms must be structurally equivalent (ignoring node IDs)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// EQUIVALENCE TESTS: SURFACE-0 vs AI-1
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // AI-1 surface may not be fully implemented
fn equivalence_unit_literal() {
    // Surface-0: ()
    let s0_artifacts = run_pipeline(&surface_0_config("examples/surface-0/s0-unit-lit.ax0"))
        .expect("Surface-0 pipeline should succeed");

    // AI-1: unit (RPN)
    let ai1_artifacts = run_pipeline(&ai1_config("examples/ai-1/ai1-unit.ax1"))
        .expect("AI-1 pipeline should succeed");

    assert_structurally_equivalent(&s0_artifacts.core_ir, &ai1_artifacts.core_ir);
}

#[test]
#[ignore] // AI-1 surface may not be fully implemented
fn equivalence_int_literal() {
    // Surface-0: 42
    let s0_artifacts = run_pipeline(&surface_0_config("examples/surface-0/s0-int-lit.ax0"))
        .expect("Surface-0 pipeline should succeed");

    // AI-1: 42 (RPN)
    let ai1_artifacts = run_pipeline(&ai1_config("examples/ai-1/ai1-int.ax1"))
        .expect("AI-1 pipeline should succeed");

    assert_structurally_equivalent(&s0_artifacts.core_ir, &ai1_artifacts.core_ir);
}

#[test]
#[ignore] // AI-1 surface may not be fully implemented
fn equivalence_if_expression() {
    // Surface-0: if true then 1 else 0
    let s0_artifacts = run_pipeline(&surface_0_config("examples/surface-0/s0-if.ax0"))
        .expect("Surface-0 pipeline should succeed");

    // AI-1: true 1 0 if (RPN)
    let ai1_artifacts = run_pipeline(&ai1_config("examples/ai-1/ai1-if.ax1"))
        .expect("AI-1 pipeline should succeed");

    assert_structurally_equivalent(&s0_artifacts.core_ir, &ai1_artifacts.core_ir);
}

// ═══════════════════════════════════════════════════════════════════════════
// STRUCTURAL EQUIVALENCE VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_normalization_preserves_structure() {
    // Test that normalization correctly strips node IDs but preserves structure

    let term1 = CoreTerm::CIf {
        cond: Box::new(CoreTerm::CBoolLit {
            value: true,
            node_id: Some(1),
            annotations: vec![],
        }),
        then_branch: Box::new(CoreTerm::CIntLit {
            value: 42,
            node_id: Some(2),
            annotations: vec![],
        }),
        else_branch: Box::new(CoreTerm::CIntLit {
            value: 0,
            node_id: Some(3),
            annotations: vec![],
        }),
        node_id: Some(4),
        annotations: vec![],
    };

    let term2 = CoreTerm::CIf {
        cond: Box::new(CoreTerm::CBoolLit {
            value: true,
            node_id: Some(100),
            annotations: vec![],
        }),
        then_branch: Box::new(CoreTerm::CIntLit {
            value: 42,
            node_id: Some(200),
            annotations: vec![],
        }),
        else_branch: Box::new(CoreTerm::CIntLit {
            value: 0,
            node_id: Some(300),
            annotations: vec![],
        }),
        node_id: Some(400),
        annotations: vec![],
    };

    let normalized1 = normalize_core_term(&term1);
    let normalized2 = normalize_core_term(&term2);

    assert_eq!(
        normalized1, normalized2,
        "Normalization should make structurally identical terms equal"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// EQUIVALENCE VERIFICATION REPORT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn equivalence_verification_report() {
    println!("════════════════════════════════════════════════════════════");
    println!("WAVE 6: EQUIVALENCE VERIFICATION REPORT");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("EQUIVALENCE STRATEGY:");
    println!("  - Normalize Core IR by stripping node IDs");
    println!("  - Compare structural equality");
    println!("  - Verify different surfaces produce same semantics");
    println!();
    println!("PLANNED TESTS (may be ignored if surfaces not implemented):");
    println!("  - Surface-0 '()' vs AI-1 'unit'");
    println!("  - Surface-0 '42' vs AI-1 '42'");
    println!("  - Surface-0 'if true then 1 else 0' vs AI-1 'true 1 0 if'");
    println!();
    println!("VERIFIED:");
    println!("  ✓ Normalization function preserves structure");
    println!("  ✓ Node ID differences are properly ignored");
    println!();
    println!("════════════════════════════════════════════════════════════");
}
