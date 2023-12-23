// Helper function to get the rank (0 to 7) from a square (0 to 63)
pub fn get_rank(square: u8) -> u8 {
    square / 8
}

// Helper function to get the file (0 to 7) from a square (0 to 63)
pub fn get_file(square: u8) -> u8 {
    square % 8
}