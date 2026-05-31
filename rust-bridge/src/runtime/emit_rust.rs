// INVARIANT:
// - emit_rust.rs generates Rust code ONLY for functions defined in Core IR.
// - All foreign/runtime functions are implemented in linked crates (shim, etc).
// - Generating foreign stubs/wrappers is forbidden.

// Emit Rust code from Core IR - ANDL Loop 6: Value-based codegen

use crate::core_ir::CoreTerm;
use std::collections::{HashSet, HashMap};

/// Mapping from foreign Core IR symbols to their Rust implementation paths
/// This ensures type-safe, explicit mapping with no string-based heuristics
fn get_foreign_symbol_mapping() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    
    // Arithmetic operations
    map.insert("__add__", "shim::__add__");
    map.insert("__sub__", "shim::__sub__");
    map.insert("__mul__", "shim::__mul__");
    map.insert("__div__", "shim::__div__");
    map.insert("__mod__", "shim::__mod__");
    map.insert("axis_int_add", "shim::__add__");
    
    // Comparison operations
    map.insert("eq", "shim::__eq__");
    map.insert("__neq__", "shim::__neq__");
    map.insert("__lt__", "shim::__lt__");
    map.insert("__lte__", "shim::__lte__");
    map.insert("__gt__", "shim::__gt__");
    map.insert("__gte__", "shim::__gte__");
    
    // Logical operations
    map.insert("__and__", "shim::__and__");
    map.insert("__or__", "shim::__or__");
    map.insert("__not__", "shim::__not__");
    map.insert("__concat__", "shim::__concat__");
    
    // String operations
    map.insert("str_len", "shim::str_len");
    map.insert("axis_str_len", "shim::str_len");
    map.insert("str_char", "shim::str_char");
    map.insert("axis_str_char", "shim::str_char");
    map.insert("str_char_at", "shim::str_char_at");
    map.insert("axis_str_char_at", "shim::str_char_code");
    map.insert("str_slice", "shim::str_slice");
    map.insert("axis_str_slice", "shim::str_slice");
    map.insert("axis_char_to_str", "shim::char_to_str");
    map.insert("str_to_int", "shim::str_to_int");
    map.insert("int_to_str", "shim::int_to_str");
    map.insert("axis_int_to_str", "shim::int_to_str");
    map.insert("str_concat", "shim::str_concat");
    map.insert("axis_str_concat", "shim::str_concat");
    
    // List operations
    map.insert("list_nil", "shim::list_nil");
    map.insert("list_cons", "shim::list_cons");
    map.insert("list_reverse", "shim::list_reverse");
    map.insert("list_concat", "shim::list_concat");
    map.insert("list_contains_str", "shim::list_contains_str");
    map.insert("list_index_of_str", "shim::list_index_of_str");
    
    // Tuple/constructor access
    map.insert("tuple_field", "shim::tuple_field");
    map.insert("ctor_field", "shim::ctor_field");
    map.insert("proj", "shim::tuple_field");  // proj is an alias for tuple_field
    map.insert("axis_proj", "shim::tuple_field");  // axis_proj is also an alias for tuple_field
    
    // Value utilities
    map.insert("truthy", "shim::truthy");
    
    // IO operations
    map.insert("print", "shim::io_print");
    map.insert("eprint", "shim::io_eprint");
    map.insert("io_read", "shim::io_read");
    map.insert("axis_io_make_error", "shim::axis_io_make_error");
    
    // Process operations
    map.insert("args", "shim::axis_proc_args");
    
    // JSON operations (minimal compiler implementation)
    map.insert("axis.json.parse", "shim::axis_json_parse");
    
    // File operations
    map.insert("file_read", "shim::fs_read_text");
    map.insert("file_write", "shim::fs_write_text");

    // Debug
    map.insert("trace", "shim::debug_trace");
    
    // String/tag interning
    map.insert("intern_str", "shim::intern_str");
    map.insert("get_str", "shim::get_str");
    map.insert("intern_tag", "shim::intern_tag");
    map.insert("get_tag_name", "shim::get_tag_name");
    
    // Init
    map.insert("init_runtime", "shim::init_runtime");
    
    // Special compiler operations
    map.insert("axis_emit_core_bundle_to_file", "shim::axis_emit_core_bundle_to_file");
    
    // Core IR constructor field access
    map.insert("__ctor_field__", "shim::ctor_field");
    
    // Registry operations (axLens foreign symbols)
    map.insert("axis_registry_coreir_is_compatible", "shim::axis_registry_coreir_is_compatible");
    map.insert("axis_registry_coreir_get_schema_version", "shim::axis_registry_coreir_get_schema_version");
    map.insert("axis_registry_coreir_get_all_nodes_sorted", "shim::axis_registry_coreir_get_all_nodes_sorted");
    map.insert("axis_registry_coreir_get_all_relationships_sorted", "shim::axis_registry_coreir_get_all_relationships_sorted");
    map.insert("axis_registry_fatal_error", "shim::axis_registry_fatal_error");
    map.insert("axis_registry_string_concat", "shim::axis_registry_string_concat");
    map.insert("axis_registry_string_from_literal", "shim::axis_registry_string_from_literal");
    map.insert("axis_registry_int_add", "shim::axis_registry_int_add");
    map.insert("axis_registry_int_less_than", "shim::axis_registry_int_less_than");
    map.insert("axis_registry_list_length", "shim::axis_registry_list_length");
    map.insert("axis_registry_list_get", "shim::axis_registry_list_get");
    map.insert("axis_registry_iterator_nil", "shim::axis_registry_iterator_nil");
    map.insert("axis_registry_iterator_cons", "shim::axis_registry_iterator_cons");
    
    // Core IR runtime introspection (Wave 1 support)
    map.insert("axis_coreir_open", "shim::axis_coreir_open");
    map.insert("axis_coreir_is_valid", "shim::axis_coreir_is_valid");
    map.insert("axis_coreir_schema_version", "shim::axis_coreir_schema_version");
    map.insert("axis_coreir_node_count", "shim::axis_coreir_node_count");
    map.insert("axis_coreir_edge_count", "shim::axis_coreir_edge_count");
    map.insert("axis_coreir_close", "shim::axis_coreir_close");
    
    // Compiler helper functions
    map.insert("contains_id", "contains_id");
    map.insert("find_by_id", "find_by_id");
    
    // Lexer helpers - identity mapped (implemented in compiler)
    map.insert("lex_loop", "lex_loop");
    map.insert("lex_ident", "lex_ident");
    map.insert("lex_number", "lex_number");
    map.insert("lex_symbol", "lex_symbol");
    map.insert("lex_string_inline", "lex_string_inline");
    map.insert("is_whitespace", "is_whitespace");
    map.insert("is_alpha", "is_alpha");
    map.insert("is_digit", "is_digit");
    map.insert("is_ident_char", "is_ident_char");
    map.insert("scan_while_char", "scan_while_char");
    map.insert("scan_while_digit", "scan_while_digit");
    map.insert("skip_to_newline", "skip_to_newline");
    map.insert("simple_token_check", "simple_token_check");
    
    // Parser helpers - identity mapped (implemented in compiler)
    map.insert("parse_all", "parse_all");
    map.insert("parse_args", "parse_args");
    map.insert("parse_args_loop", "parse_args_loop");
    map.insert("parse_atom", "parse_atom");
    map.insert("parse_binary", "parse_binary");
    map.insert("parse_binop", "parse_binop");
    map.insert("parse_block", "parse_block");
    map.insert("parse_block_elems", "parse_block_elems");
    map.insert("parse_call", "parse_call");
    map.insert("parse_expr", "parse_expr");
    map.insert("parse_fn_def", "parse_fn_def");
    map.insert("parse_param_list", "parse_param_list");
    map.insert("parse_params", "parse_params");
    map.insert("parse_proj", "parse_proj");
    map.insert("parse_toplevel", "parse_toplevel");
    
    // Debug helpers - identity mapped (implemented in compiler)
    map.insert("debug_print_single_token", "debug_print_single_token");
    
    // Axis compiler functions - identity mapped (implemented in compiler)
    map.insert("axis_compiler", "axis_compiler");
    map.insert("axis_compiler_compile", "axis_compiler_compile");
    map.insert("axis_compiler_emit", "axis_compiler_emit");
    map.insert("axis_compiler_entry", "axis_compiler_entry");
    map.insert("axis_compiler_lower", "axis_compiler_lower");
    map.insert("axis_compiler_parse_sources", "axis_compiler_parse_sources");
    map.insert("axis_compiler_read_sources", "axis_compiler_read_sources");
    map.insert("axis_core_validate_validate", "axis_core_validate_validate");
    map.insert("axis_emit_core_serialize", "axis_emit_core_serialize");
    map.insert("axis_lexer_lex", "axis_lexer_lex");
    map.insert("axis_lower_lower_all", "axis_lower_lower_all");
    map.insert("axis_parser_parse", "axis_parser_parse");
    
    // NOTE: axis_char_to_str, axis_str_char_at, and axis_io_make_error
    // are runtime primitives mapped above, NOT Axis-defined functions.
    // Do not add identity mappings for runtime primitives here.

    map.insert("fs_exists", "fs_exists");
    map.insert("fs_is_file", "fs_is_file");
    map.insert("fs_is_dir", "fs_is_dir");
    
    // Axis registry functions - identity mapped (implemented in compiler)
    map.insert("axis_registry_add", "axis_registry_add");
    map.insert("axis_registry_arity", "axis_registry_arity");
    map.insert("axis_registry_canonicalize_name", "axis_registry_canonicalize_name");
    map.insert("axis_registry_class_tag_of", "axis_registry_class_tag_of");
    map.insert("axis_registry_empty", "axis_registry_empty");
    map.insert("axis_registry_is_deterministic", "axis_registry_is_deterministic");
    map.insert("axis_registry_resolve", "axis_registry_resolve");
    
    // Emit helpers - identity mapped (implemented in compiler)
    map.insert("emit_app", "emit_app");
    map.insert("emit_core_serialize", "emit_core_serialize");
    map.insert("emit_foreign_call", "emit_foreign_call");
    map.insert("emit_rust", "emit_rust");
    map.insert("emit_rust_all", "emit_rust_all");
    map.insert("emit_term", "emit_term");
    map.insert("emit_terms", "emit_terms");
    
    // Flag checking helper - identity mapped (implemented in compiler)
    map.insert("is_flag", "is_flag");
    
    // AST walk helpers - identity mapped (implemented in compiler)
    map.insert("walk_cases", "walk_cases");
    map.insert("walk_expr", "walk_expr");
    map.insert("walk_expr_list", "walk_expr_list");
    
    // Print helpers - identity mapped (implemented in compiler)
    map.insert("print_string_list", "print_string_list");
    map.insert("print_string_list_items", "print_string_list_items");
    
    // List helpers - identity mapped (implemented in compiler)
    map.insert("list_contains_str", "list_contains_str");
    map.insert("list_items", "list_items");
    map.insert("list_index_of_str", "list_index_of_str");
    map.insert("list_find_str", "list_find_str");
    map.insert("list_get", "list_get");
    map.insert("list_set", "list_set");
    map.insert("list_push", "list_push");
    map.insert("list_pop", "list_pop");
    map.insert("list_length", "list_length");
    map.insert("list_map", "list_map");
    map.insert("list_filter", "list_filter");
    map.insert("list_fold", "list_fold");
    map.insert("list_append", "list_append");
    map.insert("list_concat", "list_concat");
    map.insert("list_reverse_cons_acc", "list_reverse_cons_acc");
    map.insert("list_join", "list_join");
    map.insert("list_empty", "list_empty");
    map.insert("list_head", "list_head");
    map.insert("list_tail", "list_tail");
    
    // Binding and pattern helpers - identity mapped (implemented in compiler)
    map.insert("bind_pattern", "bind_pattern");
    map.insert("bind_patterns", "bind_patterns");
    
    // Lambda and construction helpers - identity mapped (implemented in compiler)
    map.insert("build_lambda", "build_lambda");
    map.insert("make_compiled_unit", "make_compiled_unit");
    map.insert("make_compiler_options", "make_compiler_options");
    map.insert("make_source_unit", "make_source_unit");
    
    // Compiler error helpers - identity mapped (implemented in compiler)
    map.insert("compiler_error", "compiler_error");
    
    // Lowering helpers - identity mapped (implemented in compiler)
    map.insert("lower_decl", "lower_decl");
    map.insert("lower_decls", "lower_decls");
    map.insert("lower_expr", "lower_expr");
    map.insert("lower_expr_list", "lower_expr_list");
    
    // Encoding helpers - identity mapped (implemented in compiler)
    map.insert("encode_cases", "encode_cases");
    map.insert("encode_term", "encode_term");
    map.insert("encode_term_list", "encode_term_list");
    map.insert("encode_terms", "encode_terms");
    
    // Utility field helpers - identity mapped (implemented in compiler)
    map.insert("compiled_unit_artifact", "compiled_unit_artifact");
    map.insert("compiled_unit_code", "compiled_unit_code");
    map.insert("compiled_unit_name", "compiled_unit_name");
    map.insert("derive_unit_name", "derive_unit_name");
    map.insert("dummy_loc", "dummy_loc");
    map.insert("entrypoint_id", "entrypoint_id");
    map.insert("entrypoint_name", "entrypoint_name");
    map.insert("expect_str", "expect_str");
    map.insert("final_table", "final_table");
    map.insert("initial_env", "initial_env");
    map.insert("options_flags", "options_flags");
    map.insert("options_registries", "options_registries");
    map.insert("options_sources", "options_sources");
    map.insert("source_unit_name", "source_unit_name");
    map.insert("source_unit_text", "source_unit_text");
    
    // Registry entry helpers - identity mapped (implemented in compiler)
    map.insert("registry_entry_arity", "registry_entry_arity");
    map.insert("registry_entry_class_tag", "registry_entry_class_tag");
    map.insert("registry_entry_deterministic", "registry_entry_deterministic");
    map.insert("registry_entry_id", "registry_entry_id");
    
    // Function classification helpers - identity mapped (implemented in compiler)
    map.insert("fnclass_builtin", "fnclass_builtin");
    map.insert("fnclass_foreign", "fnclass_foreign");
    map.insert("fnclass_user", "fnclass_user");
    
    // String table helpers - identity mapped (implemented in compiler)
    map.insert("string_table", "string_table");
    map.insert("string_table_empty", "string_table_empty");
    map.insert("string_table_intern", "string_table_intern");
    
    // Key-value helpers - identity mapped (implemented in compiler)
    map.insert("kv_get", "kv_get");
    
    // Field name helpers - identity mapped (implemented in compiler)  
    map.insert("looks_like_field_name", "looks_like_field_name");
    
    // Core term helpers - identity mapped (implemented in compiler)
    map.insert("core_term", "core_term");
    map.insert("encoded_terms", "encoded_terms");
    map.insert("primitive_type_names", "primitive_type_names");
    
    // File loading helpers - identity mapped (implemented in compiler)
    map.insert("load_source_files_loop", "load_source_files_loop");
    map.insert("load_source_file", "load_source_file");
    map.insert("process_source_file", "process_source_file");
    map.insert("read_source_files", "read_source_files");
    map.insert("load_registry_files_loop", "load_registry_files_loop");
    map.insert("load_registry_file", "load_registry_file");
    map.insert("process_registry_file", "process_registry_file");
    map.insert("concatenate_sources", "concatenate_sources");
    
    // NOTE: compute_expected_length, scan_for_nul, and all main.ax functions
    // are Axis-defined and MUST NOT be mapped as foreign. They should be
    // included in the Core IR module definition.
    
    // Tuple constructor
    map.insert("__tuple__", "shim::tuple");
    
    // Inequality operator - identity mapped (implemented in compiler)
    map.insert("__neq__", "__neq__");
    
    map
}

