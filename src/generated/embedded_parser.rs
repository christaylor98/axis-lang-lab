// GENERATED CODE - DO NOT EDIT
// Generated from: lang-lab-poc-userfiles/parsing.yaml
// Source of truth: YAML spec file
// This code embeds the parser spec for sealed builds

/// Embedded parser specification
pub const PARSER_SPEC: &str = r###"
parser:
  start: Program

  grammar:
    Program:
      - 'Decl*'

    Decl:
      - 'Function'
      - 'EnumDecl'

    Function:
      - '"fn" IDENT "(" ParamList? ")" "->" Type Block'

    ParamList:
      - 'Param ("," Param)*'

    Param:
      - 'IDENT ":" Type'

    Block:
      - '"{" Stmt* Expr "}"'

    Stmt:
      - 'LetStmt'

    LetStmt:
      - '"let" IDENT "=" Expr ";"'

    Expr:
      - 'IfExpr'
      - 'MatchExpr'
      - 'CallExpr'
      - 'LambdaExpr'
      - 'TupleExpr'
      - 'ProjExpr'
      - 'Literal'
      - 'IDENT'
      - 'Block'

    IfExpr:
      - '"if" Expr Block "else" Block'

    MatchExpr:
      - '"match" Expr "{" MatchArm+ "}"'

    MatchArm:
      - 'Pattern "=>" Expr ","'

    ProjExpr:
      - '"proj" "(" Expr "," INT ")"'

    TupleExpr:
      - '"(" Expr ("," Expr)+ ")"'

    CallExpr:
      - 'IDENT "(" ArgList? ")"'

    ArgList:
      - 'Expr ("," Expr)*'

    LambdaExpr:
      - '"|" IDENT "|" Expr'

    Type:
      - 'IDENT'

    EnumDecl:
      - '"enum" IDENT "{" IDENT ("," IDENT)* "}"'

    Pattern:
      - 'IDENT'

    Literal:
      - 'INT'
      - 'BOOL'
      - 'STRING'
      - 'UNIT'
"###;

/// Get embedded parser spec content
pub fn get_embedded_parser_spec() -> &'static str {
    PARSER_SPEC
}

