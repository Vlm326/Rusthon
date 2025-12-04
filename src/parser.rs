// parser.rs
use crate::ast::{BinOp, Expr, Program, Stmt, Type};
use crate::interpreter::Value;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let first = lexer.next_token();
        Self {
            lexer,
            current_token: first,
        }
    }

    fn bump(&mut self) {
        self.current_token = self.lexer.next_token();
    }

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
                    panic!("Expected ')'");
                }
                self.bump();
                expr
            }
            Token::LBracket => self.parse_list_literal(),
            _ => panic!("Unexpected primary token: {:?}", self.current_token),
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
            other => panic!("can only call functions by name"),
        };
        // съели (
        self.bump();
        let mut args: Vec<Expr> = Vec::new();
        if self.current_token != Token::RParen {
            loop {
                let arg = self.parse_expr();
                args.push(arg);
                if self.current_token == Token::Comma {
                    self.bump();
                    continue;
                }
                if self.current_token == Token::RParen {
                    break;
                }
            }
            if self.current_token != Token::RParen {
                panic!("Expected ) at the end of the function call");
            }
            self.bump();
        }
        Expr::Call {
            callee: callee_name,
            args: args,
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

            // съели оператор
            self.bump();

            // правую часть парсим как term (чтобы * и / были приоритетнее)
            let rhs = self.parse_term();

            node = Expr::Binary {
                left: Box::new(node),
                op,
                right: Box::new(rhs),
            };
        }

        node
    }

    fn parse_var_decl(&mut self) -> Stmt {
        self.bump();

        let name = match &self.current_token {
            Token::Ident(n) => {
                let s = n.clone();
                self.bump();
                s
            }
            other => panic!("expected identifier after 'var', found {:?}", other),
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
    fn parse_stmt(&mut self) -> Stmt {
        match self.current_token {
            Token::Kwvar => self.parse_var_decl(),
            Token::KwIf => self.parse_if_stmt(),
            Token::KwWhile => self.parse_while_stmt(),
            Token::KwFor => self.parse_for_stmt(),
            _ => {
                let expr = self.parse_expr();
                if self.current_token == Token::Newline {
                    self.bump();
                }
                Stmt::ExprStmt(expr)
            }
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut stmts: Vec<Stmt> = Vec::new();
        self.skip_newlines();
        while self.current_token != Token::EOF {
            stmts.push(self.parse_stmt());
            self.skip_newlines();
        }
        Program { stmts }
    }

    fn expect(&mut self, expected: Token) {
        if self.current_token == expected {
            self.bump();
        } else {
            panic!("expected {:?}, found {:?}", expected, self.current_token);
        }
    }

    fn skip_newlines(&mut self) {
        while let Token::Newline = self.current_token {
            self.bump();
        }
    }
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
            other => panic!("expected type name, found {:?}", other),
        }
    }

    //====== branching ======
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
        // current_token == KwIf
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
        self.bump();
        let cond = self.parse_expr();
        let body = self.parse_block();

        Stmt::While { cond, body }
    }
    fn parse_for_stmt(&mut self) -> Stmt {
        self.bump();

        match &self.current_token {
            Token::LParen => {
                self.bump();
                let cond = self.parse_expr();
                if self.current_token != Token::RParen {
                    panic! {"Invalid for statement"};
                }
                let body = self.parse_block();
                Stmt::For { cond, body }
            }
            Token::Ident(name) => {
                let var_name = name.clone();
                self.bump();
                if self.current_token != Token::KwIn {
                    panic!("Invalid foreach statement");
                }
                self.bump();
                let iter_expr = self.parse_expr();
                let body = self.parse_block();
                Stmt::ForEach {
                    var_name,
                    iter_expr,
                    body,
                }
            }
            _ => panic!("Invalid for statement"),
        }
    }
}
