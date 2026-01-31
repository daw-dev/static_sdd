use crate::{EofAction, Stacks, TokenAction};

pub trait Tables<NonTerminal, Token, Prod> {
    fn query_token_table(current_state: usize, current_token: &Token) -> Option<TokenAction<Prod>>;
    fn query_eof_table(current_state: usize) -> Option<EofAction<Prod>>;
    fn query_goto_table(current_state: usize, non_terminal: &NonTerminal) -> Option<usize>;
    fn tokens_in_state(current_state: usize) -> &'static[&'static str];
}

pub trait Reduce<NonTerminal, Token, Ctx> {
    fn reduce(&self, ctx: &mut Ctx, stacks: &mut Stacks<NonTerminal, Token>) -> NonTerminal;
}

