// Wave 2: Runtime Parser Engine
// Executes validated ParserSpec mechanically against token stream
// Produces structural parse trees (NO semantic interpretation)

use crate::frontend::parserspec::{
    Element, ParserSpec, Production, RepeatKind, Terminal, TerminalSource,
};
use crate::frontend::token::{Span, Token, TokenKind};
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Structural parse tree (NOT an AST)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTree {
    pub rule: String,
    pub children: Vec<ParseNode>,
    pub span: Span,
}

/// Node in the parse tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNode {
    /// Parsed non-terminal rule
    Rule(ParseTree),
    /// Terminal token
    Terminal(Token),
}

/// Parse error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Runtime parser entry point
///
/// Parses tokens using the given spec, producing a structural parse tree.
///
/// DOES NOT:
/// - Infer terminals
/// - Fix grammar bugs
/// - Add semantic meaning
/// - Normalize or simplify trees
pub fn parse_with_spec(spec: &ParserSpec, tokens: &[Token]) -> Result<ParseTree, ParseError> {
    let mut state = ParserState::new(tokens);

    // Parse starting at the start rule
    let tree = parse_rule(&mut state, spec, &spec.start)?;

    // CRITICAL: Ensure we consumed all tokens except EOF
    if !state.at_eof() {
        return Err(ParseError {
            message: "unexpected token after end of parse".to_string(),
            span: state.current_span(),
        });
    }

    Ok(tree)
}

// ═══════════════════════════════════════════════════════════════════════════
// PARSER STATE
// ═══════════════════════════════════════════════════════════════════════════

struct ParserState<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ParserState<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        ParserState { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_span(&self) -> Span {
        if let Some(tok) = self.current() {
            tok.span.clone()
        } else {
            // At EOF - use end of last token or 0
            if let Some(last) = self.tokens.last() {
                Span::new(last.span.end, last.span.end)
            } else {
                Span::new(0, 0)
            }
        }
    }

