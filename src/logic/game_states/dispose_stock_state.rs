use crate::logic::hotel_data::Hotel;

pub struct DisposeStockState {
    pub merge_maker: usize,
    pub surviving_chain: Hotel,
    pub defunct_chain: Hotel,
    pub remaining_shares_per_player: Vec<u32>,
}

impl DisposeStockState {
    pub fn new(
        merge_maker: usize,
        surviving_chain: Hotel,
        defunct_chain: Hotel,
        remaining_shares_per_player: Vec<u32>,
    ) -> Self {
        DisposeStockState {
            merge_maker,
            surviving_chain,
            defunct_chain,
            remaining_shares_per_player,
        }
    }

    pub fn get_remaining_shares(&self, player: usize) -> u32 {
        self.remaining_shares_per_player[player]
    }

    // This is called when a player has decided what to do with their stock
    // It returns true if all players have disposed of their stock
    // signaling this state is over
    pub fn player_handled_stock(&mut self, player: usize, shares: u32) -> bool {
        if shares > self.remaining_shares_per_player[player] {
            panic!("Player cannot dispose of more shares than they have");
        }

        self.remaining_shares_per_player[player] -= shares;
        self.remaining_shares_per_player.iter().all(|&shares| shares == 0)
    }
}
