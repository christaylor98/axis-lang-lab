use std::fs;
use std::rc::Rc;

/// Lightweight program container returned by the bridge loader
pub struct CoreProgram {
    pub root_term: CoreTerm,
    pub entrypoint_id: usize,
}

#[derive(Clone, Debug)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

// In-memory CoreTerm shape used by the emitter logic (matches Core IR 0.2 schema)
#[derive(Clone, Debug)]
pub enum CoreTerm {
    IntLit(i64, Option<Span>),
    BoolLit(bool, Option<Span>),
    UnitLit(Option<Span>),
    Var(String, Option<Span>),
    Lam(String, Rc<CoreTerm>, Option<Span>),
    Let(String, Rc<CoreTerm>, Rc<CoreTerm>, Option<Span>),
    If(Rc<CoreTerm>, Rc<CoreTerm>, Rc<CoreTerm>, Option<Span>),
    App(Rc<CoreTerm>, Rc<CoreTerm>, Option<Span>),
    Call(String, Vec<CoreTerm>, Option<Span>),  // Core IR 0.3: canonical function name
}

// Stack-based iterative deserialization to handle deeply nested Core IR
enum StackFrame<'a> {
    // Leaf nodes - ready to convert
    IntLit(i64),
    BoolLit(bool),
    UnitLit,
    Var(String),

    // Non-leaf nodes - waiting for children
    Lam {
        param: String,
        body_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        body_done: bool,
    },
    App {
        fn_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        arg_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        fn_done: bool,
        arg_done: bool,
    },
    Call {
        target_name: String,  // Core IR 0.3: canonical function name
        readers: Vec<crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>>,
        children: Vec<CoreTerm>,
        next_idx: usize,
    },
    Let {
        name: String,
        value_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        body_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        value_done: bool,
        body_done: bool,
    },
    If {
        cond_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        then_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        else_reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>,
        cond_done: bool,
        then_done: bool,
        else_done: bool,
    },
}

