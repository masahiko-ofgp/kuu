#[derive(Debug, Clone)]
pub enum EditAction {
    InsertChar { line: usize, col: usize, c: char },
    DeleteChar { line: usize, col: usize, c: char },
    InsertNewline { line: usize, col: usize },
    DeleteNewline { line: usize, col: usize },
    InsertLine { line: usize, text: String },
    DeleteLine { line: usize, text: String },
    Group(Vec<EditAction>),
}

impl EditAction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::InsertChar { line, col, c } => Self::DeleteChar { line: *line, col: *col, c: *c },
            Self::DeleteChar { line, col, c } => Self::InsertChar { line: *line, col: *col, c: *c },
            Self::InsertNewline { line, col } => Self::DeleteNewline { line: *line, col: *col },
            Self::DeleteNewline { line, col } => Self::InsertNewline { line: *line, col: *col },
            Self::InsertLine { line, text } => Self::DeleteLine { line: *line, text: text.clone() },
            Self::DeleteLine { line, text } => Self::InsertLine { line: *line, text: text.clone() },
            Self::Group(actions) => {
                let reversed: Vec<_> = actions.iter()
                    .rev()
                    .map(|a| a.reverse())
                    .collect();
                Self::Group(reversed)
            }
        }
    }
}

pub struct HistoryManager {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    current_group: Option<Vec<EditAction>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_group: None,
        }
    }

    pub fn start_group(&mut self) {
        if self.current_group.is_none() {
            self.current_group = Some(Vec::new());
        }
    }

    pub fn finish_group(&mut self) {
        if let Some(actions) = self.current_group.take() {
            if !actions.is_empty() {
                self.undo_stack.push(EditAction::Group(actions));
            }
        }
    }

    pub fn push_undo(&mut self, action: EditAction) {
        self.redo_stack.clear();

        if let Some(ref mut group) = self.current_group {
            group.push(action);
        } else {
            self.undo_stack.push(action);
        }
    }

    pub fn pop_undo(&mut self) -> Option<EditAction> {
        self.finish_group();
        self.undo_stack.pop()
    }

    pub fn push_redo(&mut self, action: EditAction) {
        self.redo_stack.push(action);
    }

    pub fn pop_redo(&mut self) -> Option<EditAction> {
        self.redo_stack.pop()
    }

    pub fn push_undo_from_redo(&mut self, action: EditAction) {
        self.undo_stack.push(action);
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
    }
}
