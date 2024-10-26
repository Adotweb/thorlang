use crate::{eval_statement, interpret_code, Environment, Function, Value, ValueType, ThorLangError};

use std::time::{SystemTime, UNIX_EPOCH};

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;

use::std::io::{BufRead, self};


//this is the initializer for all the global variables
pub fn init_native_functions() -> HashMap<String, Value> {
    let mut native_functions = HashMap::new();


    //returns the type of the inputed value
    native_functions.insert(
        "typeOf".to_string(),
        Value::native_function(
            vec!["val"],
            Arc::new(|values|{

                let val = values.get("val").unwrap();

            

                return Ok(Value::string(match &val.value {
                    ValueType::String(_str) => "string",
                    ValueType::Number(_num) => "number",
                    ValueType::Nil => "nil",
                    ValueType::Object => "object",
                    ValueType::Array(_arr) => "array",
                    ValueType::Function(_func) => "function",
                    ValueType::Bool(_bool) => "bool",
                    ValueType::Error(_err) => "error"
                }.to_string()))

            }),
            None
        )
    );

    //start_timer returns this function and it can only be obtained as such
   

    // a time measuring function
    native_functions.insert(
        "get_now".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|_values|{
               
                let now = SystemTime::now();

                return Ok(Value::number(now.duration_since(UNIX_EPOCH).unwrap().as_millis() as f64))

            }),
            None
        )
    );


    //returns true if the input value is an error and false if it is a normal value
    native_functions.insert(
        "isError".to_string(),
        Value::native_function(
            vec!["val"],
            Arc::new(|values| {
                let val = values.get("val").unwrap();

                if let ValueType::Error(_err) = &val.value {
                    return Ok(Value::bool(true))
                } else {
                    return Ok(Value::bool(false))
                }
            }), 
            None
        ),
    );


    //function to make file splitting possible
    //we can return values from programs and reuse them in other files using import
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

                    Ok(interpret_code(module_text))
                } else {
                    panic!("can only import from strings")
                }
            }),
            None,
        ),
    );

    native_functions.insert(
        "get_input".to_string(),
        Value::native_function(
            vec!["message"],
            Arc::new(|values|{
               

                let message = values.get("message").unwrap();

                //this functions needs nil as an input if there is no message to print
                if let ValueType::Nil = message.value{
                         
                }else {
                    println!("{}", stringify_value(message.clone()));
                }
                

                let mut input_line = String::new();          

                let stdin = io::stdin();


                stdin.lock().read_line(&mut input_line).unwrap();


                let ret_val = Value::string(input_line.replace("\n", ""));


                return Ok(ret_val)

            }),
            None
        )
    );


    //printing but using a function instead of a statement
    native_functions.insert(
        "printf".to_string(),
        Value::native_function(
            vec!["value"],
            Arc::new(|values| {
                let value = values.get("value").unwrap();


                if let ValueType::String(str) = &value.value{
                    println!("{str}");
                } else {
                    println!("{}", stringify_value(value.clone()));
                }

                Ok(Value::default())
            }),
            None,
        ),
    );

    //thsi does not work but it would if i wanted to 
    native_functions.insert(
        "getTime".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|_values| Ok(Value::number(69420.0))),
            None,
        ),
    );

    native_functions
}

pub fn init_number_fields(init: Value) -> HashMap<String, Value> {
    let mut s = HashMap::new();

    let init_value = Some(Box::new(init));

   
    //returns the square root of a function
    s.insert(
        "sqrt".to_string(),
        Value::native_function(
            vec![],
            Arc::new(|values| {
                let self_value = values.get("self").unwrap();


                if let ValueType::Number(num) = self_value.value{
                    return Ok(Value::number(num.sqrt()))
                } else {
                    return Err(ThorLangError::EvalError("".to_string()))
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
                    return Ok(Value::number(arr.len() as f64))
                }
                else {
                    panic!("?")
                } 
            }),
            init_val.clone(),
        ),
    );


    //push updates the env tree itself, meaning that we need to get it in the arguments of init_array_fields
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
                        let _ = enclosing
                            .borrow_mut()
                            .set(var_name.clone(), Value::array(newarr.clone()))?;
                    }

                    return Ok(Value::array(newarr))
                } else {
                    return Err(ThorLangError::EvalError("".to_string()))
                }

            }),
            init_val.clone(),
        ),
    );


    fields
}


//helper function to hash values (for object retrieval still in dev)
pub fn hash_value(val: Value) -> String {
    return match val.value {
        ValueType::Bool(b) => b.to_string(),
        ValueType::Number(n) => n.to_string(),
        ValueType::String(s) => s.to_string(), 
        _ => panic!("cannot hash {:?}", val.value),
    };
}


//helper function to pretty print values (especially array and objects, later functions as well)
pub fn stringify_value(val: Value) -> String {
    let mut ret_val = "".to_string();

    match val.value {
        ValueType::Error(err) => {

            if let ThorLangError::ThorLangException { exception, throw_token_index } = err{ 
                ret_val = format!("Error({})", stringify_value(*exception));
            } else  {

                ret_val = format!("{:?}", err);
            }

        },
        ValueType::Array(arr) => {
            ret_val += "[";
        
            //add a comma for every value folowing the first one
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

            //adding "field" : "value" for every field
            //and a comma for every field following the first one
            for i in 0..obj.values().len(){

                if i > 0 {

                    ret_val += ", ";
                }
                let key = obj.keys().nth(i).unwrap();
                let value = obj.values().nth(i).unwrap();
                ret_val += &(key.to_string() + " : " + &stringify_value(value.clone()));

            }

            ret_val += " }"
        }
        _ => {
            ret_val = "Function".to_string();
        }
    }

    ret_val
}
