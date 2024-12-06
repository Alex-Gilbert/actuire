use core::fmt;
use std::fmt::Debug;

use super::hotel_data::Hotel;

pub const BOARD_ROWS: usize = 9;
pub const BOARD_COLS: usize = 12;
pub const SAFE_CHAIN_SIZE: usize = 11;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellConflictType {
    NewChain,
    Merge(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Cell {
    Empty,
    Independent,
    Conflict(CellConflictType),
    Hotel(Hotel),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellNotPlayableReason {
    ConflictOnBoard,
    HotelsAreAllActive,
    AdjacentHotelsAreSafe,
    CellIsNotEmpty,
    CellIsOffBoard,
}

impl CellNotPlayableReason {
    pub fn as_display_message(&self) -> String {
        match self {
            CellNotPlayableReason::ConflictOnBoard => "There is a conflict on the board".to_string(),
            CellNotPlayableReason::HotelsAreAllActive => "All hotels are currently active, cannot start a new chain".to_string(),
            CellNotPlayableReason::AdjacentHotelsAreSafe => "Adjacent hotels are safe and cannot be merged".to_string(),
            CellNotPlayableReason::CellIsNotEmpty => "The cell is not empty".to_string(),
            CellNotPlayableReason::CellIsOffBoard => "The cell is off the board".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlaceTileResult {
    Success,
    CellNotPlayable(CellNotPlayableReason),
    ConflictCreated(CellConflictType),
}

pub struct GameBoard {
    pub cells: [[Cell; BOARD_COLS]; BOARD_ROWS],
}

impl GameBoard {
    pub fn new() -> GameBoard {
        GameBoard {
            cells: [[Cell::Empty; BOARD_COLS]; BOARD_ROWS],
        }
    }

    pub fn get_cell_state(&self, row: usize, col: usize) -> Cell {
        match row < BOARD_ROWS && col < BOARD_COLS {
            true => self.cells[row][col],
            false => Cell::Empty,
        }
    }

    pub fn get_hotel_at(&self, row: usize, col: usize) -> Option<Hotel> {
        match self.cells[row][col] {
            Cell::Hotel(hotel) => Some(hotel),
            _ => None,
        }
    }

    pub fn get_active_hotels(&self) -> Vec<Hotel> {
        let mut hotel_is_active = [false; Hotel::count()];

        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if let Cell::Hotel(hotel) = self.cells[row][col] {
                    hotel_is_active[hotel as usize] = true;
                }
            }
        }

        let mut active_hotels = Vec::new();
        for hotel in Hotel::iter() {
            if hotel_is_active[hotel as usize] {
                active_hotels.push(hotel);
            }
        }
        active_hotels
    }

    pub fn get_inactive_hotels(&self) -> Vec<Hotel> {
        let active_hotels = self.get_active_hotels();
        Hotel::iter()
            .filter(|hotel| !active_hotels.contains(hotel))
            .collect()
    }

    pub fn get_hotel_chain_size(&self, hotel: Hotel) -> usize {
        let mut chain_size = 0;
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if let Cell::Hotel(h) = self.cells[row][col] {
                    if h == hotel {
                        chain_size += 1;
                    }
                }
            }
        }
        chain_size
    }

    pub fn get_adjacent_hotels(&self, row: usize, col: usize) -> Vec<Hotel> {
        let mut hotel_is_adjacent = [false; Hotel::count()];

        let (left, _) = col.overflowing_sub(1);
        let right = col + 1;

        let (up, _) = row.overflowing_sub(1);
        let down = row + 1;

        if let Cell::Hotel(hotel) = self.get_cell_state(row, right) {
            hotel_is_adjacent[hotel as usize] = true;
        }

        if let Cell::Hotel(hotel) = self.get_cell_state(row, left) {
            hotel_is_adjacent[hotel as usize] = true;
        }

        if let Cell::Hotel(hotel) = self.get_cell_state(down, col) {
            hotel_is_adjacent[hotel as usize] = true;
        }

        if let Cell::Hotel(hotel) = self.get_cell_state(up, col) {
            hotel_is_adjacent[hotel as usize] = true;
        }

        let mut active_hotels = Vec::new();
        for hotel in Hotel::iter() {
            if hotel_is_adjacent[hotel as usize] {
                active_hotels.push(hotel);
            }
        }

        active_hotels
    }

    fn is_cell_next_to_independent(&self, row: usize, col: usize) -> bool {
        let (left, _) = col.overflowing_sub(1);
        let right = col + 1;

        let (up, _) = row.overflowing_sub(1);
        let down = row + 1;

        if self.get_cell_state(row, right) == Cell::Independent {
            return true;
        }
        if self.get_cell_state(row, left) == Cell::Independent {
            return true;
        }
        if self.get_cell_state(down, col) == Cell::Independent {
            return true;
        }
        if self.get_cell_state(up, col) == Cell::Independent {
            return true;
        }
        false
    }

    fn would_cell_start_new_chain(&self, row: usize, col: usize) -> bool {
        // get the adjacent hotels
        let adjacent_hotels = self.get_adjacent_hotels(row, col);
        adjacent_hotels.len() == 0 && self.is_cell_next_to_independent(row, col)
    }

    fn would_cell_merge_chains(&self, row: usize, col: usize) -> bool {
        let adjacent_hotels = self.get_adjacent_hotels(row, col);
        adjacent_hotels.len() > 1
    }

    pub fn is_cell_playable(&self, row: usize, col: usize) -> Result<bool, CellNotPlayableReason> {
        // check if the cell is off the board
        if row >= BOARD_ROWS || col >= BOARD_COLS {
            return Err(CellNotPlayableReason::CellIsOffBoard);
        }

        // check if there is a conflict on the board
        if let Some(_) = self.get_conflict_on_board() {
            return Err(CellNotPlayableReason::ConflictOnBoard);
        }

        // check if the cell is not empty
        if self.cells[row][col] != Cell::Empty {
            return Err(CellNotPlayableReason::CellIsNotEmpty);
        }

        let active_hotels = self.get_active_hotels();
        let adjacent_hotels = self.get_adjacent_hotels(row, col);

        // check if placing this tile would create an 8th chain
        if active_hotels.len() >= Hotel::count() && self.would_cell_start_new_chain(row, col) {
            return Err(CellNotPlayableReason::HotelsAreAllActive);
        }

        // check if placing a tile would merge safe hotel chains
        if adjacent_hotels.len() > 1 {
            let number_of_adjacent_safe_chains: usize = adjacent_hotels
                .iter()
                .map(|hotel| self.get_hotel_chain_size(*hotel))
                .filter(|chain_size| *chain_size >= SAFE_CHAIN_SIZE)
                .count();

            if number_of_adjacent_safe_chains > 1 {
                return Err(CellNotPlayableReason::AdjacentHotelsAreSafe);
            }
        }

        Ok(true)
    }

    pub fn get_conflict_on_board(&self) -> Option<(usize, usize, CellConflictType)> {
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if let Cell::Conflict(conflict_type) = self.cells[row][col] {
                    return Some((row, col, conflict_type));
                }
            }
        }
        None
    }

    pub fn place_tile(&mut self, row: usize, col: usize) -> PlaceTileResult {
        if let Err(reason) = self.is_cell_playable(row, col) {
            return PlaceTileResult::CellNotPlayable(reason);
        }

        if self.would_cell_start_new_chain(row, col) {
            self.cells[row][col] = Cell::Conflict(CellConflictType::NewChain);
            return PlaceTileResult::ConflictCreated(CellConflictType::NewChain);
        }

        let adjacent_hotels = self.get_adjacent_hotels(row, col);

        if adjacent_hotels.len() > 1 {
            let number_of_mergers = adjacent_hotels.len() - 1;
            self.cells[row][col] = Cell::Conflict(CellConflictType::Merge(number_of_mergers));
            return PlaceTileResult::ConflictCreated(CellConflictType::Merge(number_of_mergers));
        }

        self.cells[row][col] = Cell::Independent;

        if adjacent_hotels.len() == 1 {
            self.flood_hotel(row, col, adjacent_hotels[0]);
            return PlaceTileResult::Success;
        }

        PlaceTileResult::Success
    }

    // use flood to merge chain
    fn flood_hotel(&mut self, row: usize, col: usize, hotel: Hotel) {
        fn fill(board: &mut GameBoard, row: usize, col: usize, hotel: Hotel) {
            let cell_state = board.get_cell_state(row, col);
            if cell_state == Cell::Empty {
                return;
            }
            if let Cell::Hotel(h) = board.get_cell_state(row, col) {
                if h == hotel {
                    return;
                }
            }
            board.cells[row][col] = Cell::Hotel(hotel);

            let (left, _) = col.overflowing_sub(1);
            let right = col + 1;

            let (up, _) = row.overflowing_sub(1);
            let down = row + 1;

            fill(board, down, col, hotel);
            fill(board, up, col, hotel);
            fill(board, row, right, hotel);
            fill(board, row, left, hotel);
        }

        fill(self, row, col, hotel);
    }

    pub fn acceptable_conflict_resolutions(&self) -> Vec<Hotel> {
        if let Some(conflict) = self.get_conflict_on_board() {
            let (row, col, conflict_type) = conflict;
            match conflict_type {
                CellConflictType::NewChain => self.get_inactive_hotels(),
                CellConflictType::Merge(_) => {
                    let adjacent_hotels = self.get_adjacent_hotels(row, col);
                    let max_chain_size = adjacent_hotels
                        .iter()
                        .map(|hotel| self.get_hotel_chain_size(*hotel))
                        .max()
                        .unwrap();

                    adjacent_hotels
                        .iter()
                        .filter(|hotel| self.get_hotel_chain_size(**hotel) == max_chain_size)
                        .cloned()
                        .collect()
                }
            }
        } else {
            Vec::new()
        }
    }

    pub fn resolve_conflict(&mut self, hotel: Hotel) -> Result<(), &str> {
        let acceptable_hotels = self.acceptable_conflict_resolutions();
        if !acceptable_hotels.contains(&hotel) {
            return Err("Hotel is not an acceptable resolution");
        }
        let conflict = self.get_conflict_on_board().unwrap();
        let (row, col, _) = conflict;

        self.flood_hotel(row, col, hotel);
        Ok(())
    }

    pub fn place_initial_tile(&mut self, row: usize, col: usize) {
        self.cells[row][col] = Cell::Independent;
    }

    pub fn get_hotel_stock_price(&self, hotel: Hotel) -> u32 {
        let chain_length = self.get_hotel_chain_size(hotel);
        hotel.get_stock_value(chain_length)
    }

    pub fn get_hotel_majority_stock_bonus(&self, hotel: Hotel) -> u32 {
        let chain_length = self.get_hotel_chain_size(hotel);
        hotel.get_majority_holder_bonus(chain_length)
    }

    pub fn get_hotel_minority_stock_bonus(&self, hotel: Hotel) -> u32 {
        let chain_length = self.get_hotel_chain_size(hotel);
        hotel.get_minority_holder_bonus(chain_length)
    }

    pub fn replace_defunct_hotel_with_surviving_hotel(
        &mut self,
        defunct_hotel: Hotel,
        surviving_hotel: Hotel,
    ) {
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if let Cell::Hotel(hotel) = self.cells[row][col] {
                    if hotel == defunct_hotel {
                        self.cells[row][col] = Cell::Hotel(surviving_hotel);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_board() {
        let game_board = GameBoard::new();
        assert_eq!(game_board.cells.len(), BOARD_ROWS);
        assert_eq!(game_board.cells[0].len(), BOARD_COLS);
    }

    #[test]
    fn test_get_cell_state() {
        let game_board = GameBoard::new();
        assert_eq!(game_board.get_cell_state(0, 0), Cell::Empty);
    }

    #[test]
    fn test_get_hotel_at() {
        let mut game_board = GameBoard::new();
        game_board.cells[0][0] = Cell::Hotel(Hotel::Luxor);
        assert_eq!(game_board.get_hotel_at(0, 0), Some(Hotel::Luxor));
    }

    #[test]
    fn test_get_active_hotels() {
        let mut game_board = GameBoard::new();
        game_board.cells[0][0] = Cell::Hotel(Hotel::Luxor);
        game_board.cells[0][1] = Cell::Hotel(Hotel::Luxor);

        game_board.cells[2][0] = Cell::Hotel(Hotel::Tower);
        game_board.cells[2][1] = Cell::Hotel(Hotel::Tower);

        let active_hotels = game_board.get_active_hotels();
        let inactive_hotels = game_board.get_inactive_hotels();

        for hotel in vec![Hotel::Luxor, Hotel::Tower] {
            assert_eq!(active_hotels.contains(&hotel), true);
            assert_eq!(inactive_hotels.contains(&hotel), false);
        }

        for hotel in vec![
            Hotel::American,
            Hotel::Worldwide,
            Hotel::Festival,
            Hotel::Imperial,
            Hotel::Continental,
        ] {
            assert_eq!(active_hotels.contains(&hotel), false);
            assert_eq!(inactive_hotels.contains(&hotel), true);
        }
    }

    #[test]
    fn test_would_cell_start_new_chain() {
        let mut game_board = GameBoard::new();
        game_board.cells[0][0] = Cell::Independent;
        assert_eq!(game_board.would_cell_start_new_chain(0, 1), true);
    }

    #[test]
    fn test_get_hotel_chain_size() {
        let mut game_board = GameBoard::new();

        for i in 0..2 {
            game_board.cells[0][i] = Cell::Hotel(Hotel::Luxor);
        }

        for i in 0..4 {
            game_board.cells[2][i] = Cell::Hotel(Hotel::Tower);
        }

        for i in 0..8 {
            game_board.cells[4][i] = Cell::Hotel(Hotel::American);
        }

        assert_eq!(game_board.get_hotel_chain_size(Hotel::Luxor), 2);
        assert_eq!(game_board.get_hotel_chain_size(Hotel::Tower), 4);
        assert_eq!(game_board.get_hotel_chain_size(Hotel::American), 8);
    }

    #[test]
    fn test_get_adjacent_hotels() {
        let mut game_board = GameBoard::new();
        game_board.cells[0][1] = Cell::Hotel(Hotel::Tower);
        game_board.cells[1][0] = Cell::Hotel(Hotel::Festival);
        game_board.cells[1][2] = Cell::Hotel(Hotel::Luxor);

        {
            let adjacent_hotels = game_board.get_adjacent_hotels(0, 0);
            for hotel in vec![Hotel::Tower, Hotel::Festival] {
                assert_eq!(adjacent_hotels.contains(&hotel), true);
            }
        }

        {
            let adjacent_hotels = game_board.get_adjacent_hotels(1, 1);
            for hotel in vec![Hotel::Tower, Hotel::Festival, Hotel::Luxor] {
                assert_eq!(adjacent_hotels.contains(&hotel), true);
            }
        }
    }

    #[test]
    fn test_is_cell_playable() {
        let mut game_board = GameBoard::new();
        assert_eq!(
            game_board.is_cell_playable(BOARD_ROWS, BOARD_COLS),
            Err(CellNotPlayableReason::CellIsOffBoard)
        );
        assert_eq!(game_board.is_cell_playable(0, 0), Ok(true));

        game_board.cells[0][0] = Cell::Hotel(Hotel::Luxor);
        assert_eq!(
            game_board.is_cell_playable(0, 0),
            Err(CellNotPlayableReason::CellIsNotEmpty)
        );

        game_board.cells[0][0] = Cell::Conflict(CellConflictType::NewChain);
        assert_eq!(
            game_board.is_cell_playable(1, 1),
            Err(CellNotPlayableReason::ConflictOnBoard)
        );

        // test if placing this tile would create an 8th chain
        let mut game_board = GameBoard::new();
        for (i, hotel) in [
            Hotel::Luxor,
            Hotel::Tower,
            Hotel::Festival,
            Hotel::American,
            Hotel::Imperial,
        ]
        .iter()
        .enumerate()
        {
            for j in 0..2 {
                game_board.cells[i * 2][j] = Cell::Hotel(*hotel);
            }
        }

        for (i, hotel) in [Hotel::Worldwide, Hotel::Continental].iter().enumerate() {
            for j in 0..2 {
                game_board.cells[i * 2][j + 3] = Cell::Hotel(*hotel);
            }
        }

        game_board.cells[BOARD_ROWS - 1][BOARD_COLS - 1] = Cell::Independent;
        assert_eq!(
            game_board.is_cell_playable(BOARD_ROWS - 2, BOARD_COLS - 1),
            Err(CellNotPlayableReason::HotelsAreAllActive)
        );

        // test if placing a tile would merge safe hotel chains
        let mut game_board = GameBoard::new();
        for i in 0..SAFE_CHAIN_SIZE {
            game_board.cells[0][i] = Cell::Hotel(Hotel::Luxor);
        }

        for i in 0..SAFE_CHAIN_SIZE {
            game_board.cells[2][i] = Cell::Hotel(Hotel::Tower);
        }

        assert_eq!(
            game_board.is_cell_playable(1, 0),
            Err(CellNotPlayableReason::AdjacentHotelsAreSafe)
        );
    }

    #[test]
    fn test_place_tile() {
        // basic
        let mut game_board = GameBoard::new();
        assert_eq!(game_board.place_tile(0, 0), PlaceTileResult::Success);
        assert_eq!(game_board.cells[0][0], Cell::Independent);

        // placing a tile would create a new chain
        assert_eq!(
            game_board.place_tile(0, 1),
            PlaceTileResult::ConflictCreated(CellConflictType::NewChain),
        );

        // placing a tile would start a merger
        let mut game_board = GameBoard::new();

        for (i, hotel) in [Hotel::Worldwide, Hotel::Continental].iter().enumerate() {
            for j in 0..2 {
                game_board.cells[i * 2][j] = Cell::Hotel(*hotel);
            }
        }

        for (i, hotel) in [Hotel::Imperial].iter().enumerate() {
            for j in 0..2 {
                game_board.cells[i * 2 + 1][j + 2] = Cell::Hotel(*hotel);
            }
        }
        assert_eq!(
            game_board.place_tile(1, 1),
            PlaceTileResult::ConflictCreated(CellConflictType::Merge(2))
        );

        // placing a tile causes a chain to grow
        let mut game_board = GameBoard::new();
        game_board.cells[0][0] = Cell::Hotel(Hotel::Luxor);
        game_board.cells[0][2] = Cell::Independent;
        game_board.cells[1][2] = Cell::Independent;
        assert_eq!(game_board.place_tile(0, 1), PlaceTileResult::Success);
        assert_eq!(game_board.get_hotel_chain_size(Hotel::Luxor), 4);
    }
}
