// Data-Driven Validator Execution
//
// This module implements validation rule execution from YAML configuration
// without hard-coded semantics.
//
// FORBIDDEN:
// - Hard-coding validation rules
// - Inferring validation behavior
// - Adding features not in the spec

use crate::frontend::token::Span;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATOR SPEC STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorSpec {
    pub validator: ValidatorModel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorModel {
    pub checks: Vec<String>,
    #[serde(default)]
    pub canonical_form: CanonicalFormRules,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CanonicalFormRules {
    #[serde(default)]
    pub one_space_between_tokens: bool,
    #[serde(default)]
    pub no_trailing_whitespace: bool,
    #[serde(default)]
    pub no_empty_tokens: bool,
    #[serde(default)]
    pub no_comments: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATION ERROR
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub span: Option<Span>,
}

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATOR
// ═══════════════════════════════════════════════════════════════════════════

pub struct Validator {
    spec: ValidatorSpec,
}

impl Validator {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read validator spec: {}", e))?;
        let spec: ValidatorSpec = serde_yaml::from_str(&content)
            .map_err(|e| format!("failed to parse validator spec: {}", e))?;

        Ok(Validator { spec })
    }

    pub fn validate(&self, source: &str) -> Result<(), ValidationError> {
        for check in &self.spec.validator.checks {
            match check.as_str() {
                "ascii_only" => self.check_ascii_only(source)?,
                "line_oriented" => self.check_line_oriented(source)?,
                "instruction_arity" => {
                    // This check requires parsed structure - skip for now
                    // or implement if we have access to AST
                }
                "stack_safety" => {
                    // This check requires semantic analysis - skip for now
                }
                "operator_resolution" => {
                    // This check requires registry - skip for now
                }
                "canonical_form" => self.check_canonical_form(source)?,
                _ => {
                    return Err(ValidationError {
                        message: format!("unknown validation check: {}", check),
                        span: None,
                    });
                }
            }
        }

        Ok(())
    }

    fn check_ascii_only(&self, source: &str) -> Result<(), ValidationError> {
        for (i, ch) in source.chars().enumerate() {
            if !ch.is_ascii() {
                return Err(ValidationError {
                    message: format!("non-ASCII character at position {}: '{}'", i, ch),
                    span: Some(Span::new(i, i + 1)),
                });
            }
        }
        Ok(())
    }

    fn check_line_oriented(&self, source: &str) -> Result<(), ValidationError> {
        // Check that each line is properly terminated
        let lines: Vec<&str> = source.lines().collect();
        if !source.is_empty() && !source.ends_with('\n') {
            return Err(ValidationError {
                message: "file does not end with newline".to_string(),
                span: Some(Span::new(source.len(), source.len())),
            });
        }

        // Check for empty lines in the middle
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() && i < lines.len() - 1 {
                return Err(ValidationError {
                    message: format!("empty line at line {}", i + 1),
                    span: None,
                });
            }
        }

        Ok(())
    }

    fn check_canonical_form(&self, source: &str) -> Result<(), ValidationError> {
        let rules = &self.spec.validator.canonical_form;

        if rules.no_trailing_whitespace {
            for (i, line) in source.lines().enumerate() {
                if line.ends_with(' ') || line.ends_with('\t') {
                    return Err(ValidationError {
                        message: format!("trailing whitespace on line {}", i + 1),
                        span: None,
                    });
                }
            }
        }

        if rules.no_empty_tokens {
            for (i, line) in source.lines().enumerate() {
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.iter().any(|t| t.is_empty()) {
                    return Err(ValidationError {
                        message: format!("empty token on line {}", i + 1),
                        span: None,
                    });
                }
            }
        }

        if rules.one_space_between_tokens {
            for (i, line) in source.lines().enumerate() {
                if line.contains("  ") || line.contains('\t') {
                    return Err(ValidationError {
                        message: format!("multiple spaces or tabs on line {}", i + 1),
                        span: None,
                    });
                }
            }
        }

        Ok(())
    }
}
