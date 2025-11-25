pub mod non_terminal;
pub mod production;
pub mod symbol;
pub mod token;

use crate::{non_terminal::NonTerminal, production::Production, symbol::Symbol, token::Token};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Grammar {
    non_terminals: Vec<NonTerminal>,
    tokens: Vec<Token>,
    productions: Vec<Production>,
    extra_production: Production,
    start_symbol: String,
    symbols_map: HashMap<String, usize>,
    productions_map: HashMap<String, usize>,
}

impl Grammar {
    pub fn new(
        non_terminals: Vec<NonTerminal>,
        tokens: Vec<Token>,
        productions: Vec<Production>,
        start_symbol: String,
    ) -> Self {
        let symbols_map = Self::compute_symbols_map(&non_terminals, &tokens);
        let productions_map = Self::compute_productions_map(&productions);
        let extra_production =
            Production::new("".to_string(), "".to_string(), vec![start_symbol.clone()]);
        Self {
            non_terminals,
            tokens,
            productions,
            extra_production,
            start_symbol,
            symbols_map,
            productions_map,
        }
    }

    fn compute_symbols_map(
        non_terminals: &[NonTerminal],
        tokens: &[Token],
    ) -> HashMap<String, usize> {
        std::iter::chain(
            tokens.iter().map(|token| token.name().clone()),
            non_terminals.iter().map(|non_t| non_t.name().clone()),
        )
        .enumerate()
        .map(|(a, b)| (b, a))
        .collect()
    }

    fn compute_productions_map(productions: &[Production]) -> HashMap<String, usize> {
        productions
            .iter()
            .enumerate()
            .map(|(i, prod)| (prod.name().clone(), i))
            .collect()
    }

    pub fn get_symbol(&self, name: &String) -> Option<Symbol> {
        self.symbols_map
            .get(name)
            .copied()
            .map(|index| self.get_symbol_from_id(index))
    }

    pub fn get_symbol_from_id(&self, index: usize) -> Symbol {
        index
            .checked_sub(self.tokens.len())
            .map(|i| {
                if i < self.non_terminals.len() {
                    Symbol::NonTerminal(i)
                } else {
                    Symbol::EOF
                }
            })
            .unwrap_or(Symbol::Token(index))
    }

    pub fn get_symbol_name_from_id(&self, index: usize) -> &str {
        match self.get_symbol_from_id(index) {
            Symbol::NonTerminal(i) => self.get_non_terminal(i).unwrap().name(),
            Symbol::Token(i) => self.get_token(i).unwrap().name(),
            Symbol::EOF => "$",
        }
    }

    pub fn get_production(&self, name: &String) -> Option<&Production> {
        self.productions_map
            .get(name)
            .map(|i| &self.productions[*i])
    }

    pub fn get_production_from_id(&self, id: usize) -> Option<&Production> {
        if id == usize::MAX {
            return Some(&self.extra_production);
        }
        self.productions.get(id)
    }

    pub fn get_token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    pub fn get_non_terminal(&self, index: usize) -> Option<&NonTerminal> {
        self.non_terminals.get(index)
    }

    pub fn get_production_with_head(&self, head: &String) -> Vec<usize> {
        self.productions
            .iter()
            .enumerate()
            .filter_map(|(i, prod)| (prod.head() == head).then(|| i))
            .collect()
    }

    pub fn symbols_count(&self) -> usize {
        self.tokens.len() + self.non_terminals.len() + 1
    }
}
