mod error;
mod eval;
mod lexer;
mod native_functions;
mod parser;

pub use error::*;
pub use eval::*;
pub use lexer::*;
pub use native_functions::*;
pub use parser::*;

use type_lib::*;

use std::collections::HashMap;
use std::panic;
use std::sync::{Arc, Mutex};

//allows functions files to return values that can be used by other files
//basically modules
pub fn interpret_code(text: String, env: EnvState) -> Value {
    let tokens = lexer(text.clone());
    //println!("{:#?}", tokens.clone());
    panic::set_hook(Box::new(|x| {

        //println!("{x}");
    }));

    //custom error handling can be defined in these match arms
    let ast = match parse(tokens.clone()) {
        Ok(stmts) => stmts,
        Err(err) => {
            handle_error(text, tokens.clone(), err);
            panic!();
        }
    };

    //println!("{:#?}", ast);

    //the global env instantiation (global values and functions)
    let natives: HashMap<String, Value> = register_native_functions(env);
    let global_env = Arc::new(Mutex::new(Environment {
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
    };
}
