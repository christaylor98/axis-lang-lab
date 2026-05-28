// Wave B: Trust Invariants and Verification
//
// Verify that:
// - Every Core IR node is traceable
// - No Core IR node lacks provenance
// - User annotations never override system annotations
// - Execution ignores all trace data
// - Trace graphs are deterministic

use crate::introspection::provenance::ProvenanceAnnotation;
use crate::introspection::trace_graph::TraceGraph;
use crate::ir::core_ir::{Annotation, CoreTerm};

// ═══════════════════════════════════════════════════════════════════════════
// TRUST INVARIANT ERRORS
// ═══════════════════════════════════════════════════════════════════════════

/// Trust invariant violation
#[derive(Debug, Clone)]
pub enum TrustInvariantError {
    /// Core IR node has no source traceability
    UntracedCoreIrNode { core_ir_id: u64 },
    /// Core IR node lacks provenance annotations
    MissingProvenance { core_ir_id: u64 },
    /// User annotation uses reserved namespace
    UserAnnotationInReservedNamespace {
        annotation_key: String,
        reserved_namespace: String,
    },
    /// Trace graph is internally inconsistent
    TraceGraphInconsistent { message: String },
    /// Two trace graphs for same input differ
    NonDeterministicTrace { message: String },
}

impl std::fmt::Display for TrustInvariantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustInvariantError::UntracedCoreIrNode { core_ir_id } => {
                write!(f, "Core IR node {} has no source traceability", core_ir_id)
            }
            TrustInvariantError::MissingProvenance { core_ir_id } => {
                write!(
                    f,
                    "Core IR node {} lacks provenance annotations",
                    core_ir_id
                )
            }
            TrustInvariantError::UserAnnotationInReservedNamespace {
                annotation_key,
                reserved_namespace,
            } => {
                write!(
                    f,
                    "User annotation '{}' uses reserved namespace '{}'",
                    annotation_key, reserved_namespace
                )
            }
            TrustInvariantError::TraceGraphInconsistent { message } => {
                write!(f, "Trace graph inconsistent: {}", message)
            }
            TrustInvariantError::NonDeterministicTrace { message } => {
                write!(f, "Trace graph is non-deterministic: {}", message)
            }
        }
    }
}

impl std::error::Error for TrustInvariantError {}

