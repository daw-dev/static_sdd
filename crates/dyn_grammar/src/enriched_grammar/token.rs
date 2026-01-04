use std::fmt::Display;

use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnrichedToken {
    ident: Ident,
    regexpr: String,
}

impl EnrichedToken {
    pub fn new(ident: Ident, regexpr: String) -> Self {
        Self { ident, regexpr }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Display for EnrichedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}/ => {}", self.regexpr, self.ident)
    }
}
