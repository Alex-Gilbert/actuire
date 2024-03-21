use std::{error::Error, io::Result};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
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

use crate::logic::{acquire_game::AcquireGame, game_board, tile::Tile};

struct InnerRects {
    game_board: Rect,
    stocks: Rect,
    available_tiles: Rect,
    cash: Rect,
}

#[derive(typed_builder::TypedBuilder)]
pub struct TuiApp {
    cell_width: u16,
    cell_height: u16,
    acquire_game: AcquireGame,
    exit: bool,
}

impl TuiApp {
    pub fn run(&mut self, terminal: &mut super::tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let inner_rects = self.split_rects(frame.size());
        self.render_game_board(inner_rects.game_board, frame);
        self.render_stocks(inner_rects.stocks, frame);
        self.render_available_tiles(inner_rects.available_tiles, frame);
        self.render_cash(inner_rects.cash, frame);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            event::Event::Key(event) => match (event.code, event.kind) {
                (KeyCode::Char('q'), KeyEventKind::Press) => self.exit = true,
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn split_rects(&self, rect: Rect) -> InnerRects {
        let stock_width = self.cell_width * 5;
        let available_tiles_height = self.cell_height * 2;
        let game_board_width = rect.width - stock_width;
        let game_board_height = rect.height - available_tiles_height;

        let stocks = Rect {
            x: rect.right() - stock_width,
            y: rect.y,
            width: stock_width,
            height: game_board_height,
        };

        let available_tiles = Rect {
            x: rect.x,
            y: rect.bottom() - available_tiles_height,
            width: game_board_width,
            height: available_tiles_height,
        };

        let game_board = Rect {
            x: rect.x,
            y: rect.y,
            width: game_board_width,
            height: game_board_height,
        };

        let cash = Rect {
            x: game_board.right(),
            y: game_board.bottom(),
            width: rect.width - game_board_width,
            height: available_tiles_height,
        };

        InnerRects {
            game_board,
            stocks,
            available_tiles,
            cash,
        }
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
                    .block(self.get_cell_block())
                    .style(self.get_cell_text_style())
                    .alignment(Alignment::Center);
                frame.render_widget(cell_text, *cell_rect);
            }
        }
        inner
    }

    fn get_cell_block(&self) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .border_type(BorderType::Rounded)
    }

    fn get_cell_text_style(&self) -> Style {
        Style::default().fg(Color::White).bg(Color::Black)
    }

    fn render_stocks(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Stocks ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let stocks_text = Text::from(vec![Line::from(vec!["These are stocks".into()])]);

        let inner = block.inner(area).clone();
        frame.render_widget(Paragraph::new(stocks_text).centered().block(block), area);

        inner
    }

    fn render_available_tiles(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Available Tiles ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let tiles_text = Text::from(vec![Line::from(vec!["These are available tiles".into()])]);

        let inner = block.inner(area).clone();
        frame.render_widget(Paragraph::new(tiles_text).centered().block(block), area);

        inner
    }

    fn render_cash(&self, area: Rect, frame: &mut Frame) -> Rect {
        let title = Title::from(" Cash ".bold());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let cash_text = Text::from(vec![Line::from(vec!["$6000".into()])]);

        let inner = block.inner(area).clone();
        frame.render_widget(Paragraph::new(cash_text).centered().block(block), area);

        inner
    }
}
