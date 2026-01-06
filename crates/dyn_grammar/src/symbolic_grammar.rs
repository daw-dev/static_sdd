use crate::{EnrichedGrammar, production::EnrichedProduction};
use itertools::Itertools;
use std::{collections::HashSet, fmt::Display};
use syn::Ident;

pub type SymbolicToken = usize;

pub type SymbolicNonTerminal = usize;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum SymbolicSymbol {
    Token(SymbolicToken),
    NonTerminal(SymbolicNonTerminal),
    EOF,
}

impl Display for SymbolicSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolicSymbol::Token(tok) => write!(f, "`{tok}`"),
            SymbolicSymbol::NonTerminal(nt) => write!(f, "{nt}"),
            SymbolicSymbol::EOF => write!(f, "$"),
        }
    }
}

#[derive(Debug)]
pub struct SymbolicProduction {
    production_id: usize,
    head: SymbolicNonTerminal,
    body: Vec<SymbolicSymbol>,
}

impl Display for SymbolicProduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} -> ({})",
            self.production_id,
            self.head,
            self.body().iter().format(", ")
        )
    }
}

impl SymbolicProduction {
    pub fn id(&self) -> usize {
        self.production_id
    }

    pub fn head(&self) -> &SymbolicNonTerminal {
        &self.head
    }

    pub fn body(&self) -> &Vec<SymbolicSymbol> {
        &self.body
    }

    pub fn arity(&self) -> usize {
        self.body.len()
    }

    pub fn special_production(start_symbol: SymbolicNonTerminal) -> Self {
        Self {
            production_id: usize::MAX,
            head: usize::MAX,
            body: vec![SymbolicSymbol::NonTerminal(start_symbol)],
        }
    }
}

pub struct SymbolicFirstSet {
    pub tokens: HashSet<SymbolicToken>,
    pub nullable: bool,
}

pub struct SymbolicFollowSet {
    pub tokens: HashSet<SymbolicToken>,
    pub eof_follows: bool,
}

#[derive(Debug)]
pub struct SymbolicGrammar<'a> {
    enriched_grammar: &'a EnrichedGrammar,
    token_count: usize,
    non_terminal_count: usize,
    start_symbol: SymbolicNonTerminal,
    special_production: SymbolicProduction,
    productions: Vec<SymbolicProduction>,
}

impl Display for SymbolicGrammar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymbolicGrammar {{ ")?;
        write!(
            f,
            "non_terminals: [{}], ",
            (0..self.non_terminal_count).format(", ")
        )?;
        write!(
            f,
            "tokens: [{}], ",
            (0..self.token_count).map(|i| format!("`{i}`")).format(", ")
        )?;
        write!(f, "start_symbol: {}, ", self.start_symbol)?;
        write!(
            f,
            "productions: [{}] }}",
            self.productions.iter().format(", ")
        )
    }
}

impl<'a> SymbolicGrammar<'a> {
    pub fn get_production(&self, id: usize) -> Option<&SymbolicProduction> {
        if id == usize::MAX {
            return Some(&self.special_production);
        }
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
                .map(|sym| match sym {
                    crate::enriched_symbol::EnrichedSymbol::Token(enriched_token) => {
                        SymbolicSymbol::Token(
                            enriched_grammar.token_id(enriched_token.ident()).unwrap(),
                        )
                    }
                    crate::enriched_symbol::EnrichedSymbol::NonTerminal(enriched_non_terminal) => {
                        SymbolicSymbol::NonTerminal(
                            enriched_grammar
                                .non_terminal_id(enriched_non_terminal.ident())
                                .unwrap(),
                        )
                    }
                    crate::enriched_symbol::EnrichedSymbol::EOF => SymbolicSymbol::EOF,
                })
                .collect(),
        }
    }

    fn first_set_helper(
        &self,
        beta: &[SymbolicSymbol],
        visited: &mut HashSet<usize>,
    ) -> SymbolicFirstSet {
        if beta.is_empty() {
            return SymbolicFirstSet {
                tokens: HashSet::new(),
                nullable: true,
            };
        }

        let mut res = SymbolicFirstSet {
            tokens: HashSet::new(),
            nullable: false,
        };

        for symbol in beta.iter() {
            match symbol {
                SymbolicSymbol::Token(token) => {
                    res.tokens.insert(*token);
                    return res;
                }
                SymbolicSymbol::NonTerminal(non_terminal) => {
                    let productions = self.get_productions_with_head(*non_terminal);
                    for prod in productions
                        .into_iter()
                    {
                        if visited.contains(&prod.id()) {
                            continue;
                        }
                        visited.insert(prod.id());
                        let body = prod.body();
                        let firsts = self.first_set_helper(body, visited);
                        res.tokens.extend(firsts.tokens);
                        if !firsts.nullable {
                            return res;
                        }
                    }
                }
                SymbolicSymbol::EOF => unreachable!(),
            }
        }

        res.nullable = true;
        res
    }

    pub fn first_set(&self, beta: &[SymbolicSymbol]) -> SymbolicFirstSet {
        self.first_set_helper(beta, &mut HashSet::new())
    }
}

impl<'a> From<&'a EnrichedGrammar> for SymbolicGrammar<'a> {
    fn from(value: &'a EnrichedGrammar) -> Self {
        let start_symbol = value.non_terminal_id(value.start_symbol().ident()).unwrap();
        Self {
            enriched_grammar: value,
            token_count: value.tokens().len(),
            non_terminal_count: value.non_terminals().len(),
            start_symbol,
            special_production: SymbolicProduction::special_production(start_symbol),
            productions: value
                .productions()
                .iter()
                .enumerate()
                .map(|(id, prod)| SymbolicGrammar::map_production(value, id, prod))
                .collect(),
        }
    }
}
