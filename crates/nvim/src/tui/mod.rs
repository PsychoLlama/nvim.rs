//! The built-in terminal UI: it renders the grid the editor publishes and
//! feeds the keys and mouse events it reads back in.

pub mod attrs;
pub mod cursor;
pub mod events;
pub mod input;
pub mod keys;
pub mod negotiate;
pub mod output;
pub mod paint;
pub mod quirks;
pub mod terminfo;
pub mod termkey;
pub mod tui;
pub mod ugrid;
pub mod unibi;
