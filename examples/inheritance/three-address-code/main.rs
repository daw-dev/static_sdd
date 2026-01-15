use static_sdd::*;

#[grammar]
mod compiler {
    use std::ops::Shl;
    use super::*;

    pub enum Statement {
        Label(String),
        GoTo(String),
        BinOp(String, String, String, String),
        UnOp(String, String, String),
    }

    #[context]
    #[derive(Default)]
    pub struct CompilationContext {
        current_label: usize,
    }

    impl CompilationContext {
        fn new_label(&mut self) -> String {
            let res = format!("L{}", self.current_label);
            self.current_label += 1;
            res
        }
    }

    #[non_terminal]
    #[start_symbol]
    pub type Program = Code;

    pub struct Code {
        lines: Vec<Statement>,
    }

    impl Code {
        fn new() -> Self {
            Self { lines: Vec::new() }
        }
    }

    impl FromIterator<Statement> for Code {
        fn from_iter<T: IntoIterator<Item = Statement>>(iter: T) -> Self {
            Code {
                lines: iter.into_iter().collect(),
            }
        }
    }

    impl Shl<Statement> for Code {
        type Output = Code;

        fn shl(mut self, other: Statement) -> Self::Output {
            self.lines.push(other);
            self
        }
    }

    impl<I> Shl<I> for Code
    where
        I: IntoIterator<Item = Statement>,
    {
        type Output = Code;

        fn shl(mut self, other: I) -> Self::Output {
            self.lines.extend(other);
            self
        }
    }

    impl Shl<Code> for Code {
        type Output = Code;

        fn shl(self, other: Code) -> Self::Output {
            self << other.lines
        }
    }

    #[non_terminal]
    pub type FutureStatement = FromInherited<String, Code>;

    #[derive(Clone)]
    pub struct ConditionLabels {
        t: Option<String>,
        f: Option<String>,
    }

    #[non_terminal]
    pub type Condition = FromInherited<ConditionLabels, Code>;

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

    production!(ProgramIsStatement, Program -> FutureStatement, |ctx, s| s.resolve(ctx.new_label()));

    production!(StatementIsStatements, FutureStatement -> (FutureStatement, FutureStatement), |ctx, (s1, s2)| {
        let s1_next = ctx.new_label();
        FromInherited::new(|s_next| {
            s1.resolve(s1_next) << s2.resolve(s_next)
        })
    });

    production!(IfStatement, FutureStatement -> (If, Condition, FutureStatement), |(_, b, s)| {
        FromInherited::new(|s_next: String| {
            b.resolve(ConditionLabels { t: None, f: Some(s_next.clone()) }) << s.resolve(s_next)
        })
    });

    production!(OrCondition, Condition -> (Condition, OrOp, Condition), |ctx, (b1, _, b2)| {
        let b1_true = ctx.new_label();
        FromInherited::new(|b_labels: ConditionLabels| {
            let b1_true = b_labels.t.clone().or_else(|| Some(b1_true));
            b1.resolve(ConditionLabels {
                t: b1_true.clone(),
                f: None,
            }) << b2.resolve(ConditionLabels {
                t: b_labels.t,
                f: b_labels.f,
            }) << b1_true.map(Statement::GoTo)
        })
    });

    production!(SkipStatement, FutureStatement -> Skip, |_| FromInherited::new(|_| Code::new()));

    production!(TrueCondition, Condition -> True, |_| FromInherited::new(|b_labels: ConditionLabels| {
        b_labels.t.map(Statement::GoTo).into_iter().collect()
    }));

    production!(FalseCondition, Condition -> False, |_| FromInherited::new(|b_labels: ConditionLabels| {
        b_labels.f.map(Statement::GoTo).into_iter().collect()
    }));
}

use compiler::*;

fn main() {
    let res = parse(Default::default(), [
        Token::If(If),
        Token::True(True),
        Token::OrOp(OrOp),
        Token::False(False),
        Token::Skip(Skip),
    ]).expect("couldn't parse");
}
