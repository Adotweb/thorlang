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

    let mut currentchar = text
        .chars()
        .nth(current_index + iter_skip_steps)
        .unwrap()
        .to_string();
    let mut string_value = "".to_string();


    //doublequote characters break the string
    while currentchar != "\"" {
        string_value += &currentchar;

        iter_skip_steps += 1;
        currentchar = text
            .chars()
            .nth(current_index + iter_skip_steps)
            .unwrap()
            .to_string();
    }

    return OuterIter {
        token: Token {
            token_type: TokenType::STRING(string_value),
            line: current_line,
            column
        },
        iter_skip_steps,
    };
}

fn iterate_number(current_index: usize, text: &str, current_line: i32, column : i32) -> OuterIter {

    //same as before
    let mut iter_skip_steps = 1;

    let mut currentchar = text
        .chars()
        .nth(current_index + iter_skip_steps)
        .unwrap()
        .to_string();

    let first_number_char = text.chars().nth(current_index).unwrap().to_string();

    let mut number_value = first_number_char;

    //any number and the dot is valid inside a number
    let number_regex = Regex::new(r"[0-9]|\.").unwrap();

    //however only one dot is allowed in one number
    //so this makes the lexer throw when more than one dot is encountered
    let mut dotused = false;


    //this just looks over every character that matches the above regex
    while number_regex.is_match(&currentchar) {
        //throws when second dot
        if currentchar == "." && dotused {
            panic!("you may only use one dot in numbers")
        }
        
        //check if character after dot is a number
        if currentchar == "." && !dotused {
            dotused = false;
            let nextchar = peek(current_index + iter_skip_steps, text);

            //as soon as a "not number" is encountered after the first dot the program returns 
            //the current accumulated number (next char could be part of method name)
            if !number_regex.is_match(&nextchar) {
                return OuterIter {
                    token: Token {
                        token_type: TokenType::NUMBER(number_value),
                        line: current_line,
                        column
                    },
                    iter_skip_steps,
                };
            }
        }

        //add the number to the string
        number_value += &currentchar;
        iter_skip_steps += 1;
        currentchar = text
            .chars()
            .nth(current_index + iter_skip_steps)
            .unwrap()
            .to_string();
    }


    //return the currently accumulated number
    return OuterIter {
        token: Token {
            token_type: TokenType::NUMBER(number_value),
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
    let mut iter_skip_steps: usize = 1; //counts how many iterations have to be skipped in the main iteration loop

    let first_identifier_char = text.chars().nth(current_index).unwrap().to_string();
    
    //will be optimized to use simple arrays instead of this mess in the future
    let mut currentchar = text
        .chars()
        .nth(current_index + iter_skip_steps)
        .unwrap()
        .to_string();

    let mut identifier = first_identifier_char;

    while id_regex.is_match(&currentchar) {
        identifier += &currentchar;
        iter_skip_steps += 1;

        currentchar = text
            .chars()
            .nth(current_index + iter_skip_steps)
            .unwrap()
            .to_string();
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

    let identifier_start_regex = Regex::new(r"[a-zA-Z]|_").unwrap();
    let number_start_regex = Regex::new(r"[0-9]").unwrap();

    let lines = text.split("\n");

    for (line_count, line) in lines.enumerate(){
        let line_count = line_count as i32 + 1;
        let mut skip_chars = 0;
        for (column, char) in line.chars().enumerate(){
            let column_count = column as i32;
            if skip_chars > 0{
                skip_chars -= 1;
                continue;
            } 

            let char = char.to_string(); 

            if char == "\""{
                let token_iter = iterate_string(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps;
            }

            if identifier_start_regex.is_match(&char){
                let token_iter = iterate_identifier(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps;
            }

            if number_start_regex.is_match(&char){
                let token_iter = iterate_number(column, line, line_count, column_count);
                
                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps;
            }
            
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


    tokens
}

//puts it all together
pub fn lexer(text: String) -> Vec<Token> {

    line_column_lexer(text.clone())

}
