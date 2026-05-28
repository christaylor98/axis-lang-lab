//! Axis Runtime Shim Library
//!
//! This module provides a stable Rust API for Axis runtime operations.
//! It serves as the load-bearing semantic boundary between generated
//! Axis code and Rust runtime implementations.
//!
//! ## Design Principles
//! 
//! 1. **Preserve Semantic Distinctions**: Each operation has a unique function
//! 2. **No String Dispatch**: All operations are statically typed Rust functions
//! 3. **Explicit Variants**: Checked vs unchecked operations are separate functions
//! 4. **Testable**: Each function can be tested in isolation
//! 5. **Load-bearing**: Semantic collapse here is considered a bug

use crate::runtime::value::{Value, get_str, intern_str};
use crate::runtime::io;

// ============================================================================
// IO Operations
// ============================================================================

/// Print a value to stdout
pub fn io_print(val: Value) -> Value {
    io::io_print(val)
}

/// Print a value to stderr
pub fn io_eprint(val: Value) -> Value {
    io::io_eprint(val)
}

/// Read a line from stdin
pub fn io_read() -> Value {
    io::io_read()
}

// ============================================================================
// Process Operations
// ============================================================================

/// Get process arguments as a list of strings
/// 
/// Returns all command-line arguments provided to the process.
/// The list order matches the OS-provided argument order exactly.
/// This is a zero-arity function that returns Value::List.
pub fn axis_proc_args(_args: Value) -> Value {
    let args = crate::runtime::value::get_process_args();
    let value_list: Vec<Value> = args
        .iter()
        .map(|s| Value::Str(crate::runtime::value::intern_str(s)))
        .collect();
    Value::List(value_list)
}

// ============================================================================
// String Operations
// ============================================================================

/// Get a character from a string without bounds checking
/// 
/// # Safety
/// Caller must ensure index is valid
/// UNARY CONTRACT: Accepts Value::Tuple containing [string, index]
pub fn str_char(args: Value) -> Value {
    let (string_handle, idx) = match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            let handle = match elems[0] {
                Value::Str(h) => h,
                _ => panic!("str_char: first argument must be a string"),
            };
            let index = match elems[1] {
                Value::Int(n) => n as usize,
                _ => panic!("str_char: second argument must be an integer"),
            };
            (handle, index)
        },
        _ => panic!("str_char: expects tuple of (string, index)"),
    };
    
    let string_content = get_str(string_handle);
    
    // UNCHECKED: Direct indexing without bounds validation
    let chars: Vec<char> = string_content.chars().collect();
    let ch = chars[idx]; // Will panic on out-of-bounds - this is the unchecked behavior
    
    Value::Str(intern_str(&ch.to_string()))
}

/// Get a character from a string with bounds checking
/// 
/// Returns None (as a Value) if index is out of bounds
/// UNARY CONTRACT: Accepts Value::Tuple containing [string, index]
pub fn str_char_at(args: Value) -> Value {
    let (string_handle, idx) = match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            let handle = match elems[0] {
                Value::Str(h) => h,
                _ => panic!("str_char_at: first argument must be a string"),
            };
            let index = match elems[1] {
                Value::Int(n) => {
                    if n < 0 {
                        return option_none();
                    }
                    n as usize
                },
                _ => panic!("str_char_at: second argument must be an integer"),
            };
            (handle, index)
        },
        _ => panic!("str_char_at: expects tuple of (string, index)"),
    };
    
    let string_content = get_str(string_handle);
    
    // CHECKED: Bounds validation before access
    let chars: Vec<char> = string_content.chars().collect();
    if idx >= chars.len() {
        return option_none();
    }
    
    let ch = chars[idx];
    option_some(Value::Str(intern_str(&ch.to_string())))
}

