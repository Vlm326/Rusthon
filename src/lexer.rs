#[derive(Debug, Clone, PartialEq)]

pub enum Token {
    // Структурные
    Newline,
    EOF,

    // Идентификаторы и ключевые слова
    Ident(String),
    Kwvar,
    KwMut,
    KwDef,
    KwReturn,
    KwIf,
    KwElse,
    KwFor,
    KwIn,
    KwTrue,
    KwFalse,

    // Литералы
    IntLiteral(i64),
    StrLiteral(String),

    // Операторы
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // Знаки
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LCurly,   // {
    RCurly,   // }
    Colon,    // :
    Comma,    // ,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize, // текущий индекс
}

impl Lexer {
    // Создает новый лексер из входной строки
    pub fn new(src: &str) -> Self {
        Self {
            input: src.chars().collect(),
            pos: 0,
        }
    }

    //
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        }
    }

    fn skip_spaces(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        use Token::*;

        self.skip_spaces();

        let ch = match self.advance() {
            Some(c) => c,
            None => return EOF,
        };

        match ch {
            '\n' => Newline,

            // цифра — начинаем читать число
            '0'..='9' => {
                // здесь мы уже прочитали первую цифру `ch`
                self.lex_number(ch)
            }

            // буква или '_' — идентификатор или ключевое слово
            'a'..='z' | 'A'..='Z' | '_' => self.lex_ident_or_keyword(ch),

            '"' => self.lex_string(),

            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => Slash,
            '%' => Percent,

            '{' => LCurly,
            '}' => RCurly,
            '(' => LParen,
            ')' => RParen,
            '[' => LBracket,
            ']' => RBracket,
            ':' => Colon,
            ',' => Comma,

            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    EqEq
                } else {
                    Eq
                }
            }

            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    NotEq
                } else {
                    panic!("Unexpected '!' without '='");
                }
            }

            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    LtEq
                } else {
                    Lt
                }
            }

            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    GtEq
                } else {
                    Gt
                }
            }

            // TODO: добавить поддержку комментариев и вцелом обработку ошибок
            other => panic!("Unexpected character: {:?}", other),
        }
    }

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

        match s.as_str() {
            "var" => Token::Kwvar,
            "mut" => Token::KwMut,
            "def" => Token::KwDef,
            "return" => Token::KwReturn,
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "for" => Token::KwFor,
            "in" => Token::KwIn,
            "True" => Token::KwTrue,
            "False" => Token::KwFalse,
            _ => Token::Ident(s),
        }
    }

    fn lex_string(&mut self) -> Token {
        let mut s = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => break,
                '\n' => panic!("String literal not closed before newline"),
                _ => s.push(ch),
            }
        }

        Token::StrLiteral(s)
    }
}

// TODO: доделать поодержку автоназначения типов, комментариев и обработку ошибок
