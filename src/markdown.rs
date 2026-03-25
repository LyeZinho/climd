use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
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
                self.push_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                );
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
                self.current_spans
                    .push(Span::styled(bullet, self.current_style()));
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
                self.push_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                );
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
                    self.lines
                        .push(Line::from(Span::styled(format!(" {} ", line), style)));
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
        self.current_spans
            .push(Span::styled(text, self.current_style()));
    }

    fn inline_code(&mut self, code: String) {
        let style = Style::default().fg(Color::Green);
        self.current_spans
            .push(Span::styled(format!("`{}`", code), style));
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
        HeadingLevel::H1 => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H2 => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        HeadingLevel::H3 => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
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
