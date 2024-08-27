use regex::Regex;

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

    AMP,//ampersand : &
    UP, //up arror : ^
    QMARK, //question mark : ?
    PERCENT, //percent symbol : %

    EOF,
}


//since the num str and identifier enums can all hold data we dont have the need for a literaltype
//anymore
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
    pub column : i32
}

fn simple_token(token_type: TokenType, line: i32, column : i32) -> Token {
    return Token {
        token_type,
        line,
        column
    };
}

//returns the next character in the text
fn peek(current_index: usize, text: &str) -> String {
    //returns char at iter + 1

    return text.chars().nth(current_index + 1).unwrap().to_string();
}


//just a wrapper type, unelegant and will be overworked in the future
struct OuterIter {
    token: Token,
    iter_skip_steps: usize,
}

fn iterate_string(current_index: usize, text: &str, current_line: i32, column : i32) -> OuterIter {

    //iterate string skips at least one (the first matching) character
    let mut iter_skip_steps: usize = 1;

    let mut string = "".to_string();

    while let Some(char) = text.chars().nth(current_index + iter_skip_steps){
        let char = char.to_string();
    
        //while there is some character that is not a " the string goes on and on
        if char != "\""{
            string += &char;
            iter_skip_steps += 1;
        } else {
            iter_skip_steps += 1;
            break;
        }
        
    }

    return OuterIter {
        token: Token {
            token_type: TokenType::STRING(string),
            line: current_line,
            column
        },
        iter_skip_steps,
    };
}

fn iterate_number(current_index: usize, text: &str, current_line: i32, column : i32) -> OuterIter {

    //same as before
    let mut iter_skip_steps = 0;
    let mut number = "".to_string();
    let number_regex = Regex::new(r"[0-9]").unwrap();

    while let Some(char) = text.chars().nth(current_index + iter_skip_steps){
        let char = char.to_string();

        if number_regex.is_match(&char){
            number += &char;
            iter_skip_steps += 1;
        } 
        else if char == ".".to_string(){
            let next_char = peek(current_index, text);
            if number_regex.is_match(&next_char){
                number += &char;
                iter_skip_steps += 1;
            } else {
                break
            }
        } 
        else {
            break;
        } 
    }


    //return the currently accumulated number
    return OuterIter {
        token: Token {
            token_type: TokenType::NUMBER(number),
            line: current_line,
            column
        },
        iter_skip_steps,
    };
}

fn iterate_identifier(current_index: usize, text: &str, current_line: i32, column : i32) -> OuterIter {
    //identifiers have the following regular form : [a-zA-Z]([a-zA-Z0-9]|_)*
    //this method only has to check from the second letter onwards (till the end of the "word").
    let id_regex = Regex::new(r"[a-zA-Z0-9]|_").unwrap();
    let mut iter_skip_steps: usize = 0; //counts how many iterations have to be skipped in the main iteration loop

    let mut identifier = "".to_string();

    while let Some(char) = text.chars().nth(current_index + iter_skip_steps){
        let char = char.to_string();

        if id_regex.is_match(&char){
            iter_skip_steps += 1;
            identifier += &char;
        } else {
            break;
        }
         
    }
    //makes default token_type usually this will be returned
    let mut token_type = TokenType::IDENTIFIER(identifier.clone());
    //check if identifier is part of keywords and if it is change the tokentype
    match identifier.as_str() {
        "try" => token_type = TokenType::TRY,
        "overload" => token_type = TokenType::OVERLOAD,
        "if" => token_type = TokenType::IF,
        "else" => token_type = TokenType::ELSE,
        "true" => token_type = TokenType::TRUE,
        "false" => token_type = TokenType::FALSE,
        "while" => token_type = TokenType::WHILE,
        "fn" => token_type = TokenType::FN,
        "nil" => token_type = TokenType::NIL,
        "let" => token_type = TokenType::LET,
        "print" => token_type = TokenType::PRINT,
        "do" => token_type = TokenType::DO,
        "return" => token_type = TokenType::RETURN,
        _ => (),
    }

    return OuterIter {
        token: Token {
            token_type,
            line: current_line,
            column
        },
        iter_skip_steps,
    };
}

fn iterate_comment(current_index: usize, text: &str) -> usize {
    let mut iter_skip_steps = 0;

    //iterates until it finds newline character here 0xA and then returns the number of
    //iter_skip_steps 
    //i.e counts the chars in the comment
    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        if char == 0xA as char {
            return iter_skip_steps
        }

        iter_skip_steps += 1;
    }

    return iter_skip_steps
}


