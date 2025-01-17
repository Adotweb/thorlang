use crate::{interpret_code, EnvState, Environment, ThorLangError, Value, lexer, parse, eval_statement};
use libloading::{Library, Symbol};
use type_lib::*;

use std::time::{SystemTime, UNIX_EPOCH};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ::std::io::{self, BufRead};

static FN_MAP: RegisteredFnMap = Mutex::new(None);

//loads a .so library and makes all the lib functions executable by storing the library in cache
//using an Arc
fn load_lib(path: String) -> Result<HashMap<String, Value>, ThorLangError> {
    unsafe {
        //load the lib
        let lib = Library::new(path);

        //check if the lib exists
        match lib {
            Ok(lib) => {
                //move the lib into an arc
                let lib = Arc::new(lib);

                match lib.get::<Symbol<extern "Rust" fn() -> HashMap<String, Value>>>(b"value_map")
                {
                    Ok(map) => {
                        let mut v_map = map();

                        let mut ret_map = HashMap::new();


                        v_map.iter_mut().for_each(|(key, value)|{
                            value.library = Some(lib.clone());

                            ret_map.insert(key.to_string(), value.clone());
                        });


                        Ok(ret_map)
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        Err(ThorLangError::UnknownError)
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
                Err(ThorLangError::UnknownError)
            }
        }
    }
}

pub fn execute_lib_function(
    lib_function: Value,
    arguments: HashMap<String, Value>,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
) -> Result<Value, ThorLangError> {

    let library = lib_function.library;


    //execution of a lib function works by invokint the name with the lib.get method
    if let ValueType::Function(type_lib::Function::LibFunction {
        name,
        needed_arguments,
        self_value,
        mutating,
    }) = lib_function.value
    {
        let name_string = format!("{}", name);
        let bytes = name_string.as_bytes();


        unsafe {
            let lib = library.unwrap().clone();

            if mutating {
                //function inside of the lib gets called and then executed with the arguments it needs
                let function = match lib.get::<Symbol<
                    extern "Rust" fn(
                        HashMap<String, Value>,
                        Arc<Mutex<Environment>>,
                        &mut Overloadings,
                    ) -> Value,
                >>(bytes)
                {
                    Ok(function) => Ok(function),
                    Err(e) => {
                        println!("{:?}", e);
                        Err(ThorLangError::UnknownError)
                    }
                }?;


                let mut ret_val = function(arguments, enclosing, overloadings);

                ret_val.library = Some(lib.clone());

                return Ok(ret_val);
            }

            //function inside of the lib gets called and then executed with the arguments it needs
            let function =
                match lib.get::<Symbol<extern "Rust" fn(HashMap<String, Value>) -> Value>>(bytes) {
                    Ok(function) => Ok(function),
                    Err(e) => {
                        println!("{:?}", e);
                        Err(ThorLangError::UnknownError)
                    }
                }?;


            let mut ret_val = function(arguments);

            ret_val.library = Some(lib.clone());

            return Ok(ret_val);
        }
    }

    Err(ThorLangError::UnknownError)
}

pub fn get_registered_function(name: String) -> Result<FnType, ThorLangError> {
    let mut map = FN_MAP.lock().unwrap();

    if let Some(ref mut map) = *map {
        return Ok(map.get(&name).unwrap().clone());
    }

    Err(ThorLangError::UnknownError)
}

pub fn register_string_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    //primitive methods only need a name some inputs and a reference to themselves
    //the "register function body" method can be used to register the actual calculations but not
    //to the struct but to the FN_MAP static variable instead, making the functions easily lazy
    //loadable in the future
    Value::primitive_method("length", vec![], self_value.clone())
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value, _, _, _| {
                if let ValueType::String(self_string) = &self_value.unwrap().value {
                    return Ok(Value::number(self_string.len() as f64));
                }
                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    Value::primitive_method("parse_number", vec![], self_value)
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value, _, _, _| {
                if let ValueType::String(self_value) = &self_value.unwrap().value {
                    return match self_value.parse::<f64>() {
                        Ok(num) => Ok(Value::number(num)),
                        Err(_) => Err((ThorLangError::UnknownError)),
                    };
                }
                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    map
}

pub fn register_bool_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map
}

pub fn register_object_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map
}

pub fn register_function_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    map
}

