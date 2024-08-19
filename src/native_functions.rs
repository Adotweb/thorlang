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
    }, None));


    native_functions.insert("getTime".to_string(), Value::native_function("getTime", vec![], |values| {
       
        
        Value{value_type: ValueType::NUMBER, number_value:Some(69420.0), ..Value::default()}
        
    }, None));


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

    s.insert("sqrt".to_string(), Value::native_function("sqrt", vec![], |values| {
        
        let self_value = values.get("self").unwrap();
      

        Value::number(self_value.number_value.unwrap().sqrt())

    }, init_value));

    

    s    
}


pub fn init_array_fields(arr : Value) -> HashMap<String, Value>{
    
    let mut fields = HashMap::new();

    let init_val = Some(Box::new(arr));

    fields.insert("len".to_string(), Value::native_function("len", vec![], |values| {

        let self_value = values.get("self").unwrap();


        Value::number(self_value.array.clone().unwrap().len() as f64)

    }, init_val.clone()));


    fields.insert("push".to_string(), Value::native_function("push", vec!["thing"], |values|{


        let self_value = values.get("self").unwrap();
        let thing = values.get("thing").unwrap();

        let mut newarr = self_value.array.clone().unwrap();

        newarr.push(thing.clone());

        Value::array(newarr)
    }, init_val));

    fields
}
