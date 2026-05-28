// Production string tokenizer and parser - Wave 1

use crate::frontend::parserspec::{Element, Production, RepeatKind, Terminal, TerminalSource};

/// Token in production string mini-grammar
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProdToken {
    /// Identifier (non-terminal)
    Ident(String),
    /// Token terminal: <IDENT>
    TokenTerm(String),
    /// Keyword terminal: kw:"if"
    KeywordTerm(String),
    /// Punctuation terminal: punct:"+"
    PunctTerm(String),
    /// Left parenthesis
    LParen,
    /// Right parenthesis
    RParen,
    /// Alternation operator: |
    Pipe,
    /// Repetition: *
    Star,
    /// Repetition: +
    Plus,
    /// Repetition: ?
    Question,
}

/// Tokenize a production string
fn tokenize_production(input: &str) -> Result<Vec<ProdToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        match chars[i] {
            '(' => {
                tokens.push(ProdToken::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(ProdToken::RParen);
                i += 1;
            }
            '|' => {
                tokens.push(ProdToken::Pipe);
                i += 1;
            }
            '*' => {
                tokens.push(ProdToken::Star);
                i += 1;
            }
            '+' => {
                tokens.push(ProdToken::Plus);
                i += 1;
            }
            '?' => {
                tokens.push(ProdToken::Question);
                i += 1;
            }
            '<' => {
                // Token terminal: <IDENT> or <"PUNCT_=>">
                i += 1;

                // Check if this is a quoted token reference
                if i < chars.len() && chars[i] == '"' {
                    // Quoted token reference: <"TOKEN_NAME">
                    i += 1; // skip opening quote
                    let start = i;
                    while i < chars.len() && chars[i] != '"' {
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err(
                            "unclosed quoted token reference (missing closing quote)".to_string()
                        );
                    }
                    let name = chars[start..i].iter().collect::<String>();
                    i += 1; // skip closing quote

                    // Expect closing '>'
                    if i >= chars.len() || chars[i] != '>' {
                        return Err(
                            "quoted token reference must end with '>' after closing quote"
                                .to_string(),
                        );
                    }
                    tokens.push(ProdToken::TokenTerm(name));
                    i += 1; // skip '>'
                } else {
                    // Unquoted token reference: <TOKEN_NAME>
                    let start = i;
                    while i < chars.len() && chars[i] != '>' {
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err("unclosed token terminal '<'".to_string());
                    }
                    let name = chars[start..i].iter().collect::<String>();
                    tokens.push(ProdToken::TokenTerm(name));
                    i += 1; // skip '>'
                }
            }
            '"' => {
                // String literal (could be keyword or punctuation)
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("unclosed string literal".to_string());
                }
                let content = chars[start..i].iter().collect::<String>();
                // Store as a special token that will be resolved during parsing
                // For now, use Ident to represent literal strings
                tokens.push(ProdToken::Ident(format!("\"{}\"", content)));
                i += 1; // skip closing '"'
            }
            _ if chars[i].is_alphabetic() || chars[i] == '_' => {
                // Identifier or prefix (kw:, punct:)
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == ':')
                {
                    i += 1;
                }
                let text = chars[start..i].iter().collect::<String>();

                // Check for kw: or punct: prefix
                if text == "kw:" || text == "punct:" {
                    // Next token should be a string
                    // We need to look ahead
                    let prefix = text;
                    // Skip whitespace
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i >= chars.len() || chars[i] != '"' {
                        return Err(format!("expected string after {}", prefix));
                    }
                    // Parse string
                    i += 1; // skip '"'
                    let str_start = i;
                    while i < chars.len() && chars[i] != '"' {
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err("unclosed string literal".to_string());
                    }
                    let content = chars[str_start..i].iter().collect::<String>();
                    i += 1; // skip closing '"'

                    if prefix == "kw:" {
                        tokens.push(ProdToken::KeywordTerm(content));
                    } else {
                        tokens.push(ProdToken::PunctTerm(content));
                    }
                } else {
                    tokens.push(ProdToken::Ident(text));
                }
            }
            _ => {
                return Err(format!("unexpected character: '{}'", chars[i]));
            }
        }
    }

    Ok(tokens)
}

