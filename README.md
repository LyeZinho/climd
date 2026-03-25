# climd

A fast, terminal-based Markdown viewer built with Rust.

## Features

- Browse and view Markdown files in a directory
- Syntax-highlighted rendering (headings, bold, italic, code blocks, lists, blockquotes, links)
- Vim-style keyboard navigation
- Two-panel layout: file list sidebar + rendered content

## Installation

### Linux / macOS

```bash
curl -sL https://raw.githubusercontent.com/LyeZinho/climd/main/install.sh | sh
```

### From Source

```bash
cargo install --git https://github.com/LyeZinho/climd.git
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/LyeZinho/climd/releases)

## Usage

```bash
climd
```

Opens in the current directory, lists all `.md` files in the sidebar.

## Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Next file / Scroll down |
| `k` / `↑` | Previous file / Scroll up |
| `Tab` | Switch between sidebar and content |
| `Enter` | Open selected file |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `q` / `Esc` | Quit |

## Building

```bash
cargo build --release
```

## License

MIT
