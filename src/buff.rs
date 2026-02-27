pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if let Some(line) = self.lines.get_mut(row) {
            line.insert(col, ch);
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(col) {
            // カーソル位置から右側を切り取る
            let next_line_content = line.split_off(row);
            // 次の行として挿入
            self.lines.insert(col + 1, next_line_content);
        }
    }

    pub fn join_lines(&mut self, row: usize) -> Option<usize> {
        if row > 0 && row < self.lines.len() {
            let current_line = self.lines.remove(row);
            let prev_line = &mut self.lines[row - 1];
            let join_point = prev_line.len();
            prev_line.push_str(&current_line);
            Some(join_point)
        } else {
            None
        }
    }

    pub fn insert_empty_line(&mut self, at_row: usize) {
        if at_row < self.lines.len() {
            self.lines.insert(at_row + 1, String::new());
        } else {
            self.lines.push(String::new());
        }
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        if col > 0 {
            if let Some(line) = self.lines.get_mut(row) {
                line.remove(col - 1);
            }
        }
    }
}
