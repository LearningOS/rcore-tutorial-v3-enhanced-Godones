mod basic;
mod god_terminal;
mod snake;

pub use basic::manager::SCREEN_MANAGER;
pub use basic::*;
pub use god_terminal::*;

pub use snake::{Snake, Status, SNAKE};
