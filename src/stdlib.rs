// stdlib.rs
//
// Встроенные функции языка Rusthon.
// Здесь реализован "мини-стандартный" набор:
//   - print(...)
//   - len(x)
//   - sum(xs)
//   - range(n) / range(a, b)
//   - str(x)

use crate::interpreter::Value;

pub fn call_builtin(name: &str, args: &Vec<Value>) -> Option<Value> {
    match name {
        // ===== print =====
        //
        // print(1, "hi", true, [1, 2])
        // выводит всё через пробел и возвращает ()
        "print" => {
            let mut first = true;

            for v in args {
                if !first {
                    print!(" ");
                }
                first = false;

                match v {
                    Value::Int(n) => print!("{n}"),
                    Value::Bool(b) => print!("{b}"),
                    Value::Str(s) => print!("{s}"),
                    Value::List(items) => {
                        print!("[");
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            match item {
                                Value::Int(n) => print!("{n}"),
                                Value::Bool(b) => print!("{b}"),
                                Value::Str(s) => print!("{s}"),
                                Value::List(_) => print!("<nested list>"),
                                Value::Unit => print!("()"),
                            }
                        }
                        print!("]");
                    }
                    Value::Unit => print!("()"),
                }
            }
            println!();

            Some(Value::Unit)
        }

        // ===== len =====
        //
        // len("hello") -> 5
        // len([1, 2, 3]) -> 3
        "len" => {
            if args.len() != 1 {
                panic!("len(x) expects exactly 1 argument, got {}", args.len());
            }

            let v = &args[0];
            let n = match v {
                Value::Str(s) => s.len() as i64,
                Value::List(xs) => xs.len() as i64,
                _ => panic!("len(x) only supports string and list, got {:?}", v),
            };

            Some(Value::Int(n))
        }

        // ===== sum =====
        //
        // sum([1, 2, 3]) -> 6
        "sum" => {
            if args.len() != 1 {
                panic!("sum(xs) expects exactly 1 argument, got {}", args.len());
            }

            let v = &args[0];
            let xs = match v {
                Value::List(xs) => xs,
                _ => panic!("sum(xs) expects list of ints, got {:?}", v),
            };

            let mut acc: i64 = 0;
            for item in xs {
                match item {
                    Value::Int(n) => acc += n,
                    _ => panic!("sum(xs) expects all elements to be ints, got {:?}", item),
                }
            }

            Some(Value::Int(acc))
        }

        // ===== range =====
        //
        // range(5)     -> [0, 1, 2, 3, 4]
        // range(2, 5)  -> [2, 3, 4]
        "range" => {
            let list: Vec<Value> = match args.as_slice() {
                [Value::Int(n)] => {
                    if *n < 0 {
                        panic!("range(n) with n < 0 is not supported");
                    }
                    (0..*n).map(|i| Value::Int(i)).collect()
                }
                [Value::Int(a), Value::Int(b)] => {
                    if *a > *b {
                        panic!("range(a, b) expects a <= b");
                    }
                    (*a..*b).map(|i| Value::Int(i)).collect()
                }
                _ => panic!("range expects 1 or 2 integer arguments, got {:?}", args),
            };

            Some(Value::List(list))
        }

        // ===== str =====
        //
        // str(123)        -> "123"
        // str(true)       -> "true"
        // str([1, 2, 3])  -> "[1, 2, 3]" (грубый формат)
        "str" => {
            if args.len() != 1 {
                panic!("str(x) expects exactly 1 argument, got {}", args.len());
            }

            let v = &args[0];
            let s = match v {
                Value::Int(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Str(s) => s.clone(),
                Value::Unit => "()".to_string(),
                Value::List(xs) => {
                    // простой, но рабочий формат
                    let mut out = String::from("[");
                    for (i, item) in xs.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&match item {
                            Value::Int(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Str(s) => s.clone(),
                            Value::Unit => "()".to_string(),
                            Value::List(_) => "<nested list>".to_string(),
                        });
                    }
                    out.push(']');
                    out
                }
            };

            Some(Value::Str(s))
        }

        // неизвестное имя — не встроенная функция
        _ => None,
    }
}
