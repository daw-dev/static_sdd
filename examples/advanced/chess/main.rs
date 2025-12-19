mod board;
mod chess_move;
mod piece;
mod tile;

use static_sdd::*;

#[grammar]
mod chess {
    use super::*;

    #[context]
    use crate::board::Board;

    #[non_terminal]
    #[start_symbol]
    pub enum Game {
        WhiteWon,
        BlackWon,
        StaleMate,
    }

    #[non_terminal]
    pub type SetupString = ();

    #[non_terminal]
    pub type Moves = Board;

    #[non_terminal]
    pub type Move = chess_move::Move;

    #[token = "0-0|O-O"]
    pub struct KingSideCastling;

    #[token = "0-0-0|O-O-O"]
    pub struct QueenSideCastling;

    production!(G, Game -> (SetupString, Moves), |board, _| {
        todo!()
    });

    production!(S0, SetupString -> (), |board, _| *board = Board::starting_board());

    production!(M0, Game -> Moves, |_| todo!());

    production!(M1, Moves -> (Moves, Move), |_| todo!());

    production!(M2, Moves -> (), |_| todo!());
}

fn main() {}
