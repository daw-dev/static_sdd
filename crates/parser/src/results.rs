pub enum ParseOne<Token> {
    Shifted,
    Reduced { leftover_token: Token },
}

pub enum ParseOneEof {
    Reduced,
    Accepted,
}

pub enum ParseOneError {
    ActionNotFound,
    GotoNotFound,
}

pub enum ParseOneEofError {
    ActionNotFound,
    GotoNotFound,
}

