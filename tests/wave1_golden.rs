use std::fs;

#[test]
fn two_expr_block_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident { value: "f2".into() },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                ],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/phase1_2.bin").expect("failed to read golden file");
    assert!(bytes == golden, "two-expression block output mismatch");
}

#[test]
fn three_expr_block_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident { value: "f3".into() },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                ],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/phase1_3.bin").expect("failed to read golden file");
    assert!(bytes == golden, "three-expression block output mismatch");
}
