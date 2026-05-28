// Spec-driven lexer engine - NO hard-coded token rules

use crate::frontend::lexspec::LexerSpec;
use crate::frontend::token::{Span, Token, TokenKind};
use regex::Regex;
use std::fmt;

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub span: Option<Span>,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = &self.span {
            write!(f, "lex error at {}..{}: {}", s.start, s.end, self.message)
        } else {
            write!(f, "lex error: {}", self.message)
        }
    }
}

impl std::error::Error for LexError {}

/// Compiled lexer rules from spec
struct CompiledRules {
    whitespace_re: Option<Regex>,
    comment_res: Vec<Regex>,
    ident_re: Option<Regex>,
    int_re: Option<Regex>,
    keywords: Vec<String>,
    bool_values: Vec<String>,
    unit_literal: Option<String>,
    string_delimiter: Option<String>,
    string_escapes: Vec<String>,
    string_forbid_newlines: bool,
    punctuation: Vec<String>, // Sorted longest-first
}

impl CompiledRules {
    fn from_spec(spec: &LexerSpec) -> Result<Self, LexError> {
        let cfg = &spec.lexer;

        // Compile whitespace pattern
        let whitespace_re = if let Some(ws) = &cfg.whitespace {
            Some(Regex::new(&ws.pattern).map_err(|e| LexError {
                message: format!("invalid whitespace pattern: {}", e),
                span: None,
            })?)
        } else {
            None
        };

        // Compile comment patterns
        let mut comment_res = Vec::new();
        for comment in &cfg.comments {
            let re = Regex::new(&comment.pattern).map_err(|e| LexError {
                message: format!("invalid comment pattern: {}", e),
                span: None,
            })?;
            comment_res.push(re);
        }

        // Compile identifier pattern
        let ident_re = if let Some(ident) = &cfg.identifiers {
            Some(Regex::new(&ident.pattern).map_err(|e| LexError {
                message: format!("invalid identifier pattern: {}", e),
                span: None,
            })?)
        } else {
            None
        };

        // Compile int literal pattern
        let int_re = if let Some(lits) = &cfg.literals {
            if let Some(int_rule) = &lits.int {
                Some(Regex::new(&int_rule.pattern).map_err(|e| LexError {
                    message: format!("invalid int literal pattern: {}", e),
                    span: None,
                })?)
            } else {
                None
            }
        } else {
            None
        };

        // Extract bool values
        let bool_values = if let Some(lits) = &cfg.literals {
            if let Some(bool_rule) = &lits.bool {
                bool_rule.values.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Extract unit literal
        let unit_literal = if let Some(lits) = &cfg.literals {
            lits.unit.as_ref().map(|u| u.literal.clone())
        } else {
            None
        };

        // Extract string literal rules
        let (string_delimiter, string_escapes, string_forbid_newlines) =
            if let Some(lits) = &cfg.literals {
                if let Some(str_rule) = &lits.string {
                    (
                        Some(str_rule.delimiter.clone()),
                        str_rule.escapes.clone(),
                        str_rule.forbid_newlines,
                    )
                } else {
                    (None, Vec::new(), true)
                }
            } else {
                (None, Vec::new(), true)
            };

        // Sort punctuation longest-first for longest-match
        let mut punctuation = cfg.punctuation.clone();
        punctuation.sort_by(|a, b| b.len().cmp(&a.len()));

        Ok(CompiledRules {
            whitespace_re,
            comment_res,
            ident_re,
            int_re,
            keywords: cfg.keywords.clone(),
            bool_values,
            unit_literal,
            string_delimiter,
            string_escapes,
            string_forbid_newlines,
            punctuation,
        })
    }
}

/// Lex source code using the provided spec
pub fn lex_with_spec(spec: &LexerSpec, src: &str) -> Result<Vec<Token>, LexError> {
    let rules = CompiledRules::from_spec(spec)?;
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = src.as_bytes();

    while pos < bytes.len() {
        let start_pos = pos;

        // Try whitespace
        if let Some(ws_re) = &rules.whitespace_re {
            if let Some(m) = ws_re.find(&src[pos..]) {
                if m.start() == 0 {
                    pos += m.end();
                    // Skip whitespace - don't emit token
                    continue;
                }
            }
        }

        // Try comments
        let mut comment_matched = false;
        for comment_re in &rules.comment_res {
            if let Some(m) = comment_re.find(&src[pos..]) {
                if m.start() == 0 {
                    pos += m.end();
                    comment_matched = true;
                    // Skip comment - don't emit token
                    break;
                }
            }
        }
        if comment_matched {
            continue;
        }

        // Try unit literal "()" - must come before punctuation
        if let Some(unit_lit) = &rules.unit_literal {
            if src[pos..].starts_with(unit_lit) {
                let end_pos = pos + unit_lit.len();
                tokens.push(Token::new(
                    TokenKind::UnitLit,
                    unit_lit.clone(),
                    Span::new(start_pos, end_pos),
                ));
                pos = end_pos;
                continue;
            }
        }

        // Try punctuation (longest match)
        let mut punct_matched = false;
        for punct in &rules.punctuation {
            if src[pos..].starts_with(punct) {
                let end_pos = pos + punct.len();
                tokens.push(Token::new(
                    TokenKind::Punct(punct.clone()),
                    punct.clone(),
                    Span::new(start_pos, end_pos),
                ));
                pos = end_pos;
                punct_matched = true;
                break;
            }
        }
        if punct_matched {
            continue;
        }

        // Try string literal
        if let Some(delim) = &rules.string_delimiter {
            if src[pos..].starts_with(delim) {
                match lex_string(
                    &src[pos..],
                    delim,
                    &rules.string_escapes,
                    rules.string_forbid_newlines,
                ) {
                    Ok((lexeme, consumed)) => {
                        let end_pos = pos + consumed;
                        tokens.push(Token::new(
                            TokenKind::StringLit,
                            lexeme,
                            Span::new(start_pos, end_pos),
                        ));
                        pos = end_pos;
                        continue;
                    }
                    Err(msg) => {
                        return Err(LexError {
                            message: msg,
                            span: Some(Span::new(start_pos, pos + 1)),
                        });
                    }
                }
            }
        }

        // Try int literal
        if let Some(int_re) = &rules.int_re {
            if let Some(m) = int_re.find(&src[pos..]) {
                if m.start() == 0 {
                    let lexeme = m.as_str().to_string();
                    let end_pos = pos + m.end();
                    tokens.push(Token::new(
                        TokenKind::IntLit,
                        lexeme,
                        Span::new(start_pos, end_pos),
                    ));
                    pos = end_pos;
                    continue;
                }
            }
        }

        // Try bool literals (must come BEFORE identifiers to avoid keyword collision)
        let mut bool_matched = false;
        for bool_val in &rules.bool_values {
            if src[pos..].starts_with(bool_val) {
                // Check boundary (not followed by alphanumeric or _)
                let end_idx = pos + bool_val.len();
                let at_boundary = end_idx >= src.len() || {
                    let next_char = src[end_idx..].chars().next().unwrap();
                    !next_char.is_alphanumeric() && next_char != '_'
                };

                if at_boundary {
                    tokens.push(Token::new(
                        TokenKind::BoolLit,
                        bool_val.clone(),
                        Span::new(start_pos, end_idx),
                    ));
                    pos = end_idx;
                    bool_matched = true;
                    break;
                }
            }
        }
        if bool_matched {
            continue;
        }

        // Try identifier (also checks for keywords)
        if let Some(ident_re) = &rules.ident_re {
            if let Some(m) = ident_re.find(&src[pos..]) {
                if m.start() == 0 {
                    let lexeme = m.as_str().to_string();
                    let end_pos = pos + m.end();

                    // Check if it's a keyword OR bool value (keywords take precedence over identifiers)
                    let kind = if rules.keywords.contains(&lexeme) {
                        TokenKind::Keyword(lexeme.clone())
                    } else if rules.bool_values.contains(&lexeme) {
                        // Handle bool values that are also listed as keywords
                        TokenKind::Keyword(lexeme.clone())
                    } else {
                        TokenKind::Ident
                    };

                    tokens.push(Token::new(kind, lexeme, Span::new(start_pos, end_pos)));
                    pos = end_pos;
                    continue;
                }
            }
        }

        // Unknown character - error
        let bad_char = src[pos..].chars().next().unwrap_or('?');
        return Err(LexError {
            message: format!("unexpected character '{}'", bad_char),
            span: Some(Span::new(pos, pos + bad_char.len_utf8())),
        });
    }

    // Add EOF token
    tokens.push(Token::new(
        TokenKind::Eof,
        String::new(),
        Span::new(pos, pos),
    ));

    Ok(tokens)
}

/// Lex a string literal with escape handling
fn lex_string(
    src: &str,
    delimiter: &str,
    escapes: &[String],
    forbid_newlines: bool,
) -> Result<(String, usize), String> {
    if !src.starts_with(delimiter) {
        return Err("not a string literal".to_string());
    }

    let mut pos = delimiter.len();
    let mut result = String::new();
    result.push_str(delimiter);

    while pos < src.len() {
        // Check for closing delimiter
        if src[pos..].starts_with(delimiter) {
            result.push_str(delimiter);
            pos += delimiter.len();
            return Ok((result, pos));
        }

        // Check for newline
        if forbid_newlines {
            let ch = src[pos..].chars().next().unwrap();
            if ch == '\n' || ch == '\r' {
                return Err("unterminated string literal (newline not allowed)".to_string());
            }
        }

        // Check for escape sequences
        let mut escape_matched = false;
        for esc in escapes {
            if src[pos..].starts_with(esc) {
                result.push_str(esc);
                pos += esc.len();
                escape_matched = true;
                break;
            }
        }

        if !escape_matched {
            let ch = src[pos..].chars().next().unwrap();
            result.push(ch);
            pos += ch.len_utf8();
        }
    }

    Err("unterminated string literal".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::lexspec_load::load_spec;
    use std::path::PathBuf;

    #[test]
    fn lex_simple_code() {
        let spec_path = PathBuf::from("lang-lab-poc-userfiles/lexer.yaml");
        if !spec_path.exists() {
            return; // Skip if spec not available
        }

        let spec = load_spec(&spec_path).unwrap();
        let src = "fn test() {}";
        let tokens = lex_with_spec(&spec, src).unwrap();

        // Should have: fn, test, (), {, }, EOF = 6 tokens (note: () is single unit literal)
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].kind, TokenKind::Keyword(_)));
        assert_eq!(tokens[0].lexeme, "fn");
        assert!(matches!(tokens[1].kind, TokenKind::Ident));
        assert_eq!(tokens[1].lexeme, "test");
    }

    #[test]
    fn lex_preserves_spans() {
        let spec_path = PathBuf::from("lang-lab-poc-userfiles/lexer.yaml");
        if !spec_path.exists() {
            return;
        }

        let spec = load_spec(&spec_path).unwrap();
        let src = "fn";
        let tokens = lex_with_spec(&spec, src).unwrap();

        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 2);
    }
}
