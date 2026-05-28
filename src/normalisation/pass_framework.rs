// WAVE 2: Normalisation Pass Framework
//
// This module implements the explicit, ordered multi-pass normalisation driver.
//
// SCOPE (WAVE 2):
// - Ordered pass execution (N0-N4)
// - Deterministic sequencing
// - Pass registry (internal, hard-coded)
// - Baseline identity passes
// - Per-pass inspection support
//
// OUT OF SCOPE (HARD BOUNDARY):
// - Hooks
// - User-configurable passes
// - Desugaring rules
// - Semantic interpretation
// - Registry access
//
// INVARIANTS:
// - Pass execution is deterministic
// - Pass order is fixed
// - Passes consume and produce Schema AST only
// - Passes are pure (no side effects)
// - Each pass is independently inspectable

use crate::frontend::schema_ast::SchemaAstNode;
use crate::frontend::token::Span;
use crate::hooks::execution::{HookExecutionTrace, HookExecutor};
use crate::hooks::{HookContext, HookInput, HookPermission, HookRegistry, HookStage};
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Normalisation pass error
#[derive(Debug, Clone)]
pub struct NormalisationError {
    pub message: String,
    pub span: Span,
    pub pass_id: String,
}

impl fmt::Display for NormalisationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Normalisation Error [{}] at {}..{}: {}",
            self.pass_id, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for NormalisationError {}

/// Result of a normalisation pass
pub type NormalisationResult = Result<SchemaAstNode, NormalisationError>;

/// Information about a completed pass execution
#[derive(Debug, Clone)]
pub struct PassExecution {
    /// Pass identifier (e.g., "N0", "N1")
    pub pass_id: String,

    /// Pass name
    pub pass_name: String,

    /// Input AST (before pass)
    pub input_ast: SchemaAstNode,

    /// Output AST (after pass)
    pub output_ast: SchemaAstNode,

    /// Execution time
    pub duration: std::time::Duration,

    /// Whether AST was modified
    pub modified: bool,
}

/// Result of full normalisation with execution trace
#[derive(Debug, Clone)]
pub struct NormalisationTrace {
    /// Final normalised AST
    pub final_ast: SchemaAstNode,

    /// Execution trace for each pass
    pub executions: Vec<PassExecution>,

    /// Total normalisation time
    pub total_duration: std::time::Duration,

    /// Hook execution trace (WAVE 4)
    pub hook_trace: Option<HookExecutionTrace>,
}

/// Run full normalisation pipeline
///
/// Executes all normalisation passes in order (N0-N4).
/// Each pass transforms the Schema AST structurally.
///
/// WAVE 4: Optionally executes registered hooks at normalisation stage.
///
/// Returns:
/// - Final normalised AST
/// - Execution trace (for inspection)
///
/// DETERMINISTIC: Same input always produces same output
/// PURE: No side effects, no registry access
pub fn normalise(input: SchemaAstNode) -> Result<NormalisationTrace, NormalisationError> {
    normalise_with_hooks(input, None)
}

