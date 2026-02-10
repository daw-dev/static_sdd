use semasia::*;

#[grammar]
mod sat {
    use super::*;

    #[derive(Debug)]
    pub enum Formula {
        Atom(String),
        Negation(Box<Formula>),
        Conjunction(Box<Formula>, Box<Formula>),
        Disjunction(Box<Formula>, Box<Formula>),
        Implies(Box<Formula>, Box<Formula>),
        Equivalence(Box<Formula>, Box<Formula>),
        ImpliedBy(Box<Formula>, Box<Formula>),
    }

    impl From<(Not, Formula)> for Formula {
        fn from((_, f): (Not, Formula)) -> Self {
            Formula::Negation(Box::new(f))
        }
    }

    #[non_terminal]
    #[start_symbol]
    pub type I = Formula;

    #[non_terminal]
    pub type D = Formula;

    #[non_terminal]
    pub type C = Formula;

    #[non_terminal]
    pub type N = Formula;

    #[token(regex = r"[a-zA-Z]+")]
    pub type Atom = String;

    #[token("->")]
    pub struct RightArrow;

    #[token("<-")]
    pub struct LeftArrow;

    #[token("<->")]
    pub struct LeftRightArrow;

    #[token("|")]
    pub struct Or;

    #[token("&")]
    pub struct And;

    #[token("!")]
    pub struct Not;

    #[token("(")]
    pub struct OpenPar;

    #[token(")")]
    pub struct ClosePar;

    production!(P1, I -> (D, RightArrow, I), |(d, _, i)| Formula::Implies(Box::new(d), Box::new(i)));
    production!(P2, I -> (D, LeftArrow, I), |(d, _, i)| Formula::ImpliedBy(Box::new(d), Box::new(i)));
    production!(P3, I -> (D, LeftRightArrow, I), |(d, _, i)| Formula::Equivalence(Box::new(d), Box::new(i)));
    production!(P4, I -> D);
    production!(P5, D -> (D, Or, C), |(d, _, c)| Formula::Disjunction(Box::new(d), Box::new(c)));
    production!(P6, D -> C);
    production!(P7, C -> (C, And, N), |(c, _, n)| Formula::Conjunction(Box::new(c), Box::new(n)));
    production!(P8, C -> N);
    production!(P9, N -> (Not, N)); // the into implementation is used
    production!(P10, N -> Atom, |a| Formula::Atom(a));
    production!(P11, N -> (OpenPar, I, ClosePar), |(_, i, _)| i);
}

use sat::*;

fn main() {
    let res = Parser::lex_parse("a -> b & c | !d");

    match res {
        Ok(res) => println!("{res:?}"),
        Err(err) => println!("{err:?}"),
    }
}
