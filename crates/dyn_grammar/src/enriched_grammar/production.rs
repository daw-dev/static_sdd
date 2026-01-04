use itertools::Itertools;
use std::fmt::Display;
use syn::{Ident, token};

use crate::{
    EnrichedGrammar,
    enriched_symbol::EnrichedSymbol,
    non_terminal::{self, EnrichedNonTerminal},
    token::EnrichedToken,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnrichedBaseProduction {
    ident: Ident,
    head: Ident,
    body: Vec<Ident>,
}

impl EnrichedBaseProduction {
    pub fn new(ident: Ident, head: Ident, body: Vec<Ident>) -> Self {
        Self { ident, head, body }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn arity(&self) -> usize {
        self.body.len()
    }

    pub fn head(&self) -> &Ident {
        &self.head
    }

    pub fn body(&self) -> &Vec<Ident> {
        &self.body
    }

    pub fn into_production(
        self,
        tokens: &[EnrichedToken],
        non_terminals: &[EnrichedNonTerminal],
    ) -> EnrichedProduction {
        EnrichedProduction::new(
            self.ident,
            self.head,
            self.body
                .into_iter()
                .map(|ident| {
                    tokens
                        .iter()
                        .position(|tok| tok.ident() == &ident)
                        .map(|id| EnrichedSymbol::Token(tokens[id].clone()))
                        .or_else(|| {
                            non_terminals
                                .iter()
                                .position(|nt| nt.ident() == &ident)
                                .map(|id| EnrichedSymbol::NonTerminal(non_terminals[id].clone()))
                        })
                        .expect("ident is neither a non terminal nor a token")
                })
                .collect(),
        )
    }
}

impl Display for EnrichedBaseProduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} -> ({})",
            self.ident,
            self.head,
            self.body.iter().format(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichedProduction {
    ident: Ident,
    head: Ident,
    body: Vec<EnrichedSymbol>,
}

impl Display for EnrichedProduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} -> ({})",
            self.ident,
            self.head,
            self.body.iter().format(", ")
        )
    }
}

impl EnrichedProduction {
    pub fn new(ident: Ident, head: Ident, body: Vec<EnrichedSymbol>) -> Self {
        Self { ident, head, body }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn arity(&self) -> usize {
        self.body.len()
    }

    pub fn head(&self) -> &Ident {
        &self.head
    }

    pub fn body(&self) -> &Vec<EnrichedSymbol> {
        &self.body
    }
}
