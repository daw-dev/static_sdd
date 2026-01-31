use std::fmt::{Display, Pointer};

use itertools::Itertools;
use logos::Logos;

use crate::{Parser, Reduce, Tables};

#[derive(Debug)]
pub enum ParseToken<Token> {
    Shifted,
    Reduced { leftover_token: Token },
}

#[derive(Debug)]
pub enum ParseEof {
    Reduced,
    Accepted,
}

#[derive(Debug)]
pub enum ParseTokenError<NonTerminal, Token> {
    ActionNotFound { leftover_token: Token },
    GotoNotFound { leftover_non_terminal: NonTerminal },
}

#[derive(Debug)]
pub enum ParseEofError<NonTerminal> {
    ActionNotFound,
    GotoNotFound { leftover_non_terminal: NonTerminal },
}

#[derive(Debug)]
pub enum ParseOneError<NonTerminal, Token> {
    ParseTokenError(ParseTokenError<NonTerminal, Token>),
    ParseEofError(ParseEofError<NonTerminal>),
}

#[derive(Debug)]
pub struct ParseError<
    NonTerminal: Into<StartSymbol>,
    Token,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> {
    parser: Parser<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>,
    parse_one_error: ParseOneError<NonTerminal, Token>,
}

impl<
    NonTerminal: Into<StartSymbol>,
    Token,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> ParseError<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>
{
    pub fn new(
        parser: Parser<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>,
        parse_one_error: ParseOneError<NonTerminal, Token>,
    ) -> Self {
        Self {
            parser,
            parse_one_error,
        }
    }
}

impl<
    NonTerminal: Into<StartSymbol> + Display,
    Token: Display,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> Display for ParseError<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ParseError: after [{}] expected any of [{}]",
            self.parser.stacks.symbol_stack.iter().format(", "),
            Tab::tokens_in_state(self.parser.stacks.current_state()).iter().format(", ")
        )
    }
}

#[derive(Debug)]
pub struct LexError<
    'source,
    NonTerminal: Into<StartSymbol>,
    Token: Logos<'source>,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> {
    parser: Parser<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>,
    lexer_error: Token::Error,
}

impl<
    'source,
    NonTerminal: Into<StartSymbol>,
    Token: Logos<'source>,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> LexError<'source, NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>
{
    pub fn new(
        parser: Parser<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>,
        lexer_error: Token::Error,
    ) -> Self {
        Self {
            parser,
            lexer_error,
        }
    }
}

#[derive(Debug)]
pub enum LexParseError<
    'source,
    NonTerminal: Into<StartSymbol>,
    Token: Logos<'source>,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> {
    LexError(LexError<'source, NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>),
    ParseError(ParseError<NonTerminal, Token, StartSymbol, Prod, Tab, Ctx>),
}

impl<
    'source,
    NonTerminal: Into<StartSymbol> + Display,
    Token: Logos<'source> + Display,
    StartSymbol,
    Prod: Reduce<NonTerminal, Token, Ctx>,
    Tab: Tables<NonTerminal, Token, Prod>,
    Ctx,
> Display for LexParseError<'source, NonTerminal, Token, StartSymbol, Prod, Tab, Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexParseError::LexError(lex_error) => lex_error.fmt(f),
            LexParseError::ParseError(parse_error) => Display::fmt(parse_error, f),
        }
    }
}
