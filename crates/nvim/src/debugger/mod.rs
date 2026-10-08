//! The Vimscript debugger: the `>` prompt, breakpoints and profiling points.
//!
//! Three things live here, and they share one data structure:
//!
//! - **Debug mode.** [`do_debug`] takes over the screen, reads `>` commands
//!   until one of them says to resume, and leaves `debug_break_level` set to
//!   whichever nesting depth should stop next. `do_one_cmd` calls
//!   [`dbg_check_breakpoint`] before every command to find out whether to
//!   enter.
//! - **Breakpoints** (`:breakadd`, `:breakdel`, `:breaklist`), which are
//!   patterns on a function name, a file name, or an expression whose value
//!   is watched for a change.
//! - **Profiling points** (`:profile`, `:profdel`), which reuse the same
//!   entry shape and the same parser -- the only difference is that a
//!   profiling point cannot be `here`, `expr`, or line-numbered.
//!
//! Those last two are why almost everything here is parameterised by
//! [`BreakList`]. Upstream passes `&dbg_breakp` or `&prof_ga` and then
//! compares the pointer back against `&prof_ga` to decide what the parser may
//! accept; naming the choice says the same thing without the identity test.
//!
//! A list is borrowed an entry at a time and never across user code: a watch
//! expression runs arbitrary Vimscript, which may add or delete breakpoints
//! itself. A watch is found again by its number after it ran.
//!
//! The first lives in [`mode`], which the breakpoints reach only through
//! [`do_debug`] -- and which reaches back only for the two values a changed
//! watch expression leaves for the banner to print.
//!
//! Original: `src/nvim/debugger.c`, Vim/Neovim, Vim license.

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

pub(crate) mod state;
#[cfg(test)]
mod tests;
use crate::debugger::state::{
    debug_backtrace_level, debug_break_level, debug_did_msg, debug_mode, debug_tick,
};
use crate::drawscreen::state::cmdline_row;
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::eval::{eval_expr, typval_compare, typval_tostring};
use crate::ex_docmd::state::{ex_nesting_level, ex_normal_busy};
use crate::ex_docmd::{do_cmdline_cmd, do_cmdline_typed};
use crate::ex_getln::getcmdline_bare;
use crate::fileio::file_pat_to_regpat;
use crate::getchar::state::{got_int, ignore_script};
use crate::getchar::{restore_typeahead, save_typeahead};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::keycodes::{K_SPECIAL, KE_SNR};
use crate::memory::XString;
use crate::message::msg_starthere;
use crate::message::state::{
    cmd_silent, did_emsg, emsg_silent, lines_left, msg_row, msg_scroll, need_wait_return, redir_off,
};
use crate::message_fmt::msg_bytes;
use crate::os::env::{expand_env_save_opt_of, home_replace_in};
use crate::path::fixed_fname;
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRING};
use crate::runtime::{estack_sfile_owned, sourcing_lnum};
use crate::semsg;
use crate::smsg;
use crate::state::MODE_NORMAL;
use crate::state::mode::State;
use crate::types::CmdIdx;
use crate::types::{EStackArg, ExArg, Failed, LineNr, MAXPATHL, TypVal, TypeaheadSave};
use crate::ui::state::Rows;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

pub const ESTACK_NONE: EStackArg = 0;
pub const EXPR_IS: crate::types::ExprType = 9;
pub const KS_EXTRA: c_int = 253;

// Debug mode itself: entered from `dbg_check_breakpoint` below.
mod mode;

pub use self::mode::*;

/// One breakpoint or profiling point.
pub(crate) struct Breakpoint {
    /// Breakpoint number, as `:breaklist` prints it.
    nr: c_int,
    /// [`DBG_FUNC`], [`DBG_FILE`] or [`DBG_EXPR`].
    kind: c_int,
    /// Function name, file name, or the watched expression.
    name: XString,
    /// `name` compiled, for the two name kinds; out of the entry while it is
    /// being matched.
    prog: Option<OwnedProg>,
    /// Line within the function or file.
    lnum: LineNr,
    /// `!` was used.
    forceit: bool,
    /// Last value of a watched expression.
    value: Option<TypVal>,
}

pub const DBG_FUNC: c_int = 1;
pub const DBG_FILE: c_int = 2;
pub const DBG_EXPR: c_int = 3;

