use std::fmt::Display;
use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnrichedNonTerminal {
    ident: Ident,
}

impl EnrichedNonTerminal {
    pub fn new(ident: Ident) -> Self {
        Self { ident }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Display for EnrichedNonTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}
