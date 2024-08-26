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
    pub enclosing: Option<Rc<RefCell<Environment>>>
}

//Hashmap that returns a operation given an operator (TokenType) and an arity (usize)
type Overloadings = HashMap<(TokenType, usize), (Vec<Vec<Statement>>, Vec<String>)>;

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

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(value) = self.values.borrow().get(key) {
            Some(value.clone())
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow().get(key)
        } else {
            None
        }
    }

    pub fn set(&self, key: String, value: Value) -> Result<String, ThorLangError> {
        if self.values.borrow().contains_key(&key) {
            self.values.borrow_mut().insert(key, value);
            Ok("".to_string())
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow().set(key, value)
        } else {
            return Err(ThorLangError::EvalError(format!("no such variable {key} found")))
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


    for stmt in stmts {
        match stmt {

            //this works basically like a function, except that the call operation takes place in
            //the corresponding eval function
            Statement::Overload { operator, operands, operation } => {
                
                let arity = operands.len();

                if let Some(opartionlist) = overloadings.get_mut(&(operator.clone(), arity)){
                    opartionlist.0.push(operation);
                    return Ok(Value::nil())
                }
                

                overloadings.insert((operator, arity), (vec![operation], operands));

            },
            Statement::Return { expression } => {
                let mut ret_value = eval(&expression, enclosing.clone(), overloadings)?;

                ret_value.return_true = false;

                return Ok(ret_value);
            }

            Statement::Function {
                name,
                body,
                arguments,
            } => {
                let closure = Rc::new(RefCell::new(enclosing.borrow().clone()));

                let function = Value::thor_function(arguments, *body, closure);

                enclosing
                    .borrow_mut()
                    .values
                    .borrow_mut()
                    .insert(name, function);
            }
            //a block just opens a new env tree branch
            Statement::Block { statements } => {
                let local_scope = Environment::new(Some(enclosing.clone()));
                eval_statement(statements, local_scope.clone(), overloadings);
            }

            //if statements are one to one in the host language, makes it meta-programming...?
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let ValueType::Bool(bool) = eval(&condition, enclosing.clone(), overloadings)?.value {
                    if bool {
                        let mut return_val =
                            eval_statement(*then_branch, enclosing.clone(), overloadings)?;

                        return_val.return_true = true;

                        return Ok(return_val);
                    } else {
                        if let Some(ref _else_block) = else_branch {
                            let mut return_val =
                                eval_statement(*else_branch.unwrap(), enclosing.clone(), overloadings)?;

                            return_val.return_true = true;

                            return Ok(return_val);
                        }
                    }
                }
            }

            //still need to fix returns from while as i dont have a good way of detecing whether or
            //not something got returned....
            Statement::While { condition, block } => {
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
                        return Ok(Value::default())
                    }
                }
            }

            //print is built in
            Statement::Print { expression } => {
                let result = eval(&expression, enclosing.clone(), overloadings)?;

                if let ValueType::String(ref str) = result.value {
                    println!("{str}");
                } else {
                    println!("{}", stringify_value(result));
                }

            }

            Statement::Do { expression } => {
                //runs expressions
                eval(&expression, enclosing.clone(), overloadings);
            }

            //variable declaration only ever mutates the current branch of the env tree ensuring,
            //in this case "enclosing"
            Statement::Variable { name, expression } => {
                let val = eval(&expression, enclosing.clone(), overloadings)?;

                enclosing.borrow_mut().values.borrow_mut().insert(name, val);
            }
        }
    }

    Ok(Value::default())
}

//eval expressions

