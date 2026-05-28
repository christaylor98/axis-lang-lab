// Lexer spec codegen: YAML → static Rust lexer code
// Generated code must be proven equivalent to runtime spec via tests

use crate::frontend::lexspec_load::load_spec;
use std::fmt::Write;
use std::path::Path;

pub fn generate_lexer_code(spec_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let spec = load_spec(spec_path)?;
    let cfg = &spec.lexer;

    let mut code = String::new();

    // Header
    writeln!(code, "// GENERATED CODE - DO NOT EDIT")?;
    writeln!(code, "// Generated from: {}", spec_path.display())?;
    writeln!(code, "// Source of truth: YAML spec file")?;
    writeln!(
        code,
        "// This code must pass equivalence tests against runtime lexer"
    )?;
    writeln!(code)?;
    writeln!(
        code,
        "use crate::frontend::token::{{Token, TokenKind, Span}};"
    )?;
    writeln!(code, "use regex::Regex;")?;
    writeln!(code, "use std::fmt;")?;
    writeln!(code)?;

    // Error type
    writeln!(code, "#[derive(Debug)]")?;
    writeln!(code, "pub struct LexError {{")?;
    writeln!(code, "    pub message: String,")?;
    writeln!(code, "    pub span: Option<Span>,")?;
    writeln!(code, "}}")?;
    writeln!(code)?;
    writeln!(code, "impl fmt::Display for LexError {{")?;
    writeln!(
        code,
        "    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{"
    )?;
    writeln!(code, "        if let Some(s) = &self.span {{")?;
    writeln!(
        code,
        "            write!(f, \"lex error at {{}}..{{}}: {{}}\", s.start, s.end, self.message)"
    )?;
    writeln!(code, "        }} else {{")?;
    writeln!(
        code,
        "            write!(f, \"lex error: {{}}\", self.message)"
    )?;
    writeln!(code, "        }}")?;
    writeln!(code, "    }}")?;
    writeln!(code, "}}")?;
    writeln!(code)?;
    writeln!(code, "impl std::error::Error for LexError {{}}")?;
    writeln!(code)?;

    // Lazy static regexes
    writeln!(code, "use once_cell::sync::Lazy;")?;
    writeln!(code)?;

    if cfg.whitespace.is_some() {
        let ws = cfg.whitespace.as_ref().unwrap();
        writeln!(code, "static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| {{")?;
        writeln!(
            code,
            "    Regex::new(r###\"{}\"###).unwrap()",
            escape_pattern(&ws.pattern)
        )?;
        writeln!(code, "}});")?;
        writeln!(code)?;
    }

    if !cfg.comments.is_empty() {
        writeln!(
            code,
            "static COMMENT_RES: Lazy<Vec<Regex>> = Lazy::new(|| {{"
        )?;
        writeln!(code, "    vec![")?;
        for comment in &cfg.comments {
            writeln!(
                code,
                "        Regex::new(r###\"{}\"###).unwrap(),",
                escape_pattern(&comment.pattern)
            )?;
        }
        writeln!(code, "    ]")?;
        writeln!(code, "}});")?;
        writeln!(code)?;
    }

    if cfg.identifiers.is_some() {
        let ident = cfg.identifiers.as_ref().unwrap();
        writeln!(code, "static IDENT_RE: Lazy<Regex> = Lazy::new(|| {{")?;
        writeln!(
            code,
            "    Regex::new(r###\"{}\"###).unwrap()",
            escape_pattern(&ident.pattern)
        )?;
        writeln!(code, "}});")?;
        writeln!(code)?;
    }

    if let Some(lits) = &cfg.literals {
        if let Some(int_lit) = &lits.int {
            writeln!(code, "static INT_RE: Lazy<Regex> = Lazy::new(|| {{")?;
            writeln!(
                code,
                "    Regex::new(r###\"{}\"###).unwrap()",
                escape_pattern(&int_lit.pattern)
            )?;
            writeln!(code, "}});")?;
            writeln!(code)?;
        }
    }

    // Constants
    writeln!(code, "const KEYWORDS: &[&str] = &[")?;
    for kw in &cfg.keywords {
        writeln!(code, "    \"{}\",", kw)?;
    }
    writeln!(code, "];")?;
    writeln!(code)?;

    if let Some(lits) = &cfg.literals {
        if let Some(bool_lit) = &lits.bool {
            writeln!(code, "const BOOL_VALUES: &[&str] = &[")?;
            for val in &bool_lit.values {
                writeln!(code, "    \"{}\",", val)?;
            }
            writeln!(code, "];")?;
            writeln!(code)?;
        }

        if let Some(unit_lit) = &lits.unit {
            writeln!(code, "const UNIT_LITERAL: &str = \"{}\";", unit_lit.literal)?;
            writeln!(code)?;
        }

        if let Some(str_lit) = &lits.string {
            writeln!(
                code,
                "const STRING_DELIMITER: &str = \"{}\";",
                escape_string(&str_lit.delimiter)
            )?;
            writeln!(code, "const STRING_ESCAPES: &[&str] = &[")?;
            for esc in &str_lit.escapes {
                writeln!(code, "    \"{}\",", escape_string(esc))?;
            }
            writeln!(code, "];")?;
            writeln!(
                code,
                "const STRING_FORBID_NEWLINES: bool = {};",
                str_lit.forbid_newlines
            )?;
            writeln!(code)?;
        }
    }

    // Punctuation (sorted longest-first)
    let mut punct = cfg.punctuation.clone();
    punct.sort_by(|a, b| b.len().cmp(&a.len()));
    writeln!(code, "const PUNCTUATION: &[&str] = &[")?;
    for p in &punct {
        writeln!(code, "    \"{}\",", escape_string(p))?;
    }
    writeln!(code, "];")?;
    writeln!(code)?;

    // Main lex function
    writeln!(
        code,
        "/// Generated lexer - must be equivalent to runtime spec"
    )?;
    writeln!(
        code,
        "pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {{"
    )?;
    writeln!(code, "    let mut tokens = Vec::new();")?;
    writeln!(code, "    let mut pos = 0;")?;
    writeln!(code, "    let bytes = src.as_bytes();")?;
    writeln!(code)?;
    writeln!(code, "    while pos < bytes.len() {{")?;
    writeln!(code, "        let start_pos = pos;")?;
    writeln!(code)?;

    // Whitespace
    if cfg.whitespace.is_some() {
        writeln!(code, "        // Whitespace")?;
        writeln!(
            code,
            "        if let Some(m) = WHITESPACE_RE.find(&src[pos..]) {{"
        )?;
        writeln!(code, "            if m.start() == 0 {{")?;
        writeln!(code, "                pos += m.end();")?;
        writeln!(code, "                continue;")?;
        writeln!(code, "            }}")?;
        writeln!(code, "        }}")?;
        writeln!(code)?;
    }

    // Comments
    if !cfg.comments.is_empty() {
        writeln!(code, "        // Comments")?;
        writeln!(code, "        let mut comment_matched = false;")?;
        writeln!(code, "        for comment_re in COMMENT_RES.iter() {{")?;
        writeln!(
            code,
            "            if let Some(m) = comment_re.find(&src[pos..]) {{"
        )?;
        writeln!(code, "                if m.start() == 0 {{")?;
        writeln!(code, "                    pos += m.end();")?;
        writeln!(code, "                    comment_matched = true;")?;
        writeln!(code, "                    break;")?;
        writeln!(code, "                }}")?;
        writeln!(code, "            }}")?;
        writeln!(code, "        }}")?;
        writeln!(code, "        if comment_matched {{ continue; }}")?;
        writeln!(code)?;
    }

    // Unit literal
    if let Some(lits) = &cfg.literals {
        if lits.unit.is_some() {
            writeln!(code, "        // Unit literal")?;
            writeln!(code, "        if src[pos..].starts_with(UNIT_LITERAL) {{")?;
            writeln!(code, "            let end_pos = pos + UNIT_LITERAL.len();")?;
            writeln!(code, "            tokens.push(Token::new(")?;
            writeln!(code, "                TokenKind::UnitLit,")?;
            writeln!(code, "                UNIT_LITERAL.to_string(),")?;
            writeln!(code, "                Span::new(start_pos, end_pos),")?;
            writeln!(code, "            ));")?;
            writeln!(code, "            pos = end_pos;")?;
            writeln!(code, "            continue;")?;
            writeln!(code, "        }}")?;
            writeln!(code)?;
        }
    }

    // Punctuation
    writeln!(code, "        // Punctuation")?;
    writeln!(code, "        let mut punct_matched = false;")?;
    writeln!(code, "        for punct in PUNCTUATION {{")?;
    writeln!(code, "            if src[pos..].starts_with(punct) {{")?;
    writeln!(code, "                let end_pos = pos + punct.len();")?;
    writeln!(code, "                tokens.push(Token::new(")?;
    writeln!(
        code,
        "                    TokenKind::Punct(punct.to_string()),"
    )?;
    writeln!(code, "                    punct.to_string(),")?;
    writeln!(code, "                    Span::new(start_pos, end_pos),")?;
    writeln!(code, "                ));")?;
    writeln!(code, "                pos = end_pos;")?;
    writeln!(code, "                punct_matched = true;")?;
    writeln!(code, "                break;")?;
    writeln!(code, "            }}")?;
    writeln!(code, "        }}")?;
    writeln!(code, "        if punct_matched {{ continue; }}")?;
    writeln!(code)?;

    // String literals
    if let Some(lits) = &cfg.literals {
        if lits.string.is_some() {
            writeln!(code, "        // String literal")?;
            writeln!(
                code,
                "        if src[pos..].starts_with(STRING_DELIMITER) {{"
            )?;
            writeln!(code, "            match lex_string(&src[pos..]) {{")?;
            writeln!(code, "                Ok((lexeme, consumed)) => {{")?;
            writeln!(code, "                    let end_pos = pos + consumed;")?;
            writeln!(code, "                    tokens.push(Token::new(")?;
            writeln!(code, "                        TokenKind::StringLit,")?;
            writeln!(code, "                        lexeme,")?;
            writeln!(
                code,
                "                        Span::new(start_pos, end_pos),"
            )?;
            writeln!(code, "                    ));")?;
            writeln!(code, "                    pos = end_pos;")?;
            writeln!(code, "                    continue;")?;
            writeln!(code, "                }}")?;
            writeln!(code, "                Err(msg) => {{")?;
            writeln!(code, "                    return Err(LexError {{")?;
            writeln!(code, "                        message: msg,")?;
            writeln!(
                code,
                "                        span: Some(Span::new(start_pos, pos + 1)),"
            )?;
            writeln!(code, "                    }});")?;
            writeln!(code, "                }}")?;
            writeln!(code, "            }}")?;
            writeln!(code, "        }}")?;
            writeln!(code)?;
        }
    }

    // Int literal
    if let Some(lits) = &cfg.literals {
        if lits.int.is_some() {
            writeln!(code, "        // Int literal")?;
            writeln!(code, "        if let Some(m) = INT_RE.find(&src[pos..]) {{")?;
            writeln!(code, "            if m.start() == 0 {{")?;
            writeln!(code, "                let lexeme = m.as_str().to_string();")?;
            writeln!(code, "                let end_pos = pos + m.end();")?;
            writeln!(code, "                tokens.push(Token::new(")?;
            writeln!(code, "                    TokenKind::IntLit,")?;
            writeln!(code, "                    lexeme,")?;
            writeln!(code, "                    Span::new(start_pos, end_pos),")?;
            writeln!(code, "                ));")?;
            writeln!(code, "                pos = end_pos;")?;
            writeln!(code, "                continue;")?;
            writeln!(code, "            }}")?;
            writeln!(code, "        }}")?;
            writeln!(code)?;
        }
    }

    // Bool literals
    if let Some(lits) = &cfg.literals {
        if lits.bool.is_some() {
            writeln!(code, "        // Bool literal")?;
            writeln!(code, "        let mut bool_matched = false;")?;
            writeln!(code, "        for bool_val in BOOL_VALUES {{")?;
            writeln!(code, "            if src[pos..].starts_with(bool_val) {{")?;
            writeln!(code, "                let end_idx = pos + bool_val.len();")?;
            writeln!(
                code,
                "                let at_boundary = end_idx >= src.len() || {{"
            )?;
            writeln!(
                code,
                "                    let next_char = src[end_idx..].chars().next().unwrap();"
            )?;
            writeln!(
                code,
                "                    !next_char.is_alphanumeric() && next_char != '_'"
            )?;
            writeln!(code, "                }};")?;
            writeln!(code, "                if at_boundary {{")?;
            writeln!(code, "                    tokens.push(Token::new(")?;
            writeln!(code, "                        TokenKind::BoolLit,")?;
            writeln!(code, "                        bool_val.to_string(),")?;
            writeln!(
                code,
                "                        Span::new(start_pos, end_idx),"
            )?;
            writeln!(code, "                    ));")?;
            writeln!(code, "                    pos = end_idx;")?;
            writeln!(code, "                    bool_matched = true;")?;
            writeln!(code, "                    break;")?;
            writeln!(code, "                }}")?;
            writeln!(code, "            }}")?;
            writeln!(code, "        }}")?;
            writeln!(code, "        if bool_matched {{ continue; }}")?;
            writeln!(code)?;
        }
    }

    // Identifier/keyword
    if cfg.identifiers.is_some() {
        writeln!(code, "        // Identifier or keyword")?;
        writeln!(
            code,
            "        if let Some(m) = IDENT_RE.find(&src[pos..]) {{"
        )?;
        writeln!(code, "            if m.start() == 0 {{")?;
        writeln!(code, "                let lexeme = m.as_str().to_string();")?;
        writeln!(code, "                let end_pos = pos + m.end();")?;
        writeln!(
            code,
            "                let kind = if KEYWORDS.contains(&lexeme.as_str()) {{"
        )?;
        writeln!(
            code,
            "                    TokenKind::Keyword(lexeme.clone())"
        )?;
        writeln!(code, "                }} else {{")?;
        writeln!(code, "                    TokenKind::Ident")?;
        writeln!(code, "                }};")?;
        writeln!(
            code,
            "                tokens.push(Token::new(kind, lexeme, Span::new(start_pos, end_pos)));"
        )?;
        writeln!(code, "                pos = end_pos;")?;
        writeln!(code, "                continue;")?;
        writeln!(code, "            }}")?;
        writeln!(code, "        }}")?;
        writeln!(code)?;
    }

    // Error
    writeln!(code, "        // Unknown character")?;
    writeln!(
        code,
        "        let bad_char = src[pos..].chars().next().unwrap_or('?');"
    )?;
    writeln!(code, "        return Err(LexError {{")?;
    writeln!(
        code,
        "            message: format!(\"unexpected character '{{}}'\", bad_char),"
    )?;
    writeln!(
        code,
        "            span: Some(Span::new(pos, pos + bad_char.len_utf8())),"
    )?;
    writeln!(code, "        }});")?;
    writeln!(code, "    }}")?;
    writeln!(code)?;
    writeln!(code, "    // EOF")?;
    writeln!(
        code,
        "    tokens.push(Token::new(TokenKind::Eof, String::new(), Span::new(pos, pos)));"
    )?;
    writeln!(code, "    Ok(tokens)")?;
    writeln!(code, "}}")?;
    writeln!(code)?;

    // String helper (if needed)
    if let Some(lits) = &cfg.literals {
        if lits.string.is_some() {
            writeln!(
                code,
                "fn lex_string(src: &str) -> Result<(String, usize), String> {{"
            )?;
            writeln!(code, "    if !src.starts_with(STRING_DELIMITER) {{")?;
            writeln!(
                code,
                "        return Err(\"not a string literal\".to_string());"
            )?;
            writeln!(code, "    }}")?;
            writeln!(code, "    let mut pos = STRING_DELIMITER.len();")?;
            writeln!(code, "    let mut result = String::new();")?;
            writeln!(code, "    result.push_str(STRING_DELIMITER);")?;
            writeln!(code, "    while pos < src.len() {{")?;
            writeln!(
                code,
                "        if src[pos..].starts_with(STRING_DELIMITER) {{"
            )?;
            writeln!(code, "            result.push_str(STRING_DELIMITER);")?;
            writeln!(code, "            pos += STRING_DELIMITER.len();")?;
            writeln!(code, "            return Ok((result, pos));")?;
            writeln!(code, "        }}")?;
            writeln!(code, "        if STRING_FORBID_NEWLINES {{")?;
            writeln!(
                code,
                "            let ch = src[pos..].chars().next().unwrap();"
            )?;
            writeln!(code, "            if ch == '\\n' || ch == '\\r' {{")?;
            writeln!(code, "                return Err(\"unterminated string literal (newline not allowed)\".to_string());")?;
            writeln!(code, "            }}")?;
            writeln!(code, "        }}")?;
            writeln!(code, "        let mut escape_matched = false;")?;
            writeln!(code, "        for esc in STRING_ESCAPES {{")?;
            writeln!(code, "            if src[pos..].starts_with(esc) {{")?;
            writeln!(code, "                result.push_str(esc);")?;
            writeln!(code, "                pos += esc.len();")?;
            writeln!(code, "                escape_matched = true;")?;
            writeln!(code, "                break;")?;
            writeln!(code, "            }}")?;
            writeln!(code, "        }}")?;
            writeln!(code, "        if !escape_matched {{")?;
            writeln!(
                code,
                "            let ch = src[pos..].chars().next().unwrap();"
            )?;
            writeln!(code, "            result.push(ch);")?;
            writeln!(code, "            pos += ch.len_utf8();")?;
            writeln!(code, "        }}")?;
            writeln!(code, "    }}")?;
            writeln!(code, "    Err(\"unterminated string literal\".to_string())")?;
            writeln!(code, "}}")?;
        }
    }

    Ok(code)
}

fn escape_pattern(s: &str) -> String {
    // Patterns are already regex strings, just need to avoid breaking raw string delimiters
    s.replace("###", "##\\##")
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn codegen_produces_valid_rust() {
        let spec_path = PathBuf::from("lang-lab-poc-userfiles/lexer.yaml");
        let code = generate_lexer_code(&spec_path).expect("should generate code");

        // Basic syntax checks
        assert!(code.contains("pub fn lex(src: &str)"));
        assert!(code.contains("KEYWORDS"));
        assert!(code.contains("GENERATED CODE"));
    }
}
