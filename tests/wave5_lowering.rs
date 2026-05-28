// Wave 5 Integration Tests: Schema AST → Core IR Lowering
//
// These tests verify the complete lowering pipeline from Schema AST to Core IR.
// Tests focus on:
// - Correct lowering of each node kind
// - Proper error handling
// - Determinism
// - Span preservation
// - Scope management

use axis_lang_lab::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::ir::core_ir::CoreTerm;
use axis_lang_lab::lowering::schema_lowering::{
    lower_to_bundle, lower_to_core_ir, LoweringContext, Scope,
};
use axis_lang_lab::registry::{Registry, RegistryEntry};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

fn make_test_registry() -> Registry {
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
            RegistryEntry {
                name: "not".to_string(),
                arity: 1,
                deterministic: true,
                profiles: vec![],
                id: 3,
            },
        ],
    }
}

fn make_unit_lit() -> SchemaAstNode {
    SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    }
}

#[allow(dead_code)]
fn make_ident(name: &str, start: usize, end: usize) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: name.to_string(),
            span: Span::new(start, end),
        }),
    );

    SchemaAstNode {
        kind: "Ident".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(start, end),
    }
}

fn make_lam(param_name: &str, body: SchemaAstNode, span: Span) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "param".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: param_name.to_string(),
            span: Span::new(span.start, span.start + param_name.len()),
        }),
    );
    fields.insert("body".to_string(), SchemaValue::Node(Box::new(body)));

    SchemaAstNode {
        kind: "Lam".to_string(),
        fields,
        annotations: Vec::new(),
        span,
    }
}

fn make_if(
    condition: SchemaAstNode,
    then_branch: SchemaAstNode,
    else_branch: SchemaAstNode,
    span: Span,
) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "condition".to_string(),
        SchemaValue::Node(Box::new(condition)),
    );
    fields.insert(
        "then_branch".to_string(),
        SchemaValue::Node(Box::new(then_branch)),
    );
    fields.insert(
        "else_branch".to_string(),
        SchemaValue::Node(Box::new(else_branch)),
    );

    SchemaAstNode {
        kind: "If".to_string(),
        fields,
        annotations: Vec::new(),
        span,
    }
}

fn make_call(target: &str, args: Vec<SchemaAstNode>, span: Span) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: target.to_string(),
            span: Span::new(span.start, span.start + target.len()),
        }),
    );
    fields.insert("args".to_string(), SchemaValue::Nodes(args));

    SchemaAstNode {
        kind: "Call".to_string(),
        fields,
        annotations: Vec::new(),
        span,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BASIC LOWERING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_unit_lit_to_core_ir() {
    let node = make_unit_lit();
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CUnitLit { .. } => {}
        other => panic!("Expected CUnitLit, got {:?}", other),
    }
}

#[test]
fn test_lower_lambda_to_core_ir() {
    // \x -> ()
    let node = make_lam("x", make_unit_lit(), Span::new(0, 10));
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CLam { param, body, .. } => {
            assert_eq!(param.0.as_ref(), "x");
            assert!(matches!(*body, CoreTerm::CUnitLit { .. }));
        }
        other => panic!("Expected CLam, got {:?}", other),
    }
}

#[test]
fn test_lower_nested_lambdas() {
    // \x -> \y -> ()
    let inner = make_lam("y", make_unit_lit(), Span::new(6, 15));
    let outer = make_lam("x", inner, Span::new(0, 15));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&outer, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CLam {
            param: outer_param,
            body: outer_body,
            ..
        } => {
            assert_eq!(outer_param.0.as_ref(), "x");
            match *outer_body {
                CoreTerm::CLam {
                    param: inner_param,
                    body: inner_body,
                    ..
                } => {
                    assert_eq!(inner_param.0.as_ref(), "y");
                    assert!(matches!(*inner_body, CoreTerm::CUnitLit { .. }));
                }
                other => panic!("Expected inner CLam, got {:?}", other),
            }
        }
        other => panic!("Expected outer CLam, got {:?}", other),
    }
}

#[test]
fn test_lower_if_to_core_ir() {
    // if () then () else ()
    let node = make_if(
        make_unit_lit(),
        make_unit_lit(),
        make_unit_lit(),
        Span::new(0, 25),
    );

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(matches!(*cond, CoreTerm::CUnitLit { .. }));
            assert!(matches!(*then_branch, CoreTerm::CUnitLit { .. }));
            assert!(matches!(*else_branch, CoreTerm::CUnitLit { .. }));
        }
        other => panic!("Expected CIf, got {:?}", other),
    }
}

#[test]
fn test_lower_call_to_core_ir() {
    // print(())
    let node = make_call("print", vec![make_unit_lit()], Span::new(0, 10));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CCall {
            target_name, args, ..
        } => {
            assert_eq!(target_name, "print"); // Core IR 0.3: canonical name
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], CoreTerm::CUnitLit { .. }));
        }
        other => panic!("Expected CCall, got {:?}", other),
    }
}

