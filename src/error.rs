use std::result::Result;
use crate::{Statement, Token, TokenType};

pub enum ErrorType {

}
    
pub fn handle_error(error : ThorLangError, text : String){

    let text_lines : Vec<&str> = text.split("\n").collect();
  
    let mut msg : String = Default::default();
    let mut error_line : String = Default::default();
    let mut underline : Vec<String> = vec![];
    let mut tip : String = Default::default();

    match error {

        ThorLangError::UnexpectedToken { expected, got, line, column } => {
            
            let expected = if expected.len() > 1 {
                    let string = expected.iter().map(|x| stringify_token_type(x.clone()).to_string()).reduce(|prev, curr| prev + &curr).unwrap();
                    string
                } else {
                    stringify_token_type(expected[0].clone()).to_string()
                };

            msg = format!("expected {} on line {}:{}, got {} instead \n",
                expected,
                line,
                column, 
                stringify_token_type(got.token_type)
            ); 

            error_line = line.to_string() + "| " + text_lines[line as usize - 1];
            underline = ("_".repeat(error_line.len()*2) + "\n")
                .chars()
                .map(|x| x.to_string())
                .collect();

            
            underline[got.column as usize + 1] = "^".to_string();
            
            if got.column - 1 < 0 {
                error_line = line.to_string() + "| " + text_lines[line as usize - 2];
                underline = ("_".repeat(error_line.len()*2) + "\n")
                    .chars()
                    .map(|x| x.to_string())
                    .collect();
                
                underline[error_line.len()] = "^".to_string();

            }

            let tip_start = " ".repeat(column as usize + 3);
            tip = tip_start + "try to insert a "+ &expected + " here";
            

        },
        _ => ()

    }

    println!("{msg}");
    println!("{error_line}");
    println!("{}", underline.join(""));
    println!("{tip}");
}


impl ThorLangError {

    //errors have form:
    //expected ... after ..., encountered ...
    pub fn unexpected_token<T>(expected : TokenType, encountered : Token, after : Token) -> Result<T, ThorLangError>{
        Err(ThorLangError::UnexpectedToken{
            expected : vec![expected],
            got : encountered.clone(),
            column : encountered.column,
            line : encountered.line
        })
    }

    pub fn unexpected_token_of_many<T>(expected : Vec<TokenType>, encountered : Token, after : Token) -> Result<T, ThorLangError>{
         
         Err(ThorLangError::UnexpectedToken{
            expected,
            got : encountered.clone(),
            column : after.column,
            line : after.line
        })                  
    }
}

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
        got : Token, 
        line : i32,
        column : i32
    },

    EvalError(String),
}
