use static_sdd::*;

#[grammar]
mod addition_grammar {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type E = f32;

    #[token = r"\d+(\.\d+)?"]
    pub type Id = f32;

    #[token = "+"]
    pub struct Plus;

    production!(P1, E -> (E, Plus, Id), |(e, _, id)| e + id);

    production!(P2, E -> Id);
}

fn main() {
    addition_grammar::parse("1+2+3");
}
