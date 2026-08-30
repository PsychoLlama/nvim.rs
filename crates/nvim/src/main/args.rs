//! Command-line argument scanning, and the small initialisations whose input
//! is an argument.
//!
//! [`command_line_scan`] walks argv once. Anything it cannot place is a usage
//! error; anything that needs the *next* word answers `true` for "want an
//! argument" and is collected by [`Scan::option_argument`] at the bottom of
//! the loop.
//!
//! The cursor is a pair: `argv` points at the word being read and `argv_idx`
//! at the byte within it, so `-nRo3` is four options in one word. An
//! `argv_idx` of -1 means "this word is finished", which is how an option
//! that swallowed the rest of its own word says so.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::{size_of, size_of_val};
use core::ptr;

use crate::api::private::helpers::{api_metadata_raw, cstr_as_string};
use crate::arglist::global_arglist;
use crate::arglist::{alist_add, alist_name};
use crate::ascii::ascii_isdigit;
use crate::diff::diffopt_horizontal;
use crate::eval::vars::set_vim_var_string;
use crate::event::libuv::uv_strerror;
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::ga_grow;
use crate::guard::{SavedSctx, Script};
use crate::main::exit::os_exit;
use crate::main::usage::{mainerr, usage, version};
use crate::main::{
    EDIT_FILE, EDIT_NONE, EDIT_QF, EDIT_STDIN, EDIT_TAG, ETYPE_ENV, MAX_ARG_CMDS, SESSION_FILE,
    SID_ENV, WIN_HOR, WIN_TABS, WIN_VER, current_sctx, embedded_mode, err_arg_missing,
    err_extra_cmd, err_opt_garbage, err_opt_unknown, err_too_many_args, exmode_active,
    headless_mode, kOptArabic, kOptKeymap, kOptRightleft, kOptShadafile, kOptValTypeNumber,
    kOptValTypeString, kOptVerbosefile, kOptWindow, mparm_T, nlua_disable_preload, p_lpl,
    p_shadafile, p_uc, p_verbose, p_write, readonlymode, recoverymode, silent_mode, stderr_isatty,
    stdin_fd, stdin_isatty, stdout_isatty, time_msg_at,
};
use crate::memory::{strequal, xfree, xmalloc, xstrdup};
use crate::option::{boolean_optval, reset_modifiable, set_option_value_give_err, set_options_bin};
use crate::os::cshim::{gettext, snprintf, stderr, strncasecmp};
use crate::os::env::os_getenv;
use crate::os::fs::{os_exepath, os_isdir, os_write};
use crate::os::input::os_isatty;
use crate::path::{concat_fnames, path_guess_exepath, path_tail};
use crate::profile::{time_init, time_start};
use crate::runtime::{estack_pop, estack_push};
use crate::strings::vim_snprintf;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{
    Failed, IOSIZE, MAXPATHL, NUL, OptIndex, OptInt, OptVal, OptValData, OptionSetFlags, Vv,
    aentry_T, linenr_T, ptrdiff_t, scid_T, size_t,
};
use crate::winlayer::Live;
use ::libc::{atoi, fprintf, memset, strcasecmp};

/// A bare `-V` is "a little bit verbose".
const DEFAULT_VERBOSE: c_int = 10;

/// The 'window' value a `-w`/`-W` with no digits falls back to. It is never
/// used -- both spellings that reach `get_number_arg` have already checked
/// for a digit -- but it is what upstream passes.
const DEFAULT_WINDOW: c_int = 10;

/// `-D` breaks into the debugger before anything at all.
const DEBUG_BREAK_ALL: c_int = 9999;

/// `-R` slows the undo-file writes right down: the buffer is read-only, so
/// there is nothing worth saving often.
const READONLY_UPDATECOUNT: OptInt = 10000;

/// A string option value naming a string the option layer will copy.
unsafe fn string_opt(value: *const c_char) -> OptVal {
    // SAFETY: `value` is NUL-terminated and outlives the call.
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: unsafe { cstr_as_string(value as *mut c_char) },
        },
    }
}

/// A number option value.
fn number_opt(value: OptInt) -> OptVal {
    OptVal {
        type_0: kOptValTypeNumber,
        data: OptValData { number: value },
    }
}

