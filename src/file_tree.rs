use std::fs;
use std::path::PathBuf;


pub struct FileTreeState {
    pub list: Vec<PathBuf>,
    pub selected: usize,
    pub offset: usize,
    pub show: bool,
    pub current_dir: PathBuf,
}

impl FileTreeState {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        Self {
            list: Vec::new(),
            selected: 0,
            offset: 0,
            show: true,
            current_dir,
        }
    }

    pub fn update_file_list(&mut self, path: PathBuf) {
        let target_dir = if let Ok(abs_path) = fs::canonicalize(&path) {
            if abs_path.is_dir() {
                abs_path
            } else {
                abs_path.parent()
                    .unwrap_or(&abs_path)
                    .to_path_buf()
            }
        } else {
            path
        };

        if let Ok(entries) = fs::read_dir(&target_dir) {
            self.current_dir = target_dir;
            self.list.clear();

            if let Some(parent) = self.current_dir.parent() {
                self.list.push(parent.to_path_buf());
            }

            let mut files: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect();

            files.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir != b_is_dir {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    a.cmp(b)
                }
            });

            self.list = files;

            if self.selected >= self.list.len() {
                self.selected = self.list.len().saturating_sub(1);
            }
        }
    }

    pub fn down(&mut self, h: u16) {
        if self.selected < self.list.len().saturating_sub(1) {
            self.selected += 1;
            self.scroll_tree(h);
        }
    }

    pub fn up(&mut self, h: u16) {
        if self.selected > 0 {
            self.selected -= 1;
            self.scroll_tree(h);
        }
    }

    pub fn file_tree_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            self.update_file_list(parent_path);
        }
    }

    pub fn scroll_tree(&mut self, h: u16) {
        let height = h as usize;

        if height == 0 { return; }

        if self.selected < self.offset {
            self.offset = self.selected;
        }

        if self.selected >= self.offset + height {
            self.offset = self.selected - height + 1;
        }
    }
}
