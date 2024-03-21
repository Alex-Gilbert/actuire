use std::io::Result;

use structopt::StructOpt;
use visuals::{tui, tui_app::TuiApp};

pub mod logic;
pub mod visuals;

#[derive(Debug, structopt::StructOpt)]
struct Opt {
    /// the number of players
    #[structopt(short = "-p", long, default_value = "3")]
    players: usize,

    /// The width of each cell.
    #[structopt(short = "-w", long, default_value = "6")]
    cell_width: u16,

    /// The height of each cell.
    #[structopt(short = "-H", long, default_value = "3")]
    cell_height: u16,
}

fn main() -> Result<()> {
    let Opt { players, cell_width, cell_height } = Opt::from_args();

    let mut terminal = tui::init()?;
    let mut tui_app = TuiApp::builder()
        .cell_width(cell_width)
        .cell_height(cell_height)
        .acquire_game(logic::acquire_game::AcquireGame::new(players))
        .exit(false)
        .build();

    let app_result = tui_app.run(&mut terminal);
    tui::restore()?;

    app_result
}
