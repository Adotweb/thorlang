use regex::Regex;

//the different token types
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum TokenType {
    LPAREN,
    RPAREN,
    LBRACK,
    RBRACK,
    LBRACE,
    RBRACE,
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

    DO,
    AND,
    ELSE,
    FALSE,
    FN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    TRUE,
    LET,
    WHILE,

    EOF,
}


//since the num str and identifier enums can all hold data we dont have the need for a literaltype
//anymore
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
}

fn simple_token(token_type: TokenType, line: i32) -> Token {
    return Token {
        token_type,
        line
    };
}

//returns the next character in the text
fn peek(current_index: usize, text: &str) -> String {
    //returns char at iter + 1

    return text.chars().nth(current_index + 1).unwrap().to_string();
}

struct OuterIter {
    token: Token,
    iter_skip_steps: usize,
}

fn iterate_string(current_index: usize, text: &str, current_line: i32) -> OuterIter {
    let mut iter_skip_steps: usize = 1;

    let mut currentchar = text
        .chars()
        .nth(current_index + iter_skip_steps)
        .unwrap()
        .to_string();
    let mut string_value = "".to_string();

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
        },
        iter_skip_steps,
    };
}

fn iterate_number(current_index: usize, text: &str, current_line: i32) -> OuterIter {
    let mut iter_skip_steps = 1;

    let mut currentchar = text
        .chars()
        .nth(current_index + iter_skip_steps)
        .unwrap()
        .to_string();

    let first_number_char = text.chars().nth(current_index).unwrap().to_string();

    let mut number_value = first_number_char;
    let number_regex = Regex::new(r"[0-9]|\.").unwrap();

    let mut dotused = false;

    while number_regex.is_match(&currentchar) {
        if currentchar == "." && dotused {
            panic!("you may only use one dot in numbers")
        }

        if currentchar == "." && !dotused {
            dotused = false;
            let nextchar = peek(current_index + iter_skip_steps, text);

            if !number_regex.is_match(&nextchar) {
                return OuterIter {
                    token: Token {
                        token_type: TokenType::NUMBER(number_value),
                        line: current_line,
                    },
                    iter_skip_steps,
                };
            }
        }

        number_value += &currentchar;
        iter_skip_steps += 1;
        currentchar = text
            .chars()
            .nth(current_index + iter_skip_steps)
            .unwrap()
            .to_string();
    }

    return OuterIter {
        token: Token {
            token_type: TokenType::NUMBER(number_value),
            line: current_line,
        },
        iter_skip_steps,
    };
}

fn iterate_identifier(current_index: usize, text: &str, current_line: i32) -> OuterIter {
    //identifiers have the following regular form : [a-zA-Z]([a-zA-Z0-9]|_)*
    //this method only has to check from the second letter onwards (till the end of the "word").

    let id_regex = Regex::new(r"[a-zA-Z0-9]|_").unwrap();
    let mut iter_skip_steps: usize = 1; //counts how many iterations have to be skipped in the main iteration loop

    let first_identifier_char = text.chars().nth(current_index).unwrap().to_string();

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

    let mut token_type = TokenType::IDENTIFIER(identifier.clone());
    //check if identifier is part of keywords
    match identifier.as_str() {
        "if" => token_type = TokenType::IF,
        "else" => token_type = TokenType::ELSE,
        "and" => token_type = TokenType::AND,
        "or" => token_type = TokenType::OR,
        "true" => token_type = TokenType::TRUE,
        "false" => token_type = TokenType::FALSE,
        "while" => token_type = TokenType::WHILE,
        "for" => token_type = TokenType::FOR,
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
        },
        iter_skip_steps,
    };
}

fn iterate_comment(current_index: usize, text: &str) -> usize {
    let mut iter_skip_steps = 0;

    //iterates until it finds newline character here 0xA and then returns the number of
    //iter_skip_steps
    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        if char == 0xA as char {
            return iter_skip_steps
        }

        iter_skip_steps += 1;
    }

    return iter_skip_steps
}

pub fn lexer(text: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut line_count = 1;

    let mut iter = 0;

    let text = text.trim();

    let identifier_start_regex = Regex::new(r"[a-zA-Z]|_").unwrap();
    let number_start_regex = Regex::new(r"[0-9]").unwrap();

    while iter < text.len() {
        let char = text.chars().nth(iter).unwrap().to_string();

        let mut iter_skip_steps: usize = 1; //can be changed to increase the amount of chars a certain
                                            //operation consumes.

        //checking for identifiers and numbers has to happen BEFORE special characters, because
        //numbers may contain dots and these can not be marked as DOT tokens.

        //checking for numbers

        if char == "\"" {
            let string_iter = iterate_string(iter, text, line_count);

            tokens.push(string_iter.token);
            iter_skip_steps = string_iter.iter_skip_steps + 1;
        }

        if identifier_start_regex.is_match(char.as_str()) {
            let identifier_iter = iterate_identifier(iter, text, line_count);

            tokens.push(identifier_iter.token);
            iter_skip_steps = identifier_iter.iter_skip_steps;
        }
        if number_start_regex.is_match(char.as_str()) {
            let identifier_iter = iterate_number(iter, text, line_count);

            tokens.push(identifier_iter.token);
            iter_skip_steps = identifier_iter.iter_skip_steps;
        }

        match char.as_str() {
            "\n" => line_count += 1,
            "(" => tokens.push(simple_token(TokenType::LPAREN, line_count)),
            ")" => tokens.push(simple_token(TokenType::RPAREN, line_count)),
            "{" => tokens.push(simple_token(TokenType::LBRACE, line_count)),
            "}" => tokens.push(simple_token(TokenType::RBRACE, line_count)),
            "[" => tokens.push(simple_token(TokenType::LBRACK, line_count)),
            "]" => tokens.push(simple_token(TokenType::RBRACK, line_count)),
            ";" => tokens.push(simple_token(TokenType::SEMICOLON, line_count)),
            "," => tokens.push(simple_token(TokenType::COMMA, line_count)),
            "." => tokens.push(simple_token(TokenType::DOT, line_count)),
            "*" => tokens.push(simple_token(TokenType::STAR, line_count)),
            "+" => tokens.push(simple_token(TokenType::PLUS, line_count)),
            "-" => tokens.push(simple_token(TokenType::MINUS, line_count)),

            "!" => tokens.push(simple_token(
                if peek(iter, text).as_str() == "=" {
                    iter_skip_steps = 2;
                    TokenType::BANGEQ
                } else {
                    TokenType::BANG
                },
                line_count,
            )),
            "=" => tokens.push(simple_token(
                if peek(iter, text).as_str() == "=" {
                    iter_skip_steps = 2;
                    TokenType::EQEQ
                } else {
                    TokenType::EQ
                },
                line_count,
            )),
            "<" => tokens.push(simple_token(
                if peek(iter, text).as_str() == "=" {
                    iter_skip_steps = 2;
                    TokenType::LESSEQ
                } else {
                    TokenType::LESS
                },
                line_count,
            )),
            ">" => tokens.push(simple_token(
                if peek(iter, text).as_str() == "=" {
                    iter_skip_steps = 2;
                    TokenType::GREATEREQ
                } else {
                    TokenType::GREATER
                },
                line_count,
            )),

            "/" => {
                if peek(iter, text) == "/" {
                    iter_skip_steps = iterate_comment(iter, text);
                //in case of comment skips rest of line
                } else {
                    tokens.push(simple_token(TokenType::SLASH, line_count))
                }
            }

            _ => (),
        }

        iter += iter_skip_steps
    }

    tokens.push(simple_token(TokenType::EOF, line_count));

    return tokens;
}
