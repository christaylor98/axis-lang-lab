// Regression test: Verify undeclared foreign symbols cause compilation failure
//
// This test ensures the registry is authoritative:
// - Foreign/CCall targets MUST be declared in the CLI-provided registry
// - Compilation MUST fail if a symbol is undeclared
// - No fallback registries are consulted

use axis_lang_lab::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::lowering::nf_lowering::{lower_to_core_ir, LoweringContext, Scope};
use axis_lang_lab::registry::{Registry, RegistryEntry};
use std::collections::HashMap;

#[test]
fn test_undeclared_foreign_symbol_fails_at_lowering() {
    // SETUP: Create a CallExpr node that calls an undeclared function
    let mut fields = HashMap::new();
    
    // name field: token for "missing_symbol"
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "missing_symbol".to_string(),
            span: Span::new(0, 14),
        }),
    );
    
    // args field: empty list (zero-arity call)
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![]));
    
    let call_node = SchemaAstNode {
        kind: "CallExpr".to_string(),
        fields,
        annotations: vec![],
        span: Span::new(0, 14),
    };
    
    // SETUP: Create empty registry (no functions declared)
    let empty_registry = Registry::new();
    
    let mut ctx = LoweringContext {
        registry: empty_registry,
        scope: Scope::new(),
    };
    
    // EXECUTE: Try to lower the call node
    let result = lower_to_core_ir(&call_node, &mut ctx);
    
    // ASSERT: Must fail with "not found" error
    assert!(
        result.is_err(),
        "Lowering should fail when function is not in registry"
    );
    
    let error = result.unwrap_err();
    assert!(
        error.message.contains("missing_symbol") && error.message.contains("not found"),
        "Error should mention undeclared function. Got: {}",
        error.message
    );
}

#[test]
fn test_declared_foreign_symbol_succeeds_at_lowering() {
    // SETUP: Create a CallExpr node that calls a declared function
    let mut fields = HashMap::new();
    
    // name field: token for "test_fn"
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "test_fn".to_string(),
            span: Span::new(0, 7),
        }),
    );
    
    // args field: list with one IntLit argument
    let arg_node = SchemaAstNode {
        kind: "IntLit".to_string(),
        fields: {
            let mut arg_fields = HashMap::new();
            arg_fields.insert(
                "value".to_string(),
                SchemaValue::Token(Token {
                    kind: TokenKind::IntLit,
                    lexeme: "42".to_string(),
                    span: Span::new(8, 10),
                }),
            );
            arg_fields
        },
        annotations: vec![],
        span: Span::new(8, 10),
    };
    
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![arg_node]));
    
    let call_node = SchemaAstNode {
        kind: "CallExpr".to_string(),
        fields,
        annotations: vec![],
        span: Span::new(0, 10),
    };
    
    // SETUP: Create registry with test_fn declared
    let mut registry = Registry::new();
    registry.register(
        "test_fn".to_string(),
        RegistryEntry {
            id: 1,
            name: "test_fn".to_string(),
            arity: 1,
            deterministic: true,
            profiles: vec!["all".to_string()],
        },
    );
    
    let mut ctx = LoweringContext {
        registry,
        scope: Scope::new(),
    };
    
    // EXECUTE: Try to lower the call node
    let result = lower_to_core_ir(&call_node, &mut ctx);
    
    // ASSERT: Must succeed
    assert!(
        result.is_ok(),
        "Lowering should succeed when function is declared in registry. Error: {:?}",
        result.err()
    );
}

#[test]
fn test_arity_mismatch_fails_at_lowering() {
    // SETUP: Create CallExpr with wrong number of arguments
    let mut fields = HashMap::new();
    
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "test_fn".to_string(),
            span: Span::new(0, 7),
        }),
    );
    
    // args field: TWO arguments, but registry declares arity 1
    let arg1 = SchemaAstNode {
        kind: "IntLit".to_string(),
        fields: {
            let mut arg_fields = HashMap::new();
            arg_fields.insert(
                "value".to_string(),
                SchemaValue::Token(Token {
                    kind: TokenKind::IntLit,
                    lexeme: "1".to_string(),
                    span: Span::new(8, 9),
                }),
            );
            arg_fields
        },
        annotations: vec![],
        span: Span::new(8, 9),
    };
    
    let arg2 = SchemaAstNode {
        kind: "IntLit".to_string(),
        fields: {
            let mut arg_fields = HashMap::new();
            arg_fields.insert(
                "value".to_string(),
                SchemaValue::Token(Token {
                    kind: TokenKind::IntLit,
                    lexeme: "2".to_string(),
                    span: Span::new(10, 11),
                }),
            );
            arg_fields
        },
        annotations: vec![],
        span: Span::new(10, 11),
    };
    
    fields.insert("args".to_string(), SchemaValue::Nodes(vec![arg1, arg2]));
    
    let call_node = SchemaAstNode {
        kind: "CallExpr".to_string(),
        fields,
        annotations: vec![],
        span: Span::new(0, 11),
    };
    
    // SETUP: Create registry with test_fn declared with arity 1
    let mut registry = Registry::new();
    registry.register(
        "test_fn".to_string(),
        RegistryEntry {
            id: 1,
            name: "test_fn".to_string(),
            arity: 1,  // Expects 1 argument
            deterministic: true,
            profiles: vec!["all".to_string()],
        },
    );
    
    let mut ctx = LoweringContext {
        registry,
        scope: Scope::new(),
    };
    
    // EXECUTE: Try to lower the call node
    let result = lower_to_core_ir(&call_node, &mut ctx);
    
    // ASSERT: Must fail with arity mismatch error
    assert!(
        result.is_err(),
        "Lowering should fail when arity doesn't match"
    );
    
    let error = result.unwrap_err();
    assert!(
        error.message.contains("arity") || error.message.contains("expects 1"),
        "Error should mention arity mismatch. Got: {}",
        error.message
    );
}

