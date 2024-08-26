#[derive(Clone, Debug)]
pub enum ThorLangError<'a>{
    ParsingError(&'a str),
    EvalError(&'a str),
}
