use static_sdd::*;

#[grammar]
mod ast {
    use super::*;

    enum Operator {
        Plus,
    }

    impl Operator {
        fn compute_binary(self, left: usize, right: usize) -> usize {
            match self {
                Operator::Plus => left + right,
            }
        }
    }

    enum ExprNode {
        BinExpr(Box<ExprNode>, Operator, Box<ExprNode>),
        Value(usize),
    }

    impl ExprNode {
        pub fn compute(self) -> usize {
            match self {
                ExprNode::BinExpr(left, op, right) => op.compute_binary(left, right),
                ExprNode::Value(v) => v,
            }
        }
    }

    #[non_terminal]
    #[start_symbol]
    type E = ExprNode;

    #[non_terminal]
    type T = ExprNode;

    #[token = r"\d+"]
    type Id = usize;

    #[token = r"\+"]
    struct Plus;

    production!(P1, E -> (E, Plus, T), |(e, _, t)| ExprNode::BinExpr(Box::new(e), Operator::Plus, Box::new(t)));

    production!(P2, E -> T, |t| t);

    production!(P3, T -> Id, |id| ExprNode::Value(id));
}

fn main() {
    parse();
}
