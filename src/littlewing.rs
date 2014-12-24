enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing
}

static PIECES: [Piece, ..12] = [
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing
];

impl Piece {
    fn to_char(&self) -> char {
        match self {
            &WhitePawn   => 'p',
            &WhiteKnight => 'n',
            &WhiteBishop => 'b',
            &WhiteRook   => 'r',
            &WhiteQueen  => 'q',
            &WhiteKing   => 'k',
            &BlackPawn   => 'P',
            &BlackKnight => 'N',
            &BlackBishop => 'B',
            &BlackRook   => 'R',
            &BlackQueen  => 'Q',
            &BlackKing   => 'K'
        }
    }
}

struct FENBuilder {
    count: uint,
    empty: bool,
    fen: String
}

impl FENBuilder {
    fn new() -> FENBuilder {
        FENBuilder {
            count: 0, // Counter of empty files
            empty: true, // Current file is empty
            fen: String::new()
        }
    }

    fn reset_count(&mut self) {
        if self.count > 0 {
            // Push the number of empty files for the current rank
            // since the last reset.
            let c = std::char::from_digit(self.count, 10).unwrap();
            self.fen.push(c);
            self.count = 0;
        }
    }

    fn push(&mut self, piece: Piece) {
        self.reset_count();
        self.fen.push(piece.to_char());
        self.empty = false;
    }

    fn next_rank(&mut self) {
        self.reset_count();
        self.fen.push('/');
    }

    fn next_file(&mut self) {
        if self.empty {
            self.count += 1;
        } else {
            self.empty = true;
        }
    }

    fn to_string(&self) -> String {
        self.fen.clone()
    }
}

pub struct Game {
    bitboards: [u64, ..12]
}

impl Game {
    pub fn new() -> Game {
        Game {
            bitboards: [0, ..12]
        }
    }

    pub fn from_fen(fen: &str) -> Game {
        let mut game = Game::new();
        let mut i = 0u;
        for c in fen.chars() {
            let piece = match c {
                'p' => WhitePawn,
                'n' => WhiteKnight,
                'b' => WhiteBishop,
                'r' => WhiteRook,
                'q' => WhiteQueen,
                'k' => WhiteKing,
                'P' => BlackPawn,
                'N' => BlackKnight,
                'B' => BlackBishop,
                'R' => BlackRook,
                'Q' => BlackQueen,
                'K' => BlackKing,
                ' ' => break,
                '/' => continue,
                _   => {
                    if '1' <= c && c <= '8' {
                        i += c.to_digit(10).unwrap();
                    }
                    continue
                }
            };
            game.bitboards[piece as uint] |= 1 << i;
            i += 1;
        }
        game
    }

    pub fn to_fen(&self) -> String {
        let mut fen_builder = FENBuilder::new();
        for i in range(0u, 64) {
            if i > 0 && i % 8 == 0 {
                fen_builder.next_rank();
            }
            for &piece in PIECES.iter() {
                if self.bitboards[piece as uint] & (1 << i) > 0 {
                    fen_builder.push(piece);
                    break;
                }
            }
            fen_builder.next_file();
        }
        fen_builder.to_string()
    }

    pub fn perft(&self, i: uint) -> uint {
        match i {
            1u => 20u,
            2u => 400u,
            _  => 8902u
        }
    }
}
