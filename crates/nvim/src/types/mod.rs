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
pub mod ascii;
pub mod buffer;
pub mod builders;
pub mod cmdidx;
pub mod eval;
pub mod ex_cmds;
pub mod globals;
pub mod key;
pub mod multiqueue_list;
pub mod pos;
pub mod time;
pub mod types;
pub mod typval;
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
pub use self::ascii::*;
pub use self::buffer::*;
pub use self::builders::*;
pub use self::cmdidx::*;
pub use self::eval::*;
pub use self::ex_cmds::*;
pub use self::globals::*;
pub use self::key::*;
pub use self::pos::*;
pub use self::time::*;
pub use self::types::*;
pub use self::typval::*;
pub use self::vim::*;
pub use self::window::*;
