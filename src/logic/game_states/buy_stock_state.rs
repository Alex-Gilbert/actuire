use crate::logic::acquire_constants::STOCK_TO_BUY_PER_TURN;

pub struct BuyStockState {
    pub player: usize,
    pub buys_remaining: u32,
}

impl BuyStockState {
    pub fn new(player: usize) -> Self {
        BuyStockState {
            player,
            buys_remaining: STOCK_TO_BUY_PER_TURN,
        }
    }

    // This is called when a player has bought stock
    // It returns true if the player has bought all the stock they can
    // signaling this state is over
    pub fn player_has_bought_stock(&mut self) -> bool {
        if self.buys_remaining == 0 {
            panic!("Player cannot buy more stock");
        }

        self.buys_remaining -= 1;
        self.buys_remaining == 0
    }
}
