mod ast;
mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let src = r#"
var x: int = 10
var y: int = x + 20
"#;

    let lexer = Lexer::new(src);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    println!("{:#?}", program);
}
