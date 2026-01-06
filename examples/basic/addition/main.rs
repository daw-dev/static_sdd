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

use addition_grammar::*;

fn main() {
    let res = parse((), [Token::Id(1f32), Token::Plus(Plus), Token::Id(6f32)]);
    match res {
        Ok(res) => println!("{res}"),
        Err(stacks) => println!("{stacks}"),
    }
}
