use crate::symbol::Symbol;

#[macro_export]
macro_rules! expansion {
    ($($val:expr),*) => {
        Expansion::new(vec![$($val),*])
    };
}

pub struct Expansion {
    symbols: Vec<Symbol>,
}

impl Expansion {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self { symbols }
    }
}
