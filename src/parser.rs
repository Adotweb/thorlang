use crate::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Print {
        expression: Expression,
    },
    Do {
        expression: Expression,
    },
    Variable {
        name: String,
        expression: Expression,
    },
    Block {
        statements: Vec<Statement>,
    },
    If {
        condition: Expression,
        then_branch: Box<Vec<Statement>>,
        else_branch: Option<Box<Vec<Statement>>>,
    },
    While {
        condition: Expression,
        block: Box<Vec<Statement>>,
    },
    Function {
        name: String,
        body: Box<Vec<Statement>>,
        arguments: Vec<String>,
    },
    Return {
        expression: Expression,
    },
}

//consumes the current token and returns the next one
fn consume_token(current_index: &mut usize, tokens: &Vec<Token>) -> Token {
    *current_index += 1;
    tokens.get(*current_index).unwrap().clone()
}

//generates a list of statments and returns the global "program" list (list of ASTs) that will be
//individually executed in eval_stmts later (in the context of a global "program")

pub fn statement(current_index: &mut usize, tokens: &Vec<Token>) -> Vec<Statement> {
    let mut statements = vec![];

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::RETURN => {
                consume_token(current_index, tokens);
                statements.push(return_statement(current_index, tokens));
            }
            TokenType::PRINT => {
                //consume the token
                consume_token(current_index, tokens);
                statements.push(print_statement(current_index, tokens))
            }
            TokenType::FN => {
                consume_token(current_index, tokens);
                statements.push(function_statement(current_index, tokens))
            }
            TokenType::DO => {
                consume_token(current_index, tokens);
                statements.push(do_statement(current_index, tokens))
            }
            TokenType::IF => {
                consume_token(current_index, tokens);
                statements.push(if_statement(current_index, tokens))
            }
            TokenType::WHILE => {
                consume_token(current_index, tokens);
                statements.push(while_statement(current_index, tokens))
            }
            TokenType::LET => {
                consume_token(current_index, tokens);
                statements.push(declaration(current_index, tokens))
            }
            TokenType::LBRACE => {
                consume_token(current_index, tokens);
                statements.push(Statement::Block {
                    statements: statement(current_index, tokens),
                })
            }
            TokenType::RBRACE => {
                consume_token(current_index, tokens);
                return statements;
            }
            TokenType::ELSE => {
                //dont consume the else token as it is needed one layer of recursion above
                return statements;
            }

            TokenType::EOF => return statements,

            _ => {
                //tries to automatically run and expressions when just written. Works semantically
                //the same as "do expression";
                statements.push(do_statement(current_index, tokens));
            }
        }
    }

    return statements;
}

fn return_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    let expression = expr(current_index, tokens);

    //consume the token
    consume_token(current_index, tokens);

    return Statement::Return { expression };
}

fn while_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    if tokens.get(*current_index).unwrap().token_type != TokenType::LPAREN {
        panic!(
            "expected ( after while on line {:?}",
            tokens.get(*current_index).unwrap().line
        )
    }

    consume_token(current_index, tokens);

    let condition = expr(current_index, tokens);

    if tokens.get(*current_index).unwrap().token_type != TokenType::RPAREN {
        panic!(
            "expected ) after condition on line {:?}",
            tokens.get(*current_index).unwrap().line
        )
    }

    //consume the ")" and the "{" tokens;
    consume_token(current_index, tokens); 
    consume_token(current_index, tokens);

    let block = Box::new(statement(current_index, tokens));

    return Statement::While { condition, block };
}

fn if_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    if tokens.get(*current_index).unwrap().token_type != TokenType::LPAREN {
        panic!(
            "expected ( after if on line {:?}",
            tokens.get(*current_index).unwrap().line
        )
    }
    //consume the paren token
    consume_token(current_index, tokens);

    let condition = expr(current_index, tokens);

    if tokens.get(*current_index).unwrap().token_type != TokenType::RPAREN {
        panic!(
            "expected ) after if condition on line {:?}",
            tokens.get(*current_index).unwrap().line
        );
    }

    //consume the ")" and the "{" (two tokens)
    consume_token(current_index, tokens);
    consume_token(current_index, tokens);

    let then_branch = Box::new(statement(current_index, tokens));

    let mut else_branch = None;

    if tokens.get(*current_index).unwrap().token_type == TokenType::ELSE {
        consume_token(current_index, tokens);
        else_branch = Some(Box::new(statement(current_index, tokens)));
    }

    return Statement::If {
        condition,
        then_branch,
        else_branch,
    };
}

