// Schema Lowering Usage Example: Schema AST → Core IR Lowering
//
// This example demonstrates the complete lowering pipeline from Schema AST to Core IR.
// It shows how to:
// - Build Schema AST nodes
// - Create a lowering context
// - Lower to Core IR
// - Handle errors
// - Serialize to Core IR bundle

use axis_lang_lab::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::lowering::schema_lowering::{
    lower_to_bundle, lower_to_core_ir, LoweringContext, Scope,
};
use axis_lang_lab::registry::{Registry, RegistryEntry};
use std::collections::HashMap;

fn main() {
    println!("Schema AST → Core IR Lowering Example\n");

    // ═════════════════════════════════════════════════════════════════
    // Step 1: Set up Registry
    // ═════════════════════════════════════════════════════════════════

    let registry = Registry {
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
    };

    println!(
        "✓ Registry loaded with {} functions",
        registry.entries.len()
    );
    println!("  - print (id=1, arity=1)");
    println!("  - add (id=2, arity=2)\n");

    // ═════════════════════════════════════════════════════════════════
    // Step 2: Build Schema AST nodes manually
    // ═════════════════════════════════════════════════════════════════

    // Example: \x -> if () then print(()) else add((), ())

    println!("Building Schema AST: \\x -> if () then print(()) else add((), ())\n");

    // Build unit literals
    let unit_lit = SchemaAstNode {
        kind: "UnitLit".to_string(),
        fields: HashMap::new(),
        span: Span::new(0, 2),
        annotations: Vec::new(),
    };

    // Build print(()) call
    let mut print_fields = HashMap::new();
    print_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "print".to_string(),
            span: Span::new(23, 28),
        }),
    );
    print_fields.insert(
        "args".to_string(),
        SchemaValue::Nodes(vec![unit_lit.clone()]),
    );
    let print_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: print_fields,
        span: Span::new(23, 33),
        annotations: Vec::new(),
    };

    // Build add((), ()) call
    let mut add_fields = HashMap::new();
    add_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "add".to_string(),
            span: Span::new(39, 42),
        }),
    );
    add_fields.insert(
        "args".to_string(),
        SchemaValue::Nodes(vec![unit_lit.clone(), unit_lit.clone()]),
    );
    let add_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: add_fields,
        span: Span::new(39, 52),
        annotations: Vec::new(),
    };

    // Build if expression
    let mut if_fields = HashMap::new();
    if_fields.insert(
        "condition".to_string(),
        SchemaValue::Node(Box::new(unit_lit.clone())),
    );
    if_fields.insert(
        "then_branch".to_string(),
        SchemaValue::Node(Box::new(print_call)),
    );
    if_fields.insert(
        "else_branch".to_string(),
        SchemaValue::Node(Box::new(add_call)),
    );
    let if_expr = SchemaAstNode {
        kind: "If".to_string(),
        fields: if_fields,
        span: Span::new(6, 52),
        annotations: Vec::new(),
    };

    // Build lambda
    let mut lam_fields = HashMap::new();
    lam_fields.insert(
        "param".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "x".to_string(),
            span: Span::new(1, 2),
        }),
    );
    lam_fields.insert("body".to_string(), SchemaValue::Node(Box::new(if_expr)));
    let lambda = SchemaAstNode {
        kind: "Lam".to_string(),
        fields: lam_fields,
        span: Span::new(0, 52),
        annotations: Vec::new(),
    };

    println!("✓ Schema AST built:");
    println!("  Root: {}", lambda.kind);
    println!("  Span: {}..{}\n", lambda.span.start, lambda.span.end);

    // ═════════════════════════════════════════════════════════════════
    // Step 3: Lower to Core IR
    // ═════════════════════════════════════════════════════════════════

    let mut ctx = LoweringContext {
        registry: registry.clone(),
        scope: Scope::new(),
    };

    println!("Lowering to Core IR...");
    let core_term_result = lower_to_core_ir(&lambda, &mut ctx);

    match &core_term_result {
        Ok(term) => {
            println!("✓ Lowering successful!");
            println!("  Core IR structure: {:?}", core_term_structure(term));
        }
        Err(e) => {
            println!("✗ Lowering failed: {}", e);
            return;
        }
    }

    // ═════════════════════════════════════════════════════════════════
    // Step 4: Create Core IR Bundle
    // ═════════════════════════════════════════════════════════════════

    println!("\nCreating Core IR Bundle...");
    let bundle_result = lower_to_bundle(&lambda, registry);

    match &bundle_result {
        Ok(bundle) => {
            println!("✓ Bundle created!");
            println!("  Version: {}", bundle.version);
            println!("  Core term: {:?}", core_term_structure(&bundle.core_term));
        }
        Err(e) => {
            println!("✗ Bundle creation failed: {}", e);
            return;
        }
    }

    // ═════════════════════════════════════════════════════════════════
    // Step 5: Demonstrate error handling
    // ═════════════════════════════════════════════════════════════════

    println!("\n═════════════════════════════════════════════════════════");
    println!("Error Handling Examples");
    println!("═════════════════════════════════════════════════════════\n");

    // Example 1: Missing field
    println!("1. Missing required field:");
    let mut bad_lam_fields = HashMap::new();
    bad_lam_fields.insert(
        "body".to_string(),
        SchemaValue::Node(Box::new(unit_lit.clone())),
    );
    // Missing "param" field
    let bad_lam = SchemaAstNode {
        kind: "Lam".to_string(),
        fields: bad_lam_fields,
        span: Span::new(0, 10),
        annotations: Vec::new(),
    };

    let mut ctx = LoweringContext {
        registry: Registry { entries: vec![] },
        scope: Scope::new(),
    };
    match lower_to_core_ir(&bad_lam, &mut ctx) {
        Err(e) => println!("   ✓ Correctly rejected: {}\n", e),
        Ok(_) => println!("   ✗ Should have failed!\n"),
    }

    // Example 2: Unknown function
    println!("2. Unknown function in registry:");
    let mut unknown_call_fields = HashMap::new();
    unknown_call_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "unknown_func".to_string(),
            span: Span::new(0, 12),
        }),
    );
    unknown_call_fields.insert("args".to_string(), SchemaValue::Nodes(vec![]));
    let unknown_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: unknown_call_fields,
        span: Span::new(0, 20),
        annotations: Vec::new(),
    };

    let mut ctx = LoweringContext {
        registry: Registry { entries: vec![] },
        scope: Scope::new(),
    };
    match lower_to_core_ir(&unknown_call, &mut ctx) {
        Err(e) => println!("   ✓ Correctly rejected: {}\n", e),
        Ok(_) => println!("   ✗ Should have failed!\n"),
    }

    // Example 3: Wrong arity
    println!("3. Arity mismatch:");
    let registry_with_print = Registry {
        entries: vec![RegistryEntry {
            name: "print".to_string(),
            arity: 1,
            deterministic: false,
            profiles: vec![],
            id: 1,
        }],
    };

    let mut wrong_arity_fields = HashMap::new();
    wrong_arity_fields.insert(
        "target".to_string(),
        SchemaValue::Token(Token {
            kind: TokenKind::Ident,
            lexeme: "print".to_string(),
            span: Span::new(0, 5),
        }),
    );
    wrong_arity_fields.insert(
        "args".to_string(),
        SchemaValue::Nodes(vec![unit_lit.clone(), unit_lit.clone()]), // print expects 1 arg
    );
    let wrong_arity_call = SchemaAstNode {
        kind: "Call".to_string(),
        fields: wrong_arity_fields,
        span: Span::new(0, 20),
        annotations: Vec::new(),
    };

    let mut ctx = LoweringContext {
        registry: registry_with_print,
        scope: Scope::new(),
    };
    match lower_to_core_ir(&wrong_arity_call, &mut ctx) {
        Err(e) => println!("   ✓ Correctly rejected: {}\n", e),
        Ok(_) => println!("   ✗ Should have failed!\n"),
    }

    println!("═════════════════════════════════════════════════════════");
    println!("Schema lowering complete!");
}

// Helper to describe Core IR structure
fn core_term_structure(term: &axis_lang_lab::ir::core_ir::CoreTerm) -> String {
    use axis_lang_lab::ir::core_ir::CoreTerm;
    match term {
        CoreTerm::CUnitLit { .. } => "CUnitLit".to_string(),
        CoreTerm::CLam { param, body, .. } => {
            format!("CLam({}, {})", param.0, core_term_structure(body))
        }
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            ..
        } => format!(
            "CIf({}, {}, {})",
            core_term_structure(cond),
            core_term_structure(then_branch),
            core_term_structure(else_branch)
        ),
        CoreTerm::CCall {
            target_name, args, ..
        } => {
            let arg_strs: Vec<_> = args.iter().map(core_term_structure).collect();
            format!("CCall({}, [{}])", target_name, arg_strs.join(", "))
        }
        _ => "UnknownTerm".to_string(),
    }
}
