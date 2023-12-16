use std::collections::HashMap;
use std::time::{Instant};

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    white_queens: u64,
    white_king: u64,
    
    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    black_queens: u64,
    black_king: u64,

    en_passant: Option<u8>, // None if no en passant square, otherwise the square number (0-63)
    castling_rights: u8,    // A 4-bit value representing castling rights; each bit corresponds to a possibility
    side_to_move: bool,     // True for white, False for black
    halfmove_clock: u8,     // Number of halfmoves since the last capture or pawn advance (for the fifty-move rule)
    fullmove_number: u16,   // The number of the full move, it starts at 1, and is incremented after Black's move
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    from: u8,     // Square number 0-63 from which the piece is moved
    to: u8,       // Square number 0-63 to which the piece is moved
    promotion: Option<PieceType>, // None if not a promotion, otherwise the piece type to which the pawn is promoted
    piece_type: PieceType
    // Additional fields as necessary for en passant, castling, etc.
}
impl Move {
    // Constructor method
    pub fn new(from: u8, to: u8, piece_type: PieceType, promotion: Option<PieceType>) -> Move {
        Move {
            from,
            to,
            promotion,
            piece_type,
            // Initialize additional fields as necessary
        }
    }

    // You might also want to provide getter methods to access private fields
    pub fn from(&self) -> u8 {
        self.from
    }

    pub fn to(&self) -> u8 {
        self.to
    }

    // get uci notation, e.g. e2e4
    pub fn get_uci(&self) -> String {
        let from_square = self.from;
        let to_square = self.to;
    
        let from_file = (from_square % 8) as u8 + b'a'; // converting file to a-h
        let from_rank = (from_square / 8) as u8 + 1;    // rank as 1-8
        let to_file = (to_square % 8) as u8 + b'a';     // converting file to a-h
        let to_rank = (to_square / 8) as u8 + 1;        // rank as 1-8
    
        let mut result = String::new();
        result.push(from_file as char);
        result.push_str(&from_rank.to_string()); // Convert rank number to string
        result.push(to_file as char);
        result.push_str(&to_rank.to_string());   // Convert rank number to string
    
        result
    }
}


// Move scoring structure
pub struct ScoredMove {
    mv: Move,
    score: i32,
}

impl ScoredMove {
    fn new(mv: Move, score: i32) -> Self {
        ScoredMove { mv, score }
    }
}

#[derive(Clone, Copy)]
pub struct UndoState {
    captured_piece: Option<PieceType>, // The piece type that was captured, if any
    en_passant: Option<u8>,            // The en passant square, if any
    castling_rights: u8,               // The castling rights before the move
    halfmove_clock: u8,                // The halfmove clock before the move
    fullmove_number: u16,              // The fullmove number before the move
    // Add any other state information that needs to be restored
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}


// Helper function to get the rank (0 to 7) from a square (0 to 63)
fn get_rank(square: u8) -> u8 {
    square / 8
}

// Helper function to get the file (0 to 7) from a square (0 to 63)
fn get_file(square: u8) -> u8 {
    square % 8
}

// Represents the result of a pawn move which may affect en passant
struct PawnMoveResult {
    en_passant: Option<u8>,
}

impl Board {
    // Initializes the board to the starting position
    pub fn new() -> Self {
        Board {
            white_pawns: 0xFF00, // 2nd rank
            white_knights: 0x42, // b1 and g1
            white_bishops: 0x24, // c1 and f1
            white_rooks: 0x81,   // a1 and h1
            white_queens: 0x8,   // d1
            white_king: 0x10,    // e1
            
            black_pawns: 0xFF000000000000,   // 7th rank
            black_knights: 0x4200000000000000, // b8 and g8
            black_bishops: 0x2400000000000000, // c8 and f8
            black_rooks: 0x8100000000000000,   // a8 and h8
            black_queens: 0x800000000000000,   // d8
            black_king: 0x1000000000000000,    // e8
            
            en_passant: None,
            castling_rights: 0xF, // All castling rights available initially
            side_to_move: WHITE,
            halfmove_clock: 0,
            fullmove_number: 1,
        }

    }

    fn print_self(&self) {
        println!("{:?}", self);
    }

    pub fn make_move(&mut self, mv: Move) -> UndoState {

        // Determine if the move is a pawn advance or a capture
        let is_pawn_advance = mv.piece_type == PieceType::Pawn;
        let is_capture = self.is_occupied_by_opponent(mv.to, self.side_to_move);
        let is_en_passant_capture = is_pawn_advance && self.en_passant == Some(mv.to);

        // Initialize captured_piece variable
        let captured_piece = if is_capture {
            // If it's a capture, get the type of the captured piece
            Some(self.get_piece_type(mv.to))
        } else if is_en_passant_capture {
            // If it's an en passant capture, return pawn type
            Some(PieceType::Pawn)
        } else {
            None
        };

        //save board state so we can undo it later. combined with Move, can fully undo move.
        let undo_state = UndoState {
            //TODO: Verify unmake move and undo state work appropriately with zobrist hashes
            captured_piece: captured_piece,
            en_passant: self.en_passant,
            castling_rights: self.castling_rights,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
        };

        // let from_mask = 1u64 << mv.from;
        let to_mask = 1u64 << mv.to;

        // Handle the halfmove clock with ternary operator
        self.halfmove_clock = if is_pawn_advance || is_capture { 0 } else { self.halfmove_clock + 1 };

        // Check if the 'to' square is occupied by an opponent's piece and capture it
        if is_capture {
            self.clear_square(mv.to);
        }

        // Clear the 'from' square
        self.clear_square(mv.from);

        // Set the 'to' square for the appropriate piece
        match mv.piece_type {
            PieceType::Pawn => {
                // self.halfmove_clock = 0; // Reset the halfmove clock
                if self.side_to_move == WHITE {
                    self.white_pawns |= to_mask;
                } else {
                    self.black_pawns |= to_mask;
                }
            },

            PieceType::Knight => {
                if self.side_to_move == WHITE {
                    self.white_knights |= to_mask;
                } else {
                    self.black_knights |= to_mask;
                }
            },

            PieceType::Rook => {
                if self.side_to_move == WHITE {
                    self.white_rooks |= to_mask;
                } else {
                    self.black_rooks |= to_mask;
                }
            },

            PieceType::Bishop => {
                if self.side_to_move == WHITE {
                    self.white_bishops |= to_mask;
                } else {
                    self.black_bishops |= to_mask;
                }
            },

            PieceType::Queen => {
                if self.side_to_move == WHITE {
                    self.white_queens |= to_mask;
                } else {
                    self.black_queens |= to_mask;
                }
            },

            PieceType::King => {
                if self.side_to_move == WHITE {
                    self.white_king |= to_mask;
                } else {
                    self.black_king |= to_mask;
                }
            },
        }

        // Handle castling move
        if mv.piece_type == PieceType::King && mv.from.abs_diff(mv.to) == 2 {
            self.handle_castling(mv.to);
        }

        // Handle castling rights
        self.update_castling_rights(&mv);

        // Handle en passant
        if let Some(pawn_move) = self.handle_pawn_move(&mv) {
            self.en_passant = pawn_move.en_passant;
        } else {
            self.en_passant = None;
        }

        // Handle potential promotion
        if let Some(promotion) = mv.promotion {
            self.promote_pawn(mv.to, promotion);
        }

        // Toggle the side to move
        self.side_to_move = !self.side_to_move;

        // Update the fullmove number if Black has moved
        if self.side_to_move == WHITE {
            self.fullmove_number += 1;
        }

        //print self
        // self.print_self();
        
        // Return the undo state
        undo_state
    }

