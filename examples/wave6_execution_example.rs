// Core IR Execution Example: Interpreter
//
// This example demonstrates the execution substrate (interpreter) for Core IR.
// It shows how to:
// - Build Core IR directly
// - Create an evaluation context
// - Execute Core IR
// - Handle registry calls
// - Test determinism

use axis_lang_lab::execution::interpreter::{eval_bundle, eval_core_ir, EvalContext};
use axis_lang_lab::ir::core_ir::{self, IdentOrName};
use axis_lang_lab::registry::{Registry, RegistryEntry};

fn main() {
    println!("Core IR Execution Example\n");
    println!("=================================\n");

    // ═════════════════════════════════════════════════════════════════
    // Step 1: Set up Registry
    // ═════════════════════════════════════════════════════════════════

    let registry = Registry {
        entries: vec![
            RegistryEntry {
                name: "print".to_string(),
                arity: 1,
                deterministic: false,
                profiles: vec![],
                id: 1,
            },
            RegistryEntry {
                name: "add".to_string(),
                arity: 2,
                deterministic: true,
                profiles: vec![],
                id: 2,
            },
        ],
    };

    println!(
        "✓ Registry loaded with {} functions",
        registry.entries.len()
    );
    println!("  - print (id=1, arity=1, deterministic=false)");
    println!("  - add (id=2, arity=2, deterministic=true)\n");

    // ═════════════════════════════════════════════════════════════════
    // Example 1: Evaluate Unit Literal
    // ═════════════════════════════════════════════════════════════════

    println!("Example 1: Unit Literal");
    println!("------------------------");
    println!("Core IR: ()");

    let ir1 = core_ir::unit_lit();
    let mut ctx1 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir1, &mut ctx1) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 2: Evaluate Lambda
    // ═════════════════════════════════════════════════════════════════

    println!("Example 2: Lambda Expression");
    println!("----------------------------");
    println!("Core IR: \\x -> ()");

    let ir2 = core_ir::lam(IdentOrName::new("x"), core_ir::unit_lit());
    let mut ctx2 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir2, &mut ctx2) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 3: Conditional Expression
    // ═════════════════════════════════════════════════════════════════

    println!("Example 3: Conditional Expression");
    println!("----------------------------------");
    println!("Core IR: if () then () else ()");

    let ir3 = core_ir::cif(
        core_ir::unit_lit(),
        core_ir::unit_lit(),
        core_ir::unit_lit(),
    );
    let mut ctx3 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir3, &mut ctx3) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 4: Registry Call - print
    // ═════════════════════════════════════════════════════════════════

    println!("Example 4: Registry Call - print");
    println!("---------------------------------");
    println!("Core IR: print(())");

    let ir4 = core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]);
    let mut ctx4 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir4, &mut ctx4) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 5: Registry Call - add
    // ═════════════════════════════════════════════════════════════════

    println!("Example 5: Registry Call - add");
    println!("-------------------------------");
    println!("Core IR: add((), ())");

    let ir5 = core_ir::ccall(
        "add".to_string(),
        vec![core_ir::unit_lit(), core_ir::unit_lit()],
    );
    let mut ctx5 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir5, &mut ctx5) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 6: Complex Nested Expression
    // ═════════════════════════════════════════════════════════════════

    println!("Example 6: Complex Nested Expression");
    println!("-------------------------------------");
    println!("Core IR: if () then print(()) else add((), ())");

    let ir6 = core_ir::cif(
        core_ir::unit_lit(),
        core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]),
        core_ir::ccall(
            "add".to_string(),
            vec![core_ir::unit_lit(), core_ir::unit_lit()],
        ),
    );
    let mut ctx6 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir6, &mut ctx6) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 7: Error - Missing Registry Entry
    // ═════════════════════════════════════════════════════════════════

    println!("Example 7: Error - Missing Registry Entry");
    println!("------------------------------------------");
    println!("Core IR: CCall(target=999, args=[])");

    let ir7 = core_ir::ccall("nonexistent".to_string(), vec![core_ir::unit_lit()]);
    let mut ctx7 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir7, &mut ctx7) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 8: Error - Arity Mismatch
    // ═════════════════════════════════════════════════════════════════

    println!("Example 8: Error - Arity Mismatch");
    println!("----------------------------------");
    println!("Core IR: print((), ()) -- expects 1 arg, got 2");

    let ir8 = core_ir::ccall(
        "print".to_string(),
        vec![core_ir::unit_lit(), core_ir::unit_lit()],
    );
    let mut ctx8 = EvalContext::new(registry.clone());

    match eval_core_ir(&ir8, &mut ctx8) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 9: Bundle Evaluation
    // ═════════════════════════════════════════════════════════════════

    println!("Example 9: Bundle Evaluation");
    println!("-----------------------------");
    println!("Core IR: Bundle containing print(())");

    let term9 = core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]);
    let bundle9 = core_ir::bundle_v0_2(term9);

    match eval_bundle(&bundle9, registry.clone()) {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    // ═════════════════════════════════════════════════════════════════
    // Example 10: Determinism Demonstration
    // ═════════════════════════════════════════════════════════════════

    println!("Example 10: Determinism Test");
    println!("-----------------------------");
    println!("Core IR: if () then print(()) else ()");
    println!("Evaluating twice with same IR and registry...");

    let ir10 = core_ir::cif(
        core_ir::unit_lit(),
        core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]),
        core_ir::unit_lit(),
    );

    let mut ctx10a = EvalContext::new(registry.clone());
    let result10a = eval_core_ir(&ir10, &mut ctx10a).unwrap();

    let mut ctx10b = EvalContext::new(registry.clone());
    let result10b = eval_core_ir(&ir10, &mut ctx10b).unwrap();

    println!("First result: {}", result10a);
    println!("Second result: {}", result10b);
    println!("Results match: {}\n", result10a == result10b);

    // ═════════════════════════════════════════════════════════════════
    // Summary
    // ═════════════════════════════════════════════════════════════════

    println!("=================================");
    println!("Wave 6 Execution Substrate Summary");
    println!("=================================");
    println!();
    println!("✓ Core IR execution implemented");
    println!("✓ Registry-driven external calls");
    println!("✓ Deterministic evaluation");
    println!("✓ Explicit error handling");
    println!("✓ IR-driven dispatch (no parsing)");
    println!();
    println!("Supported Core IR nodes:");
    println!("  - CUnitLit: Unit literal");
    println!("  - CLam: Lambda abstraction");
    println!("  - CIf: Conditional expression");
    println!("  - CCall: Registry function call");
    println!();
    println!("Registry functions:");
    println!("  - print: Display value (arity=1)");
    println!("  - add: Placeholder addition (arity=2)");
    println!();
}
