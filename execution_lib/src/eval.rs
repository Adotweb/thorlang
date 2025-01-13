use crate::{
    execute_lib_function, get_registered_function, hash_value, register_array_methods,
    register_bool_methods, register_number_methods, register_string_methods,
};

use std::sync::{Arc, Mutex};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use type_lib::*;

//here the magic happpens: every list of statements mutates the env tree, and as soon as a branch
//of the env tree is not needed it automatically disappears, meaning that we can only
// - mutate variables that exist
// - add variables to the currently used branch of the env tree
//
//this ensures that as soon as a branch is exited (i.e a block is done executing) we will have the
//old values back.
pub fn eval_statement(
    stmts: Vec<Statement>,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
) -> Result<Value, ThorLangError> {
    //evaluating statement by statement
    for stmt in stmts {
        match stmt {
            //not implemented quite yet
            Statement::Throw {
                exception,
                throw_token_index,
            } => {
                let exception = eval(&exception, enclosing, overloadings)?;

                return Err(ThorLangError::ThorLangException {
                    exception: Box::new(exception),
                    throw_token_index,
                });
            }

            //this works basically like a function, except that the call operation takes place in
            //the corresponding eval function
            Statement::Overload {
                operator,
                operands,
                operation,
                line: _,
            } => {
                let arity = operands.len();

                //we dont want to mess with already defined overloaded operators, so any overlaoded
                //operator will only be able to access before initialized (overloaded) operators,
                //this means that similar to function environments we have to pass a operator env
                let defined_overloadings = overloadings.clone();

                let operator_info = OperationInfo {
                    operands,
                    operation,
                    overloadings: defined_overloadings,
                };

                //if there is a list of overloadings for the given operator already we just push
                //this overloading to it. Else we create such a list for use in the future
                if let Some(operationlist) = overloadings.get_mut(&(operator.clone(), arity)) {
                    operationlist.insert(0, operator_info);
                } else {
                    overloadings.insert((operator, arity), vec![operator_info]);
                }
            }
            Statement::Return {
                expression,
                line: _,
            } => {
                let mut ret_value = eval(&expression, enclosing.clone(), overloadings)?;

                //we want the ret_value to bubble up through the statements above, so we need to
                //mark it as a return value
                ret_value.return_true = true;

                return Ok(ret_value);
            }

            Statement::Function {
                name,
                body,
                arguments,
                line: _,
            } => {
                let closure = Environment::new(Some(enclosing.clone()));

                let function = Value::thor_function(arguments, *body, closure.clone());

                //insert the function with its name into the environment
                enclosing
                    .lock()
                    .unwrap()
                    .values
                    .borrow_mut()
                    .insert(name.clone(), function.clone());

                //insert the function with its name into the closure to allow for recursion
                closure
                    .lock()
                    .unwrap()
                    .values
                    .borrow_mut()
                    .insert(name, function);
            }
            //a block just opens a new env tree branch
            Statement::Block {
                statements,
                line: _,
            } => {
                let local_scope = Environment::new(Some(enclosing.clone()));
                eval_statement(statements, local_scope.clone(), overloadings)?;
            }

            //if statements are one to one in the host language
            Statement::If {
                condition,
                then_branch,
                else_branch,
                line: _,
            } => {
                if let ValueType::Bool(bool) =
                    eval(&condition, enclosing.clone(), overloadings)?.value
                {
                    if bool {
                        let return_val =
                            eval_statement(*then_branch, enclosing.clone(), overloadings)?;

                        //when we encounter a value with return_true = true, we need to bubble the
                        //value up
                        if return_val.return_true {
                            return Ok(return_val);
                        }
                    } else {
                        if let Some(ref _else_block) = else_branch {
                            let return_val = eval_statement(
                                *else_branch.unwrap(),
                                enclosing.clone(),
                                overloadings,
                            )?;

                            //same again
                            if return_val.return_true {
                                return Ok(return_val);
                            }
                        }
                    }
                }
            }

            //while statements are as well, we ned a "return_true" value however to detect when we
            //need to return a value
            Statement::While {
                condition,
                block,
                line: _,
            } => {
                let mut condition_value =
                    eval(&condition.clone(), enclosing.clone(), overloadings)?;
                while let ValueType::Bool(bool) = condition_value.value {
                    if bool {
                        let return_val =
                            eval_statement(*block.clone(), enclosing.clone(), overloadings)?;
                        condition_value =
                            eval(&condition.clone(), enclosing.clone(), overloadings)?;

                        //if return_true is true that means that the above eval function has hit a
                        //return statement and we need to return the value here
                        if return_val.return_true {
                            return Ok(return_val);
                        }
                    } else {
                        condition_value.value = ValueType::Nil
                    }
                }
            }
            Statement::For {
                iterator,
                iteration_variable,
                block,
            } => {
                let iter = eval(&iterator, enclosing.clone(), overloadings)?;

                if let ValueType::Array(arr) = iter.value {
                    for val in arr.iter() {
                        let intermediate_environment = Environment::new(Some(enclosing.clone()));

                        let var_name = iteration_variable.get_content().unwrap();

                        intermediate_environment
                            .lock()
                            .unwrap()
                            .values
                            .borrow_mut()
                            .insert(var_name.clone(), val.clone());

                        let ret_val =
                            eval_statement(block.clone(), intermediate_environment, overloadings)?;

                        if ret_val.return_true {
                            return Ok(ret_val.clone());
                        }
                    }
                }
            }

            //print is built in, but can also be used with the printfunction
            Statement::Print {
                expression,
                line: _,
            } => {
                let result = eval(&expression, enclosing.clone(), overloadings)?;

                //println!("debug mode {:#?}", result);
                if let ValueType::String(ref str) = result.value {
                    println!("{str}");
                } else {
                    println!("{}", stringify_value(result));
                }
            }

            Statement::Do {
                expression,
                line: _,
            } => {
                //runs expressions
                eval(&expression, enclosing.clone(), overloadings)?;
            }

            //variable declaration only ever mutates the current branch of the env tree ensuring,
            //in this case "enclosing"
            Statement::Variable {
                name,
                expression,
                line: _,
            } => {
                let val = eval(&expression, enclosing.clone(), overloadings)?;

                enclosing
                    .lock()
                    .unwrap()
                    .values
                    .borrow_mut()
                    .insert(name, val);
            }
        }
    }

    Ok(Value::default())
}