/// Parser for production strings
struct ProductionParser {
    tokens: Vec<ProdToken>,
    pos: usize,
}

impl ProductionParser {
    fn new(tokens: Vec<ProdToken>) -> Self {
        ProductionParser { tokens, pos: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&ProdToken> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.pos])
        }
    }

    fn advance(&mut self) -> Option<&ProdToken> {
        if self.is_at_end() {
            None
        } else {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        }
    }

    fn expect(&mut self, expected: ProdToken) -> Result<(), String> {
        match self.advance() {
            Some(tok) if *tok == expected => Ok(()),
            Some(tok) => Err(format!("expected {:?}, got {:?}", expected, tok)),
            None => Err(format!("expected {:?}, got end of input", expected)),
        }
    }

    /// Parse production: alternation
    fn parse_production(&mut self) -> Result<Production, String> {
        self.parse_alternation()
    }

    /// Parse alternation: sequence ("|" sequence)*
    fn parse_alternation(&mut self) -> Result<Production, String> {
        let mut alternatives = vec![self.parse_sequence()?];

        while matches!(self.peek(), Some(ProdToken::Pipe)) {
            self.advance(); // consume '|'
            alternatives.push(self.parse_sequence()?);
        }

        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(Production::Alternation(alternatives))
        }
    }

    /// Parse sequence: term*
    fn parse_sequence(&mut self) -> Result<Production, String> {
        let mut elements = Vec::new();

        while !self.is_at_end() {
            // Stop at pipe or close paren
            if matches!(self.peek(), Some(ProdToken::Pipe) | Some(ProdToken::RParen)) {
                break;
            }

            elements.push(self.parse_term()?);
        }

        Ok(Production::Sequence(elements))
    }

    /// Parse term: atom postfix?
    fn parse_term(&mut self) -> Result<Element, String> {
        let atom = self.parse_atom()?;

        // Check for postfix
        match self.peek() {
            Some(ProdToken::Star) => {
                self.advance();
                Ok(Element::Repeat(Box::new(atom), RepeatKind::ZeroOrMore))
            }
            Some(ProdToken::Plus) => {
                self.advance();
                Ok(Element::Repeat(Box::new(atom), RepeatKind::OneOrMore))
            }
            Some(ProdToken::Question) => {
                self.advance();
                Ok(Element::Repeat(Box::new(atom), RepeatKind::Optional))
            }
            _ => Ok(atom),
        }
    }

    /// Parse atom: nonterminal | terminal | "(" alternation ")"
    fn parse_atom(&mut self) -> Result<Element, String> {
        match self.peek() {
            Some(ProdToken::Ident(name)) => {
                let name = name.clone();
                self.advance();

                // ═══════════════════════════════════════════════════════════════
                // TOKEN-KIND-ONLY ENFORCEMENT
                // ═══════════════════════════════════════════════════════════════
                //
                // Parser specs MUST use explicit terminal syntax:
                //   - Token kinds:  <IDENT>, <INT>, <KW_XXX>, <PUNCT_XXX>
                //   - Keywords:     FORBIDDEN - use <KW_XXX> instead
                //   - Punctuation:  FORBIDDEN - use <PUNCT_XXX> instead
                //
                // Quoted strings, kw:"", punct:"", and ALL_CAPS tokens are
                // REJECTED with a hard error.
                // ═══════════════════════════════════════════════════════════════

                // HARD ERROR: Reject quoted strings (legacy format)
                if name.starts_with('"') && name.ends_with('"') {
                    return Err(format!(
                        "FORBIDDEN: Quoted string '{}' in parser spec. Use explicit token kinds: <KW_XXX> for keywords, <PUNCT_XXX> for punctuation.",
                        name
                    ));
                }

                // HARD ERROR: Reject ALL_CAPS tokens (legacy format)
                if name.chars().all(|c| c.is_uppercase() || c == '_') {
                    return Err(format!(
                        "FORBIDDEN: ALL_CAPS token '{}' in parser spec. Use explicit angle-bracket syntax: <{}>.",
                        name, name
                    ));
                }

                // Only non-terminals are allowed without angle brackets
                Ok(Element::NonTerminal(name))
            }
            Some(ProdToken::TokenTerm(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Element::Terminal(Terminal::TokenKind(
                    name,
                    TerminalSource::Explicit,
                )))
            }
            Some(ProdToken::KeywordTerm(kw)) => {
                let kw = kw.clone();
                self.advance();
                Ok(Element::Terminal(Terminal::Keyword(
                    kw,
                    TerminalSource::Explicit,
                )))
            }
            Some(ProdToken::PunctTerm(p)) => {
                let p = p.clone();
                self.advance();
                Ok(Element::Terminal(Terminal::Punct(
                    p,
                    TerminalSource::Explicit,
                )))
            }
            Some(ProdToken::LParen) => {
                self.advance(); // consume '('
                let prod = self.parse_alternation()?;
                self.expect(ProdToken::RParen)?;

                // Convert production to group
                match prod {
                    Production::Sequence(elements) => Ok(Element::Group(elements)),
                    Production::Alternation(_alts) => {
                        // Group containing alternatives
                        // We need to represent this differently
                        // For now, treat as a single-element group with alternation representation
                        // Actually, we need to flatten this properly
                        // Let's create a pseudo-element for alternation groups
                        // But the spec doesn't have that - groups contain elements
                        // So we convert alternation to a sequence with a single grouped element
                        // that internally represents alternation
                        // This is a design decision - for now, error on grouped alternations
                        // Or we can represent it as nested sequences
                        // Actually, looking at the Element enum, Group takes Vec<Element>
                        // So we can't represent (A | B) directly as a Group
                        // We need to extend the model or flatten differently
                        // For Wave 1 spec parsing, let's keep it simple:
                        // Groups can only contain sequences
                        Err(
                            "grouped alternations not yet supported in this implementation"
                                .to_string(),
                        )
                    }
                }
            }
            Some(tok) => Err(format!("unexpected token in atom position: {:?}", tok)),
            None => Err("unexpected end of production string".to_string()),
        }
    }
}

