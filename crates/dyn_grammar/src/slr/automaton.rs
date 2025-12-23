use itertools::Itertools;

use crate::{
    slr::item::SlrItem,
    symbolic_grammar::{SymbolicGrammar, SymbolicNonTerminal, SymbolicSymbol, SymbolicToken},
};
use std::{collections::HashSet, usize};

#[derive(Debug)]
pub struct SlrState {
    kernel: HashSet<SlrItem>,
    marked: bool,
}

impl PartialEq for SlrState {
    fn eq(&self, other: &Self) -> bool {
        self.kernel == other.kernel
    }
}

impl SlrState {
    pub fn new(kernel: HashSet<SlrItem>) -> Self {
        Self {
            kernel,
            marked: false,
        }
    }

    fn closure(&self, grammar: &SymbolicGrammar) -> HashSet<SlrItem> {
        let mut stack = self.kernel.iter().cloned().collect_vec();
        let mut res = self.kernel.clone();
        while let Some(item) = stack.pop() {
            let SymbolicSymbol::NonTerminal(non_terminal) = item.pointed_symbol(grammar) else {
                continue;
            };

            for new_item in grammar
                .get_productions_with_head(non_terminal)
                .into_iter()
                .map(|prod| SlrItem::new(prod.id()))
            {
                if res.contains(&new_item) {
                    continue;
                }
                stack.push(new_item.clone());
                res.insert(new_item);
            }
        }
        res
    }
}

#[derive(Debug)]
pub struct SlrAutomaton {
    states: Vec<SlrState>,
    transitions: Vec<(Vec<Option<SymbolicToken>>, Vec<Option<SymbolicNonTerminal>>)>,
}

impl SlrAutomaton {
    pub fn compute(grammar: &SymbolicGrammar) -> Self {
        let mut automaton = SlrAutomaton {
            states: Vec::new(),
            transitions: Vec::new(),
        };
        automaton.populate(grammar);
        automaton
    }

    fn populate(&mut self, grammar: &SymbolicGrammar) {
        let first_state = SlrState::new(HashSet::from_iter([SlrItem::new(usize::MAX)]));
        self.add_state(first_state);

        while let Some(state) = self.states.iter_mut().find(|state| !state.marked) {
            state.marked = true;
            let closure = state.closure(grammar);
            let mut token_transitions = vec![HashSet::new(); grammar.token_count()];
            let mut non_terminal_transitions = vec![HashSet::new(); grammar.non_terminal_count()];
            for (symbol, mut item) in closure.into_iter().filter_map(|item| {
                (!item.is_final_item(grammar)).then(|| (item.pointed_symbol(grammar), item))
            }) {
                item.move_marker();
                match symbol {
                    SymbolicSymbol::Token(tok) => {
                        token_transitions[tok].insert(item);
                    }
                    SymbolicSymbol::NonTerminal(nt) => {
                        non_terminal_transitions[nt].insert(item);
                    }
                    SymbolicSymbol::EOF => unreachable!(),
                }
            }
            let token_transitions = token_transitions
                .into_iter()
                .map(|kernel| (!kernel.is_empty()).then(|| SlrState::new(kernel)))
                .collect::<Vec<_>>();
            let non_terminal_transitions = non_terminal_transitions
                .into_iter()
                .map(|kernel| (!kernel.is_empty()).then(|| SlrState::new(kernel)))
                .collect::<Vec<_>>();
            let token_transitions = token_transitions
                .into_iter()
                .map(|target_state| {
                    target_state.map(|target_state| {
                        match self.states.iter().position(|state| state == &target_state) {
                            Some(i) => i,
                            None => {
                                let state_id = self.states.len();
                                self.add_state(target_state);
                                state_id
                            }
                        }
                    })
                })
                .collect::<Vec<_>>();
            let non_terminal_transitions = non_terminal_transitions
                .into_iter()
                .map(|target_state| {
                    target_state.map(|target_state| {
                        match self.states.iter().position(|state| state == &target_state) {
                            Some(i) => i,
                            None => {
                                let state_id = self.states.len();
                                self.add_state(target_state);
                                state_id
                            }
                        }
                    })
                })
                .collect::<Vec<_>>();
            self.add_transitions(token_transitions, non_terminal_transitions);
        }
    }

    fn add_state(&mut self, state: SlrState) {
        self.states.push(state);
    }

    fn add_transitions(
        &mut self,
        token_transitions: Vec<Option<SymbolicToken>>,
        non_terminal_transitions: Vec<Option<SymbolicNonTerminal>>,
    ) {
        self.transitions
            .push((token_transitions, non_terminal_transitions));
    }
}
