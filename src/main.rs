mod app;
mod cli;
mod event;
mod fs;
mod markdown;
mod ui;
mod update;

use color_eyre::Result;
use crossterm::event as crossterm_event;
use crossterm::event::{Event, KeyEventKind};
use ratatui::DefaultTerminal;

use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    if cli::run_cli() {
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;
    let files = fs::discover_md_files(&current_dir)?;
    let mut app = App::new(files, current_dir);

    if app.selected_file_path().is_some() {
        event::load_initial_file(&mut app);
    }

    let terminal = ratatui::init();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Event::Key(key) = crossterm_event::read()? {
            if key.kind == KeyEventKind::Press {
                event::handle_key(app, key);
            }
        }
    }
    Ok(())
}
