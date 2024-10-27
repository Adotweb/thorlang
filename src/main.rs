mod error;
mod eval;
mod lexer;
mod native_functions;
mod parser;

use std::panic;
use error::{ThorLangError, handle_error};
use eval::{eval_statement, Environment, Function, Value, ValueType};
use lexer::{lexer, Token, TokenType};
use native_functions::{
    hash_value, init_array_fields, init_bool_fields, init_native_functions, init_number_fields,
    init_string_fields, stringify_value,
};
use parser::{parse, Expression, Statement};
use std::collections::HashMap;

use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut current_dir =
        env::current_dir().expect("something went wrong reading the current directory");


    //this will be the entry point of the cli
    match args.get(1).unwrap().as_str() {
        "run" => {
            //just run files ending with .thor

            if let Some(filename) = args.get(2) {
                let mut filename = filename.clone().to_owned();

                //tries to append the .thor filetype to allow for only putting in the filename
                if filename.contains(".thor") {
                } else {
                    filename += ".thor";
                }

                current_dir.push(filename);

                let file_text = fs::read_to_string(current_dir).expect("no such file found");

                interpret_code(file_text);
            }
        }
        _ => {
            //just run files ending with .thor

            if let Some(filename) = args.get(2) {
                let mut filename = filename.clone().to_owned();

                //tries to append the .thor filetype to allow for only putting in the filename
                if filename.contains(".thor") {
                } else {
                    filename += ".thor";
                }

                current_dir.push(filename);

                let file_text = fs::read_to_string(current_dir).expect("no such file found");

                interpret_code(file_text);
            }
        }

    }
}

//allows functions files to return values that can be used by other files
//basically modules
pub fn interpret_code(text: String) -> Value {
    let tokens = lexer(text.clone());
    //println!("{:#?}", tokens.clone());
    panic::set_hook(Box::new(|x|{

            //println!("{x}");
    }));

    //custom error handling can be defined in these match arms 
    let ast = match parse(tokens.clone()){
        Ok(stmts) => stmts,
        Err(err) => {
            handle_error(text, tokens.clone(), err);
            panic!();
        }
    };
  
    println!("{:#?}", ast);
    

    //the global env instantiation (global values and functions)
    let natives: HashMap<String, Value> = init_native_functions();
    let global_env = Rc::new(RefCell::new(Environment {
        values: natives.into(),
        enclosing: None,
    }));

    //same with overloadings
    let overloadings = &mut HashMap::new();


    return match eval_statement(ast, global_env, overloadings) {
        Ok(val) => val,
        Err(err) => {
            handle_error(text, tokens, err);
            panic!();
        }
    }
}
