#![allow(clippy::mutable_key_type)]

use crate::{
    parsing::{
        action::{EofAction, TokenAction},
        tables::{EofTable, NonTerminalTable, TokenTable, TransitionTables},
    },
    symbolic_grammar::{SymbolicGrammar, SymbolicSymbol, SymbolicToken},
};
use itertools::Itertools;
use std::{cell::RefCell, collections::HashSet, fmt::Display, hash::Hash, rc::Rc};

#[derive(Clone)]
struct LookAhead {
    tokens: HashSet<SymbolicToken>,
    can_eof_follow: bool,
}

impl Display for LookAhead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tokens = self.tokens.iter().collect_vec();
        tokens.sort();
        write!(
            f,
            "{{{}}}",
            tokens
                .into_iter()
                .map(|tok| format!("`{tok}`"))
                .chain(self.can_eof_follow.then_some("$".to_string()))
                .format(", ")
        )
    }
}

#[derive(Clone)]
struct LookAheadNodeRef(Rc<RefCell<LookAheadNode>>);

impl LookAheadNodeRef {
    pub fn initial_lookahead_node(counter: &mut usize) -> LookAheadNodeRef {
        Self::new(
            counter,
            LookAhead {
                tokens: HashSet::new(),
                can_eof_follow: true,
            },
            Vec::new(),
        )
    }

    pub fn new(
        counter: &mut usize,
        natural_lookahead: LookAhead,
        dependencies: Vec<LookAheadNodeRef>,
    ) -> Self {
        let node_id = *counter;
        *counter += 1;
        Self(Rc::new(RefCell::new(LookAheadNode {
            node_id,
            natural_lookahead,
            dependencies,
        })))
    }

    fn compute_lookahead_helper(&self, visited: &mut HashSet<usize>) -> LookAhead {
        // TODO: not so simple, graph could have cycles
        let borrow = self.0.borrow();
        let mut res = borrow.natural_lookahead.clone();
        if visited.contains(&borrow.node_id) {
            return res;
        }
        visited.insert(borrow.node_id);
        for dep in borrow.dependencies.iter() {
            let lh = dep.compute_lookahead_helper(visited);
            res.tokens.extend(lh.tokens);
            res.can_eof_follow |= lh.can_eof_follow;
        }
        res
    }

    pub fn compute_lookahead(&self) -> LookAhead {
        self.compute_lookahead_helper(&mut HashSet::new())
    }

    pub fn add_dependency(&self, dependency: LookAheadNodeRef) {
        self.0.borrow_mut().dependencies.push(dependency);
    }
}

struct LookAheadNode {
    node_id: usize,
    natural_lookahead: LookAhead,
    dependencies: Vec<LookAheadNodeRef>,
}

#[derive(Clone)]
struct LalrItem {
    production_id: usize,
    marker_position: usize,
    lookahead_node: LookAheadNodeRef,
}

impl Hash for LalrItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.production_id.hash(state);
        self.marker_position.hash(state);
    }
}

impl PartialEq for LalrItem {
    fn eq(&self, other: &Self) -> bool {
        self.production_id == other.production_id && self.marker_position == other.marker_position
    }
}

impl Eq for LalrItem {}

impl LalrItem {
    pub fn new(production_id: usize, lookahead_node: LookAheadNodeRef) -> Self {
        Self {
            production_id,
            marker_position: 0,
            lookahead_node,
        }
    }

    pub fn pointed_symbol(&self, grammar: &SymbolicGrammar) -> Option<SymbolicSymbol> {
        grammar
            .get_production(self.production_id)
            .expect("production not found")
            .body()
            .get(self.marker_position)
            .cloned()
    }

    pub fn move_marker(&mut self) {
        self.marker_position += 1;
    }

    fn is_reducing(&self, grammar: &SymbolicGrammar) -> bool {
        self.marker_position == grammar.get_production(self.production_id).unwrap().arity()
    }
}

struct LalrState {
    kernel: HashSet<LalrItem>,
    marked: bool,
    epsilon_items: HashSet<LalrItem>,
}

