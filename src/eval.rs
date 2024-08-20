use crate::{TokenType, LiteralType, Expression, Statement, 
    init_number_fields, init_array_fields, init_bool_fields, init_string_fields,
    stringify_value, hash_value};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::sync::Arc;
//eval statements



//this is a bit more complicated, Rc<Refcell<T>> provides us with the ability to mutate the entire
//environment object at will (its just some trickery so we can do that) terrible performance
//decision, but makes it work
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub values : RefCell<HashMap<String, Value>>,
    pub enclosing : Option<Rc<RefCell<Environment>>>,
}


//its easier to instantiate a get and set function that automatically search the entire env tree
//(for existence for example) to
//look for a value than doing that over and over again in the later following code
impl Environment {
    pub fn new(enclosing : Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>>{
       
        Rc::new(RefCell::new(Environment{
            values: RefCell::new(HashMap::new()),
            enclosing
        }))
    }



    pub fn get(&self, key : &str) -> Option<Value> {
       if let Some(value) = self.values.borrow().get(key) {
           Some(value.clone())
       } else if let Some(ref parent) = self.enclosing{
           parent.borrow().get(key)
       } else {
           None
       }
    }


    pub fn set(&self, key : String, value : Value){
        if self.values.borrow().contains_key(&key) {
            self.values.borrow_mut().insert(key, value);
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow().set(key, value)
        } else {
            panic!("no such variable {key} found")
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
pub fn eval_statement(stmts : Vec<Statement>, enclosing : Rc<RefCell<Environment>>) -> Value{
    
    

    

    for stmt in stmts {
        match stmt {

            Statement::Return { expression } => {
                let ret_value = eval(&expression.unwrap(), enclosing.clone());
    


                
                return ret_value
            }

            Statement::Function { name, body, arguments } => {
                
                let closure = Rc::new(RefCell::new(enclosing.borrow().clone()));

                let function = Value{
                    value_type : ValueType::THORFUNCTION,
                    string_value : Some(name.clone()),
                    function : Some(Function::ThorFunction {
                        body : *body.unwrap(),
                        needed_arguments : arguments.unwrap(),
                        closure

                    }),
                    ..Value::default()
                };
               

                enclosing.borrow_mut().values.borrow_mut().insert(name, function);


                 
            }
            //a block just opens a new env tree branch
            Statement::Block { statements } => {


                let local_scope = Environment::new(Some(enclosing.clone()));
                eval_statement(statements, local_scope.clone());
               
            },

            //if statements are one to one in the host language, makes it meta-programming...?
            Statement::If { condition, then_branch, else_branch } => {
                if eval(&condition.unwrap(),  enclosing.clone()).bool_value.expect("can only run if statements on bool values"){
                    return eval_statement(*then_branch.unwrap(), enclosing.clone())
                } else {

                    if let Some(else_block) = else_branch {
                        return eval_statement(*else_block, enclosing.clone());
                    }
                }
            },


            //still need to fix returns from while as i dont have a good way of detecing whether or
            //not something got returned....
            Statement::While { condition, block } => {

                let mut condition_true = 
                    eval(&condition.clone().unwrap(), enclosing.clone());
                while condition_true.bool_value.expect("while only accepts bool conditions"){

                    eval_statement(*block.clone().unwrap(), enclosing.clone());

                    condition_true = eval(&condition.clone().unwrap(), enclosing.clone())
                }
                
            },
            
            //print is built in
            Statement::Print { expression } => {
            
                let result = eval(&expression.unwrap(), enclosing.clone());
                
                println!("{}", stringify_value(result));

            },

            Statement::Do { expression } => {
                //runs expressions 
                eval(&expression.unwrap(), enclosing.clone());
            },

            //variable declaration only ever mutates the current branch of the env tree ensuring,
            //in this case "enclosing"
            Statement::Variable { name, expression } => {
                
                let val = eval(&expression.unwrap(), enclosing.clone());

                enclosing.borrow_mut().values.borrow_mut().insert(name, val);
            }

        }      

    
       

                  

    }

    Value::default()
}



//eval expressions

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ValueType {
    STRING, NUMBER, BOOL, NIL, NATIVEFUNCTION, THORFUNCTION, ARRAY
}

#[derive(Clone)]
pub enum Function {
    NativeFunction {body : Arc<dyn Fn(HashMap<String, Value>) -> Value>, needed_arguments : Vec<String>, self_value : Option<Box<Value>>}, 
    ThorFunction {body : Vec<Statement>, needed_arguments : Vec<String>, closure : Rc<RefCell<Environment>>}
}


//later i have to implement equality for functions
//since we cannot create a function that is "equal" in any sense to a native function it will only
//be the case for thorfunctions (equality on arguments and body, the env is not important when )
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {false}
    fn ne(&self, other: &Self) -> bool {true}
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{

            Function::NativeFunction { body, needed_arguments, self_value } => {
                f.debug_struct("Function")
                    .field("args", needed_arguments)
                    .finish()
            }, 
            Function::ThorFunction { body, needed_arguments, closure } => {
                f.debug_struct("Function")
                    .field("args", needed_arguments)
                    .finish()
            }
        }

    }
}




#[derive(PartialEq, Debug, Clone)]
pub struct Value {
    pub value_type : ValueType,
    pub string_value : Option<String>,
    pub number_value : Option<f64>,
    pub bool_value : Option<bool>, 
    pub function : Option<Function>,
    pub array : Vec<Value>,
    //methods always have access to "self" this means that just returning a normal native function
    //doesnt work, because we cannot change closures once passed in. So we use currying, makes
    //things a bit more complicated, but no user will ever see this.
    //fields will be accessible by things other than strings, however i cannot just hash f64s for
    //some reason and will need to implement own hashing algorithm to convert a Value to a string
    //and then this string will be put inside the hashmap, this makes things more complicated than
    //they need to be, but also easier than rewriting the entire typesystem
    pub fields : HashMap<String, Value>,
    pub is_nil : bool
}


//to print arrays and later objects nicely


impl Value {
    pub fn array(value : Vec<Value>) -> Value{

        Value {
            value_type : ValueType::ARRAY,
            array : (value),
            ..Value::default()
        }
    }

    pub fn number(value : f64) -> Value{
        Value{
            value_type : ValueType::NUMBER, 
            number_value : Some(value),
            ..Value::default()
        } 
    }

    pub fn string(value : String) -> Value{
        Value{
            value_type : ValueType::NUMBER, 
            string_value : Some(value),
            ..Value::default()
        } 
    }


    pub fn bool(value : bool) -> Value{
        Value{
            value_type : ValueType::NUMBER, 
            bool_value : Some(value),
            ..Value::default()
        } 
    }

    pub fn nil() -> Value {
        Value{
            is_nil : true,
            value_type : ValueType::NIL,
            ..Value::default()
        }
    }

    pub fn native_function(name : &str, arguments : Vec<&str>, body : Arc<dyn Fn(HashMap<String, Value>) -> Value>, self_value : Option<Box<Value>>) -> Value{
        Value{
            value_type:ValueType::NATIVEFUNCTION,
            string_value:Some(name.to_string()),
            function : Some(Function::NativeFunction{
                self_value,
                needed_arguments : arguments.iter().map(|x| x.to_string()).collect(),
                body,
            }),
            ..Value::default()
        }
    }

}




impl Default for Value {
    fn default() -> Value {
        Value {
            value_type:ValueType::NIL, 
            string_value:None,
            number_value:None,
            bool_value:None,
            function : None,
            array : vec![],
            fields : HashMap::new(),
            is_nil:false
        }
    }
}

//order of precedence is as follows 
// eval_statement -> eval -> eval_binary -> eval_unary -> eval_literal



fn eval_unary(operator : TokenType, right : &Expression, enclosing : Rc<RefCell<Environment>>)  -> Value{

    let r = eval(right, enclosing);      
    match operator {

        //only negate logically when is bool
        TokenType::BANG => {
            if r.value_type == ValueType::BOOL{
                return Value {
                    value_type : ValueType::BOOL, 
                    bool_value : Some(!r.bool_value.unwrap()), 
                    ..Value::default()
                }
            }  else {
                panic!("can only logically negate bools")
            }
        },
        //only negate arithmetically when is number
        TokenType::MINUS => {

            if r.value_type == ValueType::NUMBER {
                
                return Value{
                    value_type:ValueType::NUMBER,
                    number_value : Some(r.number_value.unwrap() * -1.0_f64),
                    ..Value::default()
                }

            } else {
                panic!("can only negate numbers")
            }
        },
        _ => {
            panic!("unary operation not allowed")
        }
    }

}


//evaluates the "atoms" these can not be further reduced and bubble up to form more complex data
//(not types but composed values like 1 + 2)
fn eval_literal (literal : LiteralType) -> Value{

        //turn literaltype into value wrapped in value_type
        match literal {
            LiteralType::NIL => {
                return Value::nil()
            }, 
            LiteralType::BOOL { value } => {
                return Value::bool(value)
            }, 
            LiteralType::NUMBER { value } => {
    
                return Value::number(value)                

            }, 
            LiteralType::STRING { value } => {
                return Value::string(value) 
            }

        } 

}


//just a helper to ensure that adding a string and a number throws
fn check_type_equality(value_1 : &Value, value_2 : &Value, expected_type : ValueType) -> bool{
    

    if value_1.value_type == value_2.value_type && value_1.value_type == expected_type {
        return true;
    } else {
        return false
    }

}


//all the possible binary operation combinations.
fn eval_binary(left : &Expression, operator : TokenType, right : &Expression, enclosing : Rc<RefCell<Environment>>) -> Value {

    let l = eval(left, enclosing.clone());
    let r = eval(right, enclosing);



    match operator {


        //almost every binary operation can only be applied to values of the same type 
        //check type equality, checks if two types are of the same type and if they are equal to
        //some expected_type numbers in the case of division for example
        //
        //if the two numbers are not equal and of the expected type the programm will throw
        TokenType::PLUS => {
             if check_type_equality(&l, &r, ValueType::STRING) {
                return Value{
                    value_type: ValueType::STRING,
                    string_value : Some(l.string_value.unwrap() + &r.string_value.unwrap()),
                    ..Value::default()
                } 
             }
             if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() + r.number_value.unwrap()),
                    ..Value::default()
                } 
             }
             panic!("can only add strings and numbers")
        }, 
        TokenType::MINUS => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() - r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only subtract numbers")
        }, 
        TokenType::STAR => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() * r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only multiply numbers")
        }, 
        TokenType::SLASH => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() / r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only divide numbers")
        }, 
        TokenType::LESSEQ => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() <= r.number_value.unwrap()),
                    ..Value::default()
                } 
             }


             panic!("can only compare numbers")
        },
        TokenType::LESS => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() < r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 
        TokenType::GREATEREQ => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() >= r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 
        TokenType::GREATER => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() > r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 

        //equality doesnt need a typecheck, if the Value object is the same, two values are the
        //same
        TokenType::EQEQ => {
            return Value{
                value_type: ValueType::BOOL, 
                bool_value : Some(l == r),
                ..Value::default()
            } 
        }, 
        TokenType::BANGEQ => {
            return Value{
                value_type: ValueType::BOOL, 
                bool_value : Some(l != r),
                ..Value::default()
            }
        }


        _ => ()
    }


    return Value::default()
}

