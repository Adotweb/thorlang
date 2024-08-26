#[derive(Clone, Debug, PartialEq)]
pub enum ThorLangError{
    ParsingError(String),
    EvalError(String),
}