impl PartialEq for LalrState {
    fn eq(&self, other: &Self) -> bool {
        self.kernel == other.kernel
    }
}

impl LalrState {
    fn new(kernel: HashSet<LalrItem>) -> Self {
        Self {
            kernel,
            marked: false,
            epsilon_items: HashSet::new(),
        }
    }

    fn closure(&self, counter: &mut usize, grammar: &SymbolicGrammar) -> HashSet<LalrItem> {
        let mut stack = self.kernel.clone().into_iter().collect_vec();
        let mut res = self.kernel.clone();

        while let Some(item) = stack.pop() {
            if item.is_reducing(grammar) {
                continue;
            }

            let Some(SymbolicSymbol::NonTerminal(non_terminal)) = item.pointed_symbol(grammar)
            else {
                continue;
            };

            let item_production = grammar
                .get_production(item.production_id)
                .expect("production not found!");

            let beta = &item_production.body()[item.marker_position + 1..];

            let firsts = grammar.first_set(beta);
            let natural_lookahead = LookAhead {
                tokens: firsts.tokens,
                can_eof_follow: false,
            };
            let dependencies = if firsts.nullable {
                vec![item.lookahead_node]
            } else {
                Vec::new()
            };
            let lookahead_node = LookAheadNodeRef::new(counter, natural_lookahead, dependencies);

            for new_item in grammar
                .get_productions_with_head(non_terminal)
                .into_iter()
                .map(|prod| LalrItem::new(prod.id(), lookahead_node.clone()))
            {
                match res.get(&new_item) {
                    Some(item) => {
                        item.lookahead_node.add_dependency(lookahead_node.clone());
                    }
                    None => {
                        stack.push(new_item.clone());
                        res.insert(new_item);
                    }
                }
            }
        }
        res
    }
}

pub struct LalrAutomaton {
    grammar: SymbolicGrammar,
    states: Vec<LalrState>,
    transitions: TransitionTables,
}

impl LalrAutomaton {
    pub fn compute(grammar: SymbolicGrammar) -> Self {
        let mut automaton = Self {
            grammar,
            states: Vec::new(),
            transitions: TransitionTables::new(),
        };
        automaton.populate();
        automaton
    }

