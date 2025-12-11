use static_sdd::*;

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
    pub type Computed = InheritOnce<String, ComputedType>;

    #[token = "int|float"]
    pub type Base = String;

    #[token = "["]
    pub struct LeftSquarePar;

    #[token = "]"]
    pub struct RightSquarePar;

    #[token = r"\d+"]
    pub type Size = usize;

    production!(P1, T -> (Base, Computed), |(b, c)| c.resolve(b));

    production!(P2, Computed -> (LeftSquarePar, Size, RightSquarePar, Computed), |(_, size, _, c)|
        c.pass_up_with(move |val| ComputedType::Array(size, Box::new(val)))
    );

    production!(P3, Computed -> (), |_| InheritOnce::base_map(ComputedType::BaseType));
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
    arrays::parse("int[2][3]");
}