/// Set an option from a command-line switch. A switch names no scope, so it
/// sets the local and global halves together -- upstream's bare `0`.
fn set_opt(idx: OptIndex, value: OptVal) {
    set_option_value_give_err(idx, value, OptionSetFlags::NONE);
}

/// Turn 'shadafile' off unless the user already named one.
///
/// `-es`, `-Es` and `-l` all do this: a batch process has no business
/// writing over the user's ShaDa.
unsafe fn suppress_shada() {
    // SAFETY: reads and writes one option.
    if p_shadafile.get().is_null() || unsafe { *p_shadafile.get() } as c_int == NUL {
        set_opt(kOptShadafile, unsafe { string_opt(c"NONE".as_ptr()) });
    }
}

/// Read a decimal number out of `p` starting at `*idx`, leaving `*idx` past
/// it. Answers `def` when there is no number there.
pub(crate) unsafe fn get_number_arg(p: *const c_char, idx: *mut c_int, def: c_int) -> c_int {
    // SAFETY: `p` is NUL-terminated and `*idx` indexes into it.
    if !ascii_isdigit(unsafe { *p.offset(*idx as isize) } as c_int) {
        return def;
    }
    let value = unsafe { atoi(p.offset(*idx as isize)) };
    while ascii_isdigit(unsafe { *p.offset(*idx as isize) } as c_int) {
        unsafe { *idx += 1 };
    }
    value
}

/// Whether standard input should become a buffer.
///
/// Either the user asked for it with a `-` argument, or the process was
/// handed a pipe and nothing else claims it: not headless, not an embedded
/// server without a stdin, not Ex mode reading commands from it, and `-s -`
/// did not already take it.
pub(crate) unsafe fn edit_stdin(parmp: *mut mparm_T) -> bool {
    // SAFETY: `parmp` is the caller's live parameter block.
    let implicit = !headless_mode.get()
        && !(embedded_mode.get() && stdin_fd.get() <= 0)
        && (!exmode_active.get() || unsafe { (*parmp).input_istext })
        && !stdin_isatty.get()
        && unsafe { (*parmp).edit_type } <= EDIT_STDIN as c_int
        && unsafe { (*parmp).scriptin }.is_null();
    let explicit = unsafe { (*parmp).had_stdin_file };
    explicit || implicit
}

/// The parameter block the scan fills in.
///
/// One `Live` for the whole scan: the block is the caller's, it outlives
/// `command_line_scan`, and nothing here can free it.
type Mp = Live<mparm_T>;

/// The walking cursor over argv.
struct Scan {
    parm: Mp,
    /// Words left, counting the one `argv` points at.
    argc: c_int,
    argv: *mut *mut c_char,
    /// Byte within `argv[0]`, or -1 for "this word is finished".
    argv_idx: c_int,
    /// A bare `--` was seen; everything after it is a file name.
    had_minmin: bool,
}

impl Scan {
    /// Start a scan of `argc` words at `argv`, filling in `parmp`.
    ///
    /// # Safety
    ///
    /// `parmp` must point at an `mparm_T` that outlives the scan, and `argv`
    /// at `argc` NUL-terminated words. That promise is what makes every
    /// method below ordinary safe code: the struct is private to this file
    /// and `argv_idx` only ever moves within the word `argv` points at, so
    /// each dereference is checked by construction rather than at its use.
    unsafe fn new(parmp: *mut mparm_T, argc: c_int, argv: *mut *mut c_char) -> Self {
        Self {
            // SAFETY: the caller promised `parmp` outlives the scan.
            parm: unsafe { Mp::new(parmp) },
            argc,
            argv,
            argv_idx: 1,
            had_minmin: false,
        }
    }

    /// The word being read.
    fn arg(&self) -> *mut c_char {
        // SAFETY: `argv[0]` is in range while `argc > 0`.
        unsafe { *self.argv }
    }

    /// The unread tail of the word being read.
    fn tail(&self) -> *mut c_char {
        // SAFETY: `argv_idx` indexes within `argv[0]`.
        unsafe { self.arg().offset(self.argv_idx as isize) }
    }

