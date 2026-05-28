// Wave A: End-to-End Pipeline Integration
//
// This module integrates all waves (1-6) into a single executable pipeline:
// spec files + source → lex → parse → generic AST → schema AST → Core IR → execution
//
// FORBIDDEN:
// - Redesigning existing waves
// - Adding semantic changes
// - Creating new abstractions
// - Bypassing existing wave implementations
// - Logic duplication
//
// This is PURE INTEGRATION.

use crate::execution::interpreter;
use crate::frontend::ast_builder;
use crate::frontend::ast_builder::ASTNode;
use crate::frontend::lexer_engine;
use crate::frontend::lexspec_load;
use crate::frontend::parser_runtime;
use crate::frontend::parser_runtime::ParseTree;
use crate::frontend::parserspec_load;
use crate::frontend::postfix_parser;
use crate::frontend::schema_ast;
use crate::frontend::schema_ast::SchemaAstNode;
use crate::frontend::schema_load;
use crate::frontend::token::{Span, Token};
use crate::ir::core_ir::CoreBundle;
use crate::nf_ast::NfAst;
use crate::normalize::{NormalizationContext, NormalizationError};
use crate::registry::Registry;

use std::fmt;
use std::fs;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Pipeline configuration - specifies all required spec files and source
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Path to lexer specification (lexer.yaml)
    pub lexer_spec: PathBuf,
    /// Path to parser specification (parsing.yaml)
    pub parser_spec: PathBuf,
    /// Path to AST schema (ast_schema.yaml)
    pub ast_schema: PathBuf,
    /// Path to source file to compile and execute
    pub source_file: PathBuf,
    /// Optional entry rule (if None, uses parser spec's start rule)
    pub entry_rule: Option<String>,
    /// Path to normalization rules spec (REQUIRED)
    pub normalize_spec: PathBuf,
    /// Registry for foreign function validation (REQUIRED)
    /// This is the authoritative registry from CLI --reg arguments
    pub registry: Registry,
    /// Parser mode: "grammar" (default) or "postfix" (for AI-1 RPN)
    pub parser_mode: Option<String>,
    /// Optional hook registry (WAVE 4)
    pub hook_registry: Option<crate::hooks::HookRegistry>,
}

/// Pipeline artifacts - all intermediate products from pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineArtifacts {
    /// Tokens from lexer
    pub tokens: Vec<Token>,
    /// Parse tree from parser
    pub parse_tree: ParseTree,
    /// Generic AST from structural projection
    pub generic_ast: ASTNode,
    /// Schema AST from schema projection
    pub schema_ast: SchemaAstNode,
    /// Core IR bundle
    pub core_ir: CoreBundle,
}

