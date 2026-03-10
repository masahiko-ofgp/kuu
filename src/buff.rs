use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;


pub struct Buffer {
    pub lines: Vec<String>,
    pub full_text_cache: Option<String>,
    is_dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            full_text_cache: None,
            is_dirty: true,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines: Vec<String> = reader
            .lines()
            .collect::<Result<_, _>>()?;

        if lines.is_empty() {
            lines.push(String::new());
        }

        let initial_cache = lines.join("\n");

        Ok(Self {
            lines,
            full_text_cache: Some(initial_cache),
            is_dirty: false,
        })
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let text = self.as_full_text();

        let mut file = File::create(path)?;

        file.write_all(text.as_bytes())?;

        file.write_all(b"\n")?;

        self.is_dirty = false;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.full_text_cache = None;
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if let Some(line) = self.lines.get_mut(row) {
            line.insert(col, ch);
            self.mark_dirty();
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        let line = &mut self.lines[row];
        let new_line = line.split_off(col);
        self.lines.insert(row + 1, new_line);
        self.mark_dirty();
    }

    pub fn join_lines(&mut self, row: usize) -> Option<usize> {
        if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            let current_line = self.lines.get_mut(row)?;
            let join_point = current_line.len();
            current_line.push_str(&next_line);
            self.mark_dirty();
            Some(join_point)
        } else {
            None
        }
    }

    pub fn insert_empty_line(&mut self, at_row: usize) {
        if at_row < self.lines.len() {
            self.lines.insert(at_row + 1, String::new());
            self.mark_dirty();
        } else {
            self.lines.push(String::new());
            self.mark_dirty();
        }
    }
    
    pub fn insert_line_above(&mut self, row: usize) {
        self.lines.insert(row, String::new());
        self.mark_dirty();
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(row) {
            line.remove(col);
            self.mark_dirty();
        }
    }

    pub fn delete_char_at(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(row) {
            if col < line.len() {
                line.remove(col);
                self.mark_dirty();
            } else if row < self.lines.len() - 1 {
                self.join_lines(row + 1);
                self.mark_dirty();
            }
        }
    }

    pub fn kill_line(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(row) {
            line.truncate(col);
            self.mark_dirty();
        }
    }

    pub fn as_full_text(&mut self) -> &str {
        if self.is_dirty || self.full_text_cache.is_none() {
            self.full_text_cache = Some(self.lines.join("\n"));
            self.is_dirty = false;
        }
        self.full_text_cache.as_ref().unwrap()
    }
}
