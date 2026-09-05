//! What a `:try` is in the middle of.
//!
//! The exception being propagated (`current_exception`, `did_throw`,
//! `need_rethrow`), how deep the `:try` nesting goes (`trylevel`), the
//! errors turned into exceptions along the way (`msg_list`,
//! `suppress_errthrow`) and the `:catch` history a `:finally` may resume
//! (`caught_stack`, `force_abort`). The condition stack's own two flags
//! (`did_endif`, `check_cstack`) sit with them: they are what tells
//! `do_cmdline` that a `:endif` closed something it did not open.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{Exception, MsgList};
use core::ffi::c_int;

pub(crate) static did_endif: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static current_exception: GlobalCell<*mut Exception> =
    GlobalCell::new(::core::ptr::null_mut::<Exception>());
pub(crate) static did_throw: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static need_rethrow: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static check_cstack: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static trylevel: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static force_abort: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_list: GlobalCell<*mut *mut MsgList> =
    GlobalCell::new(::core::ptr::null_mut::<*mut MsgList>());
pub(crate) static suppress_errthrow: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static caught_stack: GlobalCell<*mut Exception> =
    GlobalCell::new(::core::ptr::null_mut::<Exception>());
