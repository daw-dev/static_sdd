pub mod enriched_symbol;
pub mod non_terminal;
pub mod production;
pub mod token;

use std::fmt::Display;

use itertools::Itertools;
use syn::Ident;

use crate::{
    non_terminal::EnrichedNonTerminal,
    production::{EnrichedBaseProduction, EnrichedProduction},
    token::EnrichedToken,
};

#[derive(Debug)]
pub struct EnrichedGrammar {
    context: Option<Ident>,
    non_terminals: Vec<EnrichedNonTerminal>,
    tokens: Vec<EnrichedToken>,
    start_symbol: EnrichedNonTerminal,
    productions: Vec<EnrichedProduction>,
}

impl EnrichedGrammar {
    pub fn new(
        context: Option<Ident>,
        non_terminals: Vec<EnrichedNonTerminal>,
        tokens: Vec<EnrichedToken>,
        start_symbol: EnrichedNonTerminal,
        productions: Vec<EnrichedBaseProduction>,
    ) -> Self {
        let productions = productions
            .into_iter()
            .map(|prod| prod.into_production(&tokens, &non_terminals))
            .collect();
        Self {
            context,
            non_terminals,
            tokens,
            start_symbol,
            productions,
        }
    }

    pub fn context(&self) -> Option<&Ident> {
        self.context.as_ref()
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

impl Display for EnrichedGrammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnrichedGrammar {{ ")?;
        write!(f, "context: ")?;
        match self.context.as_ref() {
            Some(ctx) => write!(f, "Some({}), ", ctx)?,
            None => write!(f, "None, ")?,
        }
        write!(
            f,
            "non_terminals: [{}], tokens: [{}], ",
            self.non_terminals.iter().format(", "),
            self.tokens.iter().format(", ")
        )?;
        write!(f, "start_symbol: {}, ", self.start_symbol.ident())?;
        write!(
            f,
            "productions: [{}] }}",
            self.productions.iter().format(", ")
        )
    }
}