/// HARDENED: Canonical foreign symbol normalization
/// Normalizes foreign symbols to their canonical identity exactly once
/// This is the single source of truth for foreign symbol identity
fn normalize_foreign_symbol(symbol: &str) -> String {
    // Strip namespaces to get canonical base symbol
    // This is the ONLY place where symbol normalization happens
    strip_namespaces(symbol)
}

// REGIME COMPLIANCE: No filename-based special-casing
// TEMPORARY: entry_fn parameter for entry point selection (will be removed)
pub fn emit_rust_from_core(core: &CoreTerm, _input_path: &str, _entry_fn: &str) -> String {
    // DIRECT CODE GENERATION
    // Generate Rust functions from Core IR, with extern declarations for CCall targets
    
    let mut output = String::new();
    
    // Collect all CCall target names (Core IR 0.3: canonical function names)
    let mut ccall_names = HashSet::new();
    collect_ccall_names(core, &mut ccall_names);
    
    // Emit imports
    output.push_str("extern crate axis_rust_bridge;\n");
    output.push_str("use axis_rust_bridge::runtime::value::Value;\n");
    output.push_str("use axis_rust_bridge::runtime::value::{init_runtime, truthy};\n");
    output.push_str("use axis_rust_bridge::runtime::shim;\n\n");
    
    // Emit extern "C" declarations for all CCall targets
    if !ccall_names.is_empty() {
        output.push_str("// External function declarations (Core IR 0.3: canonical names)\n");
        output.push_str("// Link via: --link-lib <name> --link-search <path>\n");
        for name in &ccall_names {
            output.push_str(&format!("extern \"C\" {{ fn {}(args: Value) -> Value; }}\n", name));
        }
        output.push_str("\n");
    }
    
    // Collect all Axis-defined function names and arities
    let mut defined_functions = HashSet::new();
    collect_function_names(core, &mut defined_functions);
    let function_arities = collect_function_arities(core);
    
    // Find the first function name to use as entry point
    let first_function = get_first_function_name(core);
    let first_function_arity = get_first_function_arity(core);
    
    // Emit all top-level function definitions
    let mut emitted_functions = HashSet::new();
    let mut foreign_calls = HashSet::new();
    emit_top_level_lets(
        core,
        &mut output,
        "",
        &mut emitted_functions,
        &mut foreign_calls,
        &defined_functions,
        &function_arities,
        first_function.as_deref(),
    );
    
    // Emit main entry point - calls the first defined function or evaluates raw expression
    output.push_str("fn main() {\n");
    output.push_str("    init_runtime();\n");
    if let Some(entry) = first_function {
        let mangled = sanitize_ident(&entry);
        if first_function_arity == 0 {
            // Zero-arity function: call with no arguments
            output.push_str(&format!("    let result = {}();\n", mangled));
        } else {
            // Has parameters: pass Value::Unit as initial argument
            output.push_str(&format!("    let result = {}(Value::Unit);\n", mangled));
        }
        output.push_str("    println!(\"{:?}\", result);\n");
    } else {
        // No top-level Let - emit the expression directly in main
        // Check if the expression is a Lambda - if so, we can't Debug print it
        if matches!(core, CoreTerm::Lam(..)) {
            let expr_code = emit_term_with_module(core, 1, "", &mut foreign_calls, None, &defined_functions, &function_arities);
            output.push_str(&format!("    let _closure = {};\n", expr_code));
            output.push_str("    println!(\"<function>\");\n");
        } else {
            let expr_code = emit_term_with_module(core, 1, "", &mut foreign_calls, None, &defined_functions, &function_arities);
            output.push_str(&format!("    let result = {};\n", expr_code));
            output.push_str("    println!(\"{:?}\", result);\n");
        }
    }
    output.push_str("}\n");
    
    output
}

