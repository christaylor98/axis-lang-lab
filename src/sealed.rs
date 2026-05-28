// Wave C: Sealed pipeline mode
// Configuration-free execution with embedded specs

use crate::manifest::ArtifactManifest;

/// Sealed mode configuration - all specs embedded at compile time
pub struct SealedConfig {
    /// Whether sealed mode is enabled (compile-time flag)
    pub enabled: bool,
}

impl SealedConfig {
    /// Check if sealed mode is enabled
    pub fn is_sealed() -> bool {
        // This will be controlled by a cargo feature flag
        cfg!(feature = "sealed")
    }

    /// Get embedded lexer spec content (only available in sealed mode)
    #[cfg(feature = "sealed")]
    pub fn get_lexer_spec_yaml() -> &'static str {
        crate::generated::embedded_lexer::get_embedded_lexer_spec()
    }

    #[cfg(not(feature = "sealed"))]
    pub fn get_lexer_spec_yaml() -> &'static str {
        panic!("Sealed mode not enabled - cannot access embedded specs")
    }

    /// Get embedded parser spec content (only available in sealed mode)
    #[cfg(feature = "sealed")]
    pub fn get_parser_spec_yaml() -> &'static str {
        crate::generated::embedded_parser::get_embedded_parser_spec()
    }

    #[cfg(not(feature = "sealed"))]
    pub fn get_parser_spec_yaml() -> &'static str {
        panic!("Sealed mode not enabled - cannot access embedded specs")
    }

    /// Get embedded AST schema content (only available in sealed mode)
    #[cfg(feature = "sealed")]
    pub fn get_ast_schema_yaml() -> &'static str {
        crate::generated::embedded_schema::get_embedded_ast_schema()
    }

    #[cfg(not(feature = "sealed"))]
    pub fn get_ast_schema_yaml() -> &'static str {
        panic!("Sealed mode not enabled - cannot access embedded specs")
    }

    /// Get embedded lowering spec (only available in sealed mode)
    #[cfg(feature = "sealed")]
    pub fn get_lowering_spec() -> &'static str {
        crate::generated::embedded_lowering::get_embedded_lowering_spec()
    }

    #[cfg(not(feature = "sealed"))]
    pub fn get_lowering_spec() -> &'static str {
        panic!("Sealed mode not enabled - cannot access embedded specs")
    }

    /// Get artifact manifest (only available in sealed mode)
    #[cfg(feature = "sealed")]
    pub fn get_manifest() -> ArtifactManifest {
        crate::generated::embedded_manifest::get_artifact_manifest()
    }

    #[cfg(not(feature = "sealed"))]
    pub fn get_manifest() -> ArtifactManifest {
        panic!("Sealed mode not enabled - cannot access manifest")
    }
}

/// Sealed pipeline - runs without external configuration
#[cfg(feature = "sealed")]
pub mod sealed_pipeline {
    use super::*;
    use crate::frontend::ast_builder;
    use crate::frontend::lexer_engine;
    use crate::frontend::lexspec_load;
    use crate::frontend::parser_runtime;
    use crate::frontend::parserspec_load;
    use crate::frontend::schema_ast;
    use crate::frontend::schema_load;
    use crate::ir::core_ir::CoreBundle;
    use std::fs;
    use std::path::Path;

    /// Execute sealed pipeline on source file
    pub fn run_sealed(source_path: &Path) -> Result<CoreBundle, Box<dyn std::error::Error>> {
        // Read source
        let source = fs::read_to_string(source_path)?;

        // Load specs from embedded YAML
        let lexer_spec_yaml = SealedConfig::get_lexer_spec_yaml();
        let parser_spec_yaml = SealedConfig::get_parser_spec_yaml();
        let schema_spec_yaml = SealedConfig::get_ast_schema_yaml();

        // Parse embedded specs
        let lexer_spec = lexspec_load::load_spec_from_string(lexer_spec_yaml)
            .map_err(|e| format!("Failed to load embedded lexer spec: {}", e))?;

        // Parser spec loading is more complex - we'd need the full parserspec_load API
        // For now, this is a placeholder showing the architecture

        // Lex
        let tokens = lexer_engine::lex(&source, &lexer_spec.lexer)
            .map_err(|e| format!("Lex error: {}", e))?;

        // Note: Full sealed pipeline implementation requires extending
        // the spec loaders to support from_string APIs

        Err("Sealed pipeline: full implementation requires spec loader extensions".into())
    }
}
