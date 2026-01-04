use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{
    parsing::action::Action,
    symbolic_grammar::{SymbolicSymbol, SymbolicToken},
};

#[derive(Debug)]
pub struct ActionTable {
    tokens_count: usize,
    table: Vec<Vec<Option<Action>>>,
}

impl ActionTable {
    pub fn new(tokens_count: usize) -> Self {
        Self {
            tokens_count,
            table: Vec::new(),
        }
    }

    pub fn add_state(&mut self) -> usize {
        let state_id = self.table.len();
        self.table.push(vec![None; self.tokens_count + 1]);
        state_id
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.table.len(), self.tokens_count + 1)
    }
}

impl Index<(usize, SymbolicSymbol)> for ActionTable {
    type Output = Option<Action>;

    fn index(&self, (state, symbol): (usize, SymbolicSymbol)) -> &Self::Output {
        match symbol {
            SymbolicSymbol::Token(token) => &self.table[state][token],
            SymbolicSymbol::EOF => &self.table[state][self.tokens_count],
            SymbolicSymbol::NonTerminal(_) => {
                panic!("you shouldn't index the action table with a non terminal")
            }
        }
    }
}

impl IndexMut<(usize, SymbolicSymbol)> for ActionTable {
    fn index_mut(&mut self, (state, symbol): (usize, SymbolicSymbol)) -> &mut Self::Output {
        match symbol {
            SymbolicSymbol::Token(token) => &mut self.table[state][token],
            SymbolicSymbol::EOF => &mut self.table[state][self.tokens_count],
            SymbolicSymbol::NonTerminal(_) => {
                panic!("you shouldn't index the action table with a non terminal")
            }
        }
    }
}

impl Display for ActionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", " ".repeat(5))?;
        for i in 0..self.tokens_count {
            write!(f, "{:^5}", i)?;
        }
        write!(f, "{:^5}", "$")?;
        writeln!(f)?;
        for (state, row) in self.table.iter().enumerate() {
            write!(f, "{:^5}", state)?;
            for elem in row.iter() {
                match elem {
                    Some(target) => {
                        write!(
                            f,
                            "{:^5}",
                            match target {
                                Action::Shift(state) => format!("S{state}"),
                                Action::Reduce(_) => format!("R{state}"),
                                Action::Accept => "Acc".to_string(),
                            }
                        )
                    }
                    None => write!(f, "{}", " ".repeat(5)),
                }?
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GoToTable {
    non_terminals_count: usize,
    table: Vec<Vec<Option<usize>>>,
}

impl GoToTable {
    pub fn new(non_terminals_count: usize) -> Self {
        Self {
            non_terminals_count,
            table: Vec::new(),
        }
    }

    pub fn add_state(&mut self) -> usize {
        let state_id = self.table.len();
        self.table.push(vec![None; self.non_terminals_count]);
        state_id
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.table.len(), self.non_terminals_count)
    }
}

impl Index<(usize, SymbolicSymbol)> for GoToTable {
    type Output = Option<usize>;

    fn index(&self, (state, symbol): (usize, SymbolicSymbol)) -> &Self::Output {
        match symbol {
            SymbolicSymbol::NonTerminal(non_terminal) => &self.table[state][non_terminal],
            SymbolicSymbol::Token(_) => {
                panic!("you shouldn't index the action table with a token!")
            }
            SymbolicSymbol::EOF => panic!("you shouldn't index the action table with $!"),
        }
    }
}

impl IndexMut<(usize, SymbolicSymbol)> for GoToTable {
    fn index_mut(&mut self, (state, symbol): (usize, SymbolicSymbol)) -> &mut Self::Output {
        match symbol {
            SymbolicSymbol::NonTerminal(non_terminal) => &mut self.table[state][non_terminal],
            SymbolicSymbol::Token(_) => {
                panic!("you shouldn't index the action table with a token!")
            }
            SymbolicSymbol::EOF => panic!("you shouldn't index the action table with $!"),
        }
    }
}

impl Display for GoToTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", " ".repeat(5))?;
        for i in 0..self.non_terminals_count {
            write!(f, "{:^5}", i)?;
        }
        writeln!(f)?;
        for (state, row) in self.table.iter().enumerate() {
            write!(f, "{:^5}", state)?;
            for elem in row.iter() {
                match elem {
                    Some(target) => write!(f, "{:^5}", target),
                    None => write!(f, "{}", " ".repeat(5)),
                }?
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct TransitionTables {
    token_table: Vec<Vec<Option<usize>>>,
    non_terminal_table: Vec<Vec<Option<usize>>>,
}

impl TransitionTables {
    pub fn new() -> Self {
        Self {
            token_table: Vec::new(),
            non_terminal_table: Vec::new(),
        }
    }

    pub fn add_transitions(
        &mut self,
        token_transitions: Vec<Option<usize>>,
        non_terminal_transitions: Vec<Option<usize>>,
    ) {
        self.token_table.push(token_transitions);
        self.non_terminal_table.push(non_terminal_transitions);
    }

    pub fn token_transition(&self, starting_state: usize, token: SymbolicToken) -> Option<usize> {
        self.token_table
            .get(starting_state)?
            .get(token)
            .cloned()
            .flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Vec<Option<usize>>, &Vec<Option<usize>>)> {
        self.token_table.iter().zip(self.non_terminal_table.iter())
    }
}

impl Display for TransitionTables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TransitionTables:")?;
        write!(f, "{}", " ".repeat(5))?;
        for i in 0..self.non_terminal_table[0].len() {
            write!(f, "{:^5}", i)?;
        }
        for i in 0..self.token_table[0].len() {
            write!(f, "{:^5}", format!("`{i}`"))?;
        }
        writeln!(f)?;
        for (state, (nt_row, tok_row)) in self
            .non_terminal_table
            .iter()
            .zip(self.token_table.iter())
            .enumerate()
        {
            write!(f, "{:^5}", state)?;
            for elem in nt_row.iter() {
                match elem {
                    Some(target) => write!(f, "{:^5}", target),
                    None => write!(f, "{}", " ".repeat(5)),
                }?
            }
            for elem in tok_row.iter() {
                match elem {
                    Some(target) => write!(f, "{:^5}", target),
                    None => write!(f, "{}", " ".repeat(5)),
                }?
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