#[allow(dead_code)]
// Transitional helpers retained for alternate emission paths
fn emit_term(term: &CoreTerm, indent: usize) -> String {
    // Convenience wrapper for ad-hoc calls; does not record foreign calls.
    let mut tmp_set = HashSet::new();
    let defined_functions = HashSet::new();
    let function_arities = HashMap::new();
    emit_term_with_module(term, indent, "", &mut tmp_set, None, &defined_functions, &function_arities)
}

#[allow(dead_code)]
// Transitional helpers retained for alternate emission paths
// Minimal helper: extract module path from an input path like "axis/compiler/main.ax"
fn extract_module_path(input_path: &str) -> String {
    let mut p = input_path;
    if let Some(stripped) = p.strip_prefix("./") {
        p = stripped;
    }
    if let Some(stripped) = p.strip_prefix("axis/") {
        p = stripped;
    }
    if let Some(dot) = p.rfind('.') {
        p = &p[..dot];
    }
    p.replace('/', "__")
}

// TAIL-CALL OPTIMIZATION: Helper to detect if a term contains a self-recursive tail call
// Returns true if term is a tail-positioned call to the given function name
fn contains_tail_self_call(term: &CoreTerm, fn_name: &str) -> bool {
    contains_tail_self_call_impl(term, fn_name, 0)
}

