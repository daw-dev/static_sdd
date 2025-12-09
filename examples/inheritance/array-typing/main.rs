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
    pub struct C {
        base_type: Inherited<String>,
        computed_type: Deferred<ComputedType>,
    }

    #[token = "int|float"]
    pub type B = String;

    #[token = "["]
    pub struct LeftSquarePar;

    #[token = "]"]
    pub struct RightSquarePar;

    #[token = r"\d+"]
    pub type Size = usize;

    production!(P1, T -> (B, C), |(b, mut c)| {
        c.base_type.set(b);
        c.computed_type.unwrap()
    });

    production!(P2, C -> (LeftSquarePar, Size, RightSquarePar, C), |(_, size, _, c)| {
        C {
            base_type: Inherited::inherit(c.base_type),
            computed_type: c.computed_type.map(move |t| ComputedType::Array(size, Box::new(t))),
        }
    });

    production!(P3, C -> (), |_| {
        let (from, into) = Inherited::channel_map(ComputedType::BaseType);

        C {
            base_type: from,
            computed_type: into,
        }
    });
}

#[test]
fn array_test() {
    use arrays::*;

    let base_type = B::from("int");
    let c3 = P3::synthesize(());
    let c2 = P2::synthesize((LeftSquarePar, 3, RightSquarePar, c3));
    let c1 = P2::synthesize((LeftSquarePar, 2, RightSquarePar, c2));
    let t = P1::synthesize((base_type, c1));

    println!("{t:?}");
}

fn main() {
    arrays::parse("int[2][3]");
}