//helper function to check whether or not a operation works for the inputs provided
fn eval_overloaded(
    operation_list: Vec<OperationInfo>,
    arguments: Vec<Value>,
    enclosing: Arc<Mutex<Environment>>,
    operator_token_index: usize,
) -> Result<Value, ThorLangError> {
    let op_env = enclosing.lock().unwrap().clone();

    //loops over every operation associated with the given operator sign
    //when they throw we move on to the next one
    for mut op in operation_list {
        let operands = op.operands;
        let operation = op.operation;
        let overloadings = &mut op.overloadings;

        if operands.len() != arguments.len() {
            return ThorLangError::operation_arity_error(
                operator_token_index,
                operands.len(),
                arguments.len(),
            );
        }

        for i in 0..operands.len() {
            op_env
                .values
                .borrow_mut()
                .insert(operands[i].clone(), arguments[i].clone());
        }

        let tried_eval = eval_statement(
            operation,
            Arc::new(Mutex::new(op_env.clone())),
            overloadings,
        );

        if let Ok(result) = tried_eval {
            return Ok(result);
        }
    }

    //this error will only show up when we use a undefined special character

    ThorLangError::eval_error(operator_token_index)
}

//order of precedence is as follows
// eval_statement -> eval -> eval_binary -> eval_unary -> eval_literal
fn eval_unary(
    operator: TokenType,
    right: &Expression,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
    operator_token_index: usize,
) -> Result<Value, ThorLangError> {
    let r = eval(right, enclosing.clone(), overloadings)?;

    //first we try to eval the overloadings if there are any

    //else we just return the normally evaluated value
    match operator {
        //only negate logically when is bool
        TokenType::BANG => {
            if let ValueType::Bool(bool) = r.value {
                //just one to one in host language
                return Ok(Value::bool(!bool));
            } else {
                return ThorLangError::eval_error(operator_token_index);
            }
        }
        //only negate arithmetically when is number
        TokenType::MINUS => {
            if let ValueType::Number(num) = r.value {
                //also jsut one to one
                return Ok(Value::number(-num));
            } else {
                //these errors will be overworked in the future

                return ThorLangError::eval_error(operator_token_index);
            }
        }
        _ => (),
    }

    if let Some(operation_info) = overloadings.get(&(operator.clone(), 1)) {
        if let Ok(result) = eval_overloaded(
            operation_info.to_vec(),
            vec![r.clone()],
            enclosing.clone(),
            operator_token_index,
        ) {
            return Ok(result);
        } else {
            return ThorLangError::eval_error(operator_token_index);
        }
    } else {
        return ThorLangError::eval_error(operator_token_index);
    }
}