/// Batch-mode debugging: do not save and restore the typeahead.
static debug_greedy: GlobalCell<bool> = GlobalCell::new(false);
/// The two values a watch expression moved between, waiting to be printed
/// in the debug banner. Each is owned, and the banner takes it.
static debug_oldval: GlobalCell<Option<XString>> = GlobalCell::new(None);
static debug_newval: GlobalCell<Option<XString>> = GlobalCell::new(None);

static dbg_breakp: GlobalCell<Vec<Breakpoint>> = GlobalCell::new(Vec::new());
static prof_ga: GlobalCell<Vec<Breakpoint>> = GlobalCell::new(Vec::new());
/// Number of the last breakpoint defined; `:breakadd` hands out the next.
static last_breakp: GlobalCell<c_int> = GlobalCell::new(0);
/// Whether any `dbg_breakp` entry is a `DBG_EXPR`, so that `do_one_cmd` can
/// skip the per-command expression evaluation when none is.
static has_expr_breakpoint: GlobalCell<bool> = GlobalCell::new(false);

/// Which of the two lists of [`Breakpoint`] entries a command works on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BreakList {
    /// `:breakadd`/`:breakdel`/`:breaklist` -- the debugger's breakpoints.
    Debug,
    /// `:profile`/`:profdel` -- the same entry shape, but the parser refuses
    /// `here`, `expr` and an explicit line number for these.
    Profiling,
}

impl BreakList {
    /// The list a `:breakadd`-family command names: the `:profile` and
    /// `:profdel` spellings drive the profiling list, everything else the
    /// debugger's.
    fn of(args: &ExArg) -> Self {
        let profiling = args.cmdidx == CmdIdx::profile || args.cmdidx == CmdIdx::profdel;
        if profiling {
            Self::Profiling
        } else {
            Self::Debug
        }
    }

    fn cell(self) -> &'static GlobalCell<Vec<Breakpoint>> {
        match self {
            Self::Debug => &dbg_breakp,
            Self::Profiling => &prof_ga,
        }
    }

    /// How many entries the list holds.
    fn len(self) -> usize {
        self.cell().with(Vec::len)
    }

    fn is_empty(self) -> bool {
        self.cell().with(Vec::is_empty)
    }

    /// Run `f` on the `idx`th entry. `f` must not run user code.
    ///
    /// Asked anew each time rather than held: a `DBG_EXPR` entry's
    /// expression runs arbitrary Vimscript, which can grow or shrink the
    /// list.
    fn with<R>(self, idx: usize, f: impl FnOnce(&mut Breakpoint) -> R) -> R {
        self.cell().with_mut(|entries| f(&mut entries[idx]))
    }

    /// Keep a parsed entry, which takes over whatever it owns.
    fn push(self, entry: Breakpoint) {
        self.cell().with_mut(|entries| entries.push(entry));
    }

    /// Take the `idx`th entry out of the list.
    fn remove(self, idx: usize) -> Breakpoint {
        self.cell().with_mut(|entries| entries.remove(idx))
    }
}

// -- Breakpoint checks -----------------------------------------------------

/// The breakpoint `dbg_breakpoint` recorded, waiting for `do_one_cmd` to
/// reach a command that is actually executed.
static debug_breakpoint_name: GlobalCell<Option<XString>> = GlobalCell::new(None);
static debug_breakpoint_lnum: GlobalCell<LineNr> = GlobalCell::new(0);
/// A prompt that was owed but not shown, because the command it belonged to
/// was skipped (an untaken `:if` branch, say). A skipped command that decides
/// to run something itself calls [`dbg_check_skipped`] to collect it.
static debug_skipped: GlobalCell<bool> = GlobalCell::new(false);
static debug_skipped_name: GlobalCell<Option<XString>> = GlobalCell::new(None);