fn deserialize_core_term(reader: crate::axis_core_ir_0_3_capnp::core_term::Reader) -> Result<CoreTerm, String> {
    let mut work_stack: Vec<StackFrame> = Vec::new();
    let mut result_stack: Vec<CoreTerm> = Vec::new();
    let mut loop_counter: usize = 0;

    // Push initial reader as work
    work_stack.push(parse_reader_to_frame(reader)?);

    while let Some(frame) = work_stack.pop() {
        loop_counter += 1;
        if loop_counter % 10000 == 0 {
            eprintln!("[PROGRESS] phase=axis_rust_bridge loop=core_ir_deserialize count={}", loop_counter);
        }

        match frame {
            StackFrame::IntLit(v) => result_stack.push(CoreTerm::IntLit(v, None)),
            StackFrame::BoolLit(v) => result_stack.push(CoreTerm::BoolLit(v, None)),
            StackFrame::UnitLit => result_stack.push(CoreTerm::UnitLit(None)),
            StackFrame::Var(name) => result_stack.push(CoreTerm::Var(name, None)),

            StackFrame::Lam { param, body_reader, body_done } => {
                if !body_done {
                    work_stack.push(StackFrame::Lam { param, body_reader, body_done: true });
                    work_stack.push(parse_reader_to_frame(body_reader)?);
                } else {
                    let body = result_stack.pop().ok_or("Stack underflow: Lam body")?;
                    result_stack.push(CoreTerm::Lam(param, Rc::new(body), None));
                }
            }

            StackFrame::App { fn_reader, arg_reader, fn_done, arg_done } => {
                if !fn_done {
                    work_stack.push(StackFrame::App { fn_reader, arg_reader, fn_done: true, arg_done: false });
                    work_stack.push(parse_reader_to_frame(fn_reader)?);
                } else if !arg_done {
                    work_stack.push(StackFrame::App { fn_reader, arg_reader, fn_done: true, arg_done: true });
                    work_stack.push(parse_reader_to_frame(arg_reader)?);
                } else {
                    let arg = result_stack.pop().ok_or("Stack underflow: App arg")?;
                    let func = result_stack.pop().ok_or("Stack underflow: App func")?;
                    result_stack.push(CoreTerm::App(Rc::new(func), Rc::new(arg), None));
                }
            }

            StackFrame::Call { target_name, readers, mut children, next_idx } => {
                if next_idx < readers.len() {
                    let reader_to_process = readers[next_idx];
                    work_stack.push(StackFrame::Call { target_name, readers, children, next_idx: next_idx + 1 });
                    work_stack.push(parse_reader_to_frame(reader_to_process)?);
                } else {
                    let count = readers.len();
                    for _ in 0..count {
                        children.push(result_stack.pop().ok_or("Stack underflow: Call arg")?);
                    }
                    children.reverse();
                    result_stack.push(CoreTerm::Call(target_name, children, None));
                }
            }

            StackFrame::Let { name, value_reader, body_reader, value_done, body_done } => {
                if !value_done {
                    work_stack.push(StackFrame::Let { name, value_reader, body_reader, value_done: true, body_done: false });
                    work_stack.push(parse_reader_to_frame(value_reader)?);
                } else if !body_done {
                    work_stack.push(StackFrame::Let { name, value_reader, body_reader, value_done: true, body_done: true });
                    work_stack.push(parse_reader_to_frame(body_reader)?);
                } else {
                    let body = result_stack.pop().ok_or("Stack underflow: Let body")?;
                    let value = result_stack.pop().ok_or("Stack underflow: Let value")?;
                    result_stack.push(CoreTerm::Let(name, Rc::new(value), Rc::new(body), None));
                }
            }

            StackFrame::If { cond_reader, then_reader, else_reader, cond_done, then_done, else_done } => {
                if !cond_done {
                    work_stack.push(StackFrame::If { cond_reader, then_reader, else_reader, cond_done: true, then_done: false, else_done: false });
                    work_stack.push(parse_reader_to_frame(cond_reader)?);
                } else if !then_done {
                    work_stack.push(StackFrame::If { cond_reader, then_reader, else_reader, cond_done: true, then_done: true, else_done: false });
                    work_stack.push(parse_reader_to_frame(then_reader)?);
                } else if !else_done {
                    work_stack.push(StackFrame::If { cond_reader, then_reader, else_reader, cond_done: true, then_done: true, else_done: true });
                    work_stack.push(parse_reader_to_frame(else_reader)?);
                } else {
                    let else_branch = result_stack.pop().ok_or("Stack underflow: If else")?;
                    let then_branch = result_stack.pop().ok_or("Stack underflow: If then")?;
                    let cond = result_stack.pop().ok_or("Stack underflow: If cond")?;
                    result_stack.push(CoreTerm::If(Rc::new(cond), Rc::new(then_branch), Rc::new(else_branch), None));
                }
            }
        }
    }

    result_stack.pop().ok_or_else(|| "Empty result stack after deserialization".to_string())
}

