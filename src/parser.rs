use crate::{Token, TokenType, ThorLangError};

//the different kinds of statments and what data they hold
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Throw{
        exception : Expression,
        throw_token_index : usize
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

//returns the previous token in the tokenlist
fn get_previous_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> &'a Token{
    let current_token = tokens.get(*current_index - 1).unwrap();
    current_token
}


//returns the current token in the tokenlist
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


//returns the line (in the code) on which current token is
fn get_statement_line<'a>(current_index: &mut usize, tokens: &'a Vec<Token>) -> i32{
    let line = tokens.get(*current_index).unwrap();


    line.line
}


//consumes the current token and checks if it is of the desired token type and throws an unexpected
//token if it is not and returns the next token else
fn match_token<'a>(current_index: &mut usize, tokens: &'a Vec<Token>, token_type : TokenType) -> Result<&'a Token, ThorLangError>{
    
    let prev_token =  get_previous_token(current_index, tokens);
    let token = get_current_token(current_index, tokens);

    if token.token_type != token_type{

        if let Err(err) = ThorLangError::unexpected_token::<Statement>(token_type, *current_index){
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
                ret = do_statement(current_index, tokens);
            }
        }

        statements.push(ret?);
    }

    return Ok(statements);
}



//this is not implmenented yet, but will replace "return throw("");" in the future
fn throw_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError>{ 
    let throw_token_index = *current_index - 1; 
    let exception = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Throw{
        exception,
        throw_token_index
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

            
    //if the op character is not special and not a traditional op character this will throw as we
    //cant overload things like "a" (yet however);
    if !operations.contains(&token.token_type) && !is_op { 
        return ThorLangError::unexpected_token(TokenType::PLUS, *current_index);

    }

    let operator = token.token_type.clone();
    consume_token(current_index, tokens);


    //the part below adds the operand names and the operation information
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

                return ThorLangError::unexpected_token(TokenType::IDENTIFIER("".to_string()),*current_index);
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

//this is a simple statement (like throw, print, do...)
//this means that it has only one thing to do and that is to encapsulate the data 
fn return_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let expression = expr(current_index, tokens)?; 
    //consume and match the semicolon token
    let _ = match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Return { expression, line });
}


//again rather simple just check if the right things stand at the right places and throw else, when
//done just return a while statement object
fn while_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens) ;

    match_token(current_index, tokens, TokenType::LPAREN)?;

    let condition = expr(current_index, tokens)?;


    match_token(current_index, tokens, TokenType::RPAREN)?;


    match_token(current_index, tokens, TokenType::LBRACE)?;

    let block = Box::new(statement(current_index, tokens)?);

    return Ok(Statement::While { condition, block, line });
}


//like while but has the potential to have an else part
fn if_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);

    match_token(current_index, tokens, TokenType::LPAREN)?;


    let condition = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::RPAREN)?;

    //consume the ")" and the "{" (two tokens)

    let token = match_token(current_index, tokens, TokenType::LBRACE)?;

    let then_branch = Box::new(statement(current_index, tokens)?);

    let mut else_branch = None;

    //if the parser finds else after the closing bracket of the if block an else block will be
    //inserted in the statement else not

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


//simple statement
fn print_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let expression = expr(current_index, tokens)?;

    match_token(current_index, tokens, TokenType::SEMICOLON)?;

    return Ok(Statement::Print {
        expression,
        line
    });
}