/// Enter debug mode if a breakpoint was hit, or if `ex_nesting_level` is at
/// or below the level the last `>` command asked to stop at -- but only if
/// the command is really being executed.
///
/// Called from `do_one_cmd` before every command.
pub fn dbg_check_breakpoint(excmd: &mut ExArg) {
    debug_skipped.set(false);
    let skip = excmd.skip;
    let Some(name) = debug_breakpoint_name.take() else {
        if ex_nesting_level.get() > debug_break_level.get() {
            return;
        }
        if skip {
            debug_skipped.set(true);
            debug_skipped_name.set(None);
            return;
        }
        do_debug(excmd.line.cstr_from(excmd.line.cmd));
        return;
    };

    if skip {
        debug_skipped.set(true);
        debug_skipped_name.set(Some(name));
        return;
    }

    // A script-local function's name is stored with `K_SNR` in front of it;
    // announce it the way the user spells it.
    let byte = |code: i64| u8::try_from(code).expect("a key code byte");
    let snr = [
        byte(i64::from(K_SPECIAL)),
        byte(i64::from(KS_EXTRA)),
        byte(i64::from(KE_SNR)),
    ];
    let (prefix, rest) = match name.strip_prefix(&snr[..]) {
        Some(rest) => ("<SNR>", rest),
        None => ("", &name[..]),
    };
    let rest = msg_bytes(rest);
    smsg!(
        0,
        "Breakpoint in \"{prefix}{rest}\" line {}",
        i64::from(debug_breakpoint_lnum.get())
    );
    do_debug(excmd.line.cstr_from(excmd.line.cmd));
}

/// Enter debug mode after all, for a command that [`dbg_check_breakpoint`]
/// skipped because `args.skip` was set. True when the prompt was shown.
pub fn dbg_check_skipped(excmd: &mut ExArg) -> bool {
    if !debug_skipped.get() {
        return false;
    }
    // A previous interruption must not flush this prompt's input; only a
    // `CTRL-C` typed at it counts.
    let prev_got_int = got_int.get();
    got_int.set(false);
    debug_breakpoint_name.set(debug_skipped_name.take());
    // `skip` is true on entry, and is put back.
    excmd.skip = false;
    dbg_check_breakpoint(excmd);
    excmd.skip = true;
    got_int.set(got_int.get() | prev_got_int);
    true
}

/// Record that `name` has a breakpoint on `lnum`. Whether it is announced is
/// [`dbg_check_breakpoint`]'s decision, since the line may not be executed.
pub fn dbg_breakpoint(name: &CStr, lnum: LineNr) {
    debug_breakpoint_name.set(Some(XString::from_cstr(name)));
    debug_breakpoint_lnum.set(lnum);
}

// -- Defining and deleting -------------------------------------------------

/// Evaluate a watch expression with error messages off: a bad expression must
/// not make the editor unusable.
fn eval_expr_no_emsg(expr: &[u8]) -> Option<TypVal> {
    let _no_emsg = Suppress::emsg();
    eval_expr(expr)
}

