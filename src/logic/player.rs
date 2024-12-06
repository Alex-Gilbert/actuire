use super::{hotel_data::Hotel, tile::Tile};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Player {
    pub name: String,
    pub stocks: [u32; Hotel::count()],
    pub cash: u32,
    pub tiles: Vec<Tile>,
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            stocks: [0; Hotel::count()],
            cash: 6000,
            tiles: Vec::new(),
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new("")
    }
}
