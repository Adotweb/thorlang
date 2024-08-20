use crate::{Value, ValueType, Environment, 
eval, Expression, eval_statement, Function};

use std::collections::HashMap;
use std::time::Instant;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;



pub fn init_native_functions() -> HashMap<String, Value>{

    let mut native_functions = HashMap::new();


    native_functions.insert("printf".to_string(), Value::native_function("printf", vec!["value"], Arc::new(|values| {

        let value = values.get("value").unwrap();

        println!("{}", stringify_value(value.clone()));

        Value::default()
    }), None));


    native_functions.insert("getTime".to_string(), Value::native_function("getTime", vec![], Arc::new(|values| {
       
        
        Value{value_type: ValueType::NUMBER, number_value:Some(69420.0), ..Value::default()}
        
    }), None));


    native_functions
}


pub fn init_number_fields(init : Value) -> HashMap<String, Value>{
    
    let mut s = HashMap::new();

    let init_value = Some(Box::new(init));

    s.insert("magic_number".to_string(), Value{
        number_value : Some(89989898.0),
        value_type : ValueType::NUMBER,
        ..Value::default()
    });

    s.insert("sqrt".to_string(), Value::native_function("sqrt", vec![], Arc::new(|values| {
        
        let self_value = values.get("self").unwrap();
      

        Value::number(self_value.number_value.unwrap().sqrt())

    }), init_value));

    

    s    
}

pub fn init_string_fields(init : Value) -> HashMap<String, Value>{
    let mut fields = HashMap::new();


    fields
}

pub fn init_bool_fields(init : Value) -> HashMap<String, Value>{
    let mut fields = HashMap::new();

    fields
}

//methods like push need to be able to alter the environment so we need to pass it in as an
//argument, also since we need to know what variable (which array) is altered we need to know the
//name (or later the expression) of the variable to be able to read the value in the env
pub fn init_array_fields(arr : Value, enclosing : Rc<RefCell<Environment>>, var_name : String) -> HashMap<String, Value>{
    
    let mut fields = HashMap::new();

    let init_val = Some(Box::new(arr));

    fields.insert("len".to_string(), Value::native_function("len", vec![], Arc::new(|values| {

        let self_value = values.get("self").unwrap();


        Value::number(self_value.array.clone().len() as f64)

    }), init_val.clone()));


    fields.insert("push".to_string(), Value::native_function("push", vec!["thing"], Arc::new(move |values|{


        let self_value = values.get("self").unwrap();
        let thing = values.get("thing").unwrap();

        let mut newarr = self_value.array.clone();

        newarr.push(thing.clone());
        
        if var_name != "" {
            enclosing.borrow_mut().set(var_name.clone(), Value::array(newarr.clone()));
        }

        Value::array(newarr)
    }), init_val.clone()));

    fields.insert("map".to_string(), Value::native_function("map", vec!["func"], Arc::new(|values| {

        let self_value = values.get("self").unwrap();
        let arr = self_value.clone().array;


        let proto_func = values.get("func").unwrap().function.clone().unwrap();




        if let Function::NativeFunction { body, needed_arguments, self_value } = proto_func {
            
            let newarr = arr.iter().map(|value|{
                let mut eval_args = HashMap::new();
                if needed_arguments.len() != 1 {panic!("map functions have to have exactly one argument")}
                let var_name = needed_arguments.get(0).unwrap();

                eval_args.insert(var_name.to_string(), value.clone());
                
                body(eval_args)

            }).collect();

             

            return Value::array(newarr)
        }

        if let Function::ThorFunction { body, needed_arguments, closure } = proto_func {

            let newarr : Vec<Value> = arr.iter().map(|value| {
                if needed_arguments.len() != 1{panic!("map functions have to have exactly one argument")}
                
                let var_name = needed_arguments.get(0).unwrap();
               
                 
                let function_env = Environment::new(Some(closure.clone()));

                function_env.borrow_mut().values.borrow_mut().insert(var_name.to_string(), value.clone());

                eval_statement(body.clone(), function_env)

            }).collect();


            return Value::array(newarr)
        }

        //since we have to call a function to each value, we need to have the eval function 
        //and we have to be able to make a Func call expression
        
        

        self_value.clone()

    }), init_val));
    

    fields
}

pub fn hash_value(val : Value) -> String{
    return match val.value_type {
        ValueType::BOOL => val.bool_value.unwrap().to_string(),
        ValueType::STRING => val.string_value.unwrap(),
        ValueType::NUMBER => val.number_value.unwrap().to_string(),
        _ => panic!("cannot hash {:?}", val.value_type)
    }

}

pub fn stringify_value(val : Value) -> String{

    let mut ret_val = "".to_string();

    match val.value_type {
        ValueType::ARRAY => {
            
            let arr = val.array;
            
            ret_val += "[";

            for i in 0..arr.len() {

                if i > 0{
                    ret_val += ", "
                } 
                
                ret_val += &stringify_value(arr.get(i).unwrap().clone())
            }

            ret_val += "]"

        },
        ValueType::BOOL => {
            ret_val = val.bool_value.unwrap().to_string();
        },
        ValueType::STRING => {
            ret_val = (r#"""#.to_string()+ &val.string_value.unwrap() + r#"""#);
        },
        ValueType::NIL => {
            ret_val = "NIL".to_string();
        },
        ValueType::NUMBER => {
            ret_val = val.number_value.unwrap().to_string();
        },
        ValueType::OBJECT => {
            let obj = val.fields;

            ret_val += "{ ";

            for (key, value) in obj.iter(){
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
