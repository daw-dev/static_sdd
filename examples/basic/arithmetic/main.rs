use semasia::*;

#[grammar]
mod expressions {
    use super::*;

    #[start_symbol]
    #[non_terminal]
    pub type Expression = usize;

    #[non_terminal]
    pub type Term = usize;

    #[non_terminal]
    pub type Factor = usize;

    #[token(regex = r"\d+")]
    pub type Number = usize;

    #[token("+")]
    pub struct Plus;

    #[token("*")]
    pub struct Times;

    #[token("(")]
    pub struct OpenPar;

    #[token(")")]
    pub struct ClosedPar;

    production!(Addition, Expression -> (Expression, Plus, Term), |(e, _ ,t)| e + t);
    production!(NoAddition, Expression -> Term);
    production!(Multiplication, Term -> (Term, Times, Factor), |(t, _, f)| t * f);
    production!(NoMultiplication, Term -> Factor);
    production!(Parenthesis, Factor -> (OpenPar, Expression, ClosedPar), |(_, e, _)| e);
    production!(ActualNumber, Factor -> Number);
}
use expressions::*;

fn main() {
    let res = Parser::lex_parse_with_ctx((), "(1 + 2) * 3 + 4").ok().expect("couldn't parse");

    println!("second result is {res}");
}
