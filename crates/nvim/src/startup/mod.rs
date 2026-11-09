//! Starting and stopping the editor process.
//!
//! Upstream's `globals.h` has no translation unit of its own, so the
//! transpiler parked all ~880 of its `EXTERN` declarations here beside
//! `main()`. They have since gone to the modules that own them — the window
//! graph to [`winlayer`], the mode word to [`state`], the message area to
//! [`message`], and so on — and what is left is the process itself:
//!
//! - **What phase the process is in**: `starting` counts down through the
//!   startup stages and `exiting`/`V_DYING`/`EX_EXITVAL` describe the way
//!   out. These are the two flags the rest of the tree tests to know whether
//!   there is an editor yet, or still.
//! - **What it was started as**: the three `*_isatty` answers, `STDIN_FD`,
//!   `FULL_SCREEN`, `SILENT_MODE`, `READONLYMODE`, `RECOVERYMODE`,
//!   `EMBEDDED_MODE`, `HEADLESS_MODE` and the `ui_client_*` set, which say
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
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::global_cell::GlobalCell;
use crate::options::{
    OPT_ARABIC, OPT_CB_FLAG_UNNAMED, OPT_CB_FLAG_UNNAMEDPLUS, OPT_ERRORFILE, OPT_KEYMAP,
    OPT_RIGHTLEFT, OPT_SHADAFILE, OPT_SHORTMESS, OPT_VERBOSEFILE, OPT_WINDOW,
};
use crate::os::libc::uint64_t;
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
pub use self::eventloop::MAIN_LOOP;
pub use self::exit::*;

/// The parsed command line: what `main` was asked to do.
///
/// It owns its words. `argv` is the command line as the process got it, and
/// every string field is a copy of (a tail of) one of them, so nothing here
/// borrows from the process's own argument block.
#[derive(Debug)]
pub(crate) struct MainParams {
    /// The command line, `argv[0]` included.
    pub(crate) argv: Vec<CString>,
    /// `-u {vimrc}`; `NONE` for `--clean` and `-l`.
    pub(crate) vimrc: Option<CString>,
    /// `--clean`.
    pub(crate) clean: bool,
    /// `+cmd`, `-c cmd` and `-S`, run in order after the first file loads.
    pub(crate) commands: Vec<CString>,
    /// `--cmd cmd`, run in order before any config.
    pub(crate) pre_commands: Vec<CString>,
    /// `-l {script}`.
    pub(crate) lua_script: Option<CString>,
    /// Where in `argv` the `-l` script's own arguments start (one past the
    /// script), or -1 for no script.
    pub(crate) script_args_at: c_int,
    /// What the process is editing: one of the `EDIT_*` answers.
    pub(crate) edit_type: c_int,
    /// `-t {tag}`.
    pub(crate) tag: Option<CString>,
    /// `-q {errorfile}`.
    pub(crate) errorfile: Option<CString>,
    /// `-E`: Ex mode reads its input as text.
    pub(crate) input_is_text: bool,
    /// `-n`, `-es`, `-l` and the like: no swap file.
    pub(crate) no_swap_file: bool,
    /// `-D`, or -1.
    pub(crate) debug_break_at: c_int,
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
    pub(crate) remote_at: usize,
    /// `--server {address}`.
    pub(crate) server_addr: Option<CString>,
    /// `-s {scriptin}`.
    pub(crate) scriptin: Option<CString>,
    /// `-w`/`-W {scriptout}`.
    pub(crate) scriptout: Option<CString>,
    /// `-w` (append) rather than `-W` (overwrite).
    pub(crate) scriptout_append: bool,
    /// A `-` argument asked for standard input.
    pub(crate) dash_for_stdin: bool,
}

impl MainParams {
    /// Nothing parsed yet: every field "not given".
    pub(crate) fn new(argv: Vec<CString>) -> Self {
        Self {
            argv,
            vimrc: None,
            clean: false,
            commands: Vec::new(),
            pre_commands: Vec::new(),
            lua_script: None,
            script_args_at: -1,
            edit_type: EDIT_NONE.cast_signed(),
            tag: None,
            errorfile: None,
            input_is_text: false,
            no_swap_file: false,
            debug_break_at: -1,
            window_count: -1,
            window_layout: 0,
            diff_mode: false,
            listen_addr: None,
            remote_at: 0,
            server_addr: None,
            scriptin: None,
            scriptout: None,
            scriptout_append: false,
            dash_for_stdin: false,
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
pub static EX_EXITVAL: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub use crate::harness::starting;
pub static EXITING: GlobalCell<bool> = GlobalCell::new(false);
pub static V_DYING: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static STDIN_ISATTY: GlobalCell<bool> = GlobalCell::new(true);
pub static STDOUT_ISATTY: GlobalCell<bool> = GlobalCell::new(true);
pub static STDERR_ISATTY: GlobalCell<bool> = GlobalCell::new(true);
pub static STDIN_FD: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static FULL_SCREEN: GlobalCell<bool> = GlobalCell::new(false);
pub static SILENT_MODE: GlobalCell<bool> = GlobalCell::new(false);
pub static READONLYMODE: GlobalCell<bool> = GlobalCell::new(false);
pub static RECOVERYMODE: GlobalCell<bool> = GlobalCell::new(false);
pub static VIM_IGNORED: GlobalCell<c_int> = GlobalCell::new(0);
pub static EMBEDDED_MODE: GlobalCell<bool> = GlobalCell::new(false);
pub static HEADLESS_MODE: GlobalCell<bool> = GlobalCell::new(false);
/// The name the process was started by: a copy of `argv[0]`, kept for the
/// life of the process.
static ARGV0: GlobalCell<Option<&'static CStr>> = GlobalCell::new(None);
// The five wordings `mainerr` prints when the command line does not parse.
// Nothing writes them, so they are constants rather than cells; they are
// module-private because argument parsing is the only thing that can fail
// this early.
const ERR_ARG_MISSING: &CStr = c"Argument missing after";
const ERR_OPT_GARBAGE: &CStr = c"Garbage after option argument";
const ERR_OPT_UNKNOWN: &CStr = c"Unknown option argument";
const ERR_TOO_MANY_ARGS: &CStr = c"Too many edit arguments";
const ERR_EXTRA_CMD: &CStr =
    c"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments";
pub(crate) const MAX_ARG_CMDS: usize = 10;
pub static USED_STDIN: GlobalCell<bool> = GlobalCell::new(false);
pub static NVIM_TESTING: GlobalCell<bool> = GlobalCell::new(false);
pub static UI_CLIENT_CHANNEL_ID: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static UI_CLIENT_ERROR_EXIT: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static UI_CLIENT_EXIT_STATUS: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static UI_CLIENT_ATTACHED: GlobalCell<bool> = GlobalCell::new(false);
pub static UI_CLIENT_FORWARD_STDIN: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const WRITEBIN: &CStr = c"wb";
pub(crate) const APPENDBIN: &CStr = c"ab";
