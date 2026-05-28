use axis_lang_lab as crate_under_test;

use crate_under_test::ir::core_ir::{
    bundle_v0_2, lam, lam_dummy_unit, unit_lit, CoreBundle, IdentOrName,
};
use crate_under_test::validation::core_ir::{validate_minimal_core_bundle, CoreIrValidationError};

#[test]
fn phase1_lowering_produces_valid_core_bundle() {
    use crate_under_test::frontend::ast::{EmptyBlock, FunctionDecl, Ident, NoParams};

    let f = FunctionDecl {
        name: Ident {
            value: "example".into(),
        },
        params: NoParams,
        body: EmptyBlock,
    };

    let res = crate_under_test::lowering::minimal_lowering::lower_minimal_function(f);
    assert!(
        res.is_ok(),
        "phase1 lowering should succeed and validation should pass"
    );
}

#[test]
fn validation_rejects_version_mismatch() {
    let bundle = CoreBundle {
        version: "0.1".into(),
        core_term: lam_dummy_unit(),
    };
    let err = validate_minimal_core_bundle(&bundle).unwrap_err();
    assert_eq!(
        err,
        CoreIrValidationError::VersionMismatch {
            expected: "0.3",
            got: "0.1".into()
        }
    );
}

#[test]
fn validation_rejects_non_clam_top_level() {
    let bundle = bundle_v0_2(unit_lit());
    let err = validate_minimal_core_bundle(&bundle).unwrap_err();
    assert_eq!(err, CoreIrValidationError::TopLevelNotCLam);
}

#[test]
fn validation_rejects_clam_body_not_unit_lit() {
    // Construct a CLam whose body is another CLam (not a CUnitLit).
    let nested = lam_dummy_unit();
    let outer = lam(IdentOrName::dummy(), nested);
    let bundle = bundle_v0_2(outer);
    // With Wave 1 sequencing, nested CLam chains that end in CUnitLit are accepted.
    let res = validate_minimal_core_bundle(&bundle);
    assert!(
        res.is_ok(),
        "nested CLam chain should be accepted after Wave 1"
    );
}
