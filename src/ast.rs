// ast.rs
//
// Абстрактное синтаксическое дерево (AST) для языка Rusthon.
// Здесь описаны:
//  - статические типы (Type)
//  - операторы (Stmt)
//  - выражения (Expr)
//  - двоичные операции (BinOp)
//  - функции и программа целиком (Function, Program)

/// Статические типы языка.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Целое число
    Int,
    /// Логическое значение
    Bool,
    /// Строка
    Str,
    /// Список значений (пока без параметризации по типу элементов)
    List,
}

/// Оператор (statement).
/// Это всё, что выполняется "как действие": объявления, присваивания, if, циклы, return и т.п.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Объявление переменной:
    ///   var name: ty = init
    VarDecl { name: String, ty: Type, init: Expr },

    /// Оператор-выражение:
    ///   <expr>
    /// Например: вызов функции `print(x)`.
    ExprStmt(Expr),

    /// Присваивание:
    ///   name = expr
    Assign { name: String, expr: Expr },

    /// Ветвление if / elif* / else:
    ///
    /// if cond {
    ///     then_branch...
    /// } elif ... {
    ///     ...
    /// } else {
    ///     else_branch...
    /// }
    Branch {
        cond: Expr,
        then_branch: Vec<Stmt>,
        /// Цепочка `elif`-веток, каждая кодируется как отдельный `ElseIfBranch`.
        else_if_branches: Vec<Stmt>,
        /// Тело блока `else { ... }` (может быть пустым).
        else_branch: Vec<Stmt>,
    },

    /// Одна ветка вида `elif cond { then_branch... }`.
    ElseIfBranch { cond: Expr, then_branch: Vec<Stmt> },

    /// Цикл `while (cond) { body }`
    While { cond: Expr, body: Vec<Stmt> },

    /// Простой "for" с условием:
    ///   for (cond) { body }
    /// Семантически похож на `while (cond) { body }`.
    // C-style for (init; cond; step) { body }
    For {
        init: Option<Box<Stmt>>, // var i: int = 0   или  i = 0  или print(...)
        cond: Option<Expr>,      // i < 10           (если None — считаем, что всегда true)
        step: Option<Box<Stmt>>, // i = i + 1        или любая ExprStmt/Assign/VarDecl
        body: Vec<Stmt>,
    },

    /// Цикл for-each:
    ///
    ///   for var_name in iter_expr {
    ///       body...
    ///   }
    ///
    /// Где `iter_expr` может быть:
    ///   - Int(n)  -> 0..n-1
    ///   - Str("abc") -> посимвольно
    ///   - List([...]) -> по элементам
    ForEach {
        var_name: String,
        iter_expr: Expr,
        body: Vec<Stmt>,
    },

    /// Оператор `return` внутри функции.
    ///   return expr
    ///   return        // без значения
    Return(Option<Expr>),
}

/// Описание пользовательской функции.
///
///   func name(p1: T1, p2: T2, ...) {
///       body...
///   }
#[derive(Debug, Clone)]
pub struct Function {
    /// Имя функции.
    pub name: String,
    /// Параметры: (имя, тип).
    pub params: Vec<(String, Type)>,
    /// Тело функции — блок операторов.
    pub body: Vec<Stmt>,
}

/// Вся программа целиком:
///  - список объявленных функций
///  - список "глобальных" операторов (выполняются как main-скрипт)
#[derive(Debug, Clone)]
pub struct Program {
    /// Все `func ... { ... }`.
    pub functions: Vec<Function>,
    /// Глобальные операторы вне функций.
    pub stmts: Vec<Stmt>,
}

/// Выражения (expression).
/// Это всё, что можно вычислить и получить `Value` в интерпретаторе.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Целочисленный литерал: `123`
    Int(i64),

    /// Логический литерал: `true` / `false`
    Bool(bool),

    /// Строковый литерал: `"hello"`
    Str(String),

    /// Использование переменной по имени: `x`
    Var(String),

    /// Бинарная операция:
    ///   left <op> right
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    /// Вызов функции:
    ///   callee(arg1, arg2, ...)
    Call { callee: String, args: Vec<Expr> },

    /// Литерал списка:
    ///   [expr1, expr2, expr3, ...]
    ListLiteral(Vec<Expr>),
}

/// Бинарные операторы.
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /

    Eq,    // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=
}
