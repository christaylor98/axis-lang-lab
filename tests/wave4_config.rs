use std::fs;
use std::path::PathBuf;

#[test]
fn profile_rejection_via_config() {
    let reg_path = "registries/tmp_profile.axreg";
    let reg_src = "registry tmp 0.1\n\nfn special0\narity 0\ndeterministic true\nprofile special\nend\n";
    fs::write(reg_path, reg_src).expect("write reg");

    let cfg_path = "config/langlab.toml";
    let cfg = r#"active_profile = "default"
registries = ["registries/tmp_profile.axreg"]
"#;
    fs::create_dir_all("config").ok();
    fs::write(cfg_path, cfg).expect("write cfg");

    let func = axis_lang_lab::frontend::ast::FunctionDecl {
        name: axis_lang_lab::frontend::ast::Ident {
            value: "special_entry".into(),
        },
        params: axis_lang_lab::frontend::ast::NoParams,
        body: axis_lang_lab::frontend::ast::FunctionBody::Block(
            axis_lang_lab::frontend::ast::Block {
                exprs: vec![axis_lang_lab::frontend::ast::Expr::Call {
                    name: axis_lang_lab::frontend::ast::Ident {
                        value: "special0".into(),
                    },
                    args: vec![],
                }],
            },
        ),
    };

    let res = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func);

    assert!(res.is_err());

    let _ = fs::remove_file(reg_path);
    let _ = fs::remove_file(cfg_path);
}

#[test]
fn ambiguous_registry_definitions_rejected() {
    let a = "registries/tmp_a.axreg";
    let b = "registries/tmp_b.axreg";
    let src = "registry x 0.1\n\nfn dup\narity 0\ndeterministic true\nprofile all\nend\n";
    fs::write(a, src).expect("write a");
    fs::write(b, src).expect("write b");

    let paths = vec![PathBuf::from(a), PathBuf::from(b)];
    let res = axis_lang_lab::registry::Registry::load_from_paths(&paths);
    assert!(res.is_err());

    let _ = fs::remove_file(a);
    let _ = fs::remove_file(b);
}

#[test]
fn registry_order_changes_ids() {
    let a = "registries/tmp_ord_a.axreg";
    let b = "registries/tmp_ord_b.axreg";
    let src_a = "registry a 0.1\n\nfn fa\narity 0\ndeterministic true\nprofile all\nend\n";
    let src_b = "registry b 0.1\n\nfn fb\narity 0\ndeterministic true\nprofile all\nend\n";
    fs::write(a, src_a).expect("write a");
    fs::write(b, src_b).expect("write b");

    let p1 = vec![PathBuf::from(a), PathBuf::from(b)];
    let r1 = axis_lang_lab::registry::Registry::load_from_paths(&p1).expect("load1");
    let id_fa_first = r1
        .entries
        .iter()
        .find(|e| e.name == "fa")
        .map(|e| e.id)
        .unwrap();

    let p2 = vec![PathBuf::from(b), PathBuf::from(a)];
    let r2 = axis_lang_lab::registry::Registry::load_from_paths(&p2).expect("load2");
    let id_fa_second = r2
        .entries
        .iter()
        .find(|e| e.name == "fa")
        .map(|e| e.id)
        .unwrap();

    assert_ne!(
        id_fa_first, id_fa_second,
        "ids should differ when registry order changes"
    );

    let _ = fs::remove_file(a);
    let _ = fs::remove_file(b);
}

#[test]
fn absence_of_config_preserves_wave3() {
    let _ = fs::remove_file("config/langlab.toml");

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

    let res = axis_lang_lab::lowering::registry_lowering::lower_registry_function(func);
    assert!(res.is_ok());
}
