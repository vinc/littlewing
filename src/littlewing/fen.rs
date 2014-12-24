use std;
use littlewing::piece::Piece;

pub struct FENBuilder {
    count: uint,
    empty: bool,
    fen: String
}

impl FENBuilder {
    pub fn new() -> FENBuilder {
        FENBuilder {
            count: 0, // Counter of empty files
            empty: true, // Current file is empty
            fen: String::new()
        }
    }

    pub fn reset_count(&mut self) {
        if self.count > 0 {
            // Push the number of empty files for the current rank
            // since the last reset.
            let c = std::char::from_digit(self.count, 10).unwrap();
            self.fen.push(c);
            self.count = 0;
        }
    }

    pub fn push(&mut self, piece: Piece) {
        self.reset_count();
        self.fen.push(piece.to_char());
        self.empty = false;
    }

    pub fn next_rank(&mut self) {
        self.reset_count();
        self.fen.push('/');
    }

    pub fn next_file(&mut self) {
        if self.empty {
            self.count += 1;
        } else {
            self.empty = true;
        }
    }

    pub fn to_string(&self) -> String {
        self.fen.clone()
    }
}
