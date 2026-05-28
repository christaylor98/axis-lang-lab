// core_loader.rs removed: functionality moved to `core_ir.rs`.
// This file is intentionally left as a placeholder to avoid accidental
// use while code is transitioning. Use `core_ir::load_core_bundle` instead.

use crate::core_ir::CoreTerm;
use std::rc::Rc;
#[allow(dead_code)]
// Transitional helpers retained for alternate emission paths
fn deserialize_core_term(value: &serde_json::Value) -> Result<CoreTerm, String> {
    let obj = value.as_object().ok_or("CoreTerm must be an object")?;
    let tag = obj.get("tag")
        .and_then(|v| v.as_str())
        .ok_or("CoreTerm missing 'tag' field")?;

    match tag {
        "CIntLit" => {
            let n = obj.get("value")
                .and_then(|v| v.as_i64())
                .ok_or("CIntLit missing integer value")?;
            Ok(CoreTerm::IntLit(n, None))
        },
        "CBoolLit" => {
            let b = obj.get("value")
                .and_then(|v| v.as_bool())
                .ok_or("CBoolLit missing boolean value")?;
            Ok(CoreTerm::BoolLit(b, None))
        },
        "CUnitLit" => {
            Ok(CoreTerm::UnitLit(None))
        },
        // CStrLit removed in Core IR 0.2
        "CVar" => {
            let name = obj.get("name")
                .and_then(|v| v.as_str())
                .ok_or("CVar missing name")?;
            Ok(CoreTerm::Var(name.to_string(), None))
        },
        "CLam" => {
            let param = obj.get("param")
                .and_then(|v| v.as_str())
                .ok_or("CLam missing param")?;
            let body_val = obj.get("body")
                .ok_or("CLam missing body")?;
            let body = deserialize_core_term(body_val)?;
            Ok(CoreTerm::Lam(param.to_string(), Rc::new(body), None))
        },
        "CApp" => {
            let func_val = obj.get("fn")
                .ok_or("CApp missing fn")?;
            let arg_val = obj.get("arg")
                .ok_or("CApp missing arg")?;
            let func = deserialize_core_term(func_val)?;
            let arg = deserialize_core_term(arg_val)?;
            Ok(CoreTerm::App(Rc::new(func), Rc::new(arg), None))
        },
        // CTuple removed in Core IR 0.2
        // CProj removed in Core IR 0.2
        "CLet" => {
            let name = obj.get("name")
                .and_then(|v| v.as_str())
                .ok_or("CLet missing name")?;
            let value_val = obj.get("value")
                .ok_or("CLet missing value")?;
            let body_val = obj.get("body")
                .ok_or("CLet missing body")?;
            let value = deserialize_core_term(value_val)?;
            let body = deserialize_core_term(body_val)?;
            Ok(CoreTerm::Let(name.to_string(), Rc::new(value), Rc::new(body), None))
        },
        "CIf" => {
            let cond_val = obj.get("cond")
                .ok_or("CIf missing cond")?;
            let then_val = obj.get("then")
                .ok_or("CIf missing then")?;
            let else_val = obj.get("else")
                .ok_or("CIf missing else")?;
            let cond = deserialize_core_term(cond_val)?;
            let then_branch = deserialize_core_term(then_val)?;
            let else_branch = deserialize_core_term(else_val)?;
            Ok(CoreTerm::If(Rc::new(cond), Rc::new(then_branch), Rc::new(else_branch), None))
        },
        // CCtor removed in Core IR 0.2
        "CCall" => {
            // Core IR 0.3: target is canonical function name (string)
            let target_name = obj.get("targetName")
                .and_then(|v| v.as_str())
                .ok_or("CCall missing targetName")?
                .to_string();
            let args_val = obj.get("args")
                .and_then(|v| v.as_array())
                .ok_or("CCall missing args array")?;
            let mut args = Vec::new();
            for a in args_val {
                args.push(deserialize_core_term(a)?);
            }
            Ok(CoreTerm::Call(target_name, args, None))
        }
        _ => Err(format!("Unknown CoreTerm tag: {}", tag))
    }
}

// serialize functions are not required in the bridge; only deserialization is used
