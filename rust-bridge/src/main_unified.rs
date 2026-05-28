use std::path::PathBuf;
use clap::Parser;
use std::time::Instant;
use axis_rust_bridge::core_ir;

// Generated Cap'n Proto schema
mod axis_core_ir_0_3_capnp {
    include!(concat!(env!("OUT_DIR"), "/core_ir_spec/axis_core_ir_0_3_capnp.rs"));
}

/// Axis Rust Bridge - Core IR Linker
///
/// Links Core IR bundles and libraries into native executables.
///
/// Input semantics:
///   --coreir and --lib accept files or directories (recursive)
///   Repeated inputs are treated as idempotent set membership
///   All inputs are deduplicated deterministically
///
/// Example:
///   axis-rust-bridge --coreir hello.coreir --name hello
///
///   axis-rust-bridge --coreir main.coreir --coreir lib/ \
///                     --lib /usr/lib/libfoo.so \
///                     --name myapp
#[derive(Parser)]
#[command(name = "axis-rust-bridge")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Core IR file or directory (repeatable, directories recursively expanded)
    #[arg(long = "coreir")]
    coreir_paths: Vec<PathBuf>,

    /// Library file or directory (repeatable, directories recursively expanded)
    #[arg(long = "lib")]
    lib_paths: Vec<PathBuf>,

    /// Output executable name (default: first Core IR basename)
    #[arg(long)]
    name: Option<String>,

    /// Inspect mode: display Core IR contents without linking
    #[arg(long)]
    inspect: bool,
}

fn main() {
    let cli = Cli::parse();

    // Handle inspect mode (legacy compatibility)
    if cli.inspect {
        run_inspect_mode(&cli.coreir_paths);
        return;
    }

    // Validate required inputs
    if cli.coreir_paths.is_empty() {
        eprintln!("Error: At least one --coreir is required");
        std::process::exit(1);
    }

    // Normalize Core IR inputs (deduplicate, expand directories)
    use axis_rust_bridge::cli_util::{normalize_path_inputs, derive_name_from_first_input};
    
    let coreir_files = match normalize_path_inputs(&cli.coreir_paths, Some("coreir")) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error processing Core IR inputs: {}", e);
            std::process::exit(1);
        }
    };

    if coreir_files.is_empty() {
        eprintln!("Error: No Core IR files found in specified paths");
        std::process::exit(1);
    }

    // Normalize library inputs (deduplicate, expand directories)
    let lib_files = match normalize_path_inputs(&cli.lib_paths, None) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error processing library inputs: {}", e);
            std::process::exit(1);
        }
    };

    // Determine executable name
    // Multiple --name flags: last one wins (clap handles this automatically with Option)
    let exe_name = cli.name.unwrap_or_else(|| {
        // Try Core IR first, then libraries
        let all_inputs: Vec<PathBuf> = coreir_files.iter()
            .chain(lib_files.iter())
            .cloned()
            .collect();
        
        derive_name_from_first_input(&all_inputs)
            .unwrap_or_else(|| "a.out".to_string())
    });

    println!("Linking: {} Core IR bundles", coreir_files.len());
    if !lib_files.is_empty() {
        println!("Using {} external libraries", lib_files.len());
    }
    println!("Output: {}", exe_name);

    let phase_start = Instant::now();
    eprintln!("[PHASE] phase4_axis_rust_bridge_run=start");

    // NOTE: Linking pipeline integration is pending.
    // The old CLI only handled a single Core IR file directly.
    // The new unified CLI requires extending the linker to handle:
    // - Multiple Core IR bundles
    // - Library symbol resolution
    // - Deterministic linking order
    // This stub demonstrates the correct CLI semantics:
    // - Deterministic input normalization
    // - Set-based deduplication
    // - Default name derivation from first input
    
    eprintln!("Error: Linking pipeline integration pending");
    eprintln!("This is the new unified CLI - linker integration needed");
    
    eprintln!(
        "[PHASE] phase4_axis_rust_bridge_run=end ms={}",
        phase_start.elapsed().as_millis()
    );
    std::process::exit(1);
}

fn run_inspect_mode(coreir_paths: &[PathBuf]) {
    if coreir_paths.is_empty() {
        eprintln!("Error: --inspect requires at least one --coreir path");
        std::process::exit(1);
    }

    // Normalize inputs
    use axis_rust_bridge::cli_util::normalize_path_inputs;
    let coreir_files = match normalize_path_inputs(coreir_paths, Some("coreir")) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error processing Core IR inputs: {}", e);
            std::process::exit(1);
        }
    };

    if coreir_files.is_empty() {
        eprintln!("Error: No Core IR files found in specified paths");
        std::process::exit(1);
    }

    // Inspect each file
    for core_path in &coreir_files {
        println!("=== Inspecting: {} ===", core_path.display());
        match core_ir::inspect_core_bundle(core_path.to_str().unwrap()) {
            Ok(summary) => {
                println!("{}", summary);
            }
            Err(e) => {
                eprintln!("Failed to inspect Core IR: {}", e);
                std::process::exit(1);
            }
        }
        println!();
    }
}
