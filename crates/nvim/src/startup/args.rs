//! Command-line argument scanning, and the small initialisations whose input
//! is an argument.
//!
//! [`command_line_scan`] walks argv once. Anything it cannot place is a usage
//! error; anything that needs the *next* word answers `true` for "want an
//! argument" and is collected by [`Scan::option_argument`] at the bottom of
//! the loop.
//!
//! The cursor is a pair: `at` is the word being read and `idx` the byte
//! within it, so `-nRo3` is four options in one word. An `idx` of `None`
//! means "this word is finished", which is how an option that swallowed the
//! rest of its own word says so.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::semsg;
use crate::types::ArgListId;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::api::private::helpers::api_metadata_raw;
use crate::arglist::{alist_add, arg_name};
use crate::cstr;
use crate::diff::diffopt_horizontal;
use crate::eval::vars::set_vim_var_string;
use crate::ex_docmd::do_cmdline_cmd;
use crate::guard::{SavedSctx, Script};
use crate::lua::state::nlua_disable_preload;
use crate::message_fmt::msg_cstr;
use crate::option::vars::{P_LPL, P_UC, P_VERBOSE, P_WRITE, p_shadafile};
use crate::option::{boolean_optval, reset_modifiable, set_option_value_give_err, set_options_bin};
use crate::os::cshim::{gettext, write_c_stderr};
use crate::os::env::getenv_owned;
use crate::os::fs::{dir_exists, os_exepath, os_strerror, write_bytes};
use crate::os::input::os_isatty;
use crate::path::{concat_fnames, path_guess_exepath, path_tail};
use crate::profile::time_msg_at;
use crate::profile::{time_init, time_start};
use crate::runtime::state::{ETYPE_ENV, SID_ENV, current_sctx};
use crate::runtime::{estack_pop, estack_push};
use crate::startup::exit::os_exit;
use crate::startup::usage::{fill_template, mainerr, usage, version};
use crate::startup::{
    EDIT_FILE, EDIT_NONE, EDIT_QF, EDIT_STDIN, EDIT_TAG, MAX_ARG_CMDS, MainParams, SESSION_FILE,
    WIN_HOR, WIN_TABS, WIN_VER, embedded_mode, err_arg_missing, err_extra_cmd, err_opt_garbage,
    err_opt_unknown, err_too_many_args, headless_mode, kOptArabic, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptVerbosefile, kOptWindow, readonlymode, recoverymode, silent_mode,
    stderr_isatty, stdin_fd, stdin_isatty, stdout_isatty,
};
use crate::state::mode::exmode_active;
use crate::types::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::types::{
    Failed, LineNr, MAXPATHL, OptIndex, OptInt, OptVal, OptionSetFlags, ScriptId, Vv,
};

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

/// A number option value.
fn number_opt(value: OptInt) -> OptVal {
    OptVal::Number(value)
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
fn suppress_shada() {
    if p_shadafile(CStr::is_empty) {
        set_opt(kOptShadafile, OptVal::from_cstr(c"NONE"));
    }
}

/// Read a decimal number out of `word` starting at `*idx`, leaving `*idx`
/// past it. Answers `def` when there is no number there.
///
/// The number is C's `atoi`: one too big for a `long` is `LONG_MAX`, and the
/// answer is that cut to an `int`.
pub(crate) fn get_number_arg(word: &[u8], idx: &mut usize, def: c_int) -> c_int {
    let rest = word.get(*idx..).unwrap_or_default();
    let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 {
        return def;
    }
    let value = rest[..digits].iter().fold(0i64, |n, &d| {
        n.saturating_mul(10).saturating_add(i64::from(d - b'0'))
    });
    *idx += digits;
    value as c_int
}

/// Whether standard input should become a buffer.
///
/// Either the user asked for it with a `-` argument, or the process was
/// handed a pipe and nothing else claims it: not headless, not an embedded
/// server without a stdin, not Ex mode reading commands from it, and `-s -`
/// did not already take it.
pub(crate) fn edit_stdin(parm: &MainParams) -> bool {
    let implicit = !headless_mode.get()
        && !(embedded_mode.get() && stdin_fd.get() <= 0)
        && (!exmode_active.get() || parm.input_is_text)
        && !stdin_isatty.get()
        && parm.edit_type <= EDIT_STDIN as c_int
        && parm.scriptin.is_none();
    parm.dash_for_stdin || implicit
}

/// The walking cursor over argv.
struct Scan<'a> {
    parm: &'a mut MainParams,
    /// The whole command line, `argv[0]` included.
    argv: &'a [CString],
    /// The word being read.
    at: usize,
    /// Byte within `argv[at]`, or `None` for "this word is finished".
    idx: Option<usize>,
    /// A bare `--` was seen; everything after it is a file name.
    had_minmin: bool,
}

