use core::panic;
use std::{collections::HashSet, sync::Arc, usize};

use rand::seq::IteratorRandom;

use crate::logic::game_board::Cell;

use super::{
    acquire_constants::{MAX_NUMBER_OF_PLAYERS, MAX_STOCK_PER_HOTEL},
    acquire_game_state::AcquireGameState,
    acquire_request::AcquireRequest,
    acquire_response::{AcquireResponse, BuyStockChoice, DisposeStockChoice},
    game_board::{self, CellNotPlayableReason, GameBoard},
    game_states::{
        buy_stock_state::BuyStockState, dispose_stock_state::DisposeStockState,
        game_start_state::GameStartState, merge_state::MergerState,
    },
    hotel_data::Hotel,
    player::Player,
    tile::Tile,
};

pub trait AcquireGameCallback: Send + Sync {
    fn send_message(&self, message: &str);
}

pub struct AcquireGame<T: AcquireGameCallback> {
    pub players: Vec<Player>,
    message_callback: Arc<T>,
    pub board: GameBoard,
    available_tiles: HashSet<Tile>,
    available_stock: [u32; Hotel::count()],
    current_request: AcquireRequest,
    current_state: AcquireGameState,
}

impl<T: AcquireGameCallback> AcquireGame<T> {
    pub fn new(number_of_players: usize, message_callback: Arc<T>) -> Self {
        let mut available_tiles = HashSet::new();

        // add all tiles to the available tiles
        for row in 0..game_board::BOARD_ROWS {
            for col in 0..game_board::BOARD_COLS {
                available_tiles.insert(Tile::from((row, col)));
            }
        }

        let mut players = Vec::new();
        for i in 0..number_of_players {
            players.push(Player::new(format!("Player {}", i + 1).as_str()));
        }

        let board = GameBoard::new();

        message_callback.send_message("Welcome to Acquire!");

        Self {
            players,
            message_callback,
            board,
            available_tiles,
            available_stock: [MAX_STOCK_PER_HOTEL; Hotel::count()],
            current_request: AcquireRequest::PlayStartingTile(0),
            current_state: AcquireGameState::GameStart(GameStartState::new(number_of_players)),
        }
    }

    pub fn get_current_stock_availability(&self, hotel: Hotel) -> u32 {
        self.available_stock[hotel as usize]
    }

    pub fn get_current_stock_price(&self, hotel: Hotel) -> u32 {
        hotel.get_stock_value(self.board.get_hotel_chain_size(hotel))
    }

    pub fn get_current_request(&self) -> &AcquireRequest {
        &self.current_request
    }

    pub fn handle_player_response(&mut self, response: AcquireResponse) -> Result<(), String> {
        match self.current_request {
            AcquireRequest::PlayStartingTile(player) => {
                if let AcquireResponse::StartingTile = response {
                    let tile = self.take_random_tile();
                    self.handle_starting_tile_response(tile, player);
                    Ok(())
                } else {
                    Err("Invalid response to starting tile request".to_string())
                }
            }
            AcquireRequest::PlayTile(player) => {
                if let AcquireResponse::Tile(tile) = response {
                    self.handle_tile_response(tile, player)
                } else {
                    Err("Invalid response to tile request".to_string())
                }
            }
            AcquireRequest::ChooseNewChain(player) => {
                if let AcquireResponse::NewChain(hotel) = response {
                    self.handle_new_chain_response(hotel, player)
                } else {
                    Err("Invalid response to new chain request".to_string())
                }
            }
            AcquireRequest::ChooseMergerSurvivor(player) => {
                if let AcquireResponse::MergerSurvivor(hotel) = response {
                    self.handle_merger_survivor_response(hotel, player);
                    Ok(())
                } else {
                    Err("Invalid response to merger survivor request".to_string())
                }
            }
            AcquireRequest::ChooseDefunctChainToResolve(player) => {
                if let AcquireResponse::DefunctChainToResolve(hotel) = response {
                    self.handle_defunct_chain_response(hotel, player);
                    Ok(())
                } else {
                    Err("Invalid response to defunct chain request".to_string())
                }
            }
            AcquireRequest::DisposeStock => {
                if let AcquireResponse::DisposeStock(player, choice) = response {
                    self.handle_dispose_stock_response(choice, player)
                } else {
                    Err("Invalid response to dispose stock request".to_string())
                }
            }
            AcquireRequest::BuyStock(_) => {
                if let AcquireResponse::BuyStock(choice) = response {
                    self.handle_buy_stock_response(choice)
                } else {
                    Err("Invalid response to buy stock request".to_string())
                }
            }
            AcquireRequest::EndGame(player) => {
                if let AcquireResponse::EndGame(quit) = response {
                    self.handle_end_game_response(quit, player);
                    Ok(())
                } else {
                    Err("Invalid response to end game request".to_string())
                }
            }
        }
    }

