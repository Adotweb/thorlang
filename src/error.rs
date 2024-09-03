use std::result::Result;
use crate::{Statement, Token, TokenType};

fn hamming_distance(spelled_wrong : String, spelled_right : String) -> usize{
   

    //function that figures out how many characters are wrong
    //
    //wrong_word = "applf"
    //right_word = "apple"
    //
    //creates list like this : [(a, a), (p, p), (p, p), (l, l), (f, e)] 
    //-> [false, false, false, false, true] -> 1
    spelled_wrong.chars().zip(spelled_right.chars()).filter(|(c1, c2)|{

            c1 != c2 

    }).count()

}
   
pub fn typo_check(token : Token) -> Option<ThorLangError>{
      
    let mut stringified : String = "".to_string();
    if let TokenType::IDENTIFIER(id) = token.clone().token_type{
        stringified = id; 
    }


    let word_tokens = vec![
        "try",
        "overload",
        "if",
        "else",
        "fn",
        "return", 
        "throw",
        "let",
        "print",
        "do",

        //these three can be used in expressions later
        "false",
        "true",
        "nil",
    ];


    let mut filtered : Vec<(usize, String)>  = word_tokens.iter()
        .map(|x|{
            

                let ham_dist = hamming_distance(x.to_string(), stringified.to_string());
                (ham_dist, x.to_string().to_string())
        }).collect();

    filtered.sort_by(|a,b| a.0.cmp(&b.0));
  
    println!("{:?}", token);
    
    Some(ThorLangError::TypoError{
        line : token.line, 
        column : token.column,
        got : stringified,
        might_be : filtered[0].1.clone()
    })
}

pub fn handle_error(error : ThorLangError, text : String){

    let text_lines : Vec<&str> = text.split("\n").collect();
    
    //whenever possible thorlang will try to aid you to improve your code with a message and
    //pointing where the error occured

    let mut msg : String = Default::default();
    let mut error_line : String = Default::default();
    let mut underline : Vec<String> = vec![];
    let mut tip : String = Default::default();

    match error {
        ThorLangError::TypoError { got, might_be, line, column } => {

            msg = format!("you wrote {got} on line {line}:{column}, did you mean to write {might_be}? \n");



            error_line = line.to_string() + "| " + text_lines[line as usize - 1];
            error_line = error_line.replace("\t", " ");
            underline = ("_".repeat(error_line.len()*2) + "\n")
                .chars()
                .map(|x| x.to_string())
                .collect();

            
       
            
            for i in column+2..column +2+ (got.len() as i32){
                 
                underline[i as usize] = "^".to_string();
            }


            if column - 1 < 0 {
                error_line = line.to_string() + "| " + text_lines[line as usize - 2];
                error_line = error_line.replace("\t", " ");
                underline = ("_".repeat(error_line.len()*2) + "\n")
                    .chars()
                    .map(|x| x.to_string())
                    .collect();
                
                underline[error_line.len()] = "^".to_string();

            }

            let tip_start = " ".repeat(column as usize + 3);

            tip = tip_start + "replace this with " + &might_be
        },
        
        //handling of UnexexpectedTokenError
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

//easier methods to return nice errors
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
        got : Token, 
        line : i32,
        column : i32
    },
    TypoError{
        got : String,
        might_be : String,
        line : i32,
        column : i32
    },

    EvalError(String),
}
