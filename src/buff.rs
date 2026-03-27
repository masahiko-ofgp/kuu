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
        let byte_idx = self.char_to_byte_idx(row, col);

        if let Some(line) = self.lines.get_mut(row) {
            line.insert(byte_idx, ch);
            self.mark_dirty();
        }
    }

    pub fn split_line(&mut self, row: usize, col: usize) {
        let byte_idx = self.char_to_byte_idx(row, col);

        if let Some(line) = self.lines.get_mut(row) {
            let new_part = line.split_off(byte_idx);
            self.lines.insert(row + 1, new_part);
            self.mark_dirty();
        }
    }

    pub fn truncate_line(&mut self, row: usize, col: usize) -> String
    {
        let byte_idx = self.char_to_byte_idx(row, col);

        if let Some(line) = self.lines.get_mut(row) {
            let tail = line.split_off(byte_idx);
            self.mark_dirty();
            tail
        } else {
            String::new()
        }
    }

    pub fn join_lines(&mut self, row: usize) -> Option<usize> {
        if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            let current_line = self.lines.get_mut(row)?;
            let joint_point = current_line.chars().count();
            current_line.push_str(&next_line);
            self.mark_dirty();
            Some(joint_point)
        } else {
            None
        }
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        let byte_idx = self.char_to_byte_idx(row, col);

        if let Some(line) = self.lines.get_mut(row) {
            if byte_idx < line.len() {
                line.remove(byte_idx);
                self.mark_dirty();
            }
        }
    }

    pub fn delete_line(&mut self, y: usize) -> usize {
        if self.lines.len() > 1 {
            self.lines.remove(y);
        } else {
            self.lines[0].clear();
        }
        self.mark_dirty();
        self.lines.len()
    }

    pub fn insert_line_at(&mut self, row: usize, text: String) {
        if row <= self.lines.len() {
            self.lines.insert(row, text);
        } else {
            self.lines.push(text);
        }
        self.mark_dirty();
    }

    pub fn prepend_to_line(&mut self, row: usize, text: &str) {
        if let Some(line) = self.lines.get_mut(row) {
            line.insert_str(0, text);
            self.mark_dirty();
        }
    }

    pub fn remove_leading_spaces(&mut self, row: usize, n: usize) -> usize
    {
        if let Some(line) = self.lines.get_mut(row) {
            let mut space_count = 0;

            for c in line.chars() {
                if c == ' ' && space_count < n {
                    space_count += 1;
                } else {
                    break;
                }
            }
            if space_count > 0 {
                line.replace_range(0..space_count, "");
                self.mark_dirty();
            }
            return space_count;
        }
        0
    }

    pub fn first_non_whitespace_idx(&self, row: usize) -> usize {
        if let Some(line) = self.lines.get(row) {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .count()
        } else {
            0
        }
    }

    pub fn as_full_text(&mut self) -> &str {
        if self.is_dirty || self.full_text_cache.is_none() {
            self.full_text_cache = Some(self.lines.join("\n"));
            self.is_dirty = false;
        }
        self.full_text_cache.as_ref().unwrap()
    }

    fn char_to_byte_idx(&self, y: usize, char_x: usize) -> usize {
        if let Some(line) = self.lines.get(y) {
            line.char_indices()
                .nth(char_x)
                .map(|(idx, _)| idx)
                .unwrap_or_else(|| line.len())
        } else {
            0
        }
    }
}
