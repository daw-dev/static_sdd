pub enum Symbol<NonTerminal, Token> {
    NonTerminal(NonTerminal),
    Token(Token),
}