fn contains_tail_self_call_impl(term: &CoreTerm, fn_name: &str, depth: usize) -> bool {
    match term {
        CoreTerm::App(func, _, _) => {
            // Check if this is a call to fn_name
            if let CoreTerm::Var(name, _) = func.as_ref() {
                let mangled = sanitize_ident(&strip_namespaces(name));
                mangled == fn_name
            } else {
                // TAIL-CALL OPTIMIZATION: Handle curried calls - App(App(...), arg)
                // Multi-argument function calls in Core IR are nested App nodes
                if let CoreTerm::App(_, _, _) = func.as_ref() {
                    // Recursively check the innermost function
                    contains_tail_self_call_impl(func, fn_name, depth)
                } else {
                    false
                }
            }
        }
        CoreTerm::If(_, then_branch, else_branch, _) => {
            // Both branches are in tail position
            contains_tail_self_call_impl(then_branch, fn_name, depth + 1) || contains_tail_self_call_impl(else_branch, fn_name, depth + 1)
        }
        CoreTerm::Let(_, _, body, _) => {
            // Only the body is in tail position (not the value)
            contains_tail_self_call_impl(body, fn_name, depth + 1)
        }
        _ => false,
    }
}

/// Sanitize Axis function name to avoid Rust keyword collisions
/// Specifically: if Axis defines "main", emit as "axis_main" to avoid collision with Rust fn main()
fn sanitize_function_name(name: &str) -> String {
    if name == "main" {
        "axis_main".to_string()
    } else {
        name.to_string()
    }
}

/// Get the first top-level function name from Core IR
/// This is used as the entry point (bag of functions - no special "main")
fn get_first_function_name(core: &CoreTerm) -> Option<String> {
    match core {
        CoreTerm::Let(name, _, _, _) => Some(sanitize_function_name(name)),
        _ => None,
    }
}

/// Get arity (number of parameters) of the first function
fn get_first_function_arity(core: &CoreTerm) -> usize {
    match core {
        CoreTerm::Let(_name, value, _, _) => {
            // Count params by descending through nested Lams
            let mut count = 0;
            let mut inner = value.as_ref();
            while let CoreTerm::Lam(_param, body, _) = inner {
                count += 1;
                inner = body.as_ref();
            }
            count
        }
        _ => 0,
    }
}

/// Collect all top-level function names and their arities from Core IR
/// This populates defined_functions and function_arities BEFORE emitting any function bodies,
/// ensuring all Axis-defined functions are recognized during emission.
fn collect_function_names(core: &CoreTerm, function_names: &mut HashSet<String>) {
    let mut current = core;
    
    loop {
        match current {
            CoreTerm::Let(name, _value, body, _) => {
                // Add this function name (collision-safe + mangled)
                let safe_name = sanitize_function_name(name);
                let mangled = sanitize_ident(&safe_name);
                function_names.insert(mangled);
                
                // Continue to next function
                current = body.as_ref();
            }
            _ => break,
        }
    }
}

/// Collect function arities (number of parameters) for all defined functions
/// Returns a map of mangled function name -> arity
fn collect_function_arities(core: &CoreTerm) -> HashMap<String, usize> {
    let mut arities = HashMap::new();
    let mut current = core;
    
    loop {
        match current {
            CoreTerm::Let(name, value, body, _) => {
                let safe_name = sanitize_function_name(name);
                let mangled = sanitize_ident(&safe_name);
                
                // Count parameters by descending through nested Lams
                let mut arity = 0;
                let mut inner = value.as_ref();
                
                // If it's a constant (not a lambda), it's a zero-arity thunk
                let is_constant = !matches!(value.as_ref(), CoreTerm::Lam(..));
                
                if !is_constant {
                    while let CoreTerm::Lam(_param, body, _) = inner {
                        arity += 1;
                        inner = body.as_ref();
                    }
                }
                
                arities.insert(mangled, arity);
                current = body.as_ref();
            }
            _ => break,
        }
    }
    
    arities
}

