// LexSpec data model - loaded from YAML specification

use serde::{Deserialize, Serialize};

/// Root lexer specification
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LexerSpec {
    pub lexer: LexerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LexerConfig {
    #[serde(default = "default_charset")]
    pub charset: String,

    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,

    #[serde(default)]
    pub whitespace: Option<WhitespaceRule>,

    #[serde(default)]
    pub comments: Vec<CommentRule>,

    #[serde(default)]
    pub keywords: Vec<String>,

    #[serde(default)]
    pub identifiers: Option<IdentifierRule>,

    #[serde(default)]
    pub literals: Option<LiteralRules>,

    #[serde(default)]
    pub punctuation: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WhitespaceRule {
    pub pattern: String,
    #[serde(default = "default_true")]
    pub skip: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommentRule {
    pub pattern: String,
    #[serde(default = "default_true")]
    pub skip: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifierRule {
    pub pattern: String,
    #[serde(default)]
    pub forbid: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LiteralRules {
    #[serde(default)]
    pub int: Option<IntLiteralRule>,

    #[serde(default)]
    pub bool: Option<BoolLiteralRule>,

    #[serde(default)]
    pub unit: Option<UnitLiteralRule>,

    #[serde(default)]
    pub string: Option<StringLiteralRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IntLiteralRule {
    pub pattern: String,
    #[serde(default = "default_base_10")]
    pub base: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoolLiteralRule {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitLiteralRule {
    pub literal: String,
    #[serde(default = "default_true")]
    pub single_token: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StringLiteralRule {
    pub delimiter: String,
    #[serde(default)]
    pub escapes: Vec<String>,
    #[serde(default = "default_true")]
    pub forbid_newlines: bool,
}

// Default values for serde
fn default_charset() -> String {
    "ascii".to_string()
}

fn default_case_sensitive() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_base_10() -> u32 {
    10
}
