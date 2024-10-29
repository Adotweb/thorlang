use crate::{
    hash_value, init_array_fields, init_bool_fields, init_number_fields, init_string_fields,
    stringify_value, Expression, Statement, TokenType,
    ThorLangError
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
//eval statements

//this is a bit more complicated, Rc<Refcell<T>> provides us with the ability to mutate the entire
//environment object at will (its just some trickery so we can do that) terrible performance
//decision, but makes it work
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, Value>>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct OperationInfo {
    pub operands : Vec<String>,
    pub operation : Vec<Statement>,
    pub overloadings : Overloadings
}

//Hashmap that returns a operation given an operator (TokenType) and an arity (usize)
type Overloadings = HashMap<(TokenType, usize), Vec<OperationInfo>>;

//its easier to instantiate a get and set function that automatically search the entire env tree
//(for existence for example) to
//look for a value than doing that over and over again in the later following code
impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: RefCell::new(HashMap::new()),
            enclosing,
        }))
    }
   
    //this method recursively iterates through the environment in search of the given key
    pub fn get(&self, key: &str) -> Option<Value> {
        
        //in case that the current environment level already contains the key, return the value
        //behind it
        if let Some(value) = self.values.borrow().get(key) {
            Some(value.clone())
        } 
        //if not we return a reference to the environment that closes over the current one and
        //apply this "get" method to it
        else if let Some(ref parent) = self.enclosing {
            parent.borrow().get(key)
        }
        //if both of the above fail the key does not exist (and so the variable does not)
        else {
            None
        }
    }


    //almost the same as with get but we have to change the .borrow() (normal reference) to a
    //.borrow_mut() (mutable reference) since we want to be able to change whatever value we
    //encounter
    pub fn set(&self, key: String, value: Value, eq_token_index : usize) -> Result<Value, ThorLangError> {
        
        if self.values.borrow().contains_key(&key) {
            let p = self.values.borrow_mut().insert(key, value);
            Ok(p.unwrap())
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow_mut().set(key, value, eq_token_index)
        } else {

            //this is for safety measures, because Assignment automatically inserts any key that
            //does not yet exist in the environment


            ThorLangError::eval_error(eq_token_index)

        }
    }

}

