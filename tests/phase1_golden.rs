use std::fs;

#[test]
fn phase1_golden() {
    // Construct AST directly (no lexer/parser)
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident { value: "f".into() },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::EmptyBlock,
    };

    // Call lowering and assert success
    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");

    // Serialize Core IR via existing Cap'n Proto encoder
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    // Read golden file and compare byte-for-byte
    let golden = fs::read("tests/golden/phase1.bin").expect("failed to read golden file");
    assert!(bytes == golden, "phase1 output mismatch");
}
