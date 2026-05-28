// NF AST Module — Normal Form AST Type
//
// PURPOSE:
// This module defines the NF AST type marker that enforces the boundary
// between Normalization and Mechanical Lowering. NF AST can ONLY be
// constructed by the normalization stage, ensuring lowering receives
// exclusively NF-compliant AST.
//
// INVARIANTS:
// - NF AST cannot be constructed outside normalization
// - Lowering API accepts ONLY NF AST type
// - NF AST carries NF validation guarantee
//
// SCOPE:
// - Type-level enforcement of NF boundary
// - NF AST wrapper/marker
// - Validation guarantee tracking

use crate::frontend::schema_ast::SchemaAstNode;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// NF AST TYPE (BOUNDARY ENFORCEMENT)
// ═══════════════════════════════════════════════════════════════════════════

/// Normal Form AST — guaranteed NF-compliant by construction
///
/// This type wraps a SchemaAstNode with the guarantee that:
/// 1. All node kinds are in the NF admissible set
/// 2. All structural invariants hold
/// 3. No forbidden patterns exist
/// 4. NF validation has passed
///
/// CONSTRUCTION:
/// NfAst can ONLY be created by:
/// - Normalization stage (after rewrite + validation)
/// - Test utilities (with explicit unsafe escape hatch)
///
/// CONSUMPTION:
/// Mechanical Lowering accepts ONLY NfAst, enforcing the boundary.
#[derive(Debug, Clone)]
pub struct NfAst {
    /// The underlying AST node (NF-compliant)
    node: SchemaAstNode,

    /// NF spec version this AST conforms to
    nf_version: NfVersion,
}

impl NfAst {
    /// Construct NfAst from validated node (normalization stage only)
    ///
    /// SAFETY CONTRACT:
    /// Caller MUST ensure node is NF-compliant via validate_nf() before calling.
    /// This function is NOT public to enforce construction via normalization.
    pub(crate) fn from_validated_node(node: SchemaAstNode, nf_version: NfVersion) -> Self {
        NfAst { node, nf_version }
    }

    /// Get reference to underlying AST node
    ///
    /// Safe because NfAst construction guarantees NF compliance.
    pub fn node(&self) -> &SchemaAstNode {
        &self.node
    }

    /// Get NF spec version
    pub fn nf_version(&self) -> &NfVersion {
        &self.nf_version
    }

    /// Consume NfAst and extract underlying node
    ///
    /// Used by lowering to access the validated AST.
    pub fn into_node(self) -> SchemaAstNode {
        self.node
    }

    /// UNSAFE: Create NfAst without validation (test utilities only)
    ///
    /// # Safety
    /// This bypasses NF validation and MUST ONLY be used in tests
    /// where controlled non-NF AST is needed for negative testing.
    ///
    /// Production code MUST NOT call this.
    #[doc(hidden)]
    pub unsafe fn from_unchecked_node_for_testing(node: SchemaAstNode) -> Self {
        NfAst {
            node,
            nf_version: NfVersion::V0_1,
        }
    }
}

impl fmt::Display for NfAst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NfAst({})", self.nf_version)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NF VERSION
// ═══════════════════════════════════════════════════════════════════════════

/// NF specification version
///
/// Tracks which NF spec version this AST conforms to.
/// Lowering can verify version compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfVersion {
    /// NF Spec 0.1 (initial version)
    V0_1,
}

impl NfVersion {
    /// Get version string
    pub fn as_str(&self) -> &'static str {
        match self {
            NfVersion::V0_1 => "0.1",
        }
    }

    /// Check if this NF version is compatible with target Core IR version
    pub fn is_compatible_with_core_ir(&self, core_ir_version: &str) -> bool {
        match self {
            // NF 0.1 targets Core IR 0.3 (canonical)
            NfVersion::V0_1 => core_ir_version == "0.3",
        }
    }
}

impl fmt::Display for NfVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NF {}", self.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::schema_ast::SchemaAstNode;
    use crate::frontend::token::Span;
    use std::collections::HashMap;

    fn make_test_node(kind: &str) -> SchemaAstNode {
        SchemaAstNode {
            kind: kind.to_string(),
            fields: HashMap::new(),
            annotations: vec![],
            span: Span { start: 0, end: 1 },
        }
    }

    #[test]
    fn test_nf_version_display() {
        assert_eq!(NfVersion::V0_1.as_str(), "0.1");
        assert_eq!(format!("{}", NfVersion::V0_1), "NF 0.1");
    }

    #[test]
    fn test_nf_version_core_ir_compatibility() {
        assert!(NfVersion::V0_1.is_compatible_with_core_ir("0.3"));
        assert!(!NfVersion::V0_1.is_compatible_with_core_ir("0.1"));
        assert!(!NfVersion::V0_1.is_compatible_with_core_ir("0.2"));
        assert!(!NfVersion::V0_1.is_compatible_with_core_ir("1.0"));
    }

    #[test]
    fn test_nf_ast_construction_unsafe() {
        let dummy_node = make_test_node("IntLit");

        // UNSAFE: Only for testing
        let nf_ast = unsafe { NfAst::from_unchecked_node_for_testing(dummy_node.clone()) };

        assert_eq!(nf_ast.nf_version(), &NfVersion::V0_1);
        assert_eq!(nf_ast.node().kind, "IntLit");
    }

    #[test]
    fn test_nf_ast_into_node() {
        let dummy_node = make_test_node("IntLit");

        let nf_ast = unsafe { NfAst::from_unchecked_node_for_testing(dummy_node.clone()) };
        let extracted = nf_ast.into_node();

        assert_eq!(extracted.kind, "IntLit");
    }
}