//here the magic happpens: every list of statements mutates the env tree, and as soon as a branch
//of the env tree is not needed it automatically disappears, meaning that we can only
// - mutate variables that exist
// - add variables to the currently used branch of the env tree
//
//this ensures that as soon as a branch is exited (i.e a block is done executing) we will have the
//old values back.
pub fn eval_statement(stmts: Vec<Statement>, enclosing: Rc<RefCell<Environment>>, overloadings : &mut Overloadings) -> Result<Value, ThorLangError> {

    //evaluating statement by statement
    for stmt in stmts {
        match stmt {

            //not implemented quite yet
            Statement::Throw { exception, throw_token_index } => {

                let exception = eval(&exception, enclosing, overloadings)?;

                return Err(ThorLangError::ThorLangException{
                    exception : Box::new(exception),
                    throw_token_index
                })

            },

            //this works basically like a function, except that the call operation takes place in
            //the corresponding eval function
            Statement::Overload { operator, operands, operation, line : _ } => {
                
                let arity = operands.len();



                //we dont want to mess with already defined overloaded operators, so any overlaoded
                //operator will only be able to access before initialized (overloaded) operators,
                //this means that similar to function environments we have to pass a operator env
                let defined_overloadings = overloadings.clone();

                let operator_info = OperationInfo{ 
                        operands, 
                        operation, 
                        overloadings : defined_overloadings
                };

                //if there is a list of overloadings for the given operator already we just push
                //this overloading to it. Else we create such a list for use in the future
                if let Some(operationlist) = overloadings.get_mut(&(operator.clone(), arity)){
                    operationlist.insert(0, operator_info);
                }else{

                    overloadings.insert((operator, arity), vec![operator_info]);
                }
            },
            Statement::Return { expression, line : _ } => {
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
                line : _
            } => {
                let closure = Rc::new(RefCell::new(enclosing.borrow().clone()));

                let function = Value::thor_function(arguments, *body, closure.clone());

                //insert the function with its name into the environment
                enclosing
                    .borrow_mut()
                    .values
                    .borrow_mut()
                    .insert(name.clone(), function.clone());


                //insert the function with its name into the closure to allow for recursion
                closure
                    .borrow_mut()
                    .values
                    .borrow_mut()
                    .insert(name, function);
            }
            //a block just opens a new env tree branch
            Statement::Block { statements, line : _ } => {
                let local_scope = Environment::new(Some(enclosing.clone()));
                eval_statement(statements, local_scope.clone(), overloadings)?;
            }

            //if statements are one to one in the host language 
            Statement::If {
                condition,
                then_branch,
                else_branch,
                line : _
            } => {
                if let ValueType::Bool(bool) = eval(&condition, enclosing.clone(), overloadings)?.value {
                    if bool {
                        let return_val =
                            eval_statement(*then_branch, enclosing.clone(), overloadings)?;

                         
            
                        //when we encounter a value with return_true = true, we need to bubble the
                        //value up
                        if return_val.return_true{

                            return Ok(return_val);
                        }

                    } else {
                        if let Some(ref _else_block) = else_branch {
                            let return_val =
                                eval_statement(*else_branch.unwrap(), enclosing.clone(), overloadings)?;


                            //same again
                            if return_val.return_true{
                                return Ok(return_val);
                            }
                        }
                    }
                }
            }
            

            //while statements are as well, we ned a "return_true" value however to detect when we
            //need to return a value
            Statement::While { condition, block, line : _  } => {
                let mut condition_value = eval(&condition.clone(), enclosing.clone(), overloadings)?;
                while let ValueType::Bool(bool) = condition_value.value {
                    if bool {
                        let return_val = eval_statement(*block.clone(), enclosing.clone(), overloadings)?;
                        condition_value = eval(&condition.clone(), enclosing.clone(), overloadings)?;


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

            //print is built in, but can also be used with the printfunction 
            Statement::Print { expression, line : _ } => {
                let result = eval(&expression, enclosing.clone(), overloadings)?;

                if let ValueType::String(ref str) = result.value {
                    println!("{str}");
                } else {
                    println!("{}", stringify_value(result));
                }

            }
        

            Statement::Do { expression, line : _ } => {
                //runs expressions
                eval(&expression, enclosing.clone(), overloadings)?;
            }

            //variable declaration only ever mutates the current branch of the env tree ensuring,
            //in this case "enclosing"
            Statement::Variable { name, expression, line : _ } => {
                let val = eval(&expression, enclosing.clone(), overloadings)?;

                enclosing.borrow_mut().values.borrow_mut().insert(name, val);
            }
        }
    }

    Ok(Value::default())
}


//Functions are either built into rust (rust closures) or defined as a procedure in thor itself
//both do the same but have different data to them
#[derive(Clone)]
pub enum Function {
    NativeFunction {
        //this atrocious type is the dynamic closure type in rust (functions that have closures)
        //because we cant use pure functions
        body: Arc<dyn Fn(HashMap<String, Value>) -> Result<Value, ThorLangError>>,
        needed_arguments: Vec<String>,
        self_value: Option<Box<Value>>,
    },
    ThorFunction {
        body: Vec<Statement>,
        needed_arguments: Vec<String>,
        closure: Rc<RefCell<Environment>>,
    },
}

//later i have to implement equality for functions
//since we cannot create a function that is "equal" in any sense to a native function it will only
//be the case for thorfunctions (equality on arguments and body, the env is not important when )
impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
    fn ne(&self, _other: &Self) -> bool {
        true
    }
}

//printing functions also has to be implemented in the future
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::NativeFunction {
                body: _,
                needed_arguments,
                self_value: _,
            } => f
                .debug_struct("Function")
                .field("args", needed_arguments)
                .finish(),
            Function::ThorFunction {
                body: _,
                needed_arguments,
                closure: _,
            } => f
                .debug_struct("Function")
                .field("args", needed_arguments)
                .finish(),
        }
    }
}


