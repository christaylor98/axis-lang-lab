use clap::Parser;
use std::path::PathBuf;

/// Axis Language Lab - Direct Compilation Pipeline
///
/// Compiles Axis source files to Core IR.
/// Always runs the full pipeline: lex → parse → AST → normalize → lower → Core IR
/// Always writes Core IR output files.
///
/// Multi-file and multi-registry support:
///   --src can be specified multiple times (at least one required)
///   --reg can be specified multiple times (at least one required)
///
/// Registries are merged with last-wins collision policy.
/// All files must compile successfully or the command fails.
///
/// Example:
///   axis --lexer spec/lexer.yaml \
///        --parser spec/parser.yaml \
///        --schema spec/schema.yaml \
///        --src src/a.ax --src src/b.ax \
///        --reg base.yaml --reg user.yaml \
///        --out build/
#[derive(Parser)]
#[command(name = "axis")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to lexer spec file (YAML) - REQUIRED
    #[arg(long, required = true)]
    lexer: PathBuf,

    /// Path to parser spec file (YAML) - REQUIRED
    #[arg(long, required = true)]
    parser: PathBuf,

    /// Path to AST schema file (YAML) - REQUIRED
    #[arg(long, required = true)]
    schema: PathBuf,

    /// Path to normalization rules file (YAML)
    /// If not provided, auto-discovered from config directory
    #[arg(long)]
    normalize: Option<PathBuf>,

    /// Source file(s) to compile (can be specified multiple times) - AT LEAST ONE REQUIRED
    #[arg(long = "src", required = true)]
    files: Vec<PathBuf>,

    /// Registry file(s) (can be specified multiple times) - AT LEAST ONE REQUIRED
    /// Registries are merged with last-wins collision policy on duplicate names.
    #[arg(long = "reg", required = true)]
    registries: Vec<PathBuf>,

    /// Optional entry rule (uses parser spec's start rule if omitted)
    #[arg(long)]
    entry: Option<String>,

    /// Parser mode: "grammar" (default) or "postfix" (for AI-1 RPN)
    #[arg(long)]
    parser_mode: Option<String>,

    /// Inspect pipeline stages (can be specified multiple times)
    /// Valid values: lexer, parser, grammar, cst, schema, ast, normalisation, nf, registry, core-ir, pipeline, code_trace, all
    /// Inspections are observational only and do not skip compilation stages.
    #[arg(long = "inspect", value_name = "STAGE")]
    inspect: Vec<String>,

    /// Output path for Core IR bundle
    /// If directory: creates file named after first source file
    /// If file path: writes to that exact path
    /// Default: "coreir" directory, file named after first source file
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    // Validate all required paths exist on disk
    if !cli.lexer.exists() {
        eprintln!("Error: Lexer spec not found: {}", cli.lexer.display());
        std::process::exit(1);
    }
    if !cli.parser.exists() {
        eprintln!("Error: Parser spec not found: {}", cli.parser.display());
        std::process::exit(1);
    }
    if !cli.schema.exists() {
        eprintln!("Error: Schema spec not found: {}", cli.schema.display());
        std::process::exit(1);
    }

    // Normalize source inputs (expand directories, deduplicate)
    let source_files = match axis_lang_lab::cli_util::normalize_path_inputs(&cli.files, None) {
        Ok(files) => {
            if files.is_empty() {
                eprintln!("Error: No source files found in specified paths");
                std::process::exit(1);
            }
            files
        }
        Err(e) => {
            eprintln!("Error: Failed to load source files: {}", e);
            std::process::exit(1);
        }
    };

    // Normalize registry inputs (expand directories, deduplicate)
    let registry_files = match axis_lang_lab::cli_util::normalize_path_inputs(&cli.registries, None) {
        Ok(files) => {
            if files.is_empty() {
                eprintln!("Error: No registry files found in specified paths");
                std::process::exit(1);
            }
            files
        }
        Err(e) => {
            eprintln!("Error: Failed to load registry files: {}", e);
            std::process::exit(1);
        }
    };

    // Load and merge registries (last-wins collision policy)
    use axis_lang_lab::registry::Registry;
    let mut merged_registry = Registry::new();

    for registry_path in &registry_files {
        match Registry::from_file(registry_path) {
            Ok(reg) => {
                // Merge: last-wins on duplicate names
                for entry in reg.entries() {
                    merged_registry.register(entry.name.clone(), entry.clone());
                }
            }
            Err(e) => {
                eprintln!(
                    "Error: Failed to load registry {}: {:?}",
                    registry_path.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    // Parse inspect flags
    let mut inspect_stages = std::collections::HashSet::new();
    for stage in &cli.inspect {
        if stage == "all" {
            inspect_stages.insert("lexer".to_string());
            inspect_stages.insert("parser".to_string());
            inspect_stages.insert("grammar".to_string());
            inspect_stages.insert("cst".to_string());
            inspect_stages.insert("schema".to_string());
            inspect_stages.insert("ast".to_string());
            inspect_stages.insert("normalisation".to_string());
            inspect_stages.insert("nf".to_string());
            inspect_stages.insert("registry".to_string());
            inspect_stages.insert("core-ir".to_string());
            inspect_stages.insert("pipeline".to_string());
            inspect_stages.insert("code_trace".to_string());
        } else if matches!(
            stage.as_str(),
            "lexer"
                | "parser"
                | "grammar"
                | "cst"
                | "schema"
                | "ast"
                | "normalisation"
                | "nf"
                | "registry"
                | "core-ir"
                | "pipeline"
                | "code_trace"
        ) {
            inspect_stages.insert(stage.clone());
        } else {
            eprintln!("Error: Unknown inspect stage: {}", stage);
            eprintln!("Valid stages: lexer, parser, grammar, cst, schema, ast, normalisation, nf, registry, core-ir, pipeline, code_trace, all");
            std::process::exit(1);
        }
    }

    // Normalize parser_mode
    let normalized_parser_mode = cli.parser_mode.as_ref().map(|s| s.trim().to_lowercase());

    // Discover normalization config if not explicitly provided
    let normalize_config = match &cli.normalize {
        Some(path) => {
            if !path.exists() {
                eprintln!("Error: Normalization config not found: {}", path.display());
                std::process::exit(1);
            }
            path.clone()
        }
        None => {
            // Auto-discover from config directory (same dir as lexer)
            let config_dir = cli.lexer.parent().unwrap_or_else(|| {
                eprintln!(
                    "Error: Cannot determine config directory from lexer path: {}",
                    cli.lexer.display()
                );
                std::process::exit(1);
            });

            // Try common normalize file patterns
            let patterns = ["normalize.yaml", "*-normalize.yaml"];

            let mut found: Vec<PathBuf> = Vec::new();
            for pattern in &patterns {
                if pattern.contains('*') {
                    // Glob pattern
                    if let Ok(entries) = std::fs::read_dir(config_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.ends_with("-normalize.yaml") {
                                    found.push(path);
                                }
                            }
                        }
                    }
                } else {
                    let candidate = config_dir.join(pattern);
                    if candidate.exists() {
                        found.push(candidate);
                    }
                }
            }

            if found.is_empty() {
                eprintln!(
                    "Error: No normalization config found in {}",
                    config_dir.display()
                );
                eprintln!("Expected: normalize.yaml or *-normalize.yaml");
                eprintln!("Surface configs require normalization rules.");
                std::process::exit(1);
            } else if found.len() > 1 {
                eprintln!(
                    "Error: Multiple normalization configs found in {}",
                    config_dir.display()
                );
                for f in &found {
                    eprintln!("  - {}", f.display());
                }
                eprintln!("Specify one explicitly with --normalize");
                std::process::exit(1);
            }

            found.into_iter().next().unwrap()
        }
    };

    // Determine output file path
    let output_file = match &cli.out {
        Some(path) => {
            if path.extension().is_some() {
                // Has extension: use as-is (file path)
                path.clone()
            } else {
                // No extension: treat as directory
                if !path.exists() {
                    if let Err(e) = std::fs::create_dir_all(path) {
                        eprintln!(
                            "Error: Failed to create output directory {}: {}",
                            path.display(),
                            e
                        );
                        std::process::exit(1);
                    }
                }
                let first_file = &source_files[0];
                let filename = first_file.file_stem().unwrap().to_str().unwrap();
                path.join(format!("{}.coreir", filename))
            }
        }
        None => {
            // Default: coreir/ directory, named after first source file
            let default_dir = PathBuf::from("coreir");
            if !default_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&default_dir) {
                    eprintln!(
                        "Error: Failed to create output directory {}: {}",
                        default_dir.display(),
                        e
                    );
                    std::process::exit(1);
                }
            }
            let first_file = &source_files[0];
            let filename = first_file.file_stem().unwrap().to_str().unwrap();
            default_dir.join(format!("{}.coreir", filename))
        }
    };

    // Compilation loop: process each file
    let mut all_succeeded = true;
    let mut results = Vec::new();

    for source_file in &source_files {
        use axis_lang_lab::pipeline::{run_pipeline, PipelineConfig};

        let config = PipelineConfig {
            lexer_spec: cli.lexer.clone(),
            parser_spec: cli.parser.clone(),
            ast_schema: cli.schema.clone(),
            source_file: source_file.clone(),
            entry_rule: cli.entry.clone(),
            normalize_spec: normalize_config.clone(),
            registry: merged_registry.clone(), // Registry already loaded and merged
            parser_mode: normalized_parser_mode.clone(),
            hook_registry: None,
        };

        // Apply inspections if requested (observational only, does not skip compilation)
        if !inspect_stages.is_empty() {
            use axis_lang_lab::introspection::inspection::run_inspection;
            eprintln!("=== Inspecting file: {} ===", source_file.display());
            if let Err(e) = run_inspection(&config, inspect_stages.clone()) {
                eprintln!("Inspection failed for {}: {}", source_file.display(), e);
                all_succeeded = false;
                continue;
            }
        }

        // Always run full pipeline
        let artifacts = match run_pipeline(&config) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Compilation failed for {}: {:?}", source_file.display(), e);
                all_succeeded = false;
                continue;
            }
        };

        results.push((source_file.clone(), artifacts));
    }

    if !all_succeeded {
        eprintln!("Error: One or more files failed to compile");
        std::process::exit(1);
    }

    // Write single Core IR bundle containing all compiled files
    // For now, use the Core IR from the first file
    // TODO: Merge multiple Core IR bundles if needed
    use axis_lang_lab::ir::core_ir::encode_capnp;
    use std::fs;

    let core_ir_bundle = &results[0].1.core_ir;

    let bytes = match encode_capnp(core_ir_bundle) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Core IR encoding failed: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = fs::write(&output_file, &bytes) {
        eprintln!("Failed to write Core IR: {}", e);
        std::process::exit(1);
    }

    println!("Core IR written: {}", output_file.display());
    for (source_file, _) in &results {
        println!("  Compiled: {}", source_file.display());
    }

    std::process::exit(0);
}
