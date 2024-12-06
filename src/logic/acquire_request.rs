// This represents what the "game" is asking for
// in most cases the usize stored is the player being asked
pub enum AcquireRequest {
    PlayStartingTile(usize),
    PlayTile(usize),
    ChooseNewChain(usize),
    ChooseMergerSurvivor(usize),
    ChooseDefunctChainToResolve(usize),
    //Dispose stock can be done in any order, so no need to ask a specific player
    DisposeStock,
    BuyStock(usize),
    EndGame(usize),
}
