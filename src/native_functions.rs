use crate::{Value, ValueType};

use std::collections::HashMap;
use std::time::Instant;


pub fn init_native_functions() -> HashMap<String, Value>{

    let mut native_functions = HashMap::new();


    native_functions.insert("printf".to_string(), Value::native_function("printf", vec!["value"], |values| {

        let value = values.get("value").unwrap();
        
        match value.value_type {
                ValueType::NIL => println!("NIL"),
                ValueType::BOOL => println!("{:?}", value.bool_value.unwrap()),
                ValueType::NUMBER => println!("{:?}", value.number_value.unwrap()),
                ValueType::STRING => println!("{:?}", value.clone().string_value.unwrap()),
                ValueType::NATIVEFUNCTION => println!("{:?}", value.clone().function.unwrap())
        }

        Value::default()
    }));


    native_functions.insert("getTime".to_string(), Value::native_function("getTime", vec![], |values| {
       
        
        Value{value_type: ValueType::NUMBER, number_value:Some(69420.0), ..Value::default()}
        
    }));


    native_functions
}

