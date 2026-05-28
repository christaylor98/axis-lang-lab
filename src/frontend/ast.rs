#[derive(Debug)]
pub struct FunctionDecl {
    pub name: Ident,
    pub params: NoParams,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub value: Box<str>,
}

#[derive(Debug)]
pub struct NoParams;

// Phase 2: represent function bodies that can be either the Phase 1
// empty form or a Phase 2 single-expression block. The `FunctionBody`
// enum is the canonical representation; we also export a `pub const
// EmptyBlock` value so existing Phase 1 code that constructs
// `body: EmptyBlock` continues to work without changes.

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionBody {
    Empty,
    Block(Block),
}

#[allow(non_upper_case_globals)]
pub const EmptyBlock: FunctionBody = FunctionBody::Empty;

// Phase 2: introduce expression and block types to represent a
// single-expression function body. These are added alongside the
// existing `EmptyBlock` so Phase 1 AST shapes remain valid.

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    UnitLit,
    Ident(Ident),
    Call {
        name: Ident,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_block: Block,
        else_block: Block,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub exprs: Vec<Expr>,
}
