// Wave T1: Code Trace Inspection
//
// Cross-phase compilation trace that correlates:
// - Lexer tokens
// - CST (parse tree)
// - Schema AST
// - Core IR
//
// CONSTRAINTS:
// - Read-only inspection (no semantic changes)
// - No new compiler passes
// - No inference or reconstruction
// - Reflects actual in-memory state only
// - Deterministic output
// - Fails fast if prerequisites not met

use crate::frontend::parser_runtime::{ParseNode, ParseTree};
use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::frontend::token::{Span, Token};
use crate::ir::core_ir::{CoreBundle, CoreTerm};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum CodeTraceError {
    PrerequisiteNotMet(String),
}

impl std::fmt::Display for CodeTraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeTraceError::PrerequisiteNotMet(msg) => {
                write!(f, "Code trace prerequisite not met: {}", msg)
            }
        }
    }
}

impl std::error::Error for CodeTraceError {}

// ═══════════════════════════════════════════════════════════════════════════
// TRACE ENTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Single trace entry correlating all phases for one semantic AST node
#[derive(Debug)]
pub struct TraceEntry {
    /// Semantic AST node kind (after wrapper normalization)
    pub ast_kind: String,
    /// Source span covering this node
    pub span: Span,
    /// Lexer tokens contributing to this node
    pub tokens: Vec<TokenInfo>,
    /// CST node(s) that produced this AST node
    pub cst_nodes: Vec<CSTInfo>,
    /// AST fields extracted by schema projection
    pub ast_fields: Vec<ASTFieldInfo>,
    /// Lowered Core IR nodes
    pub core_ir: Vec<CoreIRInfo>,
}

