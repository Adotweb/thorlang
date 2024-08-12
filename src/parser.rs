use crate::{Token, LiteralType, TokenType};

#[derive(Debug, Clone)]
pub enum Statement {
    Print {
        expression : Option<Expression>
    }, 
    Do {
        expression : Option<Expression>
    },
    Variable {
        name : String, 
        expression : Option<Expression>
    }, 
    Block {
        statements : Vec<Statement>
    }, 
    If {
        condition : Option<Expression>, 
        then_branch : Option<Box<Vec<Statement>>>,
        else_branch : Option<Box<Vec<Statement>>>
    }, 
    While{
        condition : Option<Expression>, 
        block : Option<Box<Vec<Statement>>>
    }, 
}

//generates a list of statments and returns the global "program" list (list of ASTs) that will be
//individually executed in eval_stmts later (in the context of a global "program")



pub fn statement(current_index: &mut usize, tokens: &Vec<Token>) -> Vec<Statement>{

    let mut statements = vec![];

    while let Some(token) = tokens.get(*current_index){


        match token.token_type  { 
            TokenType::PRINT => {
                //consume the token
                *current_index += 1;
                statements.push(print_statement(current_index, tokens))
            }, 
            TokenType::DO => {
                *current_index += 1;
                statements.push(do_statement(current_index, tokens))
            },
            TokenType::IF => {
                *current_index += 1;
                statements.push(if_statement(current_index, tokens))
            },
            TokenType::WHILE => {
                *current_index += 1;
                statements.push(while_statement(current_index, tokens))
            },
            TokenType::LET => {
                *current_index += 1;
                statements.push(declaration(current_index, tokens))
            },
            TokenType::LBRACE => {
                *current_index += 1;

                statements.push(Statement::Block{
                    statements : statement(current_index, tokens)
                })
            },
            TokenType::RBRACE => {
                *current_index += 1;
                return statements;
            },
            TokenType::ELSE => {
                //dont consume the else token as it is needed one layer of recursion above
                return statements
            }, 

            TokenType::EOF => return statements,


            _ => {

                //tries to automatically run and expressions when just written. Works semantically
                //the same as "do expression";
                statements.push(do_statement(current_index, tokens));
            }
                
        }


        
    }


    return statements
}

fn while_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement{
    if tokens.get(*current_index).unwrap().token_type != TokenType::LPAREN{
        panic!("expected ( after while on line {:?}", tokens.get(*current_index).unwrap().line)
    } 

    *current_index += 1;

    let condition = Some(expr(current_index, tokens));

    if tokens.get(*current_index).unwrap().token_type != TokenType::RPAREN {
        panic!("expected ) after condition on line {:?}", tokens.get(*current_index).unwrap().line)
    }

    //consume the ")" and the "{" tokens;
    *current_index += 2;

    let block = Some(Box::new(statement(current_index, tokens)));

    

    return Statement::While{
        condition, 
        block
    }
}

fn if_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement{
    
    if tokens.get(*current_index).unwrap().token_type != TokenType::LPAREN {
        panic!("expected ( after if on line {:?}", tokens.get(*current_index).unwrap().line)
    } 
    //consume the paren token
    *current_index += 1;
    let condition = Some(expr(current_index, tokens));
    
    
    if tokens.get(*current_index).unwrap().token_type != TokenType::RPAREN {
        panic!("expected ) after if condition on line {:?}", tokens.get(*current_index).unwrap().line);
    }

    //consume the ")" and the "{" (two tokens)
    *current_index += 2;

    let then_branch = Some(Box::new(statement(current_index, tokens)));
   

    let mut else_branch = None;

    if tokens.get(*current_index).unwrap().token_type == TokenType::ELSE {
        
        //consume the else token
        *current_index += 1;
        else_branch = Some(Box::new(statement(current_index, tokens)));
    } 

    return Statement::If{
        condition, 
        then_branch, 
        else_branch
    }
}

fn print_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement{
    

    let expression = expr(current_index, tokens);



    if tokens.get(*current_index).unwrap().token_type != TokenType::SEMICOLON {
        panic!("expected ; after {:?}", tokens.get(*current_index));
    }

    //consume token (is ";" because throws otherwise) and move on.
    *current_index += 1;

    return Statement::Print{
        expression: Some(expression)
    }

}


