use static_sdd::*;

#[grammar]
mod addition_grammar {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type E<T> = T;

    #[non_terminal]
    // #[start_symbol]
    pub type F<T> = T;

    #[token = r"\d+(\.\d+)?"]
    pub type Id<T> = T;

    #[token = "+"]
    pub struct Plus;

    production!(P1<T>, E<T> -> (E<T>, Plus, F<T>), |(e, _, t)| e + t);

    production!(P2<T>, E<T> -> F<T>, |t| t);

    production!(P3<T>, F<T> -> Id<T>, |id| id);
}

fn main() {
    addition_grammar::parse("1.38+4.12+6");
}
