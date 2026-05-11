pub struct ViewState {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub row_offset: usize,
    //pub col_offset: usize,
    pub editor_height: u16,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            //col_offset: 0,
            editor_height: 0,
        }
    }
}
