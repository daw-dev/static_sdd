use semasia::*;

#[grammar]
mod balanced {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type S = usize;

    #[token("a")]
    pub struct A;

    #[token("b")]
    pub struct B;

    production!(P1, S -> (A, S, B), |(_, s, _)| s + 1);

    production!(P2, S -> (), |_| 0);
}

fn main() {
    let res = balanced::Parser::lex_parse("aaabbb").expect("couldn't parse");
    println!("{res}");
}
