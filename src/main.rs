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

use std::env;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;





fn main() {
    let args : Vec<String> = env::args().collect();


    let mut current_dir = env::current_dir().expect("something went wrong reading the current directory"); 
    

    match args.get(1).unwrap().as_str(){
        "run" => {
            if let Some(filename) = args.get(2){
                
                current_dir.push(filename);
                


                let file_text = fs::read_to_string(current_dir)
                    .expect("no such file found");


                interpret_code(file_text);
            }
        },
        _ => panic!("need a first command!")
    }
     


    
    
}


fn interpret_code(text : String){
    let tokens = lexer(text);
    let AST = parse(tokens.clone());
    let natives : HashMap<String, Value> = init_native_functions();
    let global_env = Rc::new(RefCell::new(Environment{
        values : natives.into(),
        enclosing : None
    }));
    eval_statement(AST, global_env); 

} 