    // when a player is asked to choose a hotel to start a new chain or resolve a conflict
    // this function is called to get the hotels that are acceptable for the player to choose
    pub fn get_acceptable_hotels_for_response(&self) -> Vec<Hotel> {
        let board = &self.board;
        match self.current_request {
            AcquireRequest::ChooseNewChain(_) => self.board.get_inactive_hotels(),
            AcquireRequest::ChooseMergerSurvivor(_) => self.board.acceptable_conflict_resolutions(),
            AcquireRequest::ChooseDefunctChainToResolve(_) => {
                if let AcquireGameState::Merger(merge_state) = &self.current_state {
                    merge_state.get_largest_defunct_chains(board)
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    fn end_turn(&mut self, player: usize) {
        self.message_callback.send_message(&format!("{}'s turn has ended", self.players[player].name));
        self.give_player_tile(player);

        let next_player = (player + 1) % self.players.len();
        self.message_callback.send_message(&format!(
            "It is now {}'s turn",
            self.players[next_player].name
        ));

        let play_tile_state = AcquireGameState::PlayTile(next_player);
        self.current_request = AcquireRequest::PlayTile(next_player);
        self.current_state = play_tile_state;
    }

    pub fn get_number_of_tiles_left(&self) -> usize {
        self.available_tiles.len()
    }

    pub fn take_random_tile(&mut self) -> Tile {
        let mut rng = rand::thread_rng();
        let tile = self
            .available_tiles
            .iter()
            .choose(&mut rng)
            .unwrap()
            .clone();
        self.available_tiles.remove(&tile);
        tile
    }

    fn handle_starting_tile_response(&mut self, tile: Tile, player: usize) {
        // confirm the tile is not in the available tiles
        assert!(!self.available_tiles.contains(&tile));
        // confirm that the tile is not already on the board
        assert!(self.board.get_cell_state(tile.row, tile.col) == Cell::Empty);

        if let AcquireGameState::GameStart(ref mut game_start_state) = self.current_state {
            self.board.place_initial_tile(tile.row, tile.col);
            self.message_callback.send_message(&format!(
                "{} placed starting tile {}",
                self.players[player].name, tile
            ));

            if game_start_state.player_played_tile(player, tile) {
                self.handle_game_start_complete();
            } else {
                self.current_request =
                    AcquireRequest::PlayStartingTile((player + 1) % self.players.len());
            }
        } else {
            panic!("Cannot handle starting tile response without a game start state");
        }
    }

    fn handle_tile_response(
        &mut self,
        tile: Tile,
        player: usize,
    ) -> Result<(), String> {
        // confirm the tile is not available
        assert!(!self.available_tiles.contains(&tile));
        // confirm that the tile is not already on the board
        assert!(self.board.get_cell_state(tile.row, tile.col) == Cell::Empty);
        // confirm that the player has the tile in their hand
        assert!(self.players[player].tiles.contains(&tile));

        // place tile on board
        let place_tile_result = self.board.place_tile(tile.row, tile.col);
        match place_tile_result {
            game_board::PlaceTileResult::Success => {
                // the with no chains created or mergers started, the player can buy stock
                self.message_callback.send_message(&format!(
                    "{} placed tile {}",
                    self.players[player].name, tile
                ));
                self.start_buy_stock_phase(player);
            }

            game_board::PlaceTileResult::ConflictCreated(conflict_type) => {
                self.message_callback.send_message(&format!(
                    "{} placed tile {}",
                    self.players[player].name, tile
                ));
                match conflict_type {
                    game_board::CellConflictType::NewChain => {
                        self.message_callback.send_message(&format!(
                            "A new chain has been started!",
                        ));
                        self.message_callback.send_message(&format!(
                            "{} must choose a new chain to start",
                            self.players[player].name
                        ));
                        self.current_request = AcquireRequest::ChooseNewChain(player);
                    }

                    game_board::CellConflictType::Merge(_) => {
                        // we need to check if this merger has a tie
                        // in which case players must choose which chain to keep

                        // acceptable conflict resolutions in this case are hotels that could survive the merger
                        let largest_chains_in_merger = self.board.acceptable_conflict_resolutions();

                        if largest_chains_in_merger.len() == 1 {
                            // there is no tie in this case; the largest chain is the one to survive
                            // and we can start the merge phase
                            self.start_merge_phase(player, largest_chains_in_merger[0]);
                        } else {
                            // there is a tie; players must choose which chain to keep before the merge phase
                            self.message_callback.send_message(&format!(
                                "A merge has been triggered, but there is a tie!",
                            ));
                            self.message_callback.send_message(&format!(
                                "{} must choose a chain to keep",
                                self.players[player].name
                            ));
                            self.current_request = AcquireRequest::ChooseMergerSurvivor(player);
                        }
                    }
                }
            }
            game_board::PlaceTileResult::CellNotPlayable(not_playable_reason) => {
                return Err(not_playable_reason.as_display_message());
            }
        }

        // remove tile from player's hand
        self.players[player].tiles.retain(|t| t != &tile);
        Ok(())
    }

    fn handle_new_chain_response(&mut self, hotel: Hotel, player: usize) -> Result<(), String> {
        // confirm that the hotel is not already on the board
        if !self.board.get_inactive_hotels().contains(&hotel) {
            return Err("Hotel is already on the board".to_string());
        }

        self.board.resolve_conflict(hotel);

        self.message_callback.send_message(&format!(
            "{} has started a new chain in {}",
            self.players[player].name, hotel
        ));

        if self.available_stock[hotel as usize] > 0 {
            self.message_callback.send_message(&format!(
                "{} will recieve founding bonus stock in {}",
                self.players[player].name, hotel
            ));
            self.give_player_stock(hotel, player, 1);
        } else {
            self.message_callback.send_message(&format!(
                "No {} stock is available. Founding bonus will not be given to {}",
                self.players[player].name, hotel
            ));
        }

        self.start_buy_stock_phase(player);
        Ok(())
    }

    fn handle_merger_survivor_response(&mut self, hotel: Hotel, player: usize) {
        self.message_callback.send_message(&format!(
            "{} has chosen {} to survive the merger",
            self.players[player].name, hotel
        ));

        self.start_merge_phase(player, hotel);
    }

    fn handle_defunct_chain_response(&mut self, hotel: Hotel, player: usize) {
        self.message_callback.send_message(&format!(
            "{} has chosen to resolve {}",
            self.players[player].name, hotel
        ));
        self.handle_defunct_hotel(hotel);
    }

    fn handle_dispose_stock_response(
        &mut self,
        choice: DisposeStockChoice,
        player: usize,
    ) -> Result<(), String> {
        if let AcquireGameState::DisposeStock(dispose_stock_state) = &mut self.current_state {
            let defunct_chain = dispose_stock_state.defunct_chain;
            let merge_maker = dispose_stock_state.merge_maker;
            let merge_survivor = dispose_stock_state.surviving_chain;

            let remaining_shares = dispose_stock_state.get_remaining_shares(player);

            if remaining_shares == 0 {
                return Err("You have already disposed of all your shares".to_string());
            }

            let shares_to_handle = match choice {
                DisposeStockChoice::Keep => 1,
                DisposeStockChoice::Sell => 1,
                DisposeStockChoice::Trade => 2,
                DisposeStockChoice::SellAll => remaining_shares,
                DisposeStockChoice::KeepAll => remaining_shares,
                // can only trade even number of shares
                DisposeStockChoice::TradeAll => (remaining_shares / 2) * 2,
            };

            if shares_to_handle > remaining_shares {
                return Err("You cannot dispose of more shares than you have".to_string());
            }

            if shares_to_handle == 0 {
                return Err("You cannot trade with only 1 share".to_string());
            }

            let next_phase;
            match choice {
                DisposeStockChoice::Keep | DisposeStockChoice::KeepAll => {
                    self.message_callback.send_message(&format!(
                        "{} has chosen to keep {} stock in {}",
                        self.players[player].name,
                        shares_to_handle,
                        dispose_stock_state.defunct_chain
                    ));
                    next_phase = dispose_stock_state.player_handled_stock(player, 1);
                }
                DisposeStockChoice::Sell | DisposeStockChoice::SellAll => {
                    self.message_callback.send_message(&format!(
                        "{} has chosen to sell {} stock in {}",
                        self.players[player].name,
                        shares_to_handle,
                        dispose_stock_state.defunct_chain
                    ));

                    next_phase = dispose_stock_state.player_handled_stock(player, shares_to_handle);
                    self.sell_off_players_stock(defunct_chain, player, shares_to_handle);
                }
                DisposeStockChoice::Trade | DisposeStockChoice::TradeAll => {
                    // check if there is enough stock available to trade
                    let stock_to_receive = shares_to_handle / 2;
                    if self.available_stock[merge_survivor as usize] < stock_to_receive {
                        return Err(format!(
                            "Not enough stock available in {} to trade",
                            merge_survivor
                        ));
                    }

                    next_phase = dispose_stock_state.player_handled_stock(player, shares_to_handle);
                    self.give_player_stock(merge_survivor, player, stock_to_receive);
                    self.take_back_players_stock(defunct_chain, player, shares_to_handle);
                }
            }

            if next_phase {
                self.message_callback.send_message(&format!(
                    "All players have disposed of their stock in {}",
                    defunct_chain
                ));

                // All hotel tiles of the old chain are removed from the board and replaced with the new chain
                self.board
                    .replace_defunct_hotel_with_surviving_hotel(defunct_chain, merge_survivor);
                self.continue_merge_phase(merge_maker, merge_survivor);
            }
        } else {
            panic!("Cannot handle dispose stock response without a dispose stock state");
        }

        Ok(())
    }

    fn handle_buy_stock_response(&mut self, choice: BuyStockChoice) -> Result<(), String> {
        if let AcquireGameState::BuyStock(buy_stock_state) = &mut self.current_state {
            let player = buy_stock_state.player;
            let end_phase;
            match choice {
                BuyStockChoice::Pass => {
                    self.message_callback.send_message(&format!(
                        "{} has chosen to pass on buying stock",
                        self.players[player].name
                    ));

                    end_phase = true;
                }
                BuyStockChoice::Buy(hotel) => {
                    if self.available_stock[hotel as usize] == 0 {
                        return Err(format!("No {} stock available to buy", hotel).to_string());
                    }

                    let stock_value = hotel.get_stock_value(self.board.get_hotel_chain_size(hotel));

                    if stock_value > self.players[player].cash {
                        return Err("You do not have enough cash to buy stock".to_string());
                    }

                    if !self.board.get_active_hotels().contains(&hotel) {
                        return Err("You cannot buy stock in an inactive chain".to_string());
                    }

                    self.message_callback.send_message(&format!(
                        "{} has chosen to buy stock in {}",
                        self.players[player].name, hotel
                    ));

                    end_phase = buy_stock_state.player_has_bought_stock();
                    self.players[player].cash -= stock_value;
                    self.give_player_stock(hotel, player, 1);
                }
            }
            if end_phase {
                self.end_turn(player);
            }
        } else {
            panic!("Cannot handle buy stock response without a buy stock state");
        }

        Ok(())
    }

    fn handle_end_game_response(&self, quit: bool, player: usize) {}

    /// when a defunct chain is chosen to be resolved
    /// this function is called to pay out the defunct
    // and begin the stock disposal process
    fn pay_out_defunct_chain(&mut self, defunct_hotel: Hotel) {
        let defunct_chain_size = self.board.get_hotel_chain_size(defunct_hotel);
        let majority_payout = defunct_hotel.get_majority_holder_bonus(defunct_chain_size);
        let minority_payout = defunct_hotel.get_minority_holder_bonus(defunct_chain_size);

        let mut majority_indices: [isize; 6] = [-1; 6];
        let mut minority_indices: [isize; 6] = [-1; 6];
        let mut max_shares = 0;
        let mut second_max_shares = 0;

        // Find the highest and second-highest share counts
        for player in &self.players {
            let shares = player.stocks[defunct_hotel as usize];
            if shares > max_shares {
                second_max_shares = max_shares;
                max_shares = shares;
            } else if shares > second_max_shares && shares < max_shares {
                second_max_shares = shares;
            }
        }

        // Classify stockholders into majority and minority, using indices
        let mut maj_count = 0;
        let mut min_count = 0;
        for (index, player) in self.players.iter().enumerate() {
            let shares = player.stocks[defunct_hotel as usize];
            if shares == max_shares {
                majority_indices[maj_count] = index as isize;
                maj_count += 1;
            } else if shares == second_max_shares {
                minority_indices[min_count] = index as isize;
                min_count += 1;
            }
        }

        // Distribute payouts to majority stockholders
        let mut total_majority_payout = majority_payout as u32;
        let mut total_minority_payout = minority_payout as u32;

        // if there are no minority stockholders or multiple majority stockholders, the minority payout is 0
        if min_count == 0 || maj_count > 1 {
            total_majority_payout += minority_payout as u32;
            total_minority_payout = 0;
        }

        if maj_count == 1 {
            self.message_callback.send_message(&format!(
                "{} has the majority in the defunct chain",
                self.players[majority_indices[0] as usize].name,
            ));
        } else {
            self.message_callback.send_message(&format!(
                "The following players have the majority in the defunct chain:",
            ));
            for i in 0..maj_count {
                self.message_callback.send_message(&format!(
                    "{}",
                    self.players[majority_indices[i] as usize].name,
                ));
            }
        }
        self.distribute_payouts_array(majority_indices, maj_count, total_majority_payout);

        // Distribute payouts to minority stockholders, if there is a single majority stockholder
        if total_minority_payout > 0 {
            if min_count == 1 {
                self.message_callback.send_message(&format!(
                    "{} has the minority in the defunct chain",
                    self.players[minority_indices[0] as usize].name,
                ));
            } else {
                self.message_callback.send_message(&format!(
                    "The following players have the minority in the defunct chain:",
                ));
                for i in 0..min_count {
                    self.message_callback.send_message(&format!(
                        "{}",
                        self.players[minority_indices[i] as usize].name,
                    ));
                }
            }
            self.distribute_payouts_array(minority_indices, min_count, total_minority_payout);
        }
    }

    fn distribute_payouts_array(&mut self, indices: [isize; 6], count: usize, total_payout: u32) {
        if count == 0 {
            return;
        }

        let payout_per_player = total_payout / (count as u32);
        for &index in indices.iter().take(count) {
            if index != -1 {
                self.message_callback.send_message(&format!(
                    "{} receives a payout of ${}",
                    self.players[index as usize].name, payout_per_player
                ));
                self.players[index as usize].cash += payout_per_player;
            }
        }
    }

    fn sell_off_players_stock(&mut self, hotel: Hotel, player: usize, shares: u32) {
        if self.players[player].stocks[hotel as usize] < shares {
            return;
        }

        let stock_value = hotel.get_stock_value(self.board.get_hotel_chain_size(hotel));
        let payout = stock_value * shares;
        self.players[player].cash += payout;
        self.available_stock[hotel as usize] += shares;
        self.players[player].stocks[hotel as usize] -= shares;

        self.message_callback.send_message(&format!(
            "{} has sold {} stock in {} for ${}",
            self.players[player].name, shares, hotel, payout
        ));
    }

    fn give_player_stock(&mut self, hotel: Hotel, player: usize, shares: u32) {
        if self.available_stock[hotel as usize] < shares {
            return;
        }

        self.message_callback.send_message(&format!(
            "{} has received {} stock in {}",
            self.players[player].name, shares, hotel
        ));

        self.players[player].stocks[hotel as usize] += shares;
        self.available_stock[hotel as usize] -= shares;
    }

    fn take_back_players_stock(&mut self, hotel: Hotel, player: usize, shares: u32) {
        if self.players[player].stocks[hotel as usize] < shares {
            return;
        }

        self.players[player].stocks[hotel as usize] -= shares;
        self.available_stock[hotel as usize] += shares;
    }

    fn start_buy_stock_phase(&mut self, player: usize) {
        let active_hotels = self.board.get_active_hotels();
        if active_hotels.is_empty() {
            self.message_callback.send_message("There are no active chains to buy stock in");
            self.end_turn(player);
            return;
        }

        if active_hotels
            .iter()
            .all(|hotel| self.available_stock[*hotel as usize] == 0)
        {
            self.message_callback.send_message("There is no stock available to buy");
            self.end_turn(player);
            return;
        }

        let buy_stock_state = BuyStockState::new(player);

        self.current_request = AcquireRequest::BuyStock(player);
        self.current_state = AcquireGameState::BuyStock(buy_stock_state);
    }

    fn start_merge_phase(&mut self, merge_maker: usize, merge_survivor: Hotel) {
        let merge_state = MergerState::new(merge_maker, merge_survivor, &self.board);

        self.message_callback.send_message(&format!(
            "{} has triggered a merge!",
            self.players[merge_maker].name
        ));

        self.message_callback.send_message(&format!(
            "The following hotel(s) will be merged into {}:",
            merge_survivor
        ));

        for hotel in merge_state.defunct_hotels_remaining.iter() {
            self.message_callback.send_message(&format!("{}", hotel));
        }

        self.current_state = AcquireGameState::Merger(merge_state);
        self.determine_next_defunct_hotel();
    }

    fn determine_next_defunct_hotel(&mut self) {
        if let AcquireGameState::Merger(merge_state) = &self.current_state {
            let merge_maker = merge_state.merge_maker;

            match merge_state.get_next_defunct_hotel(&self.board) {
                Some(defunct_hotel) => {
                    self.handle_defunct_hotel(defunct_hotel);
                }
                None => {
                    self.message_callback.send_message(&format!(
                        "There is a tie in the defunct chains! {} must choose which chain to resolve first",
                        self.players[merge_maker].name
                    ));

                    self.current_request = AcquireRequest::ChooseDefunctChainToResolve(merge_maker);
                }
            }
        } else {
            panic!("Cannot determine next defunct hotel without a merge state");
        }
    }

    fn continue_merge_phase(&mut self, merge_maker: usize, merge_survivor: Hotel) {
        let merge_state = MergerState::new(merge_maker, merge_survivor, &self.board);

        if merge_state.defunct_hotels_remaining.is_empty() {
            self.message_callback.send_message(&format!(
                "All defunct chains have been merged"
            ));

            self.message_callback.send_message(&format!(
                "The merger into {} is complete!",
                merge_survivor
            ));

            let _ = self.board.resolve_conflict(merge_survivor);

            self.message_callback.send_message(&format!(
                "{} can now buy stock",
                self.players[merge_maker].name
            ));
            self.start_buy_stock_phase(merge_maker);
        } else {
            self.message_callback.send_message(&format!("Continuing the mergo into {}", merge_survivor));

            self.current_state = AcquireGameState::Merger(merge_state);
            self.determine_next_defunct_hotel();
        }
    }

    fn handle_defunct_hotel(&mut self, defunct_hotel: Hotel) {
        if let AcquireGameState::Merger(merge_state) = &self.current_state {
            let merge_survivor = merge_state.surviving_hotel;

            self.message_callback.send_message(&format!(
                "A merger of {} into {} will now commence",
                defunct_hotel, merge_survivor
            ));

            // pay out the owners of the defunct chain
            self.pay_out_defunct_chain(defunct_hotel);

            // begin the stock disposal phase
            self.begin_stock_disposal(defunct_hotel);
        } else {
            panic!("Cannot handle defunct hotel without a merge state");
        }
    }

    fn begin_stock_disposal(&mut self, defunct_hotel: Hotel) {
        if let AcquireGameState::Merger(merge_state) = &self.current_state {
            let merge_maker = merge_state.merge_maker;
            let merge_survivor = merge_state.surviving_hotel;

            let mut remaining_shares_per_player = Vec::new();
            for player in &self.players {
                remaining_shares_per_player.push(player.stocks[defunct_hotel as usize]);
            }

            let dispose_stock_state = DisposeStockState::new(
                merge_maker,
                merge_survivor,
                defunct_hotel,
                remaining_shares_per_player,
            );

            self.current_state = AcquireGameState::DisposeStock(dispose_stock_state);
            self.current_request = AcquireRequest::DisposeStock;

            self.message_callback.send_message(&format!(
                "Players must now dispose of their stock in {}",
                defunct_hotel
            ));
        } else {
            panic!("Cannot begin stock disposal without a merge state");
        }
    }

    fn handle_game_start_complete(&mut self) {
        if let AcquireGameState::GameStart(game_start_state) = &self.current_state {
            self.message_callback.send_message(&format!("All players have placed their starting tiles."));

            let player_with_winning_tile = game_start_state.player_with_winning_tile;
            let winning_tile = game_start_state.winning_tile;
            self.message_callback.send_message(&format!(
                "{} has the winning tile {} and will start the game!",
                self.players[player_with_winning_tile].name, winning_tile
            ));

            // give each player 6 tiles
            for _ in 0..6 {
                for player in 0..self.players.len() {
                    self.give_player_tile(player);
                }
            }

            self.current_state = AcquireGameState::PlayTile(player_with_winning_tile);
            self.current_request = AcquireRequest::PlayTile(player_with_winning_tile);
        } else {
            panic!("Cannot handle game start complete without a game start state");
        }
    }

    fn give_player_tile(&mut self, player: usize) {
        let tile = self.take_random_tile();
        self.players[player].tiles.push(tile);
    }

    pub(crate) fn get_current_chain_size(&self, hotel: Hotel) -> usize {
        self.board.get_hotel_chain_size(hotel)
    }
}
