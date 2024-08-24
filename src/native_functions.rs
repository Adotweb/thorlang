use crate::{eval_statement, interpret_code, Environment, Function, Value, ValueType};

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;

pub fn init_native_functions() -> HashMap<String, Value> {
    let mut native_functions = HashMap::new();

    native_functions.insert(
        "import".to_string(),
        Value::native_function(
            vec!["namespace"],
            Arc::new(|values| {
                let namespace = values
                    .get("namespace")
                    .unwrap_or_else(|| panic!("namespace required"));

                //import only works for string

                if let ValueType::String(string) = &namespace.value {
                    let mut module_path = env::current_dir().unwrap();

                    module_path.push(string);

                    let module_text = fs::read_to_string(module_path).unwrap_or_else(|_| {
                        panic!("module {string} does not exist in the current directory")
                    });

                    interpret_code(module_text)
                } else {
                    panic!("can only import from strings")
                }
            }),
            None,
        ),
    );

    native_functions.insert(
        "printf".to_string(),
        Value::native_function(
            vec!["value"],
            Arc::new(|values| {
                let value = values.get("value").unwrap();

                println!("{:?}", value);

                if let ValueType::String(str) = &value.value{
                    println!("{str}");
                } else {
                    println!("{}", stringify_value(value.clone()));
                }

                Value::default()
            }),
            None,
        ),
    );

    native_functions.insert(
        "getTime".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|_values| Value::number(69420.0)),
            None,
        ),
    );

    native_functions
}

pub fn init_number_fields(init: Value) -> HashMap<String, Value> {
    let mut s = HashMap::new();

    let init_value = Some(Box::new(init));

   

    s.insert(
        "sqrt".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|values| {
                let self_value = values.get("self").unwrap();


                if let ValueType::Number(num) = self_value.value{
                    return Value::number(num.sqrt())
                } else {
                    panic!("no number?")
                }
            }),
            init_value,
        ),
    );

    s
}

pub fn init_string_fields(init: Value) -> HashMap<String, Value> {
    let mut fields = HashMap::new();

    let init_value = Some(Box::new(init));


    fields
}

pub fn init_bool_fields(_init: Value) -> HashMap<String, Value> {
    let fields = HashMap::new();

    fields
}

//methods like push need to be able to alter the environment so we need to pass it in as an
//argument, also since we need to know what variable (which array) is altered we need to know the
//name (or later the expression) of the variable to be able to read the value in the env
pub fn init_array_fields(
    arr: Value,
    enclosing: Rc<RefCell<Environment>>,
    var_name: String,
) -> HashMap<String, Value> {
    let mut fields = HashMap::new();

    let init_val = Some(Box::new(arr));

    fields.insert(
        "len".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|values| {
                let self_value = values.get("self").unwrap();

                if let ValueType::Array(arr) = &self_value.value {
                    return Value::number(arr.len() as f64)
                }
                else {
                    panic!("?")
                } 
            }),
            init_val.clone(),
        ),
    );

    fields.insert(
        "push".to_string(),
        Value::native_function(
            vec!["thing"],
            Arc::new(move |values| {
                let self_value = values.get("self").unwrap();
                let thing = values.get("thing").unwrap();


                if let ValueType::Array(arr) = &self_value.value {
                    let mut newarr = arr.clone();        

                    newarr.push(thing.clone());

                    if var_name != "" {
                        enclosing
                            .borrow_mut()
                            .set(var_name.clone(), Value::array(newarr.clone()));
                    }

                    return Value::array(newarr)
                } else {
                    panic!("")
                }

            }),
            init_val.clone(),
        ),
    );


    fields
}

pub fn hash_value(val: Value) -> String {
    return match val.value {
        ValueType::Bool(b) => b.to_string(),
        ValueType::Number(n) => n.to_string(),
        ValueType::String(s) => s.to_string(), 
        _ => panic!("cannot hash {:?}", val.value),
    };
}

pub fn stringify_value(val: Value) -> String {
    let mut ret_val = "".to_string();

    match val.value {
        ValueType::Array(arr) => {
            ret_val += "[";

            for i in 0..arr.len() {
                if i > 0 {
                    ret_val += ", "
                }

                ret_val += &stringify_value(arr.get(i).unwrap().clone())
            }

            ret_val += "]"
        }, 
        ValueType::Bool(b) => {
            ret_val = b.to_string();
        },
        ValueType::Number(b) => {
            ret_val = b.to_string();
        },
        ValueType::String(b) => {
            ret_val = b.to_string();
        },
        ValueType::Nil => {
            ret_val = "nil".to_string();
        }
        ValueType::Object => {
            let obj = val.fields;

            ret_val += "{ ";

            for (key, value) in obj.iter() {
                ret_val += &(key.to_string() + " : " + &stringify_value(value.clone()));
                ret_val += ", ";
            }

            ret_val += " }"
        }
        _ => {
            ret_val = "Function".to_string();
        }
    }

    ret_val
}