impl<'a> Scan<'a> {
    /// Words left, counting the one being read.
    fn left(&self) -> usize {
        self.argv.len().saturating_sub(self.at)
    }

    /// The word being read.
    fn arg(&self) -> &'a CStr {
        &self.argv[self.at]
    }

    /// The word before the one being read: the option a separate value
    /// belongs to.
    fn option_word(&self) -> &'a CStr {
        &self.argv[self.at - 1]
    }

    /// Where in the word the cursor is; only asked while it is inside one.
    fn pos(&self) -> usize {
        self.idx.expect("the cursor is inside a word")
    }

    /// Move the cursor `n` bytes on within the word.
    fn skip(&mut self, n: usize) {
        self.idx = Some(self.pos() + n);
    }

    /// The unread tail of the word being read.
    fn tail(&self) -> &'a CStr {
        cstr::in_bytes(&self.arg().to_bytes_with_nul()[self.pos()..])
    }

    /// Whether the word being read has anything left in it.
    fn has_tail(&self) -> bool {
        !self.tail().is_empty()
    }

    /// Does the unread tail *start* with `word`, case-insensitively?
    fn tail_starts_with(&self, word: &CStr) -> bool {
        let (tail, word) = (self.tail().to_bytes(), word.to_bytes());
        tail.get(..word.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(word))
    }

    /// Is the unread tail exactly `word`, case-insensitively?
    fn tail_is(&self, word: &CStr) -> bool {
        self.tail().to_bytes().eq_ignore_ascii_case(word.to_bytes())
    }

    /// Queue a `-c`/`+cmd`/`-S` command for after the first file is loaded.
    fn push_command(&mut self, cmd: CString) {
        if self.parm.commands.len() >= MAX_ARG_CMDS {
            mainerr(err_extra_cmd, None, None);
        }
        self.parm.commands.push(cmd);
    }

    /// Queue a `--cmd` command for before any config is read.
    fn push_pre_command(&mut self, cmd: CString) {
        if self.parm.pre_commands.len() >= MAX_ARG_CMDS {
            mainerr(err_extra_cmd, None, None);
        }
        self.parm.pre_commands.push(cmd);
    }

    /// Claim the one "what is this process editing" slot for `-t` or `-q`.
    fn claim_edit_type(&mut self, kind: c_int) {
        if self.parm.edit_type != EDIT_NONE as c_int {
            mainerr(err_too_many_args, Some(self.arg()), None);
        }
        self.parm.edit_type = kind;
    }

    /// `-s`, `-w` and `-W` share one complaint: a script file is already
    /// open.
    fn script_file_twice(&self) -> ! {
        let template = c"Attempt to open script file again: \"%s %s\"\n";
        let words = [self.option_word().to_bytes(), self.arg().to_bytes()];
        write_c_stderr(&fill_template(template, &words));
        os_exit(2)
    }

    /// A `--long` option. Answers whether it wants the next word.
    fn long_option(&mut self) -> bool {
        if self.tail_is(c"help") {
            usage();
            os_exit(0);
        } else if self.tail_is(c"version") {
            version();
            os_exit(0);
        } else if self.tail_is(c"api-info") {
            let data = api_metadata_raw();
            let written = write_bytes(STDOUT_FILENO, data.as_bytes(), false);
            if written < 0 {
                let why = msg_cstr(os_strerror(written as c_int));
                semsg!("E5420: Failed to write to file: {why}");
            }
            os_exit(0);
        } else if self.tail_is(c"headless") {
            headless_mode.set(true);
        } else if self.tail_is(c"embed") {
            embedded_mode.set(true);
        } else if self.tail_starts_with(c"listen") {
            self.skip(6);
            return true;
        } else if self.tail_starts_with(c"literal") {
            // Nothing to do: file arguments are always literal (#7679).
        } else if self.tail_starts_with(c"remote") {
            // Where in argv this was: everything from here on is forwarded
            // to the server verbatim.
            self.parm.remote_at = self.at;
        } else if self.tail_starts_with(c"server") {
            self.skip(6);
            return true;
        } else if self.tail_starts_with(c"noplugin") {
            P_LPL.set(false);
        } else if self.tail_starts_with(c"cmd") {
            self.skip(3);
            return true;
        } else if self.tail_starts_with(c"startuptime") {
            // The file is already open -- `init_startuptime` found this
            // argument before the scan began. Only swallow the value.
            self.skip(11);
            return true;
        } else if self.tail_starts_with(c"clean") {
            self.parm.vimrc = Some(c"NONE".to_owned());
            self.parm.clean = true;
            set_opt(kOptShadafile, OptVal::from_cstr(c"NONE"));
        } else if self.tail_starts_with(c"luamod-dev") {
            nlua_disable_preload.set(true);
        } else if self.has_tail() {
            mainerr(err_opt_unknown, Some(self.arg()), None);
        } else {
            // A bare `--`: only file names from here on.
            self.had_minmin = true;
        }
        false
    }

    /// Read a number at the cursor, leaving the cursor past it.
    fn number_arg(&mut self, def: c_int) -> c_int {
        let mut idx = self.pos();
        let n = get_number_arg(self.argv[self.at].to_bytes(), &mut idx, def);
        self.idx = Some(idx);
        n
    }

    /// A `-x` option. Answers whether it wants the next word.
    fn short_option(&mut self, c: u8) -> bool {
        match c {
            b'\0' => {
                // A bare `-`: read the buffer from standard input, unless
                // Ex mode already claimed it, where it means "silent".
                if exmode_active.get() {
                    silent_mode.set(true);
                    self.parm.no_swap_file = true;
                } else {
                    if self.parm.edit_type > EDIT_STDIN as c_int {
                        mainerr(err_too_many_args, Some(self.arg()), None);
                    }
                    self.parm.dash_for_stdin = true;
                    self.parm.edit_type = EDIT_STDIN as c_int;
                }
                self.idx = None;
            }
            b'-' => {
                let want = self.long_option();
                if !want {
                    self.idx = None;
                }
                return want;
            }
            b'A' => set_opt(kOptArabic, boolean_optval(Some(true))),
            b'b' => {
                // Before the file names are expanded: on Windows this is
                // what decides whether a shortcut is edited or followed.
                set_options_bin(Buf::current().b_p_bin != 0, true, OptionSetFlags::NONE);
                Buf::current().b_p_bin = 1;
            }
            b'D' => self.parm.debug_break_at = DEBUG_BREAK_ALL,
            b'd' => self.parm.diff_mode = true,
            b'e' => exmode_active.set(true),
            b'E' => {
                exmode_active.set(true);
                self.parm.input_is_text = true;
            }
            // `-f` meant "GUI: run in the foreground"; there is no GUI.
            b'f' => {}
            // `-?` is the MS-Windows spelling of `-h`.
            b'?' | b'h' => {
                usage();
                os_exit(0);
            }
            b'H' => {
                set_opt(kOptKeymap, OptVal::from_cstr(c"hebrew"));
                set_opt(kOptRightleft, boolean_optval(Some(true)));
            }
            b'M' | b'm' => {
                // `-M` is `-m` plus 'nomodifiable'.
                if c == b'M' {
                    reset_modifiable();
                }
                P_WRITE.set(false);
            }
            // `-N` (nocompatible) and `-X` (no X server) are always so.
            b'N' | b'X' => {}
            b'n' => self.parm.no_swap_file = true,
            b'p' | b'o' | b'O' => {
                // A count of 0 means one window per file.
                self.parm.window_count = self.number_arg(0);
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
                    self.parm.errorfile = Some(self.tail().to_owned());
                    self.idx = None;
                } else if self.left() > 1 {
                    // `-q {errorfile}`. A trailing `-q` with nothing
                    // after it falls back to the 'errorfile' option.
                    return true;
                }
            }
            b'R' => {
                readonlymode.set(true);
                Buf::current().b_p_ro = 1;
                P_UC.set(READONLY_UPDATECOUNT);
            }
            // `-L` is the historical spelling of `-r`.
            b'r' | b'L' => recoverymode.set(true),
            b's' => {
                if exmode_active.get() {
                    // `-es`: silent (batch) Ex mode.
                    silent_mode.set(true);
                    self.parm.no_swap_file = true;
                    suppress_shada();
                } else {
                    // `-s {scriptin}`
                    return true;
                }
            }
            b't' => {
                self.claim_edit_type(EDIT_TAG as c_int);
                if self.has_tail() {
                    // `-t{tag}`
                    self.parm.tag = Some(self.tail().to_owned());
                    self.idx = None;
                } else {
                    // `-t {tag}`
                    return true;
                }
            }
            b'v' => {
                version();
                os_exit(0);
            }
            b'V' => {
                let n = self.number_arg(DEFAULT_VERBOSE);
                P_VERBOSE.set(OptInt::from(n));
                if self.has_tail() {
                    // `-V{N}{file}`: whatever follows the digits is
                    // 'verbosefile', and it uses up the whole word.
                    set_opt(kOptVerbosefile, OptVal::from_cstr(self.tail()));
                    self.idx = Some(self.arg().count_bytes());
                }
            }
            b'w' => {
                // `-w{number}` is a 'window' height; `-w {scriptout}` is
                // a file to record the keystrokes into.
                if self
                    .tail()
                    .to_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_digit)
                {
                    let n = self.number_arg(DEFAULT_WINDOW);
                    set_opt(kOptWindow, number_opt(OptInt::from(n)));
                } else {
                    return true;
                }
            }
            b'c' => {
                if self.has_tail() {
                    // `-c{command}`
                    let cmd = self.tail().to_owned();
                    self.push_command(cmd);
                    self.idx = None;
                } else {
                    // `-c {command}`
                    return true;
                }
            }
            // Each of these takes the next word and nothing else.
            b'S' | b'i' | b'l' | b'u' | b'U' | b'W' => return true,
            _ => mainerr(err_opt_unknown, Some(self.arg()), None),
        }
        false
    }

    /// Collect the word an option asked for, and act on it.
    fn option_argument(&mut self, c: u8) {
        // Steps to the next word, having checked there is one -- except for
        // `-S`, whose argument is optional.
        // Nothing may follow an option that takes a separate word.
        if self.has_tail() {
            mainerr(err_opt_garbage, Some(self.arg()), None);
        }
        if self.left() <= 1 && c != b'S' {
            mainerr(err_arg_missing, Some(self.arg()), None);
        }
        self.at += 1;
        self.idx = None;

        match c {
            b'c' | b'S' => {
                if self.parm.commands.len() >= MAX_ARG_CMDS {
                    mainerr(err_extra_cmd, None, None);
                }
                if c == b'S' {
                    // `-S` with nothing after it, or with another option
                    // after it, means the default session file.
                    let file = if self.left() == 0 {
                        SESSION_FILE
                    } else if self.arg().to_bytes().first() == Some(&b'-') {
                        // Hand the option back to the loop.
                        self.at -= 1;
                        SESSION_FILE
                    } else {
                        self.arg()
                    };
                    let cmd = [&b"so "[..], file.to_bytes()].concat();
                    self.push_command(CString::new(cmd).expect("a C string's bytes"));
                } else {
                    let cmd = self.arg().to_owned();
                    self.push_command(cmd);
                }
            }
            b'-' => {
                // Which `--long` asked for this word is only knowable
                // from the word before it.
                let opt = self.option_word().to_bytes();
                let value = self.arg().to_owned();
                if opt == b"--cmd" {
                    self.push_pre_command(value);
                } else if opt == b"--listen" {
                    self.parm.listen_addr = Some(value);
                } else if opt == b"--server" {
                    self.parm.server_addr = Some(value);
                }
                // `--startuptime <file>` was handled before the scan.
            }
            b'q' => self.parm.errorfile = Some(self.arg().to_owned()),
            b'i' => set_opt(kOptShadafile, OptVal::from_cstr(self.arg())),
            b'l' => {
                // `-l {script}`: a batch Lua run. Everything after the
                // script name belongs to the script, not to us.
                headless_mode.set(true);
                silent_mode.set(true);
                P_VERBOSE.set(1);
                self.parm.no_swap_file = true;
                if self.parm.vimrc.is_none() {
                    self.parm.vimrc = Some(c"NONE".to_owned());
                }
                suppress_shada();
                self.parm.lua_script = Some(self.arg().to_owned());
                self.parm.script_args_at = c_int::try_from(self.at + 1).expect("an argv index");
                // Stop the scan: the rest is the script's own argv.
                self.at = self.argv.len();
            }
            b's' => {
                if self.parm.scriptin.is_some() {
                    self.script_file_twice();
                }
                self.parm.scriptin = Some(self.arg().to_owned());
            }
            b't' => self.parm.tag = Some(self.arg().to_owned()),
            b'u' => self.parm.vimrc = Some(self.arg().to_owned()),
            // `-U {gvimrc}`: there is no GUI.
            b'U' => {}
            b'w' | b'W' => {
                // `-w {nr}` is still a 'window' height.
                let word = self.arg().to_bytes();
                if c == b'w' && word.first().is_some_and(u8::is_ascii_digit) {
                    let n = get_number_arg(word, &mut 0, DEFAULT_WINDOW);
                    set_opt(kOptWindow, number_opt(OptInt::from(n)));
                    return;
                }
                if self.parm.scriptout.is_some() {
                    self.script_file_twice();
                }
                self.parm.scriptout = Some(self.arg().to_owned());
                // `-w` appends, `-W` overwrites.
                self.parm.scriptout_append = c == b'w';
            }
            _ => {}
        }
    }

    /// A word that is not an option: a file to edit.
    fn file_argument(&mut self) {
        self.idx = None;

        // Only one kind of editing at a time.
        if self.parm.edit_type > EDIT_STDIN as c_int {
            mainerr(err_too_many_args, Some(self.arg()), None);
        }
        self.parm.edit_type = EDIT_FILE as c_int;

        // The global argument list takes the copy made here.
        let mut path = XString::from_cstr(self.arg());

        // `nvim -d dir file` diffs `dir/file` against `file`.
        let first = arg_name(ArgListId::GLOBAL, 0);
        if self.parm.diff_mode
            && dir_exists(path.as_cstr())
            && let Some(first) = first.filter(|first| !dir_exists(first.as_cstr()))
        {
            let tail = path_tail(first.as_cstr()).to_bytes();
            path = concat_fnames(path.as_cstr().to_bytes(), tail, true);
        }

        // 1: number the buffer after the name is expanded. 2: number it
        // now and make it current -- which is wrong when standard input
        // is going to want the current buffer for itself.
        let alist_fnum_flag = if edit_stdin(self.parm) { 1 } else { 2 };
        alist_add(ArgListId::GLOBAL, path, alist_fnum_flag);
    }
}