pub fn eval(expr : &Expression, enclosing : Rc<RefCell<Environment>>) -> Value{

    //recursivley traverses the expr tree.
    match expr {
        Expression::Retrieve { retrievee, key } => {
            let key_value = eval(key, enclosing.clone());

            let retrievee_value = eval(retrievee, enclosing.clone());

            if key_value.value_type != ValueType::NUMBER{
                panic!("can only access arrays with numbers")
            }

            if key_value.number_value.unwrap().round() != key_value.number_value.unwrap(){
                panic!("can only access array with whole numbers")
            }

            let key_number = key_value.number_value.unwrap() as usize;

            if retrievee_value.value_type != ValueType::ARRAY{
                panic!("can only access arrays with brackets");
            }


        
            let return_value = retrievee_value.array
                .get(key_number).unwrap().clone();

            

            return_value   
        },

        Expression::FieldCall { callee, key } => {

            let callee_value = eval(callee, enclosing.clone());
            let key_string : String;


            if let Expression::Identifier { name } = *(*key).clone(){
                key_string = name;
            } else {
                key_string = eval(key, enclosing.clone()).string_value.unwrap_or_else(|| panic!("{:?} does not seem to be a string", key));
            }


            let mut ret_val = Value::default();
            if let Some(field) = callee_value.fields.get(&key_string){
               
                ret_val = field.clone();

            }
    

            match callee_value.value_type {

                ValueType::NUMBER => {
                    if let Some(field) = init_number_fields(callee_value.clone()).get(&key_string){
                        ret_val = field.clone();
                    }
                },
                ValueType::ARRAY => {
                    let mut var_name = "".to_string();
                    if let Expression::Identifier { name } = *callee.clone() {
                        var_name = name; 
                    }

                    if let Some(field) = init_array_fields(callee_value.clone(), enclosing.clone(), var_name).get(&key_string){
                        ret_val = field.clone();
                    }
                }
                ValueType::BOOL => {
                    if let Some(field) = init_bool_fields(callee_value.clone()).get(&key_string){
                        ret_val = field.clone();
                    }
                },
                ValueType::STRING => {
                    if let Some(field) = init_string_fields(callee_value.clone()).get(&key_string){
                        ret_val = field.clone();
                    }
                }
                _ => ()

            } 
            
             
            

            ret_val
        },

        Expression::Array { values } => {
           
            let mut value_array : Vec<Value> = vec![];

            let mut array = Value{
                value_type: ValueType::ARRAY,
                ..Value::default()
            };
                
            for value_expression in values{
                let value = eval(value_expression, enclosing.clone()); 

                value_array.push(value);
            }

            array.array = (value_array.clone());


            array
        },
        Expression::Call { callee, paren, arguments } => {
          

            //let eval_callee = eval(callee, enclosing.clone());
            
            let function_value = eval(callee, enclosing.clone());

            


            if let Function::NativeFunction { body, needed_arguments, self_value } = function_value.function.clone().unwrap() {


                    //check if arity of args is ok
                    if needed_arguments.len() != arguments.len() {
                        panic!("function {:?} requires {:?} arguments but got {:?}", 
                               function_value.string_value.unwrap(), 
                               needed_arguments.len(),
                               arguments.len())
                    }

                    let mut eval_args : HashMap<String, Value> = HashMap::new(); 

                    for i in 0..arguments.len() {

                        let arg = eval(arguments.get(i).unwrap(), enclosing.clone());
                        let arg_name = needed_arguments.get(i).unwrap();
                        
                        eval_args.insert(arg_name.to_string(), arg);
                    }

                    if let Some(sv) = self_value {
                        eval_args.insert("self".to_string(), *sv.clone());
                    }

                    let function_value = body(eval_args);
               

                    return function_value

            } 
                
            if let Function::ThorFunction { body, needed_arguments, closure } = function_value.function.clone().unwrap() {
                if needed_arguments.len() != arguments.len() {
                        panic!("Expected {} arguments but got {}", needed_arguments.len(), arguments.len());
                    }

                    // Evaluate the arguments in the current environment
                    let mut eval_args: HashMap<String, Value> = HashMap::new();
                    for i in 0..arguments.len() {
                        let arg = eval(arguments.get(i).unwrap(), enclosing.clone());
                        let arg_name = needed_arguments.get(i).unwrap();
                        eval_args.insert(arg_name.to_string(), arg);
                    }

                    // Create a new environment for the function call, using the closure's environment
                    let function_env = Environment::new(Some(closure.clone())); // Only capture the closure's environment
                    for (name, value) in eval_args {
                        function_env.borrow_mut().values.borrow_mut().insert(name, value);
                    }

                eval_statement(body, function_env)


            }


            else {
                panic!("error in function {:?}", function_value.string_value.unwrap())
            }

            
        },

        Expression::Assignment { target, value } => {
          
            let eval_value = eval(value, enclosing.clone());
         


            //iteratively go over the fields (creating them when they do not exist) and putting in
            //the value at the deepest level 
            //
            //iterating over order (a vector of keys, can be numbers for arrays or strings for
            //objects)
            let order = generate_field_order(target.clone(), enclosing.clone());
         

            let value : &mut Value = &mut enclosing
                .borrow()
                .get(&order.get(0).unwrap().get_string().unwrap().to_string())
                .unwrap().clone();

            let mut current : &mut Value = value;


            for i in 1..(order.len() - 1) {

                //cannot be out of bounds
                match order.get(i).unwrap(){


                    FieldKey::String(string) => {

                        if let Some(_field) = current.fields.get(string){
                            

                            current = current.fields.get_mut(string).unwrap();
                        } else {
                            // we can be sure that i is in bounds and this is a string
                            current.fields.insert(order.get(i).unwrap().get_string().unwrap(), Value::default());
                        }

                    },
                    _ => unimplemented!() 
                }                        

            }


            let last_key = order.get(order.len() - 1).unwrap(); 

            match last_key{
                FieldKey::String(key) => {
                    current.fields.insert(key.to_string(), eval_value.clone());
                },
                FieldKey::Int(num) => {
                    current.array[*num as usize] = eval_value.clone();
                }

            }
            

            enclosing.borrow_mut().set(order.get(0).unwrap().get_string().unwrap().to_string(), value.clone());

            return eval_value
        },

        //kind of like literals, but will replace instantly with the value behind the variable name
        //instead of going down to literals first
        Expression::Identifier { name } => {
            

            let value = enclosing.borrow().get(name).expect(&("this value does not exist ".to_string() + name));

            return value.clone()
        },
        Expression::Unary { operator, right } => {
            return eval_unary(*operator, &right, enclosing)
        }, 
        Expression::Literal { literal } => {
            return eval_literal(literal.clone())
        }, 
        Expression::Grouping { inner } => {
            return eval(&inner, enclosing) 
        }, 
        Expression::Binary { left, operator, right } => {
            return eval_binary(&left, *operator, &right, enclosing)
        }
    } 

}