    /// Whether the word being read has anything left in it.
    fn has_tail(&self) -> bool {
        // SAFETY: as `tail`.
        unsafe { *self.tail() as c_int != NUL }
    }

    /// Does the unread tail *start* with `word`, case-insensitively?
    fn tail_starts_with(&self, word: &CStr) -> bool {
        // SAFETY: as `tail`; `word` is NUL-terminated.
        unsafe { strncasecmp(self.tail(), word.as_ptr(), word.count_bytes() as size_t) == 0 }
    }

    /// Is the unread tail exactly `word`, case-insensitively?
    fn tail_is(&self, word: &CStr) -> bool {
        // SAFETY: as `tail`; `word` is NUL-terminated.
        unsafe { strcasecmp(self.tail(), word.as_ptr()) == 0 }
    }

    /// Queue a `-c`/`+cmd`/`-S` command for after the first file is loaded.
    ///
    /// `owned` marks a command this process allocated, so `exe_commands`
    /// frees it again.
    fn push_command(&mut self, cmd: *mut c_char, owned: bool) {
        // The arrays are `MAX_ARG_CMDS` long, which is what the test guards.
        if self.parm.n_commands >= MAX_ARG_CMDS {
            unsafe { mainerr(err_extra_cmd.get(), ptr::null(), ptr::null()) };
        }
        let at = self.parm.n_commands as usize;
        self.parm.cmds_tofree[at] = owned as c_char;
        self.parm.commands[at] = cmd;
        self.parm.n_commands += 1;
    }

    /// Queue a `--cmd` command for before any config is read.
    fn push_pre_command(&mut self, cmd: *mut c_char) {
        // As `push_command`.
        if self.parm.n_pre_commands >= MAX_ARG_CMDS {
            unsafe { mainerr(err_extra_cmd.get(), ptr::null(), ptr::null()) };
        }
        let at = self.parm.n_pre_commands as usize;
        self.parm.pre_commands[at] = cmd;
        self.parm.n_pre_commands += 1;
    }

    /// Claim the one "what is this process editing" slot for `-t` or `-q`.
    fn claim_edit_type(&mut self, kind: c_int) {
        // `mainerr` does not return.
        if self.parm.edit_type != EDIT_NONE as c_int {
            unsafe { mainerr(err_too_many_args.get(), self.arg(), ptr::null()) };
        }
        self.parm.edit_type = kind;
    }

    /// `-s`, `-w` and `-W` share one complaint: a script file is already
    /// open.
    fn script_file_twice(&self) -> ! {
        let mut complaint = [0 as c_char; IOSIZE as usize];
        // SAFETY: `argv[-1]` is the option and `argv[0]` its argument, both
        // in range by the time this is reachable.
        let into = complaint.as_mut_ptr();
        let fmt = gettext(c"Attempt to open script file again: \"%s %s\"\n");
        let option = unsafe { *self.argv.offset(-1) };
        let word = unsafe { *self.argv };
        unsafe { vim_snprintf(into, IOSIZE as size_t, fmt.as_ptr(), option, word) };
        unsafe { fprintf(stderr, c"%s".as_ptr(), complaint.as_ptr()) };
        unsafe { os_exit(2) }
    }

