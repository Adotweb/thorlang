use crate::{Token, Literal, LiteralType, TokenType};

#[derive(Debug)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: TokenType,
        right: Box<Expression>,
    },
    Unary {
        operator: TokenType,
        right: Box<Expression>,
    },
    Grouping {
        inner: Box<Expression>,
    },
    Literal {
        literal: LiteralType,
    },
}

fn expr(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    eq(current_index, tokens)
}

fn eq(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = comp(current_index, tokens);

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::EQEQ | TokenType::BANGEQ => {
                let operator = token.token_type.clone();
                *current_index += 1;
                let right = comp(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    expression
}

fn comp(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = term(current_index, tokens);

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::GREATEREQ | TokenType::GREATER | TokenType::LESS | TokenType::LESSEQ => {
                let operator = token.token_type.clone();
                *current_index += 1;
                let right = term(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    expression
}

fn term(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = factor(current_index, tokens);

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::PLUS | TokenType::MINUS => {
                let operator = token.token_type.clone();
                *current_index += 1;
                let right = factor(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    expression
}

fn factor(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = unary(current_index, tokens);

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::STAR | TokenType::SLASH => {
                let operator = token.token_type.clone();
                *current_index += 1;
                let right = unary(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    expression
}

fn unary(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    if let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::BANG | TokenType::MINUS => {
                let operator = token.token_type.clone();
                *current_index += 1;
                let right = unary(current_index, tokens);
                return Expression::Unary {
                    operator,
                    right: Box::new(right),
                };
            }
            _ => {}
        }
    }

    primary(current_index, tokens)
}

fn primary(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    if let Some(token) = tokens.get(*current_index) {
        *current_index += 1;
        match &token.token_type {
            TokenType::TRUE => Expression::Literal {
                literal: LiteralType::BOOL { value: true },
            },
            TokenType::FALSE => Expression::Literal {
                literal: LiteralType::BOOL { value: false },
            },
            TokenType::NUMBER => Expression::Literal {
                literal: LiteralType::NUMBER {
                    value: token.string.clone().unwrap().parse().unwrap(),
                },
            },
            TokenType::STRING => Expression::Literal {
                literal: LiteralType::STRING {
                    value: token.string.clone().unwrap(),
                },
            },
            TokenType::NIL => Expression::Literal {
                literal: LiteralType::NIL,
            },
            TokenType::LPAREN => {
                let expression = expr(current_index, tokens);
                if let Some(token) = tokens.get(*current_index) {
                    if token.token_type == TokenType::RPAREN {
                        *current_index += 1;
                    } else {
                        panic!("Expected closing parenthesis");
                    }
                }
                Expression::Grouping {
                    inner: Box::new(expression),
                }
            }
            _ => Expression::Literal {
                literal: LiteralType::NIL,
            },
        }
    } else {
        Expression::Literal {
            literal: LiteralType::NIL,
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Expression {
    let mut current_index: usize = 0;
    expr(&mut current_index, &tokens)
}

