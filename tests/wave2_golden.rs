use std::fs;

#[test]
fn simple_if_unit_branches_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "test_if".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::If {
                    cond: Box::new(axis_lang_lab::frontend::ast::Expr::UnitLit),
                    then_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                    },
                    else_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                    },
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave2_simple_if.bin").expect("failed to read golden file");
    assert!(
        bytes == golden,
        "simple if with unit branches output mismatch"
    );
}

#[test]
fn nested_if_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "test_nested".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::If {
                    cond: Box::new(axis_lang_lab::frontend::ast::Expr::UnitLit),
                    then_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::If {
                            cond: Box::new(axis_lang_lab::frontend::ast::Expr::UnitLit),
                            then_block: axis_lang_lab::frontend::ast::Block {
                                exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                            },
                            else_block: axis_lang_lab::frontend::ast::Block {
                                exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                            },
                        }],
                    },
                    else_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                    },
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave2_nested_if.bin").expect("failed to read golden file");
    assert!(bytes == golden, "nested if output mismatch");
}

#[test]
fn if_in_multi_expr_block_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "test_multi".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                    axis_lang_lab::frontend::ast::Expr::If {
                        cond: Box::new(axis_lang_lab::frontend::ast::Expr::UnitLit),
                        then_block: axis_lang_lab::frontend::ast::Block {
                            exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                        },
                        else_block: axis_lang_lab::frontend::ast::Block {
                            exprs: vec![axis_lang_lab::frontend::ast::Expr::UnitLit],
                        },
                    },
                    axis_lang_lab::frontend::ast::Expr::UnitLit,
                ],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden =
        fs::read("tests/golden/wave2_if_in_block.bin").expect("failed to read golden file");
    assert!(
        bytes == golden,
        "if in multi-expression block output mismatch"
    );
}
