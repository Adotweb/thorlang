use crate::{Token, TokenType, stringify_value, ThorLangError, typo_check};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Throw{
        exception : Expression,
    },
    Print {
        expression: Expression,
        line : i32
    },
    Do {
        expression: Expression,
        line : i32
    },
    Variable {
        name: String,
        expression: Expression,
        line : i32
    },
    Block {
        statements: Vec<Statement>,
        line : i32
    },
    If {
        condition: Expression,
        then_branch: Box<Vec<Statement>>,
        else_branch: Option<Box<Vec<Statement>>>,
        line : i32
    },
    While {
        condition: Expression,
        block: Box<Vec<Statement>>,
        line : i32
    },
    Function {
        name: String,
        body: Box<Vec<Statement>>,
        arguments: Vec<String>,
        line : i32
    },
    Return {
        expression: Expression,
        line : i32
    },
    Overload {
        operator : TokenType, 
        operands : Vec<String>,
        operation : Vec<Statement>,
        line : i32
    }
}


fn get_previous_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> &'a Token{
    let current_token = tokens.get(*current_index - 1).unwrap();
    current_token
}

fn get_current_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> &'a Token{
    let current_token = tokens.get(*current_index).unwrap();
    current_token
}

//consumes the current token and returns the next one
fn consume_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> &'a Token {
    *current_index += 1;
    let temp = &tokens[*current_index];
    temp
}

//returns the previous token
fn prev_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> &'a Token{
    &tokens[*current_index - 1]
}

fn get_statement_line<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> i32{
    let line = prev_token(current_index, tokens);

    line.line
}

fn match_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>, token_type : TokenType) -> Result<&'a Token, ThorLangError>{
    
    let prev_token =  get_previous_token(current_index, tokens);
    let token = get_current_token(current_index, tokens);

    if token.token_type != token_type{

        if let Err(err) = ThorLangError::unexpected_token::<Statement>(token_type, token.clone(), prev_token.clone()){
            return Err(err)
        };
    }
    
    *current_index += 1;

    Ok(&tokens[*current_index])
}


//generates a list of statments and returns the global "program" list (list of ASTs) that will be
//individually executed in eval_stmts later (in the context of a global "program")

pub fn statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Vec<Statement>, ThorLangError> {
    let mut statements = vec![];

    let mut ret : Result<Statement, ThorLangError>;

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type { 
            TokenType::OVERLOAD => {
                consume_token(current_index, tokens);
                ret = overload_statement(current_index, tokens)
            },
            TokenType::RETURN => {
                consume_token(current_index, tokens);
                ret = return_statement(current_index, tokens)
            },
            TokenType::THROW => {
                consume_token(current_index, tokens);
                ret = throw_statement(current_index, tokens);
            },
            TokenType::PRINT => {
                consume_token(current_index, tokens);
                ret = print_statement(current_index, tokens)
            },
            TokenType::FN => {
                consume_token(current_index, tokens);
                ret = function_statement(current_index, tokens)
            },
            TokenType::DO => {
                consume_token(current_index, tokens);
                ret = do_statement(current_index, tokens)
            },
            TokenType::IF => {
                consume_token(current_index, tokens);
                ret = if_statement(current_index, tokens)
            },
            TokenType::WHILE => {
                consume_token(current_index, tokens);
                ret = while_statement(current_index, tokens)
            },
            TokenType::LET => {
                consume_token(current_index, tokens);
                ret = declaration(current_index, tokens)
            },
            TokenType::LBRACE => {
                consume_token(current_index, tokens);
                ret = Ok(Statement::Block {
                    statements: statement(current_index, tokens)?,
                    line : token.line
                })
            },
            TokenType::RBRACE => {
                consume_token(current_index, tokens);
                return Ok(statements);
            },
            TokenType::ELSE => {
                //dont consume the else token as it is needed one layer of recursion above
                return Ok(statements);
            },

            TokenType::EOF => return Ok(statements),

            _ => {
                //first tries to understand if the first token is written incorrectly (typo)
                let token = get_current_token(current_index, tokens); 
                if let Some(typo_error) = typo_check(token.clone()){
                    return Err(typo_error)
                }             
    

                //tries to automatically run and expressions when just written. Works semantically
                //the same as "do expression";
                ret = do_statement(current_index, tokens);
            }
        }

        statements.push(ret?);
    }

    return Ok(statements);
}


fn throw_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError>{ 
    let exception = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Throw{
        exception
    })
}

