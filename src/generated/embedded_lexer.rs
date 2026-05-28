// GENERATED CODE - DO NOT EDIT
// Generated from: lang-lab-poc-userfiles/lexer.yaml
// Source of truth: YAML spec file
// This code must pass equivalence tests against runtime lexer

use crate::frontend::token::{Token, TokenKind, Span};
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

use once_cell::sync::Lazy;

static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r###"[ \t\n\r]+"###).unwrap()
});

static COMMENT_RES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r###"//.*"###).unwrap(),
    ]
});

static IDENT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r###"[A-Za-z_][A-Za-z0-9_]*"###).unwrap()
});

static INT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r###"[0-9]+"###).unwrap()
});

const KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "if",
    "else",
    "match",
    "enum",
    "proj",
];

const BOOL_VALUES: &[&str] = &[
    "true",
    "false",
];

const UNIT_LITERAL: &str = "()";

const STRING_DELIMITER: &str = "\"";
const STRING_ESCAPES: &[&str] = &[
    "\\\"",
    "\\\\",
    "\\n",
    "\\t",
];
const STRING_FORBID_NEWLINES: bool = true;

const PUNCTUATION: &[&str] = &[
    "=>",
    "->",
    "{",
    "}",
    "(",
    ")",
    ",",
    ":",
    "=",
    ";",
    "|",
];

/// Generated lexer - must be equivalent to runtime spec
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = src.as_bytes();

    while pos < bytes.len() {
        let start_pos = pos;

        // Whitespace
        if let Some(m) = WHITESPACE_RE.find(&src[pos..]) {
            if m.start() == 0 {
                pos += m.end();
                continue;
            }
        }

        // Comments
        let mut comment_matched = false;
        for comment_re in COMMENT_RES.iter() {
            if let Some(m) = comment_re.find(&src[pos..]) {
                if m.start() == 0 {
                    pos += m.end();
                    comment_matched = true;
                    break;
                }
            }
        }
        if comment_matched { continue; }

        // Unit literal
        if src[pos..].starts_with(UNIT_LITERAL) {
            let end_pos = pos + UNIT_LITERAL.len();
            tokens.push(Token::new(
                TokenKind::UnitLit,
                UNIT_LITERAL.to_string(),
                Span::new(start_pos, end_pos),
            ));
            pos = end_pos;
            continue;
        }

        // Punctuation
        let mut punct_matched = false;
        for punct in PUNCTUATION {
            if src[pos..].starts_with(punct) {
                let end_pos = pos + punct.len();
                tokens.push(Token::new(
                    TokenKind::Punct(punct.to_string()),
                    punct.to_string(),
                    Span::new(start_pos, end_pos),
                ));
                pos = end_pos;
                punct_matched = true;
                break;
            }
        }
        if punct_matched { continue; }

        // String literal
        if src[pos..].starts_with(STRING_DELIMITER) {
            match lex_string(&src[pos..]) {
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

        // Int literal
        if let Some(m) = INT_RE.find(&src[pos..]) {
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

        // Bool literal
        let mut bool_matched = false;
        for bool_val in BOOL_VALUES {
            if src[pos..].starts_with(bool_val) {
                let end_idx = pos + bool_val.len();
                let at_boundary = end_idx >= src.len() || {
                    let next_char = src[end_idx..].chars().next().unwrap();
                    !next_char.is_alphanumeric() && next_char != '_'
                };
                if at_boundary {
                    tokens.push(Token::new(
                        TokenKind::BoolLit,
                        bool_val.to_string(),
                        Span::new(start_pos, end_idx),
                    ));
                    pos = end_idx;
                    bool_matched = true;
                    break;
                }
            }
        }
        if bool_matched { continue; }

        // Identifier or keyword
        if let Some(m) = IDENT_RE.find(&src[pos..]) {
            if m.start() == 0 {
                let lexeme = m.as_str().to_string();
                let end_pos = pos + m.end();
                let kind = if KEYWORDS.contains(&lexeme.as_str()) {
                    TokenKind::Keyword(lexeme.clone())
                } else {
                    TokenKind::Ident
                };
                tokens.push(Token::new(kind, lexeme, Span::new(start_pos, end_pos)));
                pos = end_pos;
                continue;
            }
        }

        // Unknown character
        let bad_char = src[pos..].chars().next().unwrap_or('?');
        return Err(LexError {
            message: format!("unexpected character '{}'", bad_char),
            span: Some(Span::new(pos, pos + bad_char.len_utf8())),
        });
    }

    // EOF
    tokens.push(Token::new(TokenKind::Eof, String::new(), Span::new(pos, pos)));
    Ok(tokens)
}

fn lex_string(src: &str) -> Result<(String, usize), String> {
    if !src.starts_with(STRING_DELIMITER) {
        return Err("not a string literal".to_string());
    }
    let mut pos = STRING_DELIMITER.len();
    let mut result = String::new();
    result.push_str(STRING_DELIMITER);
    while pos < src.len() {
        if src[pos..].starts_with(STRING_DELIMITER) {
            result.push_str(STRING_DELIMITER);
            pos += STRING_DELIMITER.len();
            return Ok((result, pos));
        }
        if STRING_FORBID_NEWLINES {
            let ch = src[pos..].chars().next().unwrap();
            if ch == '\n' || ch == '\r' {
                return Err("unterminated string literal (newline not allowed)".to_string());
            }
        }
        let mut escape_matched = false;
        for esc in STRING_ESCAPES {
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
