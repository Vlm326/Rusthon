// interpreter.rs
//
// Интерпретатор языка Rusthon:
//  - хранит значения (Value)
//  - поддерживает стек окружений (лексическая область видимости)
//  - исполняет операторы (Stmt)
//  - вычисляет выражения (Expr)
//  - вызывает встроенные и пользовательские функции

use crate::ast::{BinOp, Expr, Function, Program, Stmt, Type};
use crate::stdlib;
use std::{collections::HashMap, fmt::Debug};

/// Все возможные значения языка на этапе исполнения.
#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),

    /// "Пустое" значение — аналог `void` / `()` / отсутствия результата.
    Unit,
}

/// Главная структура интерпретатора.
/// Хранит:
///  - стек окружений переменных (env_stack)
///  - таблицу объявленных функций (functions)
pub struct Interpreter {
    /// Стек окружений: каждый `HashMap` — отдельный scope.
    /// Верхний (последний) элемент — текущий scope.
    env_stack: Vec<HashMap<String, Value>>,

    /// Пользовательские функции: имя -> определение.
    functions: HashMap<String, Function>,
}

impl Interpreter {
    /* ====================== КОНСТРУКЦИЯ И ENV ====================== */

    /// Создаём интерпретатор с глобальным окружением.
    pub fn new() -> Self {
        Self {
            env_stack: vec![HashMap::new()], // глобальное окружение
            functions: HashMap::new(),
        }
    }

    /// Входим в новый scope (например, при входе в блок или функцию).
    fn push_env(&mut self) {
        self.env_stack.push(HashMap::new());
    }

    /// Выходим из scope.
    fn pop_env(&mut self) {
        self.env_stack.pop().expect("env stack underflow");
    }

    /// Объявляем новую переменную в текущем scope.
    fn define_var(&mut self, name: String, value: Value) {
        self.env_stack
            .last_mut()
            .expect("no environment")
            .insert(name, value);
    }

    /// Присваиваем существующей переменной (ищем по стеку сверху вниз).
    fn assign_var(&mut self, name: &str, value: Value) {
        for env in self.env_stack.iter_mut().rev() {
            if env.contains_key(name) {
                env.insert(name.to_string(), value);
                return;
            }
        }
        panic!("assignment to undeclared variable '{}'", name);
    }

    /// Читаем значение переменной по имени (ищем в стеке сверху вниз).
    fn get_var(&self, name: &str) -> Option<Value> {
        for env in self.env_stack.iter().rev() {
            if let Some(v) = env.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    /* ======================= ЗАПУСК ПРОГРАММЫ ======================= */

    /// Запускаем программу: сначала загружаем функции, потом исполняем
    /// глобальные операторы по порядку.
    pub fn run(&mut self, program: &Program) {
        // Загружаем определения функций в таблицу.
        self.functions = program
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.clone()))
            .collect();

        // Исполняем глобальные операторы.
        for stmt in &program.stmts {
            let _ = self.exec_stmt(stmt);
        }
    }

    /* ================== ИСПОЛНЕНИЕ ОПЕРАТОРОВ (Stmt) ================= */

