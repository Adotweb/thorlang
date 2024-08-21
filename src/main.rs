mod lexer;
mod parser;
mod eval;
mod native_functions;
use lexer::{lexer, TokenType, LiteralType, Token};
use parser::{parse, Expression, Statement};
use eval::{eval_statement, Environment, Value, ValueType, eval, Function};
use std::collections::HashMap;
use native_functions::{init_native_functions, init_number_fields, init_array_fields, init_bool_fields, init_string_fields, 
stringify_value, hash_value};


use std::rc::Rc;
use std::cell::RefCell;





fn main() {

    let text = r#"

        fn iter(array, func){

            let i = 0;

            while (i < array.len()){
                func(array[i]);
                i = i + 1; 
            }

        }  
      
        
        fn createObj(arr){
            
            let i = 0;

            let b = 0;

            let ret_obj;

            while(i < arr.len()){
                let field = arr[i];
                

                ret_obj[field[0]] = field[1];

                i = i + 1;
                
            }
           
            return ret_obj;
        }
    
        print createObj([
            ["hello", [0, 1, 2, 3]]
        ]);

            
        "#.to_string();

    let tokens = lexer(text);

    let AST = parse(tokens.clone());

    let natives : HashMap<String, Value> = init_native_functions();

    //global environment
    let global_env = Rc::new(RefCell::new(Environment{
        values: natives.into(),
        enclosing : None
    }));

    eval_statement(AST, global_env);

    //println!("{:#?}", AST);

    
    
}
