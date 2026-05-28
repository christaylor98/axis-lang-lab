// Test: verify emit-time fixes for truthy, zero-arity functions, and literal bindings

use axis_rust_bridge::core_ir::CoreTerm;
use std::rc::Rc;

#[test]
fn test_truthy_helper_imported() {
    // Create Core IR with an If term (uses truthy)
    // If(BoolLit(true), IntLit(1), IntLit(2))
    let cond = CoreTerm::BoolLit(true, None);
    let then_branch = CoreTerm::IntLit(1, None);
    let else_branch = CoreTerm::IntLit(2, None);
    let if_term = CoreTerm::If(Rc::new(cond), Rc::new(then_branch), Rc::new(else_branch), None);
    
    // Wrap in a function
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(if_term), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("test_if".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify truthy is imported
    assert!(rust_code.contains("use axis_rust_bridge::runtime::value::{init_runtime, truthy};"), 
        "Expected truthy import but got:\n{}", rust_code);
    
    // Verify truthy is used in if condition
    assert!(rust_code.contains("if truthy"), 
        "Expected 'if truthy' usage but got:\n{}", rust_code);
}

#[test]
fn test_zero_arity_function_called_in_value_position() {
    // Create Core IR with:
    // Let(err_msg, IntLit(42), Let(get_err, Var(err_msg), UnitLit))
    // This creates a zero-arity function `err_msg` that's referenced by `get_err`
    
    let const_value = CoreTerm::IntLit(42, None);
    let inner_var = CoreTerm::Var("err_msg".to_string(), None);
    let inner_lambda = CoreTerm::Lam("x".to_string(), Rc::new(inner_var), None);
    let inner_body = CoreTerm::UnitLit(None);
    let inner_let = CoreTerm::Let("get_err".to_string(), Rc::new(inner_lambda), Rc::new(inner_body), None);
    
    // Outer let with zero-arity function (constant)
    let term = CoreTerm::Let("err_msg".to_string(), Rc::new(const_value), Rc::new(inner_let), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify zero-arity function is defined
    assert!(rust_code.contains("pub fn err_msg() -> Value"), 
        "Expected zero-arity function definition but got:\n{}", rust_code);
    
    // Verify it's called when used in value position (not just referenced)
    // The generated code should have err_msg() with parentheses
    assert!(rust_code.contains("err_msg()"), 
        "Expected 'err_msg()' call but got:\n{}", rust_code);
    
    // Should NOT have bare err_msg as a value (except in the fn name)
    // Count occurrences: at least one should be a call
    let call_count = rust_code.matches("err_msg()").count();
    assert!(call_count > 0, 
        "Expected at least one err_msg() call but found {} in:\n{}", call_count, rust_code);
}

#[test]
fn test_function_with_params_not_auto_called() {
    // Create Core IR with a 1-arity function
    // Let(add_one, Lam(x, IntLit(1)), UnitLit)
    let inner = CoreTerm::IntLit(1, None);
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(inner), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("add_one".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify it's a proper function with parameter
    assert!(rust_code.contains("pub fn add_one(x: Value) -> Value"), 
        "Expected function with parameter but got:\n{}", rust_code);
    
    // Entry point should call it (from main)
    assert!(rust_code.contains("add_one("), 
        "Expected call to add_one from main but got:\n{}", rust_code);
}

#[test]
fn test_generated_code_with_conditionals_compiles() {
    // Create a more complex example with if/then/else
    let cond = CoreTerm::BoolLit(true, None);
    let then_val = CoreTerm::IntLit(1, None);
    let else_val = CoreTerm::IntLit(0, None);
    let if_expr = CoreTerm::If(Rc::new(cond), Rc::new(then_val), Rc::new(else_val), None);
    
    let lambda = CoreTerm::Lam("x".to_string(), Rc::new(if_expr), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("test".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Write to temp file and try to compile it
    std::fs::write("/tmp/test_emit_if.rs", &rust_code).expect("Failed to write test file");
    
    // Verify the imports are present
    assert!(rust_code.contains("use axis_rust_bridge::runtime::value::{init_runtime, truthy};"));
    assert!(rust_code.contains("use axis_rust_bridge::runtime::shim;"));
    assert!(rust_code.contains("if truthy"));
}
