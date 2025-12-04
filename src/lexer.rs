// lexer.rs
//
// Лексер (tokenizer) для языка Rusthon.
// Здесь описаны:
//   - множество лексем (Token)
//   - структура Lexer, которая бегает по входной строке и выдаёт токены
//
// Поток токенов потом ест парсер.

// ===== Токены =====

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // --- Структурные токены потока ---
    /// Перевод строки `\n`
    Newline,
    /// Конец файла / входной строки
    EOF,

    // --- Идентификаторы и ключевые слова ---
    /// Идентификатор: имя переменной, функции и т.п.
    Ident(String),

    /// Ключевое слово `var`
    Kwvar,
    /// Ключевое слово `mut` (пока не используется в парсере, но зарезервировано)
    KwMut,
    /// Ключевое слово `func`
    KwFunc,
    /// Ключевое слово `return`
    KwReturn,
    /// Ключевое слово `if`
    KwIf,
    /// Ключевое слово `elif`
    KwElseIf,
    /// Ключевое слово `else`
    KwElse,
    /// Ключевое слово `for`
    KwFor,
    /// Ключевое слово `in` (для for-each)
    KwIn,
    /// Ключевое слово `true`
    KwTrue,
    /// Ключевое слово `false`
    KwFalse,
    /// Ключевое слово `while`
    KwWhile,

    // --- Литералы ---
    /// Целочисленный литерал: `123`
    IntLiteral(i64),
    /// Строковый литерал: `"hello"`
    StrLiteral(String),

    // --- Арифметические операторы ---
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    // --- Операторы сравнения и присваивания ---
    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // --- Знаки пунктуации / скобки ---
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }
    Colon,    // :
    Semi,     // ;
    Comma,    // ,
}

// ===== Лексер =====

/// Простой лексер по массиву символов.
/// Хранит:
///   - `input` — весь текст программы
///   - `pos`   — текущий индекс (указатель) в этом массиве
#[derive(Clone)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize, // текущий индекс в input
}

impl Lexer {
    /// Создаёт новый лексер из исходной строки.
    pub fn new(src: &str) -> Self {
        Self {
            input: src.chars().collect(),
            pos: 0,
        }
    }

    /// Подсмотреть текущий символ (без сдвига позиции).
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Считать текущий символ и сдвинуть позицию вперёд на 1.
    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        }
    }

    /// Пропустить пробелы и табы (но не перенос строки).
    fn skip_spaces(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Считать следующий токен из входа.
    ///
    /// Основной метод лексера: всё остальное — помощники.
    pub fn next_token(&mut self) -> Token {
        use Token::*;

        // сначала убираем пробелы / табы
        self.skip_spaces();

        // берём следующий символ
        let ch = match self.advance() {
            Some(c) => c,
            None => return EOF,
        };

        match ch {
            // перевод строки — отдельный токен
            '\n' => Newline,

            // цифра — начинаем читать число
            '0'..='9' => {
                // мы уже прочитали первую цифру `ch`
                self.lex_number(ch)
            }

            // буква или '_' — идентификатор или ключевое слово
            'a'..='z' | 'A'..='Z' | '_' => self.lex_ident_or_keyword(ch),

            // начало строкового литерала
            '"' => self.lex_string(),

            // односивольные операторы
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => Slash,
            '%' => Percent,

            // скобки и знаки
            '{' => LBrace,
            '}' => RBrace,
            '(' => LParen,
            ')' => RParen,
            '[' => LBracket,
            ']' => RBracket,
            ':' => Colon,
            ';' => Semi,
            ',' => Comma,

            // '=' или '=='
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    EqEq
                } else {
                    Eq
                }
            }

            // '!='
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    NotEq
                } else {
                    // на данном этапе просто паникуем,
                    // позже можно превратить в нормальную лексическую ошибку
                    panic!("Unexpected '!' without '='");
                }
            }

            // '<' или '<='
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    LtEq
                } else {
                    Lt
                }
            }

            // '>' или '>='
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    GtEq
                } else {
                    Gt
                }
            }

            // TODO: здесь можно добавить поддержку комментариев:
            //   - однострочные //...
            //   - многострочные /* ... */
            // а также сделать аккуратную систему ошибок вместо panic!
            other => panic!("Unexpected character: {:?}", other),
        }
    }

    /// Разбор целого числа.
    ///
    /// На входе уже считана первая цифра `first_digit`.
    fn lex_number(&mut self, first_digit: char) -> Token {
        let mut s = String::new();
        s.push(first_digit);

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let value = s.parse::<i64>().unwrap();
        Token::IntLiteral(value)
    }

    /// Разбор идентификатора или ключевого слова.
    ///
    /// На входе уже считан первый символ `first_char` (буква или '_').
    fn lex_ident_or_keyword(&mut self, first_char: char) -> Token {
        let mut s = String::new();
        s.push(first_char);
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Проверяем, не является ли это ключевым словом.
        match s.as_str() {
            "var" => Token::Kwvar,
            "mut" => Token::KwMut,
            "func" => Token::KwFunc,
            "return" => Token::KwReturn,
            "if" => Token::KwIf,
            "elif" => Token::KwElseIf,
            "else" => Token::KwElse,
            "while" => Token::KwWhile,
            "for" => Token::KwFor,
            "in" => Token::KwIn,
            "true" => Token::KwTrue,
            "false" => Token::KwFalse,
            _ => Token::Ident(s),
        }
    }

    /// Разбор строкового литерала `"..."`.
    ///
    /// Ожидается, что ведущая кавычка уже была съедена.
    fn lex_string(&mut self) -> Token {
        let mut s = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => break, // закрывающая кавычка
                '\n' => panic!("String literal not closed before newline"),
                _ => s.push(ch),
            }
        }

        Token::StrLiteral(s)
    }
}

// TODO:
//  - поддержка комментариев
//  - нормальная система лексических ошибок (с позициями), вместо простых panic!
//  - возможно, поддержка разных видов переноса строк (\r\n и т.п.)
