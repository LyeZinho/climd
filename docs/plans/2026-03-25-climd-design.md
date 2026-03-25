# climd Design Document

## Overview

Terminal Markdown viewer/navigator in Rust. Opens in current directory, lists `.md` files in a sidebar, renders selected file with styled text in the main pane.

## Architecture

Single-mode TUI app using Ratatui + crossterm. Two-column layout: sidebar (file list with ListState) + content (rendered markdown with Paragraph + ScrollbarState). Markdown parsing via pulldown-cmark event iterator, converted to styled ratatui `Vec<Line>` through a style stack that handles nested inline formatting.

## Tech Stack

- ratatui 0.30, crossterm 0.29, pulldown-cmark 0.13, color-eyre 0.6

## Components

| Component | File | Responsibility |
|-----------|------|---------------|
| App | `src/app.rs` | State: selected file, scroll, active pane, file list |
| MdParser | `src/markdown.rs` | pulldown_cmark events to `Vec<Line<'static>>` |
| UI | `src/ui.rs` | Two-column layout rendering |
| EventHandler | `src/event.rs` | Keyboard navigation |
| Main | `src/main.rs` | Entry point, terminal setup, main loop |

## Markdown Style Mapping

| Element | Style |
|---------|-------|
| H1 | Cyan + Bold |
| H2 | Blue + Bold |
| H3 | Magenta + Bold |
| H4+ | Yellow + Bold |
| Bold | Bold modifier |
| Italic | Italic modifier |
| Inline code | Green |
| Code block | DarkGray bg, Green fg |
| Blockquote | DarkGray + Italic |
| Unordered list | White with bullet prefix |
| Ordered list | White with number prefix |
| Horizontal rule | DarkGray repeated dash |
| Link | Blue + Underline |

## Keyboard Navigation

| Key | Action |
|-----|--------|
| j/Down | Next item / scroll down |
| k/Up | Previous item / scroll up |
| Tab | Switch pane |
| Enter | Open selected file |
| q/Esc | Quit |
| g/Home | Jump to top |
| G/End | Jump to bottom |
| PgUp/PgDn | Scroll by page |