#[derive(Clone)]
pub enum Function {
    NativeFunction {
        body: Arc<dyn Fn(HashMap<String, Value>) -> Value>,
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

//to print arrays and later objects nicely

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

    pub fn native_function(
        arguments: Vec<&str>,
        body: Arc<dyn Fn(HashMap<String, Value>) -> Value>,
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

impl Default for Value {
    fn default() -> Value {
        Value {
            value: ValueType::Nil,
            fields: HashMap::new(),
            return_true: false,
        }
    }
}

//order of precedence is as follows
// eval_statement -> eval -> eval_binary -> eval_unary -> eval_literal

fn eval_unary(
    operator: TokenType,
    right: &Expression,
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings
) -> Result<Value, ThorLangError> {
    let r = eval(right, enclosing, overloadings)?;
    match operator {
        //only negate logically when is bool
        TokenType::BANG => {
            if let ValueType::Bool(bool) = r.value {
                return Ok(Value::bool(!bool));
            } else {

                Err(ThorLangError::EvalError("can only logically negate bools".to_string()))
            }
        }
        //only negate arithmetically when is number
        TokenType::MINUS => {
            if let ValueType::Number(num) = r.value {
                return Ok(Value::number(-num));
            } else {
                Err(ThorLangError::EvalError("can only negate numbers".to_string()))
            }
        }
        _ => {
            Err(ThorLangError::EvalError("unary operation not defined".to_string()))
        }
    }
}

//evaluates the "atoms" these can not be further reduced and bubble up to form more complex data
//(not types but composed values like 1 + 2)
fn eval_literal(literal: TokenType) -> Result<Value, ThorLangError> {
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
        _ => Err(ThorLangError::EvalError("this is not a literal".to_string()))
    }
}

//all the possible binary operation combinations.
fn eval_binary(
    left: &Expression,
    operator: TokenType,
    right: &Expression,
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings
) -> Result<Value, ThorLangError> {
    let l = eval(left, enclosing.clone(), overloadings)?;
    let r = eval(right, enclosing.clone(), overloadings)?;

    if let Some(operationlist) = overloadings.get(&(operator.clone(), 2)){
        let mut first_correct_operation : Vec<Statement>;
       

        
        let operation_env = enclosing.borrow().clone();

        
        let operands = operationlist.1.clone();
        operation_env.values.borrow_mut().insert(operands[0].clone(), l.clone());
        operation_env.values.borrow_mut().insert(operands[1].clone(), r.clone());

        for operation in operationlist.0.clone() {
            let tried_operation = eval_statement(operation, Rc::new(RefCell::new(operation_env.clone())), 
                                                 &mut HashMap::new());
            
            if let Ok(result) = tried_operation{
                return Ok(result)
            }
        } 
    }

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

            return Err(ThorLangError::EvalError("can only add strings and numbers".to_string()))
        }
        TokenType::MINUS => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l + r));
            }

            return Err(ThorLangError::EvalError("can only subtract numbers".to_string()))
        }
        TokenType::STAR => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l * r));
            }
            
            return Err(ThorLangError::EvalError("can only multiply numbers".to_string()))
        }
        TokenType::SLASH => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::number(l / r));
            }


            return Err(ThorLangError::EvalError("can only divide numbers".to_string()))
        }
        TokenType::LESSEQ => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l <= r));
            }

            return Err(ThorLangError::EvalError("can only compare numbers".to_string()))
        }
        TokenType::LESS => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l < r));
            }

            return Err(ThorLangError::EvalError("can only compare numbers".to_string()))
        }
        TokenType::GREATEREQ => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l >= r));
            }

            return Err(ThorLangError::EvalError("can only compare numbers".to_string()))
        }
        TokenType::GREATER => {
            if let (ValueType::Number(l), ValueType::Number(r)) = (l.value, r.value) {
                return Ok(Value::bool(l > r));
            }

            return Err(ThorLangError::EvalError("can only compare numbers".to_string()))
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

    return Ok(Value::default());
}