//this creates a function checks if the right things are in the right place and throws otherwise,
//also it 
fn function_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {

    let line = get_statement_line(current_index, tokens);
    let token = tokens.get(*current_index).unwrap();
    let function_name : String;

    if let TokenType::IDENTIFIER(str) = &token.token_type{
        function_name = str.to_string(); 
    } else {

        return ThorLangError::unexpected_token(TokenType::IDENTIFIER("".to_string()), *current_index)

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


//variables 
//"let a = 10;"
fn declaration(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Statement, ThorLangError> {
    let line = get_statement_line(current_index, tokens);
    let name : String;
    let mut token = tokens.get(*current_index).unwrap().clone();



    if let TokenType::IDENTIFIER(str) = token.token_type {
        name = str;
    }
    else if let TokenType::SPECIAL(ref str) = token.token_type {
        
        return ThorLangError::unexpected_token(TokenType::SPECIAL(str.to_string()),*current_index);
    }
    else {

        return ThorLangError::unexpected_token(
            TokenType::IDENTIFIER("".to_string()),*current_index);
    }
    
    let literal_token_index = current_index.clone();

    token = consume_token(current_index, tokens).clone();
    
    let mut init : Expression = Expression::Literal{
        literal : TokenType::NIL,
        literal_token_index
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


//unlike parser errors we know that the tokenlist works in here and we can point to the token that
//has an error 
//this means it sufficces to just put in the index to the wanted token
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Try {
        block : Vec<Statement>
    },
    Identifier {
        name: String,
        identifier_token_index : usize
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
        literal_token_index : usize
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
        eq_token_index : usize
    },
    Array {
        values: Vec<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren_token_index: usize,
        arguments: Vec<Expression>,
    },
    Retrieve {
        retrievee: Box<Expression>,
        key: Box<Expression>,
        lbrack_token_index : usize
    },
    FieldCall {
        callee: Box<Expression>,
        key: Box<Expression>,
        dot_token_index : usize
    },
}

//all of the below are part of the precedence hierarchy

//expression to listen for errors and make exceptions from them (errors that dont make the program
//halt)
//when the try block executes without any errors we return the value of the 
fn try_expression(current_index: &mut usize ,tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {

    
    match_token(current_index, tokens, TokenType::LBRACE)?;

    let block = statement(current_index, tokens)?;


    return Ok(Expression::Try {
        block
    })
}

//matches every expression
//precedence works on the deepest possible match i.e. 
//the more specific an expression the deeper the function goes
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


//highest order of operational precedence i.e. the highest functionaing operator
fn assign(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let expression = eq(current_index, tokens);

    if let Some(token) = tokens.get(*current_index) {
        if token.token_type == TokenType::EQ {
                
            let eq_token_index = current_index.clone();

            consume_token(current_index, tokens);

            let value = assign(current_index, tokens);

            return Ok(Expression::Assignment {
                target: Box::new(expression?),
                value: Box::new(value?),
                eq_token_index
            });
        }
    }

    return expression;
}


//equality comparison is one level of precedence deeper (== or !=)
fn eq(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    let mut expression = comp(current_index, tokens)?;

    
    //all the operations work almost the same, we just use an expression on the left and then an
    //expression on the right and insert them together with the given operator
    //what makes precedence work is the order in which these functions are executed (this recursive
    //descent)
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


//numerical comparison is one level of precedence deeper
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

//then comes math (first + or -)
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

//then multiplication (* and /) also special characters meaning that custom operators have the same
//precedence as multiplication
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

//unary operations (- ! and special characters as well when their arity is 1) are recursive 
//so that we can chain them !!!!true;
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


    let mut current_token = get_current_token(current_index, tokens);

    while current_token.token_type == TokenType::LPAREN
        || current_token.token_type == TokenType::LBRACK
        || current_token.token_type == TokenType::DOT
    {
        if current_token.token_type == TokenType::DOT {
             
            let dot_token_index = current_index.clone();
            consume_token(current_index, tokens);

            let key = primary(current_index, tokens);

            expression = Expression::FieldCall {
                callee: Box::new(expression),
                key: Box::new(key?),
                dot_token_index  
            }
        }

        if current_token.token_type == TokenType::LPAREN {
            //consume the ( token
            
            consume_token(current_index, tokens);

            expression = finish_call(current_index, tokens, expression.clone())?;

            consume_token(current_index, tokens);
        }

        if current_token.token_type == TokenType::LBRACK {
            let lbrack_token_index = current_index.clone();
            consume_token(current_index, tokens);

            let key = expr(current_index, tokens);

            expression = Expression::Retrieve {
                retrievee: Box::new(expression),
                key: Box::new(key?),
                lbrack_token_index
            };

            consume_token(current_index, tokens);
        }

        current_token = get_current_token(current_index, tokens);
    }

    Ok(expression)
}


//helper function to return all the stuff for a function call
fn finish_call(current_index: &mut usize, tokens: &Vec<Token>, callee: Expression) -> Result<Expression, ThorLangError> {
    let mut arguments: Vec<Expression> = vec![];

    while let Some(token) = tokens.get(*current_index) {
        match &token.token_type {
            TokenType::RPAREN => {
                return Ok(Expression::Call {
                    callee: Box::new(callee),
                    arguments,
                    paren_token_index : current_index.clone()
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
        TokenType::RPAREN,*current_index)
}


//the lowest precedence, returns the "atoms" , numbers, strings, arrays, ... and variables
//maybe objects in the future
fn primary(current_index: &mut usize, tokens: &Vec<Token>) -> Result<Expression, ThorLangError> {
    if let Some(token) = tokens.get(*current_index) {
        consume_token(current_index, tokens);
        match &token.token_type {
            TokenType::IDENTIFIER(str) => Ok(Expression::Identifier {
                name: str.to_string(),
                identifier_token_index : current_index.clone()
            }),
            TokenType::TRUE => Ok(Expression::Literal {
                literal: TokenType::TRUE,
                literal_token_index : current_index.clone()
            }),
            TokenType::FALSE => Ok(Expression::Literal {
                literal: TokenType::FALSE,
                literal_token_index : current_index.clone()
            }),
            TokenType::NUMBER(num) => Ok(Expression::Literal {
                literal: TokenType::NUMBER(num.to_string()),
                literal_token_index : current_index.clone()
            }),
            TokenType::STRING(str) => Ok(Expression::Literal {
                literal: TokenType::STRING(str.to_string()),
                literal_token_index : current_index.clone()
            }),
            TokenType::NIL => Ok(Expression::Literal {
                literal: TokenType::NIL,
                literal_token_index : current_index.clone()
            }),
            TokenType::LBRACK => {
                //when encountering an [ we start an array and the following are just expressions
                //seperated by commas (or spaces)

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
                                vec![TokenType::RBRACK, TokenType::COMMA], *current_index
                            )
                        },
                    }
                }

                Ok(Expression::Array { values: array })
            }
            //in case of an lparen we have a grouped expression (can be used to use + at a
            //precedence level lower than *)
            TokenType::LPAREN => {
                let expression = expr(current_index, tokens);
                if let Some(token) = tokens.get(*current_index) {
                    if token.token_type == TokenType::RPAREN {
                        consume_token(current_index, tokens);
                    } else {
                        return ThorLangError::unexpected_token(
                            TokenType::RPAREN,
                            *current_index
                        )
                    }
                }
                Ok(Expression::Grouping {
                    inner: Box::new(expression?),
                })
            }
            _ => Ok(Expression::Literal {
                literal: TokenType::NIL,
                literal_token_index : current_index.clone()
            }),
        }
    } else {
        Ok(Expression::Literal {
            literal: TokenType::NIL,
            literal_token_index : current_index.clone()
        })
    }
}

//puts all these things above into a single function 
//returns the list of asts
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, ThorLangError> {
    let mut current_index: usize = 0;
    Ok(statement(&mut current_index, &tokens)?)
}
