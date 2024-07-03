use crate::{Token, Literal, LiteralType, TokenType};

#[derive(Debug)]
pub enum Expression{    
    Binary { left : Box<Expression>, operator : TokenType, right : Box<Expression>},
    Unary { operator : TokenType, right : Box<Expression>},
    Grouping { inner : Box<Expression>},
    Literal { literal: LiteralType },
}

fn expr(current_index : usize, tokens : &Vec<Token>) -> Expression{


    return eq(current_index, tokens)
}

fn eq(current_index : usize, tokens : &Vec<Token>) -> Expression{


    let mut current_token_type = tokens.get(current_index).unwrap().token_type;
 
    let mut expression = comp(current_index, tokens);


    let mut iterator = 0;
    while current_token_type != TokenType::EOF {
     
        match current_token_type {

            TokenType::EQEQ | TokenType::BANGEQ => {
            
                let operator = tokens.get(current_index + iterator).unwrap().token_type;
             
                let right = comp(current_index + iterator + 1, tokens); 

                expression = Expression::Binary{
                    right : Box::new(right), 
                    operator, 
                    left : Box::new(expression)
                } 
            },

            _ => (),
        }    


        iterator += 1;  
        current_token_type = tokens.get(current_index + iterator).unwrap().token_type;
    }

    

    return expression
}

//return expression and consumed tokens
fn comp(current_index : usize, tokens : &Vec<Token>) -> Expression{

    let mut expression = term(current_index, tokens);

    let mut iterator = 0;
    let mut current_token_type = &tokens.get(current_index).unwrap().token_type;


    while *current_token_type != TokenType::EOF {
        match current_token_type {
    
            TokenType::GREATEREQ | TokenType::GREATER | TokenType::LESS | TokenType::LESSEQ => {

                let operator = tokens.get(current_index + iterator).unwrap().token_type;
                let right = term(current_index + iterator + 1, tokens);

                expression = Expression::Binary{
                    operator : operator,
                    right: Box::new(right),
                    left: Box::new(expression)
                }

            }

            _ => ()
        } 


        iterator += 1;
        current_token_type = &tokens.get(current_index + iterator).unwrap().token_type;
    }

    return expression
}


fn term(current_index : usize, tokens : &Vec<Token>) -> Expression{
    let mut expression = factor(current_index, tokens);


    let mut iterator = 0;
    let mut current_token_type = tokens.get(current_index ).unwrap().token_type;


    while current_token_type != TokenType::EOF{

        match current_token_type {

            TokenType::PLUS | TokenType::MINUS => {
                let operator = tokens.get(current_index + iterator).unwrap().token_type;
                let right = factor(current_index + iterator + 1, tokens);
                
                expression = Expression::Binary{
                    operator: operator,
                    right : Box::new(right),
                    left : Box::new(expression)
                }
            },

            _ =>()
        }


        iterator += 1;
        current_token_type = tokens.get(current_index + iterator).unwrap().token_type;
    }


    return expression
}

fn factor(current_index : usize, tokens : &Vec<Token>) -> Expression{
    
    let mut expression = unary(current_index, tokens);

    let mut iterator = 0;
    let mut current_token_type = tokens.get(current_index).unwrap().token_type;


    while current_token_type != TokenType::EOF{

        println!("{:?}", tokens );

        match current_token_type {
           

            TokenType::STAR | TokenType::SLASH => {
                    let operator = tokens.get(current_index + iterator).unwrap().token_type;
                    let right = unary(current_index + iterator + 1, tokens);
                
                    expression = Expression::Binary{
                        operator: operator,
                        right : Box::new(right),
                        left : Box::new(expression)
                    } 
            },

            _ => ()

        }

        iterator += 1;
        current_token_type = tokens.get(current_index + iterator).unwrap().token_type;
    }


    return expression
}

fn unary(current_index : usize, tokens : &Vec<Token>) -> Expression{
    let current_token_type = tokens.get(current_index).unwrap().token_type;


    if current_token_type != TokenType::EOF{

        match current_token_type {
            TokenType::BANG | TokenType::MINUS => {
                let operator = tokens.get(current_index).unwrap().token_type;
                let right = unary(current_index + 1, tokens);


                return Expression::Unary{
                    operator:operator,
                    right:Box::new(right)
                }

            },
            _ => ()
        }

    }


    return primary(current_index, tokens)
}

fn scan_between_paren(current_index : usize, tokens : &Vec<Token>) -> Expression{

    let mut iterator = 0;

    let mut inner_tokens : Vec<Token> = vec![];


    let mut current_token = tokens.get(current_index + iterator).unwrap();
    let mut current_token_type = current_token.token_type;



    while current_token_type != TokenType::RPAREN{
        
        

        if current_token_type == TokenType::SEMICOLON || current_token_type == TokenType::EOF {
            panic!("no closing paranthesis") ;
        }
        
        iterator += 1;
        
        inner_tokens.push(current_token.clone());

        current_token = tokens.get(current_index + iterator).unwrap();

        current_token_type = current_token.token_type;
        

    }

    inner_tokens.push(Token{
            token_type:TokenType::EOF,
            literal:None,
            string:None,
            line:None
    });

   
    
    
    

    println!(" ###{:?}", tokens);

    let inner = expr(0, &inner_tokens);

    return Expression::Grouping{
        inner : Box::new(inner)
    }
    
}

fn primary(current_index : usize, tokens : &Vec<Token>) -> Expression{

    let current_token = tokens.get(current_index).unwrap();
    let current_token_type = &current_token.token_type;
    
    let current_token_value = current_token.string.clone();





    match current_token_type {


        TokenType::TRUE => {
            return Expression::Literal{
                literal:LiteralType::BOOL{
                    value: true
                }
            }     
        },
        TokenType::FALSE => {
            return Expression::Literal{
                literal:LiteralType::BOOL{
                    value: false
                }
            }     
        },
        TokenType::NUMBER => {
            return Expression::Literal{
                literal:LiteralType::NUMBER{
                    value: current_token_value.unwrap().parse().unwrap()

                }
            }     
        },
        TokenType::STRING => {
            return Expression::Literal{
                literal:LiteralType::STRING{
                    value: current_token_value.unwrap()
                }
            }     
        },
        TokenType::NIL => {
            return Expression::Literal{
                literal:LiteralType::NIL
            }     
        },
    
        TokenType::LPAREN => {
            
            //check if closing paranthesis are found, if so return the subarray *between* the pars.
            //if no closing pars throw
               
            let next = tokens.iter().skip(current_index);


            let inner = scan_between_paren(current_index + 1, tokens);

            


            return Expression::Grouping{
                inner:Box::new(inner)
            }
        },
        
        _=> return Expression::Literal{
            literal:LiteralType::NIL
        }
    }

}



pub fn parse(tokens : Vec<Token>) -> Expression{

    let current_index : usize = 0;
    
    let expr = expr(0, &tokens) ;


    expr
}
