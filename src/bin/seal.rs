// Wave C: Sealing tool
// Generates embedded specs and manifest for sealed builds

use axis_lang_lab::codegen::lexer_codegen::generate_lexer_code;
use axis_lang_lab::codegen::lowering_codegen::generate_lowering_code;
use axis_lang_lab::codegen::parser_codegen::generate_parser_code;
use axis_lang_lab::codegen::schema_codegen::generate_schema_code;
use axis_lang_lab::manifest::ArtifactManifest;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Wave C: Sealing Axis Language Lab");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let output_dir = PathBuf::from("src/generated");
    fs::create_dir_all(&output_dir)?;

    let specs = vec![
        ("lexer", "lang-lab-poc-userfiles/lexer.yaml"),
        ("parser", "lang-lab-poc-userfiles/parsing.yaml"),
        ("schema", "lang-lab-poc-userfiles/ast_schema.yaml"),
        ("lowering", "lang-lab-poc-userfiles/lowering.yaml"),
    ];

    let mut manifest = ArtifactManifest::new();
    manifest
        .metadata
        .insert("wave".to_string(), "C".to_string());
    manifest
        .metadata
        .insert("mode".to_string(), "sealed".to_string());

    // Generate lexer
    println!("🔧 Generating embedded lexer...");
    let lexer_path = PathBuf::from(specs[0].1);
    let lexer_code = generate_lexer_code(&lexer_path)?;
    let lexer_out = output_dir.join("embedded_lexer.rs");
    fs::write(&lexer_out, lexer_code)?;
    manifest.add_spec("lexer".to_string(), &lexer_path)?;
    println!("   ✓ {}", lexer_out.display());

    // Generate parser
    println!("🔧 Generating embedded parser...");
    let parser_path = PathBuf::from(specs[1].1);
    let parser_code = generate_parser_code(&parser_path)?;
    let parser_out = output_dir.join("embedded_parser.rs");
    fs::write(&parser_out, parser_code)?;
    manifest.add_spec("parser".to_string(), &parser_path)?;
    println!("   ✓ {}", parser_out.display());

    // Generate schema
    println!("🔧 Generating embedded schema...");
    let schema_path = PathBuf::from(specs[2].1);
    let schema_code = generate_schema_code(&schema_path)?;
    let schema_out = output_dir.join("embedded_schema.rs");
    fs::write(&schema_out, schema_code)?;
    manifest.add_spec("schema".to_string(), &schema_path)?;
    println!("   ✓ {}", schema_out.display());

    // Generate lowering
    println!("🔧 Generating embedded lowering...");
    let lowering_path = PathBuf::from(specs[3].1);
    let lowering_code = generate_lowering_code(&lowering_path)?;
    let lowering_out = output_dir.join("embedded_lowering.rs");
    fs::write(&lowering_out, lowering_code)?;
    manifest.add_spec("lowering".to_string(), &lowering_path)?;
    println!("   ✓ {}", lowering_out.display());

    // Generate manifest
    println!("🔧 Generating artifact manifest...");
    let manifest_code = manifest.generate_embedded_code()?;
    let manifest_out = output_dir.join("embedded_manifest.rs");
    fs::write(&manifest_out, manifest_code)?;
    println!("   ✓ {}", manifest_out.display());

    // Update mod.rs
    println!("🔧 Updating generated module declarations...");
    let mod_code = r#"// GENERATED CODE - DO NOT EDIT
// Wave C: Embedded spec modules

#[cfg(feature = "sealed")]
pub mod embedded_lexer;

#[cfg(feature = "sealed")]
pub mod embedded_parser;

#[cfg(feature = "sealed")]
pub mod embedded_schema;

#[cfg(feature = "sealed")]
pub mod embedded_lowering;

#[cfg(feature = "sealed")]
pub mod embedded_manifest;
"#;
    let mod_out = output_dir.join("embedded_mod.rs");
    fs::write(&mod_out, mod_code)?;
    println!("   ✓ {}", mod_out.display());

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ Sealing complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Manifest Summary:");
    println!("  Build: {}", manifest.build_timestamp);
    if let Some(commit) = &manifest.git_commit {
        println!("  Commit: {}", commit);
    }
    if let Some(branch) = &manifest.git_branch {
        println!("  Branch: {}", branch);
    }
    println!("  Compiler: {}", manifest.compiler_version);
    println!("  Specs: {} embedded", manifest.specs.len());
    println!();
    println!("To build sealed binary:");
    println!("  cargo build --release --features sealed");
    println!();

    Ok(())
}
