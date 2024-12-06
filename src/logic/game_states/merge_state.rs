use crate::logic::{game_board::{CellConflictType, GameBoard}, hotel_data::Hotel};

pub struct MergerState {
    pub merge_maker: usize,
    pub surviving_hotel: Hotel,
    pub hotel_to_merge: Option<Hotel>,
    pub defunct_hotels_remaining: Vec<Hotel>,
}

impl MergerState {
    pub fn new(merge_maker: usize, surviving_hotel: Hotel, board: &GameBoard) -> Self {
        let defunct_hotels_remaining: Vec<Hotel> = match board.get_conflict_on_board() {
            Some((row, column, CellConflictType::Merge(_))) => board
                .get_adjacent_hotels(row, column)
                .into_iter()
                .filter(|hotel| *hotel != surviving_hotel)
                .collect(),
            _ => panic!("Cannot begin merge without a merger conflict on the board"),
        };

        MergerState {
            merge_maker,
            surviving_hotel,
            hotel_to_merge: None,
            defunct_hotels_remaining,
        }
    }

    pub fn get_largest_defunct_chains(&self, board: &GameBoard) -> Vec<Hotel> {
        let mut defunct_hotels = self.defunct_hotels_remaining.clone();

        let largest_defunct_chain_size = self.defunct_hotels_remaining
            .iter()
            .map(|hotel| board.get_hotel_chain_size(*hotel))
            .max()
            .unwrap();

        defunct_hotels
            .retain(|hotel| board.get_hotel_chain_size(*hotel) == largest_defunct_chain_size);

        defunct_hotels
    }

    // This function is used to determine the next defunct hotel to resolve
    // If there is only one defunct hotel, it will return that hotel
    // If there are multiple defunct hotels, it will return None signaling that the player must choose
    pub fn get_next_defunct_hotel(&self, board: &GameBoard) -> Option<Hotel> {
        let largest_defunct_chains = self.get_largest_defunct_chains(board);

        if largest_defunct_chains.len() == 1 {
            return Some(largest_defunct_chains[0]);
        }

        None
    }

    // The game should call this after a defunct hotel has been resolved
    // It returns true if all defunct hotels have been resolved
    // signaling this state is over
    pub fn defunct_hotel_resolved(&mut self, hotel: Hotel) -> bool {
        if !self.defunct_hotels_remaining.contains(&hotel) {
            panic!("Cannot resolve a defunct hotel that is not in the list of defunct hotels");
        }

        self.hotel_to_merge = Some(hotel);
        self.defunct_hotels_remaining.retain(|h| *h != hotel);

        self.defunct_hotels_remaining.len() == 0
    }
}
