use std::{
    borrow::Borrow,
    error::Error,
    io::Result,
    sync::{Arc, Mutex, MutexGuard},
};

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Paragraph, Widget,
    },
    Frame,
};

use crate::logic::{
    acquire_game::{AcquireGame, AcquireGameCallback},
    acquire_request::AcquireRequest,
    acquire_response::{AcquireResponse, BuyStockChoice, DisposeStockChoice},
    game_board,
    hotel_data::Hotel,
    tile::Tile,
};

struct InnerRects {
    messages: Rect,
    game_board: Rect,
    stocks: Rect,
    prompt: Rect,
    player: Rect,
}

#[derive(Debug, Default)]
struct AcquireMessages {
    messages: Mutex<Vec<String>>,
}

impl AcquireMessages {
    pub fn get_messages(&self) -> MutexGuard<Vec<String>> {
        self.messages.lock().unwrap()
    }
}

impl AcquireGameCallback for AcquireMessages {
    fn send_message(&self, message: &str) {
        self.messages.lock().unwrap().push(message.to_string());
    }
}

pub struct TuiApp {
    cell_width: u16,
    cell_height: u16,
    acquire_messages: Arc<AcquireMessages>,
    acquire_game: AcquireGame<AcquireMessages>,
    error_message_per_player: Vec<String>,
    exit: bool,
    current_player: usize,
}

const HOTEL_COLORS: [Color; 7] = [
    Color::Rgb(201, 128, 6),  // Tower
    Color::Rgb(191, 37, 45),  // Luxor
    Color::Rgb(1, 30, 80),    // American
    Color::Rgb(75, 41, 21),   // Worldwide
    Color::Rgb(0, 82, 60),    // Festival
    Color::Rgb(176, 55, 100), // Imperial
    Color::Rgb(2, 82, 86),    // Continental
];

impl TuiApp {
    pub fn new(cell_width: u16, cell_height: u16, number_of_players: usize) -> Self {
        let acquire_messages = Arc::new(AcquireMessages::default());
        let acquire_game = AcquireGame::new(number_of_players, acquire_messages.clone());

        Self {
            cell_width,
            cell_height,
            acquire_messages: acquire_messages.clone(),
            acquire_game,
            exit: false,
            current_player: 0,
            error_message_per_player: vec![String::new(); number_of_players],
        }
    }

