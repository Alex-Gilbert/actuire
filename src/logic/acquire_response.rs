use super::{hotel_data::Hotel, tile::Tile};

pub enum DisposeStockChoice {
    Keep,
    Sell,
    Trade,
    SellAll,
    KeepAll,
    TradeAll,
}

pub enum BuyStockChoice {
    Buy(Hotel),
    Pass,
}

// This represents the response from the player
pub enum AcquireResponse {
    StartingTile,
    Tile(Tile),
    NewChain(Hotel),
    DefunctChainToResolve(Hotel),
    MergerSurvivor(Hotel),
    DisposeStock(usize, DisposeStockChoice),
    BuyStock(BuyStockChoice),
    EndGame(bool),
}

pub struct AcquirePlayerResponse {
    pub player: usize,
    pub response: AcquireResponse,
}

impl AcquirePlayerResponse {
    pub fn new(response: AcquireResponse, player: usize) -> Self {
        AcquirePlayerResponse { response, player }
    }
}
