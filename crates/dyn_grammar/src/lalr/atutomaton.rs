use std::collections::HashSet;
use crate::lalr::item::LalrItem;

pub struct LalrState {
    state_id: usize,
    kernel: HashSet<LalrItem>,
}

pub struct SymbolicAutomaton {
    states: Vec<LalrState>,
}

