pub mod non_terminal;
pub mod production;
pub mod token;

use syn::Ident;

use crate::{
    non_terminal::EnrichedNonTerminal, production::EnrichedProduction, token::EnrichedToken,
};

#[derive(Debug)]
pub struct EnrichedGrammar {
    context: Option<Ident>,
    non_terminals: Vec<EnrichedNonTerminal>,
    tokens: Vec<EnrichedToken>,
    productions: Vec<EnrichedProduction>,
    start_symbol: EnrichedNonTerminal,
}

impl EnrichedGrammar {
    pub fn new(
        context: Option<Ident>,
        non_terminals: Vec<EnrichedNonTerminal>,
        tokens: Vec<EnrichedToken>,
        productions: Vec<EnrichedProduction>,
        start_symbol: EnrichedNonTerminal,
    ) -> Self {
        Self {
            context,
            non_terminals,
            tokens,
            productions,
            start_symbol,
        }
    }

    pub fn context(&self) -> &Option<Ident> {
        &self.context
    }

    pub fn tokens(&self) -> &Vec<EnrichedToken> {
        &self.tokens
    }

    pub fn non_terminals(&self) -> &Vec<EnrichedNonTerminal> {
        &self.non_terminals
    }

    pub fn start_symbol(&self) -> &EnrichedNonTerminal {
        &self.start_symbol
    }

    pub fn productions(&self) -> &Vec<EnrichedProduction> {
        &self.productions
    }

    pub fn token_id(&self, token: &Ident) -> Option<usize> {
        self.tokens().iter().position(|val| val.ident() == token)
    }

    pub fn non_terminal_id(&self, non_terminal: &Ident) -> Option<usize> {
        self.non_terminals()
            .iter()
            .position(|val| val.ident() == non_terminal)
    }
}
