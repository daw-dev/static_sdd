use std::hash::Hash;
use crate::lalr::equation::SymbolicSet;

pub struct LalrItem {
    production_name: String,
    marker_position: usize,
    lookahead_set: SymbolicSet,
}

impl Hash for LalrItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.production_name.hash(state);
        self.marker_position.hash(state);
    }
}

impl PartialEq for LalrItem {
    fn eq(&self, other: &Self) -> bool {
        self.production_name == other.production_name
            && self.marker_position == other.marker_position
    }
}

impl Eq for LalrItem {}

