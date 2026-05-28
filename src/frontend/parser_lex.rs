// Parser lexer adapter: converts spec-driven tokens to legacy Phase 1 Token enum
// This allows parser to remain unchanged while using the spec-driven lexer backend

use crate::frontend::lexer_engine::lex_with_spec;
use crate::frontend::lexspec_load::load_spec;
use std::path::Path;

// Phase 1 parser token type (kept for parser compatibility)
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Fn,
    If,
    Else,
    Ident(Box<str>),
    LParen,
    RParen,
    Comma,
    LBrace,
    RBrace,
    Semi,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lexer error: {}", self.message)
    }
}

impl std::error::Error for LexError {}

/// Lex using spec-driven lexer and convert to Phase 1 Token enum
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    // Use canonical user-provided spec
    let spec_path = Path::new("lang-lab-poc-userfiles/lexer.yaml");

    let spec = load_spec(spec_path).map_err(|e| LexError {
        message: format!("failed to load parser spec: {}", e),
    })?;

    let spec_tokens = lex_with_spec(&spec, src).map_err(|e| LexError { message: e.message })?;

    // Convert spec-driven tokens to Phase 1 Token enum
    let mut result = Vec::new();

    for tok in spec_tokens {
        use crate::frontend::token::TokenKind;

        match tok.kind {
            TokenKind::Keyword(ref kw) => match kw.as_str() {
                "fn" => result.push(Token::Fn),
                "if" => result.push(Token::If),
                "else" => result.push(Token::Else),
                _ => {
                    return Err(LexError {
                        message: format!("unexpected keyword: {}", kw),
                    })
                }
            },
            TokenKind::Ident => {
                result.push(Token::Ident(tok.lexeme.into_boxed_str()));
            }
            TokenKind::UnitLit => {
                // Unit literal "()" is represented as LParen + RParen in Phase 1
                result.push(Token::LParen);
                result.push(Token::RParen);
            }
            TokenKind::Punct(ref p) => match p.as_str() {
                "(" => result.push(Token::LParen),
                ")" => result.push(Token::RParen),
                "{" => result.push(Token::LBrace),
                "}" => result.push(Token::RBrace),
                "," => result.push(Token::Comma),
                ";" => result.push(Token::Semi),
                _ => {
                    return Err(LexError {
                        message: format!("unexpected punctuation: {}", p),
                    })
                }
            },
            TokenKind::Eof => {
                // Skip EOF token - Phase 1 parser doesn't expect it
            }
            _ => {
                return Err(LexError {
                    message: format!("unexpected token kind: {:?}", tok.kind),
                });
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_valid() {
        let t = lex("fn foo() {}").unwrap();
        assert_eq!(
            t,
            vec![
                Token::Fn,
                Token::Ident("foo".into()),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace
            ]
        );
    }

    #[test]
    fn lex_rejects_number() {
        let e = lex("fn 123() {}").unwrap_err();
        assert!(e.message.contains("unexpected character") || e.message.contains("unexpected"));
    }
}