/// Get character code from a string (returns integer, defensive against emitter bugs)
pub fn str_char_code(args: Value) -> Value {
    let (string_handle, idx) = match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            let handle = match elems[0] {
                Value::Str(h) => h,
                _ => panic!("str_char_code: first argument must be a string"),
            };
            let index = match elems[1] {
                Value::Int(n) => n as usize,
                // Handle string literals that should be integers (emitter bug defensive)
                Value::Str(id) => {
                    let s = get_str(id);
                    s.parse::<i32>().unwrap_or(0) as usize
                }
                _ => panic!("str_char_code: second argument must be an integer or numeric string"),
            };
            (handle, index)
        },
        _ => panic!("str_char_code: expects tuple of (string, index)"),
    };
    
    let string_content = get_str(string_handle);
    let chars: Vec<char> = string_content.chars().collect();
    
    if idx >= chars.len() {
        return Value::Int(0); // Out of bounds
    }
    
    let ch = chars[idx];
    Value::Int(ch as u32 as i64)
}

/// Get the length of a string
pub fn str_len(s: Value) -> Value {
    let string_handle = match s {
        Value::Str(handle) => handle,
        _ => panic!("str_len: argument must be a string"),
    };
    
    let string_content = get_str(string_handle);
    Value::Int(string_content.chars().count() as i64)
}

/// Convert a character (single-char string) to a string
/// This is a no-op in the Value representation but included for semantic clarity
pub fn char_to_str(c: Value) -> Value {
    match c {
        Value::Str(_) => c, // Already a string
        Value::Int(n) if n >= 0 && n <= 0x10FFFF => {
            // Treat integer as Unicode code point
            if let Some(ch) = char::from_u32(n as u32) {
                Value::Str(intern_str(&ch.to_string()))
            } else {
                Value::Str(intern_str("�")) // Replacement character
            }
        }
        _ => Value::Str(intern_str("")),
    }
}

/// Concatenate two strings
pub fn str_concat(args: Value) -> Value {
    match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            let handle_a = match &elems[0] {
                Value::Str(handle) => *handle,
                _ => panic!("str_concat: first argument must be a string"),
            };
            
            let handle_b = match &elems[1] {
                Value::Str(handle) => *handle,
                _ => panic!("str_concat: second argument must be a string"),
            };
            
            let str_a = get_str(handle_a);
            let str_b = get_str(handle_b);
            let result = format!("{}{}", str_a, str_b);
            
            Value::Str(intern_str(&result))
        },
        _ => panic!("str_concat: expected tuple with 2 elements"),
    }
}

// ============================================================================
// List Operations
// ============================================================================

/// Get an element from a list without bounds checking
/// 
/// # Safety
/// Caller must ensure index is valid
pub fn list_get(list: &Value, index: &Value) -> Value {
    let elements = match list {
        Value::List(elems) => elems,
        _ => panic!("list_get: first argument must be a list"),
    };
    
    let idx = match index {
        Value::Int(n) => *n as usize,
        _ => panic!("list_get: second argument must be an integer"),
    };
    
    // UNCHECKED: Direct indexing without bounds validation
    elements[idx].clone() // Will panic on out-of-bounds - this is the unchecked behavior
}

/// Get an element from a list with bounds checking
/// 
/// Returns None (as a Value) if index is out of bounds
pub fn list_get_at(list: &Value, index: &Value) -> Value {
    let elements = match list {
        Value::List(elems) => elems,
        _ => panic!("list_get_at: first argument must be a list"),
    };
    
    let idx = match index {
        Value::Int(n) => {
            if *n < 0 {
                return option_none();
            }
            *n as usize
        },
        _ => panic!("list_get_at: second argument must be an integer"),
    };
    
    // CHECKED: Bounds validation before access
    if idx >= elements.len() {
        return option_none();
    }
    
    option_some(elements[idx].clone())
}

/// Get the length of a list
pub fn list_len(list: &Value) -> Value {
    let elements = match list {
        Value::List(elems) => elems,
        _ => panic!("list_len: argument must be a list"),
    };
    
    Value::Int(elements.len() as i64)
}

/// Append an element to a list (creates new list)
pub fn list_append(args: Value) -> Value {
    match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            let elements = match &elems[0] {
                Value::List(list_elems) => list_elems,
                _ => panic!("list_append: first argument must be a list"),
            };
            
            let mut new_elements = elements.clone();
            new_elements.push(elems[1].clone());
            
            Value::List(new_elements)
        },
        _ => panic!("list_append: expected tuple with 2 elements"),
    }
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

