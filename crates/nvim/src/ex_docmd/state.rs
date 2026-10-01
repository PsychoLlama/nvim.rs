//! What `do_cmdline` is in the middle of.
//!
//! The active command modifiers (`cmdmod` — `:silent`, `:keepalt`, the
//! `:vertical`/`:tab` split direction), how the current line was reached
//! (`exec_from_reg`, `ex_normal_busy`, `ex_nesting_level`), whether a
//! range command is walking lines for `:global` (`global_busy`,
//! `listcmd_busy`), and the last Ex line typed, which `@:` replays
//! (`last_cmdline`, `new_last_cmdline`, `repeat_cmdline`).
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
use crate::memory::XString;
use crate::types::{CmdMod, CmdModFlags, RegMatch, RegProg};
use core::ffi::{CStr, c_char, c_int};

pub(crate) static exec_from_reg: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static did_emsg_syntax: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static ex_nesting_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static ex_no_reprint: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdmod: GlobalCell<CmdMod> = GlobalCell::new(CmdMod {
    cmod_flags: CmdModFlags::NONE,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter_pat: ::core::ptr::null_mut::<c_char>(),
    cmod_filter_regmatch: RegMatch::new(::core::ptr::null_mut::<RegProg>(), false),
    cmod_filter_force: false,
    cmod_verbose: 0,
    cmod_save_ei: ::core::ptr::null_mut::<c_char>(),
    cmod_did_sandbox: 0,
    cmod_verbose_save: 0,
    cmod_save_msg_silent: 0,
    cmod_save_msg_scroll: 0,
    cmod_did_esilent: 0,
});
pub(crate) static ex_normal_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static global_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static listcmd_busy: GlobalCell<bool> = GlobalCell::new(false);
/// The last command line typed, which `@:` replays and `":` reads.
pub(crate) static last_cmdline: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The first line of the command being typed, which `.` repeats when the
/// command was a single line.
pub(crate) static repeat_cmdline: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The command line just typed; it becomes [`last_cmdline`] once it has
/// run, so that `:@:` inside it still sees the one before.
pub(crate) static new_last_cmdline: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The characters a file name inserted into a command line is escaped for.
pub(crate) const ESCAPE_CHARS: &CStr = c" \t\\\"|";