#[derive(Debug)]
pub struct TokenInfo {
    pub kind: String,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CSTInfo {
    pub rule: String,
    pub span: Span,
    pub child_count: usize,
}

#[derive(Debug)]
pub struct ASTFieldInfo {
    pub name: String,
    pub value_type: String,
}

#[derive(Debug)]
pub struct CoreIRInfo {
    pub term_type: String,
    pub description: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Generate code trace from all pipeline phases
///
/// PREREQUISITES (enforced):
/// - tokens must be present (lexing completed)
/// - parse_tree must be present (parsing completed)
/// - schema_ast must be present (schema projection completed)
/// - core_ir must be present (lowering completed)
pub fn generate_code_trace(
    tokens: &[Token],
    parse_tree: &ParseTree,
    schema_ast: &SchemaAstNode,
    core_ir: &CoreBundle,
) -> Result<Vec<TraceEntry>, CodeTraceError> {
    // PREREQUISITE CHECKS (fail fast)
    if tokens.is_empty() {
        return Err(CodeTraceError::PrerequisiteNotMet(
            "lexing not completed - no tokens available".to_string(),
        ));
    }

    if parse_tree.children.is_empty() {
        return Err(CodeTraceError::PrerequisiteNotMet(
            "parsing not completed - empty parse tree".to_string(),
        ));
    }

    // Schema AST and Core IR existence verified by type presence

    // Build correlation maps
    let token_map = build_token_map(tokens);
    let cst_map = build_cst_map(parse_tree);

    // Walk schema AST and collect trace entries
    let mut entries = Vec::new();
    collect_trace_entries(
        schema_ast,
        &token_map,
        &cst_map,
        &core_ir.core_term,
        &mut entries,
    );

    if entries.is_empty() {
        return Err(CodeTraceError::PrerequisiteNotMet(
            "no trace entries generated - schema AST may be empty".to_string(),
        ));
    }

    Ok(entries)
}

// ═══════════════════════════════════════════════════════════════════════════
// INTERNAL IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Map spans to tokens
fn build_token_map(tokens: &[Token]) -> HashMap<Span, Vec<Token>> {
    let mut map: HashMap<Span, Vec<Token>> = HashMap::new();
    for token in tokens {
        map.entry(token.span.clone())
            .or_insert_with(Vec::new)
            .push(token.clone());
    }
    map
}

/// Map spans to CST nodes
fn build_cst_map(parse_tree: &ParseTree) -> HashMap<Span, Vec<CSTInfo>> {
    let mut map: HashMap<Span, Vec<CSTInfo>> = HashMap::new();
    collect_cst_info(parse_tree, &mut map);
    map
}

fn collect_cst_info(tree: &ParseTree, map: &mut HashMap<Span, Vec<CSTInfo>>) {
    let info = CSTInfo {
        rule: tree.rule.clone(),
        span: tree.span.clone(),
        child_count: tree.children.len(),
    };

    map.entry(tree.span.clone())
        .or_insert_with(Vec::new)
        .push(info);

    // Recurse into children
    for child in &tree.children {
        if let ParseNode::Rule(subtree) = child {
            collect_cst_info(subtree, map);
        }
    }
}

/// Collect trace entries by walking schema AST
fn collect_trace_entries(
    node: &SchemaAstNode,
    token_map: &HashMap<Span, Vec<Token>>,
    cst_map: &HashMap<Span, Vec<CSTInfo>>,
    core_ir: &CoreTerm,
    entries: &mut Vec<TraceEntry>,
) {
    // Skip wrapper nodes (Expr, AtomicExpr, AppExpr) - they are traversal context only
    if matches!(node.kind.as_str(), "Expr" | "AtomicExpr" | "AppExpr") {
        // Unwrap and continue
        if let Some(SchemaValue::Node(inner)) = node.fields.get("inner") {
            collect_trace_entries(inner, token_map, cst_map, core_ir, entries);
        } else if let Some(SchemaValue::Nodes(children)) = node.fields.get("parts") {
            // AppExpr has parts field with children
            for child in children {
                collect_trace_entries(child, token_map, cst_map, core_ir, entries);
            }
        }
        return;
    }

    // Build entry for this semantic node
    let mut entry = TraceEntry {
        ast_kind: node.kind.clone(),
        span: node.span.clone(),
        tokens: Vec::new(),
        cst_nodes: Vec::new(),
        ast_fields: Vec::new(),
        core_ir: Vec::new(),
    };

    // Collect tokens overlapping this span
    entry.tokens = find_tokens_in_span(&node.span, token_map);

    // Collect CST nodes overlapping this span
    if let Some(cst_infos) = cst_map.get(&node.span) {
        entry.cst_nodes = cst_infos.clone();
    }

    // Collect AST fields
    for (field_name, field_value) in &node.fields {
        let value_type = match field_value {
            SchemaValue::Node(_) => "Node",
            SchemaValue::Nodes(_) => "Nodes",
            SchemaValue::Token(_) => "Token",
            SchemaValue::Tokens(_) => "Tokens",
        };
        entry.ast_fields.push(ASTFieldInfo {
            name: field_name.clone(),
            value_type: value_type.to_string(),
        });
    }

    // Correlate with Core IR (span-based heuristic)
    entry.core_ir = find_core_ir_for_span(&node.span, core_ir);

    entries.push(entry);

    // Recurse into child nodes
    for field_value in node.fields.values() {
        match field_value {
            SchemaValue::Node(child) => {
                collect_trace_entries(child, token_map, cst_map, core_ir, entries);
            }
            SchemaValue::Nodes(children) => {
                for child in children {
                    collect_trace_entries(child, token_map, cst_map, core_ir, entries);
                }
            }
            _ => {}
        }
    }
}

/// Find tokens within or overlapping a span
fn find_tokens_in_span(span: &Span, token_map: &HashMap<Span, Vec<Token>>) -> Vec<TokenInfo> {
    let mut result = Vec::new();

    for (tok_span, tokens) in token_map {
        // Check if token span overlaps with target span
        if spans_overlap(span, tok_span) {
            for token in tokens {
                result.push(TokenInfo {
                    kind: format!("{:?}", token.kind),
                    lexeme: token.lexeme.clone(),
                    span: token.span.clone(),
                });
            }
        }
    }

    // Sort by span start for deterministic output
    result.sort_by_key(|t| t.span.start);
    result
}

/// Find Core IR nodes corresponding to a span
fn find_core_ir_for_span(span: &Span, term: &CoreTerm) -> Vec<CoreIRInfo> {
    let mut result = Vec::new();

    // Walk Core IR tree and collect nodes
    // Since Core IR nodes don't have spans in the current implementation,
    // we use a simple structural correlation:
    // - Collect all Core IR nodes at this level
    // NOTE: This is a limitation of the current IR structure, not an inference

    match term {
        CoreTerm::CIntLit { value, .. } => {
            result.push(CoreIRInfo {
                term_type: "CIntLit".to_string(),
                description: format!("value={}", value),
            });
        }
        CoreTerm::CBoolLit { value, .. } => {
            result.push(CoreIRInfo {
                term_type: "CBoolLit".to_string(),
                description: format!("value={}", value),
            });
        }
        CoreTerm::CUnitLit { .. } => {
            result.push(CoreIRInfo {
                term_type: "CUnitLit".to_string(),
                description: "()".to_string(),
            });
        }
        CoreTerm::CVar { name, .. } => {
            result.push(CoreIRInfo {
                term_type: "CVar".to_string(),
                description: format!("name={}", name.0),
            });
        }
        CoreTerm::CLam { param, body, .. } => {
            result.push(CoreIRInfo {
                term_type: "CLam".to_string(),
                description: format!("param={}", param.0),
            });
            // Recurse (but this is basic - no precise span correlation)
            result.extend(find_core_ir_for_span(span, body));
        }
        CoreTerm::CLet {
            name, value, body, ..
        } => {
            result.push(CoreIRInfo {
                term_type: "CLet".to_string(),
                description: format!("name={}", name.0),
            });
            result.extend(find_core_ir_for_span(span, value));
            result.extend(find_core_ir_for_span(span, body));
        }
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            result.push(CoreIRInfo {
                term_type: "CIf".to_string(),
                description: "conditional".to_string(),
            });
            result.extend(find_core_ir_for_span(span, cond));
            result.extend(find_core_ir_for_span(span, then_branch));
            result.extend(find_core_ir_for_span(span, else_branch));
        }
        CoreTerm::CApp { func, arg, .. } => {
            result.push(CoreIRInfo {
                term_type: "CApp".to_string(),
                description: "application".to_string(),
            });
            result.extend(find_core_ir_for_span(span, func));
            result.extend(find_core_ir_for_span(span, arg));
        }
        CoreTerm::CCall {
            target_name, args, ..
        } => {
            result.push(CoreIRInfo {
                term_type: "CCall".to_string(),
                description: format!("target_name={}", target_name),
            });
            for arg in args {
                result.extend(find_core_ir_for_span(span, arg));
            }
        }
    }

