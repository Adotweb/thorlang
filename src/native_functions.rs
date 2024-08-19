use crate::{Value, ValueType, Environment, 
eval, Expression};

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


        Value::number(self_value.array.clone().unwrap().len() as f64)

    }), init_val.clone()));


    fields.insert("push".to_string(), Value::native_function("push", vec!["thing"], Arc::new(move |values|{


        let self_value = values.get("self").unwrap();
        let thing = values.get("thing").unwrap();

        let mut newarr = self_value.array.clone().unwrap();

        newarr.push(thing.clone());
        
        if var_name != "" {
            enclosing.borrow_mut().set(var_name.clone(), Value::array(newarr.clone()));
        }

        Value::array(newarr)
    }), init_val.clone()));

    fields.insert("map".to_string(), Value::native_function("map", vec!["func"], Arc::new(|values| {

        let self_value = values.get("self").unwrap();

        let func = values.get("func").unwrap();

        //since we have to call a function to each value, we need to have the eval function 
        //and we have to be able to make a Func call expression
        
        

        self_value.clone()

    }), init_val));
    

    fields
}

pub fn stringify_value(val : Value) -> String{

    let mut ret_val = "".to_string();

    match val.value_type {
        ValueType::ARRAY => {
            
            let arr = val.array.unwrap();
            
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
        _ => {
            ret_val = "Function".to_string();
        }
    }

    ret_val
}
