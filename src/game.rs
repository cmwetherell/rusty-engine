// game.rs

use crate::board::{ Board, UndoState, WHITE, BLACK };
use crate::r#move::{Move};
use crate::piece::PieceType;


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
}