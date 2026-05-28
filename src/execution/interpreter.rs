// Core IR Execution Substrate (Interpreter)
//
// This module implements the execution layer that:
// - Consumes Core IR (from schema lowering)
// - Produces runtime values through interpretation
// - Treats Core IR as fully authoritative and final
// - Performs NO semantic interpretation beyond Core IR
// - Uses registry for all external behavior
//
// FORBIDDEN:
// - Parsing or inspecting tokens
// - Inspecting Schema AST
// - Inventing semantics not encoded in IR
// - Modifying Core IR
// - Compensating for invalid IR
// - Adding optimizations
// - Performing partial evaluation
// - Hiding execution errors
// - Introducing side channels
//
// If Core IR is invalid or unsupported: FAIL FAST

use crate::frontend::token::Span;
use crate::ir::core_ir::{CoreBundle, CoreTerm, IdentOrName};
use crate::registry::Registry;
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

/// Runtime value produced by evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Unit value
    Unit,
    /// Closure: captured environment + parameter + body
    Closure {
        param: IdentOrName,
        body: Box<CoreTerm>,
        env: Environment,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Closure { .. } => write!(f, "<closure>"),
        }
    }
}

/// Evaluation environment (variable bindings)
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// Stack of bindings (inner to outer)
    frames: Vec<HashMap<String, Value>>,
}

impl Environment {
    /// Create a new empty environment
    #[allow(dead_code)]
    pub fn new() -> Self {
        Environment {
            frames: vec![HashMap::new()],
        }
    }