    pub fn run(&mut self, terminal: &mut super::tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let inner_rects = self.split_rects(frame.size());
        if let Some(inner_rects) = inner_rects {
            self.render_game_board(inner_rects.game_board, frame);
            self.render_messages(inner_rects.messages, frame);
            self.render_stocks(inner_rects.stocks, frame);
            self.render_prompt(inner_rects.prompt, frame);
            self.render_player(inner_rects.player, frame);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;

            // Player-specific controls
            let current_request = self.acquire_game.get_current_request();
            match current_request {
                AcquireRequest::PlayStartingTile(player) if *player == self.current_player => {
                    match event {
                        event::Event::Key(event) => match (event.code, event.modifiers, event.kind) {
                            (KeyCode::Char(' '), KeyModifiers::NONE ,KeyEventKind::Press) => {
                                let response = AcquireResponse::StartingTile;
                                let res = self.acquire_game.handle_player_response(response.into());
                                if let Err(e) = res {
                                    self.error_message_per_player[self.current_player] =
                                        e.to_string();
                                } else {
                                    self.error_message_per_player[self.current_player] =
                                        String::new();
                                }
                                return Ok(());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                AcquireRequest::PlayTile(player) if *player == self.current_player => match event {
                    event::Event::Key(event) => match (event.code, event.kind) {
                        (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                            let num = c.to_digit(10).unwrap();
                            if num <= 6 {
                                let tile = self.acquire_game.players[self.current_player]
                                    .tiles
                                    .get(num as usize - 1)
                                    .cloned()
                                    .unwrap();
                                let response = AcquireResponse::Tile(tile);

                                let res = self.acquire_game.handle_player_response(response.into());
                                if let Err(e) = res {
                                    self.error_message_per_player[self.current_player] =
                                        e.to_string();
                                } else {
                                    self.error_message_per_player[self.current_player] =
                                        String::new();
                                }
                                return Ok(());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                AcquireRequest::ChooseNewChain(player) if *player == self.current_player => {
                    match event {
                        event::Event::Key(event) => match (event.code, event.kind) {
                            (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                                let num = c.to_digit(10).unwrap();
                                if num <= Hotel::count() as u32 {
                                    let hotel = Hotel::from(num as usize - 1);
                                    let response = AcquireResponse::NewChain(hotel);

                                    let res =
                                        self.acquire_game.handle_player_response(response.into());
                                    if let Err(e) = res {
                                        self.error_message_per_player[self.current_player] =
                                            e.to_string();
                                    } else {
                                        self.error_message_per_player[self.current_player] =
                                            String::new();
                                    }
                                    return Ok(());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                AcquireRequest::ChooseMergerSurvivor(player) if *player == self.current_player => {
                    match event {
                        event::Event::Key(event) => match (event.code, event.kind) {
                            (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                                let num = c.to_digit(10).unwrap();
                                if num <= Hotel::count() as u32 {
                                    let hotel = Hotel::from(num as usize - 1);
                                    let response = AcquireResponse::MergerSurvivor(hotel);

                                    let res =
                                        self.acquire_game.handle_player_response(response.into());
                                    if let Err(e) = res {
                                        self.error_message_per_player[self.current_player] =
                                            e.to_string();
                                    } else {
                                        self.error_message_per_player[self.current_player] =
                                            String::new();
                                    }
                                    return Ok(());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                AcquireRequest::ChooseDefunctChainToResolve(player)
                    if *player == self.current_player =>
                {
                    match event {
                        event::Event::Key(event) => match (event.code, event.kind) {
                            (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                                let num = c.to_digit(10).unwrap();
                                if num <= Hotel::count() as u32 {
                                    let hotel = Hotel::from(num as usize - 1);
                                    let response = AcquireResponse::DefunctChainToResolve(hotel);

                                    let res =
                                        self.acquire_game.handle_player_response(response.into());
                                    if let Err(e) = res {
                                        self.error_message_per_player[self.current_player] =
                                            e.to_string();
                                    } else {
                                        self.error_message_per_player[self.current_player] =
                                            String::new();
                                    }
                                    return Ok(());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                AcquireRequest::DisposeStock => match event {
                    event::Event::Key(event) => match (event.code, event.kind) {
                        (KeyCode::Char(c), KeyEventKind::Press) => {
                            let choice = match c {
                                'k' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::Keep,
                                ),
                                's' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::Sell,
                                ),
                                't' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::Trade,
                                ),
                                'K' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::KeepAll,
                                ),
                                'S' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::SellAll,
                                ),
                                'T' => AcquireResponse::DisposeStock(
                                    self.current_player,
                                    DisposeStockChoice::TradeAll,
                                ),
                                _ => return Ok(()),
                            };

                            let res = self.acquire_game.handle_player_response(choice);
                            if let Err(e) = res {
                                self.error_message_per_player[self.current_player] = e.to_string();
                            } else {
                                self.error_message_per_player[self.current_player] = String::new();
                            }

                            return Ok(());
                        }
                        _ => {}
                    },
                    _ => {}
                },
                AcquireRequest::BuyStock(player) if *player == self.current_player => match event {
                    event::Event::Key(event) => match (event.code, event.kind) {
                        (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                            let num = c.to_digit(10).unwrap();
                            if num <= Hotel::count() as u32 {
                                let hotel = Hotel::from(num as usize - 1);

                                let response = AcquireResponse::BuyStock(
                                    BuyStockChoice::Buy(hotel),
                                );

                                let res = self.acquire_game.handle_player_response(response.into());
                                if let Err(e) = res {
                                    self.error_message_per_player[self.current_player] =
                                        e.to_string();
                                } else {
                                    self.error_message_per_player[self.current_player] =
                                        String::new();
                                }
                                return Ok(());
                            }
                        },
                        (KeyCode::Char('S'), KeyEventKind::Press) => {
                            let response = AcquireResponse::BuyStock(BuyStockChoice::Pass);
                            let res = self.acquire_game.handle_player_response(response.into());
                            if let Err(e) = res {
                                self.error_message_per_player[self.current_player] = e.to_string();
                            } else {
                                self.error_message_per_player[self.current_player] = String::new();
                            }
                            return Ok(());
                        }
                        _ => {}
                    },
                    _ => {}
                },
                AcquireRequest::EndGame(player) if *player == self.current_player => {}

                _ => {}
            }

            // Global controls
            match event {
                event::Event::Key(event) => match (event.code, event.kind) {
                    (KeyCode::Char(c), KeyEventKind::Press) if c.is_digit(10) => {
                        let num = c.to_digit(10).unwrap();
                        if num <= self.acquire_game.players.len() as u32 {
                            self.current_player = num as usize - 1;
                        }
                    }
                    (KeyCode::Char('q'), KeyEventKind::Press) => self.exit = true,
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn split_rects(&self, rect: Rect) -> Option<InnerRects> {
        let needed_width = self.cell_width * game_board::BOARD_COLS as u16
            + self.cell_width * 10
            + self.cell_width * 10;
        let needed_height = self.cell_height * game_board::BOARD_ROWS as u16 + self.cell_height * 6;

        if rect.width < needed_width || rect.height < needed_height {
            return None;
        }

        let stock_width = self.cell_width * 10;
        let message_width = stock_width;

        let player_height = self.cell_height * 4;
        let prompt_height = self.cell_height * 2;

        let game_board_width = rect.width - stock_width - message_width;
        let game_board_height = rect.height - player_height - prompt_height;

        // messages are on the left and take up the full height
        let messages = Rect {
            x: rect.x,
            y: rect.y,
            width: message_width,
            height: rect.height,
        };

        // the game board is in the middle right of the messages
        let game_board = Rect {
            x: rect.x + message_width,
            y: rect.y,
            width: game_board_width,
            height: game_board_height,
        };

        // below the game board is the prompt box
        let prompt = Rect {
            x: game_board.x,
            y: game_board.bottom(),
            width: game_board_width,
            height: prompt_height,
        };

        // below the prompt is the player box
        let player = Rect {
            x: game_board.x,
            y: prompt.bottom(),
            width: game_board_width,
            height: player_height,
        };

        // to the right of the game board are the stocks info panel
        let stocks = Rect {
            x: game_board.right(),
            y: rect.y,
            width: stock_width,
            height: game_board_height,
        };

        Some(InnerRects {
            game_board,
            messages,
            stocks,
            prompt,
            player,
        })
    }

    fn render_game_board(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" acTUIre ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let inner = block.inner(area).clone();
        frame.render_widget(block, area);

        let cell_width = self.cell_width;
        let cell_height = self.cell_height;

        let padding = 1;

        let grid_width =
            u16::try_from(cell_width * game_board::BOARD_COLS as u16 + 2 * padding).unwrap();
        let grid_height =
            u16::try_from(cell_height * game_board::BOARD_ROWS as u16 + 2 * padding).unwrap();

        let row_constraints =
            std::iter::repeat(Constraint::Length(u16::try_from(cell_height).unwrap()))
                .take(game_board::BOARD_ROWS)
                .collect::<Vec<_>>();

        let col_constraints =
            std::iter::repeat(Constraint::Length(u16::try_from(cell_width).unwrap()))
                .take(game_board::BOARD_COLS)
                .collect::<Vec<_>>();

        let (center_x, center_y) = (inner.x + inner.width / 2, inner.y + inner.height / 2);
        let board_rect = Rect {
            x: center_x - grid_width / 2,
            y: center_y - grid_height / 2,
            width: grid_width,
            height: grid_height,
        };

        let board_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        frame.render_widget(board_block, board_rect);

        let row_rects = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(1)
            .horizontal_margin(0)
            .constraints(row_constraints.clone())
            .split(board_rect);

        for (r, row_rect) in row_rects.iter().enumerate() {
            let col_rects = Layout::default()
                .direction(Direction::Horizontal)
                .vertical_margin(0)
                .horizontal_margin(1)
                .constraints(col_constraints.clone())
                .split(*row_rect);

            for (c, cell_rect) in col_rects.iter().enumerate() {
                let tile: Tile = (r, c).into();
                let single_row_text = format!(
                    "{:^length$}",
                    tile.to_string(),
                    length = (cell_width - 2).into()
                );

                let pad_line = " ".repeat(cell_width.into());

                // 1 line for the text, 1 line each for the top and bottom of the cell == 3 lines
                // that are not eligible for padding
                let num_pad_lines = (cell_height as usize) - 3;

                // text is:
                //   pad with half the pad lines budget
                //   the interesting text
                //   pad with half the pad lines budget
                //   join with newlines
                let text = std::iter::repeat(pad_line.clone())
                    .take(num_pad_lines / 2)
                    .chain(std::iter::once(single_row_text))
                    .chain(std::iter::repeat(pad_line).take(num_pad_lines / 2))
                    .collect::<Vec<_>>()
                    .join("\n");

                let cell_text = Paragraph::new(text)
                    .block(self.get_cell_block(r, c))
                    .style(self.get_cell_text_style(r, c))
                    .alignment(Alignment::Center);
                frame.render_widget(cell_text, *cell_rect);
            }
        }
        inner
    }

    fn get_cell_block(&self, row: usize, col: usize) -> Block {
        let cell_state = self.acquire_game.board.get_cell_state(row, col);

        let (bg_color, fg_color) = match cell_state {
            game_board::Cell::Empty => (Color::Black, Color::White),
            game_board::Cell::Hotel(hotel) => (HOTEL_COLORS[hotel as usize], Color::Black),
            game_board::Cell::Independent => (Color::White, Color::Black),
            game_board::Cell::Conflict(_) => (Color::Gray, Color::Black),
        };

        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(bg_color).fg(fg_color))
            .border_type(BorderType::Rounded)
    }

    fn get_cell_text_style(&self, row: usize, col: usize) -> Style {
        let cell_state = self.acquire_game.board.get_cell_state(row, col);

        let (bg_color, fg_color) = match cell_state {
            game_board::Cell::Empty => (Color::Black, Color::White),
            game_board::Cell::Hotel(hotel) => (HOTEL_COLORS[hotel as usize], Color::Black),
            game_board::Cell::Independent => (Color::White, Color::Black),
            game_board::Cell::Conflict(_) => (Color::Gray, Color::Black),
        };

        Style::default().bg(bg_color).fg(fg_color)
    }

    fn render_stocks(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Stocks ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);
        let inner = block.inner(area).clone();

        let stock_rect_height = inner.height / 7;

        for hotel in Hotel::iter() {
            let color = HOTEL_COLORS[hotel as usize];
            let hotel_name = format!("{:?}", hotel);
            let stock_count = self.acquire_game.get_current_stock_availability(hotel);
            let current_price = self.acquire_game.get_current_stock_price(hotel);
            let chain_size = self.acquire_game.get_current_chain_size(hotel);

            let stock_rect = Rect {
                x: inner.x,
                y: inner.y + (stock_rect_height * hotel as u16),
                width: inner.width,
                height: stock_rect_height,
            };

            let stock_block = Block::default()
                .title(hotel_name.bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(color).bg(Color::Black));
            let stock_text = Text::from(vec![
                Line::from(vec![Span::styled(
                    format!("Available Stock: {}", stock_count),
                    Style::default().fg(Color::White),
                )]),
                Line::from(vec![Span::styled(
                    format!("Price: ${}.00", current_price),
                    Style::default().fg(Color::White),
                )]),
                Line::from(vec![Span::styled(
                    format!("Chain Size: {}", chain_size),
                    Style::default().fg(Color::White),
                )]),
            ]);

            frame.render_widget(Paragraph::new(stock_text).block(stock_block), stock_rect);
        }

        inner
    }

    fn render_messages(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Messages ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area).clone();
        frame.render_widget(block, area);

        let messages = self.acquire_messages.get_messages();
        for (i, message) in messages.iter().rev().enumerate() {
            if i >= inner_area.height as usize {
                break;
            }
            let message_text = Text::from(vec![Line::from(vec![message.into()])]);
            let message_rect = Rect {
                x: inner_area.x,
                y: inner_area.bottom() - 1 - i as u16,
                width: inner_area.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(message_text), message_rect);
        }

        inner_area
    }

    fn render_player(&self, area: Rect, frame: &mut Frame) -> Rect {
        let player_name = self.acquire_game.players[self.current_player].name.clone();

        let title = Title::from(player_name.bold());
        let tiles_title = Title::from(" Tiles ".bold());
        let cash_title = Title::from(" Cash ".bold());

        let parent_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let inner = parent_block.inner(area).clone();

        let cash_width = inner.width / 3;
        let half_height = inner.height / 2;

        let tiles_rect = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width - cash_width,
            height: half_height,
        };

        let cash_rect = Rect {
            x: tiles_rect.right(),
            y: inner.y,
            width: cash_width,
            height: half_height,
        };

        let holdings_rect = Rect {
            x: inner.x,
            y: tiles_rect.bottom(),
            width: inner.width,
            height: half_height,
        };

        let tiles_block = Block::default()
            .title(tiles_title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let cash_block = Block::default()
            .title(cash_title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        frame.render_widget(parent_block, area);

        // render the cash
        let cash = self.acquire_game.players[self.current_player].cash;
        let cash_text = Text::from(vec![Line::from(vec![format!("${}.00", cash).into()])]);
        frame.render_widget(
            Paragraph::new(cash_text).centered().block(cash_block),
            cash_rect,
        );

        // render the tiles
        let tiles = self.acquire_game.players[self.current_player].tiles.clone();

        let col_constraints =
            std::iter::repeat(Constraint::Length(u16::try_from(self.cell_width).unwrap()))
                .take(tiles.len() as usize)
                .collect::<Vec<_>>();

        let col_rects = Layout::default()
            .direction(Direction::Horizontal)
            .vertical_margin(0)
            .horizontal_margin(1)
            .constraints(col_constraints.clone())
            .split(tiles_rect);

        for (i, tile_rect) in col_rects.iter().enumerate() {
            let tile: Tile = tiles[i];
            let single_row_text = format!(
                "{:^length$}",
                tile.to_string(),
                length = (self.cell_width - 2).into()
            );

            let pad_line = " ".repeat(self.cell_width.into());

            // 1 line for the text, 1 line each for the top and bottom of the cell == 3 lines
            // that are not eligible for padding
            let num_pad_lines = (tile_rect.height as usize) - 3;

            // text is:
            //   pad with half the pad lines budget
            //   the interesting text
            //   pad with half the pad lines budget
            //   join with newlines
            let text = std::iter::repeat(pad_line.clone())
                .take(num_pad_lines / 2)
                .chain(std::iter::once(single_row_text))
                .chain(std::iter::repeat(pad_line).take(num_pad_lines / 2))
                .collect::<Vec<_>>()
                .join("\n");

            let cell_text = Paragraph::new(text)
                .block(self.get_cell_block(0, 0))
                .style(self.get_cell_text_style(0, 0))
                .alignment(Alignment::Center);
            frame.render_widget(cell_text, *tile_rect);
        }

        // render the holdings
        let stock_rect_width = holdings_rect.width / 7;
        for hotel in Hotel::iter() {
            let hotel_name = format!("{:?}", hotel);
            let color = HOTEL_COLORS[hotel as usize];
            let num_shares = self.acquire_game.players[self.current_player].stocks[hotel as usize];

            let hotel_rect = Rect {
                x: holdings_rect.x + (stock_rect_width * hotel as u16),
                y: holdings_rect.y,
                width: stock_rect_width,
                height: holdings_rect.height,
            };

            let stock_block = Block::default()
                .title(hotel_name.bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(color));

            let stock_text = Text::from(vec![
                Line::from(vec![Span::styled(
                    "".to_string(),
                    Style::default().fg(Color::White),
                )]),
                Line::from(vec![Span::styled(
                    format!("{}", num_shares),
                    Style::default().fg(Color::White),
                )]),
            ]);

            frame.render_widget(
                Paragraph::new(stock_text).centered().block(stock_block),
                hotel_rect,
            );
        }

        inner
    }

    fn render_prompt(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Prompt ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let current_acquire_request = self.acquire_game.get_current_request();

        let prompt_text = match current_acquire_request {
            AcquireRequest::PlayStartingTile(player) if *player == self.current_player => {
                Text::from(vec![
                    Line::from(vec!["It's your turn!".into()]),
                    Line::from(vec!["Press <SPACE> to place your starting tile.".into()]),
                    Line::from(vec![self.error_message_per_player[self.current_player]
                        .clone()
                        .into()]),
                ])
            }
            AcquireRequest::PlayTile(player) if *player == self.current_player => Text::from(vec![
                Line::from(vec!["It's your turn!".into()]),
                Line::from(vec!["Choose a tile 1-6 to play".into()]),
                Line::from(vec![self.error_message_per_player[self.current_player]
                    .clone()
                    .into()]),
            ]),
            AcquireRequest::ChooseNewChain(player) if *player == self.current_player => {
                Text::from(vec![
                    Line::from(vec!["You started a new chain!".into()]),
                    Line::from(vec!["Press 1-7 to choose a hotel".into()]),
                    Line::from(vec![self.error_message_per_player[self.current_player]
                        .clone()
                        .into()]),
                ])
            }
            AcquireRequest::ChooseMergerSurvivor(player) if *player == self.current_player => {
                Text::from(vec![
                    Line::from(vec!["Choose a surviving chain!".into()]),
                    Line::from(vec!["Press 1-7 to choose a hotel".into()]),
                    Line::from(vec![self.error_message_per_player[self.current_player]
                        .clone()
                        .into()]),
                ])
            }
            AcquireRequest::ChooseDefunctChainToResolve(player)
                if *player == self.current_player =>
            {
                Text::from(vec![
                    Line::from(vec!["Choose a defunct chain to resolve".into()]),
                    Line::from(vec!["Press 1-7 to choose a hotel".into()]),
                    Line::from(vec![self.error_message_per_player[self.current_player]
                        .clone()
                        .into()]),
                ])
            }
            AcquireRequest::DisposeStock => Text::from(vec![
                Line::from(vec!["Dispose of stock".into()]),
                Line::from(vec!["Press 1-7 to choose a hotel".into()]),
                Line::from(vec![self.error_message_per_player[self.current_player]
                    .clone()
                    .into()]),
            ]),
            AcquireRequest::BuyStock(player) if *player == self.current_player => Text::from(vec![
                Line::from(vec!["Buy stock".into()]),
                Line::from(vec!["Press 1-7 to choose a hotel".into()]),
                Line::from(vec![self.error_message_per_player[self.current_player]
                    .clone()
                    .into()]),
            ]),
            AcquireRequest::EndGame(player) if *player == self.current_player => Text::from(vec![
                Line::from(vec!["End the game?".into()]),
                Line::from(vec!["Press <SPACE> to end the game, <ESC> to cancel".into()]),
                Line::from(vec![self.error_message_per_player[self.current_player]
                    .clone()
                    .into()]),
            ]),

            AcquireRequest::PlayStartingTile(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to play their starting tile".into(),
            ])]),

            AcquireRequest::PlayTile(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to play a tile".into(),
            ])]),

            AcquireRequest::ChooseNewChain(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to choose a new chain".into(),
            ])]),

            AcquireRequest::ChooseMergerSurvivor(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to choose a surviving chain".into(),
            ])]),

            AcquireRequest::ChooseDefunctChainToResolve(player) => {
                Text::from(vec![Line::from(vec![
                    "Waiting for ".into(),
                    self.acquire_game.players[*player].name.clone().into(),
                    " to choose a defunct chain to resolve".into(),
                ])])
            }

            AcquireRequest::BuyStock(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to buy stock".into(),
            ])]),

            AcquireRequest::EndGame(player) => Text::from(vec![Line::from(vec![
                "Waiting for ".into(),
                self.acquire_game.players[*player].name.clone().into(),
                " to decide wether or not to end the game".into(),
            ])]),
        };

        let inner = block.inner(area).clone();
        frame.render_widget(Paragraph::new(prompt_text).centered().block(block), area);

        inner
    }
}