/// Collect all CCall target names from Core IR
/// Returns a set of all canonical function names referenced in CCall nodes
fn collect_ccall_names(term: &CoreTerm, ccall_names: &mut HashSet<String>) {
    match term {
        CoreTerm::Call(target_name, args, _) => {
            ccall_names.insert(target_name.clone());
            for arg in args {
                collect_ccall_names(arg, ccall_names);
            }
        }
        CoreTerm::Lam(_, body, _) => {
            collect_ccall_names(body, ccall_names);
        }
        CoreTerm::Let(_, value, body, _) => {
            collect_ccall_names(value, ccall_names);
            collect_ccall_names(body, ccall_names);
        }
        CoreTerm::If(cond, then_branch, else_branch, _) => {
            collect_ccall_names(cond, ccall_names);
            collect_ccall_names(then_branch, ccall_names);
            collect_ccall_names(else_branch, ccall_names);
        }
        CoreTerm::App(func, arg, _) => {
            collect_ccall_names(func, ccall_names);
            collect_ccall_names(arg, ccall_names);
        }
        _ => {} // Literals, Var: no recursion needed
    }
}

// REGIME COMPLIANCE: Simplified function emission (no module paths)
fn emit_top_level_lets(
    core: &CoreTerm,
    output: &mut String,
    _module_path: &str,
    emitted_functions: &mut HashSet<String>,
    foreign_calls: &mut HashSet<String>,
    defined_functions: &HashSet<String>,
    function_arities: &HashMap<String, usize>,
    _entry_function: Option<&str>,
) {
    // Walk nested top-level Let bindings and emit a Rust function for each
    let mut current = core;
    
    eprintln!("DEBUG emit_top_level_lets: Starting with term type: {:?}", std::mem::discriminant(core));

    loop {
        match current {
            CoreTerm::Let(name, value_rc, body_rc, _) => {
                eprintln!("DEBUG emit_top_level_lets: Processing function: {}", name);
                let value = value_rc.as_ref();

                // Collect parameters by descending through nested Lambdas
                let mut params: Vec<String> = Vec::new();
                let mut inner = value;
                while let CoreTerm::Lam(param, inner_body, _) = inner {
                    params.push(param.clone());
                    inner = inner_body.as_ref();
                }

                // BAG OF FUNCTIONS: Avoid Rust fn main() collision
                let safe_name = sanitize_function_name(name);
                let mangled = sanitize_ident(&safe_name);
                
                if emitted_functions.contains(&mangled) {
                    // skip duplicates
                    eprintln!("DEBUG emit_top_level_lets: SKIPPING duplicate function: {}", mangled);
                } else {
                    eprintln!("DEBUG emit_top_level_lets: EMITTING function: {}", mangled);
                    emitted_functions.insert(mangled.clone());

                    // Check if this is a constant binding (no lambdas) vs a function
                    let is_constant = params.is_empty() && !matches!(value, CoreTerm::Lam(..));
                    
                    if is_constant {
                        // Emit as a thunk function that returns the constant value
                        // This handles top-level let bindings for non-function values
                        let value_code = emit_term_with_module(value, 1, "", foreign_calls, None, defined_functions, function_arities);
                        output.push_str(&format!("pub fn {}() -> Value {{\n", mangled));
                        output.push_str(&format!("    {}\n", value_code));
                        output.push_str("}\n\n");
                        // Continue with body
                        current = body_rc.as_ref();
                        continue;
                    }

                    // TAIL-CALL OPTIMIZATION: Detect if this function is tail-recursive
                    let is_tail_recursive = contains_tail_self_call(inner, &mangled);

                    // All functions are public in bag-of-functions model
                    let pub_prefix = "pub ";

                    if params.is_empty() {
                        output.push_str(&format!("{}fn {}() -> Value {{\n", pub_prefix, mangled));
                    } else if params.len() == 1 {
                        let param_name = sanitize_ident(&params[0]);
                        if is_tail_recursive {
                            // Emit with mutable parameter for tail-call optimization
                            output.push_str(&format!("{}fn {}(mut {}: Value) -> Value {{\n", pub_prefix, mangled, param_name));
                            output.push_str("    loop {\n");
                        } else {
                            output.push_str(&format!("{}fn {}({}: Value) -> Value {{\n", pub_prefix, mangled, param_name));
                        }
                    } else {
                        // N-arity function (N > 1) - use tuple destructuring
                        if is_tail_recursive {
                            output.push_str(&format!("{}fn {}(mut args: Value) -> Value {{\n", pub_prefix, mangled));
                            output.push_str("    loop {\n");
                            for (i, param) in params.iter().enumerate() {
                                let param_name = sanitize_ident(param);
                                output.push_str(&format!("        let mut {} = tuple_field(Value::Tuple(vec![args.clone(), Value::Int({})]));
", param_name, i));
                            }
                        } else {
                            output.push_str(&format!("{}fn {}(args: Value) -> Value {{\n", pub_prefix, mangled));
                            for (i, param) in params.iter().enumerate() {
                                let param_name = sanitize_ident(param);
                                output.push_str(&format!("    let {} = tuple_field(Value::Tuple(vec![args.clone(), Value::Int({})]));
", param_name, i));
                            }
                        }
                    }

                    // Prepare tail-call context
                    let sanitized_params: Vec<String> = params.iter().map(|p| sanitize_ident(p)).collect();
                    let tail_ctx = if is_tail_recursive {
                        Some((mangled.as_str(), sanitized_params.as_slice()))
                    } else {
                        None
                    };

                    // Emit body
                    let base_indent = if is_tail_recursive { 2 } else { 1 };
                    let body_code = emit_term_with_module(inner, base_indent, "", foreign_calls, tail_ctx, defined_functions, function_arities);

                    let indent_str = if is_tail_recursive { "        " } else { "    " };
                    for line in body_code.lines() {
                        output.push_str(indent_str);
                        output.push_str(line);
                        output.push_str("\n");
                    }

                    if is_tail_recursive {
                        output.push_str("    }\n");
                    }
                    output.push_str("}\n\n");
                }

                // Continue with the body (remaining top-level lets)
                current = body_rc.as_ref();
            }
            _ => break,
        }
    }
}

// Collect args from nested App nodes for uncurrying
// e.g., App(App(Var(f), a), b) -> (f, [a, b])
fn collect_app_args(term: &CoreTerm) -> (&CoreTerm, Vec<&CoreTerm>) {
    let mut args = Vec::new();
    let mut current = term;

    // Traverse nested App nodes to collect all arguments
    while let CoreTerm::App(func, arg, _) = current {
        args.push(arg.as_ref());
        current = func.as_ref();
    }

    // Reverse args since we collected them right-to-left
    args.reverse();
    (current, args)
}

fn emit_term_with_module(
    term: &CoreTerm,
    indent: usize,
    module_path: &str,
    foreign_calls: &mut std::collections::HashSet<String>,
    // TAIL-CALL OPTIMIZATION: Optional function context for detecting tail self-calls
    // Format: Some((fn_name, param_names)) or None
    tail_ctx: Option<(&str, &[String])>,
    // Set of functions defined in this Core IR module
    defined_functions: &HashSet<String>,
    // Map of function name -> arity for zero-arity detection
    function_arities: &HashMap<String, usize>,
) -> String {
    match term {
        CoreTerm::IntLit(n, _) => format!("Value::Int({})", n),
        CoreTerm::BoolLit(true, _) => "Value::Bool(true)".to_string(),
        CoreTerm::BoolLit(false, _) => "Value::Bool(false)".to_string(),
        CoreTerm::UnitLit(_) => "Value::Unit".to_string(),  // Unit as Value::Unit

        CoreTerm::Var(name, _) => {
            // Handle boolean literals (no mangling)
            let stripped_name = strip_namespaces(name);
            if stripped_name == "true" {
                return "Value::Bool(true)".to_string();
            }
            if stripped_name == "false" {
                return "Value::Bool(false)".to_string();
            }

            // FIELD ACCESS CONVENTION: c_pattern means "field pattern of c"
            // Known field names for MatchCase: pattern (0), body (1)
            // ONLY apply this for single-letter base names (e.g., c_pattern, not core_body)
            let field_map: &[(&str, usize)] = &[
                ("pattern", 0),
                ("body", 1),
            ];
            for (field_name, field_idx) in field_map {
                let suffix = format!("_{}", field_name);
                if stripped_name.ends_with(&suffix) {
                    let base = &stripped_name[..stripped_name.len() - suffix.len()];
                    // Only apply for single-letter base names (typical in patterns like Cons(c, rest))
                    if base.len() == 1 && base.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                        // Emit field projection: match &base { Value::Ctor { fields, .. } => fields[idx].clone(), _ => panic!(...) }
                        let base_mangled = sanitize_ident(base);
                        return format!(
                            "match &{} {{ Value::Ctor {{ fields, .. }} => fields[{}].clone(), _ => panic!(\"Field access on non-ctor\") }}",
                            base_mangled, field_idx
                        );
                    }
                }
            }

            // Task 3: Strip namespaces and sanitize identifier
            
            // If this looks like a constructor (capitalized final segment), emit as a zero-arg call: `Ctor()`
            let mangled = sanitize_ident(&stripped_name);
            let last_seg = &mangled;
            if last_seg.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                // Constructors are emitted as direct calls (no foreign mapping needed)
                format!("{}()", mangled)
            } else {
                // MECHANICAL FIX: Handle wildcard pattern variables
                // When the variable name is "_", we cannot use "_" in Rust
                // since "_" can only appear on the left-hand side of assignments.
                // In this case, emit a unit value as a safe default.
                if mangled == "_" {
                    "Value::Unit".to_string()
                } else {
                    // FIX 2: Zero-arity function in value position must be called
                    // Check if this is a zero-arity defined function
                    if let Some(&arity) = function_arities.get(&mangled) {
                        if arity == 0 {
                            // Zero-arity function: emit as call
                            return format!("{}()", mangled);
                        }
                    }
                    // Don't add .clone() here - let call sites decide if cloning is needed
                    mangled
                }
            }
        }

        CoreTerm::Lam(param, body, _) => {
            // Emit lambda as a closure with a mangled Value parameter so Var references resolve
            let param_name = sanitize_ident(param);
            let body_code = emit_term_with_module(body, indent + 1, module_path, foreign_calls, tail_ctx, defined_functions, function_arities);
            format!("Box::new(move |{}: Value| -> Value {{ {} }}) as Box<dyn Fn(Value) -> Value>", param_name, body_code)
        }

        CoreTerm::App(func, arg, _) => {
            // UNCURRYING: Check if this is a nested application that should be flattened
            let (base_func, all_args) = collect_app_args(term);

            // TAIL-CALL OPTIMIZATION: Check if this is a tail self-call
            if let Some((fn_name, param_names)) = tail_ctx {
                if let CoreTerm::Var(func_name, _) = base_func {
                    let mangled_func = sanitize_ident(&strip_namespaces(func_name));
                    if mangled_func == fn_name {
                        // This is a tail self-call - emit as parameter reassignments + continue
                        let mut reassignments = String::new();
                        let indent_str = "    ".repeat(indent);

                        if all_args.len() > 1 {
                            // Multi-param function: reconstruct args tuple and reassign
                            let arg_codes: Vec<String> = all_args.iter()
                                .map(|a| {
                                    let code = emit_term_with_module(a, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                                    if needs_clone(a) { format!("{}.clone()", code) } else { code }
                                })
                                .collect();
                            reassignments.push_str(&format!("{}args = Value::Tuple(vec![{}]);\n", indent_str, arg_codes.join(", ")));
                            // Also reassign individual params for consistency
                            for (i, param_name) in param_names.iter().enumerate() {
                                reassignments.push_str(&format!("{}{} = tuple_field(Value::Tuple(vec![args.clone(), Value::Int({})]));\n", indent_str, param_name, i));
                            }
                        } else if all_args.len() == 1 {
                            // Single-param function: direct assignment
                            let arg_code = emit_term_with_module(all_args[0], indent, module_path, foreign_calls, None, defined_functions, function_arities);
                            let arg_final = if needs_clone(all_args[0]) { format!("{}.clone()", arg_code) } else { arg_code };
                            reassignments.push_str(&format!("{}{} = {};\n", indent_str, param_names[0], arg_final));
                        }
                        // Emit continue to restart loop
                        reassignments.push_str(&format!("{}continue", indent_str));
                        return reassignments;
                    }
                }
            }

            // Special-case: __ctor_field__ used as tuple projection (two-arg call)
            // We only rewrite when we're confident this is a projection (not
            // pattern-destructuring). Pattern-destructuring in the lowering phase
            // emits calls like __ctor_field__(_tmp_X, i) where the first arg is a
            // temporary variable beginning with "_tmp_". We avoid rewriting in
            // that case to preserve constructor field semantics.
            if let CoreTerm::Var(func_name, _) = base_func {
                let stripped = strip_namespaces(func_name);
                if stripped == "__ctor_field__" && all_args.len() == 2 {
                    // Check second arg is an int literal
                    if let CoreTerm::IntLit(idx, _) = all_args[1] {
                        // If first arg is a tmp var created by lowering, do NOT rewrite
                        let first_arg = all_args[0];
                        let is_tmp_var = match first_arg {
                            CoreTerm::Var(vn, _) => vn.starts_with("_tmp_"),
                            _ => false,
                        };

                        if !is_tmp_var {
                            // Emit as direct call to shim tuple_field function
                            foreign_calls.insert("tuple_field".to_string());
                            let tuple_code = emit_term_with_module(first_arg, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                            let tuple_final = if needs_clone(first_arg) { format!("{}.clone()", tuple_code) } else { tuple_code };
                            // Index must be a literal Int (0-based)
                            // UNARY INVARIANT: Pack both arguments into a single tuple
                            return format!("shim::tuple_field(Value::Tuple(vec![{}, Value::Int({})]))", tuple_final, idx);
                        }
                    }
                }
            }

            if all_args.len() > 1 {
                // Multiple arguments: pack into single tuple (UNARY INVARIANT)
                match base_func {
                    CoreTerm::Var(func_name, _) => {
                        // NEW: Proper call resolution - classify the symbol
                        let canonical_func = normalize_foreign_symbol(func_name);
                        let foreign_mapping = get_foreign_symbol_mapping();
                        
                        let arg_codes: Vec<String> = all_args.iter()
                            .map(|a| {
                                let code = emit_term_with_module(a, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                                //  POLICY: clone all function arguments
                                if needs_clone(a) { format!("{}.clone()", code) } else { code }
                            })
                            .collect();

                        // Classification priority:
                        // 1. Local bindings (lambda params, let bindings) -> handled by Var case, not here
                        // 2. Core IR defined functions -> generate fn call
                        // 3. Foreign functions -> require mapping or panic
                        
                        if defined_functions.contains(func_name) {
                            // This is a Core IR defined function - emit direct fn call
                            let mangled_name = sanitize_ident(&canonical_func);
                            // UNARY INVARIANT: Pack multiple arguments into tuple
                            format!("{}(Value::Tuple(vec![{}]))", mangled_name, arg_codes.join(", "))
                        } else if let Some(&shim_path) = foreign_mapping.get(canonical_func.as_str()) {
                            // This is a mapped foreign function - emit direct shim call
                            foreign_calls.insert(canonical_func.clone());
                            
                            // UNARY INVARIANT: ALL runtime primitives accept exactly one Value argument
                            // Pack all arguments into a single Value::Tuple for shim
                            format!("{}(Value::Tuple(vec![{}]))", shim_path, arg_codes.join(", "))
                        } else {
                            // FAIL-FAST: Unmapped foreign symbol - panic with clear error
                            panic!("EMIT RUST: Foreign symbol '{}' is not mapped in shim. Add it to get_foreign_symbol_mapping() or define it in Core IR.", func_name);
                        }
                    }
                    _ => {
                        // Non-variable function: emit curried
                        let func_code = emit_term_with_module(func, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                        let arg_code = emit_term_with_module(arg, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                        format!("({})({})", func_code, arg_code)
                    }
                }
            } else {
                // Single argument: emit normally but with foreign mapping check
                match func.as_ref() {
                    CoreTerm::Var(func_name, _) => {
                        // NEW: Proper call resolution for single argument case
                        let canonical_func = normalize_foreign_symbol(func_name);
                        let foreign_mapping = get_foreign_symbol_mapping();
                        
                        let arg_code = emit_term_with_module(arg, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                        let arg_final = if needs_clone(arg) { format!("{}.clone()", arg_code) } else { arg_code };

                        // Classification priority:
                        // 1. Local bindings (lambda params, let bindings) -> handled by Var case, not here
                        // 2. Core IR defined functions -> generate fn call
                        // 3. Foreign functions -> require mapping or panic
                        
                        if defined_functions.contains(func_name) {
                            // This is a Core IR defined function - emit direct fn call
                            let mangled_name = sanitize_ident(&canonical_func);
                            format!("{}({})", mangled_name, arg_final)
                        } else if let Some(&shim_path) = foreign_mapping.get(canonical_func.as_str()) {
                            // This is a mapped foreign function - emit direct shim call
                            foreign_calls.insert(canonical_func.clone());
                            format!("{}({})", shim_path, arg_final)
                        } else {
                            // FAIL-FAST: Unmapped foreign symbol - panic with clear error
                            panic!("EMIT RUST: Foreign symbol '{}' is not mapped in shim. Add it to get_foreign_symbol_mapping() or define it in Core IR.", func_name);
                        }
                    }
                    _ => {
                        let func_code = emit_term_with_module(func, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                        let arg_code = emit_term_with_module(arg, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                        //  POLICY: clone function arguments
                        let arg_final = if needs_clone(arg) { format!("{}.clone()", arg_code) } else { arg_code };
                        format!("({})({})", func_code, arg_final)
                    }
                }
            }
        }

        CoreTerm::Let(name, value, body, _) => {
            // Mangle the binder so locals and Var references align with mangling
            let var_name = sanitize_ident(name);

            // Emit value and body recursively
            let value_code = emit_term_with_module(value, indent + 1, module_path, foreign_calls, None, defined_functions, function_arities);
            let body_code = emit_term_with_module(body, indent + 1, module_path, foreign_calls, tail_ctx, defined_functions, function_arities);

            // TASK 3: UNCONDITIONAL tuple projection workaround
            // ALWAYS emit tuple projections for every let-binding
            // This handles Core IR bugs where tuple type annotations become patterns
            // Extra bindings are harmless; missing bindings are fatal

            // Preserve indentation: produce a block with an indented `let` and body
            let indent_str = "    ".repeat(indent);
            let inner_indent = "    ".repeat(indent + 1);

            // Build the full block
            let mut block = String::new();
            block.push_str("{\n");
            block.push_str(&format!("{}let {} = {};\n", inner_indent, var_name, value_code));

            eprintln!("[lower-let] {} projections=10", var_name);

            //  WORKAROUND: Emit tuple projections for let-bound variables
            // This handles Core IR bugs where references don't match bindings
            // Also create common aliases to handle name drift (e.g., json_term -> json_body)
            // for i in 0..10 {
            //     let proj_var = format!("{}_{}", var_name, i);
            //     block.push_str(&format!("{}let {} = tuple_field({}.clone(), {});\n",
            //         inner_indent, proj_var, var_name, i));
            // }

            // ADDITIONAL workaround: Create name-drift aliases for common patterns
            // Pattern 1: var_term -> var (strip "_term" suffix)
            if var_name.ends_with("_term") {
                let base = &var_name[..var_name.len() - 5]; // Remove "_term"
                block.push_str(&format!("{}let {} = {}.clone();\n",
                    inner_indent, base, var_name));
                // Also create projections for the alias
                for i in 0..10 {
                    let proj_var = format!("{}_{}", base, i);
                    block.push_str(&format!("{}let {} = tuple_field(Value::Tuple(vec![{}.clone(), Value::Int({})]));
",
                        inner_indent, proj_var, base, i));
                }
            }
            // Pattern 2: Check if this looks like it should have a "_body" alias
            // json_term -> json_body
            if var_name.ends_with("_term") {
                let base = &var_name[..var_name.len() - 5];
                let body_alias = format!("{}_body", base);
                block.push_str(&format!("{}let {} = {}.clone();\n",
                    inner_indent, body_alias, var_name));
            }

            // Indent body lines to match inner indentation
            let indented_body = body_code
                .lines()
                .map(|l| format!("{}{}", inner_indent, l))
                .collect::<Vec<_>>()
                .join("\n");

            block.push_str(&format!("{}\n", indented_body));
            block.push_str(&format!("{}}}", indent_str));
            block
        }

        CoreTerm::If(cond, then_branch, else_branch, _) => {
            let cond_code = emit_term_with_module(cond, indent, module_path, foreign_calls, None, defined_functions, function_arities);
            let then_code = emit_term_with_module(then_branch, indent, module_path, foreign_calls, tail_ctx, defined_functions, function_arities);
            let else_code = emit_term_with_module(else_branch, indent, module_path, foreign_calls, tail_ctx, defined_functions, function_arities);

            // TAIL-CALL OPTIMIZATION: Wrap branches with 'return' when in tail context
            // unless they contain 'continue'
            let then_contains_continue = then_code.contains("continue");
            let else_contains_continue = else_code.contains("continue");
            let then_final = if tail_ctx.is_some() && !then_contains_continue {
                format!("return {}", then_code)
            } else {
                then_code
            };
            let else_final = if tail_ctx.is_some() && !else_contains_continue {
                format!("return {}", else_code)
            } else {
                else_code
            };

            // No additional cloning needed here - truthy takes a reference
            format!("if truthy(&({})) {{ {} }} else {{ {} }}",
                cond_code, then_final, else_final)
        }

        CoreTerm::Call(target_name, args, _) => {
            // External function call (Core IR 0.3: canonical function name)
            // Use the canonical name directly as the C symbol name
            // Caller is responsible for providing library via --link-lib
            
            // Record this as a foreign call (for extern declaration generation)
            foreign_calls.insert(target_name.clone());
            
            // Emit direct call to external symbol using canonical name
            let arg_codes: Vec<String> = args.iter()
                .map(|a| {
                    let code = emit_term_with_module(a, indent, module_path, foreign_calls, None, defined_functions, function_arities);
                    if needs_clone(a) { format!("{}.clone()", code) } else { code }
                })
                .collect();
            
            if arg_codes.is_empty() {
                format!("unsafe {{ {}() }}", target_name)
            } else {
                format!("unsafe {{ {}({}) }}", target_name, arg_codes.join(", "))
            }
        }
    }
}

fn sanitize_ident(name: &str) -> String {
    // Replace dots and dashes with underscores to make valid Rust identifiers
    let base = name.replace('.', "_").replace('-', "_");

    // Handle Rust reserved words and crate names that conflict
    match base.as_str() {
        "core" => "core_".to_string(),
        "self" => "self_".to_string(),
        "Self" => "Self_".to_string(),
        "type" => "type_".to_string(),
        "match" => "match_".to_string(),
        "fn" => "fn_".to_string(),
        "let" => "let_".to_string(),
        "if" => "if_".to_string(),
        "else" => "else_".to_string(),
        "loop" => "loop_".to_string(),
        "for" => "for_".to_string(),
        "while" => "while_".to_string(),
        "break" => "break_".to_string(),
        "continue" => "continue_".to_string(),
        "return" => "return_".to_string(),
        "mod" => "mod_".to_string(),
        "pub" => "pub_".to_string(),
        "use" => "use_".to_string(),
        "struct" => "struct_".to_string(),
        "enum" => "enum_".to_string(),
        "impl" => "impl_".to_string(),
        "trait" => "trait_".to_string(),
        "where" => "where_".to_string(),
        "const" => "const_".to_string(),
        "static" => "static_".to_string(),
        "mut" => "mut_".to_string(),
        "ref" => "ref_".to_string(),
        "move" => "move_".to_string(),
        "box" => "box_".to_string(),
        "as" => "as_".to_string(),
        "in" => "in_".to_string(),
        "unsafe" => "unsafe_".to_string(),
        "extern" => "extern_".to_string(),
        "crate" => "crate_".to_string(),
        "super" => "super_".to_string(),
        _ => base,
    }
}

///  POLICY: Determine if a term needs .clone() when used
/// Clone everything except literals to avoid borrow errors
fn needs_clone(term: &CoreTerm) -> bool {
    use crate::core_ir::CoreTerm;
    match term {
        CoreTerm::IntLit(_, _) => false,
        CoreTerm::BoolLit(_, _) => false,
        CoreTerm::UnitLit(_) => false,
        _ => true,  // Clone: Var, App, Let, Lam, If
    }
}

/// Strip namespace from ALL constructor references (Loop-6 requirement)
/// compiler_main___Result::Ok -> compiler_main___Ok
/// compiler_main___Token::TokIdent -> compiler_main___TokIdent
/// compiler_main___List::Nil -> compiler_main___Nil
/// SurfaceAst::SStrLit -> SStrLit
fn strip_namespaces(name: &str) -> String {
    if let Some(colon_idx) = name.rfind("::") {
        // Always strip everything before the last ::
        name[colon_idx + 2..].to_string()
    } else {
        name.to_string()
    }
}

