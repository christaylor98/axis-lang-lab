// Schema spec codegen: YAML → static Rust schema definitions
// Generated code must be proven equivalent to runtime spec via tests

use std::fmt::Write;
use std::path::Path;

pub fn generate_schema_code(spec_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Note: Schema codegen is complex due to runtime structures
    // For now, we'll embed the YAML directly
    let spec_content = std::fs::read_to_string(spec_path)?;

    let mut code = String::new();

    // Header
    writeln!(code, "// GENERATED CODE - DO NOT EDIT")?;
    writeln!(code, "// Generated from: {}", spec_path.display())?;
    writeln!(code, "// Source of truth: YAML spec file")?;
    writeln!(code, "// This code embeds the AST schema for sealed builds")?;
    writeln!(code)?;

    // Embed the spec content as a constant
    writeln!(code, "/// Embedded AST schema specification")?;
    writeln!(code, "pub const AST_SCHEMA: &str = r###\"")?;
    write!(code, "{}", spec_content)?;
    writeln!(code, "\"###;")?;
    writeln!(code)?;

    writeln!(code, "/// Get embedded AST schema content")?;
    writeln!(code, "pub fn get_embedded_ast_schema() -> &'static str {{")?;
    writeln!(code, "    AST_SCHEMA")?;
    writeln!(code, "}}")?;
    writeln!(code)?;

    Ok(code)
}