    /// Push a new environment frame
    #[allow(dead_code)]
    fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Pop the current environment frame
    #[allow(dead_code)]
    fn pop_frame(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Bind a variable in the current frame
    #[allow(dead_code)]
    fn bind(&mut self, name: String, value: Value) {
        if let Some(current) = self.frames.last_mut() {
            current.insert(name, value);
        }
    }

    /// Look up a variable (searches from inner to outer)
    #[allow(dead_code)]
    fn lookup(&self, name: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some(value) = frame.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}

/// Evaluation context
pub struct EvalContext {
    /// Registry for external function calls
    pub registry: Registry,
    /// Current environment (variable bindings)
    env: Environment,
}

impl EvalContext {
    /// Create a new evaluation context with a registry
    pub fn new(registry: Registry) -> Self {
        EvalContext {
            registry,
            env: Environment::new(),
        }
    }
}

/// Evaluation error - all execution failures
#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Execution error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for EvalError {}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluate a Core IR term to a runtime value
///
/// This is the primary execution entry point.
///
/// Evaluation is:
/// - IR-driven: dispatched solely by CoreTerm variant
/// - Registry-bound: all external behavior via registry
/// - Deterministic: same IR + same inputs = same output
/// - Explicit: all errors are fatal and include span information
///
/// Returns:
/// - Ok(Value) on successful evaluation
/// - Err(EvalError) on any execution failure
pub fn eval_core_ir(ir: &CoreTerm, ctx: &mut EvalContext) -> Result<Value, EvalError> {
    match ir {
        CoreTerm::CIntLit { .. } => {
            // Integer literals not yet supported in interpreter
            Err(EvalError {
                message: "CIntLit evaluation not implemented (out of Language Lab scope)"
                    .to_string(),
                span: Span::new(0, 0),
            })
        }

        CoreTerm::CBoolLit { .. } => {
            // Boolean literals not yet supported in interpreter
            Err(EvalError {
                message: "CBoolLit evaluation not implemented (out of Language Lab scope)"
                    .to_string(),
                span: Span::new(0, 0),
            })
        }

        CoreTerm::CUnitLit { .. } => {
            // Unit literal evaluates to unit value
            Ok(Value::Unit)
        }

        CoreTerm::CLam { param, body, .. } => {
            // Lambda evaluates to closure capturing current environment
            Ok(Value::Closure {
                param: param.clone(),
                body: body.clone(),
                env: ctx.env.clone(),
            })
        }

        CoreTerm::CLet { .. } => {
            // Let bindings not yet supported in interpreter
            Err(EvalError {
                message: "CLet evaluation not implemented (out of Language Lab scope)".to_string(),
                span: Span::new(0, 0),
            })
        }

        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch: _,
            ..
        } => {
            // Evaluate condition
            let cond_val = eval_core_ir(cond, ctx)?;

            // Condition must be unit (since we only have unit values in this subset)
            // In a fuller IR, we'd check for boolean
            // For now, unit is truthy
            match cond_val {
                Value::Unit => {
                    // Unit is truthy - evaluate then branch
                    eval_core_ir(then_branch, ctx)
                }
                Value::Closure { .. } => {
                    // Closure is truthy - evaluate then branch
                    eval_core_ir(then_branch, ctx)
                }
            }
        }

        CoreTerm::CVar { .. } => {
            // Variable lookup not yet supported in interpreter
            Err(EvalError {
                message: "CVar evaluation not implemented (out of Language Lab scope)".to_string(),
                span: Span::new(0, 0),
            })
        }

        CoreTerm::CApp { .. } => {
            // Function application not yet supported in interpreter
            Err(EvalError {
                message: "CApp evaluation not implemented (out of Language Lab scope)".to_string(),
                span: Span::new(0, 0),
            })
        }

        CoreTerm::CCall {
            target_name,
            args,
            node_id,
            ..
        } => {
            // Core IR 0.3: Look up registry entry by canonical name
            let entry = ctx
                .registry
                .entries
                .iter()
                .find(|e| e.name == *target_name)
                .ok_or_else(|| EvalError {
                    message: format!("registry entry '{}' not found", target_name),
                    span: Span::new(0, 0), // Core IR doesn't preserve spans yet
                })?;

            // Check arity
            if args.len() != entry.arity {
                return Err(EvalError {
                    message: format!(
                        "arity mismatch for '{}': expected {}, got {}",
                        entry.name,
                        entry.arity,
                        args.len()
                    ),
                    span: Span::new(0, 0),
                });
            }

            // Clone entry data to avoid borrow issues
            let entry_id = entry.id;
            let entry_name = entry.name.clone();

            // Evaluate all arguments
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(eval_core_ir(arg, ctx)?);
            }

            // Execute registry call
            // For now, this is a stub - real implementation would dispatch to native code
            execute_registry_call(entry_id, &entry_name, &arg_values, node_id)
        }
    }
}

/// Evaluate a Core IR bundle
///
/// Convenience wrapper that evaluates the bundle's core term.
pub fn eval_bundle(bundle: &CoreBundle, registry: Registry) -> Result<Value, EvalError> {
    let mut ctx = EvalContext::new(registry);
    eval_core_ir(&bundle.core_term, &mut ctx)
}

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY EXECUTION
// ═══════════════════════════════════════════════════════════════════════════