//evaluates the "atoms" these can not be further reduced and bubble up to form more complex data
//(not types but composed values like 1 + 2)
fn eval_literal(literal: TokenType, literal_token_index: usize) -> Result<Value, ThorLangError> {
    //turn literaltype into value wrapped in value_type
    match literal {
        TokenType::NIL => return Ok(Value::nil()),
        TokenType::TRUE => {
            let mut ret_val = Value::bool(true);
            ret_val.fields = register_bool_methods(ret_val.clone());
            return Ok(ret_val);
        }
        TokenType::FALSE => {
            let mut ret_val = Value::bool(false);
            ret_val.fields = register_bool_methods(ret_val.clone());
            return Ok(ret_val);
        }
        TokenType::NUMBER(value) => {
            let mut ret_val = Value::number(value.parse().unwrap());
            ret_val.fields = register_bool_methods(ret_val.clone());
            return Ok(ret_val);
        }
        TokenType::STRING(value) => {
            let mut ret_val = Value::string(value);
            ret_val.fields = register_bool_methods(ret_val.clone());
            return Ok(ret_val);
        }
        _ => ThorLangError::eval_error(literal_token_index),
    }
}

//all the possible binary operation combinations.
fn eval_binary(
    left: &Expression,
    operator: TokenType,
    right: &Expression,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
    operator_token_index: usize,
) -> Result<Value, ThorLangError> {
    let l = eval(left, enclosing.clone(), overloadings)?;
    let r = eval(right, enclosing.clone(), overloadings)?;

    let l_copy = l.clone();
    let r_copy = r.clone();

    match operator {
        // the if let matches if the value of l and r both match the valuetype if there is no match
        // at all the match arm throws
        TokenType::PLUS => {
            if let (ValueType::String(l), ValueType::String(r)) = (l.value.clone(), r.value.clone())
            {
                return Ok(Value::string(l + &r));
            }

            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l + r));
            }
        }
        TokenType::MINUS => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l - r));
            }
        }
        TokenType::STAR => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l * r));
            }
        }
        TokenType::SLASH => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l / r));
            }
        }
        TokenType::LESSEQ => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l <= r));
            }
        }
        TokenType::LESS => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l < r));
            }
        }
        TokenType::GREATEREQ => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l >= r));
            }
        }
        TokenType::GREATER => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l > r));
            }
        }
        TokenType::TO => {
            if let (ValueType::Number(n1), ValueType::Number(n2)) = (l.value, r.value) {
                if (n1.fract() == 0.0) && (n2.fract() == 0.0) && n1 <= n2 {
                    let range = n1 as i64..n2 as i64 + 1;
                    return Ok(Value::array(
                        range.map(|x| Value::number(x as f64)).collect(),
                    ));
                }
            }
        }
        TokenType::STEP => {
            if let (ValueType::Array(arr), ValueType::Number(step)) = (l.value, r.value) {
                let new_arr = arr.iter().step_by(step as usize).map(|x| x.to_owned());

                return Ok(Value::array(new_arr.collect()));
            }
        }

        //equality doesnt need a typecheck, if the Value object is the same, two values are the
        //same
        TokenType::EQEQ => {
            return Ok(Value::bool(l == r));
        }
        TokenType::BANGEQ => {
            return Ok(Value::bool(l != r));
        }
        _ => (),
    }

    let op_vec = vec![l_copy.clone(), r_copy.clone()];
    let op_overloadings = overloadings.get(&(operator.clone(), 2));

    //again if some overloadings exist we evaluater them and return the result
    if let Some(op_overloadings) = op_overloadings {
        if let Ok(result) = eval_overloaded(
            op_overloadings.to_vec(),
            op_vec,
            enclosing.clone(),
            operator_token_index,
        ) {
            return Ok(result);
        }
    }

    return Ok(Value::default());
}

