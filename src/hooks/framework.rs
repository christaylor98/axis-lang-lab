// Hook Framework - Core Types
//
// Defines the fundamental types for the compiler hook system.
//
// Hooks are structural interception points that operate on declared data shapes
// at named pipeline boundaries with explicit read/write intent.

use crate::frontend::schema_ast::SchemaAstNode;
use crate::frontend::token::Span;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// HOOK STAGES
// ═══════════════════════════════════════════════════════════════════════════

/// Pipeline stage where a hook executes
///
/// Each stage represents a well-defined boundary in the compilation pipeline.
/// Stages are ordered and deterministic.
///
/// Spec reference: AXIS_COMPILER_HOOKS_AND_PLUGINS_SPECIFICATION_0.1.md § 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HookStage {
    /// Pre-lexing (text-level)
    ///
    /// Executes before lexing, operates on raw source text.
    /// Use for: file inclusion, macro expansion, text adaptation
    PreLex,

    /// Post-parse (surface structure)
    ///
    /// Executes after parsing, before schema projection.
    /// Operates on parse tree / surface AST.
    /// Use for: structural rewrites, surface experimentation
    PostParse,

    /// Normalisation (during normalisation passes)
    ///
    /// Executes during normalisation, operates on Schema AST.
    /// Use for: desugaring, control-flow expansion, surface collapse
    /// Output MUST conform to Normal Form.
    Normalisation,

    /// Post-normalisation (read-only)
    ///
    /// Executes after NF validation, before lowering.
    /// Operates on NF Schema AST (read-only).
    /// Use for: analysis, tracing, diagnostics, reporting
    PostNormalisation,
}

impl fmt::Display for HookStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookStage::PreLex => write!(f, "pre-lex"),
            HookStage::PostParse => write!(f, "post-parse"),
            HookStage::Normalisation => write!(f, "normalisation"),
            HookStage::PostNormalisation => write!(f, "post-normalisation"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK PERMISSIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Permission level for hook execution
///
/// Determines what a hook is allowed to do with its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPermission {
    /// Read-only access
    ///
    /// Hook can inspect but not modify data.
    /// Used for analysis, diagnostics, tracing.
    ReadOnly,

    /// Rewrite access
    ///
    /// Hook can transform data structurally.
    /// Used for desugaring, normalization, adaptation.
    Rewrite,
}

impl fmt::Display for HookPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookPermission::ReadOnly => write!(f, "read-only"),
            HookPermission::Rewrite => write!(f, "rewrite"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK CONTEXT
// ═══════════════════════════════════════════════════════════════════════════

/// Execution context provided to hooks
///
/// Contains all information a hook needs to execute, including:
/// - Current pipeline stage
/// - Normalisation pass (if in normalisation stage)
/// - Permission level
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Current pipeline stage
    pub stage: HookStage,

    /// Current normalisation pass (e.g., "N1", "N2")
    /// Only set when stage == HookStage::Normalisation
    pub normalisation_pass: Option<String>,

    /// Permission level for this execution
    pub permission: HookPermission,
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK INPUT/OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

/// Input to a hook
///
/// Different stages operate on different data types.
#[derive(Debug, Clone)]
pub enum HookInput {
    /// Raw source text (PreLex stage)
    Text(String),

    /// Schema AST (PostParse, Normalisation, PostNormalisation stages)
    Ast(SchemaAstNode),
}

/// Output from a hook
///
/// Must match the input type.
#[derive(Debug, Clone)]
pub enum HookOutput {
    /// Transformed source text (PreLex stage)
    Text(String),

    /// Transformed Schema AST (PostParse, Normalisation stages)
    /// Or metadata from read-only hooks (PostNormalisation stage)
    Ast(SchemaAstNode),

    /// Metadata or diagnostics (PostNormalisation stage, read-only hooks)
    Metadata(serde_json::Value),
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK RESULT
// ═══════════════════════════════════════════════════════════════════════════

/// Result of hook execution
#[derive(Debug)]
pub struct HookResult {
    /// Output from the hook
    pub output: HookOutput,

    /// Optional metadata (for diagnostics, tracing)
    pub metadata: Option<serde_json::Value>,
}

/// Hook execution error
#[derive(Debug, Clone)]
pub struct HookError {
    pub message: String,
    pub hook_name: String,
    pub stage: HookStage,
    pub span: Option<Span>,
}

impl fmt::Display for HookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hook Error [{}] at stage {}: {}",
            self.hook_name, self.stage, self.message
        )
    }
}

impl std::error::Error for HookError {}

pub type HookExecutionResult = Result<HookResult, HookError>;

// ═══════════════════════════════════════════════════════════════════════════
// HOOK TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Core hook trait
///
/// All hooks implement this trait.
/// Hooks are deterministic: identical inputs produce identical outputs.
pub trait Hook: Send + Sync + std::fmt::Debug {
    /// Unique name for this hook
    fn name(&self) -> &str;

    /// Pipeline stage where this hook executes
    fn stage(&self) -> HookStage;

    /// Permission level required
    fn permission(&self) -> HookPermission;

    /// Normalisation pass (if stage == Normalisation)
    ///
    /// Examples: "N1", "N2", "N3"
    /// None means "applies to all passes"
    fn normalisation_pass(&self) -> Option<&str> {
        None
    }

    /// Execute the hook
    ///
    /// INVARIANTS:
    /// - Must be deterministic
    /// - Must respect permission level
    /// - Output shape must match input shape
    /// - For Normalisation hooks: output must be NF-admissible
    fn execute(&self, input: HookInput, context: &HookContext) -> HookExecutionResult;

    /// Optional description
    fn description(&self) -> Option<&str> {
        None
    }
}