//i rewrote this to improve the code readability and logic, unlike before we can just get the value
//given that it has some type, data that is not represantable simply cant exist and we dont have no
//unwraps all over the place anymore
#[derive(PartialEq, Debug, Clone)]
pub enum ValueType {
    String(String),
    Number(f64),
    Bool(bool),
    Function(Function),
    Array(Vec<Value>),
    Error(ThorLangError),
    Object,
    Nil,
}



//this is still the same, everything 
#[derive(PartialEq, Debug, Clone)]
pub struct Value {
    pub value: ValueType,
    pub fields: HashMap<String, Value>,
    pub return_true: bool,
}

//nice instantiation functions for values 
//default will return the nil value
impl Value {
    
    pub fn array(value: Vec<Value>) -> Value {
        Value {
            value: ValueType::Array(value),
            ..Value::default()
        }
    }

    pub fn number(value: f64) -> Value {
        Value {
            value: ValueType::Number(value),
            ..Value::default()
        }
    }

    pub fn string(value: String) -> Value {
        Value {
            value: ValueType::String(value),
            ..Value::default()
        }
    }

    pub fn bool(value: bool) -> Value {
        Value {
            value: ValueType::Bool(value),
            ..Value::default()
        }
    }

    pub fn nil() -> Value {
        Value {
            value: ValueType::Nil,
            ..Value::default()
        }
    }

    pub fn error(err : ThorLangError) -> Value{
        Value{
            value : ValueType::Error(err),
            ..Value::default()
        }
    }

    //native functions still need to be instantiated using absolutely horrendous typing
    pub fn native_function(
        arguments: Vec<&str>,
        body: Arc<dyn Fn(HashMap<String, Value>) -> Result<Value, ThorLangError>>,
        self_value: Option<Box<Value>>,
    ) -> Value {
        Value {
            value: ValueType::Function(Function::NativeFunction {
                self_value,
                needed_arguments: arguments.iter().map(|x| x.to_string()).collect(),
                body,
            }),
            ..Value::default()
        }
    }

    //unlike thorfunctions which are just a holder for a block and a closure
    pub fn thor_function(
        arguments: Vec<String>,
        body: Vec<Statement>,
        closure: Rc<RefCell<Environment>>,
    ) -> Value {
        Value {
            value: ValueType::Function(Function::ThorFunction {
                needed_arguments: arguments,
                body,
                closure,
            }),
            ..Value::default()
        }
    }
}


//returns nil value
impl Default for Value {
    fn default() -> Value {
        Value {
            value: ValueType::Nil,
            fields: HashMap::new(),
            return_true: false,
        }
    }
}

//helper function to check whether or not a operation works for the inputs provided
fn eval_overloaded(operation_list : Vec<OperationInfo>, arguments : Vec<Value>, enclosing: Rc<RefCell<Environment>>, operator_token_index : usize) 
    -> Result<Value, ThorLangError>{
   
    let op_env = enclosing.borrow().clone();


    //loops over every operation associated with the given operator sign
    //when they throw we move on to the next one
    for mut op in operation_list{
        let operands = op.operands;
        let operation = op.operation;
        let overloadings = &mut op.overloadings; 
    

        if operands.len() != arguments.len(){

            return ThorLangError::operation_arity_error(operator_token_index, operands.len(), arguments.len())
        }

        for i in 0..operands.len(){
            op_env.values.borrow_mut().insert(operands[i].clone(), arguments[i].clone());
        }

        let tried_eval = eval_statement(operation, Rc::new(RefCell::new(op_env.clone())), overloadings);

        if let Ok(result) = tried_eval{
            return Ok(result)
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
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings,
    operator_token_index : usize
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

                return ThorLangError::eval_error(operator_token_index)
            }
        }
        //only negate arithmetically when is number
        TokenType::MINUS => {
            if let ValueType::Number(num) = r.value {
                //also jsut one to one
                return Ok(Value::number(-num));
            } else {

                //these errors will be overworked in the future
                
                return ThorLangError::eval_error(operator_token_index) 
                
            }
        }
        _=> ()
    }

        if let Some(operation_info) = overloadings.get(&(operator.clone(), 1)){

                if let Ok(result) = eval_overloaded(operation_info.to_vec(), vec![r.clone()], enclosing.clone(), operator_token_index){

            
                    return Ok(result)
                }else{
                    return ThorLangError::eval_error(operator_token_index);
                }

            
        }else{
                return ThorLangError::eval_error(operator_token_index);
        }
}