pub fn eval(
    expr: &Expression,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
) -> Result<Value, ThorLangError> {
    //recursivley traverses the expr tree.
    match expr {
        //retrieve has to work for arrays like this array[number];
        //and for objects like this object[key];
        Expression::Retrieve {
            retrievee,
            key,
            lbrack_token_index,
        } => {
            let key = eval(key, enclosing.clone(), overloadings)?;

            let retrievee = eval(retrievee, enclosing.clone(), overloadings)?;

            let mut ret_val = Value::nil();

            match (retrievee.value.clone(), key.value) {
                //the case of array and number
                (ValueType::Array(arr), ValueType::Number(num)) => {
                    if num.round() != num {
                        return ThorLangError::index_error(
                            lbrack_token_index.clone(),
                            retrievee,
                            num,
                        );
                    }
                    if let Some(el) = arr.get(num as usize) {
                        ret_val = el.clone();
                    } else {
                        return ThorLangError::index_error(
                            lbrack_token_index.clone(),
                            retrievee,
                            num,
                        );
                    }
                }
                (ValueType::String(str), ValueType::Number(num)) => {
                    if num.round() != num {
                        return ThorLangError::index_error(
                            lbrack_token_index.clone(),
                            retrievee,
                            num,
                        );
                    }

                    if let Some(char) = str.chars().nth(num as usize) {
                        ret_val = Value::string(char.to_string());
                    } else {
                        return ThorLangError::index_error(
                            lbrack_token_index.clone(),
                            retrievee,
                            num,
                        );
                    }
                }
                //the case of object and string
                (ValueType::Object, ValueType::String(str)) => {
                    if let Some(val) = retrievee.clone().fields.get(&str) {
                        ret_val = (val.clone());
                    } else {
                        ret_val = (Value::nil());
                    }
                }
                _ => {
                    return ThorLangError::unknown_value_error(lbrack_token_index + 1)
                }
            }


            ret_val.library = retrievee.library;

            return Ok(ret_val)
        }

        Expression::Try { block } => {
            //this ensures that errors can be returned as values
            let eval_value = eval_statement(block.to_vec(), enclosing.clone(), overloadings);
            return match eval_value {
                Ok(val) => Ok(val),
                Err(err) => {
                    let err = Value::error(err);
                    return Ok(err);
                }
            };
        }
        Expression::On {
            block,
            variables,
            on_token_index,
        } => {
            //listeners here

            if let Some(block) = block {

                for variable in variables {
                    enclosing.lock().unwrap().add_listener(
                    variable.get_content().unwrap(),
                    block.clone(),
                    *on_token_index,
                )?;

                }
                
                return Ok(Value::nil());
            }

            ThorLangError::eval_error(*on_token_index)
        }

        Expression::FieldCall {
            callee,
            key,
            dot_token_index: _,
        } => {
            //getting with the . (object.field)

            //first we need to get the key and the field we want to call from
            let callee_value = eval(callee, enclosing.clone(), overloadings)?;

            let key_string: String;

            //if the key is an identifier we turn it to a string else we would hash it (hashing
            //does not work, thinking about removing this feature)
            if let Expression::Identifier {
                name,
                identifier_token_index: _,
            } = *(*key).clone()
            {
                key_string = name;
            } else {
                key_string = hash_value(eval(key, enclosing.clone(), overloadings)?);
            }
            

            //the default value is nil (field does not exist)

            let mut ret_val = Value::default();

            //if a field with the above name does exist we return it
            if let Some(field) = callee_value.fields.get(&key_string) {
                
                
                let mut field = field.clone();
                if let Some(library) = callee_value.library{

                    field.library = Some(library.clone());
                }

                //field.library = callee_value.library;

                return Ok(field);
            }
            //else we try to return a value or method of the prototype
            //depending on whether the value we want to call from the prototype method map
            //(init_prototype_fields)
            match callee_value.value.clone() {
                ValueType::String(_str) => {
                    if let Some(field) =
                        register_string_methods(callee_value.clone()).get(&key_string)
                    {
                        ret_val = field.clone();
                    }
                }
                ValueType::Number(num) => {
                    if let Some(field) =
                        register_number_methods(callee_value.clone()).get(&key_string)
                    {
                        ret_val = field.clone()
                    }
                }
                ValueType::Array(arr) => {
                    let mut var_name = "".to_string();

                    if let Expression::Identifier {
                        name,
                        identifier_token_index,
                    } = *callee.clone()
                    {
                        var_name = name;
                    }

                    if let Some(field) =
                        register_array_methods(callee_value.clone(), var_name).get(&key_string)
                    {
                        ret_val = field.clone();
                    }
                }

                //not finished yet, but can be at every moment
                _ => (),
            }

            //if still no fields with the given name are found we return nil

            ret_val.library = callee_value.library;


            Ok(ret_val)
        }

        Expression::Array { values } => {
            //calculates the value of an array (simply a value with the type array itself)
            let mut value_array: Vec<Value> = vec![];

            for value_expression in values {
                let value = eval(value_expression, enclosing.clone(), overloadings)?;

                value_array.push(value);
            }

            Ok(Value::array(value_array))
        }
        Expression::Object { keys, values } => {
            let mut value_object = Value::nil();
            value_object.value = ValueType::Object;

            for i in 0..keys.len() {
                let key = eval(&keys[i].clone(), enclosing.clone(), overloadings)?;
                let value = eval(&values[i].clone(), enclosing.clone(), overloadings)?;

                let key_string = stringify_value(key.clone());



                value_object.fields.insert(key_string, value);
            }

            Ok(value_object)
        }
        Expression::Call {
            callee,
            paren_token_index,
            arguments,
        } => {
            //to evaluate functions we need to distinguish between native functions and thor
            //functions
            let function = eval(callee, enclosing.clone(), overloadings)?;

            if let ValueType::Function(Function::LibFunction {
                name,
                needed_arguments,
                self_value,
                mutating,
            }) = &function.value
            {
                if needed_arguments.len() != arguments.len() {
                    return ThorLangError::function_arity_error(
                        paren_token_index.clone(),
                        needed_arguments.len(),
                        arguments.len(),
                    );
                }

                let mut eval_args: HashMap<String, Value> = HashMap::new();

                for i in 0..arguments.len() {
                    let arg = eval(arguments.get(i).unwrap(), enclosing.clone(), overloadings)?;
                    let arg_name = needed_arguments.get(i).unwrap();

                    eval_args.insert(arg_name.to_string(), arg);
                }

                return execute_lib_function(function, eval_args, enclosing, overloadings);
            }

            if let ValueType::Function(Function::NamedFunction {
                name,
                needed_arguments,
                self_value,
                env_state,
                var_name,
            }) = function.clone().value
            {
                if needed_arguments.len() != arguments.len() {
                    return ThorLangError::function_arity_error(
                        paren_token_index.clone(),
                        needed_arguments.len(),
                        arguments.len(),
                    );
                }

                let mut eval_args: HashMap<String, Value> = HashMap::new();

                for i in 0..arguments.len() {
                    let arg = eval(arguments.get(i).unwrap(), enclosing.clone(), overloadings)?;
                    let arg_name = needed_arguments.get(i).unwrap();

                    eval_args.insert(arg_name.to_string(), arg);
                }

                let function = get_registered_function(name)?;

                let self_value = match self_value {
                    Some(self_value) => Some(*self_value),
                    None => None,
                };

                return function(
                    eval_args,
                    self_value,
                    Some(enclosing.clone()),
                    var_name,
                    env_state,
                );
            }

            if let ValueType::Function(Function::ThorFunction {
                body,
                needed_arguments,
                closure,
            }) = function.value
            {
                if needed_arguments.len() != arguments.len() {
                    return ThorLangError::function_arity_error(
                        paren_token_index.clone(),
                        needed_arguments.len(),
                        arguments.len(),
                    );
                }

                // Evaluate the arguments in the current environment
                let mut eval_args: HashMap<String, Value> = HashMap::new();
                for i in 0..arguments.len() {
                    let arg = eval(arguments.get(i).unwrap(), enclosing.clone(), overloadings)?;
                    let arg_name = needed_arguments.get(i).unwrap();
                    eval_args.insert(arg_name.to_string(), arg);
                }

                // Create a new environment for the function call, using the closure's environment
                let function_env = Environment::new(Some(closure.clone())); // Only capture the closure's environment

                for (name, value) in eval_args {
                    function_env
                        .lock()
                        .unwrap()
                        .values
                        .borrow_mut()
                        .insert(name, value);
                }

                //when the function is a thorfunction we need to eval the block (body of the
                //function) with an environemnt that is copied from the current one but appended
                //with the arguments
                eval_statement(body, function_env, overloadings)
            } else {
                return ThorLangError::unkown_function_error(paren_token_index.clone());
            }
        }

        Expression::Assignment {
            target,
            value,
            eq_token_index,
        } => {
            let eval_value = eval(value, enclosing.clone(), overloadings)?;

            //Assignment needs to find the target first

            //iteratively go over the fields (creating them when they do not exist) and putting in
            //the value at the deepest level
            //
            //iterating over order (a vector of keys, can be numbers for arrays or strings for
            //objects)
            let order = generate_field_order(target.clone(), enclosing.clone(), overloadings)?;

            let value: &mut Value = &mut enclosing
                .lock()
                .unwrap()
                .get(&order.get(0).unwrap().0.get_string().unwrap().to_string())
                .unwrap()
                .clone();

            if order.len() == 1 {
                let set_val = enclosing.lock().unwrap().set(
                    order.get(0).unwrap().0.get_string().unwrap(),
                    eval_value.clone(),
                    *eq_token_index,
                )?;

                if let Some(listeners) = set_val.listeners {
                    for listener in listeners {
                        let _ = eval_statement(listener, enclosing.clone(), overloadings);
                    }
                }

                return Ok(eval_value);
            }

            let mut current: &mut Value = value;

            //runs for the first n - 1 items in the order list
            for i in 1..(order.len() - 1) {
                //nil values that get fields reassigned become objects
                let immut_value = current.clone();

                let current_field_key = &order.get(i).unwrap().0;

                let current_field_key_index = &order.get(i).unwrap().1;

                //in this case we have an array call (-Assignment)
                if let FieldKey::Int(num) = current_field_key {
                    if let ValueType::Array(ref mut arr) = current.value {
                        if let Some(current_mut) = arr.get_mut(*num as usize) {
                            current = current_mut
                        } else {
                            return ThorLangError::index_error(
                                current_field_key_index - 1,
                                immut_value,
                                *num as f64,
                            );
                        }
                    }
                }

                //in this one a fieldcall (-Assignment)
                if let FieldKey::String(ref str) = current_field_key {
                    if let Some(field) = current.fields.get_mut(str) {
                        current = field;
                    } else {
                        return ThorLangError::retrieval_error(current_field_key_index - 1);
                    }
                }
            }

            let last_key = order.get(order.len() - 1).unwrap();

            //runs for the last (nth) field in the order list (or the first when we assing to a
            //single variable)
            match &last_key.0 {
                FieldKey::String(key) => {
                    current.fields.insert(key.to_string(), eval_value.clone());
                }
                FieldKey::Int(num) => {
                    if let ValueType::Array(arr) = &mut current.value {
                        if let Some(_value) = arr.get(*num as usize) {
                            arr[*num as usize] = eval_value.clone();
                        } else {
                            return ThorLangError::index_error(
                                last_key.1,
                                current.clone(),
                                *num as f64,
                            );
                        }
                    }
                }
            }

            //if we assign to a value that is nil we make it an object
            //so we can make something like this:
            //
            //let obj;
            //obj.hello = 4;
            //^^^
            //this makes obj an object
            if current.value == ValueType::Nil {
                current.value = ValueType::Object
            }

            let set_val = enclosing.lock().unwrap().set(
                order.get(0).unwrap().0.get_string().unwrap().to_string(),
                value.clone(),
                *eq_token_index,
            )?;

            if let Some(listeners) = set_val.listeners {
                for listener in listeners {
                    let _ = eval_statement(listener, enclosing.clone(), overloadings);
                }
            }

            return Ok(eval_value);
        }

        //kind of like literals, but will replace instantly with the value behind the variable name
        //instead of going down to literals first
        Expression::Identifier {
            name,
            identifier_token_index,
        } => {
            let value = enclosing.lock().unwrap().get(name);

            if let Some(val) = value {
                return Ok(val);
            } else {
                return ThorLangError::unknown_value_error(*identifier_token_index);
            }
        }

        //these just return the value they evaluate to
        Expression::Unary {
            operator,
            right,
            operator_token_index,
        } => {
            return eval_unary(
                operator.clone(),
                &right,
                enclosing.clone(),
                overloadings,
                *operator_token_index,
            )
        }
        Expression::Literal {
            literal,
            literal_token_index,
        } => return eval_literal(literal.clone(), *literal_token_index),
        Expression::Grouping { inner } => return eval(&inner, enclosing, overloadings),
        Expression::Binary {
            left,
            operator,
            right,
            operator_token_index,
        } => {
            return eval_binary(
                &left,
                operator.clone(),
                &right,
                enclosing,
                overloadings,
                *operator_token_index,
            )
        }
    }
}

