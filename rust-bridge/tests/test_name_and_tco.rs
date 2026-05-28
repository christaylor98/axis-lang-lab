// Test: verify that function names from Core IR are emitted correctly
// and that TCO rewrites are correct (not broken self-assigns)

use axis_rust_bridge::core_ir::{CoreTerm, create_core_bundle};
use std::rc::Rc;

#[test]
fn test_function_name_emission() {
    // Create a simple Core IR: Let(factorial, Lam(param, Var(param)), UnitLit)
    let inner = CoreTerm::Var("param".to_string(), None);
    let lambda = CoreTerm::Lam("param".to_string(), Rc::new(inner), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("factorial".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // Verify the function name is "factorial", not "name"
    assert!(rust_code.contains("pub fn factorial("), 
        "Expected 'pub fn factorial(' but got:\n{}", rust_code);
    assert!(!rust_code.contains("pub fn name("),
        "Should not contain placeholder 'pub fn name(' but got:\n{}", rust_code);
}

#[test]
fn test_tco_rewrite_correctness() {
    // Create a tail-recursive function:
    // Let(loop_fn, Lam(n, App(Var(loop_fn), Var(n))), UnitLit)
    let arg = CoreTerm::Var("n".to_string(), None);
    let func = CoreTerm::Var("loop_fn".to_string(), None);
    let app = CoreTerm::App(Rc::new(func), Rc::new(arg), None);
    let lambda = CoreTerm::Lam("n".to_string(), Rc::new(app), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("loop_fn".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Emit Rust code
    let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(&term, "test.coreir", "");
    
    // TCO should emit parameter reassignment, not function reference
    // WRONG: n = loop_fn.clone();
    // RIGHT: n = n.clone(); continue
    // Actually for self-tail-call with same arg, should be: continue (no reassignment needed)
    // Or: n = <evaluated_arg>; continue
    
    assert!(!rust_code.contains("n = loop_fn.clone()"),
        "TCO should not emit 'n = loop_fn.clone()' (function reference) but got:\n{}", rust_code);
    assert!(!rust_code.contains("n = loop_fn;"),
        "TCO should not emit 'n = loop_fn;' (function reference) but got:\n{}", rust_code);
}
