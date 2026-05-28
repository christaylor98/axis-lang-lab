use std::fs;

fn main() {
    // Test 1: Simple if with unit branches
    let func1 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let bundle1 = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func1)
        .expect("lowering failed for test 1");
    let bytes1 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle1)
        .expect("encode failed for test 1");
    fs::write("tests/golden/wave2_simple_if.bin", &bytes1)
        .expect("failed to write wave2_simple_if.bin");
    println!("Generated tests/golden/wave2_simple_if.bin");

    // Test 2: Nested if
    let func2 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let bundle2 = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func2)
        .expect("lowering failed for test 2");
    let bytes2 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle2)
        .expect("encode failed for test 2");
    fs::write("tests/golden/wave2_nested_if.bin", &bytes2)
        .expect("failed to write wave2_nested_if.bin");
    println!("Generated tests/golden/wave2_nested_if.bin");

    // Test 3: If in multi-expression block
    let func3 = axis_lang_lab::frontend::ast::FunctionDecl {
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

    let bundle3 = axis_lang_lab::lowering::minimal_lowering::lower_minimal_function(func3)
        .expect("lowering failed for test 3");
    let bytes3 = axis_lang_lab::ir::core_ir::encode_capnp(&bundle3)
        .expect("encode failed for test 3");
    fs::write("tests/golden/wave2_if_in_block.bin", &bytes3)
        .expect("failed to write wave2_if_in_block.bin");
    println!("Generated tests/golden/wave2_if_in_block.bin");

    println!("All Wave 2 golden files generated successfully!");
}
