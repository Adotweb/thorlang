use crate::{TokenType, LiteralType, Expression, Statement};
use std::collections::HashMap;

//eval statements

pub fn eval_statement(stmts : Vec<Statement>, global_values : Option<&mut HashMap<String, Value>>){
    
    let globals = &mut HashMap::new();

    let mut value_map = global_values.unwrap_or(globals).clone();

    for stmt in stmts {
        match stmt {
            Statement::Block { statements } => {

                eval_statement(statements, Some(&mut value_map));
               
            },
            Statement::Print { expression } => {
            
                let result = eval(&expression.unwrap(), &mut value_map);

                match result.value_type {
                    ValueType::STRING => {
                        println!("{:#?}", result.string_value.unwrap())
                    },
                    ValueType::NUMBER => {
                        println!("{:#?}", result.number_value.unwrap())
                    },
                    ValueType::BOOL => {
                        println!("{:#?}", result.bool_value.unwrap())
                    },
                    ValueType::NIL => {
                        println!("{:#?}", ValueType::NIL)
                    },

                }

            },
            Statement::Variable { name, expression } => {
                
                let val = eval(&expression.unwrap(), &mut value_map);

                value_map.insert(name, val);
            }

        }      
    }


}



//eval expressions

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ValueType {
    STRING, NUMBER, BOOL, NIL
}

#[derive(PartialEq, Debug, Clone)]
pub struct Value {
    pub value_type : ValueType,
    pub string_value : Option<String>,
    pub number_value : Option<f64>,
    pub bool_value : Option<bool>, 
    pub is_nil : bool
}

impl Default for Value {
    fn default() -> Value {
        Value {
            value_type:ValueType::NIL, 
            string_value:None,
            number_value:None,
            bool_value:None,
            is_nil:false
        }
    }
}

fn eval_unary(operator : TokenType, right : &Expression, value_map : &mut HashMap<String, Value>)  -> Value{

    let r = eval(right, value_map);      
    match operator {

        //only negate logically when is bool
        TokenType::BANG => {
            if r.value_type == ValueType::BOOL{
                return Value {
                    value_type : ValueType::BOOL, 
                    bool_value : Some(!r.bool_value.unwrap()), 
                    ..Value::default()
                }
            }  else {
                panic!("can only logically negate bools")
            }
        },
        //only negate arithmetically when is number
        TokenType::MINUS => {

            if r.value_type == ValueType::NUMBER {
                
                return Value{
                    value_type:ValueType::NUMBER,
                    number_value : Some(r.number_value.unwrap() * -1.0_f64),
                    ..Value::default()
                }

            } else {
                panic!("can only negate numbers")
            }
        },
        _ => {
            panic!("unary operation not allowed")
        }
    }

}

fn eval_literal (literal : LiteralType) -> Value{

        //turn literaltype into value wrapped in value_type
        match literal {
            LiteralType::NIL => {
                return Value{
                    value_type : ValueType::NIL,
                    is_nil:true, ..Value::default()
                }
            }, 
            LiteralType::BOOL { value } => {
                return Value{
                    value_type : ValueType::BOOL,
                    is_nil:false, bool_value : Some(value), ..Value::default()
                }
            }, 
            LiteralType::NUMBER { value } => {
                return Value{
                    value_type : ValueType::NUMBER,
                    is_nil:false, number_value : Some(value), ..Value::default()
                }
            }, 
            LiteralType::STRING { value } => {
                return Value{
                    value_type : ValueType::STRING,
                    is_nil : false, string_value : Some(value), ..Value::default()
                }
            }

        } 

}



fn check_type_equality(value_1 : &Value, value_2 : &Value, expected_type : ValueType) -> bool{
    

    if value_1.value_type == value_2.value_type && value_1.value_type == expected_type {
        return true;
    } else {
        return false
    }

}

fn eval_binary(left : &Expression, operator : TokenType, right : &Expression, value_map : &mut HashMap<String, Value>) -> Value {

    let l = eval(left, value_map);
    let r = eval(right, value_map);



    match operator {


        //almost every binary operation can only be applied to values of the same type 
        //check type equality, checks if two types are of the same type and if they are equal to
        //some expected_type numbers in the case of division for example
        //
        //if the two numbers are not equal and of the expected type the programm will throw
        TokenType::PLUS => {
             if check_type_equality(&l, &r, ValueType::STRING) {
                return Value{
                    value_type: ValueType::STRING,
                    string_value : Some(l.string_value.unwrap() + &r.string_value.unwrap()),
                    ..Value::default()
                } 
             }
             if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() + r.number_value.unwrap()),
                    ..Value::default()
                } 
             }
             panic!("can only add strings and numbers")
        }, 
        TokenType::MINUS => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() - r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only subtract numbers")
        }, 
        TokenType::STAR => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() * r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only multiply numbers")
        }, 
        TokenType::SLASH => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type: ValueType::NUMBER,
                    number_value : Some(l.number_value.unwrap() / r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only divide numbers")
        }, 
        TokenType::LESSEQ => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() <= r.number_value.unwrap()),
                    ..Value::default()
                } 
             }


             panic!("can only compare numbers")
        },
        TokenType::LESS => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() < r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 
        TokenType::GREATEREQ => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() >= r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 
        TokenType::GREATER => {
            if check_type_equality(&l, &r, ValueType::NUMBER) {
                return Value{
                    value_type : ValueType::BOOL,
                    bool_value : Some(l.number_value.unwrap() > r.number_value.unwrap()),
                    ..Value::default()
                } 
             }

             panic!("can only compare numbers")
        }, 

        //equality doesnt need a typecheck, if the Value object is the same, two values are the
        //same
        TokenType::EQEQ => {
            return Value{
                value_type: ValueType::BOOL, 
                bool_value : Some(l == r),
                ..Value::default()
            } 
        }, 
        TokenType::BANGEQ => {
            return Value{
                value_type: ValueType::BOOL, 
                bool_value : Some(l != r),
                ..Value::default()
            }
        }


        _ => ()
    }


    return Value::default()
}

pub fn eval(expr : &Expression, value_map : &mut HashMap<String, Value>) -> Value{

    //recursivley traverses the tree.
    match expr {

        Expression::Assignment { name, value } => {
          
            let eval_value = eval(value, value_map);
            value_map.insert(name.to_string(), eval_value.clone()).unwrap();
           
            return eval_value
        },
        Expression::Identifier { name } => {
            let value = value_map.get(name).expect("no such variable found");
            return value.clone()
        },
        Expression::Unary { operator, right } => {
            return eval_unary(*operator, &right, value_map)
        }, 
        Expression::Literal { literal } => {
            return eval_literal(literal.clone())
        }, 
        Expression::Grouping { inner } => {
            return eval(&inner, value_map) 
        }, 
        Expression::Binary { left, operator, right } => {
            return eval_binary(&left, *operator, &right, value_map)
        }
    } 

}