/// `text` from `at` on, past spaces and tabs.
fn skip_white(text: &[u8], at: usize) -> usize {
    let mut at = at.min(text.len());
    while matches!(text.get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    at
}

/// The run of digits at the start of `text` as `getdigits_int32` reads it
/// strictly: saturated at the integer's range.
fn digits_saturated(text: &[u8]) -> (LineNr, usize) {
    let len = text.iter().take_while(|b| b.is_ascii_digit()).count();
    let value = text[..len].iter().fold(0_i64, |n, &d| {
        n.saturating_mul(10).saturating_add(i64::from(d - b'0'))
    });
    (LineNr::try_from(value).unwrap_or(LineNr::MAX), len)
}

/// `atoi(text)`: leading white space, a sign and the digits after it, read
/// as glibc's `(int)strtol(..)` reads them -- saturated at the `long`'s range,
/// then cut to the `int`'s 32 bits.
pub(crate) fn atoi(text: &[u8]) -> c_int {
    let mut at = 0;
    while text
        .get(at)
        .is_some_and(|&b| b == b' ' || (b'\t'..=b'\r').contains(&b))
    {
        at += 1;
    }
    let negative = text.get(at) == Some(&b'-');
    if matches!(text.get(at), Some(b'-' | b'+')) {
        at += 1;
    }
    let digits = text[at.min(text.len())..]
        .iter()
        .take_while(|b| b.is_ascii_digit());
    let value = digits.fold(0_i64, |n, &d| {
        let d = i64::from(d - b'0');
        if negative {
            n.saturating_mul(10).saturating_sub(d)
        } else {
            n.saturating_mul(10).saturating_add(d)
        }
    });
    let low = u32::try_from(value.rem_euclid(1 << 32)).expect("below 2^32");
    c_int::from_ne_bytes(low.to_ne_bytes())
}

/// Parse the arguments of `:breakadd`, `:breakdel` or `:profile` into a
/// fresh entry, which the caller keeps or discards.
///
/// The entry is built *outside* the list on purpose: a `DBG_EXPR` argument
/// is evaluated here, and the Vimscript that runs can reach `:breakadd`
/// itself. Upstream's scratch slot lived one past `ga_len`, so the inner
/// command would build over the outer's half-finished entry and then commit
/// it as its own.
fn dbg_parsearg(arg: &[u8], list: BreakList) -> Result<Breakpoint, Failed> {
    let debugger = list == BreakList::Debug;
    let invalid = || {
        let arg = msg_bytes(arg);
        semsg!("E475: Invalid argument: {arg}");
        Err(Failed)
    };

    let (kind, here) = if arg.starts_with(b"func") {
        (DBG_FUNC, false)
    } else if arg.starts_with(b"file") {
        (DBG_FILE, false)
    } else if debugger && arg.starts_with(b"here") {
        if Buf::current().name.full().is_none() {
            semsg!("E32: No file name");
            return Err(Failed);
        }
        (DBG_FILE, true)
    } else if debugger && arg.starts_with(b"expr") {
        (DBG_EXPR, false)
    } else {
        return invalid();
    };

    let mut at = skip_white(arg, 4);

    // An optional line number, which only the debugger's own list accepts.
    let lnum = if here {
        Win::current().w_cursor.lnum
    } else if debugger && arg.get(at).is_some_and(u8::is_ascii_digit) {
        let (lnum, len) = digits_saturated(&arg[at..]);
        at = skip_white(arg, at + len);
        lnum
    } else {
        0
    };

    // `here` takes no name and everything else requires one; and a function
    // name is given without its parentheses.
    let rest = &arg[at..];
    if (!here && rest.is_empty())
        || (here && !rest.is_empty())
        || (kind == DBG_FUNC && rest.windows(2).any(|pair| pair == b"()"))
    {
        return invalid();
    }

    let mut value = None;
    let name = if kind == DBG_FUNC {
        // `g:` is how the user may spell a global function; the table does
        // not carry it.
        XString::from_bytes(rest.strip_prefix(b"g:").unwrap_or(rest))
    } else if here {
        XString::from_cstr(Buf::current().name.full().expect("checked above"))
    } else if kind == DBG_EXPR {
        // Its first value is the baseline the next check compares to.
        value = eval_expr_no_emsg(rest);
        XString::from_bytes(rest)
    } else {
        // Expand the file name the way `do_source` does -- twice, so that
        // `$DIR/file` expands when `$DIR` is itself `~/dir`.
        let pattern = CString::new(rest).expect("a command line holds no NUL");
        let once = expand_env_save_opt_of(&pattern, false);
        let twice = expand_env_save_opt_of(once.as_cstr(), false);
        if twice.first() == Some(&b'*') {
            twice
        } else {
            fixed_fname(twice.as_cstr()).ok_or(Failed)?
        }
    };
    Ok(Breakpoint {
        nr: 0,
        kind,
        name,
        prog: None,
        lnum,
        forceit: false,
        value,
    })
}

/// `:breakadd`, and `:profile func`/`:profile file`.
pub fn ex_breakadd(excmd: &mut ExArg) {
    let (list, forceit) = (BreakList::of(excmd), excmd.forceit);
    let Ok(mut breakpoint) = dbg_parsearg(excmd.line.arg(), list) else {
        return;
    };
    breakpoint.forceit = forceit;

    if breakpoint.kind == DBG_EXPR {
        last_breakp.set(last_breakp.get() + 1);
        breakpoint.nr = last_breakp.get();
        list.push(breakpoint);
        debug_tick.set(debug_tick.get() + 1);
        if list == BreakList::Debug {
            has_expr_breakpoint.set(true);
        }
        return;
    }

    // A name is matched as a file glob, so it is compiled the way `:next
    // *.c` would be, not as a regexp the user wrote.
    let Some(prog) = file_pat_to_regpat(breakpoint.name.as_cstr())
        .and_then(|pattern| OwnedProg::compile(pattern.as_cstr(), RE_MAGIC + RE_STRING))
    else {
        return;
    };
    breakpoint.prog = Some(prog);

    if breakpoint.lnum == 0 {
        // The default line number is the first.
        breakpoint.lnum = 1;
    }
    // A profiling point is not numbered and does not bump `debug_tick`:
    // nothing lists or deletes it by number.
    if list == BreakList::Debug {
        last_breakp.set(last_breakp.get() + 1);
        breakpoint.nr = last_breakp.get();
        debug_tick.set(debug_tick.get() + 1);
    }
    list.push(breakpoint);
}

/// Recompute [`has_expr_breakpoint`] after the list changed.
fn update_has_expr_breakpoint() {
    let any = dbg_breakp.with(|entries| entries.iter().any(|entry| entry.kind == DBG_EXPR));
    has_expr_breakpoint.set(any);
}

/// `:breakdel` and `:profdel`.
pub fn ex_breakdel(excmd: &mut ExArg) {
    let (list, cmdidx) = (BreakList::of(excmd), excmd.cmdidx);
    let arg = excmd.line.arg();

    let mut del_all = false;
    let todel = if arg.first().is_some_and(u8::is_ascii_digit) {
        // `:breakdel {nr}`
        let nr = atoi(arg);
        list.cell()
            .with(|entries| entries.iter().position(|entry| entry.nr == nr))
    } else if arg.first() == Some(&b'*') {
        del_all = true;
        Some(0)
    } else {
        // `:breakdel {func|file|expr} [lnum] {name}` -- parse it and look
        // for the closest match.
        let Ok(wanted) = dbg_parsearg(arg, list) else {
            return;
        };
        list.cell().with(|entries| {
            let mut best_lnum = 0;
            let mut found = None;
            for (i, entry) in entries.iter().enumerate() {
                let matches = wanted.kind == entry.kind
                    && wanted.name[..] == entry.name[..]
                    && (wanted.lnum == entry.lnum
                        || (wanted.lnum == 0 && (best_lnum == 0 || entry.lnum < best_lnum)));
                if matches {
                    found = Some(i);
                    best_lnum = entry.lnum;
                }
            }
            found
        })
    };

    let Some(todel) = todel else {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E161: Breakpoint not found: {arg}");
        return;
    };

    while !list.is_empty() {
        // The entry taken out owns its name, its compiled pattern and (for a
        // watch) its last value.
        drop(list.remove(todel));
        // `:profdel` is not something `:breaklist` shows, so it does not
        // invalidate anybody's cached view.
        if cmdidx == CmdIdx::breakdel {
            debug_tick.set(debug_tick.get() + 1);
        }
        if !del_all {
            break;
        }
    }

    list.cell().with_mut(|entries| {
        if entries.is_empty() {
            // Upstream freed the array once the last entry went; a vector
            // would keep the capacity for a list that may never grow again.
            *entries = Vec::new();
        }
    });
    if list == BreakList::Debug {
        update_has_expr_breakpoint();
    }
}

/// `:breaklist`.
pub fn ex_breaklist(_excmd: &mut ExArg) {
    let list = BreakList::Debug;
    if list.is_empty() {
        smsg!(0, "No breakpoints defined");
        return;
    }
    for i in 0..list.len() {
        let (nr, kind, name, lnum) = list.with(i, |entry| {
            (entry.nr, entry.kind, entry.name.clone(), entry.lnum)
        });
        if kind == DBG_EXPR {
            let name = msg_bytes(&name);
            smsg!(0, "{nr:3}  expr {name}");
            continue;
        }
        let (label, shown) = if kind == DBG_FUNC {
            ("func", name)
        } else {
            // Where upstream shortens it in `NameBuff`.
            (
                "file",
                home_replace_in(None, name.as_cstr(), MAXPATHL as usize, true),
            )
        };
        let shown = msg_bytes(&shown);
        smsg!(0, "{nr:3}  {label} {shown}  line {}", i64::from(lnum));
    }
}

// -- Lookups ---------------------------------------------------------------

/// The line to break on in function or file `name`, or 0 when nothing
/// matches. `file` says which kind `name` is.
pub(crate) fn dbg_find_breakpoint_named(file: bool, name: &CStr, after: LineNr) -> LineNr {
    debuggy_find(file, name, after, BreakList::Debug).0
}

/// Whether profiling is on for a function or sourced file.
pub(crate) fn has_profiling_named(file: bool, name: &CStr) -> bool {
    profiling_forced(file, name).is_some()
}

/// Whether profiling is on for a function or sourced file, and if so
/// whether its point was defined with `!`.
pub(crate) fn profiling_forced(file: bool, name: &CStr) -> Option<bool> {
    let (lnum, forced) = debuggy_find(file, name, 0, BreakList::Profiling);
    (lnum != 0).then_some(forced)
}

/// The shared body of the lookups: the lowest line above `after` that a name
/// entry matches, or -- for a watch expression whose value just changed --
/// `after` itself; and whether the entry that matched was defined with `!`.
fn debuggy_find(file: bool, fname: &CStr, after: LineNr, list: BreakList) -> (LineNr, bool) {
    if list.is_empty() {
        return (0, false);
    }

    // A script-local function arrives with `K_SNR` in front of its name; the
    // patterns are written against the `<SNR>` spelling.
    let respelled;
    let name = match fname.to_bytes() {
        [first, _, _, tail @ ..] if !file && i32::from(*first) == K_SPECIAL => {
            let mut spelled = b"<SNR>".to_vec();
            spelled.extend_from_slice(tail);
            respelled = CString::new(spelled).expect("a name holds no NUL");
            respelled.as_c_str()
        }
        _ => fname,
    };

    let mut lnum = 0;
    let mut forced = false;
    let mut i = 0;
    while i < list.len() {
        let (kind, line, forceit) = list.with(i, |entry| (entry.kind, entry.lnum, entry.forceit));
        // Skip entries of the wrong kind, and ones for a line beyond a
        // breakpoint already found. Every profiling entry is a candidate:
        // profiling is per file, not per line.
        let candidate = (kind == DBG_FILE) == file
            && kind != DBG_EXPR
            && (list == BreakList::Profiling || (line > after && (lnum == 0 || line < lnum)));

        if candidate {
            // A previous interruption must not cancel the match; only a
            // CTRL-C typed while matching should.
            let prev_got_int = got_int.get();
            got_int.set(false);
            // Out of the entry while it runs: the list is not borrowed
            // across the match.
            let mut prog = list.with(i, |entry| entry.prog.take());
            let matched = prog
                .as_mut()
                .is_some_and(|prog| prog.exec(name, 0, false).is_some());
            list.with(i, |entry| entry.prog = prog);
            if matched {
                lnum = line;
                forced = forceit;
            }
            got_int.set(got_int.get() | prev_got_int);
        } else if kind == DBG_EXPR && watch_changed(list, i) {
            lnum = if after > 0 { after } else { 1 };
            break;
        }
        i += 1;
    }

    (lnum, forced)
}

/// Re-evaluate the watch expression at `idx` and answer whether its value
/// moved, recording the before and after for the prompt banner when it did.
///
/// The expression runs arbitrary Vimscript, which may add or delete
/// breakpoints, so the entry is found again by its number afterwards; one
/// that deleted itself has not changed.
fn watch_changed(list: BreakList, idx: usize) -> bool {
    let (nr, expr) = list.with(idx, |entry| (entry.nr, entry.name.clone()));
    let find = move || {
        list.cell()
            .with(|entries| entries.iter().position(|entry| entry.nr == nr))
    };
    let tv = eval_expr_no_emsg(&expr);
    let Some(at) = find() else {
        return false;
    };
    let previous = list.with(at, |entry| entry.value.take());

    let Some(mut tv) = tv else {
        // The expression stopped evaluating at all, which counts as a
        // change -- but only if there was a value to change from.
        let Some(previous) = previous else {
            return false;
        };
        set_oldval(Some(&previous));
        set_newval(None);
        return true;
    };

    let Some(mut previous) = previous else {
        // First evaluation: the baseline, with no old value to show.
        set_oldval(None);
        set_newval(Some(&tv));
        list.with(at, |entry| entry.value = Some(tv));
        return true;
    };

    // `EXPR_IS` answers "is the same value"; a false answer is a change.
    let changed =
        typval_compare(&mut tv, &mut previous, EXPR_IS, false).is_ok() && tv.number_or_zero() == 0;
    if changed {
        // Render the old value before re-evaluating, because evaluating
        // can reach whatever the old value refers to.
        set_oldval(Some(&previous));
        // `typval_compare` overwrote `tv`, so the new value has to be
        // evaluated a second time before it can be shown.
        let fresh = eval_expr_no_emsg(&expr);
        set_newval(fresh.as_ref());
        drop(previous);
        if let Some(at) = find() {
            list.with(at, |entry| entry.value = fresh);
        }
    } else {
        list.with(at, |entry| entry.value = Some(previous));
    }
    drop(tv);
    changed
}

/// Record the "before" value the prompt banner prints, freeing whatever an
/// earlier change left. A missing value renders as such.
fn set_oldval(tv: Option<&TypVal>) {
    debug_oldval.set(Some(typval_tostring(tv, true)));
}

/// [`set_oldval`] for the "after" value.
fn set_newval(tv: Option<&TypVal>) {
    debug_newval.set(Some(typval_tostring(tv, true)));
}