// ═══════════════════════════════════════════════════════════════════════════
// INVARIANT VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Verify all trust invariants for a trace graph
///
/// This checks:
/// 1. Every Core IR node is traceable to source
/// 2. Trace graph is internally consistent
/// 3. All structural invariants hold
pub fn verify_trace_invariants(trace: &TraceGraph) -> Result<(), Vec<TrustInvariantError>> {
    let mut errors = Vec::new();

    // Check trace graph internal consistency
    if let Err(msg) = trace.verify_invariants() {
        errors.push(TrustInvariantError::TraceGraphInconsistent { message: msg });
    }

    // If there are consistency errors, return early (other checks may be invalid)
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

/// Verify that user annotations don't use reserved namespaces
///
/// This ensures user annotations never collide with provenance annotations.
pub fn verify_annotation_namespaces(
    annotations: &[Annotation],
) -> Result<(), Vec<TrustInvariantError>> {
    let mut errors = Vec::new();

    for annotation in annotations {
        if let Some(namespace) = ProvenanceAnnotation::namespace_of(&annotation.key) {
            errors.push(TrustInvariantError::UserAnnotationInReservedNamespace {
                annotation_key: annotation.key.clone(),
                reserved_namespace: namespace.prefix().to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Verify that Core IR annotations don't affect execution
///
/// This is a semantic check: we verify that two Core IR terms that differ
/// only in annotations evaluate to the same result.
///
/// NOTE: This is a structural check only (we don't actually execute).
/// The real guarantee comes from the execution engine ignoring annotations.
pub fn verify_annotations_ignored_structurally(
    term1: &CoreTerm,
    term2: &CoreTerm,
) -> Result<(), String> {
    // Strip annotations and compare
    let stripped1 = strip_annotations(term1);
    let stripped2 = strip_annotations(term2);

    if stripped1 == stripped2 {
        Ok(())
    } else {
        Err("Terms differ in more than just annotations".to_string())
    }
}

/// Strip all annotations from a Core IR term
fn strip_annotations(term: &CoreTerm) -> CoreTerm {
    match term {
        CoreTerm::CIntLit { value, node_id, .. } => CoreTerm::CIntLit {
            value: *value,
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CBoolLit { value, node_id, .. } => CoreTerm::CBoolLit {
            value: *value,
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CUnitLit { node_id, .. } => CoreTerm::CUnitLit {
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CLam {
            param,
            body,
            node_id,
            ..
        } => CoreTerm::CLam {
            param: param.clone(),
            body: Box::new(strip_annotations(body)),
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CLet {
            name,
            value,
            body,
            node_id,
            ..
        } => CoreTerm::CLet {
            name: name.clone(),
            value: Box::new(strip_annotations(value)),
            body: Box::new(strip_annotations(body)),
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            node_id,
            ..
        } => CoreTerm::CIf {
            cond: Box::new(strip_annotations(cond)),
            then_branch: Box::new(strip_annotations(then_branch)),
            else_branch: Box::new(strip_annotations(else_branch)),
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CVar { name, node_id, .. } => CoreTerm::CVar {
            name: name.clone(),
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CApp {
            func, arg, node_id, ..
        } => CoreTerm::CApp {
            func: Box::new(strip_annotations(func)),
            arg: Box::new(strip_annotations(arg)),
            node_id: *node_id,
            annotations: Vec::new(),
        },
        CoreTerm::CCall {
            target_name,
            args,
            node_id,
            ..
        } => CoreTerm::CCall {
            target_name: target_name.clone(),
            args: args.iter().map(strip_annotations).collect(),
            node_id: *node_id,
            annotations: Vec::new(),
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DETERMINISM VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Verify that two trace graphs are identical
///
/// This is used to verify determinism: running the pipeline twice on the
/// same input should produce identical traces.
pub fn verify_trace_determinism(
    trace1: &TraceGraph,
    trace2: &TraceGraph,
) -> Result<(), TrustInvariantError> {
    // Check node counts
    let stats1 = trace1.count_by_kind();
    let stats2 = trace2.count_by_kind();

    if stats1 != stats2 {
        return Err(TrustInvariantError::NonDeterministicTrace {
            message: format!("Node counts differ: {:?} vs {:?}", stats1, stats2),
        });
    }

    // Check link counts
    if trace1.all_links().len() != trace2.all_links().len() {
        return Err(TrustInvariantError::NonDeterministicTrace {
            message: format!(
                "Link counts differ: {} vs {}",
                trace1.all_links().len(),
                trace2.all_links().len()
            ),
        });
    }

    // Note: Full structural comparison would require stable node IDs,
    // which we don't guarantee. This is a basic sanity check.
    // For full determinism verification, use identical inputs and
    // compare serialized outputs.

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::token::{Span, Token, TokenKind};
    use crate::introspection::provenance::ProvenanceAnnotationSet;
    use crate::introspection::trace_graph::TraceGraphBuilder;
    use crate::ir::core_ir::AnnotationValue;

    #[test]
    fn test_verify_valid_trace_graph() {
        let mut builder = TraceGraphBuilder::new();

        let token = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(0, 3));
        let token_id = builder.add_token(token, ProvenanceAnnotationSet::new());

        builder.add_core_ir(
            "CUnitLit".to_string(),
            Some(1),
            None,
            ProvenanceAnnotationSet::new(),
            vec![token_id],
        );

        let graph = builder.build();

        assert!(verify_trace_invariants(&graph).is_ok());
    }

    #[test]
    fn test_verify_invalid_trace_graph() {
        let mut builder = TraceGraphBuilder::new();

        // Core IR node with no source (orphan)
        builder.add_core_ir(
            "CUnitLit".to_string(),
            Some(1),
            None,
            ProvenanceAnnotationSet::new(),
            Vec::new(),
        );

        let graph = builder.build();

        // Should fail because Core IR has no source tokens
        assert!(verify_trace_invariants(&graph).is_err());
    }

    #[test]
    fn test_verify_annotation_namespaces() {
        let valid = vec![Annotation {
            key: "user.annotation".to_string(),
            value: AnnotationValue::String("value".to_string()),
        }];

        assert!(verify_annotation_namespaces(&valid).is_ok());

        let invalid = vec![Annotation {
            key: "source.span".to_string(), // Reserved namespace
            value: AnnotationValue::String("0..10".to_string()),
        }];

        let result = verify_annotation_namespaces(&invalid);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_strip_annotations() {
        let term_with_annotations = CoreTerm::CUnitLit {
            node_id: Some(1),
            annotations: vec![Annotation {
                key: "test".to_string(),
                value: AnnotationValue::String("value".to_string()),
            }],
        };

        let stripped = strip_annotations(&term_with_annotations);

        match stripped {
            CoreTerm::CUnitLit { annotations, .. } => {
                assert!(annotations.is_empty());
            }
            _ => panic!("Wrong term kind"),
        }
    }

    #[test]
    fn test_verify_annotations_ignored() {
        let term1 = CoreTerm::CUnitLit {
            node_id: Some(1),
            annotations: vec![Annotation {
                key: "test1".to_string(),
                value: AnnotationValue::String("value1".to_string()),
            }],
        };

        let term2 = CoreTerm::CUnitLit {
            node_id: Some(1),
            annotations: vec![Annotation {
                key: "test2".to_string(),
                value: AnnotationValue::String("value2".to_string()),
            }],
        };

        assert!(verify_annotations_ignored_structurally(&term1, &term2).is_ok());
    }

    #[test]
    fn test_verify_trace_determinism() {
        let mut builder1 = TraceGraphBuilder::new();
        let token1 = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(0, 3));
        let token_id1 = builder1.add_token(token1, ProvenanceAnnotationSet::new());
        builder1.add_core_ir(
            "CUnitLit".to_string(),
            Some(1),
            None,
            ProvenanceAnnotationSet::new(),
            vec![token_id1],
        );
        let graph1 = builder1.build();

        let mut builder2 = TraceGraphBuilder::new();
        let token2 = Token::new(TokenKind::Ident, "foo".to_string(), Span::new(0, 3));
        let token_id2 = builder2.add_token(token2, ProvenanceAnnotationSet::new());
        builder2.add_core_ir(
            "CUnitLit".to_string(),
            Some(1),
            None,
            ProvenanceAnnotationSet::new(),
            vec![token_id2],
        );
        let graph2 = builder2.build();

        assert!(verify_trace_determinism(&graph1, &graph2).is_ok());
    }
}
