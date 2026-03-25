# climd Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal Markdown viewer that lists `.md` files in a sidebar and renders the selected file with styled text.

**Architecture:** Two-column TUI (ratatui + crossterm). pulldown-cmark parses markdown into events, a style-stack renderer converts them to `Vec<Line<'static>>`. App state tracks file list, selection, scroll position, and active pane.

**Tech Stack:** Rust, ratatui 0.30, crossterm 0.29, pulldown-cmark 0.13, color-eyre 0.6

**Design Doc:** `docs/plans/2026-03-25-climd-design.md`

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Initialize cargo project**

Run: `cargo init --name climd`
Expected: Creates `Cargo.toml` and `src/main.rs`

**Step 2: Add dependencies to Cargo.toml**

Replace `[dependencies]` section with:

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.29"
pulldown-cmark = "0.13"
color-eyre = "0.6"
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: scaffold climd project with dependencies"
```

---

### Task 2: App State (`src/app.rs`)

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs` (add `mod app;`)

**Step 1: Create app.rs with App struct**

```rust
use std::path::PathBuf;

use ratatui::widgets::{ListState, ScrollbarState};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ActivePane {
    #[default]
    Sidebar,
    Content,
}

pub struct App {
    pub running: bool,
    pub active_pane: ActivePane,
    pub files: Vec<PathBuf>,
    pub list_state: ListState,
    pub content_lines: Vec<ratatui::text::Line<'static>>,
    pub scroll_offset: u16,
    pub scrollbar_state: ScrollbarState,
    pub content_height: u16,
}

impl App {
    pub fn new(files: Vec<PathBuf>) -> Self {
        let mut list_state = ListState::default();
        if !files.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            running: true,
            active_pane: ActivePane::default(),
            files,
            list_state,
            content_lines: Vec::new(),
            scroll_offset: 0,
            scrollbar_state: ScrollbarState::default(),
            content_height: 0,
        }
    }

    pub fn selected_file(&self) -> Option<&PathBuf> {
        self.list_state.selected().and_then(|i| self.files.get(i))
    }

    pub fn scroll_down(&mut self, amount: u16) {
        let max = (self.content_lines.len() as u16).saturating_sub(self.content_height);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll_offset as usize);
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll_offset as usize);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.scrollbar_state = self.scrollbar_state.position(0);
    }

    pub fn scroll_to_bottom(&mut self) {
        let max = (self.content_lines.len() as u16).saturating_sub(self.content_height);
        self.scroll_offset = max;
        self.scrollbar_state = self.scrollbar_state.position(max as usize);
    }
}
```

**Step 2: Register module in main.rs**

Add `mod app;` at the top of `src/main.rs`.

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add App state with pane switching and scroll management"
```

---

### Task 3: Markdown Parser (`src/markdown.rs`)

**Files:**
- Create: `src/markdown.rs`
- Modify: `src/main.rs` (add `mod markdown;`)

**Step 1: Create markdown.rs with parse function**

This is the core module. It uses a style stack to handle nested inline formatting (e.g., bold inside a heading).

```rust
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

struct MdRenderer {
    lines: Vec<Line<'static>>,
    current_spans: Vec<Span<'static>>,
    style_stack: Vec<Style>,
    in_code_block: bool,
    code_block_buf: String,
    list_depth: usize,
    ordered_index: Option<u64>,
    in_heading: bool,
    heading_level: Option<HeadingLevel>,
}