/// Execute a registry call
///
/// This is the ONLY interface for external behavior.
/// All foreign operations must go through the registry.
///
/// For Wave 6, this is a minimal stub that:
/// - Validates the call is well-formed
/// - Returns unit (placeholder for real implementation)
///
/// Future waves may:
/// - Dispatch to native Rust functions
/// - Call FFI bindings
/// - Invoke WASM modules
/// - Execute bytecode
///
/// FORBIDDEN:
/// - Inventing behavior not specified in registry
/// - Side effects not declared by registry entry
/// - Implicit conversions or coercions
fn execute_registry_call(
    id: u64,
    name: &str,
    args: &[Value],
    _node_id: &Option<u64>,
) -> Result<Value, EvalError> {
    // For Wave 6, we implement a minimal set of registry functions
    match name {
        "print" => {
            // Print function: display value and return unit
            if args.len() != 1 {
                return Err(EvalError {
                    message: format!("print expects 1 argument, got {}", args.len()),
                    span: Span::new(0, 0),
                });
            }
            println!("[print] {}", args[0]);
            Ok(Value::Unit)
        }

        "add" => {
            // Add function: returns unit (placeholder - we don't have integers yet)
            if args.len() != 2 {
                return Err(EvalError {
                    message: format!("add expects 2 arguments, got {}", args.len()),
                    span: Span::new(0, 0),
                });
            }
            println!("[add] {} + {}", args[0], args[1]);
            Ok(Value::Unit)
        }

        _ => {
            // Unknown registry function
            Err(EvalError {
                message: format!("registry function '{}' (id={}) not implemented", name, id),
                span: Span::new(0, 0),
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::core_ir;
    use crate::registry::RegistryEntry;

    fn test_registry() -> Registry {
        Registry {
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
        }
    }

    #[test]
    fn test_eval_unit_lit() {
        // Test: () evaluates to Unit
        let ir = core_ir::unit_lit();
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_lambda() {
        // Test: \x -> () evaluates to Closure
        let ir = core_ir::lam(IdentOrName::new("x"), core_ir::unit_lit());
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        match result {
            Value::Closure { .. } => { /* success */ }
            _ => panic!("Expected closure, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_if_unit() {
        // Test: if () then () else () evaluates to ()
        let ir = core_ir::cif(
            core_ir::unit_lit(),
            core_ir::unit_lit(),
            core_ir::unit_lit(),
        );
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_registry_call_print() {
        // Test: print(()) succeeds
        let ir = core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]);
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_registry_call_add() {
        // Test: add((), ()) succeeds
        let ir = core_ir::ccall(
            "add".to_string(),
            vec![core_ir::unit_lit(), core_ir::unit_lit()],
        );
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_registry_call_missing() {
        // Test: Missing registry entry fails
        let ir = core_ir::ccall("nonexistent".to_string(), vec![core_ir::unit_lit()]);
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("registry entry 'nonexistent' not found"));
    }

    #[test]
    fn test_eval_registry_call_arity_mismatch() {
        // Test: Arity mismatch fails
        // print expects 1 arg, we give 2
        let ir = core_ir::ccall(
            "print".to_string(),
            vec![core_ir::unit_lit(), core_ir::unit_lit()],
        );
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("arity mismatch"));
    }

    #[test]
    fn test_eval_determinism() {
        // Test: Same IR evaluated twice produces same result
        let ir = core_ir::cif(
            core_ir::unit_lit(),
            core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]),
            core_ir::unit_lit(),
        );

        let mut ctx1 = EvalContext::new(test_registry());
        let result1 = eval_core_ir(&ir, &mut ctx1).unwrap();

        let mut ctx2 = EvalContext::new(test_registry());
        let result2 = eval_core_ir(&ir, &mut ctx2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_eval_bundle() {
        // Test: Evaluating a complete bundle works
        let term = core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]);
        let bundle = core_ir::bundle_v0_2(term);

        let result = eval_bundle(&bundle, test_registry()).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_nested_if() {
        // Test: Nested if expressions
        let ir = core_ir::cif(
            core_ir::unit_lit(),
            core_ir::cif(
                core_ir::unit_lit(),
                core_ir::unit_lit(),
                core_ir::unit_lit(),
            ),
            core_ir::unit_lit(),
        );
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_eval_complex_expression() {
        // Test: Complex nested expression
        // if () then print(()) else add((), ())
        let ir = core_ir::cif(
            core_ir::unit_lit(),
            core_ir::ccall("print".to_string(), vec![core_ir::unit_lit()]),
            core_ir::ccall(
                "add".to_string(),
                vec![core_ir::unit_lit(), core_ir::unit_lit()],
            ),
        );
        let mut ctx = EvalContext::new(test_registry());

        let result = eval_core_ir(&ir, &mut ctx).unwrap();
        assert_eq!(result, Value::Unit);
    }
}