/// Pipeline error - wraps all possible failures with span information
#[derive(Debug)]
pub enum PipelineError {
    /// Failed to read a file
    Io { message: String, path: PathBuf },
    /// Failed to load lexer spec
    LexerSpecLoad { message: String, span: Option<Span> },
    /// Failed to load parser spec
    ParserSpecLoad { message: String, span: Option<Span> },
    /// Failed to load AST schema
    SchemaLoad { message: String, span: Option<Span> },
    /// Failed to load registry
    RegistryLoad { message: String },
    /// Lexing failed
    Lex { message: String, span: Span },
    /// Parsing failed
    Parse { message: String, span: Span },
    /// Generic AST building failed
    AstBuild { message: String, span: Option<Span> },
    /// Schema AST projection failed
    SchemaProject { message: String, span: Span },
    /// Normalization failed (YAML-driven engine)
    Normalization { message: String, span: Span },
    /// NF validation failed (WAVE 1)
    NfValidation { message: String, span: Span },
    /// Lowering to Core IR failed
    Lower { message: String, span: Span },
    /// Execution failed
    Eval { message: String, span: Span },
    /// Inspection failed (Wave T1)
    Inspection { message: String },
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Io { message, path } => {
                write!(f, "I/O error reading {}: {}", path.display(), message)
            }
            PipelineError::LexerSpecLoad { message, span } => {
                if let Some(sp) = span {
                    write!(
                        f,
                        "lexer spec load error at {}..{}: {}",
                        sp.start, sp.end, message
                    )
                } else {
                    write!(f, "lexer spec load error: {}", message)
                }
            }
            PipelineError::ParserSpecLoad { message, span } => {
                if let Some(sp) = span {
                    write!(
                        f,
                        "parser spec load error at {}..{}: {}",
                        sp.start, sp.end, message
                    )
                } else {
                    write!(f, "parser spec load error: {}", message)
                }
            }
            PipelineError::SchemaLoad { message, span } => {
                if let Some(sp) = span {
                    write!(
                        f,
                        "schema load error at {}..{}: {}",
                        sp.start, sp.end, message
                    )
                } else {
                    write!(f, "schema load error: {}", message)
                }
            }
            PipelineError::RegistryLoad { message } => {
                write!(f, "registry load error: {}", message)
            }
            PipelineError::Lex { message, span } => {
                write!(f, "lex error at {}..{}: {}", span.start, span.end, message)
            }
            PipelineError::Parse { message, span } => {
                write!(
                    f,
                    "parse error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::AstBuild { message, span } => {
                if let Some(sp) = span {
                    write!(
                        f,
                        "AST build error at {}..{}: {}",
                        sp.start, sp.end, message
                    )
                } else {
                    write!(f, "AST build error: {}", message)
                }
            }
            PipelineError::SchemaProject { message, span } => {
                write!(
                    f,
                    "schema projection error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::Normalization { message, span } => {
                write!(
                    f,
                    "normalization error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::NfValidation { message, span } => {
                write!(
                    f,
                    "NF validation error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::Lower { message, span } => {
                write!(
                    f,
                    "lowering error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::Eval { message, span } => {
                write!(
                    f,
                    "execution error at {}..{}: {}",
                    span.start, span.end, message
                )
            }
            PipelineError::Inspection { message } => {
                write!(f, "inspection error: {}", message)
            }
        }
    }
}

impl std::error::Error for PipelineError {}

/// Run the full pipeline from specs to Core IR
///
/// This executes all stages:
/// 1. Load all specs from disk
/// 2. Load source file
/// 3. Lex source using lexer spec
/// 4. Parse tokens using parser spec
/// 5. Build generic AST from parse tree
/// 6. Project schema AST using AST schema
/// 7. Validate NF compliance (WAVE 1)
/// 8. Lower schema AST to Core IR
///
/// Returns all intermediate artifacts.
/// All errors are fatal - no partial execution.
pub fn run_pipeline(cfg: &PipelineConfig) -> Result<PipelineArtifacts, PipelineError> {
    // STAGE 0: Load all specs
    let lexer_spec =
        lexspec_load::load_spec(&cfg.lexer_spec).map_err(|e| PipelineError::LexerSpecLoad {
            message: e.message,
            span: None,
        })?;

    // Only load parser spec if NOT in postfix mode
    let parser_spec = if cfg.parser_mode.as_deref() != Some("postfix") {
        parserspec_load::load_parser_spec(&cfg.parser_spec, &lexer_spec).map_err(|e| {
            PipelineError::ParserSpecLoad {
                message: e.message,
                span: None,
            }
        })?
    } else {
        // Dummy parser spec for postfix mode (not used)
        use crate::frontend::parserspec::ParserSpec;
        use std::collections::HashMap;
        ParserSpec {
            start: "Program".to_string(),
            grammar: HashMap::new(),
        }
    };

    let ast_schema = schema_load::load_schema_from_file(&cfg.ast_schema).map_err(|e| {
        PipelineError::SchemaLoad {
            message: e.message,
            span: None,
        }
    })?;

    // STAGE 1: Load source file
    let source = fs::read_to_string(&cfg.source_file).map_err(|e| PipelineError::Io {
        message: e.to_string(),
        path: cfg.source_file.clone(),
    })?;

    // STAGE 2: Lex source
    let tokens =
        lexer_engine::lex_with_spec(&lexer_spec, &source).map_err(|e| PipelineError::Lex {
            message: e.message,
            span: e.span.unwrap_or(Span::new(0, 0)),
        })?;

    // STAGE 3: Parse tokens
    // HARD GUARD: Verify parser_mode is honored (no silent fallback)
    let parse_tree = match cfg.parser_mode.as_deref() {
        Some("postfix") => {
            // AI-1 postfix parsing (no parser spec needed, uses stack reduction)
            postfix_parser::parse_postfix(&tokens).map_err(|e| PipelineError::Parse {
                message: e.message,
                span: e.span,
            })?
        }
        None | Some("grammar") => {
            // Grammar-based parsing (default)
            parser_runtime::parse_with_spec(&parser_spec, &tokens).map_err(|e| {
                PipelineError::Parse {
                    message: e.message,
                    span: e.span,
                }
            })?
        }
        Some(unknown) => {
            // HARD ERROR: Unknown parser mode
            return Err(PipelineError::Parse {
                message: format!(
                    "unknown parser_mode '{}': expected 'postfix' or 'grammar'",
                    unknown
                ),
                span: Span::new(0, 0),
            });
        }
    };

    // STAGE 4: Build generic AST
    let generic_ast =
        ast_builder::build_generic_ast(&parse_tree).map_err(|e| PipelineError::AstBuild {
            message: e.message,
            span: None,
        })?;

    // STAGE 5: Project schema AST
    let schema_ast = schema_ast::project_schema_ast(&generic_ast, &ast_schema).map_err(|e| {
        PipelineError::SchemaProject {
            message: e.message,
            span: e.span,
        }
    })?;

    // STAGE 5.5: YAML-driven Normalization (MANDATORY)
    // Transform Schema AST to Normal Form using YAML rules
    let normalization_ctx = NormalizationContext::load(&cfg.normalize_spec).map_err(|e| {
        PipelineError::Normalization {
            message: format!("{}", e),
            span: Span::new(0, 0),
        }
    })?;

    let nf_ast: NfAst = normalization_ctx
        .normalize(schema_ast)
        .map_err(|e| match e {
            NormalizationError::ValidationError(v) => PipelineError::NfValidation {
                message: v.message.clone(),
                span: v.span,
            },
            _ => PipelineError::Normalization {
                message: format!("{}", e),
                span: Span::new(0, 0),
            },
        })?;

    // STAGE 6: Mechanical lowering (NF-only, type-enforced)
    // Use registry from config (authoritative CLI-provided registry)
    let registry = cfg.registry.clone();

    // Mechanical lowering: NfAst → CoreBundle
    // This is 100% code-driven, no config file
    // Uses universal NF lowerer that matches NORMAL_FORM_SPEC_0.1.yaml exactly
    let core_ir = crate::lowering::nf_lowering::lower_nf_to_bundle(nf_ast.node(), registry)
        .map_err(|e| PipelineError::Lower {
            message: e.message,
            span: e.span,
        })?;

    Ok(PipelineArtifacts {
        tokens,
        parse_tree,
        generic_ast,
        schema_ast: nf_ast.node().clone(),
        core_ir,
    })
}

/// Run the full pipeline and execute the resulting Core IR
///
/// This is a convenience wrapper that:
/// 1. Runs the full pipeline to get Core IR
/// 2. Executes the Core IR using the execution substrate
/// 3. Returns the final runtime value
///
/// All errors are fatal - no partial execution.
pub fn run_and_eval(
    cfg: &PipelineConfig,
    registry: Registry,
) -> Result<interpreter::Value, PipelineError> {
    // Run pipeline to get Core IR
    let artifacts = run_pipeline(cfg)?;

    // Execute Core IR
    let value = interpreter::eval_bundle(&artifacts.core_ir, registry).map_err(|e| {
        PipelineError::Eval {
            message: e.message,
            span: e.span,
        }
    })?;

    Ok(value)
}