impl MdRenderer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_spans: Vec::new(),
            style_stack: Vec::new(),
            in_code_block: false,
            code_block_buf: String::new(),
            list_depth: 0,
            ordered_index: None,
            in_heading: false,
            heading_level: None,
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, style: Style) {
        let merged = self.current_style().patch(style);
        self.style_stack.push(merged);
    }

    fn pop_style(&mut self) {
        self.style_stack.pop();
    }

    fn flush_line(&mut self) {
        if !self.current_spans.is_empty() {
            let spans = std::mem::take(&mut self.current_spans);
            self.lines.push(Line::from(spans));
        }
    }

    fn push_empty_line(&mut self) {
        self.flush_line();
        self.lines.push(Line::from(""));
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text(text.to_string()),
            Event::Code(code) => self.inline_code(code.to_string()),
            Event::SoftBreak | Event::HardBreak => self.flush_line(),
            Event::Rule => self.horizontal_rule(),
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.push_empty_line();
                self.in_heading = true;
                self.heading_level = Some(level);
                let style = heading_style(level);
                let prefix = format!("{} ", "#".repeat(heading_level_num(level)));
                self.current_spans.push(Span::styled(prefix, style));
                self.push_style(style);
            }
            Tag::Paragraph => {
                if !self.lines.is_empty() {
                    self.push_empty_line();
                }
            }
            Tag::CodeBlock(_kind) => {
                self.push_empty_line();
                self.in_code_block = true;
                self.code_block_buf.clear();
            }
            Tag::BlockQuote(_) => {
                self.push_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
            }
            Tag::List(ordered) => {
                if self.list_depth == 0 && !self.lines.is_empty() {
                    self.push_empty_line();
                }
                self.list_depth += 1;
                self.ordered_index = ordered;
            }
            Tag::Item => {
                self.flush_line();
                let indent = "  ".repeat(self.list_depth.saturating_sub(1));
                let bullet = if let Some(ref mut idx) = self.ordered_index {
                    let s = format!("{}{}. ", indent, idx);
                    *idx += 1;
                    s
                } else {
                    format!("{}\u{2022} ", indent)
                };
                self.current_spans.push(Span::styled(bullet, self.current_style()));
            }
            Tag::Emphasis => {
                self.push_style(Style::default().add_modifier(Modifier::ITALIC));
            }
            Tag::Strong => {
                self.push_style(Style::default().add_modifier(Modifier::BOLD));
            }
            Tag::Strikethrough => {
                self.push_style(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            Tag::Link { dest_url, .. } => {
                self.push_style(Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED));
                // Store URL for potential future use; for now just style the text
                let _ = dest_url;
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                self.pop_style();
                self.flush_line();
                self.in_heading = false;
                self.heading_level = None;
            }
            TagEnd::Paragraph => {
                self.flush_line();
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                let code = std::mem::take(&mut self.code_block_buf);
                let style = Style::default().fg(Color::Green).bg(Color::DarkGray);
                for line in code.lines() {
                    self.lines.push(Line::from(Span::styled(
                        format!(" {} ", line),
                        style,
                    )));
                }
                self.push_empty_line();
            }
            TagEnd::BlockQuote(_) => {
                self.pop_style();
            }
            TagEnd::List(_) => {
                self.list_depth = self.list_depth.saturating_sub(1);
                if self.list_depth == 0 {
                    self.ordered_index = None;
                }
            }
            TagEnd::Item => {
                self.flush_line();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.pop_style();
            }
            _ => {}
        }
    }

    fn text(&mut self, text: String) {
        if self.in_code_block {
            self.code_block_buf.push_str(&text);
            return;
        }
        self.current_spans.push(Span::styled(text, self.current_style()));
    }

    fn inline_code(&mut self, code: String) {
        let style = Style::default().fg(Color::Green);
        self.current_spans.push(Span::styled(format!("`{}`", code), style));
    }

    fn horizontal_rule(&mut self) {
        self.flush_line();
        self.lines.push(Line::from(Span::styled(
            "\u{2500}".repeat(40),
            Style::default().fg(Color::DarkGray),
        )));
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        HeadingLevel::H2 => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        HeadingLevel::H3 => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    }
}

fn heading_level_num(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub fn parse_markdown(input: &str) -> Vec<Line<'static>> {
    let parser = Parser::new_ext(input, Options::all());
    let mut renderer = MdRenderer::new();
    for event in parser {
        renderer.handle_event(event);
    }
    renderer.flush_line();
    renderer.lines
}
```

**Step 2: Register module in main.rs**

Add `mod markdown;` to `src/main.rs`.

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors (there may be unused warnings, that's fine)

**Step 4: Commit**

```bash
git add src/markdown.rs src/main.rs
git commit -m "feat: add markdown parser with style stack for nested formatting"
```

---

### Task 4: Event Handler (`src/event.rs`)

**Files:**
- Create: `src/event.rs`
- Modify: `src/main.rs` (add `mod event;`)

**Step 1: Create event.rs with handle_key function**

```rust
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
```

**Step 2: Register module in main.rs**

Add `mod event;` to `src/main.rs`.

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/event.rs src/main.rs
git commit -m "feat: add keyboard event handler with vim-style navigation"
```

---

### Task 5: UI Rendering (`src/ui.rs`)

**Files:**
- Create: `src/ui.rs`
- Modify: `src/main.rs` (add `mod ui;`)

**Step 1: Create ui.rs with render function**

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
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
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
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
        vec![Line::from("Select a file to view".dark_gray())]
    } else {
        app.content_lines.clone()
    };

    let total_lines = content.len();
    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    if total_lines > inner.height as usize {
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(inner.height as usize))
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
```

**Step 2: Register module in main.rs**

Add `mod ui;` to `src/main.rs`.

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/ui.rs src/main.rs
git commit -m "feat: add two-column UI with sidebar, content pane, and scrollbar"
```

---

### Task 6: Main Entry Point (`src/main.rs`)

**Files:**
- Modify: `src/main.rs`

**Step 1: Write the complete main.rs**

```rust
mod app;
mod event;
mod markdown;
mod ui;

use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
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

        if let Event::Key(key) = event::read()? {
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
        .filter(|path| {
            path.extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    files.sort();
    Ok(files)
}
```

Note: We need a small helper in `event.rs` to expose `load_initial_file`. Add this public function to `src/event.rs`:

```rust
pub fn load_initial_file(app: &mut App) {
    load_selected_file(app);
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 3: Verify it runs**

Run: `cargo run`
Expected: TUI opens showing README.md in sidebar, renders its content. Press `q` to quit.

**Step 4: Commit**

```bash
git add src/main.rs src/event.rs
git commit -m "feat: wire up main loop with file discovery and TUI rendering"
```

---

### Task 7: Build & Smoke Test

**Step 1: Build release binary**

Run: `cargo build --release`
Expected: Compiles successfully, binary at `target/release/climd.exe`

**Step 2: Create a test markdown file**

Create `test.md` with various markdown elements (headings, bold, italic, code blocks, lists, links, horizontal rules) to verify rendering.

**Step 3: Run and verify**

Run: `cargo run`
Expected:
- Sidebar shows `README.md` and `test.md`
- Arrow keys / j/k navigate sidebar
- Tab switches to content pane
- j/k scrolls content
- q quits
- All markdown elements render with correct styles

**Step 4: Clean up test file and final commit**

Remove `test.md`, then:

```bash
git add -A
git commit -m "feat: climd v0.1 - terminal markdown viewer"
```
