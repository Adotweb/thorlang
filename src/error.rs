#[derive(Clone, Debug)]
pub enum ThorLangError{
    ParsingError(String),
    EvalError(String),
}
