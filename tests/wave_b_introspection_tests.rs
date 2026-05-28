// Wave B: Integration Tests
//
// These tests verify the entire introspection subsystem:
// - Provenance annotation attachment
// - Trace graph construction
// - Introspection queries
// - Trust invariants

use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::introspection::provenance::{self, ProvenanceAnnotationSet};
use axis_lang_lab::introspection::trace_graph::{TraceGraphBuilder, TraceLinkKind};
use axis_lang_lab::introspection::trust::{
    verify_annotation_namespaces, verify_annotations_ignored_structurally,
    verify_trace_determinism, verify_trace_invariants,
};
use axis_lang_lab::introspection::*;
use axis_lang_lab::ir::core_ir::{Annotation, AnnotationValue, CoreTerm};

#[test]
fn test_end_to_end_traceability() {
    // Build a complete trace: Token → Generic AST → Schema AST → Core IR
    let mut builder = TraceGraphBuilder::new();

    // Token layer
    let mut token_annotations = ProvenanceAnnotationSet::new();
    token_annotations.add(provenance::source::file("test.ax"));
    token_annotations.add(provenance::source::text("unit"));

    let token = Token::new(TokenKind::UnitLit, "()".to_string(), Span::new(0, 2));
    let token_id = builder.add_token(token, token_annotations);

    // Generic AST layer
    let mut ast_annotations = ProvenanceAnnotationSet::new();
    ast_annotations.add(provenance::pipeline::phase("parsing"));

    let ast_id = builder.add_generic_ast(
        "unit_lit".to_string(),
        1,
        Some(Span::new(0, 2)),
        ast_annotations,
        vec![token_id],
    );

    builder.add_link(token_id, ast_id, TraceLinkKind::TokenToGenericAst);

    // Schema AST layer
    let mut schema_annotations = ProvenanceAnnotationSet::new();
    schema_annotations.add(provenance::schema::node("UnitLit"));
    schema_annotations.add(provenance::schema::rule("literal_expr"));
    schema_annotations.add(provenance::pipeline::phase("schema_projection"));

    let schema_id = builder.add_schema_ast(
        "UnitLit".to_string(),
        1,
        Some(Span::new(0, 2)),
        schema_annotations,
        vec![ast_id],
    );

    builder.add_link(ast_id, schema_id, TraceLinkKind::GenericAstToSchemaAst);

    // Core IR layer
    let mut core_ir_annotations = ProvenanceAnnotationSet::new();
    core_ir_annotations.add(provenance::lowering::rule("lower_unit_lit"));
    core_ir_annotations.add(provenance::lowering::phase("wave5"));
    core_ir_annotations.add(provenance::pipeline::phase("lowering"));

    let core_ir_id = builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(42),
        Some(Span::new(0, 2)),
        core_ir_annotations,
        vec![schema_id],
    );

    builder.add_link(schema_id, core_ir_id, TraceLinkKind::SchemaAstToCoreIr);

    // Build and verify
    let graph = builder.build();

    // Verify invariants
    assert!(verify_trace_invariants(&graph).is_ok());

    // Create inspector
    let inspector = Inspector::new(graph);

    // Query origin
    let origin = inspector.origin(42);
    assert!(origin.is_some());
    let origin = origin.unwrap();
    assert_eq!(origin.source_file, Some("test.ax".to_string()));
    assert_eq!(origin.source_spans.len(), 1);
    assert_eq!(origin.source_spans[0].start, 0);
    assert_eq!(origin.source_spans[0].end, 2);

    // Query schema rule
    let schema_info = inspector.schema_rule(42);
    assert!(schema_info.is_some());
    let schema_info = schema_info.unwrap();
    assert_eq!(schema_info.node_kind, "UnitLit");
    assert_eq!(schema_info.rule_name, Some("literal_expr".to_string()));

    // Query lowering rule
    let lowering_info = inspector.lowering_rule(42);
    assert!(lowering_info.is_some());
    let lowering_info = lowering_info.unwrap();
    assert_eq!(lowering_info.rule_name, Some("lower_unit_lit".to_string()));
    assert_eq!(lowering_info.phase, Some("wave5".to_string()));

    // Query annotations
    let ann_info = inspector.annotations(42);
    assert!(ann_info.is_some());
    let ann_info = ann_info.unwrap();
    assert!(ann_info.provenance.len() > 0);

    // Get stats
    let stats = inspector.stats();
    assert_eq!(stats.token_count, 1);
    assert_eq!(stats.generic_ast_count, 1);
    assert_eq!(stats.schema_ast_count, 1);
    assert_eq!(stats.core_ir_count, 1);

    // Explain
    let explanation = inspector.explain(42);
    assert!(explanation.is_some());
    let explanation = explanation.unwrap();
    assert!(explanation.contains("Core IR Node #42"));
    assert!(explanation.contains("UnitLit"));
    assert!(explanation.contains("lower_unit_lit"));
}

#[test]
fn test_provenance_namespace_isolation() {
    // Verify that user annotations can't use reserved namespaces

    let user_annotations = vec![
        Annotation {
            key: "user.custom".to_string(),
            value: AnnotationValue::String("ok".to_string()),
        },
        Annotation {
            key: "my_annotation".to_string(),
            value: AnnotationValue::Bool(true),
        },
    ];

    assert!(verify_annotation_namespaces(&user_annotations).is_ok());

    let invalid_annotations = vec![Annotation {
        key: "source.span".to_string(), // Reserved!
        value: AnnotationValue::String("0..10".to_string()),
    }];

    let result = verify_annotation_namespaces(&invalid_annotations);
    assert!(result.is_err());
}