//evaluates the "atoms" these can not be further reduced and bubble up to form more complex data
//(not types but composed values like 1 + 2)
fn eval_literal(literal: TokenType, literal_token_index : usize) -> Result<Value, ThorLangError> {
    //turn literaltype into value wrapped in value_type
    match literal {
        TokenType::NIL => return Ok(Value::nil()),
        TokenType::TRUE  => {
            let mut ret_val = Value::bool(true);
            ret_val.fields = init_bool_fields(ret_val.clone());
            return Ok(ret_val)
        },
        TokenType::FALSE  => {
            let mut ret_val = Value::bool(false);
            ret_val.fields = init_bool_fields(ret_val.clone());
            return Ok(ret_val)
        },
        TokenType::NUMBER(value) => { 
            let mut ret_val = Value::number(value.parse().unwrap());
            ret_val.fields = init_bool_fields(ret_val.clone());
            return Ok(ret_val)
        },
        TokenType::STRING(value) => {
            let mut ret_val = Value::string(value);
            ret_val.fields = init_bool_fields(ret_val.clone());
            return Ok(ret_val)
        },
        _ => ThorLangError::eval_error(literal_token_index) 
    }
}

//all the possible binary operation combinations.
fn eval_binary(
    left: &Expression,
    operator: TokenType,
    right: &Expression,
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings,
    operator_token_index : usize
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

        //equality doesnt need a typecheck, if the Value object is the same, two values are the
        //same
        TokenType::EQEQ => {
            return Ok(Value::bool(l == r));
        }
        TokenType::BANGEQ => {
            return Ok(Value::bool(l != r));
        }
        _ => {





            return ThorLangError::eval_error(operator_token_index)
        }
    }
   
    let op_vec = vec![l_copy.clone(), r_copy.clone()];
    let op_overloadings = overloadings.get(&(operator.clone(), 2));


    //again if some overloadings exist we evaluater them and return the result
    if let Some(op_overloadings) = op_overloadings {
        if let Ok(result) = eval_overloaded(op_overloadings.to_vec(), op_vec, enclosing.clone(), operator_token_index){
                return Ok(result)
        }
    }

    return Ok(Value::default());
}