pub fn line_column_lexer(text : String) -> Vec<Token> {
    let mut tokens : Vec<Token> = vec![];

    //regex to see whether a char is start of a number or identifier
    let identifier_start_regex = Regex::new(r"[a-zA-Z]|_").unwrap();
    let number_start_regex = Regex::new(r"[0-9]").unwrap();


    //the lines of code
    let lines = text.split("\n");

    //instead of like before this code loops over every line instead of every character, 
    //the time complexety stays the same, but line and column handling
    //and thus later error handling become easier to accomplish
    for (line_count, line) in lines.clone().enumerate(){
        //this is the number that appears in your error later 
        //we add one because iterator usually start at 0 but line 0 is a bit dumb...
        let line_count = line_count as i32 + 1;



        //the amount of characters a given "word" has is also the amount of characters we have to
        //skip 
        let mut skip_chars = 0;
        for (column, char) in line.chars().enumerate(){
            //same for columns, theyll come in handy if we want to make cool and useful error
            //messages later on
            let column_count = column as i32 + 1;

            //if we want to skip character in an iteration we just increase skip_chars
            if skip_chars > 0{
                skip_chars -= 1;
                continue;
            } 

            //convert char into string for easier comparison
            let char = char.to_string(); 


            //these below couldnt fit into a match because they are dynamic and also kind of
            //different 
            //if we match one of these characters they all go into a seperate mode and look for the
            //ending character and if we encounter it we return the accumulated string value and
            //how much characters it has (we have to skip)
            if char == "\""{
                let token_iter = iterate_string(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                //these methods overshoot by one (because of the looping behaviour) so they have to
                //be put one behind
                skip_chars = token_iter.iter_skip_steps - 1;
            }

            if identifier_start_regex.is_match(&char){
                let token_iter = iterate_identifier(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps - 1;
            }

            if number_start_regex.is_match(&char){
                let token_iter = iterate_number(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps - 1;
            }
           

            //this is straightforward
            match char.as_str() {
            "(" => tokens.push(simple_token(TokenType::LPAREN, line_count, column_count)),
            ")" => tokens.push(simple_token(TokenType::RPAREN, line_count, column_count)),
            "{" => tokens.push(simple_token(TokenType::LBRACE, line_count, column_count)),
            "}" => tokens.push(simple_token(TokenType::RBRACE, line_count, column_count)),
            "[" => tokens.push(simple_token(TokenType::LBRACK, line_count, column_count)),
            "]" => tokens.push(simple_token(TokenType::RBRACK, line_count, column_count)),
            ";" => tokens.push(simple_token(TokenType::SEMICOLON, line_count, column_count)),
            "," => tokens.push(simple_token(TokenType::COMMA, line_count, column_count)),
            "." => tokens.push(simple_token(TokenType::DOT, line_count, column_count)),
            "*" => tokens.push(simple_token(TokenType::STAR, line_count, column_count)),
            "+" => tokens.push(simple_token(TokenType::PLUS, line_count, column_count)),
            "-" => tokens.push(simple_token(TokenType::MINUS, line_count, column_count)),

            "&" => tokens.push(simple_token(TokenType::AMP, line_count, column_count)),
            "^" => tokens.push(simple_token(TokenType::UP, line_count, column_count)),
            "%" => tokens.push(simple_token(TokenType::PERCENT, line_count, column_count)),
            "?" => tokens.push(simple_token(TokenType::QMARK, line_count, column_count)),

            //any of the below matches either a single or double character token depending of the
            //next value
            "!" => tokens.push(simple_token(
                if peek(column, line).as_str() == "=" {
                    skip_chars = 2;
                    TokenType::BANGEQ
                } else {
                    TokenType::BANG
                },
                line_count,
                column_count
            )),
            "=" => tokens.push(simple_token(
                if peek(column, line).as_str() == "=" {
                    skip_chars = 2;
                    TokenType::EQEQ
                } else {
                    TokenType::EQ
                },
                line_count,
                column_count
            )),
            "<" => tokens.push(simple_token(
                if peek(column, line).as_str() == "=" {
                    skip_chars = 2;
                    TokenType::LESSEQ
                } else {
                    TokenType::LESS
                },
                line_count,
                column_count
            )),
            ">" => tokens.push(simple_token(
                if peek(column, line).as_str() == "=" {
                    skip_chars = 2;
                    TokenType::GREATEREQ
                } else {
                    TokenType::GREATER
                },
                line_count,
                column_count
            )),

            "/" => {
                if peek(column, line) == "/" {
                    skip_chars = iterate_comment(column, line);
                //in case of comment skips rest of line
                } else {
                    tokens.push(simple_token(TokenType::SLASH, line_count, column_count))
                }
            }

            _ => (),
        }

        }
        
    } 


    let lines : Vec<&str> = lines.collect();

    tokens.push(simple_token(TokenType::EOF, lines.len() as i32, 0));

    tokens
}

//puts it all together
pub fn lexer(text: String) -> Vec<Token> {

    line_column_lexer(text.clone())

}
