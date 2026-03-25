use std::path::PathBuf;

use ratatui::widgets::{ListState, ScrollbarState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEntry {
    File(String, PathBuf),
    Directory(String, PathBuf),
}

impl FileEntry {
    pub fn name(&self) -> &str {
        match self {
            FileEntry::File(name, _) => name,
            FileEntry::Directory(name, _) => name,
        }
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            FileEntry::File(_, path) => path,
            FileEntry::Directory(_, path) => path,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, FileEntry::Directory(_, _))
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ActivePane {
    #[default]
    Sidebar,
    Content,
}

pub struct App {
    pub running: bool,
    pub active_pane: ActivePane,
    pub files: Vec<FileEntry>,
    pub current_dir: PathBuf,
    pub list_state: ListState,
    pub content_lines: Vec<ratatui::text::Line<'static>>,
    pub scroll_offset: u16,
    pub scrollbar_state: ScrollbarState,
    pub content_height: u16,
}

impl App {
    pub fn new(files: Vec<FileEntry>, current_dir: PathBuf) -> Self {
        let mut list_state = ListState::default();
        if !files.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            running: true,
            active_pane: ActivePane::default(),
            files,
            current_dir,
            list_state,
            content_lines: Vec::new(),
            scroll_offset: 0,
            scrollbar_state: ScrollbarState::default(),
            content_height: 0,
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.list_state.selected().and_then(|i| self.files.get(i))
    }

    pub fn selected_file_path(&self) -> Option<PathBuf> {
        self.selected_entry().and_then(|e| {
            if let FileEntry::File(_, path) = e {
                Some(path.clone())
            } else {
                None
            }
        })
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
