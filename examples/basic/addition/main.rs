use static_sdd::grammar;
use static_sdd::production;

#[grammar]
mod addition_grammar {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type E = f32;

    #[non_terminal]
    // #[start_symbol]
    pub type T = f32;

    #[token = r"\d+(\.\d+)?"]
    pub type Id = f32;

    #[token = "+"]
    pub struct Plus;

    production!(P1, E -> (E, Plus, T), |(e, _, t)| e + t);

    production!(P2, E -> T, |t| t);

    production!(P3, T -> Id, |id| id);
}

fn main() {
    addition_grammar::parse("1+2+3");
}