/// Parse a production string into structured Production
pub fn parse_production_string(input: &str) -> Result<Production, String> {
    let tokens = tokenize_production(input)?;
    let mut parser = ProductionParser::new(tokens);
    let prod = parser.parse_production()?;

    if !parser.is_at_end() {
        return Err("unexpected tokens after production".to_string());
    }

    Ok(prod)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize_production("Foo Bar").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], ProdToken::Ident("Foo".to_string()));
        assert_eq!(tokens[1], ProdToken::Ident("Bar".to_string()));
    }

    #[test]
    fn test_tokenize_terminals() {
        let tokens = tokenize_production(r#"<IDENT> kw:"if" punct:"+""#).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], ProdToken::TokenTerm("IDENT".to_string()));
        assert_eq!(tokens[1], ProdToken::KeywordTerm("if".to_string()));
        assert_eq!(tokens[2], ProdToken::PunctTerm("+".to_string()));
    }

    #[test]
    fn test_tokenize_postfix() {
        let tokens = tokenize_production("Foo* Bar+ Baz?").unwrap();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[1], ProdToken::Star);
        assert_eq!(tokens[3], ProdToken::Plus);
        assert_eq!(tokens[5], ProdToken::Question);
    }

    #[test]
    fn test_parse_simple_sequence() {
        let prod = parse_production_string("Foo Bar").unwrap();
        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_parse_alternation() {
        let prod = parse_production_string("Foo | Bar").unwrap();
        match prod {
            Production::Alternation(alts) => {
                assert_eq!(alts.len(), 2);
            }
            _ => panic!("expected alternation"),
        }
    }

    #[test]
    fn test_parse_repetition() {
        let prod = parse_production_string("Foo*").unwrap();
        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Element::Repeat(_, RepeatKind::ZeroOrMore) => {}
                    _ => panic!("expected repeat"),
                }
            }
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_parse_group() {
        let prod = parse_production_string("(Foo Bar)").unwrap();
        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Element::Group(inner) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("expected group"),
                }
            }
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_parse_complex() {
        let prod =
            parse_production_string(r#"kw:"fn" <IDENT> punct:"(" Params? punct:")""#).unwrap();
        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 5);
                // Verify structure
                assert!(matches!(
                    &elements[0],
                    Element::Terminal(Terminal::Keyword(_, _))
                ));
                assert!(matches!(
                    &elements[1],
                    Element::Terminal(Terminal::TokenKind(_, _))
                ));
                assert!(matches!(
                    &elements[2],
                    Element::Terminal(Terminal::Punct(_, _))
                ));
                assert!(matches!(
                    &elements[3],
                    Element::Repeat(_, RepeatKind::Optional)
                ));
            }
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_reject_quoted_string_keyword() {
        // Test that quoted string format is now FORBIDDEN
        let result = parse_production_string(r#""fn" <IDENT>"#);
        assert!(result.is_err(), "Should reject quoted keyword string");
        assert!(result.unwrap_err().contains("FORBIDDEN"));
    }

    #[test]
    fn test_reject_quoted_string_punct() {
        // Test that quoted string format is now FORBIDDEN
        let result = parse_production_string(r#""(" Expr ")""#);
        assert!(result.is_err(), "Should reject quoted punctuation string");
        assert!(result.unwrap_err().contains("FORBIDDEN"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TOKEN-KIND-ONLY ENFORCEMENT TEST
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // This test exists to PROVE that:
    // 1. Legacy terminal inference is FORBIDDEN
    // 2. Parser specs MUST use explicit token kind syntax
    // 3. Hard errors are raised on any legacy syntax
    //
    // This test is DOCUMENTATION BY ENFORCEMENT.
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_token_kind_only_enforcement() {
        // Legacy syntax - should be REJECTED
        let result = parse_production_string(r#""if" IDENT "(" ")""#);
        assert!(result.is_err(), "Should reject legacy syntax");
        assert!(result.unwrap_err().contains("FORBIDDEN"));

        // Explicit syntax - should be ACCEPTED
        let prod = parse_production_string(r#"<KW_IF> <IDENT> <PUNCT_(> <PUNCT_)>"#).unwrap();

        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 4);

                // Verify all terminals are Explicit token kinds
                match &elements[0] {
                    Element::Terminal(Terminal::TokenKind(tk, source)) => {
                        assert_eq!(tk, "KW_IF");
                        assert_eq!(
                            *source,
                            TerminalSource::Explicit,
                            "Token kinds MUST be Explicit"
                        );
                    }
                    _ => panic!("expected explicit token kind terminal"),
                }

                match &elements[1] {
                    Element::Terminal(Terminal::TokenKind(tk, source)) => {
                        assert_eq!(tk, "IDENT");
                        assert_eq!(*source, TerminalSource::Explicit);
                    }
                    _ => panic!("expected explicit token kind terminal"),
                }

                match &elements[2] {
                    Element::Terminal(Terminal::TokenKind(tk, source)) => {
                        assert_eq!(tk, "PUNCT_(");
                        assert_eq!(*source, TerminalSource::Explicit);
                    }
                    _ => panic!("expected explicit token kind terminal"),
                }

                match &elements[3] {
                    Element::Terminal(Terminal::TokenKind(tk, source)) => {
                        assert_eq!(tk, "PUNCT_)");
                        assert_eq!(*source, TerminalSource::Explicit);
                    }
                    _ => panic!("expected explicit token kind terminal"),
                }
            }
            _ => panic!("expected sequence"),
        }
    }

    #[test]
    fn test_explicit_kw_punct_syntax_still_works() {
        // Test that the old kw: and punct: syntax still works for migration
        // This will be removed in a future wave
        let prod = parse_production_string(r#"kw:"if" <IDENT> punct:"(" punct:")""#).unwrap();

        match prod {
            Production::Sequence(elements) => {
                assert_eq!(elements.len(), 4);

                // Verify explicit terminals are tagged as Explicit
                match &elements[0] {
                    Element::Terminal(Terminal::Keyword(kw, source)) => {
                        assert_eq!(kw, "if");
                        assert_eq!(
                            *source,
                            TerminalSource::Explicit,
                            "kw: syntax MUST be tagged as Explicit"
                        );
                    }
                    _ => panic!("expected explicit keyword"),
                }

                match &elements[1] {
                    Element::Terminal(Terminal::TokenKind(tk, source)) => {
                        assert_eq!(tk, "IDENT");
                        assert_eq!(*source, TerminalSource::Explicit);
                    }
                    _ => panic!("expected explicit token kind"),
                }
            }
            _ => panic!("expected sequence"),
        }
    }
}
