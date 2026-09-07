//! Starting and stopping the editor process.
//!
//! Upstream's `globals.h` has no translation unit of its own, so the
//! transpiler parked all ~880 of its `EXTERN` declarations here beside
//! `main()`. They have since gone to the modules that own them — the window
//! graph to [`winlayer`], the mode word to [`state`], the message area to
//! [`message`], and so on — and what is left is the process itself:
//!
//! - **What phase the process is in**: `starting` counts down through the
//!   startup stages and `exiting`/`v_dying`/`ex_exitval` describe the way
//!   out. These are the two flags the rest of the tree tests to know whether
//!   there is an editor yet, or still.
//! - **What it was started as**: the three `*_isatty` answers, `stdin_fd`,
//!   `full_screen`, `silent_mode`, `readonlymode`, `recoverymode`,
//!   `embedded_mode`, `headless_mode` and the `ui_client_*` set, which say
//!   this process is a UI attached to another one rather than an editor.
//! - **[`MainParams`]**, the parsed command line, and the `EDIT_*`/`WIN_*`
//!   answers it records.
//!
//! The startup path itself lives in the submodules: [`entry`] is `main()` and
//! the phases under it, `args` the command-line parse, `config` the vimrc
//! search, `buffers` the initial windows and buffers, `remote` the
//! `--remote`/`--server` handoff, `usage` the `-h` text and the argument
//! errors, and `exit` the way back out.
//!
//! [`winlayer`]: crate::winlayer::graph
//! [`state`]: crate::state::mode
//! [`message`]: crate::message::state
#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`starting`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::options::{
    kOptArabic, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptErrorfile, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptShortmess, kOptVerbosefile, kOptWindow,
};
use crate::types::uint64_t;
use core::ffi::{CStr, c_char, c_int, c_uint};

mod args;
mod buffers;
mod config;
mod entry;
mod eventloop;
mod exit;
mod remote;
mod usage;

pub use self::entry::*;
pub use self::eventloop::main_loop;
pub use self::exit::*;

#[derive(Clone)] // not `Copy`: it owns several of its strings
pub struct MainParams {
    pub argc: c_int,
    pub argv: *mut *mut c_char,
    pub use_vimrc: *mut c_char,
    pub clean: bool,
    pub n_commands: c_int,
    pub commands: [*mut c_char; 10],
    pub cmds_tofree: [c_char; 10],
    pub n_pre_commands: c_int,
    pub pre_commands: [*mut c_char; 10],
    pub luaf: *mut c_char,
    pub lua_arg0: c_int,
    pub edit_type: c_int,
    pub tagname: *mut c_char,
    pub use_ef: *mut c_char,
    pub input_istext: bool,
    pub no_swap_file: c_int,
    pub use_debug_break_level: c_int,
    pub window_count: c_int,
    pub window_layout: c_int,
    pub diff_mode: c_int,
    pub listen_addr: *mut c_char,
    pub remote: c_int,
    pub server_addr: *mut c_char,
    pub scriptin: *mut c_char,
    pub scriptout: *mut c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}
pub(crate) const EDIT_QF: c_uint = 4;
pub(crate) const WIN_TABS: c_uint = 3;
pub(crate) const WIN_VER: c_uint = 2;
pub(crate) const WIN_HOR: c_uint = 1;
pub(crate) const EDIT_STDIN: c_uint = 2;
pub(crate) const EDIT_FILE: c_uint = 1;
pub(crate) const EDIT_TAG: c_uint = 3;
pub(crate) const EDIT_NONE: c_uint = 0;
pub(crate) const SESSION_FILE: &CStr = c"Session.vim";
pub(crate) const SYS_VIMRC_FILE: &CStr = c"$VIM/sysinit.vim";
pub(crate) const VIMRC_FILE: &CStr = c".nvimrc";
pub(crate) const NO_BUFFERS: c_int = 1 as c_int;
pub static ex_exitval: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
pub static exiting: GlobalCell<bool> = GlobalCell::new(false);
pub static v_dying: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static stdin_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdout_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stderr_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdin_fd: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static full_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
pub static vim_ignored: GlobalCell<c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
static argv0: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
// The five wordings `mainerr` prints when the command line does not parse.
// Nothing writes them, so they are constants rather than cells; they are
// module-private because argument parsing is the only thing that can fail
// this early.
const err_arg_missing: &CStr = c"Argument missing after";
const err_opt_garbage: &CStr = c"Garbage after option argument";
const err_opt_unknown: &CStr = c"Unknown option argument";
const err_too_many_args: &CStr = c"Too many edit arguments";
const err_extra_cmd: &CStr =
    c"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments";
pub(crate) const MAX_ARG_CMDS: c_int = 10 as c_int;
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const WRITEBIN: &CStr = c"wb";
pub(crate) const APPENDBIN: &CStr = c"ab";
