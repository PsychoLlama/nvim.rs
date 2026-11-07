//! The bundled `libtermkey`: terminal input byte streams to key events.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

pub mod csi;
pub mod driver_csi;
pub mod driver_ti;
pub mod format;
pub mod keyinfo;
pub mod keynames;
pub mod keytables;
pub mod report;
pub mod termkey;
pub mod trie;
pub mod types;
pub mod utf8;
