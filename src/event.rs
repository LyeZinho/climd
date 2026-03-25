use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{ActivePane, App};
use crate::markdown;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.running = false,
        KeyCode::Tab => toggle_pane(app),
        KeyCode::Enter => open_selected(app),
        KeyCode::Char('j') | KeyCode::Down => move_down(app),
        KeyCode::Char('k') | KeyCode::Up => move_up(app),
        KeyCode::Char('g') | KeyCode::Home => move_to_top(app),
        KeyCode::Char('G') | KeyCode::End => move_to_bottom(app),
        KeyCode::PageDown => page_down(app),
        KeyCode::PageUp => page_up(app),
        _ => {}
    }
}

fn toggle_pane(app: &mut App) {
    app.active_pane = match app.active_pane {
        ActivePane::Sidebar => ActivePane::Content,
        ActivePane::Content => ActivePane::Sidebar,
    };
}

fn open_selected(app: &mut App) {
    if app.active_pane != ActivePane::Sidebar {
        return;
    }
    load_selected_file(app);
    app.active_pane = ActivePane::Content;
}

fn move_down(app: &mut App) {
    match app.active_pane {
        ActivePane::Sidebar => {
            app.list_state.select_next();
            load_selected_file(app);
        }
        ActivePane::Content => app.scroll_down(1),
    }
}

fn move_up(app: &mut App) {
    match app.active_pane {
        ActivePane::Sidebar => {
            app.list_state.select_previous();
            load_selected_file(app);
        }
        ActivePane::Content => app.scroll_up(1),
    }
}

fn move_to_top(app: &mut App) {
    match app.active_pane {
        ActivePane::Sidebar => {
            app.list_state.select_first();
            load_selected_file(app);
        }
        ActivePane::Content => app.scroll_to_top(),
    }
}

fn move_to_bottom(app: &mut App) {
    match app.active_pane {
        ActivePane::Sidebar => {
            app.list_state.select_last();
            load_selected_file(app);
        }
        ActivePane::Content => app.scroll_to_bottom(),
    }
}

fn page_down(app: &mut App) {
    if app.active_pane == ActivePane::Content {
        app.scroll_down(app.content_height.saturating_sub(2));
    }
}

fn page_up(app: &mut App) {
    if app.active_pane == ActivePane::Content {
        app.scroll_up(app.content_height.saturating_sub(2));
    }
}

fn load_selected_file(app: &mut App) {
    if let Some(path) = app.selected_file().cloned() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            app.content_lines = markdown::parse_markdown(&content);
            app.scroll_offset = 0;
            app.scrollbar_state = app.scrollbar_state.position(0);
        }
    }
}

pub fn load_initial_file(app: &mut App) {
    load_selected_file(app);
}