    /// A `--long` option. Answers whether it wants the next word.
    fn long_option(&mut self) -> bool {
        if self.tail_is(c"help") {
            unsafe { usage() };
            unsafe { os_exit(0) };
        } else if self.tail_is(c"version") {
            unsafe { version() };
            unsafe { os_exit(0) };
        } else if self.tail_is(c"api-info") {
            let data = api_metadata_raw();
            let written = unsafe { os_write(STDOUT_FILENO, data.data(), data.len(), false) };
            if written < 0 as ptrdiff_t {
                let why = unsafe { uv_strerror(written as c_int) };
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let why = unsafe { c_str(why) };
                semsg!("E5420: Failed to write to file: {why}");
            }
            unsafe { os_exit(0) };
        } else if self.tail_is(c"headless") {
            headless_mode.set(true);
        } else if self.tail_is(c"embed") {
            embedded_mode.set(true);
        } else if self.tail_starts_with(c"listen") {
            self.argv_idx += 6;
            return true;
        } else if self.tail_starts_with(c"literal") {
            // Nothing to do: file arguments are always literal (#7679).
        } else if self.tail_starts_with(c"remote") {
            // Where in the *original* argv this was: everything from
            // here on is forwarded to the server verbatim.
            self.parm.remote = self.parm.argc - self.argc;
        } else if self.tail_starts_with(c"server") {
            self.argv_idx += 6;
            return true;
        } else if self.tail_starts_with(c"noplugin") {
            p_lpl.set(0);
        } else if self.tail_starts_with(c"cmd") {
            self.argv_idx += 3;
            return true;
        } else if self.tail_starts_with(c"startuptime") {
            // The file is already open -- `init_startuptime` found this
            // argument before the scan began. Only swallow the value.
            self.argv_idx += 11;
            return true;
        } else if self.tail_starts_with(c"clean") {
            self.parm.use_vimrc = c"NONE".as_ptr() as *mut c_char;
            self.parm.clean = true;
            set_opt(kOptShadafile, unsafe { string_opt(c"NONE".as_ptr()) });
        } else if self.tail_starts_with(c"luamod-dev") {
            nlua_disable_preload.set(true);
        } else if self.has_tail() {
            unsafe { mainerr(err_opt_unknown.get(), self.arg(), ptr::null()) };
        } else {
            // A bare `--`: only file names from here on.
            self.had_minmin = true;
        }
        false
    }

