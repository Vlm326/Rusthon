# Rusthon🦀

![CI](https://github.com/Vlm326/Rusthon/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/github/license/Vlm326/Rusthon)
![Issues](https://img.shields.io/github/issues/Vlm326/Rusthon)
![PRs](https://img.shields.io/github/issues-pr/Vlm326/Rusthon)
![Stars](https://img.shields.io/github/stars/Vlm326/Rusthon)


Небольшой интерпретируемый язык программирования, написанный на Rust ради BLAZING перформанса.  
Цель проекта — пощупать, как реально работают **лексер, парсер, AST, интерпретатор**, области видимости и простая стандартная библиотека.

Rusthon — минималистичный, но уже довольно «живой» язык:

- статическая типизация (`int`, `bool`, `str`, `list`);
- переменные и присваивания;
- `if / elif / else`;
- циклы `while` и два варианта `for`;
- пользовательские функции `func`;
- стандартные функции: `print`, `len`, `range`;
- списки и `for … in` по спискам, строкам и диапазонам;
- лексические области видимости (стек окружений) и вызовы функций.

---

## Содержание

- [Возможности языка](#возможности-языка)
  - [Типы](#типы)
  - [Переменные](#переменные)
  - [Выражения и операторы](#выражения-и-операторы)
  - [Условия](#условия)
  - [Циклы](#циклы)
  - [Функции](#функции)
  - [Списки](#списки)
  - [Стандартная библиотека](#стандартная-библиотека)
- [Пример программы](#пример-программы)
- [Сборка и запуск](#сборка-и-запуск)
  - [Требования](#требования)
  - [Сборка](#сборка)
  - [Запуск](#запуск)
- [Архитектура проекта](#архитектура-проекта)
  - [Лексер (`lexer.rs`)](#лексер-lexerrs)
  - [AST (`ast.rs`)](#ast-astrs)
  - [Парсер (`parser.rs`)](#парсер-parserrs)
  - [Интерпретатор (`interpreter.rs`)](#интерпретатор-interpreterrs)
  - [Стандартная библиотека (`stdlib.rs`)](#стандартная-библиотека-stdlibrs)
  - [Точка входа (`main.rs`)](#точка-входа-mainrs)
- [Язык Rusthon формально](#язык-rusthon-формально)
  - [Пример синтаксиса в духе BNF](#пример-синтаксиса-в-духе-bnf)
- [Как расширять язык](#как-расширять-язык)
  - [Добавить встроенную функцию](#добавить-встроенную-функцию)
  - [Добавить новый оператор](#добавить-новый-оператор)
- [Планы и TODO](#планы-и-todo)
- [Лицензия](#лицензия)

---

## Возможности языка

### Типы

Поддерживаются базовые типы:

- `int` — целое число (`i64`);
- `bool` — логический тип: `true` / `false`;
- `str` — строка;
- `list` — список значений языка (`list` пока гомогенность не проверяет строго, но хранит `Vec<Value>`).

Внутренний тип интерпретатора:

```rust
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Unit, // "ничего", используется как тип результата у print/return без значения и т.п.
}
````

### Переменные

Объявление переменных — через `var` с явным типом:

```rht
var x: int = 10
var msg: str = "hello"
var ok: bool = true
var xs: list = [1, 2, 3]
```

Присваивание — просто `=`:

```rht
x = x + 1
msg = "new message"
```

Тип проверяется при **инициализации** (`VarDecl`). Дальше типы не меняются.

### Выражения и операторы

Поддерживаются:

* арифметика: `+`, `-`, `*`, `/`;
* сравнения: `==`, `!=`, `<`, `<=`, `>`, `>=`.

Примеры:

```rht
var a: int = 2 + 3 * 4
var b: bool = a > 5
var s: str = "hello " + "world"
```

### Условия

Классический `if / elif / else` с круглой скобкой вокруг условия и `{}` для блока:

```rht
if (x < 0) {
    print("x < 0")
} elif (x == 0) {
    print("x == 0")
} else {
    print("x > 0")
}
```

В AST это:

```rust
Branch {
    cond: Expr,
    then_branch: Vec<Stmt>,
    else_if_branches: Vec<Stmt>, // внутри Stmt::ElseIfBranch
    else_branch: Vec<Stmt>,
}
```

### Циклы

#### `while`

```rht
var i: int = 0
while (i < 3) {
    print(i)
    i = i + 1
}
```

В интерпретаторе условие должно давать `bool`, иначе — panic.

#### `for` (вариант foreach)

Если после `for` сразу идёт идентификатор и `in`, это foreach-форма:

```rht
// 1) for i in N:  i = 0..N-1
for i in 5 {
    print(i)
}

// 2) for ch in "hi!"
for ch in "hi!" {
    print(ch)
}

// 3) for v in список
var xs: list = [10, 20, 30]
for v in xs {
    print(v)
}
```

Интерпретация:

* `for i in 5` — `i` пробегает от `0` до `4`;
* `for ch in "hi!"` — `ch` — строка длиной 1 (символ);
* `for v in xs` — `v` — элементы списка.

#### `for` (вариант с условием)

Второй вариант — псевдо-C-стиль, но в упрощённом виде: `for (expr) { ... }`.

Сейчас он реализован как «цикл с условием», то есть фактически **аналог while** с синтаксисом:

```rht
for (x < 10) {
    print(x)
    x = x + 1
}
```

В AST:

```rust
For {
    cond: Expr,
    body: Vec<Stmt>,
}
```

(инициализация и шаг пока не вынесены явно в грамматику, это можно добавить позже.)

### Функции

Определение функции:

```rht
func add(a: int, b: int) {
    return a + b
}

func fact(n: int) {
    var res: int = 1
    var i: int = 1
    while (i <= n) {
        res = res * i
        i = i + 1
    }
    return res
}
```

Возврат значения — через `return`.
`return` может быть без аргумента — тогда возвращается `Unit`.

Функции хранятся в AST как:

```rust
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub body: Vec<Stmt>,
}
```

И в `Program`:

```rust
pub struct Program {
    pub functions: Vec<Function>,
    pub stmts: Vec<Stmt>, // глобальные операторы
}
```

Вызов функции:

```rht
var s: int = add(2, 3)
print(s)
```

При вызове:

* создаётся новый `scope` (новый `HashMap` в `env_stack`);
* параметры кладутся как локальные переменные;
* выполняется тело; при `return` значение пробрасывается наружу;
* локальная область видимости удаляется.

### Списки

Литералы списков:

```rht
var xs: list = [1, 2, 3]
var strs: list = ["hello", "world"]
```

Внутри интерпретатора:

```rust
Expr::ListLiteral(Vec<Expr>) → Value::List(Vec<Value>)
```

Списки используются, в частности, для `for v in xs` и в функции `len(xs)`.

### Стандартная библиотека

Реализована в `stdlib.rs` через функцию:

```rust
pub fn call_builtin(name: &str, args: &Vec<Value>) -> Option<Value>
```

Сейчас есть:

#### `print(...)`

Выводит все аргументы через пробел и добавляет перевод строки:

```rht
print("answer =", 42)
```

Поддерживает `int`, `bool`, `str`, `list`, `Unit`.

#### `len(x)`

Возвращает длину строки или списка:

```rht
var s: str = "hello"
var xs: list = [1, 2, 3]

print(len(s))   # 5
print(len(xs))  # 3
```

#### `range(...)`

Создаёт список целых чисел:

```rht
range(n)      # [0, 1, 2, ..., n-1]
range(a, b)   # [a, a+1, ..., b-1]

for i in range(5) {
    print(i)
}
```

---

## Пример программы

Фрагмент демонстрации возможностей языка:

```rht
func add(a: int, b: int) {
    return a + b
}

func fact(n: int) {
    var res: int = 1
    var i: int = 1
    while (i <= n) {
        res = res * i
        i = i + 1
    }
    return res
}

func test_if(x: int) {
    if (x < 0) {
        print("x < 0")
    } elif (x == 0) {
        print("x == 0")
    } else {
        print("x > 0")
    }
}

func main_logic() {
    print("== функции add и fact ==")
    var s: int = add(2, 3)
    print("add(2, 3) =")
    print(s)

    var f: int = fact(5)
    print("fact(5) =")
    print(f)

    print("== if / elif / else ==")
    test_if(-1)
    test_if(0)
    test_if(1)

    print("== foreach по range и list ==")
    for i in range(5) {
        print(i)
    }

    var xs: list = [10, 20, 30]
    for v in xs {
        print(v)
    }
}

var answer: int = 42
print("== глобальные переменные и main_logic ==")
print(answer)
main_logic()
```

---

## Сборка и запуск

### Требования

* Установленный **Rust toolchain** (стабильная версия);
* `cargo` в `$PATH`.

Проверить:

```bash
rustc --version
cargo --version
```

### Сборка

Клонируем репозиторий и собираем:

```bash
git clone https://github.com/<USER>/<REPO>.git
cd <REPO>

# Debug-сборка
cargo build

# Release-сборка
cargo build --release
```

После `cargo build --release` бинарник появится в:

```text
target/release/Rusthon
```

### Запуск

Rusthon принимает путь к `.rht`-файлу первым аргументом:

```bash
./target/release/Rusthon path/to/program.rht
```

Если файла нет или расширение не `.rht`, интерпретатор завершится с ошибкой.

Пример:

```bash
./target/release/Rusthon examples/demo.rht
```

---

## Архитектура проекта

Структура модулей примерно такая:

```text
src/
  ast.rs          // описание AST: Expr, Stmt, Function, Program, Type, BinOp
  lexer.rs        // лексер: разбор текста в токены
  parser.rs       // парсер: токены -> AST
  interpreter.rs  // интерпретатор: выполнение AST
  stdlib.rs       // встроенные функции (print, len, range, ...)
  main.rs         // точка входа: связывает всё вместе
```

### Лексер (`lexer.rs`)

Отвечает за разбор сырого текста в токены (`Token`):

* пропускает пробелы и табы;
* определяет:

  * `Ident(String)`, `IntLiteral(i64)`, `StrLiteral(String)`;
  * ключевые слова: `var`, `func`, `if`, `elif`, `else`, `while`, `for`, `in`, `true`, `false`, `return` и т.д.
  * операторы и разделители: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `(`, `)`, `{`, `}`, `[`, `]`, `:`, `,`.

Используется парсером как итератор:

```rust
let lexer = Lexer::new(&program_text);
let mut parser = Parser::new(lexer);
```

### AST (`ast.rs`)

**AST (Abstract Syntax Tree)** — это абстрактное синтаксическое дерево, структура данных, которая описывает программу не в виде строки, а в виде иерархии узлов: выражения, операторы, функции и т.п.

Примеры:

* `Expr::Binary` — бинарное выражение (`a + b`, `x < y`, ...);
* `Stmt::VarDecl` — объявление переменной;
* `Stmt::While` — цикл `while`;
* `Stmt::ForEach` — `for v in xs { ... }`;
* `Function` — пользовательская функция;
* `Program` — корень дерева (список функций + глобальных операторов).

Интерпретатор ходит по этому дереву и выполняет программу.

### Парсер (`parser.rs`)

Парсер преобразует поток `Token` в AST:

* реализует рекурсивный спуск;

* учитывает приоритет операторов:

  * `parse_primary` → числа, строки, идентификаторы, `(...)`, списки `[...]`;
  * `parse_factor` → умножение/деление и вызовы `func(...)`;
  * `parse_term` → `*` и `/`;
  * `parse_expr` → `+`, `-`, сравнения `==`, `!=`, `<`, `>`, ...

* парсит:

  * объявления переменных: `var name: type = expr`;
  * присваивания: `name = expr`;
  * `if / elif / else`;
  * `while` и `for`;
  * `func name(params) { body }`;
  * `return`.

В случае ошибки парсер вызывает `error(...)`, выводит сообщение с текущим токеном и завершает процесс.

### Интерпретатор (`interpreter.rs`)

Исполняет AST:

* хранит **стек окружений**:

  ```rust
  struct Interpreter {
      env_stack: Vec<HashMap<String, Value>>,
      functions: HashMap<String, Function>,
  }
  ```

* при старте:

  * загружает все `Function` в `functions`;
  * выполняет глобальные операторы (`program.stmts`).

* области видимости:

  * `push_env()` / `pop_env()`;
  * новый scope создаётся:

    * при входе в функцию;
    * при выполнении блока (`exec_block`) for/while/if-ветки.

* переменные:

  * `define_var(name, value)` — кладёт в текущий (верхний) scope;
  * `assign_var` — ищет переменную снизу вверх по стеку и обновляет значение;
  * `get_var` — ищет переменную при чтении.

* выражения:

  * `eval_expr(&Expr) -> Value`;
  * арифметика и сравнения в `eval_bin`.

* операторы:

  * `exec_stmt(&Stmt) -> Option<Value>`:

    * `None` — обычное выполнение;
    * `Some(value)` — проброшенный `return` из функции.

### Стандартная библиотека (`stdlib.rs`)

Содержит реализацию встроенных функций окружения:

* `print(...)`
* `len(x)`
* `range(...)`

Интерпретатор сначала пробует вызвать builtin:

```rust
if let Some(result) = stdlib::call_builtin(&callee, &value_args) {
    return result;
}
```

А если такое имя не найдено, пытается вызвать пользовательскую функцию.

### Точка входа (`main.rs`)

Связывает всё вместе:

```rust
use std::env;
use std::fs;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    // Находим первый аргумент, который заканчивается на ".rht"
    let args: Vec<String> = env::args().collect();

    let path = args
        .iter()
        .find(|arg| arg.ends_with(".rht"))
        .expect("❌ You must pass a .rht program file as an argument.")
        .clone();

    // Читаем файл программы
    let program_text = fs::read_to_string(&path)
        .expect("❌ Failed to read the program file.");

    // Лексер + парсер → AST
    let lexer = Lexer::new(&program_text);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    // Интерпретатор
    let mut interp = Interpreter::new();
    interp.run(&program);
}
```

---

## Язык Rusthon формально

### Пример синтаксиса в духе BNF

Это не строгий EBNF, но даёт общее ощущение грамматики:

```text
program       ::= (function | stmt)* EOF

function      ::= "func" IDENT "(" param_list? ")" block
param_list    ::= param ("," param)*
param         ::= IDENT ":" type

type          ::= "int" | "bool" | "str" | "list"

stmt          ::= var_decl
                | assign
                | if_stmt
                | while_stmt
                | for_stmt
                | return_stmt
                | expr_stmt

var_decl      ::= "var" IDENT ":" type "=" expr NEWLINE?

assign        ::= IDENT "=" expr NEWLINE?

if_stmt       ::= "if" "(" expr ")" block
                  ("elif" "(" expr ")" block)*
                  ("else" block)?

while_stmt    ::= "while" "(" expr ")" block

for_stmt      ::= "for" "(" expr ")" block
                | "for" IDENT "in" expr block

return_stmt   ::= "return" expr? NEWLINE?

expr_stmt     ::= expr NEWLINE?

block         ::= "{" NEWLINE* stmt* NEWLINE* "}"

expr          ::= term (("+" | "-" | "==" | "!=" | "<" | "<=" | ">" | ">=") term)*

term          ::= factor (("*" | "/") factor)*

factor        ::= primary
                | primary "(" arg_list? ")"  // вызовы функций

primary       ::= INT_LITERAL
                | STR_LITERAL
                | "true"
                | "false"
                | IDENT
                | "(" expr ")"
                | list_literal

list_literal  ::= "[" (expr ("," expr)*)? "]"

arg_list      ::= expr ("," expr)*
```

---

## Как расширять язык

### Добавить встроенную функцию

1. Открыть `stdlib.rs`.
2. Добавить новый кейс в `match name`:

```rust
"upper" => {
    if args.len() != 1 {
        panic!("upper() expects exactly 1 argument");
    }

    let s = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => panic!("upper() expects a string"),
    };

    Some(Value::Str(s.to_uppercase()))
}
```

3. Теперь в языке можно писать:

```rht
print(upper("hello"))
```

### Добавить новый оператор

1. Добавить вариант в `BinOp` (в `ast.rs`):

```rust
pub enum BinOp {
    // ...
    Mod, // %
}
```

2. В `lexer.rs` убедиться, что `%` лексится как отдельный токен (`Percent` уже есть).
3. В `parser.rs` добавить разбор `%` в `parse_term`:

```rust
Token::Percent => {
    self.bump();
    let rhs = self.parse_factor();
    node = Expr::Binary {
        left: Box::new(node),
        op: BinOp::Mod,
        right: Box::new(rhs),
    };
}
```

4. В `interpreter.rs` добавить обработку:

```rust
BinOp::Mod => match (left, right) {
    (Value::Int(l), Value::Int(r)) => Value::Int(l % r),
    _ => panic!("Type error in modulo"),
},
```

---

## Планы и TODO

Идеи для развития Rusthon:

* [ ] Логические операторы `&&`, `||`, унарный `!` с приоритетами и short-circuit.
* [ ] Унарный минус (`-x`).
* [ ] Более «настоящий» C-style `for (init; cond; step)` с явными полями в AST.
* [ ] Комментарии (`# ...` или `// ...`) на уровне лексера.
* [ ] Нормальная система ошибок (`Result` вместо тотальных `panic!`).
* [ ] Типизация списков (`list[int]`, `list[str]` и т.п.).
* [ ] Встроенный `main()` по умолчанию (если функция `main` определена — вызывать её автоматически).
* [ ] Юнит-тесты (`cargo test`) для лексера, парсера и интерпретатора.
* [ ] CI (GitHub Actions) с автоматической сборкой и запуском тестов.

---

## Лицензия

Лицензия указана в файле [`LICENSE`](LICENSE).


