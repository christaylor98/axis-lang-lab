// BOILERPLATE: minimal library surface to make the workspace compile.
// REMOVE OR REPLACE THIS FILE WITH REAL IMPLEMENTATION LATER.

pub mod cli_util;
pub mod config;
pub mod frontend;
pub mod generated;
pub use generated::axis_core_ir_0_3_capnp;
pub mod ir;
pub mod lowering;
pub mod normalisation;
pub mod registry;
pub mod spec;
pub mod validation;

// Normalization stage (NF AST production)
pub mod normalize;

// NF AST type (boundary enforcement)
pub mod nf_ast;

// Wave 6: Native lexer component (standalone, no pipeline deps)
pub mod native_lex;

// Wave 6: Codegen support
pub mod codegen;

// Wave 6: Execution substrate (Core IR interpreter)
pub mod execution;
pub use execution::interpreter;

// Wave A: End-to-end pipeline integration (specs → execution)
pub mod pipeline;

// Wave B: Introspection, traceability, and trust surfaces
pub mod introspection;

// Wave C: Manifest and sealing infrastructure
pub mod manifest;
pub mod sealed;

// Wave 4: Compiler hook framework
pub mod hooks;

// Generated lexer from canonical YAML spec
#[path = "generated_lexer.rs"]
pub mod generated_lexer;

/// Run a tiny demonstration to ensure the crate builds and prints debug info.
/// This is BOILERPLATE and should be removed once real logic is added.
pub fn run_boilerplate_demo() {
    println!("BOILERPLATE: axis-lang-lab running minimal demo");

    // Try to construct a small AST node from `frontend::ast` if available.
    // If the real project replaces these types, update accordingly.
    use frontend::ast::{EmptyBlock, FunctionDecl, Ident, NoParams};

    let f = FunctionDecl {
        name: Ident {
            value: "example".into(),
        },
        params: NoParams,
        body: EmptyBlock,
    };

    println!("BOILERPLATE: sample AST: {:#?}", f);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs() {
        // Keep test minimal; only ensure function compiles and can be called.
        run_boilerplate_demo();
    }
}
