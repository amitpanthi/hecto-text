#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]
mod editor;
mod terminal;
mod row;
mod document;
use editor::Editor;
pub use terminal::Terminal;
pub use editor::Position;
pub use row::Row;
pub use document::Document;
pub use editor::SearchDirection;

fn main() {
    Editor::default().run();
}
