#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical homes for the types c2rust duplicated into every module.
// Every name is unique crate-wide and glob-re-exported so sibling
// files can `use super::*`.
pub mod api;
pub mod api_error;
pub mod api_string;
pub mod arglist;
pub mod ascii;
pub mod autocmd;
pub mod buffer;
pub mod builders;
pub mod channel;
pub mod cmdexpand;
pub mod cmdidx;
pub mod context;
pub mod decoration;
pub mod eval;
pub mod ex_cmds;
pub mod ex_eval;
pub mod executor;
pub mod expressions;
pub mod extmark;
pub mod fold;
pub mod globals;
pub mod grid;
pub mod hashtab;
pub mod helpers;
pub mod highlight;
pub mod indent;
pub mod key;
pub mod mark;
pub mod marktree;
pub mod mbyte;
pub mod misc;
pub mod multiqueue_list;
pub mod normal;
pub mod ops;
pub mod option;
pub mod packer;
pub mod parser;
pub mod plines;
pub mod pos;
pub mod regexp;
pub mod register;
pub mod runtime;
pub mod sign;
pub mod statusline;
pub mod time;
pub mod types;
pub mod typval;
pub mod ui;
pub mod vim;
pub mod window;

// Not part of the namespace: a guard that walks the modules above and
// asserts no two of them declare the same name (an E0659 that only fires
// once something uses it).
#[cfg(test)]
mod name_uniqueness;

pub use self::api::*;
pub use self::api_error::*;
pub use self::api_string::*;
pub use self::arglist::*;
pub use self::ascii::*;
pub use self::autocmd::*;
pub use self::buffer::*;
pub use self::builders::*;
pub use self::channel::*;
pub use self::cmdexpand::*;
pub use self::cmdidx::*;
pub use self::context::*;
pub use self::decoration::*;
pub use self::eval::*;
pub use self::ex_cmds::*;
pub use self::ex_eval::*;
pub use self::executor::*;
pub use self::expressions::*;
pub use self::extmark::*;
pub use self::fold::*;
pub use self::globals::*;
pub use self::grid::*;
pub use self::hashtab::*;
pub use self::helpers::*;
pub use self::highlight::*;
pub use self::indent::*;
pub use self::key::*;
pub use self::mark::*;
pub use self::marktree::*;
pub use self::mbyte::*;
pub use self::misc::*;
pub use self::normal::*;
pub use self::ops::*;
pub use self::option::*;
pub use self::packer::*;
pub use self::parser::*;
pub use self::plines::*;
pub use self::pos::*;
pub use self::regexp::*;
pub use self::register::*;
pub use self::runtime::*;
pub use self::sign::*;
pub use self::statusline::*;
pub use self::time::*;
pub use self::types::*;
pub use self::typval::*;
pub use self::ui::*;
pub use self::vim::*;
pub use self::window::*;
