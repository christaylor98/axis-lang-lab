// Spec-Driven Lowering for Semantic-Surface-0
//
// This module implements lowering driven entirely by YAML specification.
// No hard-coded semantic knowledge beyond Core IR construction.
//
// WAVE S2.1 EXTENSION: Spec-driven lowering is now REQUIRED and ONLY way.

use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::frontend::token::Span;
use crate::ir::core_ir::{CoreBundle, CoreTerm, IdentOrName};
use crate::registry::Registry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════════
// SPEC STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoweringSpec {
    pub version: String,
    pub target_ir: String,
    pub rules: HashMap<String, LoweringRule>,

    // Wave S3: Metadata for spec authority tracking
    #[serde(skip)]
    pub source_path: Option<String>,
    #[serde(skip)]
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoweringRule {
    pub target: String,                  // Core IR term type (e.g., "CIntLit", "CLet")
    pub fields: HashMap<String, String>, // field_name -> "token:name" | "node:name" | "nodes:name"
}

// ═══════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct LoweringError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lowering error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for LoweringError {}

// ═══════════════════════════════════════════════════════════════════════════
// SPEC LOADER
// ═══════════════════════════════════════════════════════════════════════════

pub fn load_spec(path: &Path) -> Result<LoweringSpec, LoweringError> {
    // Wave S3: Explicit existence check with clear error message
    if !path.exists() {
        return Err(LoweringError {
            message: format!("lowering spec file not found: {}", path.display()),
            span: Span::new(0, 0),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| LoweringError {
        message: format!("failed to read lowering spec: {}", e),
        span: Span::new(0, 0),
    })?;

    // Wave S3: Compute content hash for determinism verification
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let mut spec: LoweringSpec = serde_yaml::from_str(&content).map_err(|e| LoweringError {
        message: format!("failed to parse lowering spec: {}", e),
        span: Span::new(0, 0),
    })?;

    // Wave S3: Validate version
    if spec.version != "0.1" {
        return Err(LoweringError {
            message: format!(
                "unsupported lowering spec version: {} (expected: 0.1)",
                spec.version
            ),
            span: Span::new(0, 0),
        });
    }

    // Wave S3: Validate target IR
    // NOTE: spec-driven lowering is legacy; accepts 0.2 for compatibility
    // but emits Core IR 0.3 (see lower_term_with_spec)
    if spec.target_ir != "0.2" {
        return Err(LoweringError {
            message: format!(
                "unsupported target IR version: {} (expected: 0.2)",
                spec.target_ir
            ),
            span: Span::new(0, 0),
        });
    }

    // Wave S3: Validate spec is not empty
    if spec.rules.is_empty() {
        return Err(LoweringError {
            message: "lowering spec contains no rules".to_string(),
            span: Span::new(0, 0),
        });
    }

    // Wave S3: Attach metadata for inspection
    spec.source_path = Some(path.display().to_string());
    spec.content_hash = Some(hash);

    Ok(spec)
}

// ═══════════════════════════════════════════════════════════════════════════
// SPEC-DRIVEN LOWERING
// ═══════════════════════════════════════════════════════════════════════════

/// Context for spec-driven lowering
pub struct LoweringContext {
    pub spec: LoweringSpec,
    pub registry: Registry,
    pub scope: Scope,
}

/// Variable scope for tracking bindings
#[derive(Debug, Clone)]
pub struct Scope {
    bindings: Vec<HashMap<String, Span>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            bindings: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.bindings.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    pub fn bind(&mut self, name: String, span: Span) {
        if let Some(current) = self.bindings.last_mut() {
            current.insert(name, span);
        }
    }

    pub fn lookup(&self, name: &str) -> bool {
        for scope in self.bindings.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
}

/// Lower Schema AST to Core IR using spec
pub fn lower_with_spec(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Normalize wrappers first (Wave S2.1)
    let normalized = normalize_wrappers(node);

    // Look up rule in spec - clone it to avoid borrow issues
    let rule = ctx
        .spec
        .rules
        .get(&normalized.kind)
        .ok_or_else(|| LoweringError {
            message: format!("no lowering rule for node kind '{}'", normalized.kind),
            span: normalized.span.clone(),
        })?
        .clone();

    // Dispatch based on target Core IR term
    match rule.target.as_str() {
        "CIntLit" => lower_int_lit(&normalized, &rule),
        "CBoolLit" => lower_bool_lit(&normalized, &rule),
        "CUnitLit" => lower_unit_lit(&normalized, &rule),
        "CVar" => lower_var(&normalized, &rule, ctx),
        "CLet" => lower_let(&normalized, &rule, ctx),
        "CLam" => lower_lam(&normalized, &rule, ctx),
        "CIf" => lower_if(&normalized, &rule, ctx),
        "CApp" => lower_app(&normalized, &rule, ctx),
        unknown => Err(LoweringError {
            message: format!("unsupported target Core IR term: {}", unknown),
            span: normalized.span.clone(),
        }),
    }
}

/// Lower to bundle (public API)
pub fn lower_to_bundle(
    node: &SchemaAstNode,
    spec: LoweringSpec,
    registry: Registry,
) -> Result<CoreBundle, LoweringError> {
    let mut ctx = LoweringContext {
        spec,
        registry,
        scope: Scope::new(),
    };

    let core_term = lower_with_spec(node, &mut ctx)?;
    Ok(crate::ir::core_ir::bundle_v0_2(core_term))
}

// ═══════════════════════════════════════════════════════════════════════════
// WRAPPER NORMALIZATION (Wave S2.1)
// ═══════════════════════════════════════════════════════════════════════════

fn normalize_wrappers(node: &SchemaAstNode) -> SchemaAstNode {
    match node.kind.as_str() {
        "Expr" | "AtomicExpr" => match node.fields.get("inner") {
            Some(SchemaValue::Node(inner)) => normalize_wrappers(inner),
            _ => node.clone(),
        },
        _ => {
            let mut normalized_fields = HashMap::new();
            for (field_name, field_value) in &node.fields {
                let normalized_value = match field_value {
                    SchemaValue::Node(child) => {
                        SchemaValue::Node(Box::new(normalize_wrappers(child)))
                    }
                    SchemaValue::Nodes(children) => {
                        SchemaValue::Nodes(children.iter().map(|c| normalize_wrappers(c)).collect())
                    }
                    SchemaValue::Token(t) => SchemaValue::Token(t.clone()),
                    SchemaValue::Tokens(ts) => SchemaValue::Tokens(ts.clone()),
                };
                normalized_fields.insert(field_name.clone(), normalized_value);
            }

            SchemaAstNode {
                kind: node.kind.clone(),
                fields: normalized_fields,
                annotations: node.annotations.clone(),
                span: node.span.clone(),
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOWERING FUNCTIONS (Spec-Driven)
// ═══════════════════════════════════════════════════════════════════════════

fn lower_int_lit(node: &SchemaAstNode, rule: &LoweringRule) -> Result<CoreTerm, LoweringError> {
    let value_field_name = extract_token_field_name(&rule.fields, "value")?;
    let token = get_field_token(node, &value_field_name)?;

    let value: i64 = token.lexeme.parse().map_err(|_| LoweringError {
        message: format!("invalid integer literal '{}'", token.lexeme),
        span: token.span.clone(),
    })?;

    Ok(CoreTerm::CIntLit {
        value,
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_bool_lit(node: &SchemaAstNode, rule: &LoweringRule) -> Result<CoreTerm, LoweringError> {
    let value_field_name = extract_token_field_name(&rule.fields, "value")?;
    let token = get_field_token(node, &value_field_name)?;

    let value = match token.lexeme.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(LoweringError {
                message: format!(
                    "invalid boolean literal '{}', expected 'true' or 'false'",
                    other
                ),
                span: token.span.clone(),
            })
        }
    };

    Ok(CoreTerm::CBoolLit {
        value,
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_unit_lit(node: &SchemaAstNode, _rule: &LoweringRule) -> Result<CoreTerm, LoweringError> {
    Ok(CoreTerm::CUnitLit {
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_var(
    node: &SchemaAstNode,
    rule: &LoweringRule,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name_field_name = extract_token_field_name(&rule.fields, "name")?;
    let token = get_field_token(node, &name_field_name)?;
    let name = &token.lexeme;

    if !ctx.scope.lookup(name) {
        return Err(LoweringError {
            message: format!("unbound variable '{}'", name),
            span: token.span.clone(),
        });
    }

    Ok(CoreTerm::CVar {
        name: IdentOrName::new(name.clone()),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_let(
    node: &SchemaAstNode,
    rule: &LoweringRule,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name_field_name = extract_token_field_name(&rule.fields, "name")?;
    let value_field_name = extract_node_field_name(&rule.fields, "value")?;
    let body_field_name = extract_node_field_name(&rule.fields, "body")?;

    let name_token = get_field_token(node, &name_field_name)?;
    let name = name_token.lexeme.clone();

    let value_node = get_field_node(node, &value_field_name)?;
    let value_term = lower_with_spec(value_node, ctx)?;

    ctx.scope.push_scope();
    ctx.scope.bind(name.clone(), name_token.span.clone());

    let body_node = get_field_node(node, &body_field_name)?;
    let body_term = lower_with_spec(body_node, ctx)?;

    ctx.scope.pop_scope();

    Ok(CoreTerm::CLet {
        name: IdentOrName::new(name),
        value: Box::new(value_term),
        body: Box::new(body_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_lam(
    node: &SchemaAstNode,
    rule: &LoweringRule,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let param_field_name = extract_token_field_name(&rule.fields, "param")?;
    let body_field_name = extract_node_field_name(&rule.fields, "body")?;

    let param_token = get_field_token(node, &param_field_name)?;
    let param_name = param_token.lexeme.clone();

    ctx.scope.push_scope();
    ctx.scope.bind(param_name.clone(), param_token.span.clone());

    let body_node = get_field_node(node, &body_field_name)?;
    let body_term = lower_with_spec(body_node, ctx)?;

    ctx.scope.pop_scope();

    Ok(CoreTerm::CLam {
        param: IdentOrName::new(param_name),
        body: Box::new(body_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_if(
    node: &SchemaAstNode,
    rule: &LoweringRule,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let cond_field_name = extract_node_field_name(&rule.fields, "cond")?;
    let then_field_name = extract_node_field_name(&rule.fields, "then_branch")?;
    let else_field_name = extract_node_field_name(&rule.fields, "else_branch")?;

    let cond_node = get_field_node(node, &cond_field_name)?;
    let cond_term = lower_with_spec(cond_node, ctx)?;

    let then_node = get_field_node(node, &then_field_name)?;
    let then_term = lower_with_spec(then_node, ctx)?;

    let else_node = get_field_node(node, &else_field_name)?;
    let else_term = lower_with_spec(else_node, ctx)?;

    Ok(CoreTerm::CIf {
        cond: Box::new(cond_term),
        then_branch: Box::new(then_term),
        else_branch: Box::new(else_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

fn lower_app(
    node: &SchemaAstNode,
    rule: &LoweringRule,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let parts_field_name = extract_nodes_field_name(&rule.fields, "parts")?;
    let parts = get_field_nodes(node, &parts_field_name)?;

    if parts.is_empty() {
        return Err(LoweringError {
            message: "AppExpr requires at least one part (the function)".to_string(),
            span: node.span.clone(),
        });
    }

    let mut result = lower_with_spec(&parts[0], ctx)?;

    for arg_node in &parts[1..] {
        let arg_term = lower_with_spec(arg_node, ctx)?;
        result = CoreTerm::CApp {
            func: Box::new(result),
            arg: Box::new(arg_term),
            node_id: None,
            annotations: Vec::new(),
        };
    }

    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

fn extract_token_field_name(
    fields: &HashMap<String, String>,
    target: &str,
) -> Result<String, LoweringError> {
    match fields.get(target) {
        Some(s) => {
            if let Some(name) = s.strip_prefix("token:") {
                Ok(name.to_string())
            } else {
                Err(LoweringError {
                    message: format!("field '{}' must be a token field (got: {})", target, s),
                    span: Span::new(0, 0),
                })
            }
        }
        None => Err(LoweringError {
            message: format!("field '{}' not found in rule", target),
            span: Span::new(0, 0),
        }),
    }
}

fn extract_node_field_name(
    fields: &HashMap<String, String>,
    target: &str,
) -> Result<String, LoweringError> {
    match fields.get(target) {
        Some(s) => {
            if let Some(name) = s.strip_prefix("node:") {
                Ok(name.to_string())
            } else {
                Err(LoweringError {
                    message: format!("field '{}' must be a node field (got: {})", target, s),
                    span: Span::new(0, 0),
                })
            }
        }
        None => Err(LoweringError {
            message: format!("field '{}' not found in rule", target),
            span: Span::new(0, 0),
        }),
    }
}

fn extract_nodes_field_name(
    fields: &HashMap<String, String>,
    target: &str,
) -> Result<String, LoweringError> {
    match fields.get(target) {
        Some(s) => {
            if let Some(name) = s.strip_prefix("nodes:") {
                Ok(name.to_string())
            } else {
                Err(LoweringError {
                    message: format!("field '{}' must be a nodes field (got: {})", target, s),
                    span: Span::new(0, 0),
                })
            }
        }
        None => Err(LoweringError {
            message: format!("field '{}' not found in rule", target),
            span: Span::new(0, 0),
        }),
    }
}

fn get_field_node<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a SchemaAstNode, LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Node(boxed_node) => Ok(boxed_node.as_ref()),
        _ => Err(LoweringError {
            message: format!("field '{}' in {} must be a Node", field_name, node.kind),
            span: node.span.clone(),
        }),
    }
}

fn get_field_nodes<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a [SchemaAstNode], LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Nodes(nodes) => Ok(nodes.as_slice()),
        _ => Err(LoweringError {
            message: format!("field '{}' in {} must be Nodes", field_name, node.kind),
            span: node.span.clone(),
        }),
    }
}

fn get_field_token<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a crate::frontend::token::Token, LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Token(token) => Ok(token),
        _ => Err(LoweringError {
            message: format!("field '{}' in {} must be a Token", field_name, node.kind),
            span: node.span.clone(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::token::{Token, TokenKind};
    use crate::registry::Registry;

    fn make_test_spec() -> LoweringSpec {
        let yaml = r#"
version: "0.1"
target_ir: "0.2"
rules:
  IntLit:
    target: CIntLit
    fields:
      value: token:value
  
  BoolLit:
    target: CBoolLit
    fields:
      value: token:value
  
  UnitLit:
    target: CUnitLit
    fields: {}
  
  VarRef:
    target: CVar
    fields:
      name: token:name
  
  LetExpr:
    target: CLet
    fields:
      name: token:name
      value: node:value
      body: node:body
  
  LamExpr:
    target: CLam
    fields:
      param: token:param
      body: node:body
  
  IfExpr:
    target: CIf
    fields:
      cond: node:condition
      then_branch: node:then_branch
      else_branch: node:else_branch
  
  AppExpr:
    target: CApp
    fields:
      parts: nodes:parts
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    fn make_int_lit(value: i64) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(
            "value".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::IntLit,
                lexeme: value.to_string(),
                span: Span::new(0, value.to_string().len()),
            }),
        );
        SchemaAstNode {
            kind: "IntLit".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, value.to_string().len()),
        }
    }

    fn make_bool_lit(value: bool) -> SchemaAstNode {
        let lexeme = if value { "true" } else { "false" };
        let mut fields = HashMap::new();
        fields.insert(
            "value".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::BoolLit,
                lexeme: lexeme.to_string(),
                span: Span::new(0, lexeme.len()),
            }),
        );
        SchemaAstNode {
            kind: "BoolLit".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, lexeme.len()),
        }
    }

    fn make_unit_lit() -> SchemaAstNode {
        SchemaAstNode {
            kind: "UnitLit".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 2),
        }
    }

    fn make_var_ref(name: &str) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: name.to_string(),
                span: Span::new(0, name.len()),
            }),
        );
        SchemaAstNode {
            kind: "VarRef".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, name.len()),
        }
    }

    #[test]
    fn test_spec_driven_int_lit() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        let node = make_int_lit(42);
        let result = lower_with_spec(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 42),
            other => panic!("Expected CIntLit, got {:?}", other),
        }
    }

    #[test]
    fn test_spec_driven_bool_lit() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        let node = make_bool_lit(true);
        let result = lower_with_spec(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CBoolLit { value, .. } => assert_eq!(value, true),
            other => panic!("Expected CBoolLit, got {:?}", other),
        }
    }

    #[test]
    fn test_spec_driven_unit_lit() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        let node = make_unit_lit();
        let result = lower_with_spec(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CUnitLit { .. } => {}
            other => panic!("Expected CUnitLit, got {:?}", other),
        }
    }

    #[test]
    fn test_spec_driven_let_expr() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        // let x = 10 in VarRef(x)
        let mut let_fields = HashMap::new();
        let_fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(0, 1),
            }),
        );
        let_fields.insert(
            "value".to_string(),
            SchemaValue::Node(Box::new(make_int_lit(10))),
        );
        let_fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_var_ref("x"))),
        );

        let let_node = SchemaAstNode {
            kind: "LetExpr".to_string(),
            fields: let_fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let result = lower_with_spec(&let_node, &mut ctx);
        if result.is_err() {
            eprintln!("Error: {}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CLet { name, value, .. } => {
                assert_eq!(&*name.0, "x");
                match *value {
                    CoreTerm::CIntLit { value: v, .. } => assert_eq!(v, 10),
                    other => panic!("Expected CIntLit in value, got {:?}", other),
                }
            }
            other => panic!("Expected CLet, got {:?}", other),
        }
    }

    #[test]
    fn test_spec_driven_missing_rule() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        // Create a node with no rule in spec
        let unknown_node = SchemaAstNode {
            kind: "UnknownNode".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 5),
        };

        let result = lower_with_spec(&unknown_node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("no lowering rule"));
    }

    #[test]
    fn test_spec_driven_wrapper_normalization() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };
        let mut ctx = LoweringContext {
            spec,
            registry,
            scope: Scope::new(),
        };

        // Expr(IntLit(99))
        let inner = make_int_lit(99);
        let mut wrapper_fields = HashMap::new();
        wrapper_fields.insert("inner".to_string(), SchemaValue::Node(Box::new(inner)));
        let wrapper_node = SchemaAstNode {
            kind: "Expr".to_string(),
            fields: wrapper_fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let result = lower_with_spec(&wrapper_node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 99),
            other => panic!("Expected CIntLit(99), got {:?}", other),
        }
    }

    #[test]
    fn test_spec_load_invalid_version() {
        let yaml = r#"
version: "99.0"
target_ir: "0.2"
rules: {}
"#;
        let result: Result<LoweringSpec, _> = serde_yaml::from_str(yaml);
        assert!(result.is_ok());

        // Version validation happens in load_spec, not parsing
        let spec = result.unwrap();
        assert_eq!(spec.version, "99.0");
    }

    #[test]
    fn test_spec_to_bundle() {
        let spec = make_test_spec();
        let registry = Registry { entries: vec![] };

        let node = make_int_lit(42);
        let result = lower_to_bundle(&node, spec, registry);
        assert!(result.is_ok());

        let bundle = result.unwrap();
        assert_eq!(bundle.version.as_ref(), "0.3");
    }
}
