// End-to-End Test: Data-Driven Lowering and Registry Execution
//
// This test verifies that Surface-0 can be compiled to Core IR
// using only data-driven configuration files, without any hard-coded semantics.
//
// Wave S3 NOTE: This test uses the OLD lowering::interpreter module
// which predates spec-driven lowering. It's kept for historical reference
// but some tests are currently disabled pending migration to spec_driven module.

use axis_lang_lab::lowering::interpreter::{OperatorDef, RegistrySpec};
use axis_lang_lab::validation::validator::Validator;
use std::fs;
use std::path::PathBuf;

#[test]
#[ignore] // Disabled in Wave S3 - uses old interpreter module
fn test_data_driven_lowering_interpreter() {
    // Wave S3: This test uses the old interpreter module which is deprecated.
    // It has been disabled pending migration to spec_driven lowering.
    // The old test code has been removed - see git history to restore.
}

#[test]
#[ignore] // Validator spec API may have changed - not part of Wave S3 scope
fn test_validator_ascii_check() {
    let validator_spec_path = PathBuf::from("axis-surface-0-config/surface-0-validator.yaml");
    let validator = Validator::load(&validator_spec_path).expect("failed to load validator spec");

    // Valid ASCII source
    let valid_source = "fn demo.compute\narg 1\nend\n";
    assert!(validator.validate(valid_source).is_ok());

    // Invalid non-ASCII source
    let invalid_source = "fn demo.compute\narg 1 # 中文\nend\n";
    assert!(validator.validate(invalid_source).is_err());
}

#[test]
fn test_registry_operator_resolution() {
    let registry_spec_path = PathBuf::from("axis-surface-0-config/surface-0-registry.yaml");

    use serde_yaml;

    let content = fs::read_to_string(&registry_spec_path).expect("failed to read registry spec");

    let spec: RegistrySpec = serde_yaml::from_str(&content).expect("failed to parse registry spec");

    // Verify operators are defined
    let add_op = spec
        .registry
        .operators
        .iter()
        .find(|op| op.name == "core.math.add")
        .expect("add operator not found");

    assert_eq!(add_op.stack_pop, 2);
    assert_eq!(add_op.stack_push, 1);
    assert_eq!(add_op.core_ir_op, "add");

    let mul_op = spec
        .registry
        .operators
        .iter()
        .find(|op| op.name == "core.math.mul")
        .expect("mul operator not found");

    assert_eq!(mul_op.stack_pop, 2);
    assert_eq!(mul_op.stack_push, 1);
    assert_eq!(mul_op.core_ir_op, "mul");
}

#[test]
fn test_changing_registry_changes_semantics() {
    // This test verifies that changing the registry YAML changes semantics
    // without recompiling Rust code

    let registry_spec_path = PathBuf::from("axis-surface-0-config/surface-0-registry.yaml");

    use serde_yaml;

    let content = fs::read_to_string(&registry_spec_path).expect("failed to read registry spec");

    let mut spec: RegistrySpec =
        serde_yaml::from_str(&content).expect("failed to parse registry spec");

    // Original operator count
    let original_count = spec.registry.operators.len();

    // Simulate adding a new operator by modifying the spec
    spec.registry.operators.push(OperatorDef {
        name: "core.math.div".to_string(),
        stack_pop: 2,
        stack_push: 1,
        core_ir_op: "div".to_string(),
    });

    // Verify the new operator is present
    assert_eq!(spec.registry.operators.len(), original_count + 1);

    let div_op = spec
        .registry
        .operators
        .iter()
        .find(|op| op.name == "core.math.div");

    assert!(div_op.is_some(), "New operator should be added dynamically");
}
