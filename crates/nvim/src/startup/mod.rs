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
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::CString;

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

/// The parsed command line: what `main` was asked to do.
///
/// It owns its words. `argv` is the command line as the process got it, and
/// every string field is a copy of (a tail of) one of them, so nothing here
/// borrows from the process's own argument block.
pub(crate) struct MainParams {
    /// The command line, `argv[0]` included.
    pub(crate) argv: Vec<CString>,
    /// `-u {vimrc}`; `NONE` for `--clean` and `-l`.
    pub(crate) use_vimrc: Option<CString>,
    /// `--clean`.
    pub(crate) clean: bool,
    /// `+cmd`, `-c cmd` and `-S`, run in order after the first file loads.
    pub(crate) commands: Vec<CString>,
    /// `--cmd cmd`, run in order before any config.
    pub(crate) pre_commands: Vec<CString>,
    /// `-l {script}`.
    pub(crate) luaf: Option<CString>,
    /// Where in `argv` the `-l` script's own arguments start (one past the
    /// script), or -1 for no script.
    pub(crate) lua_arg0: c_int,
    /// What the process is editing: one of the `EDIT_*` answers.
    pub(crate) edit_type: c_int,
    /// `-t {tag}`.
    pub(crate) tagname: Option<CString>,
    /// `-q {errorfile}`.
    pub(crate) use_ef: Option<CString>,
    /// `-E`: Ex mode reads its input as text.
    pub(crate) input_istext: bool,
    /// `-n`, `-es`, `-l` and the like: no swap file.
    pub(crate) no_swap_file: bool,
    /// `-D`, or -1.
    pub(crate) use_debug_break_level: c_int,
    /// `-o[N]`/`-O[N]`/`-p[N]`'s count: -1 when not given, 0 for "one per
    /// file".
    pub(crate) window_count: c_int,
    /// One of the `WIN_*` answers, or 0 when not given.
    pub(crate) window_layout: c_int,
    /// `-d`.
    pub(crate) diff_mode: bool,
    /// `--listen {address}`.
    pub(crate) listen_addr: Option<CString>,
    /// Where in `argv` the `--remote*` argument is; 0 for none.
    pub(crate) remote: usize,
    /// `--server {address}`.
    pub(crate) server_addr: Option<CString>,
    /// `-s {scriptin}`.
    pub(crate) scriptin: Option<CString>,
    /// `-w`/`-W {scriptout}`.
    pub(crate) scriptout: Option<CString>,
    /// `-w` (append) rather than `-W` (overwrite).
    pub(crate) scriptout_append: bool,
    /// A `-` argument asked for standard input.
    pub(crate) had_stdin_file: bool,
}

impl MainParams {
    /// Nothing parsed yet: every field "not given".
    pub(crate) fn new(argv: Vec<CString>) -> Self {
        Self {
            argv,
            use_vimrc: None,
            clean: false,
            commands: Vec::new(),
            pre_commands: Vec::new(),
            luaf: None,
            lua_arg0: -1,
            edit_type: EDIT_NONE as c_int,
            tagname: None,
            use_ef: None,
            input_istext: false,
            no_swap_file: false,
            use_debug_break_level: -1,
            window_count: -1,
            window_layout: 0,
            diff_mode: false,
            listen_addr: None,
            remote: 0,
            server_addr: None,
            scriptin: None,
            scriptout: None,
            scriptout_append: false,
            had_stdin_file: false,
        }
    }
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
/// The name the process was started by: a copy of `argv[0]`, kept for the
/// life of the process.
static argv0: GlobalCell<Option<&'static CStr>> = GlobalCell::new(None);
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
pub(crate) const MAX_ARG_CMDS: usize = 10;
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const WRITEBIN: &CStr = c"wb";
pub(crate) const APPENDBIN: &CStr = c"ab";
