use crate::generated::axis_core_ir_0_3_capnp;

const CORE_IR_VERSION: &str = "0.3";

// ═══════════════════════════════════════════════════════════════════════════
// ANNOTATION MODEL
// ═══════════════════════════════════════════════════════════════════════════

/// Annotation attached to Core IR nodes
///
/// Annotations are data, not behavior. They must never affect semantics or execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub key: String,
    pub value: AnnotationValue,
}

/// Value types for annotations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationValue {
    Bool(bool),
    Int(i64),
    String(String),
    Symbol(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// CORE IR TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentOrName(pub Box<str>);

impl IdentOrName {
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    pub fn dummy() -> Self {
        Self("_".into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreTerm {
    CIntLit {
        value: i64,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CBoolLit {
        value: bool,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CUnitLit {
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CLam {
        param: IdentOrName,
        body: Box<CoreTerm>,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CLet {
        name: IdentOrName,
        value: Box<CoreTerm>,
        body: Box<CoreTerm>,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CIf {
        cond: Box<CoreTerm>,
        then_branch: Box<CoreTerm>,
        else_branch: Box<CoreTerm>,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CVar {
        name: IdentOrName,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CApp {
        func: Box<CoreTerm>,
        arg: Box<CoreTerm>,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
    CCall {
        target_name: String,
        args: Vec<CoreTerm>,
        node_id: Option<u64>,
        annotations: Vec<Annotation>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreBundle {
    pub version: Box<str>,
    pub core_term: CoreTerm,
}

pub fn int_lit(value: i64) -> CoreTerm {
    CoreTerm::CIntLit {
        value,
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn bool_lit(value: bool) -> CoreTerm {
    CoreTerm::CBoolLit {
        value,
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn unit_lit() -> CoreTerm {
    CoreTerm::CUnitLit {
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn lam(param: IdentOrName, body: CoreTerm) -> CoreTerm {
    CoreTerm::CLam {
        param,
        body: Box::new(body),
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn clet(name: IdentOrName, value: CoreTerm, body: CoreTerm) -> CoreTerm {
    CoreTerm::CLet {
        name,
        value: Box::new(value),
        body: Box::new(body),
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn lam_dummy_unit() -> CoreTerm {
    lam(IdentOrName::dummy(), unit_lit())
}

pub fn cif(cond: CoreTerm, then_branch: CoreTerm, else_branch: CoreTerm) -> CoreTerm {
    CoreTerm::CIf {
        cond: Box::new(cond),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn cvar(name: IdentOrName) -> CoreTerm {
    CoreTerm::CVar {
        name,
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn capp(func: CoreTerm, arg: CoreTerm) -> CoreTerm {
    CoreTerm::CApp {
        func: Box::new(func),
        arg: Box::new(arg),
        node_id: None,
        annotations: Vec::new(),
    }
}

pub fn ccall(target_name: String, args: Vec<CoreTerm>) -> CoreTerm {
    CoreTerm::CCall {
        target_name,
        args,
        node_id: None,
        annotations: Vec::new(),
    }
}

/// Create a CoreBundle with the canonical Core IR version (0.3)
pub fn bundle(core_term: CoreTerm) -> CoreBundle {
    CoreBundle {
        version: CORE_IR_VERSION.into(),
        core_term,
    }
}

/// DEPRECATED: Use `bundle()` instead (both emit Core IR 0.3)
pub fn bundle_v0_2(core_term: CoreTerm) -> CoreBundle {
    bundle(core_term)
}

/// DEPRECATED: Use `bundle()` instead
pub fn bundle_v0_3(core_term: CoreTerm) -> CoreBundle {
    bundle(core_term)
}

#[derive(Debug)]
pub enum CoreIrEncodeError {
    VersionMismatch {
        expected: &'static str,
        got: Box<str>,
    },
    UnsupportedTerm(&'static str),
    NodeIdNotMonotonic(u64, u64),
    NodeIdDuplicate(u64),
    Capnp(capnp::Error),
}

impl From<capnp::Error> for CoreIrEncodeError {
    fn from(err: capnp::Error) -> Self {
        Self::Capnp(err)
    }
}

pub fn encode_capnp(bundle: &CoreBundle) -> Result<Vec<u8>, CoreIrEncodeError> {
    if bundle.version.as_ref() != CORE_IR_VERSION {
        return Err(CoreIrEncodeError::VersionMismatch {
            expected: CORE_IR_VERSION,
            got: bundle.version.clone(),
        });
    }

    let mut message = capnp::message::Builder::new_default();
    let mut bundle_builder = message.init_root::<axis_core_ir_0_3_capnp::core_bundle::Builder>();
    bundle_builder.set_version(CORE_IR_VERSION.into());
    bundle_builder.set_entrypoint_id(0); // Default entrypoint

    {
        let core_term_builder = bundle_builder.reborrow().init_core_term();
        write_core_term(core_term_builder, &bundle.core_term)?;
    }

    bundle_builder.init_annotations(0);

    let mut bytes = Vec::new();
    capnp::serialize::write_message(&mut bytes, &message)?;
    Ok(bytes)
}

fn write_core_term(
    mut builder: axis_core_ir_0_3_capnp::core_term::Builder,
    term: &CoreTerm,
) -> Result<(), CoreIrEncodeError> {
    match term {
        CoreTerm::CIntLit {
            value,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut int_lit = builder.reborrow().init_c_int_lit();
            int_lit.set_value(*value);
            Ok(())
        }
        CoreTerm::CBoolLit {
            value,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut bool_lit = builder.reborrow().init_c_bool_lit();
            bool_lit.set_value(*value);
            Ok(())
        }
        CoreTerm::CUnitLit {
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            builder.reborrow().init_c_unit_lit();
            Ok(())
        }
        CoreTerm::CLam {
            param,
            body,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut lam = builder.reborrow().init_c_lam();
            lam.set_param(param.0.as_ref().into());
            let body_builder = lam.init_body();
            write_core_term(body_builder, body)?;
            Ok(())
        }
        CoreTerm::CLet {
            name,
            value,
            body,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut clet = builder.reborrow().init_c_let();
            clet.set_name(name.0.as_ref().into());
            let value_builder = clet.reborrow().init_value();
            write_core_term(value_builder, value)?;
            let body_builder = clet.reborrow().init_body();
            write_core_term(body_builder, body)?;
            Ok(())
        }
        CoreTerm::CIf {
            cond,
            then_branch,
            else_branch,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut cif = builder.reborrow().init_c_if();

            let cond_builder = cif.reborrow().init_cond();
            write_core_term(cond_builder, cond)?;

            let then_builder = cif.reborrow().init_then();
            write_core_term(then_builder, then_branch)?;

            let else_builder = cif.reborrow().init_else();
            write_core_term(else_builder, else_branch)?;

            Ok(())
        }
        CoreTerm::CVar {
            name,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut var = builder.reborrow().init_c_var();
            var.set_name(name.0.as_ref().into());
            Ok(())
        }
        CoreTerm::CApp {
            func,
            arg,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut app = builder.reborrow().init_c_app();
            let fn_builder = app.reborrow().init_fn();
            write_core_term(fn_builder, func)?;
            let arg_builder = app.reborrow().init_arg();
            write_core_term(arg_builder, arg)?;
            Ok(())
        }
        CoreTerm::CCall {
            target_name,
            args,
            node_id,
            annotations: _,
        } => {
            builder.set_node_id(node_id.unwrap_or(0));
            init_empty_span(builder.reborrow().init_span());
            let mut ccall = builder.reborrow().init_c_call();
            ccall.set_target_name(target_name.as_str().into());
            let mut args_builder = ccall.reborrow().init_args(args.len() as u32);
            for (i, a) in args.iter().enumerate() {
                let slot = args_builder.reborrow().get(i as u32);
                write_core_term(slot, a)?;
            }
            Ok(())
        }
    }
}

fn init_empty_span(mut span: axis_core_ir_0_3_capnp::span::Builder) {
    span.set_file("".into());
    span.set_start(0);
    span.set_end(0);
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_creation() {
        // Test creating different annotation value types
        let bool_ann = Annotation {
            key: "inline".to_string(),
            value: AnnotationValue::Bool(true),
        };
        assert_eq!(bool_ann.key, "inline");
        assert_eq!(bool_ann.value, AnnotationValue::Bool(true));

        let int_ann = Annotation {
            key: "priority".to_string(),
            value: AnnotationValue::Int(42),
        };
        assert_eq!(int_ann.key, "priority");
        assert_eq!(int_ann.value, AnnotationValue::Int(42));

        let string_ann = Annotation {
            key: "doc".to_string(),
            value: AnnotationValue::String("documentation".to_string()),
        };
        assert_eq!(string_ann.key, "doc");
        assert_eq!(
            string_ann.value,
            AnnotationValue::String("documentation".to_string())
        );

        let symbol_ann = Annotation {
            key: "target".to_string(),
            value: AnnotationValue::Symbol("wasm".to_string()),
        };
        assert_eq!(symbol_ann.key, "target");
        assert_eq!(
            symbol_ann.value,
            AnnotationValue::Symbol("wasm".to_string())
        );
    }

    #[test]
    fn test_core_term_carries_annotations() {
        // Test that CoreTerm can carry annotations
        let annotations = vec![
            Annotation {
                key: "inline".to_string(),
                value: AnnotationValue::Bool(true),
            },
            Annotation {
                key: "doc".to_string(),
                value: AnnotationValue::String("test".to_string()),
            },
        ];

        let term = CoreTerm::CUnitLit {
            node_id: None,
            annotations: annotations.clone(),
        };

        match term {
            CoreTerm::CUnitLit {
                annotations: ann, ..
            } => {
                assert_eq!(ann.len(), 2);
                assert_eq!(ann[0].key, "inline");
                assert_eq!(ann[1].key, "doc");
            }
            _ => panic!("Expected CUnitLit"),
        }
    }

    #[test]
    fn test_empty_annotations_valid() {
        // Test that empty annotations list is valid
        let term = unit_lit();
        match term {
            CoreTerm::CUnitLit { annotations, .. } => {
                assert!(annotations.is_empty());
            }
            _ => panic!("Expected CUnitLit"),
        }
    }

    #[test]
    fn test_clone_preserves_annotations() {
        // Test that cloning preserves annotations exactly
        let original = CoreTerm::CLam {
            param: IdentOrName::new("x"),
            body: Box::new(unit_lit()),
            node_id: None,
            annotations: vec![Annotation {
                key: "inline".to_string(),
                value: AnnotationValue::Bool(true),
            }],
        };

        let cloned = original.clone();

        match (&original, &cloned) {
            (
                CoreTerm::CLam {
                    annotations: orig_ann,
                    ..
                },
                CoreTerm::CLam {
                    annotations: clone_ann,
                    ..
                },
            ) => {
                assert_eq!(orig_ann.len(), clone_ann.len());
                assert_eq!(orig_ann[0].key, clone_ann[0].key);
                assert_eq!(orig_ann[0].value, clone_ann[0].value);
            }
            _ => panic!("Expected CLam for both"),
        }
    }

    #[test]
    fn test_annotations_in_nested_terms() {
        // Test that nested terms can each have their own annotations
        let inner_ann = vec![Annotation {
            key: "inner".to_string(),
            value: AnnotationValue::String("nested".to_string()),
        }];

        let outer_ann = vec![Annotation {
            key: "outer".to_string(),
            value: AnnotationValue::Bool(false),
        }];

        let inner = CoreTerm::CUnitLit {
            node_id: None,
            annotations: inner_ann,
        };

        let outer = CoreTerm::CLam {
            param: IdentOrName::new("x"),
            body: Box::new(inner),
            node_id: None,
            annotations: outer_ann,
        };

        match outer {
            CoreTerm::CLam {
                body, annotations, ..
            } => {
                assert_eq!(annotations.len(), 1);
                assert_eq!(annotations[0].key, "outer");

                match body.as_ref() {
                    CoreTerm::CUnitLit {
                        annotations: inner_annotations,
                        ..
                    } => {
                        assert_eq!(inner_annotations.len(), 1);
                        assert_eq!(inner_annotations[0].key, "inner");
                    }
                    _ => panic!("Expected CUnitLit in body"),
                }
            }
            _ => panic!("Expected CLam"),
        }
    }

    #[test]
    fn test_serialization_preserves_structure() {
        // Test that annotation-carrying terms can be encoded
        let term = CoreTerm::CUnitLit {
            node_id: Some(1),
            annotations: vec![Annotation {
                key: "test".to_string(),
                value: AnnotationValue::Bool(true),
            }],
        };

        let bundle = bundle_v0_2(term);
        let result = encode_capnp(&bundle);

        // Should succeed even with annotations (annotations are ignored in serialization)
        assert!(result.is_ok());
    }
}