//do turns expressions into statements;
//assignment only works with do at the moment
fn do_statement(current_index: &mut usize, tokens: &Vec<Token>) -> Statement{
    
    let expression = expr(current_index, tokens);

    if tokens.get(*current_index).unwrap().token_type != TokenType::SEMICOLON {
        panic!("expected ; after {:?}", tokens.get(*current_index).unwrap().token_type)
    }
    
    //consume the ; token

        
    *current_index += 1;    


    return Statement::Do{
        expression: Some(expression)
    }

}

fn declaration(current_index: &mut usize, tokens: &Vec<Token>) -> Statement{
  
    let name = tokens.get(*current_index).unwrap().clone();

    if name.token_type != TokenType::IDENTIFIER {
        panic!("exptected a variable name")
    }

    *current_index += 1;

    let mut init = Expression::Literal{
         literal : LiteralType::NIL
    };


    if tokens.get(*current_index).unwrap().token_type == TokenType::EQ {

        *current_index += 1;

        init = expr(current_index, tokens);
    }


    if tokens.get(*current_index).unwrap().token_type != TokenType::SEMICOLON {
        panic!("exptected ; after value {:?}", tokens.get(*current_index));
    }

    *current_index += 1;


    //consume token (is ";" because throws otherwise) and move on.

    return Statement::Variable{
        name : name.string.unwrap(),
        expression : Some(init)
    }
    
}




#[derive(Debug, Clone)]
pub enum Expression {
    Identifier{
        name : String, 
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
        literal: LiteralType,
    },
    Assignment {
        name : String, 
        value : Box<Expression>
    },
    Call {
        callee : Box<Expression>, 
        paren : Token, 
        arguments : Vec<Expression>
    }
}

fn expr(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {


    assign(current_index, tokens)
}

fn assign(current_index: &mut usize, tokens: &Vec<Token>) -> Expression{

    let expression = eq(current_index, tokens);


    if let Some(token) = tokens.get(*current_index){

        if token.token_type == TokenType::EQ{
           
            //consume the eq token
            *current_index += 1;

            let value = assign(current_index, tokens);
        
            if let Expression::Identifier { name } = expression  {
                
                return Expression::Assignment{
                    name, 
                    value : Box::new(value)
                }

            } else {
                panic!("incalid assigment target")
            }

        }

    }


    return expression
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

    call(current_index, tokens)
}

// needs to check whether or not the expression returned in finishcall is a function itself, and if
// it is evaluate as well given more arguments/a call invocation i.e. "()"

fn call(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {

    let mut expression = primary(current_index, tokens);

    
       
    while tokens.get(*current_index).unwrap().token_type == TokenType::LPAREN {
        
        //consume the ( token
        *current_index += 1;  


        
        expression = finish_call(current_index, tokens, expression.clone());


        *current_index += 1;

    }

    
        

    expression
}

fn finish_call(current_index: &mut usize, tokens: &Vec<Token>, callee : Expression) -> Expression {


    let mut arguments : Vec<Expression> = vec![];


    while let Some(token) = tokens.get(*current_index){
       

        match token.token_type {
            TokenType::RPAREN => {


                return Expression::Call{
                    callee : Box::new(callee), 
                    arguments,
                    paren : token.clone()
                }
            },
            TokenType::COMMA => {

                //consume the comma token
              

                *current_index += 1; 
            },
            TokenType::IDENTIFIER => {
                
                let argument = expr(current_index, tokens);


                arguments.push(argument);


            }
             

            _ => panic!("error in argument list on line {:?}", tokens.get(*current_index).unwrap())

        }
        
    }

    panic!("no delimiter in argument list on line {:?}", tokens.get(*current_index).unwrap().line)

}

fn primary(current_index: &mut usize, tokens: &Vec<Token>) -> Expression {
    if let Some(token) = tokens.get(*current_index) {
        *current_index += 1;
        match &token.token_type {
            TokenType::IDENTIFIER => Expression::Identifier{
                name : token.string.clone().unwrap()
            },
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

pub fn parse(tokens: Vec<Token>) -> Vec<Statement>{
    let mut current_index: usize = 0;
    statement(&mut current_index, &tokens)
}