    /// Исполнить один оператор.
    /// Возвращает:
    ///  - Some(Value) — если встретился `return` и нужно пробросить значение наверх
    ///  - None — обычное выполнение без выхода из функции
    fn exec_stmt(&mut self, stmt: &Stmt) -> Option<Value> {
        match stmt {
            /* ----------- объявления и простые выражения ----------- */
            Stmt::VarDecl { name, ty, init } => {
                let value = self.eval_expr(init);
                if !Self::value_matches_type(&value, ty) {
                    panic!(
                        "type error: variable '{}' declared as {:?}, but value is {:?}",
                        name, ty, value
                    );
                }
                self.define_var(name.clone(), value);
                None
            }

            Stmt::ExprStmt(expr) => {
                let _v = self.eval_expr(expr);
                None
            }

            Stmt::Assign { name, expr } => {
                let value = self.eval_expr(expr);
                self.assign_var(name, value);
                None
            }

            /* --------------------- return --------------------- */
            Stmt::Return(expr_opt) => {
                let v = match expr_opt {
                    Some(e) => self.eval_expr(e),
                    None => Value::Unit,
                };
                // сигнал "вернулись из функции"
                Some(v)
            }

            /* ---------------- if / elif / else ---------------- */
            Stmt::Branch {
                cond,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                // if (...)
                if let Value::Bool(true) = self.eval_expr(cond) {
                    if let Some(v) = self.exec_block(then_branch) {
                        return Some(v);
                    }
                    return None;
                }

                // elif ...
                for branch in else_if_branches {
                    if let Stmt::ElseIfBranch { cond, then_branch } = branch {
                        if let Value::Bool(true) = self.eval_expr(cond) {
                            if let Some(v) = self.exec_block(then_branch) {
                                return Some(v);
                            }
                            return None;
                        }
                    } else {
                        // защитный assert — по идее такого не должно быть
                        panic!("non-ElseIfBranch inside else_if_branches");
                    }
                }

                // else ...
                if !else_branch.is_empty() {
                    if let Some(v) = self.exec_block(else_branch) {
                        return Some(v);
                    }
                }

                None
            }

            /* -------------------- while -------------------- */
            Stmt::While { cond, body } => {
                loop {
                    match self.eval_expr(cond) {
                        Value::Bool(true) => {
                            if let Some(v) = self.exec_block(body) {
                                // проброс return из функции наверх
                                return Some(v);
                            }
                        }
                        Value::Bool(false) => break,
                        _ => panic!("while condition must be bool"),
                    }
                }
                None
            }

            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                // отдельный scope для всего цикла:
                // init / body / step живут в одном окружении
                self.push_env();

                // init
                if let Some(init_stmt) = init.as_deref() {
                    self.exec_stmt(init_stmt);
                }

                loop {
                    // cond: если есть — проверяем, если нет — считаем true (for(;;))
                    if let Some(cond_expr) = cond {
                        match self.eval_expr(cond_expr) {
                            Value::Bool(true) => {}
                            Value::Bool(false) => break,
                            _ => panic!("for condition must be bool"),
                        }
                    }

                    // тело
                    if let Some(v) = self.exec_block(body) {
                        // проброс return из функции
                        self.pop_env();
                        return Some(v);
                    }

                    // step
                    if let Some(step_stmt) = step.as_deref() {
                        self.exec_stmt(step_stmt);
                    }
                }

                self.pop_env();
                None
            }

            /* ---------------------- for-each ---------------------- */
            Stmt::ForEach {
                var_name,
                iter_expr,
                body,
            } => {
                let iterable = self.eval_expr(iter_expr);

                match iterable {
                    // for i in 10 { ... }  -> i = 0..9
                    Value::Int(n) => {
                        if n < 0 {
                            panic!("for-each over negative int is not supported");
                        }
                        // отдельный scope для цикла
                        self.push_env();
                        for i in 0..n {
                            self.define_var(var_name.clone(), Value::Int(i));
                            if let Some(v) = self.exec_block(body) {
                                self.pop_env();
                                return Some(v);
                            }
                        }
                        self.pop_env();
                    }

                    // for ch in "hello" { ... }
                    Value::Str(s) => {
                        self.push_env();
                        for ch in s.chars() {
                            self.define_var(var_name.clone(), Value::Str(ch.to_string()));
                            if let Some(v) = self.exec_block(body) {
                                self.pop_env();
                                return Some(v);
                            }
                        }
                        self.pop_env();
                    }

                    // for x in [1, 2, 3] { ... }
                    Value::List(list) => {
                        self.push_env();
                        for v in list {
                            self.define_var(var_name.clone(), v);
                            if let Some(v) = self.exec_block(body) {
                                self.pop_env();
                                return Some(v);
                            }
                        }
                        self.pop_env();
                    }

                    _ => {
                        panic!("for-each can iterate only over int, string or list");
                    }
                }

                None
            }

            /* ------------------ прочие / не поддержано ------------------ */
            _ => panic!("Unsupported statement: {:?}", stmt),
        }
    }

    /* =================== СООТВЕТСТВИЕ ТИПОВ / VALUE =================== */

    /// Проверка: значение `value` подходит под статический тип `ty`?
    fn value_matches_type(value: &Value, ty: &Type) -> bool {
        match (value, ty) {
            (Value::Int(_), Type::Int) => true,
            (Value::Bool(_), Type::Bool) => true,
            (Value::Str(_), Type::Str) => true,
            (Value::List(_), Type::List) => true,
            _ => false,
        }
    }

    /* ================= ВЫЧИСЛЕНИЕ ВЫРАЖЕНИЙ (Expr) ================== */

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Int(n) => Value::Int(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Str(s) => Value::Str(s.clone()),

            Expr::Var(name) => self
                .get_var(name)
                .unwrap_or_else(|| panic!("Undefined variable {}", name)),

            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                self.eval_bin(l, op, r)
            }

            Expr::ListLiteral(items) => {
                let mut vals = Vec::new();
                for e in items {
                    vals.push(self.eval_expr(e));
                }
                Value::List(vals)
            }

            Expr::Call { callee, args } => self.eval_call(callee, args),
        }
    }

    /* ================== ВЫЗОВЫ ФУНКЦИЙ (BUILTIN/USER) ================= */

    /// Вызов функции (сначала пробуем stdlib, потом пользовательские).
    fn eval_call(&mut self, callee: &String, args: &Vec<Expr>) -> Value {
        let value_args: Vec<Value> = args.iter().map(|expr| self.eval_expr(expr)).collect();

        // 1) встроенные функции (stdlib)
        if let Some(result) = stdlib::call_builtin(&callee, &value_args) {
            return result;
        }

        // 2) пользовательские функции
        if let Some(func) = self.functions.get(callee).cloned() {
            return self.call_function(&func, value_args);
        }

        panic!("Unknown function '{}'", callee);
    }

    /// Вызов пользовательской функции.
    fn call_function(&mut self, func: &Function, args: Vec<Value>) -> Value {
        if func.params.len() != args.len() {
            panic!(
                "function '{}' expected {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            );
        }

        // создаём новый scope для параметров (и локальных переменных функции)
        let mut locals = HashMap::new();
        for ((param_name, _param_type), arg_val) in func.params.iter().zip(args.into_iter()) {
            locals.insert(param_name.clone(), arg_val);
        }
        self.env_stack.push(locals);

        // выполняем тело
        let mut ret = Value::Unit;
        for stmt in &func.body {
            if let Some(v) = self.exec_stmt(stmt) {
                ret = v;
                break;
            }
        }

        // выходим из функции — убираем её scope
        self.pop_env();

        ret
    }

    /* ================= БИНАРНЫЕ ОПЕРАЦИИ (BinOp) ================= */

    fn eval_bin(&self, left: Value, op: &BinOp, right: Value) -> Value {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                (Value::Str(left), Value::Str(right)) => Value::Str(left + &right),
                _ => panic!("Type error in '+'"),
            },

            BinOp::Sub => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
                _ => panic!("Type error, you can't subtract non-int values"),
            },

            BinOp::Div => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
                _ => panic!("Type error, you can't divide non-int values"),
            },

            BinOp::Mul => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
                _ => panic!("Type error, you can't multiply non-int values"),
            },

            BinOp::Eq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left == right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool(left == right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left == right),
                _ => panic!("Type error in '=='"),
            },

            BinOp::Gt => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left > right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() > right.len()),
                _ => panic!("Type error in '>'"),
            },

            BinOp::GtEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left >= right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() >= right.len()),
                _ => panic!("Type error in '>='"),
            },

            BinOp::Lt => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left < right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() < right.len()),
                _ => panic!("Type error in '<'"),
            },

            BinOp::LtEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left <= right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left.len() <= right.len()),
                _ => panic!("Type error in '<='"),
            },

            BinOp::NotEq => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left != right),
                (Value::Str(left), Value::Str(right)) => Value::Bool(left != right),
                _ => panic!("Type error in '!='"),
            },
        }
    }

    /* ===================== ВСПОМОГАТЕЛЬНОЕ: БЛОКИ ===================== */

    /// Выполнить блок `{ ... }` с собственным scope.
    /// Если внутри блока случился `return`, он пробрасывается наружу.
    fn exec_block(&mut self, body: &[Stmt]) -> Option<Value> {
        self.push_env();
        let mut ret = None;
        for s in body {
            if let Some(v) = self.exec_stmt(s) {
                ret = Some(v);
                break;
            }
        }
        self.pop_env();
        ret
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    /// Хелпер: прогнать кусок Rusthon-кода через лексер, парсер и интерпретатор.
    fn run_source(src: &str) {
        let lexer = Lexer::new(src);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        let mut interp = Interpreter::new();
        interp.run(&program);
    }

    #[test]
    fn simple_arith_and_while_does_not_panic() {
        let src = r#"
            var x: int = 0
            var sum: int = 0

            while (x < 5) {
                sum = sum + x
                x = x + 1
            }

            print("sum =", sum)
        "#;

        // Если где-то баг в лексере/парсере/интерпретаторе — тест упадёт с panic.
        run_source(src);
    }

    #[test]
    fn functions_branching_and_foreach_does_not_panic() {
        let src = r#"
            func sum_list(xs: list) {
                var acc: int = 0
                for v in xs {
                    acc = acc + v
                }
                print("sum_list =", acc)
            }

            func classify_and_print(n: int) {
                if (n < 0) {
                    print("neg")
                } elif (n == 0) {
                    print("zero")
                } else {
                    print("pos")
                }
            }

            func main() {
                var xs: list = [1, 2, 3, 4]
                sum_list(xs)

                classify_and_print(1)
                classify_and_print(0)
                classify_and_print(1)

                print("for i in 5")
                for i in 5 {
                    print(i)
                }
            }

            main()
        "#;

        run_source(src);
    }
}
