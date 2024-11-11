use crate::{interpret_code, EnvState, Environment, ThorLangError, Value};
use type_lib::{FnType, RegisteredFnMap, ValueType};

use libloading::{Library, Symbol};

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
                        let v_map = map();

                        let ret_map = v_map
                            .iter()
                            .map(|(key, value)| {
                                //create executable lib functions by puttint a reference to the lib
                                //into them
                                match &value.value {
                                    ValueType::Function(type_lib::Function::LibFunction {
                                        name,
                                        needed_arguments,
                                        library,
                                        self_value,
                                    }) => (
                                        key.to_string(),
                                        Value::lib_function(
                                            name,
                                            needed_arguments.clone(),
                                            Some(Arc::clone(&lib)),
                                            self_value.clone(),
                                        ),
                                    ),
                                    _ => (key.to_string(), (*value).clone()),
                                }
                            })
                            .collect();

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
) -> Result<Value, ThorLangError> {
    //execution of a lib function works by invokint the name with the lib.get method
    if let ValueType::Function(type_lib::Function::LibFunction {
        name,
        needed_arguments,
        library,
        self_value,
    }) = lib_function.value
    {
        let name_string = format!("{}", name);
        let bytes = name_string.as_bytes();

        unsafe {
            let lib = library.unwrap().clone();

            //function inside of the lib gets called and then executed with the arguments it needs
            let function =
                match lib.get::<Symbol<extern "Rust" fn(HashMap<String, Value>) -> Value>>(bytes) {
                    Ok(function) => Ok(function),
                    Err(e) => {
                        println!("{:?}", e);
                        Err(ThorLangError::UnknownError)
                    }
                }?;

            return Ok(function(arguments));
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

pub fn register_string_methods(self_value: Value) -> HashMap<String, Value>{
    let mut map = HashMap::new();


    //primitive methods only need a name some inputs and a reference to themselves
    //the "register function body" method can be used to register the actual calculations but not
    //to the struct but to the FN_MAP static variable instead, making the functions easily lazy
    //loadable in the future
    Value::primitive_method("length", vec![], self_value.clone())
        .register_function_body(
            &FN_MAP, 
            Arc::new(|_, self_value, _, _, _|{
                if let ValueType::String(self_string) = &self_value.unwrap().value{
                    return Ok(Value::number(self_string.len() as f64))
                }
                Err(ThorLangError::UnknownError)
            })
        ).insert_to(&mut map);

    Value::primitive_method("parse_number", vec![], self_value)
        .register_function_body(
            &FN_MAP, 
            Arc::new(|_, self_value, _, _, _|{

                if let ValueType::String(self_value) = &self_value.unwrap().value{

                    return match self_value.parse::<f64>(){
                        Ok(num) => Ok(Value::number(num)), 
                        Err(_) => Err((ThorLangError::UnknownError))
                    }

                }
                Err(ThorLangError::UnknownError)
            })
        ).insert_to(&mut map);

    map
}

pub fn register_bool_methods(self_value: Value) -> HashMap<String, Value>{
    let mut map = HashMap::new();

    map
}

pub fn register_object_methods(self_value: Value) -> HashMap<String, Value>{
    let mut map = HashMap::new();

    map
}

pub fn register_function_methods(self_value: Value) -> HashMap<String, Value>{
    let mut map = HashMap::new();

    map
}


//creates only named functions and inserts the function body itself inside of the FN_MAP
pub fn register_number_methods(self_value: Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    Value::named_function("sqrt", vec![], Some(Box::new(self_value)), None, None, None)
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

    Value::simple_function("some_func", vec![])
        .register_function_body(
            &FN_MAP,
            Arc::new(|args, _, _, _, _| {
                println!("hello from some_func");

                Ok(Value::nil())
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
                    let path_string = env_state.unwrap().path.to_str().unwrap().to_string() + "/" + path;

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
            Arc::new(|args, _, _, _, _|{
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

            })
        ).insert_to(&mut map);

    Value::simple_function("type_of", vec!["value"])
        .register_function_body(
            &FN_MAP, 
            Arc::new(|args, _, _, _, _|{
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
            })
        ).insert_to(&mut map);
    
    Value::simple_function("stringify", vec!["value"])
        .register_function_body(
            &FN_MAP, 
            Arc::new(|args, _, _, _, _|{
                let val = args.get("value").unwrap();


                Ok(Value::string(stringify_value(val.clone())))
            })
        ).insert_to(&mut map);




    map
}

pub fn register_array_methods(
    self_value: Value,
    enclosing: Rc<RefCell<Environment>>,
    var_name: String,
) -> HashMap<String, Value> {
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
        Some(enclosing),
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
                    let _ = enclosing
                        .unwrap()
                        .borrow_mut()
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

//helper function to pretty print values (especially array and objects, later functions as well)
pub fn stringify_value(val: Value) -> String {
    let mut ret_val = "".to_string();

    match val.value {
        ValueType::Error(err) => {
            if let ThorLangError::ThorLangException {
                exception,
                throw_token_index : _,
            } = err
            {
                ret_val = format!("Error({})", stringify_value(*exception));
            } else {
                ret_val = format!("{:?}", err);
            }
        }
        ValueType::Array(arr) => {
            ret_val += "[";

            //add a comma for every value folowing the first one
            for i in 0..arr.len() {
                if i > 0 {
                    ret_val += ", "
                }
                //move through the array recursively
                ret_val += &stringify_value(arr.get(i).unwrap().clone())
            }

            ret_val += "]"
        }
        ValueType::Bool(b) => {
            ret_val = b.to_string();
        }
        ValueType::Number(b) => {
            ret_val = b.to_string();
        }
        ValueType::String(b) => {
            ret_val = b.to_string();
        }
        ValueType::Nil => {
            ret_val = "nil".to_string();
        }
        ValueType::Object => {
            let obj = val.fields;

            ret_val += "{ ";

            //adding "field" : "value" for every field
            //and a comma for every field following the first one
            for i in 0..obj.values().len() {
                if i > 0 {
                    ret_val += ", ";
                }
                let key = obj.keys().nth(i).unwrap();
                let value = obj.values().nth(i).unwrap();

                //again move through the object recursively
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