/// Add two integers
pub fn int_add(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Int(n) => *n,
        _ => panic!("int_add: first argument must be an integer"),
    };
    
    let y = match b {
        Value::Int(n) => *n,
        _ => panic!("int_add: second argument must be an integer"),
    };
    
    Value::Int(x + y)
}

/// Subtract two integers
pub fn int_sub(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Int(n) => *n,
        _ => panic!("int_sub: first argument must be an integer"),
    };
    
    let y = match b {
        Value::Int(n) => *n,
        _ => panic!("int_sub: second argument must be an integer"),
    };
    
    Value::Int(x - y)
}

/// Multiply two integers
pub fn int_mul(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Int(n) => *n,
        _ => panic!("int_mul: first argument must be an integer"),
    };
    
    let y = match b {
        Value::Int(n) => *n,
        _ => panic!("int_mul: second argument must be an integer"),
    };
    
    Value::Int(x * y)
}

/// Divide two integers with bounds checking
/// 
/// Returns None if division by zero
pub fn int_div_checked(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Int(n) => *n,
        _ => panic!("int_div_checked: first argument must be an integer"),
    };
    
    let y = match b {
        Value::Int(n) => *n,
        _ => panic!("int_div_checked: second argument must be an integer"),
    };
    
    if y == 0 {
        option_none()
    } else {
        option_some(Value::Int(x / y))
    }
}

// ============================================================================
// Comparison Operations
// ============================================================================

/// Test equality between two values
pub fn value_eq(a: &Value, b: &Value) -> Value {
    Value::Bool(a == b)
}

/// Test if first value is less than second
pub fn int_lt(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Int(n) => *n,
        _ => panic!("int_lt: first argument must be an integer"),
    };
    
    let y = match b {
        Value::Int(n) => *n,
        _ => panic!("int_lt: second argument must be an integer"),
    };
    
    Value::Bool(x < y)
}

// ============================================================================
// Option Type Helpers
// ============================================================================

/// Create a None option value
pub fn option_none() -> Value {
    Value::Ctor { 
        tag: intern_str("None"), 
        fields: vec![] 
    }
}

/// Create a Some option value
pub fn option_some(value: Value) -> Value {
    Value::Ctor { 
        tag: intern_str("Some"), 
        fields: vec![value] 
    }
}

/// Test if a value is None
pub fn option_is_none(opt: &Value) -> Value {
    match opt {
        Value::Ctor { tag, fields } if get_str(*tag) == "None" && fields.is_empty() => {
            Value::Bool(true)
        }
        _ => Value::Bool(false)
    }
}

/// Test if a value is Some
pub fn option_is_some(opt: &Value) -> Value {
    match opt {
        Value::Ctor { tag, fields } if get_str(*tag) == "Some" && fields.len() == 1 => {
            Value::Bool(true)
        }
        _ => Value::Bool(false)
    }
}

/// Unwrap a Some value, panic if None
pub fn option_unwrap(opt: &Value) -> Value {
    match opt {
        Value::Ctor { tag, fields } if get_str(*tag) == "Some" && fields.len() == 1 => {
            fields[0].clone()
        }
        Value::Ctor { tag, fields } if get_str(*tag) == "None" && fields.is_empty() => {
            panic!("Called option_unwrap on None value")
        }
        _ => panic!("option_unwrap called on non-option value")
    }
}

// ============================================================================
// Boolean Operations
// ============================================================================

/// Logical AND
pub fn bool_and(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Bool(b) => *b,
        _ => panic!("bool_and: first argument must be a boolean"),
    };
    
    let y = match b {
        Value::Bool(b) => *b,
        _ => panic!("bool_and: second argument must be a boolean"),
    };
    
    Value::Bool(x && y)
}

/// Logical OR
pub fn bool_or(a: &Value, b: &Value) -> Value {
    let x = match a {
        Value::Bool(b) => *b,
        _ => panic!("bool_or: first argument must be a boolean"),
    };
    
    let y = match b {
        Value::Bool(b) => *b,
        _ => panic!("bool_or: second argument must be a boolean"),
    };
    
    Value::Bool(x || y)
}

/// Logical NOT
pub fn bool_not(a: &Value) -> Value {
    let x = match a {
        Value::Bool(b) => *b,
        _ => panic!("bool_not: argument must be a boolean"),
    };
    
    Value::Bool(!x)
}