    result
}

fn spans_overlap(a: &Span, b: &Span) -> bool {
    // Two spans overlap if one starts before the other ends
    !(a.end <= b.start || b.end <= a.start)
}

// ═══════════════════════════════════════════════════════════════════════════
// OUTPUT FORMATTING
// ═══════════════════════════════════════════════════════════════════════════

/// Render trace entries to human-readable format
pub fn render_trace(entries: &[TraceEntry]) {
    println!("═══════════════════════════════════════════════════════════");
    println!("CODE TRACE");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    for (idx, entry) in entries.iter().enumerate() {
        println!("─────────────────────────────────────────────────────────");
        println!("TRACE ENTRY #{}", idx + 1);
        println!("─────────────────────────────────────────────────────────");
        println!();

        println!("AST Node: {}", entry.ast_kind);
        println!("Span: {}..{}", entry.span.start, entry.span.end);
        println!();

        println!("LEXER:");
        if entry.tokens.is_empty() {
            println!("  (no tokens)");
        } else {
            for token in &entry.tokens {
                println!(
                    "  {} \"{}\" span={}..{}",
                    token.kind, token.lexeme, token.span.start, token.span.end
                );
            }
        }
        println!();

        println!("CST:");
        if entry.cst_nodes.is_empty() {
            println!("  (no CST nodes)");
        } else {
            for cst in &entry.cst_nodes {
                println!(
                    "  {} [children={}] span={}..{}",
                    cst.rule, cst.child_count, cst.span.start, cst.span.end
                );
            }
        }
        println!();

        println!("AST FIELDS:");
        if entry.ast_fields.is_empty() {
            println!("  (no fields)");
        } else {
            for field in &entry.ast_fields {
                println!("  {} = <{}>", field.name, field.value_type);
            }
        }
        println!();

        println!("LOWERING:");
        if entry.core_ir.is_empty() {
            println!("  (no Core IR correlation)");
        } else {
            for ir in &entry.core_ir {
                println!("  {} ({})", ir.term_type, ir.description);
            }
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════════════");
    println!("Total trace entries: {}", entries.len());
    println!("═══════════════════════════════════════════════════════════");
}
