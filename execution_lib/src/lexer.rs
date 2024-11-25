use regex::Regex;

use type_lib::{Token, TokenType};

//just a helper function for instantiation
//could be rewritten as an impl, but would make the code even more verbose so no
fn simple_token(token_type: TokenType, line: i32, column: i32) -> Token {
    return Token {
        token_type,
        line,
        column,
    };
}

//returns the next character in the text
fn peek(current_index: usize, text: &str) -> String {
    //returns char at iter + 1

    return text.chars().nth(current_index + 1).unwrap().to_string();
}

//just a wrapper type, unelegant and will be overworked in the future
//the token is just returned, while "iter_skip_steps" helps the loop keep track of how many chars
//can safely be skipped because they have already been handled in one of the helper functions
struct OuterIter {
    token: Token,
    iter_skip_steps: usize,
}

fn iterate_string(current_index: usize, text: &str, current_line: i32, column: i32) -> OuterIter {
    //iterate string skips at least one (the first matching) character
    let mut iter_skip_steps: usize = 1;

    let mut string = "".to_string();

    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        let char = char.to_string();

        //while there is some character that is not a " the string goes on and on
        if char != "\"" {
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
            column,
        },
        iter_skip_steps,
    };
}

fn iterate_number(current_index: usize, text: &str, current_line: i32, column: i32) -> OuterIter {
    //same as before
    let mut iter_skip_steps = 0;
    let mut number = "".to_string();
    let number_regex = Regex::new(r"[0-9]").unwrap();
    let mut dot_used = false;

    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        let char = char.to_string();

        //quite simple if the current char is a number, mark it down
        if number_regex.is_match(&char) {
            number += &char;
            iter_skip_steps += 1;
        }
        //if we encounter a dot and we have already used one we cannot mark the dot down
        else if (char == ".".to_string()) && !dot_used {
            //else we need to check if the next char even is a number else we cannot mark the dot
            //as part of the number and it is a grouping operator
            let next_char = peek(current_index + iter_skip_steps, text);
            dot_used = true;
            if number_regex.is_match(&next_char) {
                number += &char;

                //this is the part that consumes the dot token in case the next char is a number
                iter_skip_steps += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    //return the currently accumulated number
    return OuterIter {
        token: Token {
            token_type: TokenType::NUMBER(number),
            line: current_line,
            column,
        },
        iter_skip_steps,
    };
}

fn iterate_identifier(
    current_index: usize,
    text: &str,
    current_line: i32,
    column: i32,
) -> OuterIter {
    //identifiers have the following regular form : [a-zA-Z]([a-zA-Z0-9]|_)*
    //this method only has to check from the second letter onwards (till the end of the "word").

    let id_regex = Regex::new(r"[a-zA-Z0-9]|_").unwrap();
    let mut iter_skip_steps: usize = 0; //counts how many iterations have to be skipped in the main iteration loop

    let mut identifier = "".to_string();

    //iterate over the identifier until there are no more letters that match the identifier_regex
    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        let char = char.to_string();

        if id_regex.is_match(&char) {
            //again we need to tell the loop how many iterations are safely to skip
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
        "to" => token_type = TokenType::TO,
        "step" => token_type = TokenType::STEP,
        "on" => token_type = TokenType::ON,

        "overload" => token_type = TokenType::OVERLOAD,
        "if" => token_type = TokenType::IF,
        "else" => token_type = TokenType::ELSE,
        "true" => token_type = TokenType::TRUE,
        "false" => token_type = TokenType::FALSE,
        "while" => token_type = TokenType::WHILE,
        "for" => token_type = TokenType::FOR,
        "in" => token_type = TokenType::IN,
        "fn" => token_type = TokenType::FN,
        "nil" => token_type = TokenType::NIL,
        "let" => token_type = TokenType::LET,
        "print" => token_type = TokenType::PRINT,
        "do" => token_type = TokenType::DO,
        "return" => token_type = TokenType::RETURN,
        "throw" => token_type = TokenType::THROW,
        _ => (),
    }

    return OuterIter {
        token: Token {
            token_type,
            line: current_line,
            column,
        },
        iter_skip_steps,
    };
}

fn iterate_comment(current_index: usize, text: &str) -> usize {
    let mut iter_skip_steps = 0;

    //iterates until it finds newline character (here 0xA) and then returns the number of
    //iter_skip_steps
    //i.e counts the chars in the comment
    while let Some(char) = text.chars().nth(current_index + iter_skip_steps) {
        if char == 0xA as char {
            return iter_skip_steps;
        }

        iter_skip_steps += 1;
    }

    return iter_skip_steps;
}

pub fn line_column_lexer(text: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    //regex to see whether a char is start of a number or identifier
    //
    //
    //the identifier one matches everything that is not a reserved character;
    let identifier_start_regex = Regex::new(r"_|[a-zA-Z]").unwrap();
    let number_start_regex = Regex::new(r"[0-9]").unwrap();

    //the lines of code
    let lines = text.split("\n");

    //instead of like before this code loops over every line instead of every character,
    //the time complexety stays the same, but line and column handling
    //and thus later error handling become easier to accomplish
    for (line_count, line) in lines.clone().enumerate() {
        //this is the number that appears in your error later
        //we add one because iterator usually start at 0 but line 0 is a bit dumb...
        let line_count = line_count as i32 + 1;

        //the amount of characters a given "word" has is also the amount of characters we have to
        //skip
        let mut skip_chars = 0;
        for (column, char) in line.chars().enumerate() {
            //same for columns, theyll come in handy if we want to make cool and useful error
            //messages later on
            let column_count = column as i32 + 1;

            //if we want to skip character in an iteration we just increase skip_chars
            if skip_chars > 0 {
                skip_chars -= 1;
                continue;
            }

            if char.is_whitespace() {
                continue;
            }
            //convert char into string for easier comparison
            let char = char.to_string();

            //these below couldnt fit into a match because they are dynamic and also kind of
            //different
            //if we match one of these characters they all go into a seperate mode and look for the
            //ending character and if we encounter it we return the accumulated string value and
            //how much characters it has (we have to skip)
            if char == "\"" {
                let token_iter = iterate_string(column, line, line_count, column_count);

                tokens.push(token_iter.token);
                //these methods overshoot by one (because of the looping behaviour) so they have to
                //be put one behind
                skip_chars = token_iter.iter_skip_steps - 1;
                continue;
            }

            if identifier_start_regex.is_match(&char) {
                let token_iter = iterate_identifier(column, line, line_count, column_count);

                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps - 1;
                continue;
            }

            if number_start_regex.is_match(&char) {
                let token_iter = iterate_number(column, line, line_count, column_count);

                tokens.push(token_iter.token);
                skip_chars = token_iter.iter_skip_steps - 1;
                continue;
            }

            //matcht the token and return its corresponding value to
            match char.as_str() {
                "(" => {
                    tokens.push(simple_token(TokenType::LPAREN, line_count, column_count));
                    continue;
                }
                ")" => {
                    tokens.push(simple_token(TokenType::RPAREN, line_count, column_count));
                    continue;
                }
                "{" => {
                    tokens.push(simple_token(TokenType::LBRACE, line_count, column_count));
                    continue;
                }
                "}" => {
                    tokens.push(simple_token(TokenType::RBRACE, line_count, column_count));
                    continue;
                }
                "[" => {
                    tokens.push(simple_token(TokenType::LBRACK, line_count, column_count));
                    continue;
                }
                "]" => {
                    tokens.push(simple_token(TokenType::RBRACK, line_count, column_count));
                    continue;
                }
                ":" => {
                    tokens.push(simple_token(TokenType::COLON, line_count, column_count));
                    continue;
                }
                ";" => {
                    tokens.push(simple_token(TokenType::SEMICOLON, line_count, column_count));
                    continue;
                }
                "," => {
                    tokens.push(simple_token(TokenType::COMMA, line_count, column_count));
                    continue;
                }
                "." => {
                    tokens.push(simple_token(TokenType::DOT, line_count, column_count));
                    continue;
                }
                "*" => {
                    tokens.push(simple_token(TokenType::STAR, line_count, column_count));
                    continue;
                }
                "+" => {
                    tokens.push(simple_token(TokenType::PLUS, line_count, column_count));
                    continue;
                }
                "-" => {
                    tokens.push(simple_token(TokenType::MINUS, line_count, column_count));
                    continue;
                }

                //any of the below matches either a single or double character token depending of the
                //next value, also if the next token also matches we need to skip to chars, if not we
                //just move on marking down the single char token
                "!" => {
                    tokens.push(simple_token(
                        if peek(column, line).as_str() == "=" {
                            skip_chars = 2;
                            TokenType::BANGEQ
                        } else {
                            TokenType::BANG
                        },
                        line_count,
                        column_count,
                    ));
                    continue;
                }
                "=" => {
                    tokens.push(simple_token(
                        if peek(column, line).as_str() == "=" {
                            skip_chars = 2;
                            TokenType::EQEQ
                        } else {
                            TokenType::EQ
                        },
                        line_count,
                        column_count,
                    ));
                    continue;
                }
                "<" => {
                    tokens.push(simple_token(
                        if peek(column, line).as_str() == "=" {
                            skip_chars = 2;
                            TokenType::LESSEQ
                        } else {
                            TokenType::LESS
                        },
                        line_count,
                        column_count,
                    ));
                    continue;
                }
                ">" => {
                    tokens.push(simple_token(
                        if peek(column, line).as_str() == "=" {
                            skip_chars = 2;
                            TokenType::GREATEREQ
                        } else {
                            TokenType::GREATER
                        },
                        line_count,
                        column_count,
                    ));
                    continue;
                }

                "/" => {
                    if peek(column, line) == "/" {
                        skip_chars = iterate_comment(column, line);
                        continue;
                    //in case of comment skips rest of line
                    } else {
                        tokens.push(simple_token(TokenType::SLASH, line_count, column_count));
                        continue;
                    }
                }

                _ => {
                    //special characters will always (like + - or similar) only consume a single
                    //character to make operator chaining easier
                    tokens.push(simple_token(
                        TokenType::SPECIAL(char),
                        line_count,
                        column_count,
                    ));
                }
            }
        }
    }

    let lines: Vec<&str> = lines.collect();

    tokens.push(simple_token(TokenType::EOF, lines.len() as i32, 0));

    tokens
}

//puts it all together
pub fn lexer(text: String) -> Vec<Token> {
    line_column_lexer(text.clone())
}
