use std::hash::Hash;

use itertools::Itertools;

use crate::{Grammar, symbol::Symbol};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct SlrItem {
    production_id: usize,
    marker_position: usize,
}

impl SlrItem {
    pub fn new(production_id: usize) -> Self {
        Self {
            production_id,
            marker_position: 0,
        }
    }

    pub fn pointed_symbol(&self, grammar: &Grammar) -> Symbol {
        let production = grammar.get_production_from_id(self.production_id).unwrap();
        match production.body().get(self.marker_position) {
            Some(symbol_name) => grammar.get_symbol(symbol_name).unwrap(),
            None => Symbol::EOF,
        }
    }

    pub fn is_final_item(&self, grammar: &Grammar) -> bool {
        self.marker_position
            == grammar
                .get_production_from_id(self.production_id)
                .unwrap()
                .arity()
    }

    pub fn display(&self, grammar: &Grammar) {
        let production = grammar.get_production_from_id(self.production_id).unwrap();
        let (first, second) = production.body().split_at(self.marker_position);
        print!(
            "{}->({}) ",
            production.head(),
            first
                .iter()
                .map(String::as_str)
                .chain(std::iter::once("·"))
                .chain(second.iter().map(String::as_str))
                .format(" "),
        );
    }

    pub fn move_marker(&mut self) {
        self.marker_position += 1;
    }
}