    pub fn unmake_move(&mut self, mv: Move, undo_state: UndoState) {
        //print self
        // self.print_self();

        self.clear_square(mv.to);

        // Restore the captured piece, if any
        if Some(mv.to) == undo_state.en_passant {
            let captured_square = if self.side_to_move == WHITE { mv.to + 8 } else { mv.to - 8 };
            self.set_square(captured_square, PieceType::Pawn);
        } else if let Some(captured_piece) = undo_state.captured_piece {
            self.set_square(mv.to, captured_piece);
        } else {
        }

        // Toggle the side to move back
        self.side_to_move = !self.side_to_move;

        // Restore the previous state
        self.en_passant = undo_state.en_passant;
        self.castling_rights = undo_state.castling_rights;
        self.halfmove_clock = undo_state.halfmove_clock;
        self.fullmove_number = undo_state.fullmove_number;
    
        // Move the piece back to its original square
        self.set_square(mv.from, mv.piece_type);
    
        // If the move was a castling move, move the rook back
        if mv.piece_type == PieceType::King && (mv.from.abs_diff(mv.to) == 2) {
            let (rook_from, rook_to) = match mv.to {
                2 | 58 => (0, 3),   // Queen-side castling
                6 | 62 => (7, 5),   // King-side castling
                _ => panic!("Invalid castling move during unmake"),
            };
            let rook_from = if self.side_to_move == WHITE { rook_from } else { rook_from + 56 };
            let rook_to = if self.side_to_move == WHITE { rook_to } else { rook_to + 56 };
        
            self.clear_square(rook_to); // TODO: this is probably unnecessary, since castling needs clear space beforehand
            self.set_square(rook_from, PieceType::Rook);
        }
    }

    // Clears a square on the bitboards
    fn clear_square(&mut self, square: u8) {
        let mask = !(1u64 << square);
        
        // Clear square for white pieces
        self.white_pawns &= mask;
        self.white_knights &= mask;
        self.white_bishops &= mask;
        self.white_rooks &= mask;
        self.white_queens &= mask;
        self.white_king &= mask;
    
        // Clear square for black pieces
        self.black_pawns &= mask;
        self.black_knights &= mask;
        self.black_bishops &= mask;
        self.black_rooks &= mask;
        self.black_queens &= mask;
        self.black_king &= mask;
    }

    // Check if a square is occupied by an opponent's piece
    fn is_occupied_by_opponent(&self, square: u8, side: bool) -> bool {
        let mask = 1u64 << square;
        if side == WHITE {
            // Check black pieces
            (self.black_pawns | self.black_knights | self.black_bishops |
            self.black_rooks | self.black_queens | self.black_king) & mask != 0
        } else {
            // Check white pieces
            (self.white_pawns | self.white_knights | self.white_bishops |
            self.white_rooks | self.white_queens | self.white_king) & mask != 0
        }
    }

    // Promote a pawn
    fn promote_pawn(&mut self, square: u8, promotion: PieceType) {
        // Clear the pawn from the square
        self.clear_square(square);

        let mask = 1u64 << square;
        match promotion {
            PieceType::Queen => {
                if self.side_to_move == WHITE {
                    self.white_queens |= mask;
                } else {
                    self.black_queens |= mask;
                }
            },

            PieceType::Knight => {
                if self.side_to_move == WHITE {
                    self.white_knights |= mask;
                } else {
                    self.black_knights |= mask;
                }
            },

            PieceType::Rook => {
                if self.side_to_move == WHITE {
                    self.white_rooks |= mask;
                } else {
                    self.black_rooks |= mask;
                }
            },

            PieceType::Bishop => {
                if self.side_to_move == WHITE {
                    self.white_bishops |= mask;
                } else {
                    self.black_bishops |= mask;
                }
            },

            _ => panic!("Invalid promotion"),
        }
    }

    // Handles the specifics of pawn moves, including double moves and en passant captures
    fn handle_pawn_move(&mut self, mv: &Move) -> Option<PawnMoveResult> {
        if mv.piece_type != PieceType::Pawn {
            return None;
        }
        let from_rank = get_rank(mv.from);
        let to_rank = get_rank(mv.to);
        if self.side_to_move == WHITE && from_rank == 1 && to_rank == 3 {
            // White pawn double move, set en passant square
            Some(PawnMoveResult { en_passant: Some(mv.to - 8) })
        } else if self.side_to_move == BLACK && from_rank == 6 && to_rank == 4 {
            // Black pawn double move, set en passant square
            Some(PawnMoveResult { en_passant: Some(mv.to + 8) })
        } else if self.en_passant == Some(mv.to) {
            // En passant capture
            let captured_square = if self.side_to_move == WHITE { mv.to - 8 } else { mv.to + 8 };
            self.clear_square(captured_square);
            None
        } else {
            None
        }
    }

    // Updates the castling rights given the current move
    fn update_castling_rights(&mut self, mv: &Move) {
        // If the king moves, remove both castling rights for that color
        if mv.piece_type == PieceType::King {
            if self.side_to_move == WHITE {
                self.castling_rights &= 0b1100; // Remove white's castling rights
            } else {
                self.castling_rights &= 0b0011; // Remove black's castling rights
            }
        }

        // If a rook moves or is captured, remove the corresponding castling right
        if mv.piece_type == PieceType::Rook {
            // If the rook on the original king-side or queen-side moves, update the rights
            match (mv.from, self.side_to_move) {
                (0, WHITE) => self.castling_rights &= 0b1101, // White queen-side rook
                (7, WHITE) => self.castling_rights &= 0b1110, // White king-side rook
                (56, BLACK) => self.castling_rights &= 0b0111, // Black queen-side rook
                (63, BLACK) => self.castling_rights &= 0b1011, // Black king-side rook
                _ => {}
            }
        }

        match (mv.to, self.side_to_move) {
            (0, BLACK) => self.castling_rights &= 0b1101,
            (7, BLACK) => self.castling_rights &= 0b1110, // White king-side rook
            (56, WHITE) => self.castling_rights &= 0b0111, // Black queen-side rook
            (63, WHITE) => self.castling_rights &= 0b1011, // Black king-side rook
            _ => {}
        }
        
    }

