use std::fs;

#[test]
fn wave3_det0_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "det0_entry".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                    name: axis_lang_lab::frontend::ast::Ident {
                        value: "det0".into(),
                    },
                    args: vec![],
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave3_det0.bin").expect("failed to read golden file");
    assert!(bytes == golden, "det0 golden mismatch");
}

#[test]
fn wave3_rand0_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "rand0_entry".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                    name: axis_lang_lab::frontend::ast::Ident {
                        value: "rand0".into(),
                    },
                    args: vec![],
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave3_rand0.bin").expect("failed to read golden file");
    assert!(bytes == golden, "rand0 golden mismatch");
}

#[test]
fn wave3_add_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "add_entry".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                    name: axis_lang_lab::frontend::ast::Ident {
                        value: "add".into(),
                    },
                    args: vec![
                        axis_lang_lab::frontend::ast::Expr::Call {
                            name: axis_lang_lab::frontend::ast::Ident {
                                value: "det0".into(),
                            },
                            args: vec![],
                        },
                        axis_lang_lab::frontend::ast::Expr::Call {
                            name: axis_lang_lab::frontend::ast::Ident {
                                value: "det0".into(),
                            },
                            args: vec![],
                        },
                    ],
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave3_add.bin").expect("failed to read golden file");
    assert!(bytes == golden, "add golden mismatch");
}

#[test]
fn wave3_cond_golden() {
    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "cond_entry".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::If {
                    cond: Box::new(axis_lang_lab::frontend::ast::Expr::Call {
                        name: axis_lang_lab::frontend::ast::Ident {
                            value: "det0".into(),
                        },
                        args: vec![],
                    }),
                    then_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                            name: axis_lang_lab::frontend::ast::Ident {
                                value: "det0".into(),
                            },
                            args: vec![],
                        }],
                    },
                    else_block: axis_lang_lab::frontend::ast::Block {
                        exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                            name: axis_lang_lab::frontend::ast::Ident {
                                value: "rand0".into(),
                            },
                            args: vec![],
                        }],
                    },
                }],
            },
        ),
    };

    let bundle = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func)
        .expect("lowering failed");
    let bytes = axis_lang_lab::ir::core_ir::encode_capnp(&bundle).expect("encode failed");

    let golden = fs::read("tests/golden/wave3_cond.bin").expect("failed to read golden file");
    assert!(bytes == golden, "cond golden mismatch");
}
