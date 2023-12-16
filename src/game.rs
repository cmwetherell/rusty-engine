mod board;
mod engine;

pub struct Game {
    board: board::Board,
    move_history: Vec<board::Move>,
    // ... other game state fields ...
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: board::Board::new(),
            move_history: Vec::new(),
            // ... initialize other fields ...
        }
    }

    // ... game-related methods ...
}

pub trait Playable {
    fn make_move(&mut self, mv: &board::Move);
    fn undo_move(&mut self);
    // ... other gameplay methods ...
}

impl Playable for Game {
    // Implement the trait methods for Game
}
