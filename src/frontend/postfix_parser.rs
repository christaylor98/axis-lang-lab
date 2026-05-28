// Wave AI1: Postfix (RPN) Parser Runtime
// Performs postfix stack reduction to build CST nodes
// Integrates with existing pipeline by producing ParseTree structures

use crate::frontend::parser_runtime::{ParseError, ParseNode, ParseTree};
use crate::frontend::token::{Span, Token, TokenKind};

/// Parse postfix (RPN) token stream into CST
///
/// Implements stack-based postfix reduction:
/// - Atoms (INT, BOOL, UNIT, IDENT) push to stack
/// - Operators (app, lam, let, if) pop operands and push result
///
/// Stack must contain exactly 1 expression at EOF.
///
/// HARD ERRORS:
/// - Stack underflow on any operator
/// - Stack size != 1 at EOF
/// - Missing IDENT for lam/let
/// - Parentheses in source (forbidden in AI-1)
pub fn parse_postfix(tokens: &[Token]) -> Result<ParseTree, ParseError> {
    let mut stack: Vec<ParseTree> = Vec::new();
    let mut token_iter = tokens.iter().peekable();

    while let Some(token) = token_iter.next() {
        // EOF: finish reduction
        if matches!(token.kind, TokenKind::Eof) {
            break;
        }

        // Process token based on kind
        match &token.kind {
            // Atoms: push to stack as CST nodes
            TokenKind::IntLit => {
                stack.push(make_int_lit(token.clone()));
            }
            TokenKind::BoolLit => {
                stack.push(make_bool_lit(token.clone()));
            }
            TokenKind::Keyword(kw) if kw == "unit" => {
                stack.push(make_unit_lit(token.clone()));
            }
            TokenKind::Keyword(kw) if kw == "true" || kw == "false" => {
                stack.push(make_bool_lit(token.clone()));
            }
            TokenKind::Ident => {
                stack.push(make_var_ref(token.clone()));
            }

            // Operators: pop operands, push result
            TokenKind::Keyword(kw) if kw == "app" => {
                // app: pop arg, pop fn, push App(fn, arg)
                let arg = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'app': stack underflow (expected arg on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                let func = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'app': stack underflow (expected fn on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                let result = make_app_expr(func, arg, token.span.clone());
                stack.push(result);
            }
            TokenKind::Keyword(kw) if kw == "lam" => {
                // lam IDENT: consume IDENT, pop body, push Lam(param, body)
                let param_token = token_iter.next().ok_or_else(|| ParseError {
                    message: "postfix 'lam': expected IDENT parameter after 'lam'".to_string(),
                    span: token.span.clone(),
                })?;
                if !matches!(param_token.kind, TokenKind::Ident) {
                    return Err(ParseError {
                        message: format!(
                            "postfix 'lam': expected IDENT, got {:?}",
                            param_token.kind
                        ),
                        span: param_token.span.clone(),
                    });
                }
                let body = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'lam': stack underflow (expected body on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                stack.push(make_lam_expr(param_token.clone(), body, token.span.clone()));
            }
            TokenKind::Keyword(kw) if kw == "let" => {
                // let IDENT: consume IDENT, pop body, pop value, push Let(name, value, body)
                let name_token = token_iter.next().ok_or_else(|| ParseError {
                    message: "postfix 'let': expected IDENT name after 'let'".to_string(),
                    span: token.span.clone(),
                })?;
                if !matches!(name_token.kind, TokenKind::Ident) {
                    return Err(ParseError {
                        message: format!(
                            "postfix 'let': expected IDENT, got {:?}",
                            name_token.kind
                        ),
                        span: name_token.span.clone(),
                    });
                }
                let body = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'let': stack underflow (expected body on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                let value = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'let': stack underflow (expected value on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                let result = make_let_expr(name_token.clone(), value, body, token.span.clone());
                stack.push(result);
            }
            TokenKind::Keyword(kw) if kw == "if" => {
                // if: pop else_branch, pop then_branch, pop cond, push If(cond, then, else)
                let else_branch = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'if': stack underflow (expected else_branch on stack)"
                        .to_string(),
                    span: token.span.clone(),
                })?;
                let then_branch = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'if': stack underflow (expected then_branch on stack)"
                        .to_string(),
                    span: token.span.clone(),
                })?;
                let cond = stack.pop().ok_or_else(|| ParseError {
                    message: "postfix 'if': stack underflow (expected cond on stack)".to_string(),
                    span: token.span.clone(),
                })?;
                stack.push(make_if_expr(
                    cond,
                    then_branch,
                    else_branch,
                    token.span.clone(),
                ));
            }

            // Forbidden: parentheses
            TokenKind::Punct(p) if p == "(" || p == ")" => {
                return Err(ParseError {
                    message: format!("AI-1 forbids parentheses: found '{}'", p),
                    span: token.span.clone(),
                });
            }

            _ => {
                return Err(ParseError {
                    message: format!("unexpected token in postfix stream: {:?}", token.kind),
                    span: token.span.clone(),
                });
            }
        }
    }

    // Final check: stack must contain exactly 1 expression
    if stack.len() != 1 {
        let span = if let Some(last_token) = tokens.last() {
            last_token.span.clone()
        } else {
            Span::new(0, 0)
        };
        return Err(ParseError {
            message: format!(
                "postfix reduction failed: stack contains {} expressions (expected 1)",
                stack.len()
            ),
            span,
        });
    }

    let result = stack.pop().unwrap();
    Ok(result)
}

// CST node constructors
// These produce ParseTree structures matching the expected schema

fn make_int_lit(token: Token) -> ParseTree {
    ParseTree {
        rule: "IntLit".to_string(),
        children: vec![ParseNode::Terminal(token.clone())],
        span: token.span.clone(),
    }
}

fn make_bool_lit(token: Token) -> ParseTree {
    ParseTree {
        rule: "BoolLit".to_string(),
        children: vec![ParseNode::Terminal(token.clone())],
        span: token.span.clone(),
    }
}

fn make_unit_lit(token: Token) -> ParseTree {
    ParseTree {
        rule: "UnitLit".to_string(),
        children: vec![ParseNode::Terminal(token.clone())],
        span: token.span.clone(),
    }
}

fn make_var_ref(token: Token) -> ParseTree {
    ParseTree {
        rule: "VarRef".to_string(),
        children: vec![ParseNode::Terminal(token.clone())],
        span: token.span.clone(),
    }
}

fn make_app_expr(func: ParseTree, arg: ParseTree, op_span: Span) -> ParseTree {
    // Span union: min(all starts), max(all ends)
    let start = func.span.start.min(arg.span.start).min(op_span.start);
    let end = func.span.end.max(arg.span.end).max(op_span.end);
    let combined_span = Span::new(start, end);

    // Children MUST be in source order for ast_builder span derivation
    let mut children_in_order = vec![ParseNode::Rule(func), ParseNode::Rule(arg)];
    children_in_order.sort_by_key(|child| match child {
        ParseNode::Terminal(t) => t.span.start,
        ParseNode::Rule(r) => r.span.start,
    });

    ParseTree {
        rule: "AppExpr".to_string(),
        children: children_in_order,
        span: combined_span,
    }
}

fn make_lam_expr(param: Token, body: ParseTree, op_span: Span) -> ParseTree {
    // Span union: include operator, param, and body spans
    let start = op_span.start.min(param.span.start).min(body.span.start);
    let end = op_span.end.max(param.span.end).max(body.span.end);
    let combined_span = Span::new(start, end);

    // Children MUST be in source order for ast_builder span derivation
    // Source order for postfix "... lam PARAM": operator, then param, then body
    let mut children_in_order = vec![ParseNode::Terminal(param.clone()), ParseNode::Rule(body)];
    children_in_order.sort_by_key(|child| match child {
        ParseNode::Terminal(t) => t.span.start,
        ParseNode::Rule(r) => r.span.start,
    });

    ParseTree {
        rule: "LamExpr".to_string(),
        children: children_in_order,
        span: combined_span,
    }
}

fn make_let_expr(name: Token, value: ParseTree, body: ParseTree, op_span: Span) -> ParseTree {
    // Span union: include operator, name token, value, and body spans
    let start = op_span
        .start
        .min(name.span.start)
        .min(value.span.start)
        .min(body.span.start);
    let end = op_span
        .end
        .max(name.span.end)
        .max(value.span.end)
        .max(body.span.end);
    let combined_span = Span::new(start, end);

    // Children MUST be in source order for ast_builder span derivation
    let mut children_in_order = vec![
        ParseNode::Terminal(name.clone()),
        ParseNode::Rule(value),
        ParseNode::Rule(body),
    ];
    children_in_order.sort_by_key(|child| match child {
        ParseNode::Terminal(t) => t.span.start,
        ParseNode::Rule(r) => r.span.start,
    });

    ParseTree {
        rule: "LetExpr".to_string(),
        children: children_in_order,
        span: combined_span,
    }
}

fn make_if_expr(
    cond: ParseTree,
    then_branch: ParseTree,
    else_branch: ParseTree,
    op_span: Span,
) -> ParseTree {
    // Span union: include operator and all branch spans
    let start = op_span
        .start
        .min(cond.span.start)
        .min(then_branch.span.start)
        .min(else_branch.span.start);
    let end = op_span
        .end
        .max(cond.span.end)
        .max(then_branch.span.end)
        .max(else_branch.span.end);
    let combined_span = Span::new(start, end);

    // Children MUST be in source order for ast_builder span derivation
    let mut children_in_order = vec![
        ParseNode::Rule(cond),
        ParseNode::Rule(then_branch),
        ParseNode::Rule(else_branch),
    ];
    children_in_order.sort_by_key(|child| match child {
        ParseNode::Terminal(t) => t.span.start,
        ParseNode::Rule(r) => r.span.start,
    });

    ParseTree {
        rule: "IfExpr".to_string(),
        children: children_in_order,
        span: combined_span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(kind: TokenKind, lexeme: &str, start: usize, end: usize) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span: Span::new(start, end),
        }
    }

    #[test]
    fn test_postfix_int_literal() {
        let tokens = vec![
            make_token(TokenKind::IntLit, "42", 0, 2),
            make_token(TokenKind::Eof, "", 2, 2),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.rule, "IntLit");
    }

    #[test]
    fn test_postfix_app() {
        // f x app  =>  App(f, x)
        let tokens = vec![
            make_token(TokenKind::Ident, "f", 0, 1),
            make_token(TokenKind::Ident, "x", 2, 3),
            make_token(TokenKind::Keyword("app".to_string()), "app", 4, 7),
            make_token(TokenKind::Eof, "", 7, 7),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.rule, "AppExpr");
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn test_postfix_lam() {
        // x lam y  =>  Lam(y, x)
        let tokens = vec![
            make_token(TokenKind::Ident, "x", 0, 1),
            make_token(TokenKind::Keyword("lam".to_string()), "lam", 2, 5),
            make_token(TokenKind::Ident, "y", 6, 7),
            make_token(TokenKind::Eof, "", 7, 7),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.rule, "LamExpr");
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn test_postfix_if() {
        // true 1 0 if  =>  If(true, 1, 0)
        let tokens = vec![
            make_token(TokenKind::Keyword("true".to_string()), "true", 0, 4),
            make_token(TokenKind::IntLit, "1", 5, 6),
            make_token(TokenKind::IntLit, "0", 7, 8),
            make_token(TokenKind::Keyword("if".to_string()), "if", 9, 11),
            make_token(TokenKind::Eof, "", 11, 11),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.rule, "IfExpr");
        assert_eq!(tree.children.len(), 3);
    }

    #[test]
    fn test_postfix_underflow() {
        // app (no operands on stack)
        let tokens = vec![
            make_token(TokenKind::Keyword("app".to_string()), "app", 0, 3),
            make_token(TokenKind::Eof, "", 3, 3),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_postfix_leftover_stack() {
        // x y (two atoms, no operator)
        let tokens = vec![
            make_token(TokenKind::Ident, "x", 0, 1),
            make_token(TokenKind::Ident, "y", 2, 3),
            make_token(TokenKind::Eof, "", 3, 3),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("stack contains 2 expressions"));
    }

    #[test]
    fn test_postfix_forbids_parens() {
        let tokens = vec![
            make_token(TokenKind::Punct("(".to_string()), "(", 0, 1),
            make_token(TokenKind::Eof, "", 1, 1),
        ];
        let result = parse_postfix(&tokens);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("forbids parentheses"));
    }
}
