#![allow(dead_code)]

use crate::std::prelude::v1::*;

use crate::color::*;

pub type Square = u8;

pub const A1: Square = 0;
pub const B1: Square = 1;
pub const C1: Square = 2;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const F1: Square = 5;
pub const G1: Square = 6;
pub const H1: Square = 7;
pub const A2: Square = 8;
pub const B2: Square = 9;
pub const C2: Square = 10;
pub const D2: Square = 11;
pub const E2: Square = 12;
pub const F2: Square = 13;
pub const G2: Square = 14;
pub const H2: Square = 15;
pub const A3: Square = 16;
pub const B3: Square = 17;
pub const C3: Square = 18;
pub const D3: Square = 19;
pub const E3: Square = 20;
pub const F3: Square = 21;
pub const G3: Square = 22;
pub const H3: Square = 23;
pub const A4: Square = 24;
pub const B4: Square = 25;
pub const C4: Square = 26;
pub const D4: Square = 27;
pub const E4: Square = 28;
pub const F4: Square = 29;
pub const G4: Square = 30;
pub const H4: Square = 31;
pub const A5: Square = 32;
pub const B5: Square = 33;
pub const C5: Square = 34;
pub const D5: Square = 35;
pub const E5: Square = 36;
pub const F5: Square = 37;
pub const G5: Square = 38;
pub const H5: Square = 39;
pub const A6: Square = 40;
pub const B6: Square = 41;
pub const C6: Square = 42;
pub const D6: Square = 43;
pub const E6: Square = 44;
pub const F6: Square = 45;
pub const G6: Square = 46;
pub const H6: Square = 47;
pub const A7: Square = 48;
pub const B7: Square = 49;
pub const C7: Square = 50;
pub const D7: Square = 51;
pub const E7: Square = 52;
pub const F7: Square = 53;
pub const G7: Square = 54;
pub const H7: Square = 55;
pub const A8: Square = 56;
pub const B8: Square = 57;
pub const C8: Square = 58;
pub const D8: Square = 59;
pub const E8: Square = 60;
pub const F8: Square = 61;
pub const G8: Square = 62;
pub const H8: Square = 63;
pub const OUT: Square = 64;

pub trait SquareExt {
    fn from_coord(s: &str) -> Self;
    fn to_coord(&self) -> String;
    fn file_to_char(&self) -> char;
    fn rank_to_char(&self) -> char;
    fn file(&self) -> u8;
    fn rank(&self) -> u8;
    fn flip(&self, c: Color) -> Self;
}

impl SquareExt for Square {
    fn from_coord(s: &str) -> Self { // TODO: -> Option<Self>
        let bytes = s.as_bytes();

        ((bytes[0] - b'a') + 8 * (bytes[1] - b'1')) as Square
    }

    fn to_coord(&self) -> String {
        // FIXME: OUT.to_coord() => a9
        format!("{}{}", self.file_to_char(), self.rank_to_char())
    }

    fn rank_to_char(&self) -> char {
        (b'1' + self.rank()) as char
    }

    fn file_to_char(&self) -> char {
        (b'a' + self.file()) as char
    }

    fn file(&self) -> u8 {
        *self as u8 % 8
    }

    fn rank(&self) -> u8 {
        *self as u8 / 8
    }

    fn flip(&self, c: Color) -> Self {
        *self ^ (A8 * c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_coord() {
        let sq: Square = SquareExt::from_coord("e2");
        assert_eq!(sq, E2);
    }

    #[test]
    fn test_to_coord() {
        assert_eq!(A1.to_coord(), "a1");
        assert_eq!(E2.to_coord(), "e2");
        assert_eq!(C6.to_coord(), "c6");
    }

    #[test]
    fn test_flip() {
        assert_eq!(E2.flip(WHITE), E2);
        assert_eq!(E2.flip(BLACK), E7);
    }
}

