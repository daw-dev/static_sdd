use static_sdd::*;

#[grammar]
mod ast {
    use super::*;

    enum Operator {
        Plus,
    }

    enum ExprNode {
        BinExpr(Box<ExprNode>, Operator, Box<ExprNode>),
        Value(usize),
    }

    #[non_terminal]
    #[start_symbol]
    type E = ExprNode;

    #[non_terminal]
    type T = ExprNode;

    #[token = ""]
    type Id = usize;

    #[token = ""]
    struct Plus;

    production!(P1, E -> (E, Plus, T), |(e, _, t)| ExprNode::BinExpr(Box::new(e), Operator::Plus, Box::new(t)));

    production!(P2, E -> T, |t| t);

    production!(P3, T -> Id, |id| ExprNode::Value(id));
}

fn main() {
    parse();
}