pub fn eval(expr: &Expression, enclosing: Rc<RefCell<Environment>>, overloadings : &mut Overloadings) -> Result<Value, ThorLangError> {
    //recursivley traverses the expr tree.
    match expr {
        //retrieve has to work for arrays like this array[number];
        //and for objects like this object[key];
        Expression::Retrieve { retrievee, key } => {
            let key = eval(key, enclosing.clone(), overloadings)?;

            let retrievee = eval(retrievee, enclosing.clone(), overloadings)?;

            match (retrievee.value.clone(), key.value) {
                //the case of array and number
                (ValueType::Array(arr), ValueType::Number(num)) => {
                    if num.round() != num {
                        return Err(ThorLangError::EvalError(format!("can only access arrays with whole numbers")));
                    }
                    if let Some(el) =  arr.get(num as usize){
                        Ok(el.clone())
                    } else {
                        return Err(ThorLangError::EvalError(format!("index : {} is out of bound : {}", num, arr.len())))
                    }    
                }
                //the case of object and string
                (ValueType::Object, ValueType::String(str)) =>{
                    if let Some(val) = retrievee.clone().fields.get(&str){
                        return Ok(val.clone())
                    } else {
                        return Err(ThorLangError::EvalError(format!("field {} does not exist on {:?}", str, stringify!(retrievee))))
                    }

                }
                _ => Err(ThorLangError::EvalError(format!("{:?} is not retrievable", retrievee))),
            }
        }
        
        Expression::Try { block } => {
            
            let eval_value = eval_statement(block.to_vec(), enclosing.clone(), overloadings); 
                return match eval_value {
                    Ok(val) => Ok(val),
                    Err(err) => {
                        let err = Value::error(err);
                        return Ok(err)
                    }
                }
        },

        Expression::FieldCall { callee, key } => {
            let callee_value = eval(callee, enclosing.clone(), overloadings)?;
            let key_string: String;

            if let Expression::Identifier { name } = *(*key).clone() {
                key_string = name;
            } else {
                key_string = hash_value(eval(key, enclosing.clone(), overloadings)?);
            }

            let mut ret_val = Value::default();
            if let Some(field) = callee_value.fields.get(&key_string) {
                ret_val = field.clone();
            }

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
                    
                    if let Expression::Identifier { name } = *callee.clone() {
                        var_name = name;
                    }

                    if let Some(field) = 
                        init_array_fields(callee_value.clone(), enclosing.clone(), var_name).get(&key_string){
                        ret_val = field.clone();
                    }
                }
                _ => (),
            }

            Ok(ret_val)
        }

        Expression::Array { values } => {
            let mut value_array: Vec<Value> = vec![];

            for value_expression in values {
                let value = eval(value_expression, enclosing.clone(), overloadings)?;

                value_array.push(value);
            }

            Ok(Value::array(value_array))
        }
        Expression::Call {
            callee,
            paren: _,
            arguments,
        } => {
            //let eval_callee = eval(callee, enclosing.clone(), overloadings);

            let function = eval(callee, enclosing.clone(), overloadings)?;

            if let ValueType::Function(Function::NativeFunction {
                body,
                needed_arguments,
                self_value,
            }) = function.clone().value
            {
                //check if arity of args is ok
                if needed_arguments.len() != arguments.len() {
                    return Err(ThorLangError::EvalError(format!(
                        "function {:?} requires {:?} arguments but got {:?}",
                        function,
                        needed_arguments.len(),
                        arguments.len()
                    )))
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

                let function = body(eval_args);

                return Ok(function);
            }

            if let ValueType::Function(Function::ThorFunction {
                body,
                needed_arguments,
                closure,
            }) = function.value
            {
                if needed_arguments.len() != arguments.len() {
                    return Err(ThorLangError::EvalError(format!(
                        "Expected {} arguments but got {}",
                        needed_arguments.len(),
                        arguments.len()
                    )));
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

                eval_statement(body, function_env, overloadings)
            } else {
                return Err(ThorLangError::EvalError(format!("error in function {:?}", function)))
            }
        }

        Expression::Assignment { target, value } => {
            let eval_value = eval(value, enclosing.clone(), overloadings)?;

            //iteratively go over the fields (creating them when they do not exist) and putting in
            //the value at the deepest level
            //
            //iterating over order (a vector of keys, can be numbers for arrays or strings for
            //objects)
            let order = generate_field_order(target.clone(), enclosing.clone(), overloadings)?;

            let value: &mut Value = &mut enclosing
                .borrow()
                .get(&order.get(0).unwrap().get_string().unwrap().to_string())
                .unwrap()
                .clone();

            if order.len() == 1 {
                enclosing.borrow_mut().set(
                    order.get(0).unwrap().get_string().unwrap(),
                    eval_value.clone(),
                );

                return Ok(eval_value);
            }

            let mut current: &mut Value = value;

            for i in 1..(order.len() - 1) {
                //nil values that get fields reassigned become objects

                if let FieldKey::Int(num) = order.get(i).unwrap() {
                    if let ValueType::Array(ref mut arr) = current.value {

                        if let Some(current_mut) = arr.get_mut(*num as usize) {

                            current = current_mut
                        } else { 
                            return Err(ThorLangError::EvalError(format!("something went wrong")));
                        }
                    }
                }

                if let FieldKey::String(str) = order.get(i).unwrap() {
                    if let Some(field) = current.fields.get_mut(str) {
                        current = field;
                    } else {
                        return Err(ThorLangError::EvalError(format!("field {str} does not exist on object")))
                    }
                }
            }

            let last_key = order.get(order.len() - 1).unwrap();

            match last_key {
                FieldKey::String(key) => {
                    current.fields.insert(key.to_string(), eval_value.clone());
                }
                FieldKey::Int(num) => {
                    if let ValueType::Array(arr) = &mut current.value {
                        arr[*num as usize] = eval_value.clone()
                    }
                }
            }

            if current.value == ValueType::Nil {
                current.value = ValueType::Object
            }

            enclosing.borrow_mut().set(
                order.get(0).unwrap().get_string().unwrap().to_string(),
                value.clone(),
            );

            return Ok(eval_value);
        }

        //kind of like literals, but will replace instantly with the value behind the variable name
        //instead of going down to literals first
        Expression::Identifier { name } => {
            let value = enclosing
                .borrow()
                .get(name)
                .expect(&("this value does not exist ".to_string() + name));

            return Ok(value.clone());
        }
        
        Expression::Unary { operator, right } => return eval_unary(operator.clone(), &right, enclosing.clone(), overloadings),
        Expression::Literal { literal } => return eval_literal(literal.clone()),
        Expression::Grouping { inner } => return eval(&inner, enclosing, overloadings),
        Expression::Binary {
            left,
            operator,
            right,
        } => return eval_binary(&left, operator.clone(), &right, enclosing, overloadings),
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

fn generate_field_order(
    target: Box<Expression>,
    enclosing: Rc<RefCell<Environment>>,
    overloadings : &mut Overloadings
) -> Result<Vec<FieldKey>, ThorLangError> {
    let mut order = vec![];

    let mut current = target;

    let mut not_ended = true;

    while not_ended {
        match *current.clone() {
            Expression::Identifier { name } => {
                order.push(FieldKey::String(name));
                not_ended = false;
            }
            Expression::FieldCall { callee, key } => {
                if let Expression::Identifier { name } = *key {
                    order.push(FieldKey::String(name));
                } else {
                    order.push(FieldKey::String(hash_value(eval(&key, enclosing.clone(), overloadings)?)));
                }

                current = callee
            }
            Expression::Retrieve { retrievee, key } => {
                let key = eval(&key, enclosing.clone(), overloadings)?;

                match key.value {
                    ValueType::String(str) => {
                        let key = str;

                        order.push(FieldKey::String(key));
                    }
                    ValueType::Number(num) => {
                        let index = num as i32;

                        order.push(FieldKey::Int(index));
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