/// Walk the command line once, filling in `parm`.
pub(crate) fn command_line_scan(parm: &mut MainParams) {
    // The words are read while the fields are written, so the scan holds
    // them apart from the block for its duration.
    let argv = core::mem::take(&mut parm.argv);
    {
        // Skip argv[0].
        let mut scan = Scan {
            parm,
            argv: &argv,
            at: 1,
            idx: Some(1),
            had_minmin: false,
        };
        while scan.at < scan.argv.len() {
            let word = scan.arg().to_bytes();
            let first = word.first().copied().unwrap_or(0);
            if first == b'+' && !scan.had_minmin {
                // `+`, `+{number}`, `+/{pat}` or `+{command}`.
                scan.idx = None;
                let cmd = if word.len() == 1 {
                    // A bare `+` means "go to the last line".
                    c"$".to_owned()
                } else {
                    cstr::in_bytes(&scan.arg().to_bytes_with_nul()[1..]).to_owned()
                };
                scan.push_command(cmd);
            } else if first == b'-' && !scan.had_minmin {
                let c = scan.tail().to_bytes().first().copied().unwrap_or(0);
                scan.skip(1);
                if scan.short_option(c) {
                    scan.option_argument(c);
                }
            } else {
                scan.file_argument();
            }

            // Move on when the word is used up, or when an option said so.
            let used_up = match scan.idx {
                None | Some(0) => true,
                Some(_) => !scan.has_tail(),
            };
            if used_up {
                scan.at += 1;
                scan.idx = Some(1);
            }
        }
    }
    parm.argv = argv;

    if embedded_mode.get() && (silent_mode.get() || parm.lua_script.is_some()) {
        mainerr(gettext(c"--embed conflicts with -es/-Es/-l"), None, None);
    }

    // The first `+cmd`/`-c` becomes `v:swapcommand`, so the ATTENTION
    // prompt can say what the process was asked to do.
    if let Some(command) = parm.commands.first() {
        let swcmd = [&b":"[..], command.to_bytes(), b"\r"].concat();
        set_vim_var_string(Vv::Swapcommand, Some(&swcmd));
    }

    time_msg_at(c"parsing arguments");
}

