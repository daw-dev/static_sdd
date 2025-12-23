use syn::Ident;

use crate::{EnrichedGrammar, production::EnrichedProduction};

pub type SymbolicToken = usize;

pub type SymbolicNonTerminal = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicSymbol {
    Token(SymbolicToken),
    NonTerminal(SymbolicNonTerminal),
    EOF,
}

pub struct SymbolicProduction {
    production_id: usize,
    head: SymbolicNonTerminal,
    body: Vec<SymbolicSymbol>,
}

impl SymbolicProduction {
    pub fn id(&self) -> usize {
        self.production_id
    }

    pub fn body(&self) -> &Vec<SymbolicSymbol> {
        &self.body
    }

    pub fn arity(&self) -> usize {
        self.body.len()
    }
}

pub struct SymbolicGrammar {
    token_count: usize,
    non_terminal_count: usize,
    start_symbol: SymbolicNonTerminal,
    productions: Vec<SymbolicProduction>,
}

impl SymbolicGrammar {
    pub fn get_production(&self, id: usize) -> Option<&SymbolicProduction> {
        self.productions.get(id)
    }

    pub fn get_productions_with_head(&self, head: SymbolicNonTerminal) -> Vec<&SymbolicProduction> {
        self.productions
            .iter()
            .filter(|prod| prod.head == head)
            .collect()
    }

    pub fn token_count(&self) -> usize {
        self.token_count
    }

    pub fn non_terminal_count(&self) -> usize {
        self.non_terminal_count
    }

    fn find_symbol(enriched_grammar: &EnrichedGrammar, ident: &Ident) -> SymbolicSymbol {
        enriched_grammar
            .token_id(ident)
            .map(SymbolicSymbol::Token)
            .or_else(|| {
                enriched_grammar
                    .non_terminal_id(ident)
                    .map(SymbolicSymbol::NonTerminal)
            })
            .unwrap_or(SymbolicSymbol::EOF)
    }

    fn map_production(
        enriched_grammar: &EnrichedGrammar,
        id: usize,
        enriched_production: &EnrichedProduction,
    ) -> SymbolicProduction {
        SymbolicProduction {
            production_id: id,
            head: enriched_grammar
                .non_terminal_id(enriched_production.head())
                .unwrap(),
            body: enriched_production
                .body()
                .iter()
                .map(|ident| SymbolicGrammar::find_symbol(enriched_grammar, ident))
                .collect(),
        }
    }
}

impl From<&EnrichedGrammar> for SymbolicGrammar {
    fn from(value: &EnrichedGrammar) -> Self {
        Self {
            token_count: value.tokens().len(),
            non_terminal_count: value.non_terminals().len(),
            start_symbol: value.non_terminal_id(value.start_symbol().ident()).unwrap(),
            productions: value
                .productions()
                .iter()
                .enumerate()
                .map(|(id, prod)| SymbolicGrammar::map_production(value, id, prod))
                .collect(),
        }
    }
}
