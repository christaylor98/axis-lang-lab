// Minimal test: create a Core IR with a tail-recursive function
// and verify the TCO emission

use axis_rust_bridge::core_ir::{CoreTerm, create_core_bundle};
use std::rc::Rc;
use std::fs;

fn main() {
    // Create: Let(factorial, Lam(n, App(Var(factorial), Var(n))), UnitLit)
    let arg = CoreTerm::Var("n".to_string(), None);
    let func = CoreTerm::Var("factorial".to_string(), None);
    let app = CoreTerm::App(Rc::new(func), Rc::new(arg), None);
    let lambda = CoreTerm::Lam("n".to_string(), Rc::new(app), None);
    let body = CoreTerm::UnitLit(None);
    let term = CoreTerm::Let("factorial".to_string(), Rc::new(lambda), Rc::new(body), None);
    
    // Save as Core IR bundle
    let bundle = create_core_bundle(&term, "factorial");
    fs::write("/tmp/test_tco.coreir", bundle).expect("Failed to write Core IR");
    
    println!("Created /tmp/test_tco.coreir");
    println!("Run: axis-rust-bridge build /tmp/test_tco.coreir --out /tmp/test_tco");
}
