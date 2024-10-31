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

use std::path::PathBuf;

//structure to get executable information later (for now it only serves so we can get the current
//execution directory)
#[derive(Clone, Debug)]
pub struct EnvState{
    path : PathBuf,
}

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
                let file_dir = current_dir.clone();
                //remove
                current_dir.pop();


                //when i introduce methods like "get_file" or something i need to have access to
                //the environment of 
                //1. the executable
                //2. the file thats run
                let env = EnvState{
                    path : current_dir
                };
            

                let file_text = fs::read_to_string(file_dir).expect("no such file found");

                interpret_code(file_text, env);
            }
        }
        _ => {
        }

    }
}


//allows functions files to return values that can be used by other files
//basically modules
pub fn interpret_code(text: String, env : EnvState) -> Value {
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
  
    //println!("{:#?}", ast);
    

    //the global env instantiation (global values and functions)
    let natives: HashMap<String, Value> = init_native_functions(env);
    let global_env = Rc::new(RefCell::new(Environment {
        values: natives.into(),
        enclosing: None,
    }));

    //same with overloadings
    let overloadings = &mut HashMap::new();

    //again try to run the given list of statements generated form "parse" and if they fail handle
    //the error
    //
    //we need to return this because the import function and later maybe an "eval" function need to
    //evaluate code from text inside of the runtime
    return match eval_statement(ast, global_env, overloadings) {
        Ok(val) => val,
        Err(err) => {
            handle_error(text, tokens, err);
            panic!();
        }
    }
}
