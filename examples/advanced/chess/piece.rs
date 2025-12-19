pub enum Color {
    Black,
    White,
}

pub enum PieceType {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

pub struct Piece {
    color: Color,
    piece_type: PieceType,
}
