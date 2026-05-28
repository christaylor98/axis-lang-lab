// Lowering spec codegen: YAML → static Rust lowering rules
// Generated code must be proven equivalent to runtime spec via tests

use std::fmt::Write;
use std::fs;
use std::path::Path;

pub fn generate_lowering_code(spec_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let spec_content = fs::read_to_string(spec_path)?;

    let mut code = String::new();

    // Header
    writeln!(code, "// GENERATED CODE - DO NOT EDIT")?;
    writeln!(code, "// Generated from: {}", spec_path.display())?;
    writeln!(code, "// Source of truth: YAML spec file")?;
    writeln!(
        code,
        "// This code embeds the lowering spec for sealed builds"
    )?;
    writeln!(code)?;

    // Embed the spec content as a constant
    writeln!(code, "/// Embedded lowering specification")?;
    writeln!(code, "pub const LOWERING_SPEC: &str = r###\"")?;
    write!(code, "{}", spec_content)?;
    writeln!(code, "\"###;")?;
    writeln!(code)?;

    writeln!(code, "/// Get embedded lowering spec content")?;
    writeln!(
        code,
        "pub fn get_embedded_lowering_spec() -> &'static str {{"
    )?;
    writeln!(code, "    LOWERING_SPEC")?;
    writeln!(code, "}}")?;
    writeln!(code)?;

    Ok(code)
}
