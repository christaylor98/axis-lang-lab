// Wave B: Trust Invariant Tests
//
// These tests verify the trust guarantees of Wave B:
// - Every Core IR node is traceable
// - No Core IR node lacks provenance
// - User annotations never override system annotations
// - Execution ignores all trace data
// - Trace graphs are deterministic

use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::introspection::provenance::{
    self, ProvenanceAnnotationSet, ProvenanceNamespace,
};
use axis_lang_lab::introspection::trace_graph::{TraceGraphBuilder, TraceLinkKind};
use axis_lang_lab::introspection::trust::{
    verify_annotation_namespaces, verify_annotations_ignored_structurally,
    verify_trace_determinism, verify_trace_invariants, TrustInvariantError,
};
use axis_lang_lab::introspection::*;
use axis_lang_lab::ir::core_ir::{unit_lit, Annotation, AnnotationValue, CoreTerm};

#[test]
fn test_every_core_ir_node_is_traceable() {
    // Invariant: Every Core IR node must be traceable to source tokens

    let mut builder = TraceGraphBuilder::new();

    // Valid: Core IR with source token
    let token = Token::new(TokenKind::UnitLit, "()".to_string(), Span::new(0, 2));
    let token_id = builder.add_token(token, ProvenanceAnnotationSet::new());

    builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(1),
        None,
        ProvenanceAnnotationSet::new(),
        vec![token_id],
    );

    let graph = builder.build();
    let result = verify_trace_invariants(&graph);

    assert!(result.is_ok(), "Valid trace should pass invariants");
}

#[test]
fn test_orphan_core_ir_fails_invariant() {
    // Invariant: Core IR nodes without source fail validation

    let mut builder = TraceGraphBuilder::new();

    // Invalid: Core IR with NO source
    builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(1),
        None,
        ProvenanceAnnotationSet::new(),
        Vec::new(), // No parents!
    );

    let graph = builder.build();
    let result = verify_trace_invariants(&graph);

    assert!(result.is_err(), "Orphan Core IR should fail invariants");
}

#[test]
fn test_user_annotations_cannot_use_reserved_namespaces() {
    // Invariant: User annotations must not use system namespaces

    // Valid user annotations
    let valid = vec![
        Annotation {
            key: "user.custom".to_string(),
            value: AnnotationValue::String("ok".to_string()),
        },
        Annotation {
            key: "my_annotation".to_string(),
            value: AnnotationValue::Bool(true),
        },
        Annotation {
            key: "metadata".to_string(),
            value: AnnotationValue::Int(42),
        },
    ];

    let result = verify_annotation_namespaces(&valid);
    assert!(result.is_ok(), "Valid user annotations should pass");

    // Invalid: using 'source' namespace
    let invalid_source = vec![Annotation {
        key: "source.span".to_string(),
        value: AnnotationValue::String("0..10".to_string()),
    }];

    let result = verify_annotation_namespaces(&invalid_source);
    assert!(result.is_err(), "source.* namespace is reserved");
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        TrustInvariantError::UserAnnotationInReservedNamespace {
            reserved_namespace, ..
        } => {
            assert_eq!(reserved_namespace, "source");
        }
        _ => panic!("Wrong error type"),
    }

    // Invalid: using 'schema' namespace
    let invalid_schema = vec![Annotation {
        key: "schema.node".to_string(),
        value: AnnotationValue::String("IfExpr".to_string()),
    }];

    assert!(verify_annotation_namespaces(&invalid_schema).is_err());

    // Invalid: using 'lowering' namespace
    let invalid_lowering = vec![Annotation {
        key: "lowering.rule".to_string(),
        value: AnnotationValue::String("rule_name".to_string()),
    }];

    assert!(verify_annotation_namespaces(&invalid_lowering).is_err());

    // Invalid: using 'registry' namespace
    let invalid_registry = vec![Annotation {
        key: "registry.id".to_string(),
        value: AnnotationValue::String("id".to_string()),
    }];

    assert!(verify_annotation_namespaces(&invalid_registry).is_err());

    // Invalid: using 'pipeline' namespace
    let invalid_pipeline = vec![Annotation {
        key: "pipeline.phase".to_string(),
        value: AnnotationValue::String("phase".to_string()),
    }];

    assert!(verify_annotation_namespaces(&invalid_pipeline).is_err());
}

