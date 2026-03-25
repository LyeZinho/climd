use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

use crate::app::{ActivePane, App};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .split(frame.area());

    render_sidebar(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
}

fn render_sidebar(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let is_active = app.active_pane == ActivePane::Sidebar;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|f| {
            let name = f
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "???".to_string());
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Files ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_content(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let is_active = app.active_pane == ActivePane::Content;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = app
        .selected_file()
        .and_then(|f| f.file_name())
        .map(|n| format!(" {} ", n.to_string_lossy()))
        .unwrap_or_else(|| " No file selected ".to_string());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    app.content_height = inner.height;

    let content = if app.content_lines.is_empty() {
        vec![Line::from("Select a file to view")]
    } else {
        app.content_lines.clone()
    };

    let total_lines = content.len();
    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(paragraph, area);

    if total_lines > inner.height as usize {
        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(inner.height as usize))
                .position(app.scroll_offset as usize);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}
