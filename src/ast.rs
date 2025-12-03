#[derive(Debug)]
pub enum Type {
    Int,
    Bool,
    Str,
    // дальше добавим ещё (float, list[...] и т.п.)
}

#[derive(Debug)]
pub enum Stmt {
    VarDecl { name: String, ty: Type, init: Expr },
    ExprStmt(Expr),
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}
