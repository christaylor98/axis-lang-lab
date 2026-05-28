use axis_lang_lab::frontend::parser::parse;
use axis_lang_lab::frontend::parser_lex::lex;
use axis_lang_lab::ir::core_ir::encode_capnp;
use axis_lang_lab::lowering::minimal_lowering::lower_minimal_function;
use axis_lang_lab::validation::core_ir::validate_minimal_core_bundle;

#[test]
fn wave2_end_to_end_simple_if() {
    let src = "fn test() { if {} { {} } else { {} } }";

    let tokens = lex(src).expect("lexer failed");
    let ast = parse(&tokens).expect("parser failed");
    let bundle = lower_minimal_function(ast).expect("lowering failed");

    validate_minimal_core_bundle(&bundle).expect("validation failed");

    let bytes = encode_capnp(&bundle).expect("encoding failed");
    assert!(!bytes.is_empty(), "encoded bytes should not be empty");
}

#[test]
fn wave2_end_to_end_nested_if() {
    let src = "fn test() { if {} { if {} { {} } else { {} } } else { {} } }";

    let tokens = lex(src).expect("lexer failed");
    let ast = parse(&tokens).expect("parser failed");
    let bundle = lower_minimal_function(ast).expect("lowering failed");

    validate_minimal_core_bundle(&bundle).expect("validation failed");

    let bytes = encode_capnp(&bundle).expect("encoding failed");
    assert!(!bytes.is_empty(), "encoded bytes should not be empty");
}

#[test]
fn wave2_end_to_end_if_in_block() {
    let src = "fn test() { {}; if {} { {} } else { {} }; {} }";

    let tokens = lex(src).expect("lexer failed");
    let ast = parse(&tokens).expect("parser failed");
    let bundle = lower_minimal_function(ast).expect("lowering failed");

    validate_minimal_core_bundle(&bundle).expect("validation failed");

    let bytes = encode_capnp(&bundle).expect("encoding failed");
    assert!(!bytes.is_empty(), "encoded bytes should not be empty");
}

#[test]
fn wave1_backward_compat_empty_function() {
    let src = "fn test() {}";

    let tokens = lex(src).expect("lexer failed");
    let ast = parse(&tokens).expect("parser failed");
    let bundle = lower_minimal_function(ast).expect("lowering failed");

    validate_minimal_core_bundle(&bundle).expect("validation failed");

    let bytes = encode_capnp(&bundle).expect("encoding failed");
    assert!(!bytes.is_empty(), "encoded bytes should not be empty");
}

#[test]
fn wave1_backward_compat_multi_expr() {
    let src = "fn test() { {}; {}; {} }";

    let tokens = lex(src).expect("lexer failed");
    let ast = parse(&tokens).expect("parser failed");
    let bundle = lower_minimal_function(ast).expect("lowering failed");

    validate_minimal_core_bundle(&bundle).expect("validation failed");

    let bytes = encode_capnp(&bundle).expect("encoding failed");
    assert!(!bytes.is_empty(), "encoded bytes should not be empty");
}
