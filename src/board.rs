pub fn draw(squares: Vec<String>) -> String {
    let line = "  +---+---+---+---+---+---+---+---+\n";
    let file = "";
    draw_with(squares, line, file)
}

pub fn draw_with_coordinates(squares: Vec<String>) -> String {
    let line = "  +---+---+---+---+---+---+---+---+\n";
    let file = "    a   b   c   d   e   f   g   h\n";
    draw_with(squares, line, file)
}

pub fn draw_compact_with_coordinates(squares: Vec<String>) -> String {
    let line = "+--------+\n";
    let file = " abcdefgh\n";
    draw_with(squares, line, file)
}

fn draw_with(squares: Vec<String>, line: &str, file: &str) -> String {
    debug_assert!(squares.len() == 64);
    let with_spaces = line.len() > 12;
    let with_coords = !file.is_empty();
    let mut out = String::new();
    out.push_str(line);
    for i in (0..8).rev() {
        if with_spaces {
            out.push_str("  ");
        } else {
            out.push_str("|");
        }
        for j in 0..8 {
            let s = &squares[8 * i + j];
            if with_spaces {
                out.push_str(&format!("| {} ", s));
            } else {
                out.push_str(&format!("{}", s));
            }
        }
        if with_coords {
            out.push_str(&format!("| {}\n", i + 1));
        } else {
            out.push_str("|\n");
        }
        if with_spaces {
            out.push_str(line);
        }
    }
    if !with_spaces {
        out.push_str(line);
    }
    out.push_str(file);
    out
}

#[allow(dead_code)]
static PIECES_WITHOUT_COORDS: &str = "  \
  +---+---+---+---+---+---+---+---+
  | r | n | b | q | k | b | n | r |
  +---+---+---+---+---+---+---+---+
  | p | p | p | p | p | p | p | p |
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   |
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   |
  +---+---+---+---+---+---+---+---+
  |   |   |   |   | P |   |   |   |
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   |
  +---+---+---+---+---+---+---+---+
  | P | P | P | P |   | P | P | P |
  +---+---+---+---+---+---+---+---+
  | R | N | B | Q | K | B | N | R |
  +---+---+---+---+---+---+---+---+
";

#[allow(dead_code)]
static PIECES_WITH_COORDS: &str = "  \
  +---+---+---+---+---+---+---+---+
  | r | n | b | q | k | b | n | r | 8
  +---+---+---+---+---+---+---+---+
  | p | p | p | p | p | p | p | p | 7
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   | 6
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   | 5
  +---+---+---+---+---+---+---+---+
  |   |   |   |   | P |   |   |   | 4
  +---+---+---+---+---+---+---+---+
  |   |   |   |   |   |   |   |   | 3
  +---+---+---+---+---+---+---+---+
  | P | P | P | P |   | P | P | P | 2
  +---+---+---+---+---+---+---+---+
  | R | N | B | Q | K | B | N | R | 1
  +---+---+---+---+---+---+---+---+
    a   b   c   d   e   f   g   h
";

#[allow(dead_code)]
static BITBOARD_WITH_COORDS: &str = " \
 Bitboard  (0xFFFF00001000EFFF)
+--------+
|11111111| 8
|11111111| 7
|00000000| 6
|00000000| 5
|00001000| 4
|00000000| 3
|11110111| 2
|11111111| 1
+--------+
 abcdefgh
";

#[cfg(test)]
mod tests {
    use bitboard::BitboardExt;
    use color::*;
    use common::*;
    use fen::FEN;
    use game::Game;
    use piece_move::PieceMove;
    use piece_move_generator::PieceMoveGenerator;
    use square::*;
    use super::*;

    #[test]
    fn test_draw() {
        colored::control::set_override(false);
        let mut game = Game::from_fen(DEFAULT_FEN);
        game.make_move(PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));
        assert_eq!(format!("{}", game), PIECES_WITHOUT_COORDS);
    }

    #[test]
    fn test_draw_with_coordinates() {
        colored::control::set_override(false);
        let mut game = Game::from_fen(DEFAULT_FEN);
        game.show_coordinates = true;
        game.make_move(PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));
        assert_eq!(format!("{}", game), PIECES_WITH_COORDS);
    }

    #[test]
    fn test_draw_compact_with_coordinates() {
        colored::control::set_override(false);
        let mut game = Game::from_fen(DEFAULT_FEN);
        game.make_move(PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));
        let bb = game.bitboards[WHITE as usize] | game.bitboards[BLACK as usize];
        assert_eq!(format!("{}", bb.to_debug_string()), BITBOARD_WITH_COORDS);
    }
}