#[test]
fn test_lower_call_with_multiple_args() {
    // add((), ())
    let node = make_call(
        "add",
        vec![make_unit_lit(), make_unit_lit()],
        Span::new(0, 15),
    );

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CCall {
            target_name, args, ..
        } => {
            assert_eq!(target_name, "add"); // Core IR 0.3: canonical name
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected CCall, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPLEX STRUCTURE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_nested_if() {
    // if () then (if () then () else ()) else ()
    let inner_if = make_if(
        make_unit_lit(),
        make_unit_lit(),
        make_unit_lit(),
        Span::new(11, 35),
    );

    let outer_if = make_if(inner_if, make_unit_lit(), make_unit_lit(), Span::new(0, 45));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&outer_if, &mut ctx);
    assert!(result.is_ok());

    // Verify structure
    match result.unwrap() {
        CoreTerm::CIf { cond, .. } => {
            assert!(matches!(*cond, CoreTerm::CIf { .. }));
        }
        other => panic!("Expected CIf, got {:?}", other),
    }
}

#[test]
fn test_lower_call_with_nested_calls() {
    // print(add((), ()))
    let inner_call = make_call(
        "add",
        vec![make_unit_lit(), make_unit_lit()],
        Span::new(6, 18),
    );

    let outer_call = make_call("print", vec![inner_call], Span::new(0, 19));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&outer_call, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CCall { args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], CoreTerm::CCall { .. }));
        }
        other => panic!("Expected CCall, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR HANDLING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_error_unknown_function() {
    // unknown_func(())
    let node = make_call("unknown_func", vec![make_unit_lit()], Span::new(0, 20));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("unknown function 'unknown_func'"));
}

#[test]
fn test_error_wrong_arity() {
    // print((), ()) -- print expects 1 arg
    let node = make_call(
        "print",
        vec![make_unit_lit(), make_unit_lit()],
        Span::new(0, 20),
    );

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("expects 1 arguments"));
}

#[test]
fn test_error_missing_required_field() {
    // Create a lambda without param field
    let mut fields = HashMap::new();
    fields.insert(
        "body".to_string(),
        SchemaValue::Node(Box::new(make_unit_lit())),
    );

    let node = SchemaAstNode {
        kind: "Lam".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 10),
    };

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("missing required field 'param'"));
}

#[test]
fn test_error_wrong_field_type() {
    // Create a lambda with wrong type for param
    let mut fields = HashMap::new();
    fields.insert(
        "param".to_string(),
        SchemaValue::Node(Box::new(make_unit_lit())), // Should be Token
    );
    fields.insert(
        "body".to_string(),
        SchemaValue::Node(Box::new(make_unit_lit())),
    );

    let node = SchemaAstNode {
        kind: "Lam".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 10),
    };

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("must be a Token"));
}

#[test]
fn test_error_unknown_node_kind() {
    let node = SchemaAstNode {
        kind: "UnknownKind".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 10),
    };

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("unknown schema node kind"));
}

// ═══════════════════════════════════════════════════════════════════════════
// DETERMINISM TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_determinism_simple() {
    let node = make_unit_lit();

    let result1 = {
        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };
        lower_to_core_ir(&node, &mut ctx).unwrap()
    };

    let result2 = {
        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };
        lower_to_core_ir(&node, &mut ctx).unwrap()
    };

    assert_eq!(result1, result2);
}

#[test]
fn test_determinism_complex() {
    // \x -> if () then print(()) else add((), ())
    let then_branch = make_call("print", vec![make_unit_lit()], Span::new(18, 30));
    let else_branch = make_call(
        "add",
        vec![make_unit_lit(), make_unit_lit()],
        Span::new(36, 50),
    );
    let if_expr = make_if(make_unit_lit(), then_branch, else_branch, Span::new(6, 50));
    let lambda = make_lam("x", if_expr, Span::new(0, 50));

    let result1 = {
        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };
        lower_to_core_ir(&lambda, &mut ctx).unwrap()
    };

    let result2 = {
        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };
        lower_to_core_ir(&lambda, &mut ctx).unwrap()
    };

    assert_eq!(result1, result2);
}

// ═══════════════════════════════════════════════════════════════════════════
// BUNDLE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_to_bundle_simple() {
    let node = make_unit_lit();
    let result = lower_to_bundle(&node, make_test_registry());

    assert!(result.is_ok());
    let bundle = result.unwrap();
    assert_eq!(bundle.version.as_ref(), "0.3");
    assert!(matches!(bundle.core_term, CoreTerm::CUnitLit { .. }));
}

#[test]
fn test_lower_to_bundle_complex() {
    // \x -> print(())
    let call = make_call("print", vec![make_unit_lit()], Span::new(6, 16));
    let lambda = make_lam("x", call, Span::new(0, 16));

    let result = lower_to_bundle(&lambda, make_test_registry());

    assert!(result.is_ok());
    let bundle = result.unwrap();
    assert_eq!(bundle.version.as_ref(), "0.3");
    assert!(matches!(bundle.core_term, CoreTerm::CLam { .. }));
}

// ═══════════════════════════════════════════════════════════════════════════
// SCOPE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_scope_binds_lambda_parameter() {
    // The lambda should bind its parameter in the body scope
    // This is tested indirectly through successful lowering
    let node = make_lam("x", make_unit_lit(), Span::new(0, 10));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    // Should not error even though we're binding x
    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());
}

#[test]
fn test_scope_nested_lambdas() {
    // \x -> \y -> () should bind both x and y properly
    let inner = make_lam("y", make_unit_lit(), Span::new(6, 15));
    let outer = make_lam("x", inner, Span::new(0, 15));

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&outer, &mut ctx);
    assert!(result.is_ok());
}
