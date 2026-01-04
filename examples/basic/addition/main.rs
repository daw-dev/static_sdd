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

#[test]
fn showcase() {
    use addition_grammar::*;

    let str = "1+2+3";
    let mut lex = vec![
        Token::Id(1f32),
        Token::Plus(Plus),
        Token::Id(2f32),
        Token::Plus(Plus),
        Token::Id(3f32),
    ]
    .into_iter();
    let Some(Token::Id(id1)) = lex.next() else {
        unreachable!()
    };
    let e1 = P2::synthesize(&mut (), id1);
    let Some(Token::Plus(p1)) = lex.next() else {
        unreachable!()
    };
    let Some(Token::Id(id2)) = lex.next() else {
        unreachable!()
    };
    let e2 = P2::synthesize(&mut (), id2);
    let e12 = P1::synthesize(&mut (), (e1, p1, e2));
    let Some(Token::Plus(p2)) = lex.next() else {
        unreachable!()
    };
    let Some(Token::Id(id3)) = lex.next() else {
        unreachable!()
    };
    let e3 = P2::synthesize(&mut (), id3);
    let e123 = P1::synthesize(&mut (), (e12, p2, e3));
    assert_eq!(e123, 6f32);
    assert!(lex.next().is_none());
}

#[test]
fn table_test() {
    const A: addition_grammar::Action = addition_grammar::ACTION_TABLE[0][0];
}

use addition_grammar::*;

fn main() {
    let res = parse((), [Token::Id(1f32), Token::Plus(Plus), Token::Id(6f32)]);
    println!("result is {res}");
}
