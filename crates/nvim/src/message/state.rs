//! Where the message area is, and what is on it.
//!
//! The cursor into the message area (`msg_col`, `msg_row`), how far it has
//! scrolled and whether the user still owes it a `<CR>` (`msg_scrolled`,
//! `need_wait_return`, `lines_left`, `quit_more`), the message that survives
//! the next redraw (`keep_msg`), and the flags every `msg_*` entry point
//! consults on the way in (`msg_silent`, `msg_scroll`, `msg_didout`,
//! `msg_hist_off`).
//!
//! The `emsg_*` half is the same thing for errors: whether one has been
//! given (`did_emsg`, `called_emsg`), whether the next is to be swallowed
//! (`emsg_off`, `emsg_silent`, `emsg_skip`), and the three cells
//! `assert_fails()` uses to capture one instead of showing it.
//!
//! `redir_*` and `capture_ga` are here for the same reason: they are read on
//! the way *out* of every message, by the tee in [`redir`] that `:redir` and
//! `'verbosefile'` turn on.
//!
//! [`redir`]: super::redir
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::types::{Callback, FILE, GArray, ScreenGrid};
use core::ffi::{CStr, c_char, c_int, c_long};

pub(crate) static on_print: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub(crate) static top_bot_msg: &CStr = c"search hit TOP, continuing at BOTTOM";
pub(crate) static bot_top_msg: &CStr = c"search hit BOTTOM, continuing at TOP";
pub(crate) static line_msg: &CStr = c" line ";
pub(crate) static msg_ext_skip_flush: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_ext_overwrite: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_ext_skip_verbose: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
pub(crate) static msg_grid_pos: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static msg_scrolled_at_flush: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static msg_grid_scroll_discount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static msg_listdo_overwrite: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cmdmsg_rl: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_col: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static msg_row: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static msg_scrolled: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static msg_scrolled_ign: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_did_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static keep_msg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static keep_msg_hl_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static need_fileinfo: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_scroll: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static msg_didout: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_didany: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_nowait: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static emsg_off: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static info_message: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_hist_off: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static need_clr_eos: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_skip: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static emsg_severe: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static emsg_assert_fails_msg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static emsg_assert_fails_lnum: GlobalCell<c_long> = GlobalCell::new(0 as c_long);
pub(crate) static emsg_assert_fails_context: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static did_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static called_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static emsg_on_display: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static no_wait_return: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static need_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static did_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static quit_more: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static lines_left: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub(crate) static msg_no_more: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static msg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static emsg_noredir: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static cmd_silent: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static in_assert_fails: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static redir_off: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static redir_fd: GlobalCell<*mut FILE> =
    GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub(crate) static redir_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static redir_vname: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static capture_ga: GlobalCell<*mut GArray> =
    GlobalCell::new(::core::ptr::null_mut::<GArray>());
pub(crate) static no_lines_msg: &CStr = c"--No lines in buffer--";
