use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Hotel {
    Tower,
    Luxor,
    American,
    Worldwide,
    Festival,
    Imperial,
    Continental,
}

impl Hotel {
    pub const fn count() -> usize {
        7
    }

    fn get_hotel_row_advantage(&self) -> usize {
        match self {
            Hotel::Tower | Hotel::Luxor => 0,
            Hotel::American | Hotel::Worldwide | Hotel::Festival => 1,
            Hotel::Imperial | Hotel::Continental => 2,
        }
    }

    fn get_row_from_chain_length(&self, chain_length: usize) -> u32 {
        let base_row: u32 = match chain_length {
            0..=2 => 0,
            3 => 1,
            4 => 2,
            5 => 3,
            6..=10 => 4,
            11..=20 => 5,
            21..=30 => 6,
            31..=40 => 7,
            _ => 8,
        };
        base_row + self.get_hotel_row_advantage() as u32
    }

    pub fn get_stock_value(&self, chain_length: usize) -> u32 {
        (self.get_row_from_chain_length(chain_length) + 2) * 100
    }

    pub fn get_majority_holder_bonus(&self, chain_length: usize) -> u32 {
        self.get_stock_value(chain_length) * 10
    }

    pub fn get_minority_holder_bonus(&self, chain_length: usize) -> u32 {
        self.get_stock_value(chain_length) * 5
    }

    pub fn iter() -> HotelIter {
        HotelIter::new()
    }
}

impl From<usize> for Hotel {
    fn from(index: usize) -> Self {
        match index {
            0 => Hotel::Tower,
            1 => Hotel::Luxor,
            2 => Hotel::American,
            3 => Hotel::Worldwide,
            4 => Hotel::Festival,
            5 => Hotel::Imperial,
            6 => Hotel::Continental,
            _ => panic!("Invalid index for Hotel"), // or handle more gracefully
        }
    }
}

impl From<Hotel> for usize {
    fn from(hotel: Hotel) -> Self {
        hotel as usize
    }
}

impl fmt::Display for Hotel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Hotel::Festival => "Festival",
            Hotel::Continental => "Continental",
            Hotel::Worldwide => "Worldwide",
            Hotel::Tower => "Tower",
            Hotel::American => "American",
            Hotel::Imperial => "Imperial",
            Hotel::Luxor => "Luxor",
        };
        write!(f, "{}", name)
    }
}

pub struct HotelIter {
    index: usize,
}

impl HotelIter {
    fn new() -> Self {
        HotelIter { index: 0 }
    }
}

impl Iterator for HotelIter {
    type Item = Hotel;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(Hotel::Tower),
            1 => Some(Hotel::Luxor),
            2 => Some(Hotel::American),
            3 => Some(Hotel::Worldwide),
            4 => Some(Hotel::Festival),
            5 => Some(Hotel::Imperial),
            6 => Some(Hotel::Continental),
            _ => None,
        };
        self.index += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(Hotel::count(), 7);
    }

    #[test]
    fn test_get_hotel_row_advantage() {
        assert_eq!(Hotel::Tower.get_hotel_row_advantage(), 0);
        assert_eq!(Hotel::Luxor.get_hotel_row_advantage(), 0);
        assert_eq!(Hotel::American.get_hotel_row_advantage(), 1);
        assert_eq!(Hotel::Worldwide.get_hotel_row_advantage(), 1);
        assert_eq!(Hotel::Festival.get_hotel_row_advantage(), 1);
        assert_eq!(Hotel::Imperial.get_hotel_row_advantage(), 2);
        assert_eq!(Hotel::Continental.get_hotel_row_advantage(), 2);
    }

    #[test]
    fn test_get_row_from_chain_length() {
        assert_eq!(Hotel::Tower.get_row_from_chain_length(2), 0);
        assert_eq!(Hotel::American.get_row_from_chain_length(5), 4);
        assert_eq!(Hotel::Imperial.get_row_from_chain_length(11), 7);
    }
}
