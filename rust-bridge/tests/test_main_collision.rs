// Test: verify that Axis functions named "main" don't collide with Rust fn main()

use axis_rust_bridge::core_ir::CoreTerm;
use std::rc::Rc;

#[test]
fn test_axis_main_renamed_to_avoid_collision() {
    // Create Core IR with a function named "main"
    // Let(main, Lam(x, Var(x)), UnitLit)
    let inner = CoreTerm::Var("x".to_string(), None);
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(inner), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("main".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify that:
    // 1. Axis function "main" is emitted as "axis_main" to avoid collision
    assert!(rust_code.contains("pub fn axis_main("), 
        "Expected 'pub fn axis_main(' but got:\n{}", rust_code);
    
    // 2. Rust entry point fn main() calls axis_main()
    assert!(rust_code.contains("fn main() {"), 
        "Expected 'fn main() {{' but got:\n{}", rust_code);
    assert!(rust_code.contains("axis_main("), 
        "Expected call to 'axis_main(' but got:\n{}", rust_code);
    
    // 3. No duplicate main definitions
    let main_count = rust_code.matches("fn main(").count();
    assert_eq!(main_count, 1, "Should have exactly 1 'fn main(' but found {} in:\n{}", main_count, rust_code);
}

#[test]
fn test_non_main_function_unchanged() {
    // Create Core IR with a function NOT named "main"
    // Let(entry, Lam(x, Var(x)), UnitLit)
    let inner = CoreTerm::Var("x".to_string(), None);
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(inner), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("entry".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify that non-"main" functions are not renamed
    assert!(rust_code.contains("pub fn entry("), 
        "Expected 'pub fn entry(' but got:\n{}", rust_code);
    assert!(!rust_code.contains("pub fn axis_entry("),
        "Should not rename non-main functions but got:\n{}", rust_code);
    
    // Rust entry point should call the actual function name
    assert!(rust_code.contains("entry("), 
        "Expected call to 'entry(' but got:\n{}", rust_code);
}

#[test]
fn test_shim_import_present() {
    // Create Core IR with a simple function
    let inner = CoreTerm::IntLit(42, None);
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(inner), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("test_fn".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify shim module is imported correctly
    // Should be "use axis_rust_bridge::runtime::shim;" not "shim::*"
    assert!(rust_code.contains("use axis_rust_bridge::runtime::shim;"), 
        "Expected 'use axis_rust_bridge::runtime::shim;' but got:\n{}", rust_code);
    assert!(!rust_code.contains("use axis_rust_bridge::runtime::shim::*;"),
        "Should use qualified import, not wildcard, but got:\n{}", rust_code);
}
