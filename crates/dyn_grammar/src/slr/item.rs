use crate::symbolic_grammar::{SymbolicGrammar, SymbolicSymbol};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct SlrItem {
    pub(crate) production_id: usize,
    pub(crate) marker_position: usize,
}

impl SlrItem {
    pub fn new(production_id: usize) -> Self {
        Self {
            production_id,
            marker_position: 0,
        }
    }

    pub fn pointed_symbol(&self, grammar: &SymbolicGrammar) -> SymbolicSymbol {
        grammar
            .get_production(self.production_id)
            .expect("production not found")
            .body()
            .get(self.marker_position)
            .cloned()
            .unwrap_or(SymbolicSymbol::EOF)
    }

    pub fn is_final_item(&self, grammar: &SymbolicGrammar) -> bool {
        self.marker_position == grammar.get_production(self.production_id).unwrap().arity()
    }

    pub fn move_marker(&mut self) {
        self.marker_position += 1;
    }
}
