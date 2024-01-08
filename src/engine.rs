// engine.rs
//TODO: Make sure alpha beta pruning is set up corerctly
use std::time::Instant;
use std::collections::HashMap;

use crate::board::Board;
use crate::r#move::ScoredMove;
use crate::piece::PieceType;
use crate::utils::{get_rank, get_file};

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

#[derive(Clone, Debug)]
pub struct Engine {
    board: Board,
    // Engine-specific fields, if any
}

impl Engine {
    pub fn new() -> Self {
        // Initialize the engine
        Self {
            board: Board::new(),
        }
    }

    pub fn with_board(board: Option<Board>) -> Self {
        match board {
            Some(existing_board) => Self { board: existing_board },
            None => Self::new(),
        }
    }

    // A very simple evaluation function
    pub fn evaluate(&mut self) -> i32 {
        // Check for terminal conditions first
        if self.board.check_for_checkmate() {
            return if self.board.side_to_move == true {
                i32::MIN // Checkmate against White
            } else {
                i32::MAX // Checkmate against Black
            };
        } else if self.board.check_for_draw() {
            return 0; // Draw
        }
        let white_material = self.bitboard_material(self.board.white_pawns) * 1 +
                                self.bitboard_material(self.board.white_knights) * 3 +
                                self.bitboard_material(self.board.white_bishops) * 3 +
                                self.bitboard_material(self.board.white_rooks) * 5 +
                                self.bitboard_material(self.board.white_queens) * 9;

        let black_material = self.bitboard_material(self.board.black_pawns) * 1 +
                                self.bitboard_material(self.board.black_knights) * 3 +
                                self.bitboard_material(self.board.black_bishops) * 3 +
                                self.bitboard_material(self.board.black_rooks) * 5 +
                                self.bitboard_material(self.board.black_queens) * 9;

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
            let mut scored_moves = self.depth_first_search_parallel(depth);

            // Sort moves by score
            scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

            // Update the list of best moves if better moves are found at this depth
            if !scored_moves.is_empty() {
                best_moves = scored_moves.into_iter().take(n_moves).collect();
            }

            println!("Depth: {}, Best moves: {:?}", depth, best_moves);

            // Break early if the maximum depth is reached
            if depth == max_depth {
                break;
            }
        }

        best_moves
    }

    // // Depth-first search implementation
    // fn depth_first_search(&mut self, depth: usize) -> Vec<ScoredMove> {
    //     let legal_moves = self.board.generate_legal_moves();
    //     let mut scored_moves = Vec::new();

    //     for mv in legal_moves {
    //         let undo_state = self.board.make_move(mv);
    //         let score = -self.minimax(depth - 1, -i32::MAX, i32::MAX);
    //         self.board.unmake_move(mv, undo_state);

    //         scored_moves.push(ScoredMove::new(mv, score));
    //     }

    //     scored_moves
    // }

    // Parallel depth-first search implementation
    fn depth_first_search_parallel(&mut self, depth: usize) -> Vec<ScoredMove> {
        let legal_moves = self.board.generate_legal_moves();

        let pool = ThreadPoolBuilder::new().build().unwrap();

        let scored_moves: Vec<ScoredMove> = pool.install(|| {
            legal_moves.par_iter().map(|&mv| {
                let mut cloned_engine = self.clone();
                let undo_state = cloned_engine.board.make_move(mv);
                let score = cloned_engine.minimax(depth - 1, -i32::MAX, i32::MAX);
                cloned_engine.board.unmake_move(mv, undo_state);
                ScoredMove::new(mv, score)
            }).collect()
        });

        scored_moves
    }

    // Minimax algorithm with alpha-beta pruning
    fn minimax(&mut self, depth: usize, alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.evaluate();
        }

        let mut alpha = alpha;
        let legal_moves = self.board.generate_legal_moves();

        let mut best_score = -i32::MAX;
        for mv in legal_moves {
            let undo_state = self.board.make_move(mv);
            
            // Get score from minimax and safely negate it
            let mut score = self.minimax(depth - 1, -alpha, -beta);
            if score == i32::MIN {
                score = i32::MAX; // or handle this some other way
            } else {
                score = -score;
            }

            self.board.unmake_move(mv, undo_state);

            best_score = best_score.max(score);
            alpha = alpha.max(score);

            if alpha >= beta {
                break; // Beta cutoff
            }
        }

        best_score
    }

    // // Helper method to evaluate the board for checkmate or stalemate
    // pub fn evaluate_checkmate_or_stalemate(&self, current_depth: i32) -> i32 {
    //     //TODO: is this right?
    //     if self.board.is_in_check(self.board.side_to_move) {
    //         return i32::MAX - 1000 + current_depth // Checkmate
    //     } else {
    //         return 0; // Stalemate
    //     }
    // }
    
    
    pub fn perft(&mut self, depth: usize) {
        let start_time = Instant::now();
        let mut top_level_moves_count: HashMap<(u8, u8, Option<PieceType>), usize> = HashMap::new();
    
        let legal_moves = self.board.generate_legal_moves();
        for mv in legal_moves {
            let undo_state = self.board.make_move(mv);
            let nodes_count = self.perft_helper(depth - 1);
            top_level_moves_count.insert((mv.from, mv.to, mv.promotion), nodes_count);
            self.board.unmake_move(mv, undo_state);
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
        let legal_moves = self.board.generate_legal_moves();
    
        for mv in legal_moves {
            let undo_state = self.board.make_move(mv);
            total_moves += self.perft_helper(depth - 1);
            self.board.unmake_move(mv, undo_state);
        }
    
        total_moves
    }

}