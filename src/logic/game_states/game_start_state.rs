use crate::logic::tile::Tile;

pub struct GameStartState {
    pub player_with_winning_tile: usize,
    pub winning_tile: Tile,
    pub remaining_number_of_players: usize,
}

impl GameStartState {
    pub fn new(number_of_players: usize) -> Self {
        GameStartState {
            player_with_winning_tile: usize::MAX,
            winning_tile: Tile::from((usize::MAX, usize::MAX)),
            remaining_number_of_players: number_of_players,
        }
    }

    // call this after a player has played their starting tile
    // returns true if all players have played their starting tile
    pub fn player_played_tile(&mut self, player: usize, tile: Tile) -> bool {
        if self.remaining_number_of_players == 0 {
            panic!("GameStartState: All players have already played their tiles");
        }

        if self.winning_tile > tile {
            self.winning_tile = tile;
            self.player_with_winning_tile = player;
        }

        self.remaining_number_of_players -= 1;

        self.remaining_number_of_players == 0
    }
}
