use crate::ir::core_ir::{CoreBundle, CoreTerm};

/// Errors produced when validating Core IR.
#[derive(Debug, PartialEq, Eq)]
pub enum CoreIrValidationError {
    VersionMismatch {
        expected: &'static str,
        got: Box<str>,
    },
    TopLevelNotCLam,
    CLamBodyNotUnitLit,
    NodeIdPresent(u64),
    EmptyTargetName,
}

/// Validate a `CoreBundle` according to minimal lowering invariants.
///
/// This checks only invariants (no semantics are assigned here). Returns
/// `Ok(())` when the bundle satisfies minimal lowering canonical shape.
pub fn validate_minimal_core_bundle(bundle: &CoreBundle) -> Result<(), CoreIrValidationError> {
    // Rule: version must equal "0.3"
    if bundle.version.as_ref() != "0.3" {
        return Err(CoreIrValidationError::VersionMismatch {
            expected: "0.3",
            got: bundle.version.clone(),
        });
    }

    // Exactly one top-level CoreTerm exists by type; ensure it's the expected shape.
    match &bundle.core_term {
        CoreTerm::CLam { param: _, body, .. } => {
            // Accept either a direct CUnitLit (minimal lowering canonical), or a nested
            // CLam chain that eventually terminates in a CUnitLit (sequencing
            // encoding), or a CIf conditional expression.
            fn is_valid_term(term: &CoreTerm) -> bool {
                match term {
                    CoreTerm::CIntLit { .. } => true,
                    CoreTerm::CBoolLit { .. } => true,
                    CoreTerm::CUnitLit { .. } => true,
                    CoreTerm::CLam { body, .. } => is_valid_term(body.as_ref()),
                    CoreTerm::CLet { value, body, .. } => {
                        is_valid_term(value.as_ref()) && is_valid_term(body.as_ref())
                    }
                    CoreTerm::CIf {
                        cond,
                        then_branch,
                        else_branch,
                        ..
                    } => {
                        is_valid_term(cond.as_ref())
                            && is_valid_term(then_branch.as_ref())
                            && is_valid_term(else_branch.as_ref())
                    }
                    CoreTerm::CVar { .. } => true,
                    CoreTerm::CApp { func, arg, .. } => {
                        is_valid_term(func.as_ref()) && is_valid_term(arg.as_ref())
                    }
                    CoreTerm::CCall {
                        args, target_name, ..
                    } => {
                        // Core IR 0.3: target_name must be non-empty
                        if target_name.is_empty() {
                            return false;
                        }
                        for a in args {
                            if !is_valid_term(a) {
                                return false;
                            }
                        }
                        true
                    }
                }
            }

            if !is_valid_term(body.as_ref()) {
                return Err(CoreIrValidationError::CLamBodyNotUnitLit);
            }

            // Strict rule: node IDs must be absent everywhere.
            // Also check CCall target_name is non-empty.
            fn check_no_node_ids(term: &CoreTerm) -> Result<(), CoreIrValidationError> {
                match term {
                    CoreTerm::CIntLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CBoolLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CUnitLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CLam { body, node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(body.as_ref())
                    }
                    CoreTerm::CLet {
                        value,
                        body,
                        node_id,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(value.as_ref())?;
                        check_no_node_ids(body.as_ref())
                    }
                    CoreTerm::CIf {
                        cond,
                        then_branch,
                        else_branch,
                        node_id,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(cond.as_ref())?;
                        check_no_node_ids(then_branch.as_ref())?;
                        check_no_node_ids(else_branch.as_ref())
                    }
                    CoreTerm::CVar { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CApp {
                        func, arg, node_id, ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(func.as_ref())?;
                        check_no_node_ids(arg.as_ref())
                    }
                    CoreTerm::CCall {
                        args,
                        node_id,
                        target_name,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        // Core IR 0.3: target_name must be non-empty
                        if target_name.is_empty() {
                            return Err(CoreIrValidationError::EmptyTargetName);
                        }
                        for a in args {
                            check_no_node_ids(a)?;
                        }
                        Ok(())
                    }
                }
            }

            check_no_node_ids(&bundle.core_term)
        }
        _ => Err(CoreIrValidationError::TopLevelNotCLam),
    }
}

// Registry validator: checks same invariants as minimal lowering plus call node shapes.
pub fn validate_registry_core_bundle(
    bundle: &CoreBundle,
    registry: &crate::registry::Registry,
    active_profile: &str,
) -> Result<(), CoreIrValidationError> {
    // version
    if bundle.version.as_ref() != "0.3" {
        return Err(CoreIrValidationError::VersionMismatch {
            expected: "0.3",
            got: bundle.version.clone(),
        });
    }

    match &bundle.core_term {
        CoreTerm::CLam { param: _, body, .. } => {
            fn is_valid_term(
                term: &CoreTerm,
                registry: &crate::registry::Registry,
                active_profile: &str,
            ) -> bool {
                match term {
                    CoreTerm::CIntLit { .. } => true,
                    CoreTerm::CBoolLit { .. } => true,
                    CoreTerm::CUnitLit { .. } => true,
                    CoreTerm::CLam { body, .. } => {
                        is_valid_term(body.as_ref(), registry, active_profile)
                    }
                    CoreTerm::CLet { value, body, .. } => {
                        is_valid_term(value.as_ref(), registry, active_profile)
                            && is_valid_term(body.as_ref(), registry, active_profile)
                    }
                    CoreTerm::CIf {
                        cond,
                        then_branch,
                        else_branch,
                        ..
                    } => {
                        is_valid_term(cond.as_ref(), registry, active_profile)
                            && is_valid_term(then_branch.as_ref(), registry, active_profile)
                            && is_valid_term(else_branch.as_ref(), registry, active_profile)
                    }
                    CoreTerm::CVar { .. } => true,
                    CoreTerm::CApp { func, arg, .. } => {
                        is_valid_term(func.as_ref(), registry, active_profile)
                            && is_valid_term(arg.as_ref(), registry, active_profile)
                    }
                    CoreTerm::CCall {
                        args, target_name, ..
                    } => {
                        // verify args structurally valid
                        for a in args {
                            if !is_valid_term(a, registry, active_profile) {
                                return false;
                            }
                        }
                        // Core IR 0.3: verify target_name exists in registry and is allowed by profile
                        if let Some(e) = registry.entries.iter().find(|e| e.name == *target_name) {
                            e.profiles.iter().any(|p| p == "all" || p == active_profile)
                        } else {
                            false
                        }
                    }
                }
            }

            if !is_valid_term(body.as_ref(), registry, active_profile) {
                return Err(CoreIrValidationError::CLamBodyNotUnitLit);
            }

            fn check_no_node_ids(term: &CoreTerm) -> Result<(), CoreIrValidationError> {
                match term {
                    CoreTerm::CIntLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CBoolLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CUnitLit { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CLam { body, node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(body.as_ref())
                    }
                    CoreTerm::CLet {
                        value,
                        body,
                        node_id,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(value.as_ref())?;
                        check_no_node_ids(body.as_ref())
                    }
                    CoreTerm::CIf {
                        cond,
                        then_branch,
                        else_branch,
                        node_id,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(cond.as_ref())?;
                        check_no_node_ids(then_branch.as_ref())?;
                        check_no_node_ids(else_branch.as_ref())
                    }
                    CoreTerm::CVar { node_id, .. } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        Ok(())
                    }
                    CoreTerm::CApp {
                        func, arg, node_id, ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        check_no_node_ids(func.as_ref())?;
                        check_no_node_ids(arg.as_ref())
                    }
                    CoreTerm::CCall {
                        args,
                        node_id,
                        target_name,
                        ..
                    } => {
                        if let Some(id) = node_id {
                            return Err(CoreIrValidationError::NodeIdPresent(*id));
                        }
                        // Core IR 0.3: target_name must be non-empty
                        if target_name.is_empty() {
                            return Err(CoreIrValidationError::EmptyTargetName);
                        }
                        for a in args {
                            check_no_node_ids(a)?;
                        }
                        Ok(())
                    }
                }
            }

            check_no_node_ids(&bundle.core_term)
        }
        _ => Err(CoreIrValidationError::TopLevelNotCLam),
    }
}
