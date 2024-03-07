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

    fn get_row_from_chain_length(&self, chain_length: usize) -> usize {
        let base_row = match chain_length {
            2 => 0,
            3 => 1,
            4 => 2,
            5 => 3,
            6..=10 => 4,
            11..=20 => 5,
            21..=30 => 6,
            31..=40 => 7,
            _ => 8,
        };
        base_row + self.get_hotel_row_advantage()
    }

    pub fn get_stock_value(&self, chain_length: usize) -> usize {
        (self.get_row_from_chain_length(chain_length) + 2) * 100
    }

    pub fn get_majority_holder_bonus(&self, chain_length: usize) -> usize {
        self.get_stock_value(chain_length) * 10
    }

    pub fn get_minority_holder_bonus(&self, chain_length: usize) -> usize {
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
