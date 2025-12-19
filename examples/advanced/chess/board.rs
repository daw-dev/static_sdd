use crate::{
    chess_move::Move,
    piece::{Color, Piece, PieceType},
};

#[derive(Default)]
pub struct Board {
    board: [[Option<Piece>; 8]; 8],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            board: [[None; 8]; 8],
        }
    }

    pub fn starting_board() -> Self {
        Self {
            board: [
                [
                    PieceType::Rook,
                    PieceType::Knight,
                    PieceType::Bishop,
                    PieceType::Queen,
                    PieceType::King,
                    PieceType::Bishop,
                    PieceType::Knight,
                    PieceType::Rook,
                ]
                .map(|pice_type| Some(Piece::new(Color::White, pice_type))),
                [PieceType::Pawn; 8].map(|piece_type| Some(Piece::new(Color::White, piece_type))),
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [PieceType::Pawn; 8].map(|piece_type| Some(Piece::new(Color::Black, piece_type))),
                [
                    PieceType::Rook,
                    PieceType::Knight,
                    PieceType::Bishop,
                    PieceType::Queen,
                    PieceType::King,
                    PieceType::Bishop,
                    PieceType::Knight,
                    PieceType::Rook,
                ]
                .map(|pice_type| Some(Piece::new(Color::Black, pice_type))),
            ],
        }
    }

    pub fn do_move(&mut self, chess_move: Move) {}
}
