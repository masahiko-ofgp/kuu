pub struct SearchResult {
    pub line_idx: usize,
    pub char_idx: usize,
}

pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub current_match_idx: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            current_match_idx: 0,
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.current_match_idx = 0;
    }
}
