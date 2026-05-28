use std::env;
use std::time::Instant;
use axis_rust_bridge::core_ir;

// Generated Cap'n Proto schema
mod axis_core_ir_0_3_capnp {
    include!(concat!(env!("OUT_DIR"), "/core_ir_spec/axis_core_ir_0_3_capnp.rs"));
}

fn usage_and_exit() -> ! {
    eprintln!("Usage:");
    eprintln!("  axis-rust-bridge build <path-to.coreir> --out <output-dir>/<binary> [--link-lib <name>] [--link-search <path>]");
    eprintln!("  axis-rust-bridge inspect <path-to.coreir>");
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --out <path>            Output binary path (artifacts written to same directory)");
    eprintln!("  --link-lib <name>       Link external library by name (repeatable)");
    eprintln!("  --link-search <path>    Add library search path (repeatable)");
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage_and_exit();
    }

    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "inspect" => {
            // Inspect a Core IR file
            if args.len() < 3 {
                eprintln!("Usage: axis-rust-bridge inspect <path-to.coreir>");
                std::process::exit(1);
            }
            let core_path = &args[2];
            match core_ir::inspect_core_bundle(core_path) {
                Ok(summary) => {
                    println!("{}", summary);
                    std::process::exit(0);
                },
                Err(e) => {
                    eprintln!("Failed to inspect Core IR: {}", e);
                    std::process::exit(1);
                }
            }
        },
        "build" => {
            // Build a binary from Core IR
            run_build(&args[2..]);
        },
        _ => {
            usage_and_exit();
        }
    }
}

fn run_build(args: &[String]) {
    let phase_start = Instant::now();
    eprintln!("[PHASE] phase4_axis_rust_bridge_run=start");
    
    let exit_code = (|| {
        // Parse arguments: build <input.coreir> --out <output-binary> [--link-lib <path>] [--link-search <path>]
        if args.is_empty() {
            eprintln!("usage: axis-rust-bridge build <input.coreir> --out <output-binary> [--link-lib <path>] [--link-search <path>]");
            return 1;
        }
        
        let input_path = &args[0];
        let mut output_binary = "a.out";
        let mut link_libs: Vec<String> = Vec::new();
        let mut link_searches: Vec<String> = Vec::new();
        
        // Parse flags
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--out" if i + 1 < args.len() => {
                    output_binary = &args[i + 1];
                    i += 2;
                }
                "--link-lib" if i + 1 < args.len() => {
                    link_libs.push(args[i + 1].clone());
                    i += 2;
                }
                "--link-search" if i + 1 < args.len() => {
                    link_searches.push(args[i + 1].clone());
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        // Load Core IR bundle
        let program = match axis_rust_bridge::core_ir::load_core_bundle(input_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: failed to load Core IR bundle: {:?}", e);
                return 1;
            }
        };
        
        // Emit Rust code
        let rust_code = axis_rust_bridge::runtime::emit_rust::emit_rust_from_core(
            &program.root_term,
            input_path,
            "main"
        );
        
        // Determine output directory from output_binary path
        let output_path = std::path::Path::new(output_binary);
        let output_dir = output_path.parent().unwrap_or(std::path::Path::new("."));
        let binary_name = output_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("a.out");
        
        // Create output directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(output_dir) {
            eprintln!("error: failed to create output directory: {}", e);
            return 1;
        }
        
        // Write generated.rs to output directory
        let generated_rs_path = output_dir.join("generated.rs");
        if let Err(e) = std::fs::write(&generated_rs_path, &rust_code) {
            eprintln!("error: failed to write generated.rs: {}", e);
            return 1;
        }
        
        // Compile with rustc, linking against axis-rust-bridge library
        // Find the rust-bridge library relative to this executable
        let exe_path = std::env::current_exe()
            .expect("Failed to get current executable path");
        let exe_dir = exe_path.parent()
            .expect("Failed to get executable directory");
        
        // The rlib is in the same directory as the executable
        // For release: rust-bridge/target/release/axis-rust-bridge and rust-bridge/target/release/libaxis_rust_bridge.rlib
        // For debug: rust-bridge/target/debug/axis-rust-bridge and rust-bridge/target/debug/libaxis_rust_bridge.rlib
        let target_dir = exe_dir;
        
        let mut rustc_cmd = std::process::Command::new("rustc");
        rustc_cmd
            .arg(&generated_rs_path)
            .arg("-o")
            .arg(output_binary)
            .arg("--extern")
            .arg(format!("axis_rust_bridge={}/libaxis_rust_bridge.rlib", target_dir.display()))
            .arg("-L")
            .arg(format!("dependency={}/deps", target_dir.display()));
        
        // Add external library search paths
        for search_path in &link_searches {
            rustc_cmd.arg("-L").arg(search_path);
        }
        
        // Add external libraries (by name, not path)
        for lib_name in &link_libs {
            rustc_cmd.arg("-l").arg(lib_name);
        }
        
        eprintln!("DEBUG: rustc command: {:?}", rustc_cmd);
        
        let rustc_status = rustc_cmd.status();
        
        match rustc_status {
            Ok(status) if status.success() => {
                eprintln!("success: binary written to {}", output_binary);
                0
            }
            Ok(status) => {
                eprintln!("error: rustc failed with exit code {:?}", status.code());
                1
            }
            Err(e) => {
                eprintln!("error: failed to invoke rustc: {}", e);
                1
            }
        }
    })();
    
    eprintln!(
        "[PHASE] phase4_axis_rust_bridge_run=end ms={}",
        phase_start.elapsed().as_millis()
    );
    std::process::exit(exit_code);
}
