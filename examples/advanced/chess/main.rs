mod board;
mod chess_move;
mod piece;
mod tile;

use static_sdd::*;

#[grammar]
mod chess {
    use crate::board::Board;
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub enum Game {
        WhiteWon,
        BlackWon,
        StaleMate,
    }

    #[non_terminal]
    pub type SetupString = Board;

    #[non_terminal]
    pub type Moves = Board;

    #[non_terminal]
    pub type Move = chess_move::Move;

    #[token = "0-0|O-O"]
    pub struct KingSideCastling;

    #[token = "0-0-0|O-O-O"]
    pub struct QueenSideCastling;

    production!(P0, Game -> (SetupString, Moves), |_| todo!());

    production!(P1, Game -> Moves, |_| todo!());

    production!(P2, Moves -> (Moves, Move), |_| todo!());

    production!(P3, Moves -> (), |_| todo!());
}

fn main() {

}
