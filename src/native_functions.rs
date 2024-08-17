use crate::{Value, ValueType};

use std::collections::HashMap;
use std::time::Instant;


pub fn init_native_functions() -> HashMap<String, Value>{

    let mut native_functions = HashMap::new();


    native_functions.insert("printf".to_string(), Value::native_function("printf", vec!["value"], |values| {

        let value = values.get("value").unwrap();
        
        match value.value_type {
                ValueType::ARRAY => println!("{:?}", value.array.clone().unwrap()),                
                ValueType::NIL => println!("NIL"),
                ValueType::BOOL => println!("{:?}", value.bool_value.unwrap()),
                ValueType::NUMBER => println!("{:?}", value.number_value.unwrap()),
                ValueType::STRING => println!("{:?}", value.clone().string_value.unwrap()),
                ValueType::NATIVEFUNCTION => println!("{:?}", value.clone().function.unwrap()),
                ValueType::THORFUNCTION => println!("{:?}", value.clone().function.unwrap())
        }

        Value::default()
    }));


    native_functions.insert("getTime".to_string(), Value::native_function("getTime", vec![], |values| {
       
        
        Value{value_type: ValueType::NUMBER, number_value:Some(69420.0), ..Value::default()}
        
    }));


    native_functions
}


pub fn init_number_methods() -> HashMap<String, Value>{
    
    let mut methods = HashMap::new();
 
    methods.insert("magic_number".to_string(), Value{
        number_value : Some(89989898.0),
        value_type : ValueType::NUMBER,
        ..Value::default()
    });

    methods
}
