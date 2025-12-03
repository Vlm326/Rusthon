mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let src = r#"
var x: int = 10
var y: int = x + 20
y
"#;

    let lexer = Lexer::new(src);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    println!("AST:\n{:#?}", program);

    let mut interp = Interpreter::new();
    interp.run(&program);
}
