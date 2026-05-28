// Minimal Phase 1 lexer: recognizes only the tokens required for Phase 1.
use std::fmt;

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

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexer error: {}", self.message)
    }
}

impl std::error::Error for LexError {}

/// Lex the given source string into a vector of `Token`.
/// Accepts only: `fn` keyword, identifiers, and the six punctuation tokens.
/// Whitespace between tokens is allowed. Any other character or token will
/// return an explicit `LexError`.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(&ch) = chars.peek() {
        // Skip ASCII whitespace
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        // Punctuation
        match ch {
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
                continue;
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
                continue;
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
                continue;
            }
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
                continue;
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
                continue;
            }
            ';' => {
                tokens.push(Token::Semi);
                chars.next();
                continue;
            }
            _ => {}
        }

        // Identifier or keyword: ASCII letters or underscore start
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(&c2) = chars.peek() {
                if c2.is_ascii_alphanumeric() || c2 == '_' {
                    ident.push(c2);
                    chars.next();
                } else {
                    break;
                }
            }

            if ident == "fn" {
                tokens.push(Token::Fn);
            } else if ident == "if" {
                tokens.push(Token::If);
            } else if ident == "else" {
                tokens.push(Token::Else);
            } else {
                tokens.push(Token::Ident(ident.into_boxed_str()));
            }
            continue;
        }

        // Any other character is explicitly rejected.
        return Err(LexError {
            message: format!("unexpected character '{}'", ch),
        });
    }

    Ok(tokens)
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
        assert!(e.message.contains("unexpected character"));
    }
}
