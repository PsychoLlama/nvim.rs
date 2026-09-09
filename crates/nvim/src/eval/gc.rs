//! Heads of the intrusive lists that let the VimL garbage collector reach
//! every live list and dict, however deeply they are nested.
//!
//! `tv_list_alloc`/`tv_dict_alloc` push onto these; `tv_list_free`/
//! `tv_dict_free` unlink. `garbage_collect` (eval.rs) walks them to find
//! the objects nothing references any more.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{Dict, List};

pub(crate) static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);

/// Most recently allocated dict.
pub static gc_first_dict: GlobalCell<*mut Dict> = GlobalCell::new(::core::ptr::null_mut::<Dict>());

/// Most recently allocated list.
///
/// `pub` rather than `pub(crate)` because the chain's head *is* the
/// assertion in `crates/nvim/tests/unit/channel_reader.rs`: a list handed to
/// a job callback and not stored is freed again, which is only visible as
/// the head coming back to where it was.
pub static gc_first_list: GlobalCell<*mut List> = GlobalCell::new(::core::ptr::null_mut::<List>());