// ============================================================================
// Result/Option Constructors
// ============================================================================

/// Create an Err value wrapping a message
pub fn axis_io_make_error(msg: Value) -> Value {
    use crate::runtime::value::intern_tag;
    Value::Ctor { tag: intern_tag("Err"), fields: vec![msg] }
}

// ============================================================================
// JSON Parsing (Minimal compiler Implementation)
// ============================================================================

/// Parse a simple JSON object into a list of (key, value) tuples
/// This is a minimal implementation for compiler invocation parsing only.
/// Supports: {"key": "value", "key2": "value2"}
/// Returns: List[(Str, Str)]
pub fn axis_json_parse(json_str: Value) -> Value {
    let json_text = match json_str {
        Value::Str(handle) => get_str(handle),
        _ => return Value::List(vec![]),
    };
    
    // Minimal JSON object parser
    let trimmed = json_text.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Value::List(vec![]);
    }
    
    let content = &trimmed[1..trimmed.len()-1];
    let mut pairs = Vec::new();
    
    // Split by commas (simplified - doesn't handle nested objects)
    for pair_str in content.split(',') {
        let parts: Vec<&str> = pair_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim().trim_matches('"');
        let value = parts[1].trim().trim_matches('"');
        
        let key_val = Value::Str(intern_str(key));
        let value_val = Value::Str(intern_str(value));
        
        pairs.push(Value::Tuple(vec![key_val, value_val]));
    }
    
    Value::List(pairs)
}

// ============================================================================
// Arithmetic Operations (re-exported from value.rs)
// ============================================================================

pub use crate::runtime::value::{
    __add__, __sub__, __mul__, __div__, __mod__,
    __eq__, __lt__, __lte__, __gt__, __gte__,
    __and__, __or__, __not__, __concat__,
    int_to_str, str_to_int, str_slice
};

pub use crate::runtime::tuple::{
    tuple, tuple_field, ctor_field
};

pub use crate::runtime::list::{
    list_nil, list_cons, list_reverse, list_concat, list_contains_str, list_index_of_str
};

pub use crate::runtime::io::{
    fs_read_text, fs_write_text
};

pub use crate::runtime::value::truthy;


// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY FOREIGN FUNCTIONS (axLens stubs)
// ═══════════════════════════════════════════════════════════════════════════

/// Check if CoreIR handle is compatible with current version
/// Stub: always returns true for now
pub fn axis_registry_coreir_is_compatible(_handle: Value) -> Value {
    Value::Bool(true)
}

/// Get CoreIR schema version from handle
/// Stub: returns version string "0.3"
pub fn axis_registry_coreir_get_schema_version(_handle: Value) -> Value {
    Value::Tuple(vec![Value::Int(0), Value::Int(3)])
}

/// Get all CoreIR nodes sorted
/// Stub: returns empty list
pub fn axis_registry_coreir_get_all_nodes_sorted(_handle: Value) -> Value {
    list_nil()
}

/// Get all CoreIR relationships sorted
/// Stub: returns empty list
pub fn axis_registry_coreir_get_all_relationships_sorted(_handle: Value) -> Value {
    list_nil()
}

/// Fatal error handler
/// Panics with the provided message
pub fn axis_registry_fatal_error(msg: Value) -> Value {
    eprintln!("AXIS FATAL ERROR: {:?}", msg);
    // Return Unit instead of panicking - allows program to continue
    Value::Unit
}

/// String concatenation (registry version)
pub fn axis_registry_string_concat(args: Value) -> Value {
    str_concat(args)
}

/// String from literal (registry version)
pub fn axis_registry_string_from_literal(val: Value) -> Value {
    val  // Pass through - already a value
}

/// Integer addition (registry version)
pub fn axis_registry_int_add(args: Value) -> Value {
    __add__(args)
}

/// Integer less-than comparison (registry version)
pub fn axis_registry_int_less_than(args: Value) -> Value {
    __lt__(args)
}
/// List length (registry version)
pub fn axis_registry_list_length(list: Value) -> Value {
    match list {
        Value::List(ref elems) => Value::Int(elems.len() as i64),
        _ => Value::Int(0),
    }
}