//returns a overload statement
fn overload_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError>{


    //traditional operators
    let operations = vec![
        TokenType::PLUS, 
        TokenType::MINUS,
        TokenType::STAR,
        TokenType::SLASH,
        TokenType::BANG,
        TokenType::EQEQ,
        TokenType::GREATER,  
        TokenType::GREATEREQ,
        TokenType::LESSEQ,
    ];


    let line = get_statement_line(current_index, tokens);

    let token = get_current_token(current_index, tokens);
    //checks if operator is special character
    let mut is_op = false;
    if let TokenType::SPECIAL(_id) = &token.token_type {
        is_op = true; 
    }


    if !operations.contains(&token.token_type) && !is_op { 


        return ThorLangError::unexpected_token(TokenType::PLUS
                                                   , token.clone()
                                                   , get_previous_token(current_index, tokens).clone());


    }

    let operator = token.token_type.clone();
    consume_token(current_index, tokens);


    let mut token = match_token(current_index, tokens, TokenType::LPAREN)?;
    
    let mut operands : Vec<String> = vec![];
  
    while token.token_type != TokenType::RPAREN{
    
        match &token.token_type {
            TokenType::COMMA => {

            },
            TokenType::IDENTIFIER(name) => {
                operands.push(name.to_string())
            },
            _ => { 

                return ThorLangError::unexpected_token(
                    TokenType::IDENTIFIER("".to_string()),
                    token.clone(),
                    tokens.get(*current_index - 1).unwrap().clone()
                );
            }
        }
        token = consume_token(current_index, tokens);
    }

    match_token(current_index, tokens, TokenType::RPAREN)?;

    match_token(current_index, tokens, TokenType::LBRACE)?;

    let operation = statement(current_index, tokens)?;

    
    Ok(Statement::Overload{
        operator,
        operands,
        operation,
        line
    })
}

fn return_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let expression = expr(current_index, tokens)?;


    
    //consume and match the semicolon token
    let _ = match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Return { expression, line });
}

fn while_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens) ;

    match_token(current_index, tokens, TokenType::LPAREN)?;

    let condition = expr(current_index, tokens)?;


    match_token(current_index, tokens, TokenType::RPAREN)?;


    match_token(current_index, tokens, TokenType::LBRACE)?;

    let block = Box::new(statement(current_index, tokens)?);

    return Ok(Statement::While { condition, block, line });
}

fn if_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);

    match_token(current_index, tokens, TokenType::LPAREN)?;


    let condition = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::RPAREN)?;

    //consume the ")" and the "{" (two tokens)

    let token = match_token(current_index, tokens, TokenType::LBRACE)?;

    let then_branch = Box::new(statement(current_index, tokens)?);

    let mut else_branch = None;

    if token.token_type == TokenType::ELSE {
        consume_token(current_index, tokens);
        else_branch = Some(Box::new(statement(current_index, tokens)?));
    }

    return Ok(Statement::If {
        condition,
        then_branch,
        else_branch,
        line
    });
}

fn print_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let expression = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Print {
        expression,
        line
    });
}

fn function_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {

    let line = get_statement_line(current_index, tokens);
    let token = tokens.get(*current_index).unwrap();
    let function_name : String;

    if let TokenType::IDENTIFIER(str) = &token.token_type{
        function_name = str.to_string(); 
    } else {

        return ThorLangError::unexpected_token(TokenType::IDENTIFIER("".to_string()),
            token.clone(),
            tokens.get(*current_index - 1).unwrap().clone());

    }
    //consume the identifier token
    let mut token = &consume_token(current_index, tokens).clone();

    let mut args = vec![];

    token = match_token(current_index, tokens, TokenType::LPAREN)?;


    

    while token.token_type != TokenType::RPAREN {

        match &token.token_type {
            TokenType::IDENTIFIER(str) => {
                args.push(str.clone());
            }
            _ => (),
        }
        token = consume_token(current_index, tokens);
    }

    //consume rparen 
    consume_token(current_index, tokens);

    match_token(current_index, tokens, TokenType::LBRACE)?;

    let block = statement(current_index, tokens);

    Ok(Statement::Function {
        arguments: args,
        name: function_name,
        body: Box::new(block?),
        line
    })
}

//do turns expressions into statements;
fn do_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let expression = expr(current_index, tokens)?;
    
    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Do {
        expression,
        line
    });
}

