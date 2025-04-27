//! INFO: Provides wrappers for output to fs
mod common;
// mod ffmpeg;
mod jpg;
mod png;
mod wl_clipboard;

pub use jpg::save_to_jpg;
pub use png::save_to_png;
pub use wl_clipboard::save_to_wl_clipboard;
