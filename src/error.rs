use type_lib::{ TokenType, Token, ValueType, ThorLangError};

use crate::stringify_value;


   

//handles error when parsing (unexpected tokens and typos)
pub fn handle_error(text : String, tokens : Vec<Token>, error : ThorLangError){

    let text_lines : Vec<&str> = text.split("\n").collect();
    
    //whenever possible thorlang will try to aid you to improve your code with a message and
    //pointing where the error occured

    let mut msg : String = Default::default();
    let mut error_line : String = Default::default();
    let mut pointer : &str = "";
    let mut tip : String = Default::default();

    //almost all of the below work in the same way, the token_indices are retrieved from the token
    //list and then their value is displayed in the error message, then the entire line of the
    //error is displayed
    match error {
        ThorLangError::UnknownFunctionError { function_paren_token } => {
            
            let function_paren_token = tokens[function_paren_token - 1].clone();


            msg = format!("the function '{}' on line {}:{}, is not found in the current scope", 
                function_paren_token.token_type.get_content().unwrap(),
                function_paren_token.line,
                function_paren_token.column
                );



            error_line = format!("{} | {}", function_paren_token.line, text_lines[function_paren_token.line as usize - 1])
        },
        ThorLangError::UnknownValueError { identifier_token_index } => {
            
            let unknown_value_token = tokens[identifier_token_index - 1].clone();


            msg = format!("the value '{}' on line {}:{},  is not found in the current scope", 
                unknown_value_token.token_type.get_content().unwrap(),
                unknown_value_token.line,
                unknown_value_token.column
                );



            error_line = format!("{} | {}", unknown_value_token.line, text_lines[unknown_value_token.line as usize - 1])
        },    
        
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
            //also every given argument also needs to be counted
            //and if there is more than one then we need to also count the commas
            //
            //
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
        ThorLangError::OperationArityError { operator_token_index, expected_arguments, provided_arguments } => {
            let op_token = tokens[operator_token_index - 1].clone();

 
            msg = format!("the operator '{}' on line {}:{}\nexpects {} arguments but got {}", 
                stringify_token_type(op_token.token_type),
                op_token.line, 
                op_token.column, 
                expected_arguments, 
                provided_arguments
                );

            error_line = format!("{} | {}", op_token.line, text_lines[op_token.line as usize - 1]);
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



        },
        ThorLangError::EvalError { operation_token_index } => {
           
            let left_op = tokens[operation_token_index - 1].clone();
            let right_op = tokens[operation_token_index + 1].clone();
            let operation_token = tokens[operation_token_index].clone();


            let left_type = stringify_token_type(left_op.token_type);
            let right_type = stringify_token_type(right_op.token_type);

            let op_type = stringify_token_type(operation_token.token_type);

            msg = format!("the operation {} on line {}:{}, cannot be performed on types {:?} and {:?} \noverloading might help", 
                op_type,
                operation_token.line,
                operation_token.column,
                left_type, 
                right_type
                );

             
            error_line = format!("{} | {}", operation_token.line, text_lines[operation_token.line as usize - 1]);

        },

        _ => println!("{:?}", error)

    }

    println!("\n{msg}\n"); 
    println!("{error_line}\n");
    //println!("{tip}");
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
            TokenType::MINUS => "- (minus)",
            TokenType::PLUS => "+ (plus)",
            TokenType::SEMICOLON => "semicolon ;",
            TokenType::SLASH => "/ (slash)",
            TokenType::STAR => "* (star)",
            TokenType::BANG => "! (bang)",
            TokenType::BANGEQ => "!= (not equal)",
            TokenType::EQ => "= (assign)",
            TokenType::EQEQ => "== (equal)",
            TokenType::GREATER => "> (greater than)",
            TokenType::GREATEREQ => ">= (greater or equal)",
            TokenType::LESS => "< (less than)",
            TokenType::LESSEQ => "<= (less or equal)",

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



