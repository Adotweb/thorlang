use crate::{TokenType, LiteralType, Expression, Statement};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
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



    fn get(&self, key : &str) -> Option<Value> {
       if let Some(value) = self.values.borrow().get(key) {
           Some(value.clone())
       } else if let Some(ref parent) = self.enclosing{
           parent.borrow().get(key)
       } else {
           None
       }
    }


    fn set(&self, key : String, value : Value){
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


                let function = Value{
                    value_type : ValueType::THORFUNCTION,
                    string_value : Some(name.clone()),
                    function : Some(Function::ThorFunction {
                        body : *body.unwrap(),
                        needed_arguments : arguments.unwrap(),
                        closure : enclosing.clone()

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

                match result.value_type {
                    ValueType::NATIVEFUNCTION => {
                        println!("function : {:#?} with body {:#?}", result.string_value.unwrap(), result.function.unwrap())
                    },
                    ValueType::THORFUNCTION => {

                        println!("function : {:#?} with body {:#?}", result.string_value.unwrap(), result.function.unwrap())
                    }
                    ValueType::STRING => {
                        println!("{:#?}", result.string_value.unwrap())
                    },
                    ValueType::NUMBER => {
                        println!("{:#?}", result.number_value.unwrap())
                    },
                    ValueType::BOOL => {
                        println!("{:#?}", result.bool_value.unwrap())
                    },
                    ValueType::NIL => {
                        println!("{:#?}", ValueType::NIL)
                    },

                }

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
    STRING, NUMBER, BOOL, NIL, NATIVEFUNCTION, THORFUNCTION
}


//still thinking about whether or not i should implement functions as closures (in rust) or as
//functions that are defined as code blocks
//
//probably both (closures for native functions, together with a enclosing param so we can use )
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NativeFunction { 
    pub body : fn(HashMap<String, Value>) -> Value,
    pub arguments : Vec<String>
}

#[derive(PartialEq, Debug, Clone)]
pub struct ThorFunction {
    pub body : Vec<Statement>, 
    pub arguments : Vec<String>, 
    pub closure : Rc<RefCell<Environment>>
}

#[derive(PartialEq, Debug, Clone)]
pub enum Function {
    NativeFunction {body : fn(HashMap<String, Value>) -> Value, needed_arguments : Vec<String>}, 
    ThorFunction {body : Vec<Statement>, needed_arguments : Vec<String>, closure : Rc<RefCell<Environment>>}
}

#[derive(PartialEq, Debug, Clone)]
pub struct Value {
    pub value_type : ValueType,
    pub string_value : Option<String>,
    pub number_value : Option<f64>,
    pub bool_value : Option<bool>, 
    pub function : Option<Function>,
    pub is_nil : bool
}



impl Value {
    pub fn native_function(name : &str, arguments : Vec<&str>, body : fn(HashMap<String, Value>) -> Value) -> Value{
        Value{
            value_type:ValueType::NATIVEFUNCTION,
            string_value:Some(name.to_string()),
            function : Some(Function::NativeFunction{
                needed_arguments : arguments.iter().map(|x| x.to_string()).collect(),
                body
            }),
            ..Value::default()
        }
    }

    pub fn thor_function(name : &str, arguments : Vec<&str>, body : Vec<Statement>, closure : Rc<RefCell<Environment>>) -> Value {
        Value{
            value_type : ValueType::THORFUNCTION,
            string_value : Some(name.to_string()),
            function : Some(Function::ThorFunction{
                needed_arguments : arguments.iter().map(|x| x.to_string()).collect(),
                body,
                closure
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
                return Value{
                    value_type : ValueType::NIL,
                    is_nil:true, ..Value::default()
                }
            }, 
            LiteralType::BOOL { value } => {
                return Value{
                    value_type : ValueType::BOOL,
                    is_nil:false, bool_value : Some(value), ..Value::default()
                }
            }, 
            LiteralType::NUMBER { value } => {
                return Value{
                    value_type : ValueType::NUMBER,
                    is_nil:false, number_value : Some(value), ..Value::default()
                }
            }, 
            LiteralType::STRING { value } => {
                return Value{
                    value_type : ValueType::STRING,
                    is_nil : false, string_value : Some(value), ..Value::default()
                }
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

        Expression::Call { callee, paren, arguments } => {
          

            //let eval_callee = eval(callee, enclosing.clone());

            let function_value = enclosing
                .borrow()
                .get(&eval(callee, enclosing.clone()).string_value.unwrap())
                .expect("function not found");


            if let Function::NativeFunction { body, needed_arguments } = function_value.function.clone().unwrap() {


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

        Expression::Assignment { name, value } => {
          
            let eval_value = eval(value, enclosing.clone());
         
            //assignment is the only thing that can change variables in higher scope, but will
            //always find the closest variable with this name
            enclosing.borrow_mut().set(name.to_string(), eval_value.clone());

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