    fn at_eof(&self) -> bool {
        match self.current() {
            Some(tok) => matches!(tok.kind, TokenKind::Eof),
            None => true,
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn checkpoint(&self) -> usize {
        self.pos
    }

    fn restore(&mut self, checkpoint: usize) {
        self.pos = checkpoint;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CORE PARSER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Parse a non-terminal rule
fn parse_rule(
    state: &mut ParserState,
    spec: &ParserSpec,
    rule: &str,
) -> Result<ParseTree, ParseError> {
    let start_span = state.current_span();

    // Lookup productions for this rule
    let productions = spec.grammar.get(rule).ok_or_else(|| ParseError {
        message: format!("undefined nonterminal: {}", rule),
        span: start_span.clone(),
    })?;

    // Try productions in order (backtracking across alternatives)
    let mut last_error: Option<ParseError> = None;

    for production in productions {
        let checkpoint = state.checkpoint();

        match parse_production(state, spec, production) {
            Ok(children) => {
                let end_span = if !children.is_empty() {
                    match children.last().unwrap() {
                        ParseNode::Rule(t) => t.span.clone(),
                        ParseNode::Terminal(tok) => tok.span.clone(),
                    }
                } else {
                    start_span.clone()
                };

                let span = Span::new(start_span.start, end_span.end);

                return Ok(ParseTree {
                    rule: rule.to_string(),
                    children,
                    span,
                });
            }
            Err(e) => {
                // Backtrack and try next alternative
                state.restore(checkpoint);
                last_error = Some(e);
            }
        }
    }

    // All alternatives failed
    Err(last_error.unwrap_or_else(|| ParseError {
        message: format!("no production matched for rule {}", rule),
        span: start_span,
    }))
}

/// Parse a production (sequence or alternation)
fn parse_production(
    state: &mut ParserState,
    spec: &ParserSpec,
    production: &Production,
) -> Result<Vec<ParseNode>, ParseError> {
    match production {
        Production::Sequence(elements) => {
            let mut children = Vec::new();
            for element in elements {
                let mut nodes = parse_element(state, spec, element)?;
                children.append(&mut nodes);
            }
            Ok(children)
        }

        Production::Alternation(alternatives) => {
            // Try each alternative in order
            let mut last_error: Option<ParseError> = None;

            for alt in alternatives {
                let checkpoint = state.checkpoint();

                match parse_production(state, spec, alt) {
                    Ok(children) => return Ok(children),
                    Err(e) => {
                        state.restore(checkpoint);
                        last_error = Some(e);
                    }
                }
            }

            Err(last_error.unwrap_or_else(|| ParseError {
                message: "no alternative matched".to_string(),
                span: state.current_span(),
            }))
        }
    }
}

/// Parse an element (NonTerminal, Terminal, Repeat, Group)
fn parse_element(
    state: &mut ParserState,
    spec: &ParserSpec,
    element: &Element,
) -> Result<Vec<ParseNode>, ParseError> {
    match element {
        Element::NonTerminal(name) => {
            let tree = parse_rule(state, spec, name)?;
            Ok(vec![ParseNode::Rule(tree)])
        }

        Element::Terminal(terminal) => {
            let tok = parse_terminal(state, terminal)?;
            Ok(vec![ParseNode::Terminal(tok)])
        }

        Element::Repeat(inner, kind) => parse_repeat(state, spec, inner, kind),

        Element::Group(elements) => {
            // Inline parse of grouped elements
            let mut children = Vec::new();
            for element in elements {
                let mut nodes = parse_element(state, spec, element)?;
                children.append(&mut nodes);
            }
            Ok(children)
        }
    }
}

/// Parse a terminal token
fn parse_terminal(state: &mut ParserState, terminal: &Terminal) -> Result<Token, ParseError> {
    // CRITICAL: Reject legacy inferred terminals
    let source = match terminal {
        Terminal::TokenKind(_, src) => src,
        Terminal::Keyword(_, src) => src,
        Terminal::Punct(_, src) => src,
    };

    if matches!(source, TerminalSource::InferredLegacy) {
        return Err(ParseError {
            message: format!(
                "terminal {:?} marked InferredLegacy is forbidden at runtime",
                terminal
            ),
            span: state.current_span(),
        });
    }

    let current = state.current().ok_or_else(|| ParseError {
        message: format!("expected {:?}, found EOF", terminal),
        span: state.current_span(),
    })?;

    // Match terminal against token
    let matches = match terminal {
        Terminal::TokenKind(expected_kind, _) => {
            // Normalize token kind names to support both forms:
            // YAML uses INT, BOOL, STRING, UNIT
            // TokenKind uses IntLit, BoolLit, StringLit, UnitLit
            // Accept both "INT" and "INT_LIT", etc.
            match &current.kind {
                TokenKind::Ident if expected_kind == "IDENT" => true,
                TokenKind::IntLit if expected_kind == "INT_LIT" || expected_kind == "INT" => true,
                TokenKind::BoolLit if expected_kind == "BOOL_LIT" || expected_kind == "BOOL" => {
                    true
                }
                TokenKind::StringLit
                    if expected_kind == "STRING_LIT" || expected_kind == "STRING" =>
                {
                    true
                }
                TokenKind::UnitLit if expected_kind == "UNIT_LIT" || expected_kind == "UNIT" => {
                    true
                }
                TokenKind::Keyword(kw) if expected_kind == &format!("KW_{}", kw.to_uppercase()) => {
                    true
                }
                TokenKind::Punct(p) if expected_kind == &format!("PUNCT_{}", p) => true,
                _ => false,
            }
        }

        Terminal::Keyword(expected_kw, _) => {
            matches!(&current.kind, TokenKind::Keyword(kw) if kw == expected_kw)
        }

        Terminal::Punct(expected_p, _) => {
            matches!(&current.kind, TokenKind::Punct(p) if p == expected_p)
        }
    };

    if !matches {
        return Err(ParseError {
            message: format!("expected {:?}, found {}", terminal, current.kind),
            span: current.span.clone(),
        });
    }

    let tok = current.clone();
    state.advance();
    Ok(tok)
}

/// Parse repeated element (* + ?)
fn parse_repeat(
    state: &mut ParserState,
    spec: &ParserSpec,
    element: &Element,
    kind: &RepeatKind,
) -> Result<Vec<ParseNode>, ParseError> {
    let mut children = Vec::new();

    match kind {
        RepeatKind::ZeroOrMore => {
            // Loop until failure
            loop {
                let checkpoint = state.checkpoint();
                match parse_element(state, spec, element) {
                    Ok(mut nodes) => {
                        children.append(&mut nodes);
                    }
                    Err(_) => {
                        // Backtrack and stop
                        state.restore(checkpoint);
                        break;
                    }
                }
            }
            Ok(children)
        }

        RepeatKind::OneOrMore => {
            // First occurrence is required
            let mut nodes = parse_element(state, spec, element)?;
            children.append(&mut nodes);

            // Then zero or more additional
            loop {
                let checkpoint = state.checkpoint();
                match parse_element(state, spec, element) {
                    Ok(mut nodes) => {
                        children.append(&mut nodes);
                    }
                    Err(_) => {
                        state.restore(checkpoint);
                        break;
                    }
                }
            }
            Ok(children)
        }

        RepeatKind::Optional => {
            // Try once, skip on failure
            let checkpoint = state.checkpoint();
            match parse_element(state, spec, element) {
                Ok(mut nodes) => {
                    children.append(&mut nodes);
                }
                Err(_) => {
                    state.restore(checkpoint);
                }
            }
            Ok(children)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parserspec::Production;
    use std::collections::HashMap;

    fn make_token(kind: TokenKind, lexeme: &str, start: usize) -> Token {
        Token::new(
            kind,
            lexeme.to_string(),
            Span::new(start, start + lexeme.len()),
        )
    }

    fn make_eof() -> Token {
        Token::new(TokenKind::Eof, "".to_string(), Span::new(0, 0))
    }

    #[test]
    fn test_simple_terminal_match() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Terminal(
                Terminal::Keyword("fn".to_string(), TerminalSource::Explicit),
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_eof(),
        ];

        let result = parse_with_spec(&spec, &tokens);
        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let tree = result.unwrap();
        assert_eq!(tree.rule, "start");
        assert_eq!(tree.children.len(), 1);
    }

    #[test]
    fn test_terminal_mismatch() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Terminal(
                Terminal::Keyword("if".to_string(), TerminalSource::Explicit),
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_eof(),
        ];

        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("expected"));
    }

    #[test]
    fn test_legacy_terminal_rejection() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Terminal(
                Terminal::Keyword("fn".to_string(), TerminalSource::InferredLegacy),
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_eof(),
        ];

        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("InferredLegacy"));
        assert!(err.message.contains("forbidden"));
    }

    #[test]
    fn test_eof_enforcement() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Terminal(
                Terminal::Keyword("fn".to_string(), TerminalSource::Explicit),
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_token(TokenKind::Keyword("if".to_string()), "if", 3),
            make_eof(),
        ];

        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unexpected token after end of parse"));
    }

