//! What `do_cmdline` is in the middle of.
//!
//! The active command modifiers (`cmdmod` — `:silent`, `:keepalt`, the
//! `:vertical`/`:tab` split direction), how the current line was reached
//! (`EXEC_FROM_REG`, `EX_NORMAL_BUSY`, `EX_NESTING_LEVEL`), whether a
//! range command is walking lines for `:global` (`GLOBAL_BUSY`,
//! `LISTCMD_BUSY`), and the last Ex line typed, which `@:` replays
//! (`LAST_CMDLINE`, `NEW_LAST_CMDLINE`, `REPEAT_CMDLINE`).
//!
//! All but `cmdmod` are one [`ExState`] behind one cell, with a `const`
//! selector per field under the old name -- see
//! [`state_record`]. `cmdmod` stays a
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

use crate::getchar::state::TypeaheadSave;
use crate::global_cell::{GlobalCell, state_record};
use crate::memory::XString;
use crate::types::Key;
use crate::types::{CmdMod, CmdModFlags};
use core::ffi::{CStr, c_int};

pub(crate) static CMDMOD: GlobalCell<CmdMod> = GlobalCell::new(CmdMod {
    cmod_flags: CmdModFlags::NONE,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter: None,
    cmod_filter_force: false,
    cmod_verbose: 0,
    cmod_save_ei: None,
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
    pub(crate) EXEC_FROM_REG: bool = false;
    /// An error was given for the command's syntax: no further message.
    pub(crate) DID_EMSG_SYNTAX: bool = false;
    /// How deep `do_cmdline` is nested.
    pub(crate) EX_NESTING_LEVEL: c_int = 0;
    /// The command printed the line itself (`:z`, `:p`): don't print it
    /// again.
    pub(crate) EX_NO_REPRINT: bool = false;
    /// Nonzero while `:normal` (or a menu, or `nvim_input`) runs keys.
    pub(crate) EX_NORMAL_BUSY: c_int = 0;
    /// Nonzero while `:global` runs its command on each line.
    pub(crate) GLOBAL_BUSY: c_int = 0;
    /// `:argdo`/`:bufdo`/... is running its command.
    pub(crate) LISTCMD_BUSY: bool = false;
    /// The last command line typed, which `@:` replays and `":` reads.
    pub(crate) LAST_CMDLINE: Option<XString> = None;
    /// The first line of the command being typed, which `.` repeats when
    /// the command was a single line.
    pub(crate) REPEAT_CMDLINE: Option<XString> = None;
    /// The command line just typed; it becomes `LAST_CMDLINE` once it has
    /// run, so that `:@:` inside it still sees the one before.
    pub(crate) NEW_LAST_CMDLINE: Option<XString> = None;
}

/// The characters a file name inserted into a command line is escaped for.
pub(crate) const ESCAPE_CHARS: &CStr = c" \t\\\"|";

/// What `:normal` sets aside. Not `Copy`: `tabuf` owns the typeahead it
/// saved. `Default` is the state its callers declare it in.
#[derive(Default, Debug)]
pub struct SaveState {
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_restart_edit: Key,
    pub save_msg_didout: bool,
    pub save_state: ::core::ffi::c_int,
    pub save_finish_op: bool,
    pub save_opcount: ::core::ffi::c_int,
    pub save_reg_executing: u8,
    pub save_pending_end_reg_executing: bool,
    pub tabuf: TypeaheadSave,
}
