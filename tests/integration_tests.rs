// use rusty_engine::board::Board;
// use rusty_engine::board::Move;
// use rusty_engine::board::PieceType;


// #[test]
// fn test_board_functionality() {
//     let mut board = Board::new();
//     // pub struct Move {
//     //     from: u8,     // Square number 0-63 from which the piece is moved
//     //     to: u8,       // Square number 0-63 to which the piece is moved
//     //     promotion: Option<PieceType>, // None if not a promotion, otherwise the piece type to which the pawn is promoted
//     //     piece_type: PieceType
//     //     // Additional fields as necessary for en passant, castling, etc.
//     // // }
//     let m = Move {
//         from: 10,
//         to: 17,
//         promotion: None,
//         piece_type: PieceType::Pawn,
//     };

//     board.make_move(m);
//     // perform some actions on the board
//     // assert the results are as expected
// }