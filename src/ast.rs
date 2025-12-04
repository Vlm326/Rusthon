// ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    List,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        name: String,
        ty: Type,
        init: Expr,
    },
    ExprStmt(Expr),
    Assign {
        name: String,
        expr: Expr,
    },
    Branch {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_if_branches: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    ElseIfBranch {
        cond: Expr,
        then_branch: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        cond: Expr,
        body: Vec<Stmt>,
    },

    // 🔹 for x in expr { body }
    ForEach {
        var_name: String,
        iter_expr: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
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
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    ListLiteral(Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,   // +
    Sub,   // -
    Mul,   // *
    Div,   // /
    Eq,    // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=
}