    /// A `-x` option. Answers whether it wants the next word.
    fn short_option(&mut self, c: u8) -> bool {
        match c {
            b'\0' => {
                // A bare `-`: read the buffer from standard input, unless
                // Ex mode already claimed it, where it means "silent".
                if exmode_active.get() {
                    silent_mode.set(true);
                    self.parm.no_swap_file = 1;
                } else {
                    if self.parm.edit_type > EDIT_STDIN as c_int {
                        unsafe { mainerr(err_too_many_args.get(), self.arg(), ptr::null()) };
                    }
                    self.parm.had_stdin_file = true;
                    self.parm.edit_type = EDIT_STDIN as c_int;
                }
                self.argv_idx = -1;
            }
            b'-' => {
                let want = self.long_option();
                if !want {
                    self.argv_idx = -1;
                }
                return want;
            }
            b'A' => set_opt(kOptArabic, boolean_optval(Some(true))),
            b'b' => {
                // Before the file names are expanded: on Windows this is
                // what decides whether a shortcut is edited or followed.
                set_options_bin(cur_buf().b_p_bin != 0, true, OptionSetFlags::NONE);
                cur_buf().b_p_bin = 1;
            }
            b'D' => self.parm.use_debug_break_level = DEBUG_BREAK_ALL,
            b'd' => self.parm.diff_mode = 1,
            b'e' => exmode_active.set(true),
            b'E' => {
                exmode_active.set(true);
                self.parm.input_istext = true;
            }
            // `-f` meant "GUI: run in the foreground"; there is no GUI.
            b'f' => {}
            // `-?` is the MS-Windows spelling of `-h`.
            b'?' | b'h' => {
                unsafe { usage() };
                unsafe { os_exit(0) };
            }
            b'H' => {
                set_opt(kOptKeymap, unsafe { string_opt(c"hebrew".as_ptr()) });
                set_opt(kOptRightleft, boolean_optval(Some(true)));
            }
            b'M' | b'm' => {
                // `-M` is `-m` plus 'nomodifiable'.
                if c == b'M' {
                    reset_modifiable();
                }
                p_write.set(0);
            }
            // `-N` (nocompatible) and `-X` (no X server) are always so.
            b'N' | b'X' => {}
            b'n' => self.parm.no_swap_file = 1,
            b'p' | b'o' | b'O' => {
                // A count of 0 means one window per file.
                let word = self.arg();
                self.parm.window_count = unsafe { get_number_arg(word, &raw mut self.argv_idx, 0) };
                self.parm.window_layout = match c {
                    b'p' => WIN_TABS as c_int,
                    b'o' => WIN_HOR as c_int,
                    _ => WIN_VER as c_int,
                };
            }
            b'q' => {
                self.claim_edit_type(EDIT_QF as c_int);
                if self.has_tail() {
                    // `-q{errorfile}`
                    self.parm.use_ef = self.tail();
                    self.argv_idx = -1;
                } else if self.argc > 1 {
                    // `-q {errorfile}`. A trailing `-q` with nothing
                    // after it falls back to the 'errorfile' option.
                    return true;
                }
            }
            b'R' => {
                readonlymode.set(true);
                cur_buf().b_p_ro = 1;
                p_uc.set(READONLY_UPDATECOUNT);
            }
            // `-L` is the historical spelling of `-r`.
            b'r' | b'L' => recoverymode.set(true),
            b's' => {
                if exmode_active.get() {
                    // `-es`: silent (batch) Ex mode.
                    silent_mode.set(true);
                    self.parm.no_swap_file = 1;
                    unsafe { suppress_shada() };
                } else {
                    // `-s {scriptin}`
                    return true;
                }
            }
            b't' => {
                self.claim_edit_type(EDIT_TAG as c_int);
                if self.has_tail() {
                    // `-t{tag}`
                    self.parm.tagname = self.tail();
                    self.argv_idx = -1;
                } else {
                    // `-t {tag}`
                    return true;
                }
            }
            b'v' => {
                unsafe { version() };
                unsafe { os_exit(0) };
            }
            b'V' => {
                let word = self.arg();
                let n = unsafe { get_number_arg(word, &raw mut self.argv_idx, DEFAULT_VERBOSE) };
                p_verbose.set(n as OptInt);
                if self.has_tail() {
                    // `-V{N}{file}`: whatever follows the digits is
                    // 'verbosefile', and it uses up the whole word.
                    set_opt(kOptVerbosefile, unsafe { string_opt(self.tail()) });
                    self.argv_idx = unsafe { cstr::bytes_at(self.arg()) }.len() as c_int;
                }
            }
            b'w' => {
                // `-w{number}` is a 'window' height; `-w {scriptout}` is
                // a file to record the keystrokes into.
                if ascii_isdigit(unsafe { *self.tail() } as c_int) {
                    let word = self.arg();
                    let n = unsafe { get_number_arg(word, &raw mut self.argv_idx, DEFAULT_WINDOW) };
                    set_opt(kOptWindow, number_opt(n as OptInt));
                } else {
                    return true;
                }
            }
            b'c' => {
                if self.has_tail() {
                    // `-c{command}`
                    let cmd = self.tail();
                    self.push_command(cmd, false);
                    self.argv_idx = -1;
                } else {
                    // `-c {command}`
                    return true;
                }
            }
            // Each of these takes the next word and nothing else.
            b'S' | b'i' | b'l' | b'u' | b'U' | b'W' => return true,
            _ => unsafe { mainerr(err_opt_unknown.get(), self.arg(), ptr::null()) },
        }
        false
    }

