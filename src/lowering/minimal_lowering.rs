use std::fmt;

/// Errors that can be returned by the minimal lowering pass.
#[derive(Debug, PartialEq, Eq)]
pub enum MinimalLowerError {
    UnsupportedParams,
    UnsupportedBody,
    ValidationError(crate::validation::core_ir::CoreIrValidationError),
}

impl fmt::Display for MinimalLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinimalLowerError::UnsupportedParams => {
                write!(f, "unsupported function parameters for minimal lowering")
            }
            MinimalLowerError::UnsupportedBody => {
                write!(f, "unsupported function body for minimal lowering")
            }
            MinimalLowerError::ValidationError(e) => {
                write!(f, "minimal lowering validation failed: {:?}", e)
            }
        }
    }
}

impl std::error::Error for MinimalLowerError {}

fn validate_expr_lowerable(expr: &crate::frontend::ast::Expr) -> Result<(), MinimalLowerError> {
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
        crate::frontend::ast::Expr::Call { .. } => {
            // Minimal lowering does not support calls
            Err(MinimalLowerError::UnsupportedBody)
        }
        crate::frontend::ast::Expr::Ident(_) => {
            // Identifiers are not valid in minimal lowering
            Err(MinimalLowerError::UnsupportedBody)
        }
    }
}

fn lower_expr(expr: &crate::frontend::ast::Expr) -> crate::ir::core_ir::CoreTerm {
    match expr {
        crate::frontend::ast::Expr::UnitLit => crate::ir::core_ir::unit_lit(),
        crate::frontend::ast::Expr::If {
            cond,
            then_block,
            else_block,
        } => {
            let lowered_cond = lower_expr(cond);
            let lowered_then = lower_block(then_block);
            let lowered_else = lower_block(else_block);
            crate::ir::core_ir::cif(lowered_cond, lowered_then, lowered_else)
        }
        crate::frontend::ast::Expr::Call { .. } => {
            panic!("calls not supported in minimal lowering")
        }
        crate::frontend::ast::Expr::Ident(_) => {
            panic!("identifiers not supported in minimal lowering")
        }
    }
}

fn lower_block(block: &crate::frontend::ast::Block) -> crate::ir::core_ir::CoreTerm {
    if block.exprs.is_empty() {
        crate::ir::core_ir::unit_lit()
    } else if block.exprs.len() == 1 {
        lower_expr(&block.exprs[0])
    } else {
        // Multi-expression block: sequence as nested lambdas
        let mut t = lower_expr(&block.exprs[block.exprs.len() - 1]);
        for _ in 1..block.exprs.len() {
            t = crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t);
        }
        t
    }
}

/// Lower a `frontend::ast::FunctionDecl` according to minimal lowering rules.
///
/// Rules (strict): only `NoParams` and `EmptyBlock` are accepted. On success
/// this emits the canonical Core IR semantic `CLam(dummy_param, CUnitLit)`
/// wrapped in a `CoreBundle` via `bundle_v0_2`.
pub fn lower_minimal_function(
    func: crate::frontend::ast::FunctionDecl,
) -> Result<crate::ir::core_ir::CoreBundle, MinimalLowerError> {
    // Explicitly validate shape according to minimal lowering constraints.
    let crate::frontend::ast::NoParams = func.params;

    let body = func.body;

    match &body {
        crate::frontend::ast::FunctionBody::Empty => {
            // Minimal lowering empty-body form: ok
        }
        crate::frontend::ast::FunctionBody::Block(crate::frontend::ast::Block { exprs }) => {
            // Validate all expressions are lowerable
            for expr in exprs {
                validate_expr_lowerable(expr)?;
            }
        }
    }

    // Semantic assignment:
    // - Single-expression or Empty => CLam(dummy, CUnitLit)
    // - Multi-expression => encode sequencing as nested CLam nodes ending in CUnitLit
    // - If expression => lower to CIf with lowered branches
    let term = match body {
        crate::frontend::ast::FunctionBody::Empty => crate::ir::core_ir::lam_dummy_unit(),
        crate::frontend::ast::FunctionBody::Block(crate::frontend::ast::Block { exprs }) => {
            if exprs.is_empty() {
                crate::ir::core_ir::lam_dummy_unit()
            } else if exprs.len() == 1 {
                // Single expression: lower it directly and wrap in lambda
                let lowered = lower_expr(&exprs[0]);
                crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), lowered)
            } else {
                // Multi-expression: sequence them as nested lambdas
                let mut t = lower_expr(&exprs[exprs.len() - 1]);
                for _ in 1..exprs.len() {
                    t = crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t);
                }
                crate::ir::core_ir::lam(crate::ir::core_ir::IdentOrName::dummy(), t)
            }
        }
    };
    let bundle = crate::ir::core_ir::bundle_v0_2(term);
    // Validate canonical Core IR invariants for minimal lowering.
    if let Err(e) = crate::validation::core_ir::validate_minimal_core_bundle(&bundle) {
        return Err(MinimalLowerError::ValidationError(e));
    }
    Ok(bundle)
}
