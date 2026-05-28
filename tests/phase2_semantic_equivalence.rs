use std::fs;

#[test]
fn phase2_semantic_equivalence() {
    // Construct a Phase 2 style AST: single-expression block with UnitLit
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident { value: "f".into() },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
            },
        ),
    };

    // Call Phase 1 lowering entrypoint (it accepts Phase 2 single-expression bodies)
    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("phase1 lowering failed for Phase 2 AST");

    // Serialize Core IR via existing encoder
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    // Load existing Phase 1 golden bytes and assert exact equality
    let golden = fs::read("tests/golden/phase1.bin").expect("failed to read golden file");
    assert!(
        bytes == golden,
        "phase2 output drifted from phase1 golden (byte-for-byte mismatch)"
    );
}