    /// Collect the word an option asked for, and act on it.
    fn option_argument(&mut self, c: u8) {
        // Steps `argv` forward by one, having checked there is one -- except
        // for `-S`, whose argument is optional.
        // Nothing may follow an option that takes a separate word.
        if self.has_tail() {
            unsafe { mainerr(err_opt_garbage.get(), self.arg(), ptr::null()) };
        }

        self.argc -= 1;
        if self.argc < 1 && c != b'S' {
            unsafe { mainerr(err_arg_missing.get(), self.arg(), ptr::null()) };
        }
        self.argv = unsafe { self.argv.offset(1) };
        self.argv_idx = -1;

        match c {
            b'c' | b'S' => {
                if self.parm.n_commands >= MAX_ARG_CMDS {
                    unsafe { mainerr(err_extra_cmd.get(), ptr::null(), ptr::null()) };
                }
                if c == b'S' {
                    // `-S` with nothing after it, or with another option
                    // after it, means the default session file.
                    let file = if self.argc < 1 {
                        SESSION_FILE.as_ptr() as *mut c_char
                    } else if unsafe { *self.arg() } as u8 == b'-' {
                        // Hand the option back to the loop.
                        self.argc += 1;
                        self.argv = unsafe { self.argv.offset(-1) };
                        SESSION_FILE.as_ptr() as *mut c_char
                    } else {
                        self.arg()
                    };
                    // "so " + the name + the NUL, with room to spare.
                    let size = unsafe { cstr::bytes_at(file) }.len() + 9;
                    let cmd = unsafe { xmalloc(size) } as *mut c_char;
                    unsafe { snprintf(cmd, size, c"so %s".as_ptr(), file) };
                    self.push_command(cmd, true);
                } else {
                    let cmd = self.arg();
                    self.push_command(cmd, false);
                }
            }
            b'-' => {
                // Which `--long` asked for this word is only knowable
                // from the word before it.
                let opt = unsafe { *self.argv.offset(-1) };
                if unsafe { strequal(opt, c"--cmd".as_ptr()) } {
                    let cmd = self.arg();
                    self.push_pre_command(cmd);
                } else if unsafe { strequal(opt, c"--listen".as_ptr()) } {
                    self.parm.listen_addr = self.arg();
                } else if unsafe { strequal(opt, c"--server".as_ptr()) } {
                    self.parm.server_addr = self.arg();
                }
                // `--startuptime <file>` was handled before the scan.
            }
            b'q' => self.parm.use_ef = self.arg(),
            b'i' => set_opt(kOptShadafile, unsafe { string_opt(self.arg()) }),
            b'l' => {
                // `-l {script}`: a batch Lua run. Everything after the
                // script name belongs to the script, not to us.
                headless_mode.set(true);
                silent_mode.set(true);
                p_verbose.set(1 as OptInt);
                self.parm.no_swap_file = 1;
                if self.parm.use_vimrc.is_null() {
                    self.parm.use_vimrc = c"NONE".as_ptr() as *mut c_char;
                }
                unsafe { suppress_shada() };
                self.parm.luaf = self.arg();
                self.argc -= 1;
                if self.argc >= 0 {
                    self.parm.lua_arg0 = self.parm.argc - self.argc;
                    // Stop the scan: the rest is the script's own argv.
                    self.argc = 0;
                }
            }
            b's' => {
                if !self.parm.scriptin.is_null() {
                    self.script_file_twice();
                }
                self.parm.scriptin = self.arg();
            }
            b't' => self.parm.tagname = self.arg(),
            b'u' => self.parm.use_vimrc = self.arg(),
            // `-U {gvimrc}`: there is no GUI.
            b'U' => {}
            b'w' | b'W' => {
                // `-w {nr}` is still a 'window' height.
                if c == b'w' && ascii_isdigit(unsafe { *self.arg() } as c_int) {
                    self.argv_idx = 0;
                    let word = self.arg();
                    let n = unsafe { get_number_arg(word, &raw mut self.argv_idx, DEFAULT_WINDOW) };
                    set_opt(kOptWindow, number_opt(n as OptInt));
                    self.argv_idx = -1;
                    return;
                }
                if !self.parm.scriptout.is_null() {
                    self.script_file_twice();
                }
                self.parm.scriptout = self.arg();
                // `-w` appends, `-W` overwrites.
                self.parm.scriptout_append = c == b'w';
            }
            _ => {}
        }
    }

    /// A word that is not an option: a file to edit.
    fn file_argument(&mut self) {
        // `argv[0]` is a NUL-terminated file name; the global argument list
        // takes ownership of the copy made here.
        self.argv_idx = -1;

        // Only one kind of editing at a time.
        if self.parm.edit_type > EDIT_STDIN as c_int {
            unsafe { mainerr(err_too_many_args.get(), self.arg(), ptr::null()) };
        }
        self.parm.edit_type = EDIT_FILE as c_int;

        let alist = global_arglist();
        unsafe { ga_grow(&raw mut (*alist).al_ga, 1) };
        let mut path = unsafe { xstrdup(self.arg()) };

        // `nvim -d dir file` diffs `dir/file` against `file`.
        if self.parm.diff_mode != 0
            && unsafe { os_isdir(path) }
            && unsafe { (*alist).al_ga.ga_len } > 0
            && !unsafe { os_isdir(alist_name((*alist).al_ga.ga_data as *mut aentry_T)) }
        {
            let first = unsafe { alist_name((*alist).al_ga.ga_data as *mut aentry_T) };
            let joined = unsafe { concat_fnames(path, path_tail(first), true) };
            unsafe { xfree(path as *mut c_void) };
            path = joined;
        }

        // 1: number the buffer after the name is expanded. 2: number it
        // now and make it current -- which is wrong when standard input
        // is going to want the current buffer for itself.
        let alist_fnum_flag = if unsafe { edit_stdin(self.parm.raw()) } {
            1
        } else {
            2
        };
        unsafe { alist_add(alist, path, alist_fnum_flag) };
    }
}

