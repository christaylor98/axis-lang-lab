// GENERATED CODE - DO NOT EDIT
// Generated from: lang-lab-poc-userfiles/ast_schema.yaml
// Source of truth: YAML spec file
// This code embeds the AST schema for sealed builds

/// Embedded AST schema specification
pub const AST_SCHEMA: &str = r###"
nodes:
  Program:
    fields:
      decls: [Decl]

  Decl:
    union:
      - Function
      - EnumDecl

  Function:
    fields:
      name: Ident
      params: [Param]
      return_type: Type
      body: Block

  Param:
    fields:
      name: Ident
      type: Type

  Block:
    fields:
      statements: [LetStmt]
      result: Expr

  LetStmt:
    fields:
      name: Ident
      value: Expr

  Expr:
    union:
      - IfExpr
      - MatchExpr
      - CallExpr
      - LambdaExpr
      - TupleExpr
      - ProjExpr
      - Literal
      - IdentRef
      - Block

  ProjExpr:
    fields:
      expr: Expr
      index: int

  TupleExpr:
    fields:
      items: [Expr]
"###;

/// Get embedded AST schema content
pub fn get_embedded_ast_schema() -> &'static str {
    AST_SCHEMA
}