pub fn eval(expr: &Expression, enclosing: Rc<RefCell<Environment>>, overloadings : &mut Overloadings) -> Result<Value, ThorLangError> {
    //recursivley traverses the expr tree.
    match expr {
        //retrieve has to work for arrays like this array[number];
        //and for objects like this object[key];
        Expression::Retrieve { retrievee, key, lbrack_token_index } => {
            let key = eval(key, enclosing.clone(), overloadings)?;

            let retrievee = eval(retrievee, enclosing.clone(), overloadings)?;

            match (retrievee.value.clone(), key.value) {
                //the case of array and number
                (ValueType::Array(arr), ValueType::Number(num)) => {
                    if num.round() != num {
                        return ThorLangError::index_error(lbrack_token_index.clone(), retrievee, num);
                    }
                    if let Some(el) =  arr.get(num as usize){
                        Ok(el.clone())
                    } else {

                        return ThorLangError::index_error(lbrack_token_index.clone(), retrievee, num);
                    }    
                },
                (ValueType::String(str), ValueType::Number(num)) => {
                    if num.round() != num {
                        return ThorLangError::index_error(lbrack_token_index.clone(), retrievee, num);
                    }

                    if let Some(char) = str.chars().nth(num as usize){
                        Ok(Value::string(char.to_string()))
                    }else {

                        return ThorLangError::index_error(lbrack_token_index.clone(), retrievee, num);
                    }    
                },
                //the case of object and string
                (ValueType::Object, ValueType::String(str)) =>{
                    if let Some(val) = retrievee.clone().fields.get(&str){
                        return Ok(val.clone())
                    } else {
                        return Ok(Value::nil())    
                    }

                }
                _ => ThorLangError::unknown_value_error(lbrack_token_index + 1)

            }
        }
        
        Expression::Try { block } => {
            //this ensures that errors can be returned as values 
            let eval_value = eval_statement(block.to_vec(), enclosing.clone(), overloadings); 
                return match eval_value {
                    Ok(val) => Ok(val),
                    Err(err) => {
                        let err = Value::error(err);
                        return Ok(err)
                    }
                }
        },

        Expression::FieldCall { callee, key, dot_token_index : _ } => {

            //getting with the . (object.field)

            //first we need to get the key and the field we want to call from
            let callee_value = eval(callee, enclosing.clone(), overloadings)?;
            let key_string: String;
            


            //if the key is an identifier we turn it to a string else we would hash it (hashing
            //does not work, thinking about removing this feature)
            if let Expression::Identifier { name, identifier_token_index : _ } = *(*key).clone() {
                key_string = name;
            } else {
                key_string = hash_value(eval(key, enclosing.clone(), overloadings)?);
            }
            
            //the default value is nil (field does not exist)
            
            let mut ret_val = Value::default();

            //if a field with the above name does exist we return it
            if let Some(field) = callee_value.fields.get(&key_string) {
                return Ok(field.clone());
            }
            //else we try to return a value or method of the prototype
            //depending on whether the value we want to call from the prototype method map
            //(init_prototype_fields)
            match callee_value.value.clone() {
                ValueType::String(_str) => {
                    if let Some(field) = init_string_fields(callee_value.clone()).get(&key_string) {
                        ret_val = field.clone();
                    }
                },
                ValueType::Number(num) => {
                    if let Some(field) = init_number_fields(callee_value.clone()).get(&key_string){
                        ret_val = field.clone()
                    }
                },
                ValueType::Array(arr) => {
                    let mut var_name = "".to_string();
                    
                    if let Expression::Identifier { name, identifier_token_index } = *callee.clone() {
                        var_name = name;
                    }

                    if let Some(field) = 
                        init_array_fields(callee_value.clone(), enclosing.clone(), var_name).get(&key_string){
                        ret_val = field.clone();
                    }
                }

                //not finished yet, but can be at every moment
                _ => (),
            }
    
            
            //if still no fields with the given name are found we return nil

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
        Expression::Call {
            callee,
            paren_token_index,
            arguments,
        } => {
            //to evaluate functions we need to distinguish between native functions and thor
            //functions
            let function = eval(callee, enclosing.clone(), overloadings)?;

            if let ValueType::Function(Function::NativeFunction {
                body,
                needed_arguments,
                self_value,
            }) = function.clone().value
            {

                 
                //check if arity of args is ok
                if needed_arguments.len() != arguments.len() {

                    return ThorLangError::function_arity_error(paren_token_index.clone(), needed_arguments.len(), arguments.len());
                }

                let mut eval_args: HashMap<String, Value> = HashMap::new();

                for i in 0..arguments.len() {
                    let arg = eval(arguments.get(i).unwrap(), enclosing.clone(), overloadings)?;
                    let arg_name = needed_arguments.get(i).unwrap();

                    eval_args.insert(arg_name.to_string(), arg);
                }

                if let Some(sv) = self_value {
                    eval_args.insert("self".to_string(), *sv.clone());
                }

                //we can just call the function with the args given that the functions is rust
                //builtin
                let function = body(eval_args)?;

                return Ok(function);
            }

            if let ValueType::Function(Function::ThorFunction {
                body,
                needed_arguments,
                closure,
            }) = function.value
            {
                if needed_arguments.len() != arguments.len() {
                    

                    return ThorLangError::function_arity_error(paren_token_index.clone(), needed_arguments.len(), arguments.len());

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
                        .borrow_mut()
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

        Expression::Assignment { target, value, eq_token_index } => {
            let eval_value = eval(value, enclosing.clone(), overloadings)?;

            //Assignment needs to find the target first

            //iteratively go over the fields (creating them when they do not exist) and putting in
            //the value at the deepest level
            //
            //iterating over order (a vector of keys, can be numbers for arrays or strings for
            //objects)
            let order = generate_field_order(target.clone(), enclosing.clone(), overloadings)?;

            let value: &mut Value = &mut enclosing
                .borrow()
                .get(&order.get(0).unwrap().0.get_string().unwrap().to_string())
                .unwrap()
                .clone();

            if order.len() == 1 {
                enclosing.borrow_mut().set(
                    order.get(0).unwrap().0.get_string().unwrap(),
                    eval_value.clone(),
                    *eq_token_index
                )?;

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
                            
                            return ThorLangError::index_error(current_field_key_index - 1, immut_value , *num as f64)
                        }
                    }
                }

                //in this one a fieldcall (-Assignment)
                if let FieldKey::String(ref str) = current_field_key {
                    if let Some(field) = current.fields.get_mut(str) {
                        current = field;
                    } else {
                        return ThorLangError::retrieval_error(current_field_key_index - 1)
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

                        if let Some(_value) = arr.get(*num as usize){
                            arr[*num as usize] = eval_value.clone();
                        }
                        else  {
                            return ThorLangError::index_error(last_key.1, current.clone(), *num as f64)
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

            enclosing.borrow_mut().set(
                order.get(0).unwrap().0.get_string().unwrap().to_string(),
                value.clone(),
                *eq_token_index
            )?;

            return Ok(eval_value);
        }

        //kind of like literals, but will replace instantly with the value behind the variable name
        //instead of going down to literals first
        Expression::Identifier { name, identifier_token_index } => {
            let value = enclosing
                .borrow()
                .get(name);

            if let Some(val) = value{
                return Ok(val)
            } else {
                return ThorLangError::unknown_value_error(*identifier_token_index);
            }
        }
       
        //these just return the value they evaluate to 
        Expression::Unary { operator, right, operator_token_index } => return eval_unary(operator.clone(), &right, enclosing.clone(), overloadings, *operator_token_index),
        Expression::Literal { literal, literal_token_index } => return eval_literal(literal.clone(), *literal_token_index),
        Expression::Grouping { inner } => return eval(&inner, enclosing, overloadings),
        Expression::Binary {
            left,
            operator,
            right,
            operator_token_index
        } => return eval_binary(&left, operator.clone(), &right, enclosing, overloadings, *operator_token_index),
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
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings
) -> Result<Vec<(FieldKey, usize)>, ThorLangError> {
    let mut order = vec![];

    let mut current = target;

    let mut not_ended = true;

    while not_ended {
        match *current.clone() {

            //when a single identifier we just return the name (of the variable)
            Expression::Identifier { name, identifier_token_index } => {
                order.push((FieldKey::String(name), identifier_token_index));
                not_ended = false;
            }

            //if it is a fieldcall we return the key name
            Expression::FieldCall { callee, key, dot_token_index } => {
                if let Expression::Identifier { name, identifier_token_index } = *key {
                    order.push((FieldKey::String(name), identifier_token_index.clone()));
                } else {
                    order.push((FieldKey::String(hash_value(eval(&key, enclosing.clone(), overloadings)?)), dot_token_index));
                }

                current = callee
            }

            //same in the case of a retrieve however this can also be an integer
            Expression::Retrieve { retrievee, key, lbrack_token_index } => {
                let key = eval(&key, enclosing.clone(), overloadings)?;


                match key.value {
                    ValueType::String(str) => {
                        let key = str;

                        order.push((FieldKey::String(key), lbrack_token_index));
                    }
                    ValueType::Number(num) => {
                        
                        if num.round() != num.round(){
                            
                            let eval_value = eval(&retrievee, enclosing.clone(), overloadings)?;

                            if let Err(err) = ThorLangError::index_error(lbrack_token_index, eval_value, num){
                                return Err(err)
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