fn print_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    let expression = expr(current_index, tokens);

    if tokens.get(*current_index).unwrap().token_type != TokenType::SEMICOLON {
        panic!("expected ; after {:?}", tokens.get(*current_index));
    }

    //consume token (is ";" because throws otherwise) and move on.
    consume_token(current_index, tokens);

    return Statement::Print {
        expression
    };
}

fn function_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {

    let mut token = tokens.get(*current_index).unwrap();
    let mut function_name : String;

    if let TokenType::IDENTIFIER(str) = &token.token_type{
        function_name = str.to_string(); 
    } else {
        panic!("expected identifier after fn keyword on line {:?}", token.line);
    }
    //consume the identifier token
    let mut token = &consume_token(current_index, tokens).clone();

    let mut args = vec![];

    if token.token_type != TokenType::LPAREN {
        panic!("expected ( after function name for argument list on line {:?}", token.line)
    }

    //consume the paren token
    
    let mut token = &consume_token(current_index, tokens).clone();

    while token.token_type != TokenType::RPAREN {

        match &token.token_type {
            TokenType::COMMA => {
                *current_index += 1;
            }
            TokenType::IDENTIFIER(str) => {
                args.push(str.clone());
                *current_index += 1;
            }
            _ => (),
        }
        token = tokens.get(*current_index).unwrap();
    }

    //consume rparen
    
    let token = &consume_token(current_index, tokens);

    if token.token_type != TokenType::LBRACE {
        panic!("expected block after function declaration on line {:?}", token.line)
    }

    //consume the rbrace token
    consume_token(current_index, tokens);

    let block = statement(current_index, tokens);

    Statement::Function {
        arguments: args,
        name: function_name,
        body: Box::new(block),
    }
}

//do turns expressions into statements;
fn do_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    let expression = expr(current_index, tokens);
    
    let token = tokens.get(*current_index).unwrap();

    if token.token_type != TokenType::SEMICOLON {
        panic!(
            "expected ; after {:?} on line {:?}", token.token_type, token.line)
    }

    consume_token(current_index, tokens);

    return Statement::Do {
        expression,
    };
}

fn declaration(current_index: &mut usize, tokens: &Vec<Token>) -> Statement {
    let name : String;
    let token = tokens.get(*current_index).unwrap().clone();


    if let TokenType::IDENTIFIER(str) = token.token_type {
        name = str;
    } else {
        panic!("exptected a variable name")
    }

    let token = &consume_token(current_index, tokens);
    
    let mut init : Expression = Expression::Literal{
        literal : TokenType::NIL
    };

    if token.token_type == TokenType::EQ {
    
        consume_token(current_index, tokens);
        init = expr(current_index, tokens);
    }

    if tokens.get(*current_index).unwrap().token_type != TokenType::SEMICOLON {
        panic!("exptected ; after value {:?}", tokens.get(*current_index));
    }

    consume_token(current_index, tokens);

    //consume token (is ";" because throws otherwise) and move on.

    return Statement::Variable {
        name,
        expression:init,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier {
        name: String,
    },
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
        literal: TokenType,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Array {
        values: Vec<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren: Token,
        arguments: Vec<Expression>,
    },
    Retrieve {
        retrievee: Box<Expression>,
        key: Box<Expression>,
    },
    FieldCall {
        callee: Box<Expression>,
        key: Box<Expression>,
    },
}

fn expr(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    assign(current_index, tokens)
}

fn assign(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let expression = eq(current_index, tokens);

    if let Some(token) = tokens.get(*current_index) {
        if token.token_type == TokenType::EQ {
            
            consume_token(current_index, tokens);

            let value = assign(current_index, tokens);

            return Expression::Assignment {
                target: Box::new(expression),
                value: Box::new(value),
            };
        }
    }

    return expression;
}

fn eq(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = comp(current_index, tokens);

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::EQEQ | TokenType::BANGEQ => {
                let operator = token.token_type.clone();
                
                consume_token(current_index, tokens);

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
                
                consume_token(current_index, tokens);

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
                
                consume_token(current_index, tokens);

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
                
                consume_token(current_index, tokens);

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
                
                consume_token(current_index, tokens);

                let right = unary(current_index, tokens);
                return Expression::Unary {
                    operator,
                    right: Box::new(right),
                };
            }
            _ => {}
        }
    }

    call(current_index, tokens)
}

