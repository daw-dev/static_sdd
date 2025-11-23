use crate::{expansion::Expansion, non_terminal::NonTerminal};

pub struct Production {
    name: String,
    head: NonTerminal,
    body: Expansion,
}
