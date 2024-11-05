use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use std::rc::Rc;

use std::fmt;

use libloading::Library;

//the different token types
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum TokenType {
    LPAREN,//left parenthesis : (
    RPAREN,//right parenthesis : )
    LBRACK,//left bracket : [
    RBRACK,//right bracket : ]
    LBRACE,//left brace : {
    RBRACE,//right brace : }
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    BANG,
    BANGEQ,
    EQ,
    EQEQ,
    GREATER,
    GREATEREQ,
    LESS,
    LESSEQ,

    IDENTIFIER(String),
    STRING(String),
    NUMBER(String),

    SPECIAL(String),

    TRY,
    OVERLOAD,
    DO,
    AND,
    ELSE,
    FALSE,
    FN,
    IF,
    NIL,
    PRINT,
    RETURN,
    TRUE,
    LET,
    WHILE,
    THROW,

    EOF,
}

//methods to get the string values out of the tokentypes without having to make if lets all the
//time
impl TokenType {
    pub fn get_content(&self) -> Option<String>{

        match &self{
            TokenType::NUMBER(num) => Some(num.to_string()),
            TokenType::STRING(str) => Some(str.to_string()),
            TokenType::IDENTIFIER(id) => Some(id.to_string()),
            _ => None
        }

    }
}

//since the num str and identifier enums can all hold data we dont have the need for a literaltype
//anymore
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
    pub column : i32
}

//statements

//the different kinds of statments and what data they hold
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Throw{
        exception : Expression,
        throw_token_index : usize
    },
    Print {
        expression: Expression,
        line : i32
    },
    Do {
        expression: Expression,
        line : i32
    },
    Variable {
        name: String,
        expression: Expression,
        line : i32
    },
    Block {
        statements: Vec<Statement>,
        line : i32
    },
    If {
        condition: Expression,
        then_branch: Box<Vec<Statement>>,
        else_branch: Option<Box<Vec<Statement>>>,
        line : i32
    },
    While {
        condition: Expression,
        block: Box<Vec<Statement>>,
        line : i32
    },
    Function {
        name: String,
        body: Box<Vec<Statement>>,
        arguments: Vec<String>,
        line : i32
    },
    Return {
        expression: Expression,
        line : i32
    },
    Overload {
        operator : TokenType, 
        operands : Vec<String>,
        operation : Vec<Statement>,
        line : i32
    }
}


//unlike parser errors we know that the tokenlist works in here and we can point to the token that
//has an error 
//this means it sufficces to just put in the index to the wanted token
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Try {
        block : Vec<Statement>
    },
    Identifier {
        name: String,
        identifier_token_index : usize
    },
    Binary {
        left: Box<Expression>,
        operator: TokenType,
        right: Box<Expression>,
        operator_token_index : usize
    },
    Unary {
        operator: TokenType,
        right: Box<Expression>,
        operator_token_index : usize
    },
    Grouping {
        inner: Box<Expression>,
    },
    Literal {
        literal: TokenType,
        literal_token_index : usize
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
        eq_token_index : usize
    },
    Array {
        values: Vec<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren_token_index: usize,
        arguments: Vec<Expression>,
    },
    Retrieve {
        retrievee: Box<Expression>,
        key: Box<Expression>,
        lbrack_token_index : usize
    },
    FieldCall {
        callee: Box<Expression>,
        key: Box<Expression>,
        dot_token_index : usize
    },
}


//this is a bit more complicated, Rc<Refcell<T>> provides us with the ability to mutate the entire
//environment object at will (its just some trickery so we can do that) terrible performance
//decision, but makes it work
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, Value>>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

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



//Functions are either built into rust (rust closures) or defined as a procedure in thor itself
//both do the same but have different data to them
#[derive(Clone)]
pub enum Function{
    //libfunctions work by referencing the name of the function inside of the library to prevent
    //overcrossing
    LibFunction{
        name : &'static str,
        needed_arguments : Vec<String>,
        library : Option<Arc<Library>>,
        //only needs self value in case of method
        self_value : Option<Box<Value>>
    },
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
            Function::LibFunction { name, needed_arguments, library, self_value } => {
                f.debug_struct("Function")
                    .field("name", name)
                    .finish()
            },
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


