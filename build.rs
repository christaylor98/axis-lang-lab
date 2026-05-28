fn main() {
    // Compile Cap'n Proto schema
    capnpc::CompilerCommand::new()
        .file("core_ir_spec/axis_core_ir_0_3.capnp")
        .run()
        .expect("capnpc schema compilation failed");

    // Wave C: Generate embedded specs if sealed mode is enabled
    #[cfg(feature = "sealed")]
    {
        println!("cargo:warning=Wave C: Generating embedded specs for sealed mode");
        generate_embedded_specs();
    }
}

#[cfg(feature = "sealed")]
fn generate_embedded_specs() {
    use std::fs;
    use std::path::PathBuf;

    let out_dir = PathBuf::from("src/generated");
    fs::create_dir_all(&out_dir).expect("Failed to create generated directory");

    // Note: We can't use the main crate's codegen here because build.rs runs before compilation
    // Instead, we'll generate placeholder files and rely on a pre-build step

    println!("cargo:warning=Sealed mode requires pre-generated specs in src/generated/");
    println!("cargo:warning=Run: cargo run --bin seal to generate specs before building with --features sealed");

    // Verify embedded spec files exist
    let required_files = vec![
        "src/generated/embedded_lexer.rs",
        "src/generated/embedded_parser.rs",
        "src/generated/embedded_schema.rs",
        "src/generated/embedded_lowering.rs",
        "src/generated/embedded_manifest.rs",
    ];

    for file in &required_files {
        if !PathBuf::from(file).exists() {
            panic!(
                "Sealed mode enabled but {} not found. Run 'cargo run --bin seal' first.",
                file
            );
        }
    }
}