fn declaration(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let name : String;
    let mut token = tokens.get(*current_index).unwrap().clone();



    if let TokenType::IDENTIFIER(str) = token.token_type {
        name = str;
    }
    else if let TokenType::SPECIAL(ref str) = token.token_type {
        
        return ThorLangError::unexpected_token(
            TokenType::SPECIAL(str.to_string()),
            token.clone(),
            tokens.get(*current_index - 1).unwrap().clone()
        );
    }
    else {

        return ThorLangError::unexpected_token(
            TokenType::IDENTIFIER("".to_string()),
            token.clone(),
            tokens.get(*current_index - 1).unwrap().clone()
        );
    }

    token = consume_token(current_index, tokens).clone();
    
    let mut init : Expression = Expression::Literal{
        literal : TokenType::NIL
    };


    if token.token_type == TokenType::EQ {
      
        token = consume_token(current_index, tokens).clone();
        init = expr(current_index, tokens)?;
    }

    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Variable {
        name,
        expression:init,
        line
    });
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Try {
        block : Vec<Statement>
    },
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



fn try_expression(current_index: &mut usize ,tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {

    
    match_token(current_index, tokens, TokenType::LBRACE)?;

    let block = statement(current_index, tokens)?;


    return Ok(Expression::Try {
        block
    })
}

fn expr(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {

    //here all block exprs go for example try
    
    let token = tokens.get(*current_index).unwrap();

    match token.token_type {
        TokenType::TRY => {
            consume_token(current_index, tokens);
            return try_expression(current_index, tokens);

        },
            _=> ()
    }
    

    assign(current_index, tokens)
}

fn assign(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let expression = eq(current_index, tokens);

    if let Some(token) = tokens.get(*current_index) {
        if token.token_type == TokenType::EQ {
            
            consume_token(current_index, tokens);

            let value = assign(current_index, tokens);

            return Ok(Expression::Assignment {
                target: Box::new(expression?),
                value: Box::new(value?),
            });
        }
    }

    return expression;
}

fn eq(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = comp(current_index, tokens)?;

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::EQEQ | TokenType::BANGEQ => {
                let operator = token.token_type.clone();
                
                consume_token(current_index, tokens);

                let right = comp(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right?),
                };
            }
            _ => break,
        }
    }

    Ok(expression)
}

fn comp(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = term(current_index, tokens)?;

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::GREATEREQ | TokenType::GREATER | TokenType::LESS | TokenType::LESSEQ => {
                let operator = token.token_type.clone();
                
                consume_token(current_index, tokens);

                let right = term(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right?),
                };
            },

            _ => break,
        }
    }

    Ok(expression)
}

fn term(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = factor(current_index, tokens)?;

    while let Some(token) = tokens.get(*current_index) {
        match token.token_type {
            TokenType::PLUS | TokenType::MINUS => {
                let operator = token.token_type.clone();
                
                consume_token(current_index, tokens);

                let right = factor(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right?),
                };
            }
            _ => break,
        }
    }

    Ok(expression)
}

fn factor(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = unary(current_index, tokens)?;

    while let Some(token) = tokens.get(*current_index) {
        match &token.token_type {
            TokenType::STAR | TokenType::SLASH => {
                let operator = token.token_type.clone();
                
                consume_token(current_index, tokens);

                let right = unary(current_index, tokens);
                expression = Expression::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right?),
                };
            },
            TokenType::SPECIAL(_id) => {
                let operator = token.token_type.clone();

                consume_token(current_index, tokens);
                let right = unary(current_index, tokens);
                expression = Expression::Binary{
                    left : Box::new(expression),
                    operator,
                    right: Box::new(right?)
                }
            }
            _ => break,
        }
    }

    Ok(expression)
}


fn unary(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    if let Some(token) = tokens.get(*current_index) {

        let operators = vec![
            TokenType::BANG,
            TokenType::MINUS,
            TokenType::PLUS,
            TokenType::STAR, 
            TokenType::SLASH,
            TokenType::GREATER,
            TokenType::GREATEREQ, 
            TokenType::LESS,
            TokenType::LESSEQ,
        ];

        if operators.contains(&token.token_type) {
            let operator = token.token_type.clone();
            consume_token(current_index, tokens);

            let right = unary(current_index, tokens);
            return Ok(Expression::Unary{
                operator, 
                right : Box::new(right?)
            })
        }

        if let TokenType::SPECIAL(_id) = &token.token_type{
            let operator = token.token_type.clone();
            consume_token(current_index, tokens);
            let right = unary(current_index, tokens);

            return Ok(Expression::Unary{
                operator, 
                right : Box::new(right?)
            })
        }

    }

    Ok(call(current_index, tokens)?)
}