    pub fn lib_function(
        name : &'static str,
        needed_arguments : Vec<String>,
        library : Option<Arc<Library>>,
        self_value: Option<Box<Value>>
    ) -> Value{
        Value{
            value : ValueType::Function(Function::LibFunction{
                name , 
                needed_arguments, 
                library,
                self_value
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



#[derive(Clone, Debug, PartialEq)]
pub enum ThorLangError{
    UnexpectedToken{
        expected : Vec<TokenType>,
        encountered : usize 
    },
    //evaluation errors : 
    
    IndexError{
        index_number_token_index : usize,
        array_value: Box<Value>,
        tried_index : f64
    },
    
    RetrievalError{
        //the token of either "[" or "."
        retrieve_seperator_token_index : usize
    },
    FunctionArityError{
        function_paren_token : usize,
        needed_arguments_length : usize,
        arguments_length : usize
    },
    OperationArityError{
        operator_token_index : usize,
        expected_arguments : usize,
        provided_arguments : usize
    },
    UnknownFunctionError{
        function_paren_token : usize
    },
    UnknownValueError{
        identifier_token_index : usize
    },
    
    //can be used to throw userside
    ThorLangException{
        exception :  Box<Value>,
        throw_token_index : usize 
    }, 
    EvalError{
        operation_token_index : usize
    },

    UnknownError
}


//easier methods to return nice errors
impl ThorLangError {

    //errors have form:
    //expected ... after ..., encountered ...
    pub fn unexpected_token<T>(expected : TokenType, encountered_index : usize) -> Result<T, ThorLangError>{
        Err(ThorLangError::UnexpectedToken{
            expected : vec![expected],
            encountered : encountered_index
        })
    }
    pub fn unexpected_token_of_many<T>(expected : Vec<TokenType>, encountered_index : usize) -> Result<T, ThorLangError>{
         
         Err(ThorLangError::UnexpectedToken{
            expected,
            encountered : encountered_index
        })                  
    }

    //object and field can be retrieved from the tokens surrounding the dot or the lbrack
    pub fn retrieval_error(retrieve_seperator_token_index : usize) -> Result<Value, ThorLangError>{
        Err(ThorLangError::RetrievalError{
            retrieve_seperator_token_index
        })
    }


    //index is out of bounds or a non natural number
    pub fn index_error(index_number_token_index : usize, array_value : Value, tried_index : f64) -> Result<Value, ThorLangError>{

        Err(ThorLangError::IndexError{
            index_number_token_index,
            array_value : Box::new(array_value),
            tried_index
        })

    }

    //function "..." expeceted n arguments but got m
    pub fn function_arity_error(function_paren_token : usize, needed_arguments_length : usize, arguments_length : usize) -> Result<Value, ThorLangError>{

        Err(ThorLangError::FunctionArityError{
            function_paren_token,
            needed_arguments_length, 
            arguments_length
        })

    }

    //this almost never happens (really almost always and only if we use special characters)
    //operator "." expected n aruments but got m
    pub fn operation_arity_error(operator_token_index : usize, expected_arguments : usize,provided_arguments : usize) -> Result<Value, ThorLangError>{
        Err(ThorLangError::OperationArityError{
            operator_token_index,
            expected_arguments, 
            provided_arguments
        })
    }


    //the function "..." could not be found in the current scope
    pub fn unkown_function_error(function_paren_token : usize) -> Result<Value, ThorLangError>{
        
        Err(ThorLangError::UnknownFunctionError{
            function_paren_token
        })
    }

    //the value "..." could not be found in the current scope
    pub fn unknown_value_error(identifier_token_index : usize) -> Result<Value, ThorLangError>{

        Err(ThorLangError::UnknownValueError{
            identifier_token_index
        })

    }


    //the operation "." is defined for values of type "..." and "..."
    pub fn eval_error(operation_token_index : usize) -> Result<Value, ThorLangError>{
        Err(ThorLangError::EvalError{
            operation_token_index
        })
    }
}
