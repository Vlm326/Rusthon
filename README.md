# Rusthon

![CI](https://github.com/Vlm326/Rusthon/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/github/license/Vlm326/Rusthon)
![Issues](https://img.shields.io/github/issues/Vlm326/Rusthon)
![PRs](https://img.shields.io/github/issues-pr/Vlm326/Rusthon)
![Stars](https://img.shields.io/github/stars/Vlm326/Rusthon)

**Rusthon** is a minimal, statically-typed, interpreted programming language implemented in Rust. It is an **academic / educational project** — designed solely for learning how lexers, parsers, ASTs, interpreters, and runtime environments work under the hood.

> **⚠️ DISCLAIMER**: Rusthon is **not production-ready**. It is a toy language with no stability guarantees, no package ecosystem, and no performance optimizations. Do not use it for real-world applications, data processing, or any task where correctness or reliability matters. The language exists exclusively for educational exploration of compiler and interpreter internals.

---

## Overview

Rusthon compiles (or rather, interprets) a C-like syntax with static typing and lexical scoping. The implementation covers the full pipeline from source text to execution:

- **Lexer** — tokenization of raw source into `Token` stream
- **Parser** — recursive-descent parsing into an Abstract Syntax Tree (AST)
- **Interpreter** — tree-walking evaluation over an environment stack

### Supported features

| Feature        | Description |
|----------------|-------------|
| Types          | `int`, `bool`, `str`, `list` |
| Variables      | Explicit `var name: type = value` declaration |
| Control flow   | `if` / `elif` / `else`, `while`, `for` |
| Functions      | User-defined `func name(params) { body }` with `return` |
| Builtins       | `print(...)`, `len(x)`, `range(...)` |
| Scoping        | Lexical scoping via environment stack (`push_env` / `pop_env`) |

---

## Example

```rht
func fact(n: int) {
    var res: int = 1
    var i: int = 1
    while (i <= n) {
        res = res * i
        i = i + 1
    }
    return res
}

print("fact(5) =", fact(5))
```

More examples can be found in the [`exemples/`](exemples/) directory.

---

## Build & Run

### Prerequisites

- Rust toolchain (stable)

### Build

```bash
git clone https://github.com/Vlm326/Rusthon.git
cd Rusthon
cargo build --release
```

### Run

```bash
./target/release/Rusthon path/to/program.rht
```

The interpreter expects a single `.rht` file as its argument.

---

## Project Architecture

```
src/
├── ast.rs           — AST node definitions (Expr, Stmt, Function, Program, Type, BinOp)
├── lexer.rs         — Lexer: source text → Token stream
├── parser.rs        — Parser: Token stream → AST (recursive descent)
├── interpreter.rs   — Interpreter: AST → execution (tree-walking, environment stack)
├── stdlib.rs        — Built-in functions: print, len, range
└── main.rs          — Entry point: wires lexer → parser → interpreter
```

Each module is narrowly scoped and intentionally kept simple to illustrate the core concepts of language implementation.

### Data flow

```
Source (.rht) ──▶ Lexer ──▶ Tokens ──▶ Parser ──▶ AST ──▶ Interpreter ──▶ Output
```

---

## Formal grammar (abridged BNF)

```text
program       ::= (function | stmt)* EOF
function      ::= "func" IDENT "(" param_list? ")" block
stmt          ::= var_decl | assign | if_stmt | while_stmt | for_stmt | return_stmt | expr_stmt
var_decl      ::= "var" IDENT ":" type "=" expr
block         ::= "{" stmt* "}"
expr          ::= term (("+" | "-" | "==" | "!=" | "<" | "<=" | ">" | ">=") term)*
term          ::= factor (("*" | "/") factor)*
factor        ::= primary | primary "(" arg_list? ")"
primary       ::= INT_LITERAL | STR_LITERAL | "true" | "false" | IDENT | "(" expr ")" | list_literal
```

---

## Motivation

Rusthon was created to gain hands-on experience with:

- Tokenization and lexical analysis
- Recursive-descent parsing and operator precedence
- Abstract syntax tree design and traversal
- Environment-based scoping and symbol resolution
- Tree-walking interpretation in Rust

It is **not** intended as a usable programming language.

---

## License

See [`LICENSE`](LICENSE).