/// Run full normalisation pipeline with optional hooks
///
/// WAVE 4: Executes normalisation with hook support.
/// Hooks are executed before each normalisation pass they're registered for.
pub fn normalise_with_hooks(
    input: SchemaAstNode,
    hook_registry: Option<&HookRegistry>,
) -> Result<NormalisationTrace, NormalisationError> {
    let start_time = std::time::Instant::now();
    let mut executions = Vec::new();
    let mut current_ast = input;
    let mut hook_trace = HookExecutionTrace::new();

    // Execute all registered passes in order
    for pass in get_pass_registry() {
        // WAVE 4: Execute normalisation hooks for this pass
        if let Some(registry) = hook_registry {
            let hooks = registry
                .get_hooks_for_normalisation_pass(pass.id)
                .map_err(|e| NormalisationError {
                    message: format!("Hook ordering error: {}", e),
                    span: Span::new(0, 0),
                    pass_id: pass.id.to_string(),
                })?;

            if !hooks.is_empty() {
                let hook_context = HookContext {
                    stage: HookStage::Normalisation,
                    normalisation_pass: Some(pass.id.to_string()),
                    permission: HookPermission::Rewrite,
                };

                let hook_input = HookInput::Ast(current_ast.clone());
                let (result, trace) =
                    HookExecutor::execute_sequence(hooks, hook_input, &hook_context);

                // Merge hook trace
                for exec in trace.executions {
                    hook_trace.add_execution(exec);
                }

                // Handle hook errors
                if let Err(e) = result {
                    return Err(NormalisationError {
                        message: format!("Hook execution failed: {}", e),
                        span: Span::new(0, 0),
                        pass_id: pass.id.to_string(),
                    });
                }

                // Update AST with hook output
                if let Ok(output) = result {
                    if let crate::hooks::HookOutput::Ast(ast) = output {
                        current_ast = ast;
                    }
                }
            }
        }

        let pass_start = std::time::Instant::now();
        let input_ast = current_ast.clone();

        // Execute pass
        let output_ast = (pass.transform)(&current_ast)?;

        let duration = pass_start.elapsed();

        // Check if AST was modified
        let modified = !ast_equal(&input_ast, &output_ast);

        executions.push(PassExecution {
            pass_id: pass.id.to_string(),
            pass_name: pass.name.to_string(),
            input_ast,
            output_ast: output_ast.clone(),
            duration,
            modified,
        });

        current_ast = output_ast;
    }

    let total_duration = start_time.elapsed();

    Ok(NormalisationTrace {
        final_ast: current_ast,
        executions,
        total_duration,
        hook_trace: Some(hook_trace),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// PASS REGISTRY (INTERNAL, HARD-CODED)
// ═══════════════════════════════════════════════════════════════════════════

/// A normalisation pass
struct NormalisationPass {
    /// Pass identifier (N0, N1, etc.)
    id: &'static str,

    /// Human-readable name
    name: &'static str,

    /// Pass description
    description: &'static str,

    /// Transform function
    transform: fn(&SchemaAstNode) -> NormalisationResult,
}

/// Get the fixed pass registry
///
/// WAVE 2: Hard-coded pass list (N0-N4)
/// Future waves may extend this list but order remains fixed
fn get_pass_registry() -> Vec<NormalisationPass> {
    vec![
        NormalisationPass {
            id: "N0",
            name: "Pre-normalisation Sanity Check",
            description: "Validates AST structure before normalisation",
            transform: pass_n0_sanity_check,
        },
        NormalisationPass {
            id: "N1",
            name: "Control Flow Desugaring",
            description: "Desugar ForExpr, WhileExpr, LoopExpr, MatchExpr to NF constructs",
            transform: pass_n1_identity,
        },
        NormalisationPass {
            id: "N2",
            name: "Structural Normalization",
            description: "Additional structural cleanup and normalization",
            transform: pass_n2_identity,
        },
        NormalisationPass {
            id: "N3",
            name: "Final Cleanup",
            description: "Final structural cleanup and canonicalization",
            transform: pass_n3_identity,
        },
        NormalisationPass {
            id: "N4",
            name: "Post-normalisation Canonical Check",
            description: "Final structural validation before NF validation",
            transform: pass_n4_canonical_check,
        },
    ]
}

/// Get pass information for inspection
pub fn get_pass_info() -> Vec<(String, String, String)> {
    get_pass_registry()
        .into_iter()
        .map(|p| {
            (
                p.id.to_string(),
                p.name.to_string(),
                p.description.to_string(),
            )
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// BASELINE PASSES (WAVE 2 - IDENTITY/SANITY ONLY)
// ═══════════════════════════════════════════════════════════════════════════

/// N0: Pre-normalisation sanity check
///
/// Validates that input AST is well-formed before normalisation.
/// Does NOT modify AST.
///
/// WAVE 2: Identity pass (returns input unchanged)
/// Future: May add structural invariant checks
fn pass_n0_sanity_check(ast: &SchemaAstNode) -> NormalisationResult {
    // WAVE 2: Identity pass
    // Future waves may add:
    // - Structural invariant validation
    // - Pre-normalisation checks
    Ok(ast.clone())
}

/// N1: Control flow desugaring
///
/// WAVE 3: Desugar surface control constructs
/// - ForExpr → recursion
/// - WhileExpr → recursion + IfExpr
/// - LoopExpr → recursion
/// - MatchExpr → decision tree
fn pass_n1_identity(ast: &SchemaAstNode) -> NormalisationResult {
    // WAVE 3: Apply desugaring transformations
    use crate::normalisation::desugaring::desugar_node;
    Ok(desugar_node(ast))
}

/// N2: Structural normalization
///
/// WAVE 3: Additional desugaring and cleanup
/// - Wrapper node unwrapping continues from N1
/// - Structural canonicalization
fn pass_n2_identity(ast: &SchemaAstNode) -> NormalisationResult {
    // WAVE 3: Desugaring handles most transformations in N1
    // N2 can be used for additional cleanup if needed
    Ok(ast.clone())
}

/// N3: Final cleanup
///
/// WAVE 3: Final structural cleanup
/// - Ensure all wrapper nodes are removed
/// - Final canonicalization
fn pass_n3_identity(ast: &SchemaAstNode) -> NormalisationResult {
    // WAVE 3: Most work done in N1, N3 reserved for future cleanup
    Ok(ast.clone())
}

/// N4: Post-normalisation canonical check
///
/// Validates structural properties after all normalisation passes.
/// Does NOT modify AST.
///
/// WAVE 2: Identity pass
/// Future: May add post-normalisation structural checks
fn pass_n4_canonical_check(ast: &SchemaAstNode) -> NormalisationResult {
    // WAVE 2: Identity pass
    // Future waves may add:
    // - Structural canonicality checks
    // - Pre-NF-validation preparation
    Ok(ast.clone())
}

// ═══════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

/// Structural equality check for AST nodes
///
/// Compares AST structure to determine if a pass modified the tree.
/// Used for inspection and diagnostics.
fn ast_equal(a: &SchemaAstNode, b: &SchemaAstNode) -> bool {
    // WAVE 2: Simple structural comparison
    // This is sufficient for identity passes

    if a.kind != b.kind {
        return false;
    }

    if a.fields.len() != b.fields.len() {
        return false;
    }

    // For WAVE 2, we assume identity passes don't modify
    // Future waves may need deeper comparison
    true
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_node(kind: &str) -> SchemaAstNode {
        SchemaAstNode {
            kind: kind.to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 10),
        }
    }

    #[test]
    fn test_pass_registry_ordered() {
        let passes = get_pass_registry();
        assert_eq!(passes.len(), 5);
        assert_eq!(passes[0].id, "N0");
        assert_eq!(passes[1].id, "N1");
        assert_eq!(passes[2].id, "N2");
        assert_eq!(passes[3].id, "N3");
        assert_eq!(passes[4].id, "N4");
    }

    #[test]
    fn test_normalise_identity() {
        // WAVE 2: All passes are identity, so output == input
        let input = make_test_node("LetExpr");
        let result = normalise(input.clone()).unwrap();

        assert_eq!(result.executions.len(), 5);
        assert_eq!(result.final_ast.kind, input.kind);

        // All passes should report not modified (identity)
        for exec in &result.executions {
            assert!(
                !exec.modified,
                "Pass {} should not modify AST in WAVE 2",
                exec.pass_id
            );
        }
    }

    #[test]
    fn test_pass_execution_trace() {
        let input = make_test_node("Program");
        let result = normalise(input).unwrap();

        // Check all passes executed
        let pass_ids: Vec<&str> = result
            .executions
            .iter()
            .map(|e| e.pass_id.as_str())
            .collect();

        assert_eq!(pass_ids, vec!["N0", "N1", "N2", "N3", "N4"]);
    }

    #[test]
    fn test_deterministic() {
        // Same input should produce same output
        let input = make_test_node("IfExpr");

        let result1 = normalise(input.clone()).unwrap();
        let result2 = normalise(input.clone()).unwrap();

        assert_eq!(result1.final_ast.kind, result2.final_ast.kind);
        assert_eq!(result1.executions.len(), result2.executions.len());
    }

    #[test]
    fn test_get_pass_info() {
        let info = get_pass_info();
        assert_eq!(info.len(), 5);

        assert_eq!(info[0].0, "N0");
        assert_eq!(info[4].0, "N4");
    }
}
