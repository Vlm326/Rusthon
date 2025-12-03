use crate::interpreter::Value;

pub fn call_builtin(name: &str, args: &Vec<Value>) -> Option<Value> {
    match name {
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
                    Value::Unit => print!("()"),
                }
            }
            println!();

            Some(Value::Unit)
        }
        _ => None,
    }
}
