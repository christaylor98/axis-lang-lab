// Native compiler component - standalone lexer API
// Does NOT depend on parser/AST/lowering/Core IR

use crate::frontend::lexer_engine::{lex_with_spec, LexError};
use crate::frontend::lexspec_load::{load_spec, LoadError};
use crate::frontend::token::Token;
use std::fs;
use std::path::Path;

/// Load spec and lex a source file
pub fn lex_file(spec_path: &Path, src_path: &Path) -> Result<Vec<Token>, LexFileError> {
    let spec = load_spec(spec_path)?;
    let src = fs::read_to_string(src_path)?;
    let tokens = lex_with_spec(&spec, &src)?;
    Ok(tokens)
}

/// Load spec and lex source string
pub fn lex_str(spec_path: &Path, src: &str) -> Result<Vec<Token>, LexFileError> {
    let spec = load_spec(spec_path)?;
    let tokens = lex_with_spec(&spec, src)?;
    Ok(tokens)
}

#[derive(Debug)]
pub enum LexFileError {
    SpecLoad(LoadError),
    Io(std::io::Error),
    Lex(LexError),
}

impl std::fmt::Display for LexFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexFileError::SpecLoad(e) => write!(f, "spec load failed: {}", e),
            LexFileError::Io(e) => write!(f, "I/O error: {}", e),
            LexFileError::Lex(e) => write!(f, "lex error: {}", e),
        }
    }
}

impl std::error::Error for LexFileError {}

impl From<LoadError> for LexFileError {
    fn from(e: LoadError) -> Self {
        LexFileError::SpecLoad(e)
    }
}

impl From<std::io::Error> for LexFileError {
    fn from(e: std::io::Error) -> Self {
        LexFileError::Io(e)
    }
}

impl From<LexError> for LexFileError {
    fn from(e: LexError) -> Self {
        LexFileError::Lex(e)
    }
}
