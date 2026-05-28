// Load lexer spec from YAML file

use crate::frontend::lexspec::LexerSpec;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LoadError {
    pub message: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lex spec load error: {}", self.message)
    }
}

impl std::error::Error for LoadError {}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError {
            message: format!("I/O error: {}", e),
        }
    }
}

impl From<serde_yaml::Error> for LoadError {
    fn from(e: serde_yaml::Error) -> Self {
        LoadError {
            message: format!("YAML parse error: {}", e),
        }
    }
}

/// Load a lexer specification from a YAML file
pub fn load_spec(path: &Path) -> Result<LexerSpec, LoadError> {
    let content = fs::read_to_string(path)?;
    let spec: LexerSpec = serde_yaml::from_str(&content)?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_existing_spec() {
        let path = PathBuf::from("lang-lab-poc-userfiles/lexer.yaml");
        if path.exists() {
            let spec = load_spec(&path).expect("should load existing spec");
            assert_eq!(spec.lexer.charset, "ascii");
            assert!(spec.lexer.case_sensitive);
            assert!(spec.lexer.keywords.contains(&"fn".to_string()));
        }
    }
}
