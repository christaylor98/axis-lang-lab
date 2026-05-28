// BOILERPLATE: expose frontend submodules used by the minimal demo.
// Remove or replace when implementing real frontend module layout.
pub mod ast;

// Legacy Phase 1 lexer - feature-gated, DO NOT USE in new code
#[cfg(feature = "legacy-lexer")]
pub mod lexer;

// Parser lexer adapter (uses spec-driven lexer backend)
pub mod parser_lex;

pub mod parser;

// Wave 6: Spec-driven lexer components
pub mod lexer_engine;
pub mod lexspec;
pub mod lexspec_load;
pub mod token;

// Wave 1: Parser specification loader
pub mod parserspec;
pub mod parserspec_load;
pub mod production_parser;

// Wave 2: Runtime parser engine
pub mod parser_runtime;

// Wave AI1: Postfix (RPN) parser runtime
pub mod postfix_parser;

// Wave 3: Generic AST builder (structural projection)
pub mod ast_builder;

// Wave 4: Schema-driven AST projection (explicit semantics)
pub mod schema_ast;
pub mod schema_load;

// Re-export commonly-used items for convenience in demo code.
pub use ast::*;
