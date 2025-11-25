use static_sdd::*;

#[grammar]
mod balanced {
    use super::*;

    #[token = "a"]
    struct A;

    #[token = "b"]
    struct B;

    #[non_terminal]
    #[start_symbol]
    struct S {
        count: usize,
    }

    production!(P1, S -> (A, S, B), |(_, s, _)| S { count: s.count + 1 });

    production!(P2, S -> (), |_| S { count: 0 });
}

fn main() {
    parse();
}
