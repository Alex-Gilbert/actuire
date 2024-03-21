use std::collections::HashSet;

use super::{game_board::{self, GameBoard}, hotel_data::Hotel, player::Player, tile::Tile};

pub struct AcquireGame {
    pub players: Vec<Player>,
    pub board: GameBoard,
    pub current_player: usize,
    pub available_tiles: HashSet<Tile>,
    pub available_stock: [usize; Hotel::count()],
}

impl AcquireGame {
    pub fn new(number_of_players: usize) -> Self {
        let mut available_tiles = HashSet::new();
        for row in 0..game_board::BOARD_ROWS {
            for col in 0..game_board::BOARD_COLS {
                available_tiles.insert(Tile::from((row, col)));
            }
        }

        let mut players = Vec::new();
        for i in 0..number_of_players {
            players.push(Player::new(&format!("Player {}", i + 1)));
        }

        Self {
            players,
            board: GameBoard::new(),
            current_player: 0,
            available_tiles,
            available_stock: [25; Hotel::count()],
        }
    }
}