#[test]
fn test_annotations_do_not_affect_structure() {
    // Invariant: Annotations are data only, not behavior

    let term1 = unit_lit();

    let term2 = CoreTerm::CUnitLit {
        node_id: None,
        annotations: vec![
            Annotation {
                key: "source.file".to_string(),
                value: AnnotationValue::String("test.ax".to_string()),
            },
            Annotation {
                key: "schema.node".to_string(),
                value: AnnotationValue::String("UnitLit".to_string()),
            },
            Annotation {
                key: "user.custom".to_string(),
                value: AnnotationValue::Bool(true),
            },
        ],
    };

    // These should be structurally equivalent (ignoring annotations)
    let result = verify_annotations_ignored_structurally(&term1, &term2);
    assert!(result.is_ok(), "Annotations should not affect structure");
}

#[test]
fn test_annotations_do_not_affect_complex_structure() {
    // Test with nested structures

    use axis_lang_lab::ir::core_ir::{lam, IdentOrName};

    let term1 = lam(IdentOrName::new("x"), unit_lit());

    let term2 = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(CoreTerm::CUnitLit {
            node_id: None,
            annotations: vec![Annotation {
                key: "nested.annotation".to_string(),
                value: AnnotationValue::String("nested".to_string()),
            }],
        }),
        node_id: None,
        annotations: vec![Annotation {
            key: "outer.annotation".to_string(),
            value: AnnotationValue::String("outer".to_string()),
        }],
    };

    let result = verify_annotations_ignored_structurally(&term1, &term2);
    assert!(
        result.is_ok(),
        "Nested annotations should not affect structure"
    );
}

#[test]
fn test_trace_determinism() {
    // Invariant: Same input produces identical trace graphs

    let build_trace = || {
        let mut builder = TraceGraphBuilder::new();

        // Build identical trace structure
        let token1 = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(0, 3));
        let token2 = Token::new(TokenKind::Ident, "bar".to_string(), Span::new(4, 7));

        let mut annotations1 = ProvenanceAnnotationSet::new();
        annotations1.add(provenance::source::file("test.ax"));

        let token_id1 = builder.add_token(token1, annotations1);
        let token_id2 = builder.add_token(token2, ProvenanceAnnotationSet::new());

        let ast_id = builder.add_generic_ast(
            "call".to_string(),
            2,
            Some(Span::new(0, 7)),
            ProvenanceAnnotationSet::new(),
            vec![token_id1, token_id2],
        );

        let mut core_ir_annotations = ProvenanceAnnotationSet::new();
        core_ir_annotations.add(provenance::lowering::rule("lower_call"));

        builder.add_core_ir(
            "CCall".to_string(),
            Some(1),
            Some(Span::new(0, 7)),
            core_ir_annotations,
            vec![ast_id],
        );

        builder.build()
    };

    let trace1 = build_trace();
    let trace2 = build_trace();

    // Verify structural equivalence
    let result = verify_trace_determinism(&trace1, &trace2);
    assert!(
        result.is_ok(),
        "Identical construction should produce deterministic traces"
    );
}

#[test]
fn test_all_provenance_namespaces_are_reserved() {
    // Verify all defined namespaces are properly reserved

    let namespaces = vec![
        ("source.test", ProvenanceNamespace::Source),
        ("schema.test", ProvenanceNamespace::Schema),
        ("lowering.test", ProvenanceNamespace::Lowering),
        ("registry.test", ProvenanceNamespace::Registry),
        ("pipeline.test", ProvenanceNamespace::Pipeline),
    ];

    for (key, expected_ns) in namespaces {
        // Verify namespace detection
        let detected = ProvenanceAnnotation::namespace_of(key);
        assert_eq!(
            detected,
            Some(expected_ns),
            "Failed to detect namespace for {}",
            key
        );

        // Verify annotation validation rejects it
        let annotation = Annotation {
            key: key.to_string(),
            value: AnnotationValue::String("test".to_string()),
        };

        let result = verify_annotation_namespaces(&vec![annotation]);
        assert!(
            result.is_err(),
            "Reserved namespace {} should be rejected",
            key
        );
    }
}

