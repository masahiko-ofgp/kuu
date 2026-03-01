pub mod vim;
pub mod emacs;

use crossterm::event::KeyEvent;
use crate::app::App;


pub trait KeyHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App);
}

pub fn get_handler(mode: crate::app::KeyBindMode) -> Box<dyn KeyHandler>
{
    match mode {
        crate::app::KeyBindMode::Vim => Box::new(vim::VimHandler),
        crate::app::KeyBindMode::Emacs => Box::new(emacs::EmacsHandler),
    }
}
