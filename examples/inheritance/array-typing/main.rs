use semasia::*;

#[grammar]
mod arrays {
    use super::*;

    #[derive(Debug)]
    pub enum ComputedType {
        BaseType(String),
        Array(usize, Box<ComputedType>),
    }

    #[non_terminal]
    #[start_symbol]
    pub type T = ComputedType;

    #[non_terminal]
    pub type Computed = FromInherited<String, ComputedType>;

    #[non_terminal]
    pub type Base = String;

    #[token("int")]
    pub struct Int;

    #[token("float")]
    pub struct Float;

    #[token("[")]
    pub struct LeftSquarePar;

    #[token("]")]
    pub struct RightSquarePar;

    #[token(regex = r"\d+")]
    pub type Size = usize;

    production!(FinalType, T -> (Base, Computed), |(b, c)| c.resolve(b));

    production!(ArrayType, Computed -> (LeftSquarePar, Size, RightSquarePar, Computed), |(_, size, _, c)|
        c.map(move |val| ComputedType::Array(size, Box::new(val)))
    );

    production!(NoArray, Computed -> (), |_| FromInherited::new(ComputedType::BaseType));

    production!(BaseIsInt, Base -> Int, |_| "int".to_string());

    production!(BaseIsFloat, Base -> Float, |_| "int".to_string());
}

#[test]
fn array_test() {
    use arrays::*;

    let base_type = Base::from("int");
    let c3 = P3::synthesize(());
    let c2 = P2::synthesize((LeftSquarePar, 3, RightSquarePar, c3));
    let c1 = P2::synthesize((LeftSquarePar, 2, RightSquarePar, c2));
    let t = P1::synthesize((base_type, c1));

    println!("{t:?}");
}

fn main() {
    let res = arrays::Parser::lex_parse("int[2][3]").expect("couldn't parse");
    println!("{res:?}");
}