#[test]
fn test_trace_graph_link_consistency() {
    // Verify that trace graph validates link integrity

    let mut builder = TraceGraphBuilder::new();

    let token = Token::new(TokenKind::UnitLit, "()".to_string(), Span::new(0, 2));
    let token_id = builder.add_token(token, ProvenanceAnnotationSet::new());

    let ast_id = builder.add_generic_ast(
        "unit".to_string(),
        1,
        Some(Span::new(0, 2)),
        ProvenanceAnnotationSet::new(),
        vec![token_id],
    );

    // Add explicit link
    builder.add_link(token_id, ast_id, TraceLinkKind::TokenToGenericAst);

    let core_ir_id = builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(1),
        Some(Span::new(0, 2)),
        ProvenanceAnnotationSet::new(),
        vec![ast_id],
    );

    builder.add_link(ast_id, core_ir_id, TraceLinkKind::SchemaAstToCoreIr);

    let graph = builder.build();

    // Verify internal consistency
    assert!(graph.verify_invariants().is_ok());

    // Verify links are accessible
    let links_from_token = graph.links_from(token_id);
    assert_eq!(links_from_token.len(), 1);

    let links_to_core_ir = graph.links_to(core_ir_id);
    assert_eq!(links_to_core_ir.len(), 1);
}

#[test]
fn test_provenance_annotations_preserve_origin() {
    // Verify that provenance annotations correctly preserve origin info

    let mut builder = TraceGraphBuilder::new();

    let token = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(10, 13));

    let mut token_annotations = ProvenanceAnnotationSet::new();
    token_annotations.add(provenance::source::file("main.ax"));
    token_annotations.add(provenance::source::text("foo"));
    token_annotations.add(provenance::source::span(&Span::new(10, 13)));

    let token_id = builder.add_token(token, token_annotations);

    let mut schema_annotations = ProvenanceAnnotationSet::new();
    schema_annotations.add(provenance::schema::node("Ident"));

    let schema_id = builder.add_schema_ast(
        "Ident".to_string(),
        1,
        Some(Span::new(10, 13)),
        schema_annotations,
        vec![token_id],
    );

    let mut core_ir_annotations = ProvenanceAnnotationSet::new();
    core_ir_annotations.add(provenance::lowering::rule("ident_to_unit"));

    builder.add_core_ir(
        "CUnitLit".to_string(),
        Some(42),
        Some(Span::new(10, 13)),
        core_ir_annotations,
        vec![schema_id],
    );

    let graph = builder.build();
    let inspector = Inspector::new(graph);

    // Verify origin is preserved
    let origin = inspector.origin(42).unwrap();
    assert_eq!(origin.source_file, Some("main.ax".to_string()));
    assert_eq!(origin.source_spans.len(), 1);
    assert_eq!(origin.source_spans[0].start, 10);
    assert_eq!(origin.source_spans[0].end, 13);

    // Verify schema info is preserved
    let schema_info = inspector.schema_rule(42).unwrap();
    assert_eq!(schema_info.node_kind, "Ident");

    // Verify lowering info is preserved
    let lowering_info = inspector.lowering_rule(42).unwrap();
    assert_eq!(lowering_info.rule_name, Some("ident_to_unit".to_string()));
}

#[test]
fn test_namespace_collision_prevention() {
    // Edge case: user annotation that looks like it might be reserved

    // Valid: has the word "source" but not in namespace format
    let ok1 = vec![Annotation {
        key: "sourcecode".to_string(), // No dot separator
        value: AnnotationValue::String("ok".to_string()),
    }];
    assert!(verify_annotation_namespaces(&ok1).is_ok());

    // Valid: has dot but wrong prefix
    let ok2 = vec![Annotation {
        key: "my_source.file".to_string(),
        value: AnnotationValue::String("ok".to_string()),
    }];
    assert!(verify_annotation_namespaces(&ok2).is_ok());

    // Invalid: exact reserved namespace
    let bad = vec![Annotation {
        key: "source.file".to_string(),
        value: AnnotationValue::String("bad".to_string()),
    }];
    assert!(verify_annotation_namespaces(&bad).is_err());
}