//creates only named functions and inserts the function body itself inside of the FN_MAP
pub fn register_number_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    Value::named_function("ceil", vec![], Some(Box::new(self_value.clone())), None, None)
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value: Option<Value>, _, _, _| {
                if let ValueType::Number(num) = &self_value.unwrap().value {
                    return Ok(Value::number(num.ceil()));
                }
                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    Value::named_function("floor", vec![], Some(Box::new(self_value.clone())), None, None)
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value: Option<Value>, _, _, _| {
                if let ValueType::Number(num) = &self_value.unwrap().value {
                    return Ok(Value::number(num.floor()));
                }
                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);



    Value::named_function("sqrt", vec![], Some(Box::new(self_value)), None, None)
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value: Option<Value>, _, _, _| {
                if let ValueType::Number(num) = &self_value.unwrap().value {
                    return Ok(Value::number(num.sqrt()));
                }
                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    map
}

pub fn register_native_functions(env: EnvState) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    Value::env_function("eval", vec!["code"], env.clone())
        .register_function_body(
            &FN_MAP, 
            Arc::new(|args, _, enclosing, _, env_state|{
                let env_state = env_state.unwrap();
                

                let eval_code = args.get("code").unwrap();

                if let ValueType::String(eval_code) = &eval_code.value{
            

                    let enclosing = enclosing.unwrap().clone();

                    let mut overloadings = enclosing.lock().unwrap().get_overloadings();

                    let lexed = lexer(eval_code.to_string());

                    let ast = parse(lexed)?;

                    let result = eval_statement(ast, enclosing, &mut overloadings)?;


                    return Ok(result)
                } 

                 

                Ok(Value::nil())
            })
        )
        .insert_to(&mut map);

       Value::simple_function("get_now", vec![])
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, _, _, _, _| {
                let now = UNIX_EPOCH.elapsed().unwrap().as_millis() as f64;
                Ok(Value::number(now))
            }),
        )
        .insert_to(&mut map);

    Value::env_function("import", vec!["namespace"], env.clone())
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, env_state| {
                let path = env_state.clone().unwrap().path;

                let namespace = args
                    .get("namespace")
                    .unwrap_or_else(|| panic!("namespace required"));

                //import only works for string

                if let ValueType::String(string) = &namespace.value {
                    let mut module_path = path;

                    module_path.push(string);

                    let module_text = fs::read_to_string(module_path).unwrap_or_else(|_| {
                        panic!("module {string} does not exist in the current directory")
                    });

                    return Ok(interpret_code(module_text, env_state.unwrap().clone()));
                } else {
                    panic!("can only import from strings")
                }
            }),
        )
        .insert_to(&mut map);

    Value::env_function("import_lib", vec!["namespace"], env)
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, env_state| {
                let namespace = args.get("namespace").unwrap();

                if let ValueType::String(path) = &namespace.value {
                    let path_string =
                        env_state.unwrap().path.to_str().unwrap().to_string() + "/" + path;

                    let lib_map = load_lib(path_string);

                    let mut ret = Value::nil();
                    ret.value = ValueType::Object;

                    ret.fields = lib_map?;

                    return Ok(ret);
                }

                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    Value::simple_function("get_input", vec!["message"])
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, _| {
                let message = args.get("message").unwrap();

                //this functions needs nil as an input if there is no message to print
                if let ValueType::Nil = message.value {
                } else {
                    println!("{}", stringify_value(message.clone()));
                }

                let mut input_line = String::new();

                let stdin = io::stdin();

                stdin.lock().read_line(&mut input_line).unwrap();

                let ret_val = Value::string(input_line.replace("\n", ""));

                return Ok(ret_val);
            }),
        )
        .insert_to(&mut map);

    Value::simple_function("type_of", vec!["value"])
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, _| {
                let val = args.get("value").unwrap();

                return Ok(Value::string(
                    match &val.value {
                        ValueType::String(_str) => "string",
                        ValueType::Number(_num) => "number",
                        ValueType::Nil => "nil",
                        ValueType::Object => "object",
                        ValueType::Array(_arr) => "array",
                        ValueType::Function(_func) => "function",
                        ValueType::Bool(_bool) => "bool",
                        ValueType::Error(_err) => "error",
                    }
                    .to_string(),
                ));
            }),
        )
        .insert_to(&mut map);

    Value::simple_function("stringify", vec!["value"])
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, _| {
                let val = args.get("value").unwrap();

                Ok(Value::string(stringify_value(val.clone())))
            }),
        )
        .insert_to(&mut map);

    map
}

pub fn register_array_methods(self_value: Value, var_name: String) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    Value::primitive_method("len", vec![], self_value.clone())
        .register_function_body(
            &FN_MAP,
            Arc::new(|_, self_value, _, _, _| {
                if let ValueType::Array(arr) = &self_value.unwrap().value {
                    return Ok(Value::number(arr.len() as f64));
                }

                Err(ThorLangError::UnknownError)
            }),
        )
        .insert_to(&mut map);

    Value::named_function(
        "push",
        vec!["value"],
        Some(Box::new(self_value)),
        Some(var_name),
        None,
    )
    .register_function_body(
        &FN_MAP,
        Arc::new(|args, self_value: Option<Value>, enclosing, var_name, _| {
            if let ValueType::Array(arr) = &self_value.unwrap().value {
                let value = args.get("value").unwrap();
                let mut new_arr = arr.clone();
                new_arr.push(value.clone());

                let new_arr_value = Value::array(new_arr);

                if let Some(var_name) = var_name {
                    let _ =
                        enclosing
                            .unwrap()
                            .lock()
                            .unwrap()
                            .set(var_name, new_arr_value.clone(), 0);
                }

                return Ok(new_arr_value);
            }

            Err(ThorLangError::UnknownError)
        }),
    )
    .insert_to(&mut map);

    map
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
