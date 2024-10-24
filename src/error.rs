use std::result::Result;
use crate::{Statement, Token, TokenType, Value, stringify_value, ValueType};
use std::ops::Range;

   

//handles error when parsing (unexpected tokens and typos)
pub fn handle_error(text : String, tokens : Vec<Token>, error : ThorLangError){

    let text_lines : Vec<&str> = text.split("\n").collect();
    
    //whenever possible thorlang will try to aid you to improve your code with a message and
    //pointing where the error occured

    let mut msg : String = Default::default();
    let mut error_line : String = Default::default();
    let mut pointer : &str = "";
    let mut tip : String = Default::default();

    match error {
        
        
        //handling of UnexexpectedTokenError
        ThorLangError::UnexpectedToken { expected, encountered } => {
            let encountered_token = tokens[encountered - 1].clone();
            let next_token = tokens[encountered].clone();
           
            
            let expected_tokens = if(expected.len() > 1){
                "one of".to_string() + &format!("{:?}", expected).to_owned()
            }else {
                format!("{:?}", expected[0]) 
            };

            

            msg = format!("expected {:?} after {:?} on line {:?}:{:?},\nfound {:?} instead", 
                expected_tokens,
                encountered_token.token_type, 
                encountered_token.line,
                encountered_token.column,
                next_token.token_type
                );


            error_line = format!("{} | {}", encountered_token.line, text_lines[encountered_token.line as usize - 1])
        },
        ThorLangError::IndexError { index_number_token_index, array_value, tried_index } => {
            let number_token = tokens[index_number_token_index].clone();
            let array_token = tokens[index_number_token_index - 1].clone();


            if let ValueType::Array(arr) = array_value.value{
                let possible_length = arr.len();
               

                msg = format!("array '{}' on line {}:{} only has length {}.\naccessing its {}. ({} + 1) element is not possible", 
                    array_token.token_type.get_content().unwrap(),
                    array_token.line, 
                    array_token.column,
                    possible_length, 
                    tried_index + 1.0, 
                    tried_index);

                error_line = format!("{} | {}", array_token.line, text_lines[array_token.line as usize - 1])
                    
            }
        },
        ThorLangError::FunctionArityError { function_paren_token, needed_arguments_length, arguments_length } => {


            let paren_token = tokens[function_paren_token].clone();

            //-2 because i registered the RPAREN 
            let function_name_token = tokens[function_paren_token - 2].clone(); 

            msg = format!("function '{}' on line {}:{}\nexpects {} arguments but got {}", 
                    function_name_token.token_type.get_content().unwrap(),
                    paren_token.line, 
                    paren_token.column,
                    needed_arguments_length, 
                    arguments_length
                );

            error_line = format!("{} | {}", paren_token.line, text_lines[paren_token.line as usize - 1]);


        }, 
        ThorLangError::ThorLangException { exception, throw_token_index } => {
            let throw_token = tokens[throw_token_index].clone();

            msg = format!("the user has decided to throw {:?} on line {}:{} using the throw token", 
                stringify_value(*exception),
                throw_token.line, throw_token.column);



            error_line = format!("{} | {}", 
                throw_token.line,
                text_lines[throw_token.line as usize - 1]);

        },
        ThorLangError::RetrievalError { retrieve_seperator_token_index } => {
            let seperator_token = tokens[retrieve_seperator_token_index].clone();

            let object_token = tokens[retrieve_seperator_token_index - 1].clone();
            
            let key_token = tokens[retrieve_seperator_token_index + 1].clone();
             

            msg = format!("the object {} on line {}:{} doesn't have a field {}", 
                object_token.token_type.get_content().unwrap(), 
                seperator_token.line,
                seperator_token.column,
                key_token.token_type.get_content().unwrap());

            error_line = format!("{} | {}", 
                seperator_token.line,
                text_lines[seperator_token.line as usize - 1]);



        }

        _ => ()

    }

    println!("\n{msg}\n"); 
    println!("{error_line}\n");
    //println!("{tip}");
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

    pub fn function_arity_error(function_paren_token : usize, needed_arguments_length : usize, arguments_length : usize) -> Result<Value, ThorLangError>{

        Err(ThorLangError::FunctionArityError{
            function_paren_token,
            needed_arguments_length, 
            arguments_length
        })

    }

    pub fn unkown_function_error(function_paren_token : usize) -> Result<Value, ThorLangError>{
        
        Err(ThorLangError::UnknownFunctionError{
            function_paren_token
        })
    }

    pub fn unknown_value_error(identifier_token_index : usize) -> Result<Value, ThorLangError>{

        Err(ThorLangError::UnknownValueError{
            identifier_token_index
        })

    }
}


//returns the type of token that is wrong or that was expected
pub fn stringify_token_type(token_type : TokenType) -> &'static str{
   
    if let TokenType::IDENTIFIER(str) = token_type {
        return "identifier"
    }

    if let TokenType::NUMBER(str) = token_type {
        return "number"
    }

    if let TokenType::STRING(str) = token_type {
        return "string"
    }

    if let TokenType::SPECIAL(str) = token_type {
        return "special character"

    }
    let string : &str = match token_type {
            TokenType::LPAREN => "(",//left parenthesis : (
            TokenType::RPAREN => ")",//right parenthesis : )
            TokenType::LBRACK => "[",//left bracket : [
            TokenType::RBRACK => "]",//right bracket : ]
            TokenType::LBRACE => "{",//left brace : {
            TokenType::RBRACE => "}",//right brace : }
            TokenType::COMMA => "comma",
            TokenType::DOT => ".",
            TokenType::MINUS => "operation - (minus)",
            TokenType::PLUS => "operation + (plus)",
            TokenType::SEMICOLON => "semicolon ;",
            TokenType::SLASH => "operation / (slash)",
            TokenType::STAR => "operation * (star)",
            TokenType::BANG => "operation ! (bang)",
            TokenType::BANGEQ => "comparison != (not equal)",
            TokenType::EQ => "assignment = (assign)",
            TokenType::EQEQ => "comparison == (equal)",
            TokenType::GREATER => "comparison > (greater than)",
            TokenType::GREATEREQ => "comparison >= (greater or equal)",
            TokenType::LESS => "comparison < (less than)",
            TokenType::LESSEQ => "comparison <= (less or equal)",

            TokenType::TRY => "try",
            TokenType::OVERLOAD => "overload",
            TokenType::DO => "do",
            TokenType::ELSE => "else",
            TokenType::FALSE => "false",
            TokenType::FN => "fn",
            TokenType::IF => "if",
            TokenType::NIL => "nil",
            TokenType::PRINT => "print",
            TokenType::RETURN => "return",
            TokenType::TRUE => "true",
            TokenType::LET => "let",
            TokenType::WHILE => "while",
            TokenType::THROW => "throw",

            TokenType::EOF => "eof",
            _ => "unknown token"
    };

    return string
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

    EvalError(String),
}


