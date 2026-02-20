use dyn_grammar::{EnrichedGrammar, lalr::LalrAutomaton};
use syn::Ident;
use std::rc::Rc;

pub struct Constructor {
    pub enriched_grammar: Rc<EnrichedGrammar>,
    pub automaton: LalrAutomaton,
    pub internal_mod_name: Option<Ident>,
}
