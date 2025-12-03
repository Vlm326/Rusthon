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
    // Получаем аргументы командной строки
    let args: Vec<String> = env::args().collect();

    // Ищем файл с расширением .rht
    let path = args
        .iter()
        .find(|arg| arg.ends_with(".rht"))
        .expect("❌ You must pass a .rht program file as an argument.")
        .clone();

    // Читаем текст программы
    let program_text = fs::read_to_string(&path).expect("❌ Failed to read the program file.");

    // Создаём лексер на основе текста
    let lexer = Lexer::new(&program_text);

    // Парсер принимает лексер
    let mut parser = Parser::new(lexer);

    // Парсим AST
    let program = parser.parse_program();

    // println!("AST:\n{:#?}", program);

    // Создаём интерпретатор
    let mut interp = Interpreter::new();

    // Исполняем программу
    interp.run(&program);
}