// needs to check whether or not the expression returned in finishcall is a function itself, and if
// it is evaluate as well given more arguments/a call invocation i.e. "()"

fn call(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    let mut expression = primary(current_index, tokens);

    let mut current_token = tokens.get(*current_index).unwrap().token_type.clone();

    while current_token == TokenType::LPAREN
        || current_token == TokenType::LBRACK
        || current_token == TokenType::DOT
    {
        if current_token == TokenType::DOT {
            
            consume_token(current_index, tokens);

            let key = primary(current_index, tokens);

            expression = Expression::FieldCall {
                callee: Box::new(expression),
                key: Box::new(key),
            }
        }

        if current_token == TokenType::LPAREN {
            //consume the ( token
            
            consume_token(current_index, tokens);

            expression = finish_call(current_index, tokens, expression.clone());

            consume_token(current_index, tokens);
        }

        if current_token == TokenType::LBRACK {
            consume_token(current_index, tokens);

            let key = expr(current_index, tokens);

            expression = Expression::Retrieve {
                retrievee: Box::new(expression),
                key: Box::new(key),
            };

            consume_token(current_index, tokens);
        }

        current_token = tokens.get(*current_index).unwrap().token_type.clone();
    }

    expression
}

fn finish_call(current_index: &mut usize, tokens: &Vec<Token>, callee: Expression) -> Expression {
    let mut arguments: Vec<Expression> = vec![];

    while let Some(token) = tokens.get(*current_index) {
        match &token.token_type {
            TokenType::RPAREN => {
                return Expression::Call {
                    callee: Box::new(callee),
                    arguments,
                    paren: token.clone(),
                }
            }
            TokenType::COMMA => {
                //consume the comma token

                consume_token(current_index, tokens);
            }
            TokenType::IDENTIFIER(_str) => {
                let argument = expr(current_index, tokens);

                arguments.push(argument);
            }

            _ => {
                let argument = expr(current_index, tokens);

                arguments.push(argument);
            }
        }
    }

    panic!(
        "no delimiter in argument list on line {:?}",
        tokens.get(*current_index).unwrap().line
    )
}

fn primary(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    if let Some(token) = tokens.get(*current_index) {
        consume_token(current_index, tokens);
        match &token.token_type {
            TokenType::IDENTIFIER(str) => Expression::Identifier {
                name: str.to_string(),
            },
            TokenType::TRUE => Expression::Literal {
                literal: TokenType::TRUE,
            },
            TokenType::FALSE => Expression::Literal {
                literal: TokenType::FALSE,
            },
            TokenType::NUMBER(num) => Expression::Literal {
                literal: TokenType::NUMBER(num.to_string())
            },
            TokenType::STRING(str) => Expression::Literal {
                literal: TokenType::STRING(str.to_string())
            },
            TokenType::NIL => Expression::Literal {
                literal: TokenType::NIL
            },
            TokenType::LBRACK => {
                if token.token_type == TokenType::RBRACK {
                    return Expression::Array { values: vec![] };
                }

                let mut array: Vec<Expression> = vec![];

                array.push(expr(current_index, tokens));

                while let Some(token) = tokens.get(*current_index) {
                    match token.token_type {
                        TokenType::RBRACK => {
                            consume_token(current_index, tokens);
                            return Expression::Array { values: array };
                        }
                        TokenType::COMMA => {
                            consume_token(current_index, tokens); 
                            let value = expr(current_index, tokens);

                            array.push(value);
                        }

                        _ => unimplemented!(),
                    }
                }

                Expression::Array { values: array }
            }
            TokenType::LPAREN => {
                let expression = expr(current_index, tokens);
                if let Some(token) = tokens.get(*current_index) {
                    if token.token_type == TokenType::RPAREN {
                        consume_token(current_index, tokens);
                    } else {
                        panic!("Expected closing parenthesis");
                    }
                }
                Expression::Grouping {
                    inner: Box::new(expression),
                }
            }
            _ => Expression::Literal {
                literal: TokenType::NIL,
            },
        }
    } else {
        Expression::Literal {
            literal: TokenType::NIL,
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {
    let mut current_index: usize = 0;
    statement(&mut current_index, &tokens)
}
