use static_sdd::*;

#[grammar]
mod ambiguous {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type E = usize;

    #[token = r"\d+"]
    pub type Id = usize;

    #[token = "+"]
    pub struct Plus;
    
    #[token = "-"]
    pub struct Minus;

    #[token = "*"]
    pub struct Times;

    #[token = "/"]
    pub struct Division;

    #[token = "^"]
    pub struct Power;

    #[token = "("]
    pub struct OpenPar;

    #[token = ")"]
    pub struct ClosePar;

    #[precedence = 0]
    #[left_associative]
    production!(P1, E -> (E, Plus, E), |(e1, _, e2)| e1 + e2);

    #[precedence = 0]
    #[left_associative]
    production!(P2, E -> (E, Minus, E), |(e1, _, e2)| e1 - e2);

    #[precedence = 1]
    #[left_associative]
    production!(P3, E -> (E, Times, E), |(e1, _, e2)| e1 * e2);

    #[precedence = 1]
    #[left_associative]
    production!(P4, E -> (E, Division, E), |(e1, _, e2)| e1 * e2);

    #[precedence = 2]
    #[right_associative]
    production!(P5, E -> (E, Power, E), |(e1, _, e2)| e1.pow(e2 as u32));

    #[precedence = 3]
    production!(P6, E -> (OpenPar, E, ClosePar), |(_, e, _)| e);

    #[precedence = 3]
    production!(P7, E -> Id);
}

fn main() {
    ambiguous::parse("(1+2)*3^2-5");
}
