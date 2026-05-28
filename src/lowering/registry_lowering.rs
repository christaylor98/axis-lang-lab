use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryLowerError {
    UnsupportedParams,
    UnsupportedBody,
    RegistryError(String),
    ValidationError(crate::validation::core_ir::CoreIrValidationError),
}

impl fmt::Display for RegistryLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryLowerError::UnsupportedParams => {
                write!(f, "unsupported function parameters for registry lowering")
            }
            RegistryLowerError::UnsupportedBody => {
                write!(f, "unsupported function body for registry lowering")
            }
            RegistryLowerError::RegistryError(s) => write!(f, "registry error: {}", s),
            RegistryLowerError::ValidationError(e) => {
                write!(f, "registry lowering validation failed: {:?}", e)
            }
        }
    }
}

impl std::error::Error for RegistryLowerError {}

fn validate_expr_lowerable(expr: &crate::frontend::ast::Expr) -> Result<(), RegistryLowerError> {
    match expr {
        crate::frontend::ast::Expr::UnitLit => Ok(()),
        crate::frontend::ast::Expr::If {
            cond,
            then_block,
            else_block,
        } => {
            validate_expr_lowerable(cond)?;
            for e in &then_block.exprs {
                validate_expr_lowerable(e)?;
            }
            for e in &else_block.exprs {
                validate_expr_lowerable(e)?;
            }
            Ok(())
        }
        crate::frontend::ast::Expr::Call { name: _, args } => {
            for a in args {
                match a {
                    crate::frontend::ast::Expr::Ident(_) => {
                        // Identifiers as call arguments are rejected: registry arg handling
                        // is not implemented beyond matching; per spec they may be rejected.
                        return Err(RegistryLowerError::UnsupportedBody);
                    }
                    _ => validate_expr_lowerable(a)?,
                }
            }
            Ok(())
        }
        crate::frontend::ast::Expr::Ident(_) => {
            // Ident outside call args is illegal
            Err(RegistryLowerError::UnsupportedBody)
        }
    }
}

fn lower_expr(
    expr: &crate::frontend::ast::Expr,
    registry: &crate::registry::Registry,
    active_profile: &str,
) -> Result<crate::ir::core_ir::CoreTerm, RegistryLowerError> {
    match expr {
        crate::frontend::ast::Expr::UnitLit => Ok(crate::ir::core_ir::unit_lit()),
        crate::frontend::ast::Expr::If {
            cond,
            then_block,
            else_block,
        } => {
            let lowered_cond = lower_expr(cond, registry, active_profile)?;
            let lowered_then = lower_block(then_block, registry, active_profile)?;
            let lowered_else = lower_block(else_block, registry, active_profile)?;
            Ok(crate::ir::core_ir::cif(
                lowered_cond,
                lowered_then,
                lowered_else,
            ))
        }
        crate::frontend::ast::Expr::Call { name, args } => {
            // Resolve name against registry
            let entry = registry.find_by_name(&name.value).ok_or_else(|| {
                RegistryLowerError::RegistryError(format!("unknown function {}", name.value))
            })?;
            if entry.arity != args.len() {
                return Err(RegistryLowerError::RegistryError(format!(
                    "arity mismatch for {}: expected {} got {}",
                    name.value,
                    entry.arity,
                    args.len()
                )));
            }
            // Profiles: consult active profile; "all" admits it.
            if !entry
                .profiles
                .iter()
                .any(|p| p == "all" || p == active_profile)
            {
                return Err(RegistryLowerError::RegistryError(format!(
                    "profile disallows usage of {}",
                    name.value
                )));
            }
            // Lower args left-to-right
            let mut lowered_args: Vec<crate::ir::core_ir::CoreTerm> = Vec::new();
            for a in args {
                match a {
                    crate::frontend::ast::Expr::Ident(_) => {
                        return Err(RegistryLowerError::RegistryError(
                            "identifier args not supported".into(),
                        ));
                    }
                    _ => {
                        lowered_args.push(lower_expr(a, registry, active_profile)?);
                    }
                }
            }
            // Core IR 0.3: use canonical function name (entry.name) as target
            Ok(crate::ir::core_ir::ccall(entry.name.clone(), lowered_args))
        }
        crate::frontend::ast::Expr::Ident(_) => Err(RegistryLowerError::UnsupportedBody),
    }
}

fn lower_block(
    block: &crate::frontend::ast::Block,
    registry: &crate::registry::Registry,
    active_profile: &str,
) -> Result<crate::ir::core_ir::CoreTerm, RegistryLowerError> {
    if block.exprs.is_empty() {
        Ok(crate::ir::core_ir::unit_lit())
    } else if block.exprs.len() == 1 {
        lower_expr(&block.exprs[0], registry, active_profile)
    } else {
        let mut t = lower_expr(
            &block.exprs[block.exprs.len() - 1],
            registry,
            active_profile,
        )?;
        for _ in 1..block.exprs.len() {
            t = crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t);
        }
        Ok(t)
    }
}

pub fn lower_registry_function(
    func: crate::frontend::ast::FunctionDecl,
) -> Result<crate::ir::core_ir::CoreBundle, RegistryLowerError> {
    let crate::frontend::ast::NoParams = func.params;
    let body = func.body;

    // Load config and registry
    let cfg = match crate::config::language_lab::LanglabConfig::load() {
        Ok(c) => c,
        Err(_) => crate::config::language_lab::LanglabConfig::default(),
    };
    let registry = crate::registry::Registry::load_active().map_err(|e| {
        RegistryLowerError::RegistryError(format!("failed to load registry: {:?}", e))
    })?;

    match &body {
        crate::frontend::ast::FunctionBody::Empty => {
            // ok
        }
        crate::frontend::ast::FunctionBody::Block(crate::frontend::ast::Block { exprs }) => {
            for expr in exprs {
                validate_expr_lowerable(expr)?;
            }
        }
    }

    let term = match body {
        crate::frontend::ast::FunctionBody::Empty => crate::ir::core_ir::lam_dummy_unit(),
        crate::frontend::ast::FunctionBody::Block(crate::frontend::ast::Block { exprs }) => {
            if exprs.is_empty() {
                crate::ir::core_ir::lam_dummy_unit()
            } else if exprs.len() == 1 {
                let lowered = lower_expr(&exprs[0], &registry, &cfg.active_profile)?;
                crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), lowered)
            } else {
                let mut t = lower_expr(&exprs[exprs.len() - 1], &registry, &cfg.active_profile)?;
                for _ in 1..exprs.len() {
                    t = crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t);
                }
                crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t)
            }
        }
    };

    let bundle = crate::ir::core_ir::bundle_v0_2(term);

    if let Err(e) = crate::validation::core_ir::validate_registry_core_bundle(
        &bundle,
        &registry,
        &cfg.active_profile,
    ) {
        return Err(RegistryLowerError::ValidationError(e));
    }

    Ok(bundle)
}