// needs to check whether or not the expression returned in finishcall is a function itself, and if
// it is evaluate as well given more arguments/a call invocation i.e. "()"

fn call(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = primary(current_index, tokens)?;

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
                key: Box::new(key?),
            }
        }

        if current_token == TokenType::LPAREN {
            //consume the ( token
            
            consume_token(current_index, tokens);

            expression = finish_call(current_index, tokens, expression.clone())?;

            consume_token(current_index, tokens);
        }

        if current_token == TokenType::LBRACK {
            consume_token(current_index, tokens);

            let key = expr(current_index, tokens);

            expression = Expression::Retrieve {
                retrievee: Box::new(expression),
                key: Box::new(key?),
            };

            consume_token(current_index, tokens);
        }

        current_token = tokens.get(*current_index).unwrap().token_type.clone();
    }

    Ok(expression)
}

fn finish_call(current_index: &mut usize, tokens: &Vec<Token>, callee: Expression) -> Result<Expression, ThorLangError> {
    let mut arguments: Vec<Expression> = vec![];

    while let Some(token) = tokens.get(*current_index) {
        match &token.token_type {
            TokenType::RPAREN => {
                return Ok(Expression::Call {
                    callee: Box::new(callee),
                    arguments,
                    paren: token.clone(),
                })
            }
            TokenType::COMMA => {
                //consume the comma token

                consume_token(current_index, tokens);
            }
            TokenType::IDENTIFIER(_str) => {
                let argument = expr(current_index, tokens);

                arguments.push(argument?);
            }

            _ => {
                let argument = expr(current_index, tokens);

                arguments.push(argument?);
            }
        }
    }


    return ThorLangError::unexpected_token::<Expression>(
        TokenType::RPAREN,
        tokens.get(*current_index).unwrap().clone(),
        tokens.get(*current_index - 1).unwrap().clone()
    )
}

fn primary(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    if let Some(token) = tokens.get(*current_index) {
        consume_token(current_index, tokens);
        match &token.token_type {
            TokenType::IDENTIFIER(str) => Ok(Expression::Identifier {
                name: str.to_string(),
            }),
            TokenType::TRUE => Ok(Expression::Literal {
                literal: TokenType::TRUE,
            }),
            TokenType::FALSE => Ok(Expression::Literal {
                literal: TokenType::FALSE,
            }),
            TokenType::NUMBER(num) => Ok(Expression::Literal {
                literal: TokenType::NUMBER(num.to_string())
            }),
            TokenType::STRING(str) => Ok(Expression::Literal {
                literal: TokenType::STRING(str.to_string())
            }),
            TokenType::NIL => Ok(Expression::Literal {
                literal: TokenType::NIL
            }),
            TokenType::LBRACK => {
                if token.token_type == TokenType::RBRACK {
                    return Ok(Expression::Array { values: vec![] });
                }

                let mut array: Vec<Expression> = vec![];

                array.push(expr(current_index, tokens)?);

                while let Some(token) = tokens.get(*current_index) {
                    match token.token_type {
                        TokenType::RBRACK => {
                            consume_token(current_index, tokens);
                            return Ok(Expression::Array { values: array });
                        }
                        TokenType::COMMA => {
                            consume_token(current_index, tokens); 
                            let value = expr(current_index, tokens);

                            array.push(value?);
                        }
                        _ => {

                            return ThorLangError::unexpected_token_of_many(
                                vec![TokenType::RBRACK, TokenType::COMMA],
                                token.clone(),
                                tokens.get(*current_index - 1).unwrap().clone()
                            )
                        },
                    }
                }

                Ok(Expression::Array { values: array })
            }
            TokenType::LPAREN => {
                let expression = expr(current_index, tokens);
                if let Some(token) = tokens.get(*current_index) {
                    if token.token_type == TokenType::RPAREN {
                        consume_token(current_index, tokens);
                    } else {
                        return ThorLangError::unexpected_token(
                            TokenType::RPAREN,
                            token.clone(),
                            tokens.get(*current_index - 1).unwrap().clone()
                        )
                    }
                }
                Ok(Expression::Grouping {
                    inner: Box::new(expression?),
                })
            }
            _ => Ok(Expression::Literal {
                literal: TokenType::NIL,
            }),
        }
    } else {
        Ok(Expression::Literal {
            literal: TokenType::NIL,
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ThorLangError> {
    let mut current_index: usize = 0;
    Ok(statement(&mut current_index, &tokens)?)
}
