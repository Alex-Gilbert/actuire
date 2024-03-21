use core::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tile {
    pub row: usize,
    pub col: usize,
}

impl From<(usize, usize)> for Tile {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // row is a letter, col is a number
        // use the ASCII value of 'A' to get the letter
        write!(f, "{}-{}", self.col + 1, (b'A' + self.row as u8) as char)
    }
}
