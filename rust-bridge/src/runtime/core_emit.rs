// Core IR emission runtime — linked from generated Axis code
// This module provides `axis_emit_core_bundle_to_file` as a runtime function.

use crate::core_ir::CoreTerm;
use crate::runtime::value::Value;
use crate::runtime::value::{get_str, get_tag_name};
use std::fs;
use std::rc::Rc;

/// Runtime entry point called from generated Axis code
/// Signature: axis_emit_core_bundle_to_file(bundle: CoreBundle, path: Str) -> Result[Unit]
/// UNARY CONTRACT: Accepts Value::Tuple containing [bundle, path]
pub fn axis_emit_core_bundle_to_file(args: Value) -> Value {
    // Unpack arguments from tuple
    let (bundle, path) = match args {
        Value::Tuple(ref elems) if elems.len() >= 2 => {
            (elems[0].clone(), elems[1].clone())
        },
        _ => {
            let err_tag = crate::runtime::value::intern_tag("Err");
            let msg = crate::runtime::value::intern_str("axis_emit_core_bundle_to_file: expected tuple with 2 elements");
            return Value::Ctor { tag: err_tag, fields: vec![Value::Str(msg)] };
        }
    };
    match emit_core_bundle_impl(bundle, path) {
        Ok(_) => {
            // Ok(Unit) constructor
            let ok_tag = crate::runtime::value::intern_tag("Ok");
            Value::Ctor {
                tag: ok_tag,
                fields: vec![Value::Unit],
            }
        }
        Err(e) => {
            // Err(msg) constructor
            let err_tag = crate::runtime::value::intern_tag("Err");
            let msg_str = crate::runtime::value::intern_str(&e);
            Value::Ctor {
                tag: err_tag,
                fields: vec![Value::Str(msg_str)],
            }
        }
    }
}

fn emit_core_bundle_impl(bundle: Value, path: Value) -> Result<(), String> {
    // Extract path string
    let path_str = match path {
        Value::Str(handle) => get_str(handle).to_string(),
        _ => return Err(format!("Expected Str for path, got {:?}", path)),
    };

    // Decode CoreBundle: expect CoreBundle(CoreTerm) (string_table removed in 0.2)
    let core_term_val = match &bundle {
        Value::Ctor { tag, fields } if get_tag_name(*tag) == "CoreBundle" && fields.len() >= 1 => {
            // accept either [core_term] or legacy [string_table, core_terms]
            fields.last().unwrap()
        }
        _ => return Err(format!("Expected CoreBundle constructor, got {:?}", bundle)),
    };

    let root_term = value_to_core_term(core_term_val)?;

    // Serialize to Cap'n Proto using compiler's approach
    let bytes = crate::core_ir::create_core_bundle(&root_term, "main");

    // Write to file
    fs::write(&path_str, bytes)
        .map_err(|e| format!("Failed to write Core bundle to {}: {}", path_str, e))?;

    Ok(())
}

/// Convert runtime Value representation of CoreTerm to Rust CoreTerm
fn value_to_core_term(val: &Value) -> Result<CoreTerm, String> {
    match val {
        Value::Ctor { tag, fields } => {
            let tag_name = get_tag_name(*tag);
            
            match tag_name.as_str() {
                "CIntLit" if fields.len() == 1 => {
                    Ok(CoreTerm::IntLit(fields[0].as_int(), None))
                }
                "CBoolLit" if fields.len() == 1 => {
                    Ok(CoreTerm::BoolLit(fields[0].as_bool(), None))
                }
                "CUnitLit" => {
                    Ok(CoreTerm::UnitLit(None))
                }
                // CStrLit removed in Core IR 0.2
                "CVar" if fields.len() == 1 => {
                    match &fields[0] {
                        Value::Str(handle) => Ok(CoreTerm::Var(get_str(*handle).to_string(), None)),
                        _ => Err(format!("Expected Str in CVar, got {:?}", fields[0])),
                    }
                }
                "CLam" if fields.len() == 2 => {
                    let param = match &fields[0] {
                        Value::Str(handle) => get_str(*handle).to_string(),
                        _ => return Err(format!("Expected Str param in CLam, got {:?}", fields[0])),
                    };
                    let body = value_to_core_term(&fields[1])?;
                    Ok(CoreTerm::Lam(param, Rc::new(body), None))
                }
                "CApp" if fields.len() == 2 => {
                    let func = value_to_core_term(&fields[0])?;
                    let arg = value_to_core_term(&fields[1])?;
                    Ok(CoreTerm::App(Rc::new(func), Rc::new(arg), None))
                }
                // CTuple removed in Core IR 0.2
                // CProj removed in Core IR 0.2
                "CLet" if fields.len() == 3 => {
                    let name = match &fields[0] {
                        Value::Str(handle) => get_str(*handle).to_string(),
                        _ => return Err(format!("Expected Str name in CLet, got {:?}", fields[0])),
                    };
                    let value = value_to_core_term(&fields[1])?;
                    let body = value_to_core_term(&fields[2])?;
                    Ok(CoreTerm::Let(name, Rc::new(value), Rc::new(body), None))
                }
                "CIf" if fields.len() == 3 => {
                    let cond = value_to_core_term(&fields[0])?;
                    let then_br = value_to_core_term(&fields[1])?;
                    let else_br = value_to_core_term(&fields[2])?;
                    Ok(CoreTerm::If(Rc::new(cond), Rc::new(then_br), Rc::new(else_br), None))
                }
                // CCtor removed in Core IR 0.2
                // CMatch / pattern removed in Core IR 0.2
                "CCall" if fields.len() == 2 => {
                    // Core IR 0.3: target is canonical function name (String)
                    let target_name = match &fields[0] {
                        Value::Str(handle) => get_str(*handle).to_string(),
                        _ => return Err(format!("Expected Str for CCall target, got {:?}", fields[0])),
                    };
                    let args = match &fields[1] {
                        Value::List(aslist) => {
                            let mut result = Vec::new();
                            for a in aslist { result.push(value_to_core_term(a)?); }
                            result
                        }
                        _ => return Err(format!("Expected List in CCall args, got {:?}", fields[1])),
                    };
                    Ok(CoreTerm::Call(target_name, args, None))
                }
                _ => Err(format!("Unknown CoreTerm constructor: {}", tag_name)),
            }
        }
        _ => Err(format!("Expected Ctor for CoreTerm, got {:?}", val)),
    }
}

// Pattern conversion removed (patterns are not part of Core IR 0.2)

// Note: local bundle serialization and Pattern helpers removed — use `core_ir::create_core_bundle`
