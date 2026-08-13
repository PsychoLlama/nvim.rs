//! Extmarks: positions in a buffer that move with the text.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`set`] | `extmark_set()` -- placing a mark |
//! | [`del`] | removing one mark, or a namespace's worth |
//! | [`get`] | reading marks back, and freeing them all |
//! | [`undo`] | recording and replaying a change's effect on marks |
//! | [`splice`] | moving marks when the text moves |
//!
//! What stays here is the `kExtmark*` operation and undo-object alphabet the
//! five share, and the empty-container initialisers the marktree and the
//! namespace id maps are built from.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::types::{
    ExtmarkInfoArray, ExtmarkOp, ExtmarkType, MTPair, Map_uint32_t_uint32_t, MapHash, Set_uint32_t,
    UndoObjectType, size_t, uint32_t,
};

// The carve of the transpiled module; see each child's docs.
mod del;
mod get;
mod set;
mod splice;
mod undo;

pub use self::del::*;
pub use self::get::*;
pub use self::set::*;
pub use self::splice::*;
pub use self::undo::*;

pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kExtmarkNone: ExtmarkType = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const KV_INITIAL_VALUE: ExtmarkInfoArray = ExtmarkInfoArray {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<MTPair>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint32_t_uint32_t = Map_uint32_t_uint32_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<uint32_t>(),
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
