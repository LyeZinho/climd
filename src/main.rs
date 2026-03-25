mod app;
mod event;
mod markdown;
mod ui;

use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event as crossterm_event;
use crossterm::event::{Event, KeyEventKind};
use ratatui::DefaultTerminal;

use app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let files = discover_md_files()?;
    let mut app = App::new(files);

    // Load first file if available
    if app.selected_file().is_some() {
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

fn discover_md_files() -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();

    files.sort();
    Ok(files)
}
