use itertools::Itertools;
use syn::Ident;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnrichedProduction {
    ident: Ident,
    head: Ident,
    body: Vec<Ident>,
}

impl EnrichedProduction {
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