fn parse_reader_to_frame<'a>(reader: crate::axis_core_ir_0_3_capnp::core_term::Reader<'a>) -> Result<StackFrame<'a>, String> {
    use crate::axis_core_ir_0_3_capnp::core_term::Which;

    match reader.which() {
        Ok(Which::CIntLit(lit)) => {
            let lit = lit.map_err(|e| format!("Failed to read CIntLit: {}", e))?;
            Ok(StackFrame::IntLit(lit.get_value()))
        },
        Ok(Which::CBoolLit(lit)) => {
            let lit = lit.map_err(|e| format!("Failed to read CBoolLit: {}", e))?;
            Ok(StackFrame::BoolLit(lit.get_value()))
        },
        Ok(Which::CUnitLit(_)) => Ok(StackFrame::UnitLit),
        Ok(Which::CVar(var)) => {
            let var = var.map_err(|e| format!("Failed to read CVar: {}", e))?;
            let name = var.get_name().map_err(|e| format!("Failed to get var name: {}", e))?;
            let name_str = name.to_str().map_err(|e| format!("Invalid UTF-8 in var name: {}", e))?.to_string();
            Ok(StackFrame::Var(name_str))
        },
        Ok(Which::CLam(lam)) => {
            let lam = lam.map_err(|e| format!("Failed to read CLam: {}", e))?;
            let param = lam.get_param().map_err(|e| format!("Failed to get param: {}", e))?;
            let body_reader = lam.get_body().map_err(|e| format!("Failed to get body: {}", e))?;
            Ok(StackFrame::Lam { param: param.to_str().map_err(|e| format!("Invalid UTF-8 in param: {}", e))?.to_string(), body_reader, body_done: false })
        },
        Ok(Which::CApp(app)) => {
            let app = app.map_err(|e| format!("Failed to read CApp: {}", e))?;
            let fn_reader = app.get_fn().map_err(|e| format!("Failed to get fn: {}", e))?;
            let arg_reader = app.get_arg().map_err(|e| format!("Failed to get arg: {}", e))?;
            Ok(StackFrame::App { fn_reader, arg_reader, fn_done: false, arg_done: false })
        },
        Ok(Which::CCall(call)) => {
            let call = call.map_err(|e| format!("Failed to read CCall: {}", e))?;
            let target_name = call.get_target_name()
                .map_err(|e| format!("Failed to get target name: {}", e))?
                .to_str()
                .map_err(|e| format!("Invalid UTF-8 in target name: {}", e))?
                .to_string();
            let args_reader = call.get_args().map_err(|e| format!("Failed to get args: {}", e))?;
            let mut readers = Vec::new();
            for i in 0..args_reader.len() {
                readers.push(args_reader.get(i));
            }
            Ok(StackFrame::Call { target_name, readers, children: Vec::new(), next_idx: 0 })
        },
        Ok(Which::CLet(let_)) => {
            let let_ = let_.map_err(|e| format!("Failed to read CLet: {}", e))?;
            let name = let_.get_name().map_err(|e| format!("Failed to get name: {}", e))?;
            let value_reader = let_.get_value().map_err(|e| format!("Failed to get value: {}", e))?;
            let body_reader = let_.get_body().map_err(|e| format!("Failed to get body: {}", e))?;
            Ok(StackFrame::Let { name: name.to_str().map_err(|e| format!("Invalid UTF-8 in let name: {}", e))?.to_string(), value_reader, body_reader, value_done: false, body_done: false })
        },
        Ok(Which::CIf(if_)) => {
            let if_ = if_.map_err(|e| format!("Failed to read CIf: {}", e))?;
            let cond_reader = if_.get_cond().map_err(|e| format!("Failed to get cond: {}", e))?;
            let then_reader = if_.get_then().map_err(|e| format!("Failed to get then: {}", e))?;
            let else_reader = if_.get_else().map_err(|e| format!("Failed to get else: {}", e))?;
            Ok(StackFrame::If { cond_reader, then_reader, else_reader, cond_done: false, then_done: false, else_done: false })
        },
        Err(e) => Err(format!("Unknown CoreTerm variant: {:?}", e)),
    }
}

// ============================================================
// SERIALIZATION: Write Core IR to Cap'n Proto binary format
// ============================================================

/// Serialize a CoreTerm to Cap'n Proto format
fn serialize_core_term(term: &CoreTerm, builder: crate::axis_core_ir_0_3_capnp::core_term::Builder) {
    match term {
        CoreTerm::IntLit(n, _) => {
            let mut lit = builder.init_c_int_lit();
            lit.set_value(*n);
        },
        CoreTerm::BoolLit(b, _) => {
            let mut lit = builder.init_c_bool_lit();
            lit.set_value(*b);
        },
        CoreTerm::UnitLit(_) => {
            builder.init_c_unit_lit();
        },
        CoreTerm::Var(name, _) => {
            let mut var = builder.init_c_var();
            var.set_name(name);
        },
        CoreTerm::Lam(param, body, _) => {
            let mut lam = builder.init_c_lam();
            lam.set_param(param);
            let body_builder = lam.init_body();
            serialize_core_term(body, body_builder);
        },
        CoreTerm::App(func, arg, _) => {
            let mut app = builder.init_c_app();
            let func_builder = app.reborrow().init_fn();
            serialize_core_term(func, func_builder);
            let arg_builder = app.init_arg();
            serialize_core_term(arg, arg_builder);
        },
        CoreTerm::Let(name, value, body, _) => {
            let mut let_node = builder.init_c_let();
            let_node.set_name(name);
            let value_builder = let_node.reborrow().init_value();
            serialize_core_term(value, value_builder);
            let body_builder = let_node.init_body();
            serialize_core_term(body, body_builder);
        },
        CoreTerm::If(cond, then_branch, else_branch, _) => {
            let mut if_node = builder.init_c_if();
            let cond_builder = if_node.reborrow().init_cond();
            serialize_core_term(cond, cond_builder);
            let then_builder = if_node.reborrow().init_then();
            serialize_core_term(then_branch, then_builder);
            let else_builder = if_node.init_else();
            serialize_core_term(else_branch, else_builder);
        },
        CoreTerm::Call(target_name, args, _) => {
            let mut call = builder.init_c_call();
            call.set_target_name(target_name);
            let mut args_builder = call.init_args(args.len() as u32);
            for (i, arg) in args.iter().enumerate() {
                let arg_builder = args_builder.reborrow().get(i as u32);
                serialize_core_term(arg, arg_builder);
            }
        },
    }
}


