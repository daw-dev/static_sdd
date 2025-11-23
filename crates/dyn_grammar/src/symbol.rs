use crate::{non_terminal::NonTerminal, token::Token};

pub enum Symbol {
    NonTerminal(NonTerminal),
    Token(Token),
}
