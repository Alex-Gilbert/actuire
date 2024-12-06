use std::usize;

use super::game_states::{
    buy_stock_state::BuyStockState, dispose_stock_state::DisposeStockState, game_start_state::GameStartState, merge_state::MergerState
};


pub enum AcquireGameState {
    GameStart(GameStartState),
    PlayTile(usize),
    DisposeStock(DisposeStockState),
    Merger(MergerState),
    BuyStock(BuyStockState),
    EndGame(usize),
}
