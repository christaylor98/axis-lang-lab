use std::fs;

fn main() {
    // Create ASTs for registry lowering goldens
    let func1 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let func2 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let func3 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let func4 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let bundle1 =
        axis_lang_lab::lowering::registry_lowering::lower_registry_function(func1)
            .expect("lowering failed for det0");
    let bundle2 =
        axis_lang_lab::lowering::registry_lowering::lower_registry_function(func2)
            .expect("lowering failed for rand0");
    let bundle3 =
        axis_lang_lab::lowering::registry_lowering::lower_registry_function(func3)
            .expect("lowering failed for add");
    let bundle4 =
        axis_lang_lab::lowering::registry_lowering::lower_registry_function(func4)
            .expect("lowering failed for cond");

    let bytes1 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle1).expect("encode failed");
    let bytes2 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle2).expect("encode failed");
    let bytes3 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle3).expect("encode failed");
    let bytes4 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle4).expect("encode failed");

    fs::create_dir_all("tests/golden").ok();
    fs::write("tests/golden/wave3_det0.bin", &bytes1).expect("write golden");
    fs::write("tests/golden/wave3_rand0.bin", &bytes2).expect("write golden");
    fs::write("tests/golden/wave3_add.bin", &bytes3).expect("write golden");
    fs::write("tests/golden/wave3_cond.bin", &bytes4).expect("write golden");
}