/// Open the `--startuptime` file, before anything worth timing happens.
///
/// This runs its own tiny scan of argv because the real one is far too late:
/// the point of `--startuptime` is to time the whole startup, the argument
/// scan included.
pub(crate) fn init_startuptime(parm: &MainParams) {
    // The last word cannot be either of these: both take a value.
    let words = parm
        .argv
        .get(1..parm.argv.len().saturating_sub(1))
        .unwrap_or_default();
    let is = |word: &CString, name: &[u8]| word.to_bytes().eq_ignore_ascii_case(name);
    let names_embed = words.iter().any(|word| is(word, b"--embed"));
    if let Some(at) = words.iter().position(|word| is(word, b"--startuptime")) {
        let label = if names_embed {
            c"Embedded"
        } else {
            c"Primary (or UI client)"
        };
        // `words` starts at argv[1], so its `at` is argv's `at + 1`.
        time_init(&parm.argv[at + 2], label);
        time_start(c"--- NVIM STARTING ---");
    }
}

/// Remember which of the three standard streams are terminals.
pub(crate) fn check_and_set_isatty() {
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
pub(crate) fn init_path(exename: &CStr) {
    let progpath = os_exepath().unwrap_or_else(|| path_guess_exepath(exename, MAXPATHL as usize));
    set_vim_var_string(Vv::Progpath, Some(&progpath));
    set_vim_var_string(Vv::Progname, Some(path_tail(exename).to_bytes()));
}

/// `-d` with no `-o`/`-O` splits the way 'diffopt' asks.
pub(crate) fn set_window_layout(parm: &mut MainParams) {
    if parm.diff_mode && parm.window_layout == 0 {
        parm.window_layout = if diffopt_horizontal() {
            WIN_HOR as c_int
        } else {
            WIN_VER as c_int
        };
    }
}

/// The pseudo-script `$VIMINIT`/`$EXINIT` runs as, held for the scope.
fn env_script() -> SavedSctx {
    Script::context(
        current_sctx
            .get()
            .with_sid(SID_ENV as ScriptId)
            .with_seq(0)
            .with_lnum(0),
    )
}

/// Run the commands in `$VIMINIT` or `$EXINIT`, if it is set.
///
/// Answers `Ok` when the variable existed -- which is what makes it count as
/// a config source, whether or not the commands in it worked.
pub(crate) fn execute_env(env: &'static CStr) -> Result<(), Failed> {
    let initstr = getenv_owned(env).ok_or(Failed)?;

    estack_push(ETYPE_ENV, crate::types::FrameName::Static(env), 0 as LineNr);
    let sctx = env_script();

    let _ = do_cmdline_cmd(initstr.as_cstr());

    estack_pop();
    drop(sctx);
    Ok(())
}