    // Determine if a move is a capture
    fn is_capture(&self, mv: &Move) -> bool {
        self.is_occupied_by_opponent(mv.to, self.side_to_move)
    }

    // Handles the specifics of castling moves
    fn handle_castling(&mut self, to: u8) {
        let rook_from: u8;
        let rook_to: u8;

        match to {
            2 => { // White queen-side castling
                rook_from = 0; // Initial rook square
                rook_to = 3;   // Rook's new square after castling
                self.castling_rights &= 0b1100; // Remove white's castling rights
            },
            6 => { // White king-side castling
                rook_from = 7;
                rook_to = 5;
                self.castling_rights &= 0b1100;
            },
            58 => { // Black queen-side castling
                rook_from = 56;
                rook_to = 59;
                self.castling_rights &= 0b0011;
            },
            62 => { // Black king-side castling
                rook_from = 63;
                rook_to = 61;
                self.castling_rights &= 0b0011;
            },
            _ => panic!("Invalid castling move"),
        }

        // Move the rook
        self.clear_square(rook_from);
        self.set_square(rook_to, PieceType::Rook);
    }

    // Sets a square on the bitboards with the specified piece type
    fn set_square(&mut self, square: u8, piece_type: PieceType) {
        let mask = 1u64 << square;

        match piece_type {

            PieceType::Rook => {
                if self.side_to_move == WHITE {
                    self.white_rooks |= mask;
                } else {
                    self.black_rooks |= mask;
                }
            },

            PieceType::Knight => {
                if self.side_to_move == WHITE {
                    self.white_knights |= mask;
                } else {
                    self.black_knights |= mask;
                }
            },

            PieceType::Bishop => {
                if self.side_to_move == WHITE {
                    self.white_bishops |= mask;
                } else {
                    self.black_bishops |= mask;
                }
            },

            PieceType::Queen => {
                if self.side_to_move == WHITE {
                    self.white_queens |= mask;
                } else {
                    self.black_queens |= mask;
                }
            },

            PieceType::Pawn => {
                if self.side_to_move == WHITE {
                    self.white_pawns |= mask;
                } else {
                    self.black_pawns |= mask;
                }
            },

            PieceType::King => {
                if self.side_to_move == WHITE {
                    self.white_king |= mask;
                } else {
                    self.black_king |= mask;
                }
            },

        }
    }

    // A very simple evaluation function
    pub fn evaluate(&self) -> i32 {
        let white_material = self.bitboard_material(self.white_pawns) * 1 +
                             self.bitboard_material(self.white_knights) * 3 +
                             self.bitboard_material(self.white_bishops) * 3 +
                             self.bitboard_material(self.white_rooks) * 5 +
                             self.bitboard_material(self.white_queens) * 9;

        let black_material = self.bitboard_material(self.black_pawns) * 1 +
                             self.bitboard_material(self.black_knights) * 3 +
                             self.bitboard_material(self.black_bishops) * 3 +
                             self.bitboard_material(self.black_rooks) * 5 +
                             self.bitboard_material(self.black_queens) * 9;

        white_material - black_material
    }

    // Helper function to count the bits of a bitboard
    fn bitboard_material(&self, bitboard: u64) -> i32 {
        bitboard.count_ones() as i32
    }

    // // Checks if the game is over
    // pub fn is_game_over(&self) -> bool {
    //     // Check for checkmate or stalemate
    //     if self.generate_legal_moves().is_empty() {
    //         if self.is_in_check(self.side_to_move) {
    //             println!("Checkmate!");
    //         } else {
    //             println!("Stalemate!");
    //         }
    //         return true;
    //     }

    //     // Check for fifty-move rule (this needs to be tracked after each move)
    //     if self.halfmove_clock >= 100 {
    //         println!("Draw by fifty-move rule");
    //         return true;
    //     }

    //     // Check for threefold repetition (this requires a move history)
    //     // ...

    //     // Check for insufficient material (requires a more complex function)
    //     // ...

    //     false
    // }

    // // Stub for checking if the current side is in check
    // // You'll need to implement actual check detection logic here
    // fn is_in_check(&self, side: bool) -> bool {
    //     // ...
    //     false // Placeholder
    // }

    // Generates all legal moves for the current position
    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        let initial_state = self.clone(); // Clone the initial state
        //TODO: Remove board wrapping for all pieces
        let mut moves: Vec<Move> = Vec::new();

        // Generate moves for each piece type
        moves.append(&mut self.generate_pawn_moves());
        moves.append(&mut self.generate_knight_moves());
        moves.append(&mut self.generate_bishop_moves());
        moves.append(&mut self.generate_rook_moves());
        moves.append(&mut self.generate_queen_moves());
        moves.append(&mut self.generate_king_moves());

        // Filter out illegal moves
        moves.retain(|&mv| {
            let undo_state = self.make_move(mv);
            let is_legal = !self.is_in_check(!self.side_to_move); // Check if the current side is in check
            self.unmake_move(mv, undo_state);
            is_legal
        });

        if self != &initial_state {
            // This will panic if the board states are not equal
            println!("moves generated and applied and undone, but board state changed");
            println!("{:?}", moves);
            println!("new state board representation:");
            self.print_board();
            println!("new state {:?}", self);
            println!("initial state board representation:");
            initial_state.print_board();
            println!("initial state {:?}", initial_state);
            assert_eq!(self, &initial_state, "Board state changed after generate_legal_moves");
        }

