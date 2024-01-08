// move.rs
use crate::piece::PieceType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub from: u8,     // Square number 0-63 from which the piece is moved
    pub to: u8,       // Square number 0-63 to which the piece is moved
    pub promotion: Option<PieceType>, // None if not a promotion, otherwise the piece type to which the pawn is promoted
    pub piece_type: PieceType,
    // Additional fields as necessary for en passant, castling, etc.
}

impl Move {
    // Constructor method
    pub fn new(from: u8, to: u8, piece_type: PieceType, promotion: Option<PieceType>) -> Self {
        Self {
            from,
            to,
            promotion,
            piece_type,
            // Initialize additional fields as necessary
        }
    }

    // Getter methods to access fields
    pub fn from(&self) -> u8 {
        self.from
    }

    pub fn to(&self) -> u8 {
        self.to
    }

    // Get UCI notation, e.g. e2e4
    pub fn get_uci(&self) -> String {
        let from_file = (self.from % 8) as u8 + b'a'; // converting file to a-h
        let from_rank = (self.from / 8) as u8 + 1;    // rank as 1-8
        let to_file = (self.to % 8) as u8 + b'a';     // converting file to a-h
        let to_rank = (self.to / 8) as u8 + 1;        // rank as 1-8
    
        format!("{}{}{}{}", from_file as char, from_rank, to_file as char, to_rank)
    }
}

// Move scoring structure
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i32,
}

impl ScoredMove {
    // Constructor method for ScoredMove
    pub fn new(mv: Move, score: i32) -> Self {
        Self { mv, score }
    }
    
}
