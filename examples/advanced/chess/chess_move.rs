use crate::{piece::PieceType, tile::Tile};

pub struct SimpleMove {
    piece_type: PieceType,
    target_tile: Tile,
    source_file: Option<char>,
    source_rank: Option<usize>,
    takes: bool,
    check: bool,
    checkmate: bool,
    promotion_piece_type: Option<PieceType>,
}

pub enum Castling {
    KingSide,
    QueenSide,
}

pub enum Move {
    SimpleMove(SimpleMove),
    Castling(Castling),
}
