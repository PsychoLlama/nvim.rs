//! What `do_cmdline` is in the middle of.
//!
//! The active command modifiers (`cmdmod` — `:silent`, `:keepalt`, the
//! `:vertical`/`:tab` split direction), how the current line was reached
//! (`exec_from_reg`, `ex_normal_busy`, `ex_nesting_level`), whether a
//! range command is walking lines for `:global` (`global_busy`,
//! `listcmd_busy`), and the last Ex line typed, which `@:` replays
//! (`last_cmdline`, `new_last_cmdline`, `repeat_cmdline`).
//!
//! All but `cmdmod` are one [`ExState`] behind one cell, with a `const`
//! selector per field under the old name -- see
//! [`state_record`](crate::global_cell::state_record). `cmdmod` stays a
//! cell of its own: callers borrow it whole (`cmdmod.with(..)`) and the
//! unit suite sets its flags.
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

use crate::global_cell::{GlobalCell, state_record};
use crate::memory::XString;
use crate::types::{CmdMod, CmdModFlags, RegMatch, RegProg};
use core::ffi::{CStr, c_char, c_int};

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

state_record! {
    /// What `do_cmdline` is in the middle of. See the [module docs](self).
    pub(crate) struct ExState in EX as ExField;

    /// The command line being run came from a register (`@:`, `:@r`).
    pub(crate) exec_from_reg: bool = false;
    /// An error was given for the command's syntax: no further message.
    pub(crate) did_emsg_syntax: bool = false;
    /// How deep `do_cmdline` is nested.
    pub(crate) ex_nesting_level: c_int = 0;
    /// The command printed the line itself (`:z`, `:p`): don't print it
    /// again.
    pub(crate) ex_no_reprint: bool = false;
    /// Nonzero while `:normal` (or a menu, or `nvim_input`) runs keys.
    pub(crate) ex_normal_busy: c_int = 0;
    /// Nonzero while `:global` runs its command on each line.
    pub(crate) global_busy: c_int = 0;
    /// `:argdo`/`:bufdo`/... is running its command.
    pub(crate) listcmd_busy: bool = false;
    /// The last command line typed, which `@:` replays and `":` reads.
    pub(crate) last_cmdline: Option<XString> = None;
    /// The first line of the command being typed, which `.` repeats when
    /// the command was a single line.
    pub(crate) repeat_cmdline: Option<XString> = None;
    /// The command line just typed; it becomes `last_cmdline` once it has
    /// run, so that `:@:` inside it still sees the one before.
    pub(crate) new_last_cmdline: Option<XString> = None;
}

/// The characters a file name inserted into a command line is escaped for.
pub(crate) const ESCAPE_CHARS: &CStr = c" \t\\\"|";