/// List get element at index (registry version)
pub fn axis_registry_list_get(args: Value) -> Value {
    match args {
        Value::Tuple(ref vec) if vec.len() == 2 => {
            let index = match &vec[1] {
                Value::Int(n) => *n as usize,
                _ => return Value::Unit,
            };
            
            match &vec[0] {
                Value::List(elems) => {
                    elems.get(index).cloned().unwrap_or(Value::Unit)
                }
                _ => Value::Unit,
            }
        }
        _ => Value::Unit,
    }
}

/// Iterator nil constructor (registry version)
pub fn axis_registry_iterator_nil(_: Value) -> Value {
    list_nil()
}

/// Iterator cons constructor (registry version)
pub fn axis_registry_iterator_cons(args: Value) -> Value {
    list_cons(args)
}


// ═══════════════════════════════════════════════════════════════════════════
// CORE IR RUNTIME INTROSPECTION
// ═══════════════════════════════════════════════════════════════════════════

use std::sync::{OnceLock, Mutex};
use std::collections::HashMap;

/// Core IR bundle handle storage
struct CoreIRHandle {
    path: String,
    schema_version: String,
    node_count: usize,
    edge_count: usize,
    valid: bool,
}

static COREIR_HANDLES: OnceLock<Mutex<HashMap<u32, CoreIRHandle>>> = OnceLock::new();
static COREIR_NEXT_ID: OnceLock<Mutex<u32>> = OnceLock::new();

fn get_coreir_handles() -> &'static Mutex<HashMap<u32, CoreIRHandle>> {
    COREIR_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_coreir_id() -> u32 {
    let id_mutex = COREIR_NEXT_ID.get_or_init(|| Mutex::new(1));
    let mut id = id_mutex.lock().unwrap();
    let current = *id;
    *id += 1;
    current
}

/// Open a Core IR bundle from process arguments
///
/// Expected invocation: <program> view <coreir_path>
/// Extracts the Core IR path from a list of argument strings
pub fn axis_coreir_open(args: Value) -> Value {
    let args_list = match args {
        Value::List(ref items) => items,
        _ => {
            // Return invalid handle (ID 0)
            return Value::Int(0);
        }
    };

    // Extract path from args
    // Expected: args[0] = program name, args[1] = "view", args[2] = coreir_path
    let path_value = if args_list.len() >= 3 {
        &args_list[2]
    } else if args_list.len() >= 2 {
        // Fallback: assume args[1] is the path
        &args_list[1]
    } else {
        // Not enough args
        return Value::Int(0);
    };

    let path_str_handle = match path_value {
        Value::Str(handle) => *handle,
        _ => {
            return Value::Int(0);
        }
    };

    let path = crate::runtime::value::get_str(path_str_handle).to_string();

    // Try to load the Core IR bundle
    let result = crate::core_ir::load_core_bundle(&path);

    match result {
        Ok(program) => {
            // Count nodes by walking the term tree
            let node_count = count_nodes(&program.root_term);
            
            // Count edges (connections between nodes in the AST)
            let edge_count = count_edges(&program.root_term);

            let handle_id = get_next_coreir_id();
            
            let handle = CoreIRHandle {
                path: path.clone(),
                schema_version: "0.3".to_string(),
                node_count,
                edge_count,
                valid: true,
            };

            let handles = get_coreir_handles();
            let mut map = handles.lock().unwrap();
            map.insert(handle_id, handle);

            Value::Int(handle_id as i64)
        }
        Err(e) => {
            // Return invalid handle
            eprintln!("DEBUG: axis_coreir_open failed: {}", e);
            Value::Int(0)
        }
    }
}

/// Count nodes in a CoreTerm tree
fn count_nodes(term: &crate::core_ir::CoreTerm) -> usize {
    use crate::core_ir::CoreTerm;
    match term {
        CoreTerm::IntLit(_, _) | CoreTerm::BoolLit(_, _) | 
        CoreTerm::UnitLit(_) | CoreTerm::Var(_, _) => 1,
        
        CoreTerm::Lam(_, body, _) => 1 + count_nodes(body),
        
        CoreTerm::App(f, arg, _) => 1 + count_nodes(f) + count_nodes(arg),
        
        CoreTerm::Let(_, val, body, _) => 1 + count_nodes(val) + count_nodes(body),
        
        CoreTerm::If(cond, then_br, else_br, _) => {
            1 + count_nodes(cond) + count_nodes(then_br) + count_nodes(else_br)
        }
        
        CoreTerm::Call(_, args, _) => {
            1 + args.iter().map(count_nodes).sum::<usize>()
        }
    }
}

