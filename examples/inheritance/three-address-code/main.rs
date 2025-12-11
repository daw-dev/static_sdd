use static_sdd::*;

#[grammar]
mod compiler {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    pub enum Statement {
        Label(String),
        GoTo(String),
        BinOp(String, String, String, String),
        UnOp(String, String, String),
    }

    #[non_terminal]
    #[start_symbol]
    pub type P = Code;

    pub struct SNext {
        label: String,
    }

    impl PassDown for SNext {
        fn pass_down(self) -> SNext {
            self
        }
    }

    pub struct Code {
        lines: Vec<Statement>,
    }

    #[non_terminal]
    pub type S = InheritOnce<SNext, Code>;

    pub struct BLabels {
        t: String,
        f: String,
    }

    impl PassDown for BLabels {
        fn pass_down(self) -> BLabels {
            self
        }
    }

    #[non_terminal]
    pub type B = InheritOnce<BLabels, Code>;

    #[token = "skip"]
    pub struct Skip;

    #[token = "true"]
    pub struct True;

    #[token = "false"]
    pub struct False;

    #[token = "||"]
    pub struct OrOp;

    #[token = "&&"]
    pub struct AndOp;

    #[token = "if"]
    pub struct If;

    fn new_label() -> String {
        "L0".into()
    }

    production!(P0, P -> S, |s| s.resolve(SNext { label: new_label() }));

    production!(P1, S -> (S, S), |(s1, s2)|
        s2.pass_up_with(|code| {
            let mut res = s1.resolve(SNext { label: new_label() });
            res.lines.extend(code.lines);
            res
        })
    );

    production!(P2, S -> (If, B, S), |(_, b, s)| {
        let s_next = Rc::new(RefCell::new(None));
        let s_next_clone = s_next.clone();
        s.also(move |s_next| {
            *s_next_clone.borrow_mut() = Some(s_next.label.clone());
        }).pass_up_with(move |s_code| {
            let mut res = b.resolve(BLabels {
                    t: new_label(), f: Rc::try_unwrap(s_next).unwrap().into_inner().unwrap()
                });
            res.lines.extend(s_code.lines);
            res
        })
    });
}

fn main() {
    compiler::parse("hello")
}
