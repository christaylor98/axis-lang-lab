// Minimal Phase 1 parser: accepts only `fn IDENT() {}` and constructs
// the precise AST nodes required by Phase 1 lowering.
use crate::frontend::ast::{Block, Expr, FunctionBody, FunctionDecl, Ident, NoParams};
use crate::frontend::parser_lex::Token;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: &[Token]) -> Result<FunctionDecl, ParseError> {
    let mut cursor = 0;

    let expect = |idx: usize, want: &str| -> Result<(), ParseError> {
        if idx >= tokens.len() {
            return Err(ParseError {
                message: format!("unexpected end of tokens, expected {}", want),
            });
        }
        Ok(())
    };

    // Fn
    expect(cursor, "fn")?;
    match tokens.get(cursor) {
        Some(Token::Fn) => {}
        _ => {
            return Err(ParseError {
                message: "expected 'fn' keyword".into(),
            })
        }
    }
    cursor += 1;

    // Ident
    expect(cursor, "identifier")?;
    let name = match tokens.get(cursor) {
        Some(Token::Ident(s)) => s.clone(),
        _ => {
            return Err(ParseError {
                message: "expected identifier after 'fn'".into(),
            })
        }
    };
    cursor += 1;

    // LParen
    expect(cursor, "(")?;
    match tokens.get(cursor) {
        Some(Token::LParen) => {}
        _ => {
            return Err(ParseError {
                message: "expected '(' after identifier".into(),
            })
        }
    }
    cursor += 1;

    // RParen
    expect(cursor, ")")?;
    match tokens.get(cursor) {
        Some(Token::RParen) => {}
        _ => {
            return Err(ParseError {
                message: "expected ')' after '('".into(),
            })
        }
    }
    cursor += 1;

    // LBrace
    expect(cursor, "{")?;
    match tokens.get(cursor) {
        Some(Token::LBrace) => {}
        _ => {
            return Err(ParseError {
                message: "expected '{' after ')'".into(),
            })
        }
    }
    cursor += 1;

    // Parse block body
    let (block, new_cursor) = parse_block(tokens, cursor)?;
    cursor = new_cursor;

    // No extra tokens allowed
    if cursor != tokens.len() {
        return Err(ParseError {
            message: "unexpected extra tokens after function declaration".into(),
        });
    }

    Ok(FunctionDecl {
        name: Ident { value: name },
        params: NoParams,
        body: FunctionBody::Block(block),
    })
}

fn parse_block(tokens: &[Token], start: usize) -> Result<(Block, usize), ParseError> {
    let mut cursor = start;
    let mut exprs: Vec<Expr> = Vec::new();

    // If immediate RBrace, interpret as single UnitLit per spec
    if let Some(Token::RBrace) = tokens.get(cursor) {
        exprs.push(Expr::UnitLit);
        cursor += 1;
        return Ok((Block { exprs }, cursor));
    }

    // Parse sequence of expressions until RBrace
    loop {
        let (expr, new_cursor) = parse_expr(tokens, cursor)?;
        exprs.push(expr);
        cursor = new_cursor;

        // After an expression, either a semicolon, another expression, or a closing brace must follow
        match tokens.get(cursor) {
            Some(Token::Semi) => {
                cursor += 1;
            }
            Some(Token::RBrace) => {
                cursor += 1;
                break;
            }
            Some(Token::LBrace) | Some(Token::If) => { /* adjacent expression, continue loop */ }
            Some(tok) => {
                return Err(ParseError {
                    message: format!("unexpected token in block: {:?}", tok),
                })
            }
            None => {
                return Err(ParseError {
                    message: "unexpected end while parsing block".into(),
                })
            }
        }
    }

    Ok((Block { exprs }, cursor))
}

