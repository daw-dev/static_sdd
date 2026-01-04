use static_sdd::*;

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

    #[token = r"\d+"]
    pub type Number = usize;

    #[token = r"\+"]
    pub struct Plus;

    #[token = r"\*"]
    pub struct Times;

    #[token = r"\("]
    pub struct OpenPar;

    #[token = r"\)"]
    pub struct ClosedPar;

    production!(Addition, Expression -> (Expression, Plus, Term), |(e, _ ,t)| e + t);
    production!(NoAddition, Expression -> Term);
    production!(Multiplication, Term -> (Term, Times, Factor), |(t, _, f)| t * f);
    production!(NoMultiplication, Term -> Factor);
    production!(Parenthesis, Factor -> (OpenPar, Expression, ClosedPar), |(_, e, _)| e);
    production!(ActualNumber, Factor -> Number);
}

fn main() {
    use expressions::*;

    let res = parse(
        (),
        [
            Token::Number(5),
            Token::Plus(Plus),
            Token::Number(2),
            Token::Times(Times),
            Token::OpenPar(OpenPar),
            Token::Number(3),
            Token::Plus(Plus),
            Token::Number(1),
            Token::ClosedPar(ClosedPar),
        ],
    );

    println!("result is {res}");
}
