//! The bundled `libtermkey`: terminal input byte streams to key events.

pub mod csi;
pub mod driver_csi;
pub mod driver_ti;
pub mod format;
pub mod keynames;
pub mod keytables;
pub mod report;
pub mod termkey;
pub mod trie;
pub mod utf8;