fn parse_expr(tokens: &[Token], start: usize) -> Result<(Expr, usize), ParseError> {
    let cursor = start;

    if cursor >= tokens.len() {
        return Err(ParseError {
            message: "unexpected end of tokens, expected expression".into(),
        });
    }

    match tokens.get(cursor) {
        Some(Token::LBrace) => {
            // UnitLit: {}
            if let Some(Token::RBrace) = tokens.get(cursor + 1) {
                Ok((Expr::UnitLit, cursor + 2))
            } else {
                Err(ParseError {
                    message: "expected '}' for unit literal expression".into(),
                })
            }
        }
        Some(Token::If) => {
            // if <expr> { <block> } else { <block> }
            let mut cursor = cursor + 1;

            // Parse condition expression
            let (cond, new_cursor) = parse_expr(tokens, cursor)?;
            cursor = new_cursor;

            // Expect then block: { <block> }
            if !matches!(tokens.get(cursor), Some(Token::LBrace)) {
                return Err(ParseError {
                    message: "expected '{' for then block".into(),
                });
            }
            cursor += 1;

            let (then_block, new_cursor) = parse_block(tokens, cursor)?;
            cursor = new_cursor;

            // Expect else keyword
            if !matches!(tokens.get(cursor), Some(Token::Else)) {
                return Err(ParseError {
                    message: "expected 'else' after then block".into(),
                });
            }
            cursor += 1;

            // Expect else block: { <block> }
            if !matches!(tokens.get(cursor), Some(Token::LBrace)) {
                return Err(ParseError {
                    message: "expected '{' for else block".into(),
                });
            }
            cursor += 1;

            let (else_block, new_cursor) = parse_block(tokens, cursor)?;
            cursor = new_cursor;

            Ok((
                Expr::If {
                    cond: Box::new(cond),
                    then_block,
                    else_block,
                },
                cursor,
            ))
        }
        Some(Token::Ident(_)) => {
            // Ident starting an expression must be a call (IDENT LPAREN).
            // Standalone identifiers are not valid expressions here.
            if matches!(tokens.get(cursor + 1), Some(Token::LParen)) {
                // Parse call expression starting at cursor
                parse_call(tokens, cursor)
            } else {
                Err(ParseError {
                    message: "unexpected identifier outside call expression".into(),
                })
            }
        }
        _ => Err(ParseError {
            message: "expected expression".into(),
        }),
    }
}

fn parse_call(tokens: &[Token], start: usize) -> Result<(Expr, usize), ParseError> {
    // start points at IDENT
    let mut cursor = start;
    let name = match tokens.get(cursor) {
        Some(Token::Ident(s)) => s.clone(),
        _ => {
            return Err(ParseError {
                message: "expected identifier for call".into(),
            })
        }
    };
    cursor += 1;

    // Expect LParen
    if !matches!(tokens.get(cursor), Some(Token::LParen)) {
        return Err(ParseError {
            message: "expected '(' after call name".into(),
        });
    }
    cursor += 1;

    let mut args: Vec<Expr> = Vec::new();

    // Empty args
    if matches!(tokens.get(cursor), Some(Token::RParen)) {
        cursor += 1;
        return Ok((
            Expr::Call {
                name: Ident { value: name },
                args,
            },
            cursor,
        ));
    }

    loop {
        // Parse an argument. Arguments may be:
        // - a nested call (IDENT LPAREN)
        // - a bare identifier (IDENT) parsed as Expr::Ident
        // - a full expression (UnitLit or If)
        match tokens.get(cursor) {
            Some(Token::Ident(_)) => {
                if matches!(tokens.get(cursor + 1), Some(Token::LParen)) {
                    let (expr, new_cursor) = parse_expr(tokens, cursor)?;
                    args.push(expr);
                    cursor = new_cursor;
                } else {
                    // Bare identifier as argument
                    if let Some(Token::Ident(s)) = tokens.get(cursor) {
                        args.push(Expr::Ident(Ident { value: s.clone() }));
                        cursor += 1;
                    }
                }
            }
            Some(Token::LBrace) | Some(Token::If) => {
                let (expr, new_cursor) = parse_expr(tokens, cursor)?;
                args.push(expr);
                cursor = new_cursor;
            }
            Some(tok) => {
                return Err(ParseError {
                    message: format!("unexpected token in call arg: {:?}", tok),
                })
            }
            None => {
                return Err(ParseError {
                    message: "unexpected end while parsing call arguments".into(),
                })
            }
        }

        match tokens.get(cursor) {
            Some(Token::Comma) => {
                cursor += 1;
                continue;
            }
            Some(Token::RParen) => {
                cursor += 1;
                break;
            }
            Some(tok) => {
                return Err(ParseError {
                    message: format!("expected ',' or ')' after call arg, found {:?}", tok),
                })
            }
            None => {
                return Err(ParseError {
                    message: "unexpected end after call argument".into(),
                })
            }
        }
    }

    Ok((
        Expr::Call {
            name: Ident { value: name },
            args,
        },
        cursor,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser_lex::lex;

    #[test]
    fn parse_valid() {
        let toks = lex("fn x() {}").unwrap();
        let ast = parse(&toks).unwrap();
        assert_eq!(&*ast.name.value, "x");
    }

    #[test]
    fn parse_rejects_extra() {
        let toks = lex("fn x() {} foo").unwrap();
        let e = parse(&toks).unwrap_err();
        assert!(e.message.contains("extra tokens"));
    }
}
