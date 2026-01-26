use std::marker::PhantomData;
use symbol::Symbol;
use crate::results::{ParseOne, ParseOneEof, ParseOneEofError, ParseOneError};

mod results;
mod symbol;

pub enum TokenAction<Prod> {
    Shift(usize),
    Reduce(Prod),
}

pub enum EofAction<Prod> {
    Reduce(Prod),
    Accept
}

pub struct Stacks<NonTerminal, Token> {
    state_stack: Vec<usize>,
    symbol_stack: Vec<Symbol<NonTerminal, Token>>,
}

impl<NonTerminal, Token> Stacks<NonTerminal, Token> {
    pub fn new() -> Self {
        Self {
            state_stack: vec![0],
            symbol_stack: Vec::new(),
        }
    }

    pub fn current_state(&self) -> usize {
        *self.state_stack.last().expect("state stack is empty!")
    }

    pub fn shift(&mut self, new_state: usize, token: Token) {
        self.state_stack.push(new_state);
        self.symbol_stack.push(Symbol::Token(token));
    }

    pub fn goto(&mut self, new_state: usize, non_terminal: NonTerminal) {
        self.state_stack.push(new_state);
        self.symbol_stack.push(Symbol::NonTerminal(non_terminal));
    }
}

pub trait Tables<NonTerminal, Token, Prod> {
    fn query_token_table(current_state: usize, current_token: &Token) -> Option<TokenAction<Prod>>;
    fn query_eof_table(current_state: usize) -> Option<EofAction<Prod>>;
    fn query_goto_table(current_state: usize, non_terminal: &NonTerminal) -> Option<usize>;
}

pub trait Reduce<NonTerminal, Token, Ctx> {
    fn reduce(&self, ctx: &mut Ctx, stacks: &mut Stacks<NonTerminal, Token>) -> NonTerminal;
}

pub struct Parser<
    NonTerminal,
    Token,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> {
    stacks: Stacks<NonTerminal, Token>,
    ctx: Ctx,
    phantom_data: PhantomData<(Prod, Tab)>,
}

impl<
    NonTerminal,
    Token,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> Parser<NonTerminal, Token, Prod, Tab, Ctx>
{
    pub fn new(ctx: Ctx) -> Self {
        Self {
            stacks: Stacks::new(),
            ctx,
            phantom_data: PhantomData,
        }
    }

    pub fn parse_one(&mut self, token: Token) -> Result<ParseOne<Token>, ParseOneError> {
        let current_state = self.stacks.current_state();
        match Tab::query_token_table(current_state, &token) {
            Some(TokenAction::Shift(new_state)) => {
                self.stacks.shift(new_state, token);
                Ok(ParseOne::Shifted)
            }
            Some(TokenAction::Reduce(prod)) => {
                let head = prod.reduce(&mut self.ctx, &mut self.stacks);
                let new_current_state = self.stacks.current_state();
                let Some(next_state) = Tab::query_goto_table(new_current_state, &head) else {
                    return Err(ParseOneError::GotoNotFound);
                };
                self.stacks.goto(next_state, head);
                Ok(ParseOne::Reduced {
                    leftover_token: token,
                })
            }
            None => Err(ParseOneError::ActionNotFound),
        }
    }

    pub fn parse_one_eof(&mut self) -> Result<ParseOneEof, ParseOneEofError>{
        let current_state = self.stacks.current_state();
        match Tab::query_eof_table(current_state) {
            Some(EofAction::Reduce(prod)) => {
                let head = prod.reduce(&mut self.ctx, &mut self.stacks);
                let new_current_state = self.stacks.current_state();
                let Some(next_state) = Tab::query_goto_table(new_current_state, &head) else {
                    return Err(ParseOneEofError::GotoNotFound);
                };
                self.stacks.goto(next_state, head);
                Ok(ParseOneEof::Reduced)
            }
            Some(EofAction::Accept) => {
                Ok(ParseOneEof::Accepted)
            }
            None => {
                Err(ParseOneEofError::ActionNotFound)
            }
        }
    }

    pub fn parse(ctx: Ctx, tokens: impl IntoIterator<Item = Token>) {
        let mut parser = Self::new(ctx);
        for mut token in tokens.into_iter() {
            loop {
                match parser.parse_one(token) {
                    Ok(ParseOne::Shifted) => {
                        break;
                    }
                    Ok(ParseOne::Reduced { leftover_token }) => {
                        token = leftover_token;
                    }
                    Err(_err) => todo!("propagate error"),
                }
            }
        }
        loop {
            match parser.parse_one_eof() {
                Ok(ParseOneEof::Accepted) => {
                    break;
                },
                Ok(ParseOneEof::Reduced) => {
                    continue;
                }
                Err(_err) => todo!("propagate error"),
            }
        }
    }
}
