use littlewing::common::*;

pub trait SquareString {
    fn from_coord(s: String) -> Self;
    fn to_coord(&self) -> String;
    fn rank(&self) -> u8;
    fn file(&self) -> u8;
}

impl SquareString for Square {
    fn from_coord(s: String) -> Self {
        let bytes = s.as_bytes();

        ((bytes[0] - b'a') + 8 * (bytes[1] - b'1')) as Square
    }
    fn to_coord(&self) -> String {
        let f = b'a' + self.file();
        let r = b'1' + self.rank();

        String::from_utf8(vec![f, r]).unwrap()
    }
    fn file(&self) -> u8 {
        *self as u8 % 8
    }
    fn rank(&self) -> u8 {
        *self as u8 / 8
    }
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use super::SquareString;

    #[test]
    fn test_from_coord() {
        let sq: Square = SquareString::from_coord("e2".to_string());
        assert_eq!(sq, E2);
    }

    #[test]
    fn test_to_coord() {
        assert_eq!(A1.to_coord(), "a1");
        assert_eq!(E2.to_coord(), "e2");
        assert_eq!(C6.to_coord(), "c6");
    }
}