/// Walk the command line once, filling in `parmp`.
pub(crate) unsafe fn command_line_scan(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block and holds the
    // process's own argv, which outlives everything here -- which is why the
    // strings stored into `parmp` are borrowed rather than copied.
    // Skip argv[0].
    let argc = unsafe { (*parmp).argc } - 1;
    let argv = unsafe { (*parmp).argv.offset(1) };
    let mut scan = unsafe { Scan::new(parmp, argc, argv) };

    while scan.argc > 0 {
        let first = unsafe { *scan.arg() } as u8;
        if first == b'+' && !scan.had_minmin {
            // `+`, `+{number}`, `+/{pat}` or `+{command}`.
            scan.argv_idx = -1;
            let cmd = if unsafe { *scan.arg().offset(1) } as c_int == NUL {
                // A bare `+` means "go to the last line".
                c"$".as_ptr() as *mut c_char
            } else {
                unsafe { scan.arg().offset(1) }
            };
            scan.push_command(cmd, false);
        } else if first == b'-' && !scan.had_minmin {
            let c = unsafe { *scan.tail() } as u8;
            scan.argv_idx += 1;
            if scan.short_option(c) {
                scan.option_argument(c);
            }
        } else {
            scan.file_argument();
        }

        // Move on when the word is used up, or when an option said so.
        if scan.argv_idx <= 0 || !scan.has_tail() {
            scan.argc -= 1;
            scan.argv = unsafe { scan.argv.offset(1) };
            scan.argv_idx = 1;
        }
    }

    if embedded_mode.get() && (silent_mode.get() || !unsafe { (*parmp).luaf }.is_null()) {
        let msg = gettext(c"--embed conflicts with -es/-Es/-l");
        unsafe { mainerr(msg.as_ptr(), ptr::null(), ptr::null()) };
    }

    // The first `+cmd`/`-c` becomes `v:swapcommand`, so the ATTENTION
    // prompt can say what the process was asked to do.
    if unsafe { (*parmp).n_commands } > 0 {
        let len = unsafe { cstr::bytes_at((*parmp).commands[0]) }.len() + 2;
        let swcmd = unsafe { xmalloc(len + 1) } as *mut c_char;
        unsafe { snprintf(swcmd, len + 1, c":%s\r".as_ptr(), (*parmp).commands[0]) };
        unsafe { set_vim_var_string(Vv::Swapcommand, swcmd, len as ptrdiff_t) };
        unsafe { xfree(swcmd as *mut c_void) };
    }

    time_msg_at(c"parsing arguments");
}

/// Zero the parameter block, and set the fields whose "not given" value is
/// not zero.
pub(crate) unsafe fn init_params(paramp: *mut mparm_T, argc: c_int, argv: *mut *mut c_char) {
    // SAFETY: `paramp` points at one live `mparm_T`.
    unsafe { memset(paramp as *mut c_void, 0, size_of::<mparm_T>()) };
    unsafe { (*paramp).argc = argc };
    unsafe { (*paramp).argv = argv };
    unsafe { (*paramp).use_debug_break_level = -1 };
    unsafe { (*paramp).window_count = -1 };
    unsafe { (*paramp).listen_addr = ptr::null_mut() };
    unsafe { (*paramp).server_addr = ptr::null_mut() };
    unsafe { (*paramp).remote = 0 };
    unsafe { (*paramp).luaf = ptr::null_mut() };
    unsafe { (*paramp).lua_arg0 = -1 };
}

