use itertools::Itertools;

use crate::{Grammar, slr::item::SlrItem, symbol::Symbol};
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

    fn closure(&self, grammar: &Grammar) -> HashSet<SlrItem> {
        let mut stack = self.kernel.iter().cloned().collect_vec();
        let mut res = self.kernel.clone();
        while let Some(item) = stack.pop() {
            let Symbol::NonTerminal(non_terminal) = item.pointed_symbol(grammar) else {
                continue;
            };

            for new_item in grammar
                .get_productions_with_head(grammar.get_non_terminal(non_terminal).unwrap().name())
                .into_iter()
                .map(SlrItem::new)
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

    fn display(&self, grammar: &Grammar) {
        eprint!("{{");
        for item in self.kernel.iter() {
            item.display(grammar);
        }
        eprintln!("}}");
    }
}

#[derive(Debug)]
pub struct SlrAutomaton {
    states: Vec<SlrState>,
    symbols_count: usize,
    transitions: Vec<Vec<Option<usize>>>,
}

impl SlrAutomaton {
    pub fn compute(grammar: &Grammar) -> Self {
        let mut automaton = SlrAutomaton {
            states: Vec::new(),
            transitions: Vec::new(),
            symbols_count: grammar.symbols_count(),
        };
        automaton.populate(grammar);
        automaton
    }

    fn populate(&mut self, grammar: &Grammar) {
        let first_state = SlrState::new(HashSet::from_iter([SlrItem::new(usize::MAX)]));
        self.add_state(first_state);

        while let Some(state) = self.states.iter_mut().find(|state| !state.marked) {
            eprintln!("current state:");
            state.display(grammar);
            state.marked = true;
            let closure = state.closure(grammar);
            eprintln!("with closure:");
            for item in closure.iter() {
                item.display(grammar);
            }
            let mut transitions = vec![HashSet::new(); self.symbols_count];
            for (symbol, mut item) in closure.into_iter().filter_map(|item| {
                (!item.is_final_item(grammar)).then(|| (item.pointed_symbol(grammar), item))
            }) {
                item.move_marker();
                transitions[symbol.index(grammar)].insert(item);
            }
            let transitions = transitions
                .into_iter()
                .map(|kernel| (!kernel.is_empty()).then(|| SlrState::new(kernel)))
                .collect::<Vec<_>>();
            let transitions = transitions
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
            eprintln!("\ntransitions:");
            for (label, target_state) in transitions.iter().enumerate().filter_map(|(i, target)| {
                target.map(|target| (grammar.get_symbol_name_from_id(i), &self.states[target]))
            }) {
                eprint!("{label}: ");
                target_state.display(grammar);
            }
            self.add_transitions(transitions);
        }
    }

    fn add_state(&mut self, state: SlrState) {
        self.states.push(state);
    }

    fn add_transitions(&mut self, transitions: Vec<Option<usize>>) {
        self.transitions.push(transitions);
    }

    pub fn display_table(&self, grammar: &Grammar) {
        const COL_WIDTH: usize = 10;
        eprint!("{}", " ".repeat(COL_WIDTH));
        for sym in (0..grammar.symbols_count()).map(|i| grammar.get_symbol_name_from_id(i)) {
            eprint!("{sym:^width$}", width = COL_WIDTH);
        }
        eprintln!();
        for (row_i, row) in self.transitions.iter().enumerate() {
            eprint!("{row_i:^width$}", width = COL_WIDTH);
            for target_state in row.iter() {
                match target_state {
                    Some(id) => eprint!("{id:^width$}", width = COL_WIDTH),
                    None => eprint!("{}", " ".repeat(COL_WIDTH)),
                }
            }
            eprintln!();
        }
    }
}
