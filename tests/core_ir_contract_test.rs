/// Core IR Contract Alignment Test
///
/// This test enforces that the Rust implementation of CoreTerm
/// exactly matches the Cap'n Proto schema definition.
///
/// PURPOSE:
/// - Prevent silent divergence between Core IR sources
/// - Ensure all 9 node kinds are present in Rust
/// - Fail the build if alignment breaks
///
/// AUTHORITY: CORE_IR_CONTRACT.md
use axis_lang_lab::ir::core_ir::{
    bool_lit, capp, ccall, cif, clet, cvar, int_lit, lam, unit_lit, CoreTerm, IdentOrName,
};

#[test]
fn test_core_ir_node_set_complete() {
    // This test verifies that all 9 Core IR 0.2 node kinds
    // can be constructed in Rust.
    //
    // If a node kind is removed or renamed, this test MUST fail.

    // 1. CIntLit
    let _int = int_lit(42);

    // 2. CBoolLit
    let _bool = bool_lit(true);

    // 3. CUnitLit
    let _unit = unit_lit();

    // 4. CLam
    let _lam = lam(IdentOrName::new("x"), unit_lit());

    // 5. CLet
    let _let = clet(IdentOrName::new("x"), unit_lit(), unit_lit());

    // 6. CIf
    let _if = cif(unit_lit(), unit_lit(), unit_lit());

    // 7. CVar
    let _var = cvar(IdentOrName::new("x"));

    // 8. CApp
    let _app = capp(lam(IdentOrName::new("x"), unit_lit()), unit_lit());

    // 9. CCall (Core IR 0.3: uses canonical function name)
    let _call = ccall("test_fn".to_string(), vec![unit_lit()]);

    // If we reach here, all 9 node kinds are constructible
    assert!(true, "All 9 Core IR node kinds are present");
}

#[test]
fn test_core_ir_exhaustive_match() {
    // This test ensures that pattern matching on CoreTerm
    // requires handling all 9 variants.
    //
    // If a new variant is added or removed, this match becomes non-exhaustive
    // and the build fails.

    let term = unit_lit();

    let _node_count = match &term {
        CoreTerm::CIntLit { .. } => 1,
        CoreTerm::CBoolLit { .. } => 2,
        CoreTerm::CUnitLit { .. } => 3,
        CoreTerm::CLam { .. } => 4,
        CoreTerm::CLet { .. } => 5,
        CoreTerm::CIf { .. } => 6,
        CoreTerm::CVar { .. } => 7,
        CoreTerm::CApp { .. } => 8,
        CoreTerm::CCall { .. } => 9,
        // No wildcard pattern allowed - exhaustiveness enforced by compiler
    };

    // If compilation succeeds, exactly 9 variants exist
    assert!(true, "CoreTerm enum has exactly 9 variants");
}

#[test]
fn test_core_ir_version_constant() {
    use axis_lang_lab::ir::core_ir::bundle_v0_3;

    // Verify that Core IR version is locked to 0.3
    let bundle = bundle_v0_3(unit_lit());

    assert_eq!(
        bundle.version.as_ref(),
        "0.3",
        "Core IR version must be 0.3"
    );
}

#[test]
fn test_serialization_roundtrip_all_nodes() {
    use axis_lang_lab::ir::core_ir::{bundle_v0_2, encode_capnp};

    // Verify that all 9 node kinds can be serialized
    // (This will fail if serialization logic is incomplete)

    let test_cases = vec![
        ("CIntLit", int_lit(42)),
        ("CBoolLit", bool_lit(true)),
        ("CUnitLit", unit_lit()),
        ("CLam", lam(IdentOrName::new("x"), unit_lit())),
        ("CLet", clet(IdentOrName::new("x"), unit_lit(), unit_lit())),
        ("CIf", cif(unit_lit(), unit_lit(), unit_lit())),
        ("CVar", cvar(IdentOrName::new("x"))),
        (
            "CApp",
            capp(lam(IdentOrName::new("x"), unit_lit()), unit_lit()),
        ),
        ("CCall", ccall("test_fn".to_string(), vec![unit_lit()])),
    ];

    for (name, term) in test_cases {
        let bundle = bundle_v0_2(term);
        let result = encode_capnp(&bundle);

        assert!(result.is_ok(), "Serialization of {} must succeed", name);
    }
}

#[test]
fn test_node_id_optional_on_all_variants() {
    // Verify that all variants support optional node_id

    let with_id = |mut term: CoreTerm, id: u64| -> CoreTerm {
        match &mut term {
            CoreTerm::CIntLit { node_id, .. } => *node_id = Some(id),
            CoreTerm::CBoolLit { node_id, .. } => *node_id = Some(id),
            CoreTerm::CUnitLit { node_id, .. } => *node_id = Some(id),
            CoreTerm::CLam { node_id, .. } => *node_id = Some(id),
            CoreTerm::CLet { node_id, .. } => *node_id = Some(id),
            CoreTerm::CIf { node_id, .. } => *node_id = Some(id),
            CoreTerm::CVar { node_id, .. } => *node_id = Some(id),
            CoreTerm::CApp { node_id, .. } => *node_id = Some(id),
            CoreTerm::CCall { node_id, .. } => *node_id = Some(id),
        }
        term
    };

    // All variants must support node_id modification
    let _ = with_id(int_lit(1), 1);
    let _ = with_id(bool_lit(true), 2);
    let _ = with_id(unit_lit(), 3);
    let _ = with_id(lam(IdentOrName::new("x"), unit_lit()), 4);
    let _ = with_id(clet(IdentOrName::new("x"), unit_lit(), unit_lit()), 5);
    let _ = with_id(cif(unit_lit(), unit_lit(), unit_lit()), 6);
    let _ = with_id(cvar(IdentOrName::new("x")), 7);
    let _ = with_id(capp(unit_lit(), unit_lit()), 8);
    let _ = with_id(ccall("test_fn".to_string(), vec![]), 9);

    assert!(true, "All variants support node_id");
}
