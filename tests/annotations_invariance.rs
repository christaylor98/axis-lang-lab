// Annotation Invariance Tests
//
// These tests verify that annotations do NOT affect execution semantics.
// Execution must be identical with or without annotations.

use axis_lang_lab::execution::interpreter::{eval_core_ir, EvalContext, Value};
use axis_lang_lab::ir::core_ir::{Annotation, AnnotationValue, CoreTerm, IdentOrName};
use axis_lang_lab::registry::Registry;

#[test]
fn test_unit_lit_with_and_without_annotations() {
    // Create two identical unit literals - one with annotations, one without
    let without_ann = CoreTerm::CUnitLit {
        node_id: None,
        annotations: Vec::new(),
    };

    let with_ann = CoreTerm::CUnitLit {
        node_id: None,
        annotations: vec![Annotation {
            key: "inline".to_string(),
            value: AnnotationValue::Bool(true),
        }],
    };

    let registry = Registry {
        entries: Vec::new(),
    };

    let mut ctx1 = EvalContext::new(registry.clone());
    let mut ctx2 = EvalContext::new(registry);

    // Evaluate both
    let result1 = eval_core_ir(&without_ann, &mut ctx1).expect("eval failed");
    let result2 = eval_core_ir(&with_ann, &mut ctx2).expect("eval failed");

    // Results must be identical
    assert_eq!(result1, result2);
    assert!(matches!(result1, Value::Unit));
}

#[test]
fn test_lambda_with_and_without_annotations() {
    // Create two identical lambdas - one with annotations, one without
    let body = CoreTerm::CUnitLit {
        node_id: None,
        annotations: Vec::new(),
    };

    let without_ann = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(body.clone()),
        node_id: None,
        annotations: Vec::new(),
    };

    let with_ann = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(body),
        node_id: None,
        annotations: vec![
            Annotation {
                key: "doc".to_string(),
                value: AnnotationValue::String("test lambda".to_string()),
            },
            Annotation {
                key: "inline".to_string(),
                value: AnnotationValue::Bool(true),
            },
        ],
    };

    let registry = Registry {
        entries: Vec::new(),
    };

    let mut ctx1 = EvalContext::new(registry.clone());
    let mut ctx2 = EvalContext::new(registry);

    // Evaluate both
    let result1 = eval_core_ir(&without_ann, &mut ctx1).expect("eval failed");
    let result2 = eval_core_ir(&with_ann, &mut ctx2).expect("eval failed");

    // Results must be closures and structurally equivalent
    match (&result1, &result2) {
        (Value::Closure { param: p1, .. }, Value::Closure { param: p2, .. }) => {
            assert_eq!(p1, p2);
        }
        _ => panic!("Expected closures"),
    }
}

#[test]
fn test_if_with_and_without_annotations() {
    // Create two identical if expressions - one with annotations, one without
    let unit = CoreTerm::CUnitLit {
        node_id: None,
        annotations: Vec::new(),
    };

    let without_ann = CoreTerm::CIf {
        cond: Box::new(unit.clone()),
        then_branch: Box::new(unit.clone()),
        else_branch: Box::new(unit.clone()),
        node_id: None,
        annotations: Vec::new(),
    };

    let with_ann = CoreTerm::CIf {
        cond: Box::new(unit.clone()),
        then_branch: Box::new(unit.clone()),
        else_branch: Box::new(unit),
        node_id: None,
        annotations: vec![Annotation {
            key: "priority".to_string(),
            value: AnnotationValue::Int(10),
        }],
    };

    let registry = Registry {
        entries: Vec::new(),
    };

    let mut ctx1 = EvalContext::new(registry.clone());
    let mut ctx2 = EvalContext::new(registry);

    // Evaluate both
    let result1 = eval_core_ir(&without_ann, &mut ctx1).expect("eval failed");
    let result2 = eval_core_ir(&with_ann, &mut ctx2).expect("eval failed");

    // Results must be identical
    assert_eq!(result1, result2);
    assert!(matches!(result1, Value::Unit));
}

#[test]
fn test_nested_terms_with_mixed_annotations() {
    // Create complex nested structure with annotations at different levels
    let inner = CoreTerm::CUnitLit {
        node_id: None,
        annotations: vec![Annotation {
            key: "inner".to_string(),
            value: AnnotationValue::String("nested".to_string()),
        }],
    };

    let middle = CoreTerm::CLam {
        param: IdentOrName::new("y"),
        body: Box::new(inner),
        node_id: None,
        annotations: vec![Annotation {
            key: "middle".to_string(),
            value: AnnotationValue::Bool(false),
        }],
    };

    let outer = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(middle),
        node_id: None,
        annotations: vec![Annotation {
            key: "outer".to_string(),
            value: AnnotationValue::Int(42),
        }],
    };

    // Compare with version without annotations
    let inner_plain = CoreTerm::CUnitLit {
        node_id: None,
        annotations: Vec::new(),
    };

    let middle_plain = CoreTerm::CLam {
        param: IdentOrName::new("y"),
        body: Box::new(inner_plain),
        node_id: None,
        annotations: Vec::new(),
    };

    let outer_plain = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(middle_plain),
        node_id: None,
        annotations: Vec::new(),
    };

    let registry = Registry {
        entries: Vec::new(),
    };

    let mut ctx1 = EvalContext::new(registry.clone());
    let mut ctx2 = EvalContext::new(registry);

    // Evaluate both
    let result1 = eval_core_ir(&outer_plain, &mut ctx1).expect("eval failed");
    let result2 = eval_core_ir(&outer, &mut ctx2).expect("eval failed");

    // Both should produce closures with same parameter
    match (&result1, &result2) {
        (Value::Closure { param: p1, .. }, Value::Closure { param: p2, .. }) => {
            assert_eq!(p1, p2);
        }
        _ => panic!("Expected closures"),
    }
}

#[test]
fn test_determinism_with_annotations() {
    // Same term with annotations should produce identical results across multiple runs
    let term = CoreTerm::CLam {
        param: IdentOrName::new("x"),
        body: Box::new(CoreTerm::CUnitLit {
            node_id: None,
            annotations: Vec::new(),
        }),
        node_id: None,
        annotations: vec![
            Annotation {
                key: "test1".to_string(),
                value: AnnotationValue::Bool(true),
            },
            Annotation {
                key: "test2".to_string(),
                value: AnnotationValue::Int(123),
            },
            Annotation {
                key: "test3".to_string(),
                value: AnnotationValue::String("value".to_string()),
            },
        ],
    };

    let registry = Registry {
        entries: Vec::new(),
    };

    // Run multiple times
    let mut results = Vec::new();
    for _ in 0..5 {
        let mut ctx = EvalContext::new(registry.clone());
        let result = eval_core_ir(&term, &mut ctx).expect("eval failed");
        results.push(result);
    }

    // All results must be identical
    for result in &results[1..] {
        assert_eq!(&results[0], result);
    }
}
