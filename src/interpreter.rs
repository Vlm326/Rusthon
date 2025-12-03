use crate::ast::{BinOp, Expr, Program, Stmt, Type};
use crate::stdlib;
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
}
pub struct Interpreter {
    env: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }
    pub fn run(&mut self, program: &Program) {
        for stmt in &program.stmts {
            self.exec_stmt(&stmt);
        }
    }
    fn exec_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, ty, init } => {
                let value = self.eval_expr(init);
                if !Self::value_matches_type(&value, ty) {
                    panic!(
                        "type error: variable '{}' declared as {:?}, but value is {:?}",
                        name, ty, value
                    );
                }
                self.env.insert(name.clone(), value);
            }
            Stmt::ExprStmt(expr) => {
                let _v = self.eval_expr(expr);
            }
        }
    }
    fn value_matches_type(value: &Value, ty: &Type) -> bool {
        match (value, ty) {
            (Value::Int(_), Type::Int) => true,
            (Value::Bool(_), Type::Bool) => true,
            (Value::Str(_), Type::Str) => true,
            _ => false,
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Int(n) => Value::Int(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Str(s) => Value::Str(s.clone()),

            Expr::Var(name) => self
                .env
                .get(name)
                .cloned()
                .unwrap_or_else(|| panic!("Undefined type {}", name)),
            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                self.eval_bin(l, op, r)
            }
            Expr::Call { callee, args } => self.eval_call(callee, args),
        }
    }

    fn eval_call(&mut self, callee: &String, args: &Vec<Expr>) -> Value {
        let value_args: Vec<Value> = args.iter().map(|expr| self.eval_expr(expr)).collect();
        if let Some(result) = stdlib::call_builtin(&callee, &value_args) {
            result
        } else {
            panic!("Unknown argumets or function");
        }
    }

    fn eval_bin(&self, left: Value, op: &BinOp, right: Value) -> Value {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                (Value::Str(left), Value::Str(right)) => Value::Str(left + &right),
                _ => panic!("Type error"),
            },
            BinOp::Sub => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                _ => panic!("Type error, you can't subtract not a number values"),
            },
            BinOp::Div => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
                _ => panic!("Type error, you can't divide not a number values"),
            },
            BinOp::Mul => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
                _ => panic!("Type error, you can't multiply not a number values"),
            },
            BinOp::Eq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left == right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool(left == right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.eq(&right)),
                _ => panic!("Type error in equal"),
            },
            BinOp::Gt => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left > right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool((left as i8) > right as i8),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() > right.len()),
                _ => panic!("Type error in greater then"),
            },
            BinOp::GtEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left >= right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool((left as i8) >= right as i8),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() >= right.len()),
                _ => panic!("Type error in greater or equal then"),
            },
            BinOp::Lt => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left < right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool((left as i8) < right as i8),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() < right.len()),
                _ => panic!("Type error in less then"),
            },
            BinOp::LtEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left <= right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool((left as i8) <= right as i8),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() <= right.len()),
                _ => panic!("Type error in less or equal then"),
            },
            BinOp::NotEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left != right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool((left as i8) != right as i8),
                (Value::Str(left), Value::Str(right)) => Value::Bool(!left.eq(&right)),
                _ => panic!("Type error in not equal"),
            },
        }
    }
}
