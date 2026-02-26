#[derive(Debug, PartialEq)]
pub enum AppMode {
    Normal,
    Insert,
    Quit,
}

pub struct App {
    pub mode: AppMode,
    pub content: String, // 最初はStringで、後にGap Bufferへ
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            content: String::new(),
        }
    }
}