#[derive(Debug)]
enum FieldKey {
    Int(i32),
    String(String)
}

//methods to get the strings and numbers easier without doing if lets all the time
impl FieldKey {
    fn get_num(&self) -> Option<i32>{
        return match self{
            FieldKey::Int(num) => Some(*num),
            FieldKey::String(string) => None
        }
    }

    fn get_string(&self) -> Option<String>{
         return match self{
            FieldKey::Int(num) => None,
            FieldKey::String(string) => Some(string.to_string())
        }
    }
}

fn generate_field_order(target : Box<Expression>, enclosing : Rc<RefCell<Environment>>) -> Vec<FieldKey>{
    let mut order = vec![];

    let mut current = target;

    let mut not_ended = true;

    while not_ended {
        match *current.clone() {
            Expression::Identifier { name } => {
                order.push(FieldKey::String(name));
                not_ended = false;
            },
            Expression::FieldCall { callee, key } => {
                let mut key_name = "".to_string();
                if let Expression::Identifier { name } = *key{
                    key_name = name; 
                } else {
                    key_name = hash_value(eval(&key, enclosing.clone()));
                }

                order.push(FieldKey::String(key_name));

                current = callee
            },
            Expression::Retrieve { retrievee, key } => {
                let index = eval(&key, enclosing.clone()).number_value.unwrap() as i32;

                order.push(FieldKey::Int(index));

                current = retrievee;
            }
            _ => ()
        }

    }
   
    order.reverse();
    order
}

//handles assignments to arrays and objects
fn handle_field_assignment(target : Box<Expression>, value : Box<Expression>, enclosing : Rc<RefCell<Environment>>) -> Value{

    if let Expression::Identifier { name } = *target.clone(){
        
        return enclosing.borrow_mut().get(&name).unwrap();

    }

    


    Value::default()
}
