use std::io;
use rusty_engine::board::{Board, Move, PieceType};

fn main() {
    let mut board = Board::new();
    // board.set_pos("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"); // test from: https://www.chessprogramming.org/Perft_Results
    // board.set_pos("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1"); // test for en passant
    // board.set_pos("rnbqkbnr/p5pp/8/1ppppp2/3PP3/2N1BQ2/PPP2PPP/R3KBNR w KQkq - 0 6");
    // board.set_pos("rnbqkbnr/p5pp/8/1ppppp2/3PP3/2N1BQ2/PPP2PPP/RB2K2R w KQkq - 0 6");
    // board.set_pos("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    // board.set_pos("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    loop {
        board.print_board();

        let valid_moves = board.generate_legal_moves();
        // create new list of reverese parse moves
        let mut reverse_parse_moves = Vec::new();
        for mv in valid_moves.iter() {
            reverse_parse_moves.push(mv.get_uci());
        }

        //sort the list
        reverse_parse_moves.sort();
        // print reverse parse moves
        println!("Valid moves: {:?}", reverse_parse_moves);
        
        // print total length of valid moves
        println!("Total valid moves: {}", valid_moves.len());
        println!("Enter your move (e.g., e2e4), 'perft [depth]', or 'quit' to exit:");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed_input = input.trim();

        if trimmed_input == "quit" {
            break;
        } else if trimmed_input.starts_with("perft") {
            if let Some(depth_str) = trimmed_input.split_whitespace().nth(1) {
                if let Ok(depth) = depth_str.parse::<usize>() {
                    board.perft(depth);
                } else {
                    println!("Invalid depth. Please provide a numeric depth value.");
                }
            } else {
                println!("Please provide a depth for perft testing.");
            }
        } else if let Some(mv) = parse_move(trimmed_input, board.clone()) {
            // Check if move is in valid moves, else print error
            if valid_moves.contains(&mv) {
                board.make_move(mv);
            } else {
                println!("Invalid move");
            }
        } else {
            println!("Invalid move format. Please use algebraic notation (e.g., e2e4).");
        }
    }
}

// Add your parse_move function and other relevant code here


fn parse_move(input: &str, board: Board) -> Option<Move> {
    if input.len() == 4 {
        let from_file = input.chars().nth(0)?;
        let from_rank = input.chars().nth(1)?.to_digit(10)? as u8; // Cast to u8 here to match types for addition
        let to_file = input.chars().nth(2)?;
        let to_rank = input.chars().nth(3)?.to_digit(10)? as u8; // Cast to u8 here as well

        // Ensure both operands are u8 before addition
        let from_square = (from_rank - 1) * 8 + (from_file as u8 - b'a');
        let to_square = (to_rank - 1) * 8 + (to_file as u8 - b'a');

        // print from_square and to_square to see if they are correct
        println!("from_square: {}, to_square: {}", from_square, to_square);

        let piece_type: PieceType = board.get_piece_type(from_square);
        
        // Use the constructor here instead of setting fields directly
        Some(Move::new(from_square, to_square, piece_type, None))

    } else {
        None
    }
}