/// Open the `--startuptime` file, before anything worth timing happens.
///
/// This runs its own tiny scan of argv because the real one is far too late:
/// the point of `--startuptime` is to time the whole startup, the argument
/// scan included.
pub(crate) unsafe fn init_startuptime(paramp: *mut mparm_T) {
    // SAFETY: `paramp.argv[0..argc]` are the process arguments.
    // The last word cannot be either of these: both take a value.
    let last = unsafe { (*paramp).argc } - 1;
    let names_embed = (1..last).any(
        |i| unsafe { strcasecmp(*(*paramp).argv.offset(i as isize), c"--embed".as_ptr()) } == 0,
    );
    for i in 1..last {
        let word = unsafe { *(*paramp).argv.offset(i as isize) };
        if unsafe { strcasecmp(word, c"--startuptime".as_ptr()) } == 0 {
            let label = if names_embed {
                c"Embedded".as_ptr()
            } else {
                c"Primary (or UI client)".as_ptr()
            };
            let file = unsafe { *(*paramp).argv.offset((i + 1) as isize) };
            unsafe { time_init(file, label) };
            unsafe { time_start(c"--- NVIM STARTING ---".as_ptr()) };
            break;
        }
    }
}

/// Remember which of the three standard streams are terminals.
pub(crate) unsafe fn check_and_set_isatty(_paramp: *mut mparm_T) {
    // SAFETY: three `isatty` calls on the standard descriptors.
    stdin_isatty.set(os_isatty(STDIN_FILENO));
    stdout_isatty.set(os_isatty(STDOUT_FILENO));
    stderr_isatty.set(os_isatty(STDERR_FILENO));
    time_msg_at(c"window checked");
}

/// Set `v:progpath` and `v:progname`.
///
/// `v:progpath` is the absolute path of this executable, which the OS
/// usually knows; `exename` (argv[0]) is the fallback for when it does not
/// -- a missing procfs, say (#6734).
pub(crate) unsafe fn init_path(exename: *const c_char) {
    // SAFETY: `exename` is NUL-terminated; `exepath` is `MAXPATHL` bytes.
    let mut exepath: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    let mut exepathlen: size_t = MAXPATHL as size_t;
    if unsafe { os_exepath(exepath.as_mut_ptr(), &raw mut exepathlen) } != 0 {
        unsafe { path_guess_exepath(exename, exepath.as_mut_ptr(), size_of_val(&exepath)) };
    }
    unsafe { set_vim_var_string(Vv::Progpath, exepath.as_mut_ptr(), -1 as ptrdiff_t) };
    unsafe { set_vim_var_string(Vv::Progname, path_tail(exename), -1 as ptrdiff_t) };
}

/// `-d` with no `-o`/`-O` splits the way 'diffopt' asks.
pub(crate) unsafe fn set_window_layout(paramp: *mut mparm_T) {
    // SAFETY: `paramp` is the caller's live parameter block.
    if unsafe { (*paramp).diff_mode } != 0 && unsafe { (*paramp).window_layout } == 0 {
        let layout = if diffopt_horizontal() {
            WIN_HOR as c_int
        } else {
            WIN_VER as c_int
        };
        unsafe { (*paramp).window_layout = layout };
    }
}

/// The pseudo-script `$VIMINIT`/`$EXINIT` runs as, held for the scope.
fn env_script() -> SavedSctx {
    Script::context(
        current_sctx
            .get()
            .with_sid(SID_ENV as scid_T)
            .with_seq(0)
            .with_lnum(0),
    )
}

/// Run the commands in `$VIMINIT` or `$EXINIT`, if it is set.
///
/// Answers `Ok` when the variable existed -- which is what makes it count as
/// a config source, whether or not the commands in it worked.
pub(crate) unsafe fn execute_env(env: *mut c_char) -> Result<(), Failed> {
    // SAFETY: `env` names an environment variable; `os_getenv` hands over an
    // owned copy of its value.
    let initstr = unsafe { os_getenv(env) };
    if initstr.is_null() {
        return Err(Failed);
    }

    estack_push(ETYPE_ENV, env, 0 as linenr_T);
    let sctx = env_script();

    let _ = unsafe { do_cmdline_cmd(initstr) };

    estack_pop();
    drop(sctx);
    unsafe { xfree(initstr as *mut c_void) };
    Ok(())
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
