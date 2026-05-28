use std::fs;

fn main() {
    // Build two bundles using the library crate
    let two = {
        use axis_lang_lab::frontend::ast::{
            Block, Expr, FunctionBody, FunctionDecl, Ident, NoParams,
        };
        use axis_lang_lab::lowering::minimal_lowering::lower_minimal_function;
        let func = FunctionDecl {
            name: Ident { value: "f2".into() },
            params: NoParams,
            body: FunctionBody::Block(Block {
                exprs: vec![Expr::UnitLit, Expr::UnitLit],
            }),
        };
        lower_minimal_function(func).expect("lowering two failed")
    };

    let three = {
        use axis_lang_lab::frontend::ast::{
            Block, Expr, FunctionBody, FunctionDecl, Ident, NoParams,
        };
        use axis_lang_lab::lowering::minimal_lowering::lower_minimal_function;
        let func = FunctionDecl {
            name: Ident { value: "f3".into() },
            params: NoParams,
            body: FunctionBody::Block(Block {
                exprs: vec![Expr::UnitLit, Expr::UnitLit, Expr::UnitLit],
            }),
        };
        lower_minimal_function(func).expect("lowering three failed")
    };

    let two_bytes =
        axis_lang_lab::ir::core_ir::encode_capnp(&two).expect("encode two failed");
    let three_bytes =
        axis_lang_lab::ir::core_ir::encode_capnp(&three).expect("encode three failed");

    fs::create_dir_all("tests/golden").expect("create golden dir");
    fs::write("tests/golden/phase1_2.bin", &two_bytes).expect("write two golden");
    fs::write("tests/golden/phase1_3.bin", &three_bytes).expect("write three golden");

    println!("Wrote tests/golden/phase1_2.bin and phase1_3.bin");
}
