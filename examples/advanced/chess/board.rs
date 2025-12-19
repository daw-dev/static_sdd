use crate::{chess_move::Move, piece::Piece};

pub struct Board {
    board: [[Option<Piece>; 8]; 8],
}

impl Board {
    pub fn do_move(&mut self, chess_move: Move) {

    }
}
