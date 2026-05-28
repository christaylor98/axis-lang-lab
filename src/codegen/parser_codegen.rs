// Parser spec codegen: YAML → static Rust parser config
// Generated code must be proven equivalent to runtime spec via tests

use std::fmt::Write;
use std::path::Path;

pub fn generate_parser_code(spec_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Note: Parser codegen is complex due to runtime structures
    // For now, we'll embed the YAML directly like lowering spec
    let spec_content = std::fs::read_to_string(spec_path)?;

    let mut code = String::new();

    // Header
    writeln!(code, "// GENERATED CODE - DO NOT EDIT")?;
    writeln!(code, "// Generated from: {}", spec_path.display())?;
    writeln!(code, "// Source of truth: YAML spec file")?;
    writeln!(
        code,
        "// This code embeds the parser spec for sealed builds"
    )?;
    writeln!(code)?;

    // Embed the spec content as a constant
    writeln!(code, "/// Embedded parser specification")?;
    writeln!(code, "pub const PARSER_SPEC: &str = r###\"")?;
    write!(code, "{}", spec_content)?;
    writeln!(code, "\"###;")?;
    writeln!(code)?;

    writeln!(code, "/// Get embedded parser spec content")?;
    writeln!(code, "pub fn get_embedded_parser_spec() -> &'static str {{")?;
    writeln!(code, "    PARSER_SPEC")?;
    writeln!(code, "}}")?;
    writeln!(code)?;

    Ok(code)
}