        moves
    }

    // Stub for checking if the current side is in check
    pub fn is_in_check(&self, side: bool) -> bool {
        // Find the king's position
        let king_pos = if side == WHITE {
            self.bitboard_to_square(self.white_king)
        } else {
            self.bitboard_to_square(self.black_king)
        };

        // Check for attacks from pawns
        if self.is_attacked_by_pawns(king_pos, side) {
            return true;
        }

        // Check for attacks from knights
        if self.is_attacked_by_knights(king_pos, side) {
            return true;
        }

        // Check for attacks from bishops and queens (diagonally)
        if self.is_attacked_by_sliding_pieces(king_pos, side, &[-9, -7, 7, 9]) {
            return true;
        }

        // Check for attacks from rooks and queens (straight lines)
        if self.is_attacked_by_sliding_pieces(king_pos, side, &[-8, -1, 1, 8]) {
            return true;
        }

        // Check for attacks from the opposing king
        if self.is_attacked_by_king(king_pos, side) {
            return true;
        }

        false
    }

    fn is_square_attacked(&self, square: u8, side: bool,) -> bool {
        // Check for attacks from pawns
        if self.is_attacked_by_pawns(square, side) {
            return true;
        }
    
        // Check for attacks from knights
        if self.is_attacked_by_knights(square, side) {
            return true;
        }
    
        // Check for attacks from bishops and queens (diagonally)
        if self.is_attacked_by_sliding_pieces(square, side, &[-9, -7, 7, 9]) {
            return true;
        }
    
        // Check for attacks from rooks and queens (straight lines)
        if self.is_attacked_by_sliding_pieces(square, side, &[-8, -1, 1, 8]) {
            return true;
        }
    
        // Check for attacks from the opposing king
        if self.is_attacked_by_king(square, side) {
            return true;
        }
    
        false
    }

    // Helper method to convert a bitboard to a square index
    fn bitboard_to_square(&self, bitboard: u64) -> u8 {
        bitboard.trailing_zeros() as u8
    }

    // Helper method to check for pawn attacks
    fn is_attacked_by_pawns(&self, square: u8, side: bool) -> bool {
        let pawn_attacks = if side == WHITE {
            [7, 9] // Black pawn attack offsets
        } else {
            [-9, -7] // White pawn attack offsets
        };

        for &offset in &pawn_attacks {
            let attack_square = (square as i8).wrapping_add(offset) as u8;
            if self.is_on_board(attack_square) {
                let pawn_bitboard = if side == WHITE {
                    self.black_pawns
                } else {
                    self.white_pawns
                };
                if pawn_bitboard & (1u64 << attack_square) != 0 {
                    // only attacking if the attack file is different by 1
                    let from_file = get_file(square);
                    let to_file = get_file(attack_square);
                    if (from_file as i8 - to_file as i8).abs() == 1 {
                        return true;
                    }
                }
            }
        }

        false
    }

    // Helper method to check for knight attacks
    fn is_attacked_by_knights(&self, square: u8, side: bool) -> bool {
        let knight_attacks = [
            -17, -15, -10, -6, 6, 10, 15, 17
        ];

        for &offset in &knight_attacks {
            let attack_square = (square as i8).wrapping_add(offset) as u8;
            if self.is_on_board(attack_square) {
                let knight_bitboard = if side == WHITE {
                    self.black_knights
                } else {
                    self.white_knights
                };
                if knight_bitboard & (1u64 << attack_square) != 0 {
                    // we can make sure rank or file doesnt chang eby more than 2 to avoid wrapping attacks
                    let from_rank = get_rank(square);
                    let to_rank = get_rank(attack_square);
                    let from_file = get_file(square);
                    let to_file = get_file(attack_square);
                    if (from_rank as i8 - to_rank as i8).abs() <= 2 && (from_file as i8 - to_file as i8).abs() <= 2 {
                        return true;
                    }
                }
            }
        }

        false
    }

    // Helper method to check for sliding piece attacks (bishops, rooks, queens)
    fn is_attacked_by_sliding_pieces(&self, square: u8, side: bool, directions: &[i8]) -> bool {
        // Check if a square is attacked by sliding pieces (bishops, rooks, queens)
        //
        // :param square: The square to check
        // :param side: The side to check for attacks
        // :param directions: The directions to check for attacks
        // :return: True if the square is attacked, False otherwise

        // TODO: Make sure no wrapping that shouldnt be allowed.

        let (enemy_bishops, enemy_rooks, enemy_queens) = if side == WHITE {
            (self.black_bishops, self.black_rooks, self.black_queens)
        } else {
            (self.white_bishops, self.white_rooks, self.white_queens)
        };

        for &dir in directions {
            let mut attack_square = square as i8;

            loop {
                attack_square += dir;
                if !self.is_on_board(attack_square as u8) {
                    break;
                }

                let attack_square_bitboard = 1u64 << attack_square;
                if (enemy_bishops | enemy_queens) & attack_square_bitboard != 0 && (dir == -9 || dir == -7 || dir == 7 || dir == 9) {
                    // when moving like a bishop, need to make sure that rank and file dont change by different amounts
                    let from_rank = get_rank(square);
                    let to_rank = get_rank(attack_square as u8);
                    let from_file = get_file(square);
                    let to_file = get_file(attack_square as u8);
                    if (from_rank as i8 - to_rank as i8).abs() == (from_file as i8 - to_file as i8).abs() {
                        return true;
                    }
                }
                if (enemy_rooks | enemy_queens) & attack_square_bitboard != 0 && (dir == -8 || dir == -1 || dir == 1 || dir == 8) {
                    // when moving like a rook, need to make sure that it didnt wrap around the baord
                    let from_rank = get_rank(square);
                    let to_rank = get_rank(attack_square as u8);
                    let from_file = get_file(square);
                    let to_file = get_file(attack_square as u8);
                    if (from_rank as i8 - to_rank as i8).abs() == 0 || (from_file as i8 - to_file as i8).abs() == 0 {
                        return true;
                    }
                }
                if self.is_occupied(attack_square as u8) {
                    break;
                }
            }
        }

        false
    }

    // Helper method to check for king attacks
    fn is_attacked_by_king(&self, square: u8, side: bool) -> bool {
        // check if distance between kings is > 9, if so, false
        if (self.bitboard_to_square(self.black_king) as i8 - self.bitboard_to_square(self.white_king) as i8).abs() > 9 {
            return false;
        }
        // check if kings are separated by more than one rank or more than one file


        let king_attacks = [
            -9, -8, -7, -1, 1, 7, 8, 9
        ];

        for &offset in &king_attacks {
            let attack_square = (square as i8).wrapping_add(offset) as u8;
            if self.is_on_board(attack_square) {
                let king_bitboard = if side == WHITE {
                    self.black_king
                } else {
                    self.white_king
                };
                if king_bitboard & (1u64 << attack_square) != 0 {
                    // now make sure the king only moved at mmost one rank or file
                    let from_rank = get_rank(square);
                    let to_rank = get_rank(attack_square);
                    let from_file = get_file(square);
                    let to_file = get_file(attack_square);
                    if (from_rank as i8 - to_rank as i8).abs() <= 1 && (from_file as i8 - to_file as i8).abs() <= 1 {
                        return true;
                    }
                }
            }
        }

        false
    }

    // Generate moves for each piece type
    fn generate_pawn_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let pawns = if self.side_to_move == WHITE { self.white_pawns } else { self.black_pawns };
        let start_rank = if self.side_to_move == WHITE { 1 } else { 6 };
        let direction = if self.side_to_move == WHITE { 8 } else { -8 };

        for from in 0..64 {
            let from_mask = 1u64 << from;
            if pawns & from_mask != 0 {
                // Single move forward
                let to = (from as i8 + direction) as u8;
                if !self.is_occupied(to) {
                    // Check for promotion
                    if get_rank(to) == if self.side_to_move == WHITE { 7 } else { 0 } {
                        moves.push(Move { from, to, promotion: Some(PieceType::Queen), piece_type: PieceType::Pawn });
                        moves.push(Move { from, to, promotion: Some(PieceType::Rook), piece_type: PieceType::Pawn });
                        moves.push(Move { from, to, promotion: Some(PieceType::Bishop), piece_type: PieceType::Pawn });
                        moves.push(Move { from, to, promotion: Some(PieceType::Knight), piece_type: PieceType::Pawn });
                    } else {
                        moves.push(Move { from, to, promotion: None, piece_type: PieceType::Pawn });
                    }
                }

                // Double move forward
                if get_rank(from as u8) == start_rank {
                    let to = (from as i8 + 2 * direction) as u8;
                    if !self.is_occupied(to) && !self.is_occupied((from as i8 + direction) as u8) {
                        moves.push(Move { from, to, promotion: None, piece_type: PieceType::Pawn });
                    }
                }

                // Captures
                let attack_offsets = if self.side_to_move == WHITE { [7, 9] } else { [-9, -7] };
                for &offset in &attack_offsets {
                    let to = (from as i8 + offset) as u8;
                    if self.is_on_board(to) && self.is_occupied_by_opponent(to, self.side_to_move) {
                        let from_file = from % 8;
                        let to_file = to % 8;
                        // Check if the move does not wrap around the board
                        if (from_file as i8 - to_file as i8).abs() <= 1 {
                            // Check for promotion
                            if get_rank(to) == if self.side_to_move == WHITE { 7 } else { 0 } {
                                moves.push(Move { from, to, promotion: Some(PieceType::Queen), piece_type: PieceType::Pawn });
                                moves.push(Move { from, to, promotion: Some(PieceType::Rook), piece_type: PieceType::Pawn });
                                moves.push(Move { from, to, promotion: Some(PieceType::Bishop), piece_type: PieceType::Pawn });
                                moves.push(Move { from, to, promotion: Some(PieceType::Knight), piece_type: PieceType::Pawn });
                            } else {
                                moves.push(Move { from, to, promotion: None, piece_type: PieceType::Pawn });
                            }
                        }
                    }
                }

                // En passant captures
                if let Some(en_passant_square) = self.en_passant {
                    for &offset in &attack_offsets {

                        let to = (from as i8 + offset) as u8;
                        if to == en_passant_square {
                            // Check if the move does not wrap around the board
                            let from_file = from % 8;
                            let to_file = to % 8;
                            if (from_file as i8 - to_file as i8).abs() <= 1 {
                                let captured_pawn_square = if self.side_to_move == WHITE {
                                    en_passant_square.wrapping_sub(8)
                                } else {
                                    en_passant_square.wrapping_add(8)
                                };

                                // Create a temporary copy of the board to simulate the move
                                // TODO: remove board copy! and use unmake move
                                let mut board_copy = self.clone();

                                // Simulate the en passant capture
                                board_copy.make_move(Move {
                                    from,
                                    to,
                                    promotion: None,
                                    piece_type: PieceType::Pawn,
                                });

                                // IMPORTANT: Here, you need to remove the pawn that was captured en passant
                                let captured_pawn_bitboard = 1u64 << captured_pawn_square;
                                if self.side_to_move == WHITE {
                                    board_copy.black_pawns &= !captured_pawn_bitboard;
                                } else {
                                    board_copy.white_pawns &= !captured_pawn_bitboard;
                                }

                                // Check if this leaves the king in check
                                if !board_copy.is_in_check(self.side_to_move) {
                                    moves.push(Move {
                                        from,
                                        to,
                                        promotion: None,
                                        piece_type: PieceType::Pawn,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        moves
    }

    fn generate_knight_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let knights = if self.side_to_move == WHITE { self.white_knights } else { self.black_knights };

        for from in 0..64 {
            let from_mask = 1u64 << from;
            if knights & from_mask != 0 {
                // The possible moves for a knight from any position
                let knight_moves = [
                    -17, -15, -10, -6, 6, 10, 15, 17
                ];
                for &move_offset in &knight_moves {
                    let to = (from as i8).wrapping_add(move_offset) as u8;
                
                    // Check if the move is within the bounds of the board
                    if self.is_on_board(to) && !self.is_occupied_by_side(to, self.side_to_move) {
                        let from_row = from / 8;
                        let from_col = from % 8;
                        let to_row = to / 8;
                        let to_col = to % 8;
                
                        // Check if the move is a valid knight move
                        if (from_row as i8 - to_row as i8).abs() == 2 && (from_col as i8 - to_col as i8).abs() == 1 ||
                           (from_row as i8 - to_row as i8).abs() == 1 && (from_col as i8 - to_col as i8).abs() == 2 {
                            moves.push(Move {
                                from,
                                to,
                                promotion: None,
                                piece_type: PieceType::Knight,
                            });
                        }
                    }
                }
            }
        }

        moves
    }
    fn generate_bishop_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let bishops = if self.side_to_move == WHITE { self.white_bishops } else { self.black_bishops };
    
        for from in 0..64 {
            let from_mask = 1u64 << from;
            if bishops & from_mask != 0 {
                // The possible directions for a bishop from any position
                let directions = [-9, -7, 7, 9];
    
                for &dir in &directions {
                    let mut to = from as i8;
    
                    loop {
                        to += dir; // Move in the direction
    
                        // Additional check to prevent wrapping around
                        if (dir == -9 && (to % 8 == 7 || to < 0)) || (dir == -7 && (to % 8 == 0 || to < 0)) ||
                           (dir == 9 && (to % 8 == 0 || to > 63)) || (dir == 7 && (to % 8 == 7 || to > 63)) {
                            break;
                        }
    
                        // Check if the move is within the bounds of the board
                        if !self.is_on_board(to as u8) {
                            break;
                        }
        
                        if self.is_occupied_by_side(to as u8, self.side_to_move) {
                            break; // Stop if we hit a friendly piece
                        }
        
                        moves.push(Move {
                            from,
                            to: to as u8,
                            promotion: None,
                            piece_type: PieceType::Bishop,
                        });
        
                        if self.is_occupied_by_opponent(to as u8, self.side_to_move) {
                            break; // Stop if we hit an opponent's piece (capture it)
                        }
                    }
                }
            }
        }

        moves
    }

    fn generate_rook_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let rooks = if self.side_to_move == WHITE { self.white_rooks } else { self.black_rooks };
    
        for from in 0..64 {
            let from_mask = 1u64 << from;
            if rooks & from_mask != 0 {
                // The possible directions for a rook from any position
                let directions = [-8, -1, 1, 8];
    
                for &dir in &directions {
                    let mut to = from as i8;
    
                    loop {
                        to += dir; // Move in the direction
    
                        // Additional check to prevent wrapping around
                        if (dir == -1 && to % 8 == 7) || (dir == 1 && to % 8 == 0) {
                            break;
                        }
    
                        if !self.is_on_board(to as u8) || self.is_occupied_by_side(to as u8, self.side_to_move) {
                            break; // Stop if we hit the edge of the board or a friendly piece
                        }
    
                        moves.push(Move {
                            from,
                            to: to as u8,
                            promotion: None,
                            piece_type: PieceType::Rook,
                        });
    
                        if self.is_occupied_by_opponent(to as u8, self.side_to_move) {
                            break; // Stop if we hit an opponent's piece (capture it)
                        }
                    }
                }
            }
        }
    
        moves
    }
    

    fn generate_queen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let queens = if self.side_to_move == WHITE { self.white_queens } else { self.black_queens };
    
        for from in 0..64 {
            let from_mask = 1u64 << from;
            if queens & from_mask != 0 {
                // The possible directions for a queen from any position
                let directions = [-9, -8, -7, -1, 1, 7, 8, 9];
    
                for &dir in &directions {
                    let mut to = from as i8;
    
                    loop {
                        to += dir; // Move in the direction
    
                        // Additional check to prevent wrapping around
                        if (dir == -1 && to % 8 == 7) || (dir == 1 && to % 8 == 0) ||
                           (dir == -9 && (to % 8 == 7 || to < 0)) || (dir == -7 && (to % 8 == 0 || to < 0)) ||
                           (dir == 9 && (to % 8 == 0 || to > 63)) || (dir == 7 && (to % 8 == 7 || to > 63)) {
                            break;
                        }
    
                        if !self.is_on_board(to as u8) || self.is_occupied_by_side(to as u8, self.side_to_move) {
                            break; // Stop if we hit the edge of the board or a friendly piece
                        }
    
                        moves.push(Move {
                            from,
                            to: to as u8,
                            promotion: None,
                            piece_type: PieceType::Queen,
                        });
    
                        if self.is_occupied_by_opponent(to as u8, self.side_to_move) {
                            break; // Stop if we hit an opponent's piece (capture it)
                        }
                    }
                }
            }
        }
    
        moves
    }
    

    fn generate_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let king = if self.side_to_move == WHITE { self.white_king } else { self.black_king };
    
        for from in 0..64 {
            let from_mask = 1u64 << from;
            if king & from_mask != 0 {
                let king_moves = [-9, -8, -7, -1, 1, 7, 8, 9];
                let from_file = from % 8;
    
                for &move_offset in &king_moves {
                    let to = (from as i8 + move_offset) as u8;
    
                    // Additional check to prevent wrapping around
                    let to_file = to % 8;
                    if (from_file as i8 - to_file as i8).abs() > 1 {
                        continue; // Skip this move as it would wrap around the board
                    }
    
                    // Check if the move is within the bounds of the board and not landing on a friendly piece
                    if self.is_on_board(to) && !self.is_occupied_by_side(to, self.side_to_move) {
                        moves.push(Move {
                            from,
                            to,
                            promotion: None,
                            piece_type: PieceType::King,
                        });
                    }
                }
                
                if (self.side_to_move == WHITE && self.castling_rights & 0b0001 != 0) ||
                (self.side_to_move == BLACK && self.castling_rights & 0b0100 != 0) {
                    if self.can_castle_kingside() {
                        let castle_move_to = if self.side_to_move == WHITE { 6 } else { 62 };
                        moves.push(Move::new(from, castle_move_to, PieceType::King, None));
                    }
                }
                if (self.side_to_move == WHITE && self.castling_rights & 0b0010 != 0) ||
                    (self.side_to_move == BLACK && self.castling_rights & 0b1000 != 0) {
                        if self.can_castle_queenside() {
                            let castle_move_to = if self.side_to_move == WHITE { 2 } else { 58 };
                            moves.push(Move::new(from, castle_move_to, PieceType::King, None));
                        }
                }
            }
        }

        moves
    }

    // Helper method to determine if kingside castling is legal
    fn can_castle_kingside(&self) -> bool {
        let castling_rights_mask = if self.side_to_move == WHITE { 1 } else { 0b0100 };
        if self.castling_rights & castling_rights_mask == 0 {
            return false; // Castling right not available
        }
    
        // Positions between king and rook must be empty
        let empty_squares_mask = if self.side_to_move == WHITE { 0b01100000 } else { 0b01100000 << (7 * 8) }; // Shift the mask to the 8th rank for black
        let all_pieces = self.white_pawns | self.white_knights | self.white_bishops | self.white_rooks | self.white_queens | self.white_king |
                         self.black_pawns | self.black_knights | self.black_bishops | self.black_rooks | self.black_queens | self.black_king;
                         
        if all_pieces & empty_squares_mask != 0 {
            return false; // There are pieces in the way
        }
    
        if self.is_in_check(self.side_to_move) {
            return false; // Can't castle out of check
        }
    
        // Make sure the king doesn't pass through check
        let king_pass_through_squares = if self.side_to_move == WHITE { [5, 6] } else { [61, 62] }; // e1 to g1 or e8 to g8
        for &square in &king_pass_through_squares {
            if self.is_square_attacked(square, self.side_to_move) {
                return false; // King can't move through check
            }
        }
    
        true
    }

    // Helper method to determine if queenside castling is legal
    fn can_castle_queenside(&self) -> bool {
        let castling_rights_mask = if self.side_to_move == WHITE { 0b0010 } else { 0b1000 };
        if self.castling_rights & castling_rights_mask == 0 {
            return false; // Castling right not available
        }
    
        // Check for no pieces between king and rook
        let empty_squares_mask = if self.side_to_move == WHITE { 0b00001110 } else { 0b00001110 << (7 * 8) }; // Shift the mask to the 8th rank for black
        let all_pieces = self.white_pawns | self.white_knights | self.white_bishops | self.white_rooks | self.white_queens | self.white_king |
                         self.black_pawns | self.black_knights | self.black_bishops | self.black_rooks | self.black_queens | self.black_king;
        if all_pieces & empty_squares_mask != 0 {
            return false; // There are pieces in the way
        }
        
        if self.is_in_check(self.side_to_move) {
            return false; // Can't castle out of check
        }
    
        // Make sure the king doesn't pass through check
        let king_pass_through_squares = if self.side_to_move == WHITE { [2, 3] } else { [58, 59] }; // d1 to c1 or d8 to c8 for white and black, respectively
        for &square in &king_pass_through_squares {
            if self.is_square_attacked(square, self.side_to_move) {
                return false; // King can't move through check
            }
        }
    
        true
    }

    // Check if a square is occupied
    fn is_occupied(&self, square: u8) -> bool {
        let mask = 1u64 << square;

        let is_occupied_bool = (self.white_pawns | self.white_knights | self.white_bishops |
         self.white_rooks | self.white_queens | self.white_king |
         self.black_pawns | self.black_knights | self.black_bishops |
         self.black_rooks | self.black_queens | self.black_king) & mask != 0;

         is_occupied_bool
    }

    // Check if a square is on the board
    fn is_on_board(&self, square: u8) -> bool {
        square < 64
    }

    // Check if a square is occupied by a piece of the given side
    fn is_occupied_by_side(&self, square: u8, side: bool) -> bool {
        let mask = 1u64 << square;
        let occupied = if side == WHITE {
            self.white_pawns | self.white_knights | self.white_bishops |
            self.white_rooks | self.white_queens | self.white_king
        } else {
            self.black_pawns | self.black_knights | self.black_bishops |
            self.black_rooks | self.black_queens | self.black_king
        };
        occupied & mask != 0
    }

    // Method to print the board
    pub fn print_board(&self) {
        println!("  a b c d e f g h");
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square = (rank * 8 + file) as u8;
                print!("{} ", self.get_piece_char(square));
            }
            println!(" {}", rank + 1);
        }
        println!("  a b c d e f g h\n");
    }

    // Method to get character representation for a piece on a given square
    fn get_piece_char(&self, square: u8) -> char {
        let mask = 1u64 << square;
        if self.white_pawns & mask != 0 {
            return 'P';
        } else if self.white_knights & mask != 0 {
            return 'N';
        } else if self.white_bishops & mask != 0 {
            return 'B';
        } else if self.white_rooks & mask != 0 {
            return 'R';
        } else if self.white_queens & mask != 0 {
            return 'Q';
        } else if self.white_king & mask != 0 {
            return 'K';
        } else if self.black_pawns & mask != 0 {
            return 'p';
        } else if self.black_knights & mask != 0 {
            return 'n';
        } else if self.black_bishops & mask != 0 {
            return 'b';
        } else if self.black_rooks & mask != 0 {
            return 'r';
        } else if self.black_queens & mask != 0 {
            return 'q';
        } else if self.black_king & mask != 0 {
            return 'k';
        }
        '.'
    }

    pub fn get_piece_type(&self, square: u8) -> PieceType {
        let mask = 1u64 << square;
        if self.white_pawns & mask != 0 {
            return PieceType::Pawn;
        } else if self.white_knights & mask != 0 {
            return PieceType::Knight;
        } else if self.white_bishops & mask != 0 {
            return PieceType::Bishop;
        } else if self.white_rooks & mask != 0 {
            return PieceType::Rook;
        } else if self.white_queens & mask != 0 {
            return PieceType::Queen;
        } else if self.white_king & mask != 0 {
            return PieceType::King;
        } else if self.black_pawns & mask != 0 {
            return PieceType::Pawn;
        } else if self.black_knights & mask != 0 {
            return PieceType::Knight;
        } else if self.black_bishops & mask != 0 {
            return PieceType::Bishop;
        } else if self.black_rooks & mask != 0 {
            return PieceType::Rook;
        } else if self.black_queens & mask != 0 {
            return PieceType::Queen;
        } else if self.black_king & mask != 0 {
            return PieceType::King;
        } else {
            panic!("No piece on square {}", square);
        }
    }


    // Entry point for the search with iterative deepening
    pub fn search_moves(&mut self, n_moves: usize, max_depth: usize) -> Vec<ScoredMove> {
        let mut best_moves: Vec<ScoredMove> = Vec::new();

        for depth in 1..=max_depth {
            let mut scored_moves = self.depth_first_search(depth);

            // Sort moves by score
            scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

            // Update the list of best moves if better moves are found at this depth
            if !scored_moves.is_empty() {
                best_moves = scored_moves.into_iter().take(n_moves).collect();
            }

            // Break early if the maximum depth is reached
            if depth == max_depth {
                break;
            }
        }

        best_moves
    }

    // Depth-first search implementation
    fn depth_first_search(&mut self, depth: usize) -> Vec<ScoredMove> {
        let legal_moves = self.generate_legal_moves();
        let mut scored_moves = Vec::new();

        for mv in legal_moves {
            let undo_state = self.make_move(mv);
            let score = -self.minimax(depth - 1, -i32::MAX, i32::MAX);
            self.unmake_move(mv, undo_state);

            scored_moves.push(ScoredMove::new(mv, score));
        }

        scored_moves
    }

    // Minimax algorithm with alpha-beta pruning
    fn minimax(&mut self, depth: usize, alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.evaluate();
        }

        let mut alpha = alpha;
        let legal_moves = self.generate_legal_moves();
        if legal_moves.is_empty() {
            return self.evaluate_checkmate_or_stalemate();
        }

        let mut best_score = -i32::MAX;
        for mv in legal_moves {
            let undo_state = self.make_move(mv);
            let score = -self.minimax(depth - 1, -beta, -alpha);
            self.unmake_move(mv, undo_state);

            best_score = best_score.max(score);
            alpha = alpha.max(score);

            if alpha >= beta {
                break; // Beta cutoff
            }
        }

        best_score
    }

    // Helper method to evaluate the board for checkmate or stalemate
    pub fn evaluate_checkmate_or_stalemate(&self) -> i32 {
        if self.is_in_check(self.side_to_move) {
            return -i32::MAX; // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    
    
    pub fn perft(&mut self, depth: usize) {
        let start_time = Instant::now();
        let mut top_level_moves_count: HashMap<(u8, u8, Option<PieceType>), usize> = HashMap::new();
    
        let legal_moves = self.generate_legal_moves();
        for mv in legal_moves {
            let undo_state = self.make_move(mv);
            let nodes_count = self.perft_helper(depth - 1);
            top_level_moves_count.insert((mv.from, mv.to, mv.promotion), nodes_count);
            self.unmake_move(mv, undo_state);
        }
    
        let duration = start_time.elapsed();
    
        // Print the move counts for top-level moves
        for ((from, to, promo), count) in &top_level_moves_count {
            // println!("Top-level move from {} to {}: generates {} nodes", from, to, count);
            // convert square to rank a-h and file 1-8
            let from_rank = get_rank(*from);
            let from_file = get_file(*from);
            let to_rank = get_rank(*to);
            let to_file = get_file(*to);
            // convert file to char
            let from_file_char = match from_file {
                0 => 'a',
                1 => 'b',
                2 => 'c',
                3 => 'd',
                4 => 'e',
                5 => 'f',
                6 => 'g',
                7 => 'h',
                _ => panic!("Invalid file"),
            };
            let to_file_char = match to_file {
                0 => 'a',
                1 => 'b',
                2 => 'c',
                3 => 'd',
                4 => 'e',
                5 => 'f',
                6 => 'g',
                7 => 'h',
                _ => panic!("Invalid file"),
            };
            let promo_char = match promo {
                Some(PieceType::Queen) => 'q',
                Some(PieceType::Rook) => 'r',
                Some(PieceType::Bishop) => 'b',
                Some(PieceType::Knight) => 'n',
                None => ' ',
                _ => panic!("Invalid promotion"),
            };
            println!("{}{}{}{}{}: {}", from_file_char, from_rank + 1, to_file_char, to_rank + 1, promo_char, count);
        }

        // sum all counts in top level move counts and print it
        let mut total_nodes = 0;
        for (_, count) in &top_level_moves_count {
            total_nodes += count;
        }
        println!("Total nodes: {}", total_nodes);
    
        // Print the total time taken
        println!("Time taken: {:?}", duration);
    }
    
    fn perft_helper(&mut self, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }
    
        let mut total_moves = 0;
        let legal_moves = self.generate_legal_moves();
    
        for mv in legal_moves {
            let undo_state = self.make_move(mv);
            total_moves += self.perft_helper(depth - 1);
            self.unmake_move(mv, undo_state);
        }
    
        total_moves
    }

    pub fn set_pos(&mut self, fen: &str) {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            panic!("Invalid FEN string");
        }

        self.reset_board(); // Clear the board or reset it to default state

        self.set_pieces(parts[0]); // Parts[0] contains piece placement
        self.side_to_move = if parts[1] == "w" { WHITE } else { BLACK };
        self.set_castling_rights(parts[2]);
        self.set_en_passant(parts[3]);
        self.halfmove_clock = parts[4].parse().unwrap_or(0);
        self.fullmove_number = parts[5].parse().unwrap_or(1);
    }

    fn reset_board(&mut self) {
        // Reset the board to the default starting position
        self.white_pawns = 0;
        self.white_knights = 0;
        self.white_bishops = 0;
        self.white_rooks = 0;
        self.white_queens = 0;
        self.white_king = 0;
        self.black_pawns = 0;
        self.black_knights = 0;
        self.black_bishops = 0;
        self.black_rooks = 0;
        self.black_queens = 0;
        self.black_king = 0;
        self.side_to_move = WHITE;
        self.castling_rights = 0b1111;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
    }

    fn set_pieces(&mut self, pieces: &str) {
        // Parse piece placement from the FEN and set the board

        // Iterate over each rank
        let mut rank = 7;
        let mut file = 0;
        for c in pieces.chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if c.is_digit(10) {
                file += c.to_digit(10).unwrap();
            } else {
                let square = rank * 8 + file;
                let bitboard = 1u64 << square;
                match c {
                    'P' => self.white_pawns |= bitboard,
                    'N' => self.white_knights |= bitboard,
                    'B' => self.white_bishops |= bitboard,
                    'R' => self.white_rooks |= bitboard,
                    'Q' => self.white_queens |= bitboard,
                    'K' => self.white_king |= bitboard,
                    'p' => self.black_pawns |= bitboard,
                    'n' => self.black_knights |= bitboard,
                    'b' => self.black_bishops |= bitboard,
                    'r' => self.black_rooks |= bitboard,
                    'q' => self.black_queens |= bitboard,
                    'k' => self.black_king |= bitboard,
                    _ => panic!("Invalid FEN string"),
                }
                file += 1;
            }
        }

    }

    fn set_castling_rights(&mut self, rights: &str) {
        // Set the castling rights from the FEN
        self.castling_rights = 0;
        for c in rights.chars() {
            match c {
                'K' => self.castling_rights |= 0b0001,
                'Q' => self.castling_rights |= 0b0010,
                'k' => self.castling_rights |= 0b0100,
                'q' => self.castling_rights |= 0b1000,
                '-' => break,
                _ => panic!("Invalid FEN string"),
            }
        }
    }

    fn set_en_passant(&mut self, square: &str) {
        // Set the en passant target square from the FEN
        if square == "-" {
            self.en_passant = None;
        } else {
            let file = square.chars().nth(0).unwrap() as i8 - 'a' as i8;
            let rank = square.chars().nth(1).unwrap() as i8 - '1' as i8;
            self.en_passant = Some((rank * 8 + file) as u8);
        }
    }

    
    
    // Your other functions and implementations...
    
}


// #[cfg(test)]
// mod tests {
//     use super::*; // Import everything from the outer module.

//     #[test]
//     fn test_new_board() {
//         let board = Board::new();
//         // You should have specific conditions to assert the initial board state.
//         assert_eq!(board.white_pawns, 0xFF00);
//         assert_eq!(board.black_pawns, 0xFF000000000000);
//         // ... more assertions for initial setup ...
//     }

//     #[test]
//     fn test_make_move() {
//         let mut board = Board::new();
//         let mv = Move {
//             from: 52, // Example starting square for a pawn
//             to: 36,   // Example ending square for a pawn
//             promotion: None,
//             piece_type: PieceType::Pawn,
//             // ... other fields ...
//         };
//         board.make_move(mv);
//         // Now assert the state of the board after the move.
//         // For example, if we moved a white pawn:
//         assert_eq!(board.white_pawns & (1u64 << 36), 1u64 << 36); // Pawn should be on the 'to' square.
//         assert_eq!(board.white_pawns & (1u64 << 52), 0); // 'from' square should be empty.
//     }

//     // ... more tests ...
// }
