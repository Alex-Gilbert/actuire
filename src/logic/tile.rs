use core::fmt;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Tile {
    row: usize,
    col: usize,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // row is a letter, col is a number
        // use the ASCII value of 'A' to get the letter
        write!(f, "{}{}", (b'A' + self.row as u8) as char, self.col + 1)
    }
}