#[test]
fn test_deterministic_trace_construction() {
    // Build the same trace twice and verify they're identical

    let build_trace = || {
        let mut builder = TraceGraphBuilder::new();

        let token1 = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(0, 3));
        let token2 = Token::new(TokenKind::Ident, "bar".to_string(), Span::new(4, 7));

        let token_id1 = builder.add_token(token1, ProvenanceAnnotationSet::new());
        let token_id2 = builder.add_token(token2, ProvenanceAnnotationSet::new());

        let ast_id = builder.add_generic_ast(
            "call".to_string(),
            2,
            Some(Span::new(0, 7)),
            ProvenanceAnnotationSet::new(),
            vec![token_id1, token_id2],
        );

        builder.add_core_ir(
            "CCall".to_string(),
            Some(1),
            Some(Span::new(0, 7)),
            ProvenanceAnnotationSet::new(),
            vec![ast_id],
        );

        builder.build()
    };

    let trace1 = build_trace();
    let trace2 = build_trace();

    // Verify determinism
    assert!(verify_trace_determinism(&trace1, &trace2).is_ok());
}

#[test]
fn test_annotation_independence_from_execution() {
    // Verify that annotations don't affect Core IR structure

    let term_without_annotations = CoreTerm::CUnitLit {
        node_id: Some(1),
        annotations: Vec::new(),
    };

    let term_with_annotations = CoreTerm::CUnitLit {
        node_id: Some(1),
        annotations: vec![
            Annotation {
                key: "source.file".to_string(),
                value: AnnotationValue::String("test.ax".to_string()),
            },
            Annotation {
                key: "schema.node".to_string(),
                value: AnnotationValue::String("UnitLit".to_string()),
            },
        ],
    };

    // Structurally, these should be equivalent (modulo annotations)
    assert!(verify_annotations_ignored_structurally(
        &term_without_annotations,
        &term_with_annotations
    )
    .is_ok());
}

#[test]
fn test_trace_graph_invariant_enforcement() {
    // Test that orphan Core IR nodes fail invariants

    let mut builder = TraceGraphBuilder::new();

    // Core IR node with NO source tokens (violates invariant)
    builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(1),
        None,
        ProvenanceAnnotationSet::new(),
        Vec::new(),
    );

    let graph = builder.build();

    // This should fail because the Core IR has no source
    let result = verify_trace_invariants(&graph);
    assert!(result.is_err());
}

#[test]
fn test_multi_level_traceability() {
    // Verify that we can trace through multiple layers

    let mut builder = TraceGraphBuilder::new();

    // Three tokens
    let token1 = Token::new(
        TokenKind::Keyword("if".to_string()),
        "if".to_string(),
        Span::new(0, 2),
    );
    let token2 = Token::new(TokenKind::Ident, "x".to_string(), Span::new(3, 4));
    let token3 = Token::new(TokenKind::UnitLit, "()".to_string(), Span::new(5, 7));

    let token_id1 = builder.add_token(token1, ProvenanceAnnotationSet::new());
    let token_id2 = builder.add_token(token2, ProvenanceAnnotationSet::new());
    let token_id3 = builder.add_token(token3, ProvenanceAnnotationSet::new());

    // Three Generic AST nodes
    let ast_id1 = builder.add_generic_ast(
        "keyword".to_string(),
        0,
        Some(Span::new(0, 2)),
        ProvenanceAnnotationSet::new(),
        vec![token_id1],
    );
    let ast_id2 = builder.add_generic_ast(
        "ident".to_string(),
        0,
        Some(Span::new(3, 4)),
        ProvenanceAnnotationSet::new(),
        vec![token_id2],
    );
    let ast_id3 = builder.add_generic_ast(
        "unit".to_string(),
        0,
        Some(Span::new(5, 7)),
        ProvenanceAnnotationSet::new(),
        vec![token_id3],
    );

    // One Schema AST node combining all three
    let schema_id = builder.add_schema_ast(
        "IfExpr".to_string(),
        3,
        Some(Span::new(0, 7)),
        ProvenanceAnnotationSet::new(),
        vec![ast_id1, ast_id2, ast_id3],
    );

    // One Core IR node
    let _core_ir_id = builder.add_core_ir(
        "CIf".to_string(),
        Some(100),
        Some(Span::new(0, 7)),
        ProvenanceAnnotationSet::new(),
        vec![schema_id],
    );

    let graph = builder.build();
    let inspector = Inspector::new(graph);

    // Verify we can trace back to all three tokens
    let spans = inspector.source_spans(100);
    assert_eq!(spans.len(), 3);

    // Verify origin includes all parents
    let origin = inspector.origin(100).unwrap();
    assert_eq!(origin.source_spans.len(), 3);
}

#[test]
fn test_inspector_find_by_rule() {
    let mut builder = TraceGraphBuilder::new();

    // Create multiple Core IR nodes with different rules
    for i in 1..=3 {
        let token = Token::new(TokenKind::UnitLit, "()".to_string(), Span::new(0, 2));
        let token_id = builder.add_token(token, ProvenanceAnnotationSet::new());

        let mut annotations = ProvenanceAnnotationSet::new();
        if i == 2 {
            annotations.add(provenance::lowering::rule("special_rule"));
        } else {
            annotations.add(provenance::lowering::rule("normal_rule"));
        }

        builder.add_core_ir(
            "CUnitLit".to_string(),
            Some(i),
            None,
            annotations,
            vec![token_id],
        );
    }

    let graph = builder.build();
    let inspector = Inspector::new(graph);

    let special_nodes = inspector.find_by_lowering_rule("special_rule");
    assert_eq!(special_nodes.len(), 1);
    assert_eq!(special_nodes[0], 2);

    let normal_nodes = inspector.find_by_lowering_rule("normal_rule");
    assert_eq!(normal_nodes.len(), 2);
}
