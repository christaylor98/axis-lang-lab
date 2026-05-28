// Core IR JSON Serialization
//
// WAVE 6: Determinism & Equivalence Validation
//
// This module provides JSON serialization for Core IR bundles.
// JSON format is human-readable, diffable, and version-controlled.
//
// DESIGN:
// - Full structural representation (no elision)
// - Preserves node IDs and annotations
// - Deterministic field ordering
// - Pretty-printed for diffs

use crate::ir::core_ir::{Annotation, AnnotationValue, CoreBundle, CoreTerm};
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════════════════
// SERIALIZABLE WRAPPERS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Serialize)]
pub struct CoreBundleJson {
    pub version: String,
    pub core_term: CoreTermJson,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum CoreTermJson {
    CIntLit {
        value: i64,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CBoolLit {
        value: bool,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CUnitLit {
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CLam {
        param: String,
        body: Box<CoreTermJson>,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CLet {
        name: String,
        value: Box<CoreTermJson>,
        body: Box<CoreTermJson>,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CIf {
        cond: Box<CoreTermJson>,
        then_branch: Box<CoreTermJson>,
        else_branch: Box<CoreTermJson>,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CVar {
        name: String,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CApp {
        func: Box<CoreTermJson>,
        arg: Box<CoreTermJson>,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
    CCall {
        target_name: String,
        args: Vec<CoreTermJson>,
        node_id: Option<u64>,
        annotations: Vec<AnnotationJson>,
    },
}

#[derive(Serialize)]
pub struct AnnotationJson {
    pub key: String,
    pub value: AnnotationValueJson,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value")]
pub enum AnnotationValueJson {
    Bool(bool),
    Int(i64),
    String(String),
    Symbol(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// CONVERSION FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

impl From<&CoreBundle> for CoreBundleJson {
    fn from(bundle: &CoreBundle) -> Self {
        CoreBundleJson {
            version: bundle.version.to_string(),
            core_term: (&bundle.core_term).into(),
        }
    }
}

impl From<&CoreTerm> for CoreTermJson {
    fn from(term: &CoreTerm) -> Self {
        match term {
            CoreTerm::CIntLit {
                value,
                node_id,
                annotations,
            } => CoreTermJson::CIntLit {
                value: *value,
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CBoolLit {
                value,
                node_id,
                annotations,
            } => CoreTermJson::CBoolLit {
                value: *value,
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CUnitLit {
                node_id,
                annotations,
            } => CoreTermJson::CUnitLit {
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CLam {
                param,
                body,
                node_id,
                annotations,
            } => CoreTermJson::CLam {
                param: param.0.to_string(),
                body: Box::new(body.as_ref().into()),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CLet {
                name,
                value,
                body,
                node_id,
                annotations,
            } => CoreTermJson::CLet {
                name: name.0.to_string(),
                value: Box::new(value.as_ref().into()),
                body: Box::new(body.as_ref().into()),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CIf {
                cond,
                then_branch,
                else_branch,
                node_id,
                annotations,
            } => CoreTermJson::CIf {
                cond: Box::new(cond.as_ref().into()),
                then_branch: Box::new(then_branch.as_ref().into()),
                else_branch: Box::new(else_branch.as_ref().into()),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CVar {
                name,
                node_id,
                annotations,
            } => CoreTermJson::CVar {
                name: name.0.to_string(),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CApp {
                func,
                arg,
                node_id,
                annotations,
            } => CoreTermJson::CApp {
                func: Box::new(func.as_ref().into()),
                arg: Box::new(arg.as_ref().into()),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
            CoreTerm::CCall {
                target_name,
                args,
                node_id,
                annotations,
            } => CoreTermJson::CCall {
                target_name: target_name.clone(),
                args: args.iter().map(|a| a.into()).collect(),
                node_id: *node_id,
                annotations: annotations.iter().map(|a| a.into()).collect(),
            },
        }
    }
}

impl From<&Annotation> for AnnotationJson {
    fn from(ann: &Annotation) -> Self {
        AnnotationJson {
            key: ann.key.clone(),
            value: (&ann.value).into(),
        }
    }
}

impl From<&AnnotationValue> for AnnotationValueJson {
    fn from(val: &AnnotationValue) -> Self {
        match val {
            AnnotationValue::Bool(b) => AnnotationValueJson::Bool(*b),
            AnnotationValue::Int(i) => AnnotationValueJson::Int(*i),
            AnnotationValue::String(s) => AnnotationValueJson::String(s.clone()),
            AnnotationValue::Symbol(s) => AnnotationValueJson::Symbol(s.clone()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Serialize Core IR bundle to pretty-printed JSON
pub fn to_json(bundle: &CoreBundle) -> Result<String, serde_json::Error> {
    let json_bundle: CoreBundleJson = bundle.into();
    serde_json::to_string_pretty(&json_bundle)
}

/// Serialize Core IR bundle to compact JSON (no whitespace)
pub fn to_json_compact(bundle: &CoreBundle) -> Result<String, serde_json::Error> {
    let json_bundle: CoreBundleJson = bundle.into();
    serde_json::to_string(&json_bundle)
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_unit_lit() {
        let bundle = CoreBundle {
            version: "0.3".into(),
            core_term: CoreTerm::CUnitLit {
                node_id: Some(1),
                annotations: vec![],
            },
        };

        let json = to_json(&bundle).expect("serialization should succeed");
        assert!(json.contains("CUnitLit"));
        assert!(json.contains("\"node_id\": 1"));
    }

    #[test]
    fn test_serialize_int_lit() {
        let bundle = CoreBundle {
            version: "0.3".into(),
            core_term: CoreTerm::CIntLit {
                value: 42,
                node_id: Some(1),
                annotations: vec![],
            },
        };

        let json = to_json(&bundle).expect("serialization should succeed");
        assert!(json.contains("CIntLit"));
        assert!(json.contains("\"value\": 42"));
        assert!(json.contains("\"node_id\": 1"));
    }

    #[test]
    fn test_serialize_bool_lit() {
        let bundle = CoreBundle {
            version: "0.3".into(),
            core_term: CoreTerm::CBoolLit {
                value: true,
                node_id: Some(1),
                annotations: vec![],
            },
        };

        let json = to_json(&bundle).expect("serialization should succeed");
        assert!(json.contains("CBoolLit"));
        assert!(json.contains("\"value\": true"));
    }

    #[test]
    fn test_serialize_if_expr() {
        let bundle = CoreBundle {
            version: "0.3".into(),
            core_term: CoreTerm::CIf {
                cond: Box::new(CoreTerm::CBoolLit {
                    value: true,
                    node_id: Some(1),
                    annotations: vec![],
                }),
                then_branch: Box::new(CoreTerm::CIntLit {
                    value: 1,
                    node_id: Some(2),
                    annotations: vec![],
                }),
                else_branch: Box::new(CoreTerm::CIntLit {
                    value: 0,
                    node_id: Some(3),
                    annotations: vec![],
                }),
                node_id: Some(4),
                annotations: vec![],
            },
        };

        let json = to_json(&bundle).expect("serialization should succeed");
        assert!(json.contains("CIf"));
        assert!(json.contains("cond"));
        assert!(json.contains("then_branch"));
        assert!(json.contains("else_branch"));
    }
}
