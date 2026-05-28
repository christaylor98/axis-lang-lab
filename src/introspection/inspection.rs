// Pipeline Stage Inspection
//
// This module provides observability and introspection for all pipeline stages.
// Each stage can be inspected independently to dump its exact runtime state.
//
// NO GUESSING MODE: Every loaded spec and intermediate artifact is explicitly
// dumped in deterministic, machine-readable format (YAML).

use crate::frontend::ast_builder;
use crate::frontend::lexer_engine;
use crate::frontend::lexspec_load;
use crate::frontend::parser_runtime;
use crate::frontend::parserspec_load;
use crate::frontend::schema_ast;
use crate::frontend::schema_load;
use crate::frontend::token::Span;
use crate::lowering::spec_driven;
use crate::pipeline::{PipelineConfig, PipelineError};
use crate::registry::Registry;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;

/// Run inspection mode - dump requested pipeline stages and exit
pub fn run_inspection(cfg: &PipelineConfig, stages: HashSet<String>) -> Result<(), PipelineError> {
    let start_time = std::time::Instant::now();
    let mut stage_timings: Vec<(String, std::time::Duration)> = Vec::new();

    // Track which stages executed
    let mut executed_stages: Vec<String> = Vec::new();

    // STAGE 0: Load lexer spec
    let stage_start = std::time::Instant::now();
    let lexer_spec =
        lexspec_load::load_spec(&cfg.lexer_spec).map_err(|e| PipelineError::LexerSpecLoad {
            message: e.message,
            span: None,
        })?;
    let lexer_load_time = stage_start.elapsed();
    stage_timings.push(("lexer_load".to_string(), lexer_load_time));
    executed_stages.push("lexer_load".to_string());

    if stages.contains("lexer") {
        dump_lexer_inspection(&lexer_spec);
    }

    // STAGE 1: Load parser spec
    let stage_start = std::time::Instant::now();
    let parser_spec =
        parserspec_load::load_parser_spec(&cfg.parser_spec, &lexer_spec).map_err(|e| {
            PipelineError::ParserSpecLoad {
                message: e.message,
                span: None,
            }
        })?;
    let parser_load_time = stage_start.elapsed();
    stage_timings.push(("parser_load".to_string(), parser_load_time));
    executed_stages.push("parser_load".to_string());

    if stages.contains("parser") || stages.contains("grammar") {
        dump_grammar_inspection(&parser_spec);
    }

    // STAGE 2: Load schema
    let stage_start = std::time::Instant::now();
    let ast_schema = schema_load::load_schema_from_file(&cfg.ast_schema).map_err(|e| {
        PipelineError::SchemaLoad {
            message: e.message,
            span: None,
        }
    })?;
    let schema_load_time = stage_start.elapsed();
    stage_timings.push(("schema_load".to_string(), schema_load_time));
    executed_stages.push("schema_load".to_string());

    if stages.contains("schema") {
        dump_schema_inspection(&ast_schema);
    }

    // STAGE 3: Use registry from config (already loaded by CLI)
    let stage_start = std::time::Instant::now();
    let registry = cfg.registry.clone();
    let registry_load_time = stage_start.elapsed();
    stage_timings.push(("registry_load".to_string(), registry_load_time));
    executed_stages.push("registry_load".to_string());

    if stages.contains("registry") {
        dump_registry_inspection(&registry);
    }

    // STAGE 4: Load source file
    let stage_start = std::time::Instant::now();
    let source = fs::read_to_string(&cfg.source_file).map_err(|e| PipelineError::Io {
        message: e.to_string(),
        path: cfg.source_file.clone(),
    })?;
    let source_load_time = stage_start.elapsed();
    stage_timings.push(("source_load".to_string(), source_load_time));
    executed_stages.push("source_load".to_string());

    // STAGE 5: Lex source
    let stage_start = std::time::Instant::now();
    let tokens =
        lexer_engine::lex_with_spec(&lexer_spec, &source).map_err(|e| PipelineError::Lex {
            message: e.message,
            span: e.span.unwrap_or(Span::new(0, 0)),
        })?;
    let lex_time = stage_start.elapsed();
    stage_timings.push(("lex".to_string(), lex_time));
    executed_stages.push("lex".to_string());

    // STAGE 6: Parse tokens
    let stage_start = std::time::Instant::now();
    let parse_tree =
        match cfg.parser_mode.as_deref() {
            Some("postfix") => {
                use crate::frontend::postfix_parser;
                postfix_parser::parse_postfix(&tokens).map_err(|e| PipelineError::Parse {
                    message: e.message,
                    span: e.span,
                })?
            }
            None | Some("grammar") => parser_runtime::parse_with_spec(&parser_spec, &tokens)
                .map_err(|e| PipelineError::Parse {
                    message: e.message,
                    span: e.span,
                })?,
            Some(unknown) => {
                return Err(PipelineError::Parse {
                    message: format!(
                        "unknown parser_mode '{}': expected 'postfix' or 'grammar'",
                        unknown
                    ),
                    span: Span::new(0, 0),
                });
            }
        };
    let parse_time = stage_start.elapsed();
    stage_timings.push(("parse".to_string(), parse_time));
    executed_stages.push("parse".to_string());

    if stages.contains("cst") {
        dump_cst_inspection(&parse_tree, &tokens);
    }

    // STAGE 7: Build generic AST
    let stage_start = std::time::Instant::now();
    let generic_ast =
        ast_builder::build_generic_ast(&parse_tree).map_err(|e| PipelineError::AstBuild {
            message: e.message,
            span: None,
        })?;
    let ast_build_time = stage_start.elapsed();
    stage_timings.push(("ast_build".to_string(), ast_build_time));
    executed_stages.push("ast_build".to_string());

    // STAGE 8: Project schema AST
    let stage_start = std::time::Instant::now();
    let schema_ast_node =
        schema_ast::project_schema_ast(&generic_ast, &ast_schema).map_err(|e| {
            PipelineError::SchemaProject {
                message: e.message,
                span: e.span,
            }
        })?;
    let schema_project_time = stage_start.elapsed();
    stage_timings.push(("schema_project".to_string(), schema_project_time));
    executed_stages.push("schema_project".to_string());

    if stages.contains("ast") {
        dump_ast_inspection(&schema_ast_node);
    }

    // STAGE 8: Normalization (YAML-driven)
    let stage_start = std::time::Instant::now();
    let normalization_ctx = crate::normalize::NormalizationContext::load(&cfg.normalize_spec)
        .map_err(|e| PipelineError::Normalization {
            message: format!("{}", e),
            span: crate::frontend::token::Span::new(0, 0),
        })?;

    let nf_ast = normalization_ctx
        .normalize(schema_ast_node)
        .map_err(|e| match e {
            crate::normalize::NormalizationError::ValidationError(v) => {
                PipelineError::NfValidation {
                    message: v.message.clone(),
                    span: v.span,
                }
            }
            _ => PipelineError::Normalization {
                message: format!("{}", e),
                span: crate::frontend::token::Span::new(0, 0),
            },
        })?;
    let normalisation_time = stage_start.elapsed();
    stage_timings.push(("normalisation".to_string(), normalisation_time));
    executed_stages.push("normalisation".to_string());

    if stages.contains("normalisation") || stages.contains("nf") {
        eprintln!("=== NORMALIZATION ===");
        eprintln!("NF version: {}", nf_ast.nf_version());
        eprintln!("Root node: {}", nf_ast.node().kind);
    }

    // STAGE 9: Lower to Core IR (mechanical lowering)
    let stage_start = std::time::Instant::now();
    let core_ir = crate::lowering::nf_lowering::lower_nf_to_bundle(nf_ast.node(), registry)
        .map_err(|e| PipelineError::Lower {
            message: e.message,
            span: e.span,
        })?;
    let lowering_time = stage_start.elapsed();
    stage_timings.push(("lowering".to_string(), lowering_time));
    executed_stages.push("lowering".to_string());

    if stages.contains("lowering") {
        eprintln!("=== LOWERING ===");
        eprintln!("Mechanical lowering: NfAst -> CoreIR");
    }

    if stages.contains("core-ir") {
        dump_core_ir_inspection(&core_ir);
    }

    // WAVE T1: Code trace inspection
    if stages.contains("code_trace") {
        dump_code_trace_inspection(&tokens, &parse_tree, nf_ast.node(), &core_ir)?;
    }

    // Pipeline summary
    if stages.contains("pipeline") {
        let total_time = start_time.elapsed();
        dump_pipeline_inspection(&executed_stages, &stage_timings, total_time, cfg);
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// INSPECTION DUMP FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

fn dump_lexer_inspection(spec: &crate::frontend::lexspec::LexerSpec) {
    println!("═══════════════════════════════════════════════════════════");
    println!("LEXER INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct LexerRule {
        index: usize,
        name: String,
        pattern: String,
        skip: bool,
    }

    #[derive(Serialize)]
    struct LexerInspection {
        charset: String,
        case_sensitive: bool,
        total_rules: usize,
        rules: Vec<LexerRule>,
    }

    let mut rules = Vec::new();
    let mut index = 0;

    // Whitespace
    if let Some(ws) = &spec.lexer.whitespace {
        rules.push(LexerRule {
            index,
            name: "WHITESPACE".to_string(),
            pattern: ws.pattern.clone(),
            skip: ws.skip,
        });
        index += 1;
    }

    // Comments
    for (i, comment) in spec.lexer.comments.iter().enumerate() {
        rules.push(LexerRule {
            index,
            name: format!("COMMENT_{}", i),
            pattern: comment.pattern.clone(),
            skip: comment.skip,
        });
        index += 1;
    }

    // Keywords
    for kw in &spec.lexer.keywords {
        rules.push(LexerRule {
            index,
            name: format!("KW_{}", kw.to_uppercase()),
            pattern: kw.clone(),
            skip: false,
        });
        index += 1;
    }

    // Literals
    if let Some(literals) = &spec.lexer.literals {
        if let Some(int_lit) = &literals.int {
            rules.push(LexerRule {
                index,
                name: "INT_LIT".to_string(),
                pattern: int_lit.pattern.clone(),
                skip: false,
            });
            index += 1;
        }
        if let Some(bool_lit) = &literals.bool {
            rules.push(LexerRule {
                index,
                name: "BOOL_LIT".to_string(),
                pattern: format!("{:?}", bool_lit.values),
                skip: false,
            });
            index += 1;
        }
        if let Some(unit_lit) = &literals.unit {
            rules.push(LexerRule {
                index,
                name: "UNIT_LIT".to_string(),
                pattern: unit_lit.literal.clone(),
                skip: false,
            });
            index += 1;
        }
        if let Some(string_lit) = &literals.string {
            rules.push(LexerRule {
                index,
                name: "STRING_LIT".to_string(),
                pattern: format!("delim: {}", string_lit.delimiter),
                skip: false,
            });
            index += 1;
        }
    }

    // Identifiers
    if let Some(ident) = &spec.lexer.identifiers {
        rules.push(LexerRule {
            index,
            name: "IDENT".to_string(),
            pattern: ident.pattern.clone(),
            skip: false,
        });
        index += 1;
    }

    // Punctuation
    for punct in &spec.lexer.punctuation {
        rules.push(LexerRule {
            index,
            name: format!("PUNCT_{}", punct),
            pattern: punct.clone(),
            skip: false,
        });
        index += 1;
    }

    let inspection = LexerInspection {
        charset: spec.lexer.charset.clone(),
        case_sensitive: spec.lexer.case_sensitive,
        total_rules: rules.len(),
        rules,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

fn dump_grammar_inspection(spec: &crate::frontend::parserspec::ParserSpec) {
    println!("═══════════════════════════════════════════════════════════");
    println!("GRAMMAR INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct GrammarInspection {
        start: String,
        nonterminals: std::collections::BTreeMap<String, Vec<String>>,
    }

    let mut nonterminals = std::collections::BTreeMap::new();

    for (nt, prods) in &spec.grammar {
        let mut prod_strings = Vec::new();
        for prod in prods {
            prod_strings.push(format_production(prod));
        }
        nonterminals.insert(nt.clone(), prod_strings);
    }

    let inspection = GrammarInspection {
        start: spec.start.clone(),
        nonterminals,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

fn format_production(prod: &crate::frontend::parserspec::Production) -> String {
    use crate::frontend::parserspec::Production;

    match prod {
        Production::Sequence(elements) => {
            let elem_strs: Vec<String> = elements.iter().map(|e| format_element(e)).collect();
            elem_strs.join(" ")
        }
        Production::Alternation(alts) => {
            let alt_strs: Vec<String> = alts.iter().map(|p| format_production(p)).collect();
            format!("({})", alt_strs.join(" | "))
        }
    }
}

fn format_element(elem: &crate::frontend::parserspec::Element) -> String {
    use crate::frontend::parserspec::{Element, RepeatKind, Terminal};

    match elem {
        Element::NonTerminal(nt) => nt.clone(),
        Element::Terminal(term) => match term {
            Terminal::TokenKind(tk, _) => format!("<{}>", tk),
            Terminal::Keyword(kw, _) => format!("kw:\"{}\"", kw),
            Terminal::Punct(p, _) => format!("punct:\"{}\"", p),
        },
        Element::Repeat(inner, kind) => {
            let inner_str = format_element(inner);
            match kind {
                RepeatKind::ZeroOrMore => format!("{}*", inner_str),
                RepeatKind::OneOrMore => format!("{}+", inner_str),
                RepeatKind::Optional => format!("{}?", inner_str),
            }
        }
        Element::Group(elems) => {
            let elem_strs: Vec<String> = elems.iter().map(|e| format_element(e)).collect();
            format!("({})", elem_strs.join(" "))
        }
    }
}

fn dump_cst_inspection(
    parse_tree: &crate::frontend::parser_runtime::ParseTree,
    tokens: &[crate::frontend::token::Token],
) {
    println!("═══════════════════════════════════════════════════════════");
    println!("CST INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct CSTNode {
        node_type: String,
        span: (usize, usize),
        token_value: Option<String>,
        children: Vec<CSTNode>,
    }

    fn convert_parse_tree(
        pt: &crate::frontend::parser_runtime::ParseTree,
        tokens: &[crate::frontend::token::Token],
    ) -> CSTNode {
        use crate::frontend::parser_runtime::ParseNode;

        let child_nodes: Vec<CSTNode> = pt
            .children
            .iter()
            .map(|child| match child {
                ParseNode::Rule(sub_tree) => convert_parse_tree(sub_tree, tokens),
                ParseNode::Terminal(token) => CSTNode {
                    node_type: token.kind.to_string(),
                    span: (token.span.start, token.span.end),
                    token_value: Some(token.lexeme.clone()),
                    children: Vec::new(),
                },
            })
            .collect();

        CSTNode {
            node_type: pt.rule.clone(),
            span: (pt.span.start, pt.span.end),
            token_value: None,
            children: child_nodes,
        }
    }

    let cst = convert_parse_tree(parse_tree, tokens);
    let yaml = serde_yaml::to_string(&cst).unwrap();
    println!("{}", yaml);
}

fn dump_schema_inspection(schema: &crate::frontend::schema_ast::AstSchema) {
    println!("═══════════════════════════════════════════════════════════");
    println!("SCHEMA INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct SchemaNodeInfo {
        kind: String,
        match_kind: String,
        fields: Vec<String>,
        is_enum: bool,
        variants: Vec<String>,
    }

    #[derive(Serialize)]
    struct SchemaInspection {
        total_nodes: usize,
        nodes: std::collections::BTreeMap<String, SchemaNodeInfo>,
    }

    let mut nodes = std::collections::BTreeMap::new();

    for (name, node_def) in &schema.nodes {
        let field_names: Vec<String> = node_def.fields.keys().cloned().collect();

        nodes.insert(
            name.clone(),
            SchemaNodeInfo {
                kind: name.clone(),
                match_kind: node_def.match_kind.clone(),
                fields: field_names,
                is_enum: false,
                variants: Vec::new(),
            },
        );
    }

    let inspection = SchemaInspection {
        total_nodes: nodes.len(),
        nodes,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

fn dump_ast_inspection(ast: &crate::frontend::schema_ast::SchemaAstNode) {
    println!("═══════════════════════════════════════════════════════════");
    println!("AST INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct ASTNodeInfo {
        kind: String,
        span: (usize, usize),
        variant: Option<String>,
        fields: std::collections::BTreeMap<String, String>,
        children_count: usize,
    }

    fn convert_ast_node(node: &crate::frontend::schema_ast::SchemaAstNode) -> ASTNodeInfo {
        use crate::frontend::schema_ast::SchemaValue;

        let mut fields = std::collections::BTreeMap::new();

        for (name, field_value) in &node.fields {
            let field_type = match field_value {
                SchemaValue::Node(_) => "Node".to_string(),
                SchemaValue::Nodes(_) => "Nodes".to_string(),
                SchemaValue::Token(_) => "Token".to_string(),
                SchemaValue::Tokens(_) => "Tokens".to_string(),
            };
            fields.insert(name.clone(), field_type);
        }

        ASTNodeInfo {
            kind: node.kind.clone(),
            span: (node.span.start, node.span.end),
            variant: None,
            fields,
            children_count: 0,
        }
    }

    let ast_info = convert_ast_node(ast);
    let yaml = serde_yaml::to_string(&ast_info).unwrap();
    println!("{}", yaml);
}

fn dump_registry_inspection(registry: &Registry) {
    println!("═══════════════════════════════════════════════════════════");
    println!("REGISTRY INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct RegistryOp {
        name: String,
        arity: usize,
        deterministic: bool,
        profiles: Vec<String>,
        id: u64,
    }

    #[derive(Serialize)]
    struct RegistryInspection {
        total_ops: usize,
        ops: Vec<RegistryOp>,
    }

    let ops: Vec<RegistryOp> = registry
        .entries
        .iter()
        .map(|e| RegistryOp {
            name: e.name.clone(),
            arity: e.arity,
            deterministic: e.deterministic,
            profiles: e.profiles.clone(),
            id: e.id,
        })
        .collect();

    let inspection = RegistryInspection {
        total_ops: ops.len(),
        ops,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

// Wave S3: Lowering spec inspection
#[allow(dead_code)]
fn dump_lowering_spec_inspection(spec: &spec_driven::LoweringSpec) {
    println!("═══════════════════════════════════════════════════════════");
    println!("LOWERING SPEC INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct LoweringSpecInfo {
        version: String,
        target_ir: String,
        source_path: Option<String>,
        content_hash: Option<String>,
        total_rules: usize,
        rule_names: Vec<String>,
    }

    let info = LoweringSpecInfo {
        version: spec.version.clone(),
        target_ir: spec.target_ir.clone(),
        source_path: spec.source_path.clone(),
        content_hash: spec.content_hash.clone(),
        total_rules: spec.rules.len(),
        rule_names: spec.rules.keys().cloned().collect(),
    };

    let yaml = serde_yaml::to_string(&info).unwrap();
    println!("{}", yaml);
}

#[allow(dead_code)]
fn dump_lowering_inspection(ast: &crate::frontend::schema_ast::SchemaAstNode) {
    println!("═══════════════════════════════════════════════════════════");
    println!("LOWERING INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct LoweringStep {
        node_kind: String,
        variant: Option<String>,
        action: String,
    }

    #[derive(Serialize)]
    struct LoweringInspection {
        steps: Vec<LoweringStep>,
    }

    fn collect_lowering_steps(
        node: &crate::frontend::schema_ast::SchemaAstNode,
        steps: &mut Vec<LoweringStep>,
    ) {
        use crate::frontend::schema_ast::SchemaValue;

        steps.push(LoweringStep {
            node_kind: node.kind.clone(),
            variant: None,
            action: "lower_node".to_string(),
        });

        for (_name, field_value) in &node.fields {
            match field_value {
                SchemaValue::Node(child) => collect_lowering_steps(child, steps),
                SchemaValue::Nodes(children) => {
                    for child in children {
                        collect_lowering_steps(child, steps);
                    }
                }
                _ => {}
            }
        }
    }

    let mut steps = Vec::new();
    collect_lowering_steps(ast, &mut steps);

    let inspection = LoweringInspection { steps };
    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

fn dump_core_ir_inspection(core_ir: &crate::ir::core_ir::CoreBundle) {
    println!("═══════════════════════════════════════════════════════════");
    println!("CORE IR INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct CoreIRInspection {
        version: String,
        term_type: String,
    }

    use crate::ir::core_ir::CoreTerm;

    let term_type = match &core_ir.core_term {
        CoreTerm::CIntLit { .. } => "CIntLit".to_string(),
        CoreTerm::CBoolLit { .. } => "CBoolLit".to_string(),
        CoreTerm::CUnitLit { .. } => "CUnitLit".to_string(),
        CoreTerm::CLam { .. } => "CLam".to_string(),
        CoreTerm::CLet { .. } => "CLet".to_string(),
        CoreTerm::CIf { .. } => "CIf".to_string(),
        CoreTerm::CVar { .. } => "CVar".to_string(),
        CoreTerm::CApp { .. } => "CApp".to_string(),
        CoreTerm::CCall { .. } => "CCall".to_string(),
    };

    let inspection = CoreIRInspection {
        version: core_ir.version.to_string(),
        term_type,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

fn dump_pipeline_inspection(
    executed_stages: &[String],
    timings: &[(String, std::time::Duration)],
    total_time: std::time::Duration,
    config: &PipelineConfig,
) {
    println!("═══════════════════════════════════════════════════════════");
    println!("PIPELINE INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct StageTiming {
        stage: String,
        duration_ms: f64,
    }

    #[derive(Serialize)]
    struct PipelineInspection {
        specs_loaded: Vec<String>,
        stages_executed: Vec<String>,
        total_stages: usize,
        timings: Vec<StageTiming>,
        total_duration_ms: f64,
    }

    let specs_loaded = vec![
        config.lexer_spec.display().to_string(),
        config.parser_spec.display().to_string(),
        config.ast_schema.display().to_string(),
    ];

    let stage_timings: Vec<StageTiming> = timings
        .iter()
        .map(|(name, dur)| StageTiming {
            stage: name.clone(),
            duration_ms: dur.as_secs_f64() * 1000.0,
        })
        .collect();

    let inspection = PipelineInspection {
        specs_loaded,
        stages_executed: executed_stages.to_vec(),
        total_stages: executed_stages.len(),
        timings: stage_timings,
        total_duration_ms: total_time.as_secs_f64() * 1000.0,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
}

// WAVE 2: Normalisation inspection
#[allow(dead_code)]
fn dump_normalisation_inspection(trace: &crate::normalisation::NormalisationTrace) {
    println!("═══════════════════════════════════════════════════════════");
    println!("NORMALISATION INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct PassInspection {
        pass_id: String,
        pass_name: String,
        duration_ms: f64,
        modified: bool,
        input_node_kind: String,
        output_node_kind: String,
    }

    #[derive(Serialize)]
    struct NormalisationInspection {
        total_passes: usize,
        total_duration_ms: f64,
        passes: Vec<PassInspection>,
        final_node_kind: String,
    }

    let mut passes = Vec::new();
    for exec in &trace.executions {
        passes.push(PassInspection {
            pass_id: exec.pass_id.clone(),
            pass_name: exec.pass_name.clone(),
            duration_ms: exec.duration.as_secs_f64() * 1000.0,
            modified: exec.modified,
            input_node_kind: exec.input_ast.kind.clone(),
            output_node_kind: exec.output_ast.kind.clone(),
        });
    }

    let inspection = NormalisationInspection {
        total_passes: trace.executions.len(),
        total_duration_ms: trace.total_duration.as_secs_f64() * 1000.0,
        passes,
        final_node_kind: trace.final_ast.kind.clone(),
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
    println!();

    // Pass-by-pass breakdown
    println!("Pass Execution Details:");
    println!();
    for exec in &trace.executions {
        println!("  {} - {}", exec.pass_id, exec.pass_name);
        println!(
            "    Duration: {:.3}ms",
            exec.duration.as_secs_f64() * 1000.0
        );
        println!("    Modified: {}", if exec.modified { "yes" } else { "no" });
        println!("    Input:  {}", exec.input_ast.kind);
        println!("    Output: {}", exec.output_ast.kind);
        println!();
    }

    println!("Final AST: {}", trace.final_ast.kind);
    println!();

    // WAVE 2: All passes are identity
    println!("Note: WAVE 2 passes are identity/sanity only.");
    println!("Future waves will implement desugaring rules.");
    println!();
}

// WAVE 1: Normal Form inspection
#[allow(dead_code)]
fn dump_nf_inspection(
    node: &schema_ast::SchemaAstNode,
    validation_result: &crate::normalisation::NfValidationResult,
) {
    println!("═══════════════════════════════════════════════════════════");
    println!("NORMAL FORM (NF) INSPECTION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    #[derive(Serialize)]
    struct NfInspection {
        nf_version: String,
        validation_passed: bool,
        error: Option<String>,
        admissible_nodes: Vec<String>,
        forbidden_nodes: Vec<String>,
        nodes_in_ast: Vec<String>,
    }

    let admissible_nodes: Vec<String> = crate::normalisation::get_admissible_nodes()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let forbidden_nodes: Vec<String> = crate::normalisation::get_forbidden_nodes()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let nodes_in_ast = crate::normalisation::collect_node_kinds(node);

    let (validation_passed, error) = match validation_result {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    let inspection = NfInspection {
        nf_version: "H1-1.0".to_string(),
        validation_passed,
        error,
        admissible_nodes,
        forbidden_nodes,
        nodes_in_ast,
    };

    let yaml = serde_yaml::to_string(&inspection).unwrap();
    println!("{}", yaml);
    println!();

    if !validation_passed {
        println!("❌ NF VALIDATION FAILED");
        println!();
        println!("The Schema AST contains non-NF constructs.");
        println!("Normalisation is required before lowering.");
        println!();
    } else {
        println!("✅ NF VALIDATION PASSED");
        println!();
        println!("The Schema AST conforms to Normal Form H1.");
        println!("Ready for lowering.");
        println!();
    }
}

// Wave T1: Code trace inspection
fn dump_code_trace_inspection(
    tokens: &[crate::frontend::token::Token],
    parse_tree: &crate::frontend::parser_runtime::ParseTree,
    schema_ast: &crate::frontend::schema_ast::SchemaAstNode,
    core_ir: &crate::ir::core_ir::CoreBundle,
) -> Result<(), PipelineError> {
    use crate::introspection::code_trace;

    let entries = code_trace::generate_code_trace(tokens, parse_tree, schema_ast, core_ir)
        .map_err(|e| PipelineError::Inspection {
            message: e.to_string(),
        })?;

    code_trace::render_trace(&entries);
    Ok(())
}

// WAVE 4: Hook execution inspection
#[allow(dead_code)]
fn dump_hook_inspection(trace: &crate::hooks::HookExecutionTrace) {
    // Use the built-in formatting from HookExecutionTrace
    print!("{}", trace.format_for_inspection());
}