/// Create a Core bundle binary from a CoreTerm
pub fn create_core_bundle(term: &CoreTerm, entrypoint_name: &str) -> Vec<u8> {
    use capnp::message::Builder;
    use capnp::serialize;
    
    let mut message = Builder::new_default();
    
    {
        let mut bundle = message.init_root::<crate::axis_core_ir_0_3_capnp::core_bundle::Builder>();
        bundle.set_version("0.3");
        bundle.set_entrypoint_name(entrypoint_name);
        bundle.set_entrypoint_id(0);
        
        let core_term_builder = bundle.init_core_term();
        serialize_core_term(term, core_term_builder);
    }
    
    let mut buf = Vec::new();
    serialize::write_message(&mut buf, &message).unwrap();
    buf
}

/// Write a Core bundle to a file path
pub fn write_core_bundle_to_file(term: &CoreTerm, entrypoint_name: &str, path: &str) -> Result<(), String> {
    let bytes = create_core_bundle(term, entrypoint_name);
    fs::write(path, bytes).map_err(|e| format!("Failed to write Core bundle: {}", e))
}

/// Inspect a Core bundle file and return a summary
pub fn inspect_core_bundle(path: &str) -> Result<String, String> {
    let program = load_core_bundle(path)?;
    Ok(format!(
        "Core bundle: {}\n  Version: 0.3\n  Entrypoint ID: {}\n  Root term: {:?}",
        path,
        program.entrypoint_id,
        core_term_summary(&program.root_term)
    ))
}

/// Generate a brief summary of a CoreTerm (for inspection)
fn core_term_summary(term: &CoreTerm) -> String {
    match term {
        CoreTerm::IntLit(n, _) => format!("IntLit({})", n),
        CoreTerm::BoolLit(b, _) => format!("BoolLit({})", b),
        CoreTerm::UnitLit(_) => "UnitLit".to_string(),
        CoreTerm::Var(name, _) => format!("Var({})", name),
        CoreTerm::Lam(param, _, _) => format!("Lam({}, ...)", param),
        CoreTerm::App(_, _, _) => "App(...)".to_string(),
        CoreTerm::Let(name, _, _, _) => format!("Let({}, ...)", name),
        CoreTerm::If(_, _, _, _) => "If(...)".to_string(),
        CoreTerm::Call(target_name, args, _) => format!("Call(target={}, {} args)", target_name, args.len()),
    }
}

