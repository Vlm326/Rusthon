// parser.rs

use crate::ast::{BinOp, Expr, Function, Program, Stmt, Type};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    /* ======================== БАЗА ======================== */

    pub fn new(mut lexer: Lexer) -> Self {
        let first = lexer.next_token();
        Self {
            lexer,
            current_token: first,
        }
    }

    /// Сдвигаем текущий токен вперёд.
    fn bump(&mut self) {
        self.current_token = self.lexer.next_token();
        // eprintln!("[DEBUG] bump -> token = {:?}", self.current_token);
    }

    /// Подглядеть следующий токен, не потребляя его.
    fn peek_token(&mut self) -> Token {
        let mut cloned_lexer = self.lexer.clone();
        cloned_lexer.next_token()
    }

    /// Унифицированная функция ошибки парсера.
    fn error(&self, msg: &str) -> ! {
        panic!("Parse error near token {:?}: {}", self.current_token, msg);
    }

    /// Проверяем, что текущий токен — expected, и сдвигаем его.
    fn expect(&mut self, expected: Token) {
        if self.current_token == expected {
            self.bump();
        } else {
            self.error(&format!(
                "expected {:?}, found {:?}",
                expected, self.current_token
            ));
        }
    }

    /// Пропускаем все пустые строки.
    fn skip_newlines(&mut self) {
        while let Token::Newline = self.current_token {
            self.bump();
        }
    }

    /* ======================== ТИПЫ ======================== */

    fn parse_type(&mut self) -> Type {
        match &self.current_token {
            Token::Ident(name) if name == "int" => {
                self.bump();
                Type::Int
            }
            Token::Ident(name) if name == "bool" => {
                self.bump();
                Type::Bool
            }
            Token::Ident(name) if name == "str" => {
                self.bump();
                Type::Str
            }
            Token::Ident(name) if name == "list" => {
                self.bump();
                Type::List
            }
            other => self.error(&format!("expected type name, found {:?}", other)),
        }
    }

    /* ====================== ВЫРАЖЕНИЯ ====================== */
    // Грамматика по приоритетам:
    // primary -> factor -> term -> expr (пока без && и ||)

    fn parse_primary(&mut self) -> Expr {
        match &self.current_token {
            Token::IntLiteral(value) => {
                let expr = Expr::Int(*value);
                self.bump();
                expr
            }
            Token::StrLiteral(s) => {
                let expr = Expr::Str(s.clone());
                self.bump();
                expr
            }
            Token::KwTrue => {
                self.bump();
                Expr::Bool(true)
            }
            Token::KwFalse => {
                self.bump();
                Expr::Bool(false)
            }
            Token::Ident(name) => {
                let expr = Expr::Var(name.clone());
                self.bump();
                expr
            }
            Token::LParen => {
                self.bump();
                let expr = self.parse_expr();
                if self.current_token != Token::RParen {
                    self.error("expected ')' after parenthesized expression");
                }
                self.bump(); // съели ')'
                expr
            }
            Token::LBracket => self.parse_list_literal(),
            other => self.error(&format!(
                "unexpected token in primary expression: {:?}",
                other
            )),
        }
    }

    fn parse_factor(&mut self) -> Expr {
        let mut node = self.parse_primary();
        loop {
            match self.current_token {
                Token::LParen => {
                    node = self.parse_call(node);
                }
                _ => break,
            }
        }
        node
    }

    fn parse_call(&mut self, calle_expr: Expr) -> Expr {
        let callee_name = match calle_expr {
            Expr::Var(name) => name,
            other => self.error(&format!(
                "can only call functions by name, got expression: {:?}",
                other
            )),
        };

        // сейчас current_token == LParen
        self.bump(); // съели '('

        let mut args: Vec<Expr> = Vec::new();

        // если следующий токен НЕ ')', значит, есть аргументы
        if self.current_token != Token::RParen {
            loop {
                let arg = self.parse_expr();
                args.push(arg);

                if self.current_token == Token::Comma {
                    self.bump();
                    continue;
                } else {
                    break;
                }
            }
        }

        // тут мы ДОЛЖНЫ быть на ')'
        if self.current_token != Token::RParen {
            self.error("expected ')' at the end of the function call");
        }
        self.bump(); // съели ')'

        Expr::Call {
            callee: callee_name,
            args,
        }
    }

    fn parse_term(&mut self) -> Expr {
        let mut node = self.parse_factor();

        loop {
            match self.current_token {
                Token::Star => {
                    self.bump();
                    let rhs = self.parse_factor();
                    node = Expr::Binary {
                        left: Box::new(node),
                        op: BinOp::Mul,
                        right: Box::new(rhs),
                    };
                }
                Token::Slash => {
                    self.bump();
                    let rhs = self.parse_factor();
                    node = Expr::Binary {
                        left: Box::new(node),
                        op: BinOp::Div,
                        right: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        node
    }

    /// Полное выражение: +, -, сравнения и т.п.
    pub fn parse_expr(&mut self) -> Expr {
        let mut node = self.parse_term();

        loop {
            let op = match self.current_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };

            self.bump();
            let rhs = self.parse_term();

            node = Expr::Binary {
                left: Box::new(node),
                op,
                right: Box::new(rhs),
            };
        }

        node
    }

    fn parse_list_literal(&mut self) -> Expr {
        self.bump(); // съели '['

        let mut items = Vec::new();

        if self.current_token != Token::RBracket {
            loop {
                let expr = self.parse_expr();
                items.push(expr);

                if self.current_token == Token::Comma {
                    self.bump();
                    continue;
                }
                break;
            }
        }

        if self.current_token != Token::RBracket {
            self.error("expected ']' at end of list literal");
        }
        self.bump(); // съели ']'

        Expr::ListLiteral(items)
    }

    /* ===================== ОПЕРАТОРЫ ====================== */

    fn parse_var_decl(&mut self) -> Stmt {
        self.bump(); // съели 'var'

        let name = match &self.current_token {
            Token::Ident(n) => {
                let s = n.clone();
                self.bump();
                s
            }
            other => self.error(&format!(
                "expected identifier after 'var', found {:?}",
                other
            )),
        };

        self.expect(Token::Colon);

        let ty = self.parse_type();

        self.expect(Token::Eq);

        let init = self.parse_expr();

        if self.current_token == Token::Newline {
            self.bump();
        }

        Stmt::VarDecl { name, ty, init }
    }

    fn parse_assign_stmt(&mut self) -> Stmt {
        let name = match &self.current_token {
            Token::Ident(n) => {
                let s = n.clone();
                self.bump();
                s
            }
            other => self.error(&format!(
                "expected identifier at start of assignment, found {:?}",
                other
            )),
        };

        self.expect(Token::Eq);

        let expr = self.parse_expr();

        if self.current_token == Token::Newline {
            self.bump();
        }

        Stmt::Assign { name, expr }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        self.bump(); // съели 'return'

        if self.current_token == Token::Newline || self.current_token == Token::RBrace {
            if self.current_token == Token::Newline {
                self.bump();
            }
            Stmt::Return(None)
        } else {
            let expr = self.parse_expr();
            if self.current_token == Token::Newline {
                self.bump();
            }
            Stmt::Return(Some(expr))
        }
    }

    /* ================== БЛОКИ И ВЕТВЛЕНИЯ ================== */

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::LBrace);
        self.skip_newlines();
        let mut stmts = Vec::new();

        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            let stmt = self.parse_stmt();
            stmts.push(stmt);
            self.skip_newlines();
        }
        self.expect(Token::RBrace);
        stmts
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        self.bump(); // съели 'if'

        let cond = self.parse_expr();

        let then_branch = self.parse_block();

        let mut else_if_branches: Vec<Stmt> = Vec::new();

        self.skip_newlines();

        loop {
            if self.current_token == Token::KwElseIf {
                self.bump(); // съели 'elif'

                let cond = self.parse_expr();
                let then_branch = self.parse_block();

                else_if_branches.push(Stmt::ElseIfBranch { cond, then_branch });

                self.skip_newlines();
            } else {
                break;
            }
        }

        let else_branch = if self.current_token == Token::KwElse {
            self.bump(); // съели 'else'
            let block = self.parse_block();
            block
        } else {
            Vec::new()
        };

        Stmt::Branch {
            cond,
            then_branch,
            else_if_branches,
            else_branch,
        }
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        self.bump(); // съели 'while'
        let cond = self.parse_expr();
        let body = self.parse_block();

        Stmt::While { cond, body }
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        self.bump(); // съели 'for'

        match &self.current_token {
            // ---------- foreach: for x in xs { ... } ----------
            Token::Ident(name) => {
                let var_name = name.clone();
                self.bump(); // съели имя

                if self.current_token != Token::KwIn {
                    self.error("invalid foreach statement: expected 'in'");
                }
                self.bump(); // съели 'in'

                let iter_expr = self.parse_expr();
                let body = self.parse_block();

                Stmt::ForEach {
                    var_name,
                    iter_expr,
                    body,
                }
            }

            // ---------- C-style for: for ( init ; cond ; step ) { ... } ----------
            Token::LParen => {
                self.bump(); // съели '('

                // --- init: либо пусто, либо обычный statement (var, assign, exprstmt) ---
                let init: Option<Box<Stmt>> = if self.current_token == Token::Semi {
                    // сразу ';' -> пустой init
                    None
                } else {
                    // парсим statement до ';'
                    let init_stmt = self.parse_stmt();
                    Some(Box::new(init_stmt))
                };

                // ожидаем ';'
                self.expect(Token::Semi);

                // --- cond: либо пусто, либо выражение до следующего ';' ---
                let cond: Option<Expr> = if self.current_token == Token::Semi {
                    // пустое условие -> бесконечный цикл (как for(;;))
                    None
                } else {
                    Some(self.parse_expr())
                };

                // ожидаем ';'
                self.expect(Token::Semi);

                // --- step: либо пусто, либо statement до ')' ---
                let step: Option<Box<Stmt>> = if self.current_token == Token::RParen {
                    None
                } else {
                    let step_stmt = self.parse_stmt();
                    Some(Box::new(step_stmt))
                };

                // ожидаем ')'
                self.expect(Token::RParen);

                // тело — обычный блок { ... }
                let body = self.parse_block();

                Stmt::For {
                    init,
                    cond,
                    step,
                    body,
                }
            }

            other => self.error(&format!("invalid for-statement start: {:?}", other)),
        }
    }

    fn parse_stmt(&mut self) -> Stmt {
        // eprintln!("[DEBUG] parse_stmt: current_token = {:?}", self.current_token);

        match self.current_token {
            Token::Kwvar => self.parse_var_decl(),
            Token::KwIf => self.parse_if_stmt(),
            Token::KwWhile => self.parse_while_stmt(),
            Token::KwFor => self.parse_for_stmt(),
            Token::KwReturn => self.parse_return_stmt(),

            Token::Ident(_) => {
                // либо присваивание, либо выражение / вызов
                if self.peek_token() == Token::Eq {
                    self.parse_assign_stmt()
                } else {
                    let expr = self.parse_expr();
                    if self.current_token == Token::Newline {
                        self.bump();
                    }
                    Stmt::ExprStmt(expr)
                }
            }

            _ => {
                let expr = self.parse_expr();
                if self.current_token == Token::Newline {
                    self.bump();
                }
                Stmt::ExprStmt(expr)
            }
        }
    }

    /* ==================== ФУНКЦИИ / ПРОГРАММА ==================== */

    fn parse_function(&mut self) -> Function {
        self.bump(); // съели 'func'

        let name = match &self.current_token {
            Token::Ident(n) => {
                let s = n.clone();
                self.bump();
                s
            }
            other => self.error(&format!(
                "expected function name after 'func', found {:?}",
                other
            )),
        };

        self.expect(Token::LParen);

        let mut params: Vec<(String, Type)> = Vec::new();

        if self.current_token != Token::RParen {
            loop {
                let param_name = match &self.current_token {
                    Token::Ident(n) => {
                        let s = n.clone();
                        self.bump();
                        s
                    }
                    other => self.error(&format!("expected parameter name, found {:?}", other)),
                };

                self.expect(Token::Colon);

                let param_type = self.parse_type();

                params.push((param_name, param_type));

                if self.current_token == Token::Comma {
                    self.bump();
                    continue;
                } else {
                    break;
                }
            }
        }

        self.expect(Token::RParen);

        let body = self.parse_block();

        Function { name, params, body }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions: Vec<Function> = Vec::new();
        let mut stmts: Vec<Stmt> = Vec::new();

        self.skip_newlines();

        while self.current_token != Token::EOF {
            match self.current_token {
                Token::KwFunc => {
                    let func = self.parse_function();
                    functions.push(func);
                }
                _ => {
                    let stmt = self.parse_stmt();
                    stmts.push(stmt);
                }
            }
            self.skip_newlines();
        }

        Program { functions, stmts }
    }
}
