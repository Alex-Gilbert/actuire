use super::{hotel_data::Hotel, tile::Tile};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Player {
    pub name: String,
    pub stocks: [usize; Hotel::count()],
    pub cash: usize,
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