    pub fn populate(&mut self) {
        let mut counter = 0;
        let first_state = LalrState::new(HashSet::from_iter([LalrItem::new(
            usize::MAX,
            LookAheadNodeRef::initial_lookahead_node(&mut counter),
        )]));
        self.add_state(first_state);

        while let Some(state) = self.states.iter_mut().find(|state| !state.marked) {
            state.marked = true;
            let closure = state.closure(&mut counter, &self.grammar);
            for eps_item in closure.iter().filter(|item| {
                self.grammar
                    .get_production(item.production_id)
                    .unwrap()
                    .arity()
                    == 0
            }) {
                state.epsilon_items.insert(eps_item.clone());
            }
            let mut token_transitions = vec![HashSet::new(); self.grammar.token_count()];
            let mut non_terminal_transitions =
                vec![HashSet::new(); self.grammar.non_terminal_count()];
            for (symbol, mut item) in closure.into_iter().filter_map(|item| {
                (!item.is_reducing(&self.grammar))
                    .then(|| (item.pointed_symbol(&self.grammar).unwrap(), item))
            }) {
                item.move_marker();
                match symbol {
                    SymbolicSymbol::Token(tok) => {
                        token_transitions[tok].insert(item);
                    }
                    SymbolicSymbol::NonTerminal(nt) => {
                        non_terminal_transitions[nt].insert(item);
                    }
                }
            }
            let token_transitions = token_transitions
                .into_iter()
                .map(|kernel| (!kernel.is_empty()).then(|| LalrState::new(kernel)))
                .collect::<Vec<_>>();
            let non_terminal_transitions = non_terminal_transitions
                .into_iter()
                .map(|kernel| (!kernel.is_empty()).then(|| LalrState::new(kernel)))
                .collect::<Vec<_>>();
            let token_transitions = token_transitions
                .into_iter()
                .map(|target_state| {
                    target_state.map(|target_state| {
                        match self.states.iter().position(|state| state == &target_state) {
                            Some(i) => {
                                for new_item in target_state.kernel.iter() {
                                    self.states[i]
                                        .kernel
                                        .get(new_item)
                                        .unwrap()
                                        .lookahead_node
                                        .add_dependency(new_item.lookahead_node.clone());
                                }
                                i
                            }
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
            self.transitions
                .add_transitions(token_transitions, non_terminal_transitions);
        }
    }

    fn add_state(&mut self, state: LalrState) {
        self.states.push(state);
    }

    pub fn states_count(&self) -> usize {
        self.states.len()
    }

    pub fn generate_tables(&self) -> (TokenTable, EofTable, NonTerminalTable) {
        let mut token_table = TokenTable::new(self.grammar.token_count());
        let mut eof_table = EofTable::new();
        let mut goto_table = NonTerminalTable::new(self.grammar.non_terminal_count());

        for ((state_id, state), (token_transitions, non_terminal_transitions)) in
            self.states.iter().enumerate().zip(self.transitions.iter())
        {
            token_table.add_state();
            eof_table.add_state();

            for reducing_item in state
                .kernel
                .iter()
                .filter(|item| item.is_reducing(&self.grammar))
                .chain(state.epsilon_items.iter())
            {
                let lookahead = reducing_item.lookahead_node.compute_lookahead();
                for token in lookahead.tokens.into_iter() {
                    let action = TokenAction::Reduce(reducing_item.production_id);
                    let entry = &mut token_table[(state_id, token)];
                    if let Some(reduce) = entry.take() {
                        eprintln!("reduce/reduce conflict");
                        eprintln!("current reduce: {reduce:?}");
                        eprintln!("new reduce: {action:?}");
                    }
                    *entry = Some(action);
                }
                if lookahead.can_eof_follow {
                    let action = if reducing_item.production_id == usize::MAX {
                        EofAction::Accept
                    } else {
                        EofAction::Reduce(reducing_item.production_id)
                    };
                    let entry = &mut eof_table[state_id];
                    if let Some(reduce) = entry.take() {
                        eprintln!("reduce/reduce conflict");
                        eprintln!("current reduce: {reduce:?}");
                        eprintln!("new reduce: {action:?}");
                    }
                    *entry = Some(action);
                }
            }

            for (token, target) in token_transitions.iter().enumerate() {
                let Some(target) = target else {
                    continue;
                };
                let entry = &mut token_table[(state_id, token)];
                if let Some(reduce) = entry.take() {
                    eprintln!("shift/reduce conflict");
                    eprintln!("shift: {:?}", TokenAction::Shift(*target));
                    eprintln!("reduce: {reduce:?}");
                }
                *entry = Some(TokenAction::Shift(*target));
            }

            goto_table.add_state();
            for (non_terminal, target) in non_terminal_transitions.iter().enumerate() {
                goto_table[(state_id, non_terminal)] = *target;
            }
        }

        (token_table, eof_table, goto_table)
    }
}

impl From<SymbolicGrammar> for LalrAutomaton {
    fn from(value: SymbolicGrammar) -> Self {
        Self::compute(value)
    }
}

impl Display for LalrAutomaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SlrAutomaton:")?;
        writeln!(f, "States:")?;
        for (state_id, state) in self.states.iter().enumerate() {
            writeln!(
                f,
                "{state_id}: {{{}}}",
                state
                    .kernel
                    .iter()
                    .map(|item| {
                        let production = self.grammar.get_production(item.production_id).unwrap();
                        let (before_marker, after_marker) =
                            production.body().split_at(item.marker_position);
                        format!(
                            "{} -> {}·{}, {}",
                            production.head(),
                            before_marker.iter().format(" "),
                            after_marker.iter().format(" "),
                            item.lookahead_node.compute_lookahead(),
                        )
                    })
                    .format(", ")
            )?;
        }
        write!(f, "{}", self.transitions)
    }
}
