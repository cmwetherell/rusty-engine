// engine.rs
use std::time::Instant;
use std::collections::HashMap;

use crate::board::Board;
use crate::r#move::{Move, ScoredMove};
use crate::piece::PieceType;
use crate::utils::{get_rank, get_file};

pub struct Engine {
    // Engine-specific fields, if any
}

impl Engine {
    pub fn new() -> Self {
        // Initialize the engine
        Engine {
            // ...
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

}