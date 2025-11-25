use crate::Grammar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Symbol {
    NonTerminal(usize),
    Token(usize),
    EOF,
}

impl Symbol {
    pub fn index(&self, grammar: &Grammar) -> usize {
        match self {
            Symbol::Token(i) => *i,
            Symbol::NonTerminal(i) => i + grammar.tokens.len(),
            Symbol::EOF => grammar.tokens.len() + grammar.non_terminals.len(),
        }
    }
}