/// Load a core bundle binary file produced by `axis-compiler`
pub fn load_core_bundle(path: &str) -> Result<CoreProgram, String> {
    use capnp::message::ReaderOptions;
    use capnp::serialize;
    use std::io::{BufReader, Read, Seek, SeekFrom};
    
    eprintln!("DEBUG: load_core_bundle called with path: {}", path);
    
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open Core bundle: {}", e))?;
    
    // DEBUG PROBE 1: Read first 16 bytes
    let mut header = [0u8; 16];
    file.read_exact(&mut header)
        .map_err(|e| format!("Failed to read header: {}", e))?;
    eprintln!("DEBUG PROBE 1: First 16 bytes from Rust reader:");
    eprintln!("  {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x}",
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14], header[15]);
    
    // Seek back to start
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    
    let mut buf_reader = BufReader::new(file);

    let mut opts = ReaderOptions::new();

    // Allow very large compiler IRs (trusted input)
    opts.traversal_limit_in_words = Some(1024 * 1024 * 1024); // ~8GB logical traversal
    opts.nesting_limit = 1_000_000;                     // extremely deep trees

    eprintln!("DEBUG: About to call read_message");
    
    let reader = serialize::read_message(
        &mut buf_reader,
        opts
    ).map_err(|e| {
        eprintln!("DEBUG: read_message failed with error: {:?}", e);
        format!("Failed to read Cap'n Proto message: {}", e)
    })?;
    
    eprintln!("DEBUG: read_message succeeded");
    
    let bundle = reader.get_root::<crate::axis_core_ir_0_3_capnp::core_bundle::Reader>()
        .map_err(|e| format!("Failed to get root: {}", e))?;
    
    let version = bundle.get_version()
        .map_err(|e| format!("Failed to get version: {}", e))?;
    
    if version.to_str().map_err(|e| format!("Invalid UTF-8 in version: {}", e))? != "0.3" {
        return Err(format!("Unsupported Core bundle version: {:?}", version));
    }
    
    let entrypoint_id = bundle.get_entrypoint_id() as usize;
    
    let core_term_reader = bundle.get_core_term()
        .map_err(|e| format!("Failed to get core term: {}", e))?;
    
    let root_term = deserialize_core_term(core_term_reader)?;

    Ok(CoreProgram { root_term, entrypoint_id })
}

pub fn load_core_bundle_from_bytes(bytes: &[u8]) -> Result<CoreProgram, String> {
    use capnp::message::ReaderOptions;
    use capnp::serialize;
    
    let mut opts = ReaderOptions::new();
    opts.traversal_limit_in_words = Some(1024 * 1024 * 1024);
    opts.nesting_limit = 1_000_000;

    let reader = serialize::read_message(&mut &bytes[..], opts)
        .map_err(|e| format!("Failed to read Cap'n Proto message: {}", e))?;
    
    let bundle = reader.get_root::<crate::axis_core_ir_0_3_capnp::core_bundle::Reader>()
        .map_err(|e| format!("Failed to get root: {}", e))?;
    
    let version = bundle.get_version()
        .map_err(|e| format!("Failed to get version: {}", e))?;
    
    if version.to_str().map_err(|e| format!("Invalid UTF-8 in version: {}", e))? != "0.3" {
        return Err(format!("Unsupported Core bundle version: {:?}", version));
    }
    
    let entrypoint_id = bundle.get_entrypoint_id() as usize;
    
    let core_term_reader = bundle.get_core_term()
        .map_err(|e| format!("Failed to get core term: {}", e))?;
    
    let root_term = deserialize_core_term(core_term_reader)?;

    Ok(CoreProgram { root_term, entrypoint_id })
}

// ═══════════════════════════════════════════════════════════════════════════
// ANNOTATION CONSUMPTION EXAMPLE
// ═══════════════════════════════════════════════════════════════════════════

/*
 * Example: How a bridge would consume annotations from Core IR
 *
 * If this bridge had access to the main library's CoreTerm with annotations,
 * it could extract and use them. For example:
 *
 * ```rust
 * fn extract_inline_hint(term: &main_lib::CoreTerm) -> Option<bool> {
 *     let annotations = match term {
 *         main_lib::CoreTerm::CUnitLit { annotations, .. } => annotations,
 *         main_lib::CoreTerm::CLam { annotations, .. } => annotations,
 *         main_lib::CoreTerm::CIf { annotations, .. } => annotations,
 *         main_lib::CoreTerm::CCall { annotations, .. } => annotations,
 *     };
 *
 *     for ann in annotations {
 *         if ann.key == "inline" {
 *             if let AnnotationValue::Bool(b) = &ann.value {
 *                 return Some(*b);
 *             }
 *         }
 *     }
 *     None
 * }
 *
 * fn emit_rust_function(term: &main_lib::CoreTerm) -> String {
 *     let mut output = String::new();
 *     
 *     // Check for @inline annotation
 *     if let Some(true) = extract_inline_hint(term) {
 *         output.push_str("#[inline]\n");
 *     }
 *     
 *     output.push_str("fn generated_function() { ... }\n");
 *     output
 * }
 * ```
 *
 * This demonstrates:
 * - Annotations are detected and extracted
 * - They influence output (e.g., #[inline] attribute)
 * - They do NOT affect semantics (execution is unchanged)
 * - Unknown annotations are ignored
 */

