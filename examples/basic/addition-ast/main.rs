use semasia::*;

#[grammar]
mod ast {
    use super::*;

    #[derive(Debug)]
    pub enum ExprNode {
        Plus(Box<ExprNode>, Box<ExprNode>),
        Value(usize),
    }

    impl ExprNode {
        pub fn compute(self) -> usize {
            match self {
                ExprNode::Plus(left, right) => left.compute() + right.compute(),
                ExprNode::Value(v) => v,
            }
        }
    }

    #[non_terminal]
    #[start_symbol]
    pub type E = ExprNode;

    #[token(regex = r"\d+")]
    pub type Id = usize;

    #[token("+")]
    pub struct Plus;

    production!(P1, E -> (E, Plus, Id), |(e, _, t)| ExprNode::Plus(Box::new(e), Box::new(ExprNode::Value(t))));

    production!(P3, E -> Id, |id| ExprNode::Value(id));
}

fn main() {
    let res = ast::Parser::lex_parse("1+2+3").expect("couldn't parse");
    println!("result is {res:?}");
    println!("=> {}", res.compute());
}
