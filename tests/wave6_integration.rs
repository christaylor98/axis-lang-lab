// Wave 6 Integration Tests: Full Pipeline Execution
//
// These tests verify the complete pipeline from Schema AST to execution:
// Schema AST (Wave 4) -> Core IR (Wave 5) -> Execution (Wave 6)

use axis_lang_lab::execution::interpreter::eval_bundle;
use axis_lang_lab::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::lowering::schema_lowering::lower_to_bundle;
use axis_lang_lab::registry::{Registry, RegistryEntry};
use std::collections::HashMap;

fn test_registry() -> Registry {
    Registry {
        entries: vec![
            RegistryEntry {
                name: "print".to_string(),
                arity: 1,
                deterministic: false,
                profiles: vec![],
                id: 1,
            },
            RegistryEntry {
                name: "add".to_string(),
                arity: 2,
                deterministic: true,
                profiles: vec![],
                id: 2,
            },
        ],
    }
}

#[test]
fn test_wave6_integration_unit_lit() {
    // Test: Complete pipeline for unit literal
    // Schema AST -> Core IR -> Execution

    let schema_ast = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    };

    let registry = test_registry();

    // Lower to Core IR
    let bundle = lower_to_bundle(&schema_ast, registry.clone()).unwrap();

    // Execute
    let result = eval_bundle(&bundle, registry).unwrap();

    // Verify
    assert_eq!(format!("{}", result), "()");
}

#[test]
fn test_wave6_integration_lambda() {
    // Test: Complete pipeline for lambda expression
    // \x -> ()

    let mut fields = HashMap::new();
    fields.insert(
        "param".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "x".to_string(),
            span: Span::new(1, 2),
        }),
    );
    fields.insert(
        "body".to_string(),
        SchemaValue::Node(Box::new(SchemaAstNode {
            kind: "UnitLit".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(6, 8),
        })),
    );

    let schema_ast = SchemaAstNode {
        kind: "Lam".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 8),
    };

    let registry = test_registry();

    // Lower to Core IR
    let bundle = lower_to_bundle(&schema_ast, registry.clone()).unwrap();

    // Execute
    let result = eval_bundle(&bundle, registry).unwrap();

    // Verify
    assert_eq!(format!("{}", result), "<closure>");
}

#[test]
fn test_wave6_integration_if_expression() {
    // Test: Complete pipeline for conditional
    // if () then () else ()

    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    };

    let mut fields = HashMap::new();
    fields.insert(
        "condition".to_string(),
        SchemaValue::Node(Box::new(unit_lit.clone())),
    );
    fields.insert(
        "then_branch".to_string(),
        SchemaValue::Node(Box::new(unit_lit.clone())),
    );
    fields.insert(
        "else_branch".to_string(),
        SchemaValue::Node(Box::new(unit_lit.clone())),
    );

    let schema_ast = SchemaAstNode {
        kind: "If".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 20),
    };

    let registry = test_registry();

    // Lower to Core IR
    let bundle = lower_to_bundle(&schema_ast, registry.clone()).unwrap();

    // Execute
    let result = eval_bundle(&bundle, registry).unwrap();

    // Verify
    assert_eq!(format!("{}", result), "()");
}

#[test]
fn test_wave6_integration_registry_call() {
    // Test: Complete pipeline for registry call
    // print(())

    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(6, 8),
    };

    let mut fields = HashMap::new();
    fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "print".to_string(),
            span: Span::new(0, 5),
        }),
    );
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![unit_lit]));

    let schema_ast = SchemaAstNode {
        kind: "Call".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 9),
    };

    let registry = test_registry();

    // Lower to Core IR
    let bundle = lower_to_bundle(&schema_ast, registry.clone()).unwrap();

    // Execute
    let result = eval_bundle(&bundle, registry).unwrap();

    // Verify
    assert_eq!(format!("{}", result), "()");
}

#[test]
fn test_wave6_integration_complex_expression() {
    // Test: Complete pipeline for complex nested expression
    // if () then print(()) else add((), ())

    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    };

    // Build print(()) call
    let mut print_fields = HashMap::new();
    print_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "print".to_string(),
            span: Span::new(11, 16),
        }),
    );
    print_fields.insert(
        "args".to_string(),
        SchemaValue::Nodes(vec![unit_lit.clone()]),
    );
    let print_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: print_fields,
        annotations: Vec::new(),
        span: Span::new(11, 21),
    };

    // Build add((), ()) call
    let mut add_fields = HashMap::new();
    add_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "add".to_string(),
            span: Span::new(27, 30),
        }),
    );
    add_fields.insert(
        "args".to_string(),
        SchemaValue::Nodes(vec![unit_lit.clone(), unit_lit.clone()]),
    );
    let add_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: add_fields,
        annotations: Vec::new(),
        span: Span::new(27, 40),
    };

    // Build if expression
    let mut if_fields = HashMap::new();
    if_fields.insert(
        "condition".to_string(),
        SchemaValue::Node(Box::new(unit_lit)),
    );
    if_fields.insert(
        "then_branch".to_string(),
        SchemaValue::Node(Box::new(print_call)),
    );
    if_fields.insert(
        "else_branch".to_string(),
        SchemaValue::Node(Box::new(add_call)),
    );

    let schema_ast = SchemaAstNode {
        kind: "If".to_string(),
        fields: if_fields,
        annotations: Vec::new(),
        span: Span::new(0, 40),
    };

    let registry = test_registry();

    // Lower to Core IR
    let bundle = lower_to_bundle(&schema_ast, registry.clone()).unwrap();

    // Execute
    let result = eval_bundle(&bundle, registry).unwrap();

    // Verify
    assert_eq!(format!("{}", result), "()");
}

#[test]
fn test_wave6_integration_missing_registry_entry() {
    // Test: Pipeline correctly fails on missing registry entry

    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    };

    let mut fields = HashMap::new();
    fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "undefined_function".to_string(),
            span: Span::new(0, 18),
        }),
    );
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![unit_lit]));

    let schema_ast = SchemaAstNode {
        kind: "Call".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 20),
    };

    let registry = test_registry();

    // Lowering should fail (registry lookup happens during lowering)
    let result = lower_to_bundle(&schema_ast, registry);
    assert!(result.is_err());
}

#[test]
fn test_wave6_integration_determinism() {
    // Test: Pipeline produces deterministic results

    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    };

    let mut fields = HashMap::new();
    fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "print".to_string(),
            span: Span::new(0, 5),
        }),
    );
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![unit_lit]));

    let schema_ast = SchemaAstNode {
        kind: "Call".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 9),
    };

    let registry1 = test_registry();
    let registry2 = test_registry();

    // Lower and execute twice
    let bundle1 = lower_to_bundle(&schema_ast, registry1.clone()).unwrap();
    let result1 = eval_bundle(&bundle1, registry1).unwrap();

    let bundle2 = lower_to_bundle(&schema_ast, registry2.clone()).unwrap();
    let result2 = eval_bundle(&bundle2, registry2).unwrap();

    // Results should be identical
    assert_eq!(result1, result2);
}