#[derive(Debug)]
enum FieldKey {
    Int(i32),
    String(String),
}

//methods to get the strings and numbers easier without doing if lets all the time
impl FieldKey {
    fn get_string(&self) -> Option<String> {
        return match self {
            FieldKey::Int(_num) => None,
            FieldKey::String(string) => Some(string.to_string()),
        };
    }
}

//generates the field Assignment order by traversing the field calls and returning the steps we
//need to take
//
//obj.hello[0]["hello"] turns to ["hello", 0, "hello"]
fn generate_field_order(
    target: Box<Expression>,
    enclosing: Arc<Mutex<Environment>>,
    overloadings: &mut Overloadings,
) -> Result<Vec<(FieldKey, usize)>, ThorLangError> {
    let mut order = vec![];

    let mut current = target;

    let mut not_ended = true;

    while not_ended {
        match *current.clone() {
            //when a single identifier we just return the name (of the variable)
            Expression::Identifier {
                name,
                identifier_token_index,
            } => {
                order.push((FieldKey::String(name), identifier_token_index));
                not_ended = false;
            }

            //if it is a fieldcall we return the key name
            Expression::FieldCall {
                callee,
                key,
                dot_token_index,
            } => {
                if let Expression::Identifier {
                    name,
                    identifier_token_index,
                } = *key
                {
                    order.push((FieldKey::String(name), identifier_token_index.clone()));
                } else {
                    order.push((
                        FieldKey::String(hash_value(eval(&key, enclosing.clone(), overloadings)?)),
                        dot_token_index,
                    ));
                }

                current = callee
            }

            //same in the case of a retrieve however this can also be an integer
            Expression::Retrieve {
                retrievee,
                key,
                lbrack_token_index,
            } => {
                let key = eval(&key, enclosing.clone(), overloadings)?;

                match key.value {
                    ValueType::String(str) => {
                        let key = str;

                        order.push((FieldKey::String(key), lbrack_token_index));
                    }
                    ValueType::Number(num) => {
                        if num.round() != num.round() {
                            let eval_value = eval(&retrievee, enclosing.clone(), overloadings)?;

                            if let Err(err) =
                                ThorLangError::index_error(lbrack_token_index, eval_value, num)
                            {
                                return Err(err);
                            }
                        }

                        let index = num as i32;

                        order.push((FieldKey::Int(index), lbrack_token_index));
                    }
                    _ => unimplemented!(),
                }

                current = retrievee;
            }
            _ => (),
        }
    }

    order.reverse();
    Ok(order)
}

//helper function used in dot with numbers around
//this function could also be made with a "format!" call but this is some math wizardry and i like
//it
fn combine_f64(num1: f64, num2: f64) -> f64 {
    // Determine the number of decimal places for num2
    let decimal_places = 10f64.powi(num2.abs().log10().floor() as i32 + 1);

    // Shift num2 to the right to match it as a fractional part
    let fractional_part = num2 / decimal_places;

    // Combine num1 with the fractional part of num2
    num1 + fractional_part
}
