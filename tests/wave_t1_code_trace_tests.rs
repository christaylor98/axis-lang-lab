// Wave T1: Code Trace Inspection Tests
//
// Validates --inspect code_trace functionality

use axis_lang_lab::frontend::{ast_builder, lexer_engine, parser_runtime, schema_ast};
use axis_lang_lab::frontend::{lexspec_load, parserspec_load, schema_load};
use axis_lang_lab::introspection::code_trace;
use axis_lang_lab::lowering::nf_lowering;
use axis_lang_lab::registry::Registry;
use std::path::PathBuf;

#[test]
#[ignore = "Test bypasses normalization - semantic-surface-0 AppExpr needs normalization to convert list-form to binary NF form"]
fn test_code_trace_simple_int() {
    // Load specs
    let lexer_spec = lexspec_load::load_spec(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-lexer.yaml",
    ))
    .expect("load lexer spec");

    let parser_spec = parserspec_load::load_parser_spec(
        &PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        &lexer_spec,
    )
    .expect("load parser spec");

    let ast_schema = schema_load::load_schema_from_file(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-ast.yaml",
    ))
    .expect("load schema");

    let registry = Registry::load_active().expect("load registry");

    // Compile source
    let source = "42";
    let tokens = lexer_engine::lex_with_spec(&lexer_spec, source).expect("lex");

    let parse_tree = parser_runtime::parse_with_spec(&parser_spec, &tokens).expect("parse");

    let generic_ast = ast_builder::build_generic_ast(&parse_tree).expect("build generic AST");

    let schema_ast_node =
        schema_ast::project_schema_ast(&generic_ast, &ast_schema).expect("project schema AST");

    let core_ir =
        nf_lowering::lower_nf_to_bundle(&schema_ast_node, registry).expect("lower to Core IR");

    // Generate code trace
    let entries = code_trace::generate_code_trace(&tokens, &parse_tree, &schema_ast_node, &core_ir)
        .expect("generate code trace");

    // Validate trace entries
    assert!(!entries.is_empty(), "trace should have entries");

    // Should have IntLit entry (wrappers filtered)
    let int_lit_entries: Vec<_> = entries.iter().filter(|e| e.ast_kind == "IntLit").collect();
    assert_eq!(
        int_lit_entries.len(),
        1,
        "should have exactly one IntLit entry"
    );

    let int_lit = &int_lit_entries[0];
    assert_eq!(int_lit.span.start, 0);
    assert_eq!(int_lit.span.end, 2);

    // Should have tokens
    assert!(!int_lit.tokens.is_empty(), "IntLit should have tokens");
    assert_eq!(int_lit.tokens[0].lexeme, "42");

    // Should have Core IR correlation
    assert!(
        !int_lit.core_ir.is_empty(),
        "should have Core IR correlation"
    );
    assert_eq!(int_lit.core_ir[0].term_type, "CIntLit");
}

#[test]
#[ignore = "Test bypasses normalization - semantic-surface-0 AppExpr needs normalization to convert list-form to binary NF form"]
fn test_code_trace_if_expression() {
    // Load specs
    let lexer_spec = lexspec_load::load_spec(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-lexer.yaml",
    ))
    .expect("load lexer spec");

    let parser_spec = parserspec_load::load_parser_spec(
        &PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        &lexer_spec,
    )
    .expect("load parser spec");

    let ast_schema = schema_load::load_schema_from_file(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-ast.yaml",
    ))
    .expect("load schema");

    let registry = Registry::load_active().expect("load registry");

    // Compile source
    let source = "if true then 1 else 0";
    let tokens = lexer_engine::lex_with_spec(&lexer_spec, source).expect("lex");

    let parse_tree = parser_runtime::parse_with_spec(&parser_spec, &tokens).expect("parse");

    let generic_ast = ast_builder::build_generic_ast(&parse_tree).expect("build generic AST");

    let schema_ast_node =
        schema_ast::project_schema_ast(&generic_ast, &ast_schema).expect("project schema AST");

    let core_ir =
        nf_lowering::lower_nf_to_bundle(&schema_ast_node, registry).expect("lower to Core IR");

    // Generate code trace
    let entries = code_trace::generate_code_trace(&tokens, &parse_tree, &schema_ast_node, &core_ir)
        .expect("generate code trace");

    // Validate trace entries
    assert!(!entries.is_empty(), "trace should have entries");

    // Should have IfExpr, BoolLit, and two IntLit entries
    let if_entries: Vec<_> = entries.iter().filter(|e| e.ast_kind == "IfExpr").collect();
    assert_eq!(if_entries.len(), 1, "should have exactly one IfExpr entry");

    let bool_entries: Vec<_> = entries.iter().filter(|e| e.ast_kind == "BoolLit").collect();
    assert_eq!(
        bool_entries.len(),
        1,
        "should have exactly one BoolLit entry"
    );

    let int_entries: Vec<_> = entries.iter().filter(|e| e.ast_kind == "IntLit").collect();
    assert_eq!(
        int_entries.len(),
        2,
        "should have exactly two IntLit entries"
    );

    // Verify no wrapper nodes (Expr, AtomicExpr, AppExpr) appear
    for entry in &entries {
        assert!(
            !matches!(entry.ast_kind.as_str(), "Expr" | "AtomicExpr" | "AppExpr"),
            "wrapper nodes should be filtered: {}",
            entry.ast_kind
        );
    }
}

#[test]
#[ignore = "Test bypasses normalization - semantic-surface-0 AppExpr needs normalization to convert list-form to binary NF form"]
fn test_code_trace_prerequisite_checks() {
    // Empty tokens should fail
    let empty_tokens: Vec<axis_lang_lab::frontend::token::Token> = vec![];
    let lexer_spec = lexspec_load::load_spec(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-lexer.yaml",
    ))
    .expect("load lexer spec");
    let parser_spec = parserspec_load::load_parser_spec(
        &PathBuf::from("axis-surface-0-config/semantic-surface-0-parse.yaml"),
        &lexer_spec,
    )
    .expect("load parser spec");
    let source = "42";
    let tokens = lexer_engine::lex_with_spec(&lexer_spec, source).expect("lex");
    let parse_tree = parser_runtime::parse_with_spec(&parser_spec, &tokens).expect("parse");
    let ast_schema = schema_load::load_schema_from_file(&PathBuf::from(
        "axis-surface-0-config/semantic-surface-0-ast.yaml",
    ))
    .expect("load schema");
    let generic_ast = ast_builder::build_generic_ast(&parse_tree).expect("build generic AST");
    let schema_ast_node =
        schema_ast::project_schema_ast(&generic_ast, &ast_schema).expect("project schema AST");
    let registry = Registry::load_active().expect("load registry");
    let core_ir =
        nf_lowering::lower_nf_to_bundle(&schema_ast_node, registry).expect("lower to Core IR");

    let result =
        code_trace::generate_code_trace(&empty_tokens, &parse_tree, &schema_ast_node, &core_ir);

    assert!(result.is_err(), "should fail with empty tokens");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("lexing not completed"),
        "error message should mention lexing"
    );
}
