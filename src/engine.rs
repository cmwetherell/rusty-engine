pub trait Engine {
    fn best_move(&self, game: &Game) -> board::Move;
    // ... other engine-related methods ...
}
