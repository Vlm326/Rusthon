use crate::ast::{BinOp, Expr, Program, Stmt, Type};
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
            match self.current_token {
                Token::Plus => {
                    self.bump();
                    let rhs = self.parse_term();
                    node = Expr::Binary {
                        left: Box::new(node),
                        op: BinOp::Add,
                        right: Box::new(rhs),
                    };
                }
                Token::Minus => {
                    self.bump();
                    let rhs = self.parse_term();
                    node = Expr::Binary {
                        left: Box::new(node),
                        op: BinOp::Sub,
                        right: Box::new(rhs),
                    };
                }
                _ => break,
            }
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
}