    #[test]
    fn test_repeat_zero_or_more() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Repeat(
                Box::new(Element::Terminal(Terminal::TokenKind(
                    "IDENT".to_string(),
                    TerminalSource::Explicit,
                ))),
                RepeatKind::ZeroOrMore,
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        // Zero occurrences
        let tokens = vec![make_eof()];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.children.len(), 0);

        // Multiple occurrences
        let tokens = vec![
            make_token(TokenKind::Ident, "x", 0),
            make_token(TokenKind::Ident, "y", 2),
            make_eof(),
        ];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn test_repeat_one_or_more() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Repeat(
                Box::new(Element::Terminal(Terminal::TokenKind(
                    "IDENT".to_string(),
                    TerminalSource::Explicit,
                ))),
                RepeatKind::OneOrMore,
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        // Zero occurrences - should fail
        let tokens = vec![make_eof()];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_err());

        // One occurrence - should succeed
        let tokens = vec![make_token(TokenKind::Ident, "x", 0), make_eof()];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.children.len(), 1);
    }

    #[test]
    fn test_optional_element() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![Element::Repeat(
                Box::new(Element::Terminal(Terminal::TokenKind(
                    "IDENT".to_string(),
                    TerminalSource::Explicit,
                ))),
                RepeatKind::Optional,
            )])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        // Present
        let tokens = vec![make_token(TokenKind::Ident, "x", 0), make_eof()];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().children.len(), 1);

        // Absent
        let tokens = vec![make_eof()];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().children.len(), 0);
    }

    #[test]
    fn test_backtracking_alternation() {
        let mut grammar = HashMap::new();

        // start -> "fn" IDENT | "if" IDENT
        grammar.insert(
            "start".to_string(),
            vec![Production::Alternation(vec![
                Production::Sequence(vec![
                    Element::Terminal(Terminal::Keyword(
                        "fn".to_string(),
                        TerminalSource::Explicit,
                    )),
                    Element::Terminal(Terminal::TokenKind(
                        "IDENT".to_string(),
                        TerminalSource::Explicit,
                    )),
                ]),
                Production::Sequence(vec![
                    Element::Terminal(Terminal::Keyword(
                        "if".to_string(),
                        TerminalSource::Explicit,
                    )),
                    Element::Terminal(Terminal::TokenKind(
                        "IDENT".to_string(),
                        TerminalSource::Explicit,
                    )),
                ]),
            ])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        // First alternative
        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_token(TokenKind::Ident, "x", 3),
            make_eof(),
        ];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());

        // Second alternative
        let tokens = vec![
            make_token(TokenKind::Keyword("if".to_string()), "if", 0),
            make_token(TokenKind::Ident, "x", 3),
            make_eof(),
        ];
        let result = parse_with_spec(&spec, &tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_determinism() {
        let mut grammar = HashMap::new();
        grammar.insert(
            "start".to_string(),
            vec![Production::Sequence(vec![
                Element::Terminal(Terminal::Keyword(
                    "fn".to_string(),
                    TerminalSource::Explicit,
                )),
                Element::Terminal(Terminal::TokenKind(
                    "IDENT".to_string(),
                    TerminalSource::Explicit,
                )),
            ])],
        );

        let spec = ParserSpec {
            start: "start".to_string(),
            grammar,
        };

        let tokens = vec![
            make_token(TokenKind::Keyword("fn".to_string()), "fn", 0),
            make_token(TokenKind::Ident, "test", 3),
            make_eof(),
        ];

        // Parse twice and compare
        let result1 = parse_with_spec(&spec, &tokens);
        let result2 = parse_with_spec(&spec, &tokens);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
