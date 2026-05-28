// ParserSpec data model - Wave 1: Spec Loader Only

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root parser specification
#[derive(Debug, Clone)]
pub struct ParserSpec {
    pub start: NonTerminal,
    pub grammar: HashMap<NonTerminal, Vec<Production>>,
}

/// Non-terminal symbol identifier
pub type NonTerminal = String;

/// Parsed production rule (structured form)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Production {
    /// Sequence of elements that must appear in order
    Sequence(Vec<Element>),
    /// Alternative choices (A | B | C)
    Alternation(Vec<Production>),
}

/// Element within a production
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// Reference to another non-terminal
    NonTerminal(String),
    /// Terminal symbol (keyword, punctuation, or token kind)
    Terminal(Terminal),
    /// Repeated element with quantifier
    Repeat(Box<Element>, RepeatKind),
    /// Grouped sub-expression
    Group(Vec<Element>),
}

// ═══════════════════════════════════════════════════════════════════════════
// LEGACY INFERENCE WARNING — WAVE 1 COMPATIBILITY ONLY
// ═══════════════════════════════════════════════════════════════════════════
//
// The TerminalSource enum exists to track whether a terminal was explicitly
// specified in the grammar (e.g., kw:"fn") or inferred from legacy syntax
// (e.g., "fn" → kw:"fn").
//
// IMPORTANT:
// - Legacy inference is TEMPORARY and exists ONLY for Wave 1 spec loading
// - Legacy inference is NON-AUTHORITATIVE
// - Wave 2+ runtime parsing MUST NOT depend on inference
// - All canonical grammars MUST use explicit terminal syntax
//
// This tag ensures Wave 2 cannot accidentally rely on inferred terminals.
// ═══════════════════════════════════════════════════════════════════════════

/// Source of a terminal symbol (explicit vs inferred)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalSource {
    /// Explicitly specified using formal syntax: <TOKEN>, kw:"...", punct:"..."
    Explicit,
    /// Inferred from legacy syntax (Wave 1 compatibility ONLY)
    /// Examples: "fn" → kw:"fn", IDENT → <IDENT>
    /// THIS IS NON-AUTHORITATIVE AND TEMPORARY
    InferredLegacy,
}

/// Terminal symbol types (explicit, typed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminal {
    /// Token kind from lexer: <IDENT>, <INT_LIT>
    TokenKind(String, TerminalSource),
    /// Keyword: kw:"if"
    Keyword(String, TerminalSource),
    /// Punctuation/operator: punct:"+"
    Punct(String, TerminalSource),
}

/// Repetition quantifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatKind {
    /// Zero or more: *
    ZeroOrMore,
    /// One or more: +
    OneOrMore,
    /// Optional: ?
    Optional,
}

// YAML deserialization structures (raw form)

/// Raw YAML parser specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawParserSpec {
    pub parser: RawParserConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawParserConfig {
    pub start: String,
    pub grammar: HashMap<String, Vec<String>>,
}