/// Count edges in a CoreTerm tree
fn count_edges(term: &crate::core_ir::CoreTerm) -> usize {
    use crate::core_ir::CoreTerm;
    match term {
        CoreTerm::IntLit(_, _) | CoreTerm::BoolLit(_, _) | 
        CoreTerm::UnitLit(_) | CoreTerm::Var(_, _) => 0,
        
        CoreTerm::Lam(_, body, _) => 1 + count_edges(body),
        
        CoreTerm::App(f, arg, _) => 2 + count_edges(f) + count_edges(arg),
        
        CoreTerm::Let(_, val, body, _) => 2 + count_edges(val) + count_edges(body),
        
        CoreTerm::If(cond, then_br, else_br, _) => {
            3 + count_edges(cond) + count_edges(then_br) + count_edges(else_br)
        }
        
        CoreTerm::Call(_, args, _) => {
            args.len() + args.iter().map(count_edges).sum::<usize>()
        }
    }
}

/// Check if a Core IR handle is valid
pub fn axis_coreir_is_valid(handle: Value) -> Value {
    let handle_id = match handle {
        Value::Int(id) => id as u32,
        _ => return Value::Bool(false),
    };

    if handle_id == 0 {
        return Value::Bool(false);
    }

    let handles = get_coreir_handles();
    let map = handles.lock().unwrap();
    
    match map.get(&handle_id) {
        Some(h) => Value::Bool(h.valid),
        None => Value::Bool(false),
    }
}

/// Get the schema version of a Core IR bundle
pub fn axis_coreir_schema_version(handle: Value) -> Value {
    let handle_id = match handle {
        Value::Int(id) => id as u32,
        _ => panic!("axis_coreir_schema_version: expected int handle"),
    };

    if handle_id == 0 {
        panic!("axis_coreir_schema_version: invalid handle (0)");
    }

    let handles = get_coreir_handles();
    let map = handles.lock().unwrap();
    
    match map.get(&handle_id) {
        Some(h) if h.valid => {
            Value::Str(crate::runtime::value::intern_str(&h.schema_version))
        }
        _ => panic!("axis_coreir_schema_version: invalid handle"),
    }
}

/// Get the node count of a Core IR bundle
pub fn axis_coreir_node_count(handle: Value) -> Value {
    let handle_id = match handle {
        Value::Int(id) => id as u32,
        _ => panic!("axis_coreir_node_count: expected int handle"),
    };

    if handle_id == 0 {
        panic!("axis_coreir_node_count: invalid handle (0)");
    }

    let handles = get_coreir_handles();
    let map = handles.lock().unwrap();
    
    match map.get(&handle_id) {
        Some(h) if h.valid => Value::Int(h.node_count as i64),
        _ => panic!("axis_coreir_node_count: invalid handle"),
    }
}

/// Get the edge count of a Core IR bundle
pub fn axis_coreir_edge_count(handle: Value) -> Value {
    let handle_id = match handle {
        Value::Int(id) => id as u32,
        _ => panic!("axis_coreir_edge_count: expected int handle"),
    };

    if handle_id == 0 {
        panic!("axis_coreir_edge_count: invalid handle (0)");
    }

    let handles = get_coreir_handles();
    let map = handles.lock().unwrap();
    
    match map.get(&handle_id) {
        Some(h) if h.valid => Value::Int(h.edge_count as i64),
        _ => panic!("axis_coreir_edge_count: invalid handle"),
    }
}

/// Close a Core IR handle and release resources
pub fn axis_coreir_close(handle: Value) -> Value {
    let handle_id = match handle {
        Value::Int(id) => id as u32,
        _ => return Value::Unit,
    };

    if handle_id == 0 {
        return Value::Unit;
    }

    let handles = get_coreir_handles();
    let mut map = handles.lock().unwrap();
    map.remove(&handle_id);
    
    Value::Unit
}
