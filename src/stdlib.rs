use crate::interpreter::Value;

/// Встроенные функции языка.
/// Если имя совпадает с одной из функций ниже — возвращаем Some(Value),
/// иначе None (значит, нужно искать пользовательскую функцию).
pub fn call_builtin(name: &str, args: &Vec<Value>) -> Option<Value> {
    match name {
        // --------------------------
        // print(x, y, z, ...)
        // Печатает значения через пробел и возвращает Unit.
        // --------------------------
        "print" => {
            let mut first = true;

            for v in args {
                if !first {
                    print!(" ");
                }
                first = false;
                print_value(v);
            }
            println!();
            Some(Value::Unit)
        }

        // --------------------------
        // len(x)
        // Строка -> её длина (в символах)
        // Список -> количество элементов
        // --------------------------
        "len" => {
            if args.len() != 1 {
                panic!("len(x) expects exactly 1 argument");
            }
            let v = &args[0];
            let n = match v {
                Value::Str(s) => s.chars().count() as i64,
                Value::List(items) => items.len() as i64,
                other => panic!("len(...) is not defined for value {:?}", other),
            };
            Some(Value::Int(n))
        }

        // --------------------------
        // range(n)
        // Создаёт список [0, 1, ..., n-1]
        // --------------------------
        "range" => {
            if args.len() != 1 {
                panic!("range(n) expects exactly 1 argument");
            }
            let n = match args[0] {
                Value::Int(n) => n,
                ref other => panic!("range(n): n must be int, got {:?}", other),
            };
            if n < 0 {
                panic!("range(n): n must be >= 0");
            }
            let mut items = Vec::new();
            for i in 0..n {
                items.push(Value::Int(i));
            }
            Some(Value::List(items))
        }

        // --------------------------
        // push(list, value)
        // Возвращает НОВЫЙ список с добавленным элементом.
        //
        //   xs = push(xs, 10)
        // --------------------------
        "push" => {
            if args.len() != 2 {
                panic!("push(list, value) expects exactly 2 arguments");
            }
            let list = match &args[0] {
                Value::List(items) => items.clone(),
                other => panic!("push(list, value): first arg must be list, got {:?}", other),
            };
            let mut new_list = list;
            new_list.push(args[1].clone());
            Some(Value::List(new_list))
        }

        // --------------------------
        // head(list)
        // Первый элемент списка.
        // --------------------------
        "head" => {
            if args.len() != 1 {
                panic!("head(list) expects exactly 1 argument");
            }
            match &args[0] {
                Value::List(items) => {
                    if items.is_empty() {
                        panic!("head([]): empty list");
                    }
                    Some(items[0].clone())
                }
                other => panic!("head(list): argument must be list, got {:?}", other),
            }
        }

        // --------------------------
        // tail(list)
        // Все элементы списка, кроме первого.
        // --------------------------
        "tail" => {
            if args.len() != 1 {
                panic!("tail(list) expects exactly 1 argument");
            }
            match &args[0] {
                Value::List(items) => {
                    if items.is_empty() {
                        panic!("tail([]): empty list");
                    }
                    let tail_slice = &items[1..];
                    Some(Value::List(tail_slice.to_vec()))
                }
                other => panic!("tail(list): argument must be list, got {:?}", other),
            }
        }

        // --------------------------
        // str(x)
        // Преобразование к строке:
        //   int  -> "123"
        //   bool -> "true"/"false"
        //   str  -> как есть
        //   list -> строка вида "[1, 2, 3]" (упрощённо)
        // --------------------------
        "str" => {
            if args.len() != 1 {
                panic!("str(x) expects exactly 1 argument");
            }
            let s = match &args[0] {
                Value::Int(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Str(s) => s.clone(),
                Value::List(items) => {
                    // Простое представление списка
                    let mut parts = Vec::new();
                    for it in items {
                        parts.push(format!("{:?}", it));
                    }
                    format!("[{}]", parts.join(", "))
                }
                Value::Unit => "()".to_string(),
            };
            Some(Value::Str(s))
        }

        // --------------------------
        // int(x)
        // Преобразование к целому:
        //   int  -> int
        //   bool -> 0/1
        //   str  -> parse::<i64>()
        // --------------------------
        "int" => {
            if args.len() != 1 {
                panic!("int(x) expects exactly 1 argument");
            }
            let n = match &args[0] {
                Value::Int(n) => *n,
                Value::Bool(b) => {
                    if *b {
                        1
                    } else {
                        0
                    }
                }
                Value::Str(s) => s.parse::<i64>().unwrap_or_else(|_| {
                    panic!("int(x): cannot parse string {:?} as integer", s);
                }),
                other => panic!("int(x) is not defined for {:?}", other),
            };
            Some(Value::Int(n))
        }

        // неизвестная функция — пусть ищет пользовательскую
        _ => None,
    }
}

/// Внутренний helper для print: красиво печатает любое Value.
fn print_value(v: &Value) {
    match v {
        Value::Int(n) => print!("{n}"),
        Value::Bool(b) => print!("{b}"),
        Value::Str(s) => print!("{s}"),
        Value::Unit => print!("()"),

        Value::List(items) => {
            print!("[");
            let mut first = true;
            for item in items {
                if !first {
                    print!(", ");
                }
                first = false;
                match item {
                    Value::Int(n) => print!("{n}"),
                    Value::Bool(b) => print!("{b}"),
                    Value::Str(s) => print!("\"{s}\""),
                    Value::Unit => print!("()"),
                    // Вложенные списки/сложные значения пока просто через Debug
                    Value::List(_) => print!("{:?}", item),
                }
            }
            print!("]");
        }
    }
}
