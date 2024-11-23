mod lexer;
mod parser;
mod error;
mod native_functions;
mod eval;

pub use lexer::*;
pub use parser::*;
pub use error::*;
pub use native_functions::*;
pub use eval::*;

use type_lib::*;

use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::panic;


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
