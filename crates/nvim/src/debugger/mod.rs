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
//! The first lives in [`mode`], which the breakpoints reach only through
//! [`do_debug`] -- and which reaches back only for the two values a changed
//! watch expression leaves for the banner to print.
//!
//! Original: `src/nvim/debugger.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int32, skipwhite};
use crate::cstr;
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::eval::typval::tv_free;
use crate::eval::{eval_expr, typval_compare, typval_tostring};
use crate::ex_docmd::{do_cmdline, do_cmdline_cmd};
use crate::ex_getln::{getcmdline_prompt, getexline};
use crate::fileio::file_pat_to_reg_pat;
use crate::getchar::{restore_typeahead, save_typeahead};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::keycodes::{K_SPECIAL, KE_SNR};
use crate::main::{
    Rows, State, cmd_silent, cmdline_row, curbuf, curwin, debug_backtrace_level, debug_break_level,
    debug_did_msg, debug_mode, debug_tick, did_emsg, emsg_silent, ex_nesting_level, ex_normal_busy,
    got_int, ignore_script, lines_left, msg_row, msg_scroll, need_wait_return, redir_off,
};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::msg_starthere;
use crate::message_fmt::c_str;
use crate::os::cshim::strstr;
use crate::os::env::{expand_env_save, home_replace};
use crate::path::fix_fname;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_prog, vim_regfree};
use crate::runtime::{estack_sfile, sourcing_lnum};
use crate::semsg;
use crate::smsg;
use crate::state::MODE_NORMAL;
use crate::types::CmdIdx;
use crate::types::{
    Callback, Failed, MAXPATHL, NUL, buf_T, colnr_T, estack_arg_T, exarg_T, int32_t, int64_t,
    linenr_T, regprog_T, size_t, tasave_T, typval_T, uint8_t,
};
use ::libc::{atoi, strcpy};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

pub const ESTACK_NONE: estack_arg_T = 0;
pub const EXPR_IS: crate::types::exprtype_T = 9;
pub const NULL: *mut c_void = ptr::null_mut::<c_void>();
pub const KS_EXTRA: c_int = 253;

// Debug mode itself: entered from `dbg_check_breakpoint` below.
mod mode;

pub use self::mode::*;

/// One breakpoint or profiling point.
pub struct debuggy {
    /// Breakpoint number, as `:breaklist` prints it.
    pub dbg_nr: c_int,
    /// [`DBG_FUNC`], [`DBG_FILE`] or [`DBG_EXPR`].
    pub dbg_type: c_int,
    /// Function name, file name, or the watched expression.
    pub dbg_name: *mut c_char,
    /// `dbg_name` compiled, for the two name kinds.
    pub dbg_prog: *mut regprog_T,
    /// Line within the function or file.
    pub dbg_lnum: linenr_T,
    /// `!` was used.
    pub dbg_forceit: c_int,
    /// Last value of a watched expression.
    pub dbg_val: *mut typval_T,
    /// Stored nesting level, for `DBG_EXPR`.
    pub dbg_level: c_int,
}

impl debuggy {
    /// An entry the parser is about to fill in. It owns nothing yet, so
    /// dropping it on a parse error frees nothing.
    fn new() -> Self {
        Self {
            dbg_nr: 0,
            dbg_type: 0,
            dbg_name: ptr::null_mut(),
            dbg_prog: ptr::null_mut(),
            dbg_lnum: 0,
            dbg_forceit: 0,
            dbg_val: ptr::null_mut(),
            dbg_level: 0,
        }
    }
}

pub const DBG_FUNC: c_int = 1;
pub const DBG_FILE: c_int = 2;
pub const DBG_EXPR: c_int = 3;

/// Batch-mode debugging: do not save and restore the typeahead.
static debug_greedy: GlobalCell<bool> = GlobalCell::new(false);
/// The before/after values of a watched expression that just changed; shown
/// once, on the way into the prompt, then freed.
static debug_oldval: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static debug_newval: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

static dbg_breakp: GlobalCell<Vec<debuggy>> = GlobalCell::new(Vec::new());
static prof_ga: GlobalCell<Vec<debuggy>> = GlobalCell::new(Vec::new());
/// Number of the last breakpoint defined; `:breakadd` hands out the next.
static last_breakp: GlobalCell<c_int> = GlobalCell::new(0);
/// Whether any `dbg_breakp` entry is a `DBG_EXPR`, so that `do_one_cmd` can
/// skip the per-command expression evaluation when none is.
static has_expr_breakpoint: GlobalCell<bool> = GlobalCell::new(false);

/// Which of the two lists of [`debuggy`] entries a command works on.
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
    fn of(eap: &exarg_T) -> Self {
        let profiling = eap.cmdidx == CmdIdx::profile || eap.cmdidx == CmdIdx::profdel;
        if profiling {
            Self::Profiling
        } else {
            Self::Debug
        }
    }

    fn cell(self) -> &'static GlobalCell<Vec<debuggy>> {
        match self {
            Self::Debug => &dbg_breakp,
            Self::Profiling => &prof_ga,
        }
    }

    /// How many entries the list holds.
    fn len(self) -> c_int {
        self.cell().with(|entries| entries.len() as c_int)
    }

    fn is_empty(self) -> bool {
        self.cell().with(Vec::is_empty)
    }

    /// The `idx`th entry.
    ///
    /// Recomputed on every call rather than cached, because a `DBG_EXPR`
    /// entry's expression runs arbitrary Vimscript and anything it does --
    /// including another `:breakadd` -- can grow the list and move every
    /// entry with it.
    ///
    /// # Safety
    /// `idx` must be below [`BreakList::len`], and the pointer must not be
    /// held across anything that can add to the list.
    unsafe fn entry(self, idx: c_int) -> *mut debuggy {
        self.cell()
            .with_mut(|entries| entries.as_mut_ptr().wrapping_offset(idx as isize))
    }

    /// Keep a parsed entry, which takes over whatever it owns.
    fn push(self, entry: debuggy) {
        self.cell().with_mut(|entries| entries.push(entry));
    }

    /// Take the `idx`th entry out of the list, leaving the caller to release
    /// what it owns.
    fn remove(self, idx: c_int) -> debuggy {
        self.cell().with_mut(|entries| entries.remove(idx as usize))
    }
}

// -- Breakpoint checks -----------------------------------------------------

/// The breakpoint `dbg_breakpoint` recorded, waiting for `do_one_cmd` to
/// reach a command that is actually executed.
static debug_breakpoint_name: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static debug_breakpoint_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
/// A prompt that was owed but not shown, because the command it belonged to
/// was skipped (an untaken `:if` branch, say). A skipped command that decides
/// to run something itself calls [`dbg_check_skipped`] to collect it.
static debug_skipped: GlobalCell<bool> = GlobalCell::new(false);
static debug_skipped_name: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// Enter debug mode if a breakpoint was hit, or if `ex_nesting_level` is at
/// or below the level the last `>` command asked to stop at -- but only if
/// the command is really being executed.
///
/// Called from `do_one_cmd` before every command.
///
/// # Safety
/// `eap` must be the live `exarg_T`.
pub unsafe fn dbg_check_breakpoint(eap: *mut exarg_T) {
    debug_skipped.set(false);
    // SAFETY: caller contract.
    let skip = unsafe { (*eap).skip != 0 };
    let name = debug_breakpoint_name.get();

    if name.is_null() {
        if ex_nesting_level.get() > debug_break_level.get() {
            return;
        }
        if skip {
            debug_skipped.set(true);
            debug_skipped_name.set(ptr::null_mut());
            return;
        }
        // SAFETY: caller contract.
        unsafe { do_debug((*eap).cmd) };
        return;
    }

    if skip {
        debug_skipped.set(true);
        debug_skipped_name.set(name);
        debug_breakpoint_name.set(ptr::null_mut());
        return;
    }

    // A script-local function's name is stored with `K_SNR` in front of it;
    // announce it the way the user spells it.
    // SAFETY: `name` is the NUL-terminated function or file name the
    // breakpoint matched.
    let is_snr = unsafe { *name } as uint8_t as c_int == K_SPECIAL
        && unsafe { *name.offset(1) } as uint8_t as c_int == KS_EXTRA
        && unsafe { *name.offset(2) } as c_int == KE_SNR as c_int;
    let (prefix, rest) = if is_snr {
        (c"<SNR>".as_ptr(), unsafe { name.offset(3) })
    } else {
        (c"".as_ptr(), name)
    };
    // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
    let (prefix, rest) = unsafe { (c_str(prefix), c_str(rest)) };
    smsg!(
        0,
        "Breakpoint in \"{prefix}{rest}\" line {}",
        debug_breakpoint_lnum.get() as int64_t
    );
    debug_breakpoint_name.set(ptr::null_mut());
    unsafe { do_debug((*eap).cmd) };
}

/// Enter debug mode after all, for a command that [`dbg_check_breakpoint`]
/// skipped because `eap.skip` was set. True when the prompt was shown.
///
/// # Safety
/// As [`dbg_check_breakpoint`].
pub unsafe fn dbg_check_skipped(eap: *mut exarg_T) -> bool {
    if !debug_skipped.get() {
        return false;
    }
    // A previous interruption must not flush this prompt's input; only a
    // `CTRL-C` typed at it counts.
    let prev_got_int = got_int.get();
    got_int.set(false);
    debug_breakpoint_name.set(debug_skipped_name.get());
    // SAFETY: caller contract; `eap.skip` is true on entry, and is put back.
    unsafe { (*eap).skip = 0 };
    unsafe { dbg_check_breakpoint(eap) };
    unsafe { (*eap).skip = 1 };
    got_int.set(got_int.get() | prev_got_int);
    true
}

/// Record that `name` has a breakpoint on `lnum`. Whether it is announced is
/// [`dbg_check_breakpoint`]'s decision, since the line may not be executed.
pub fn dbg_breakpoint(name: *mut c_char, lnum: linenr_T) {
    debug_breakpoint_name.set(name);
    debug_breakpoint_lnum.set(lnum);
}

// -- Defining and deleting -------------------------------------------------

/// Evaluate a watch expression with error messages off: a bad expression must
/// not make the editor unusable.
///
/// # Safety
/// `bp` must point at a live entry whose `dbg_name` is the expression.
unsafe fn eval_expr_no_emsg(bp: *mut debuggy) -> *mut typval_T {
    let _no_emsg = Suppress::emsg();
    // SAFETY: caller contract.
    unsafe { eval_expr((*bp).dbg_name, ptr::null_mut()) }
}

/// Parse the arguments of `:breakadd`, `:breakdel` or `:profile` into a
/// fresh entry, which the caller keeps or discards.
///
/// `dbg_name` comes out allocated. `Err` means nothing was allocated that
/// the caller has to clean up.
///
/// The entry is built *outside* the list on purpose: a `DBG_EXPR` argument
/// is evaluated here, and the Vimscript that runs can reach `:breakadd`
/// itself. Upstream's scratch slot lived one past `ga_len`, so the inner
/// command would build over the outer's half-finished entry and then commit
/// it as its own.
///
/// # Safety
/// `arg` must be NUL-terminated.
unsafe fn dbg_parsearg(arg: *mut c_char, list: BreakList) -> Result<debuggy, Failed> {
    let mut entry = debuggy::new();
    let bp = &raw mut entry;
    let debugger = list == BreakList::Debug;

    // SAFETY: caller contract; every read below stays inside `arg`, and `bp`
    // is this frame's entry, which nothing else can reach.
    let (kind, here) = unsafe {
        if cstr::starts_with(arg, b"func") {
            (DBG_FUNC, false)
        } else if cstr::starts_with(arg, b"file") {
            (DBG_FILE, false)
        } else if debugger && cstr::starts_with(arg, b"here") {
            if (*curbuf.get()).b_ffname.is_null() {
                semsg!("E32: No file name");
                return Err(Failed);
            }
            (DBG_FILE, true)
        } else if debugger && cstr::starts_with(arg, b"expr") {
            (DBG_EXPR, false)
        } else {
            semsg!("E475: Invalid argument: {}", c_str(arg));
            return Err(Failed);
        }
    };
    // SAFETY: `bp` is the reserved scratch entry.
    unsafe { (*bp).dbg_type = kind };

    // SAFETY: the keyword was four bytes, so this stays inside `arg`.
    let mut p = unsafe { skipwhite(arg.offset(4)) };

    // An optional line number, which only the debugger's own list accepts.
    // SAFETY: `p` is inside `arg`, and `getdigits_int32` only advances it.
    let lnum = unsafe {
        if here {
            (*curwin.get()).w_cursor.lnum
        } else if debugger && ascii_isdigit(*p as c_int) {
            let lnum = getdigits_int32(&raw mut p, true, 0 as int32_t) as linenr_T;
            p = skipwhite(p);
            lnum
        } else {
            0 as linenr_T
        }
    };
    // SAFETY: as above.
    unsafe { (*bp).dbg_lnum = lnum };

    // `here` takes no name and everything else requires one; and a function
    // name is given without its parentheses.
    // SAFETY: `p` is inside `arg`.
    let malformed = unsafe {
        let empty = *p as c_int == NUL;
        (!here && empty)
            || (here && !empty)
            || (kind == DBG_FUNC && !strstr(p, c"()".as_ptr()).is_null())
    };
    if malformed {
        // SAFETY: caller contract.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
        return Err(Failed);
    }

    // SAFETY: `p` is inside `arg`; every branch leaves `dbg_name` owning an
    // allocation or null.
    let name = unsafe {
        if kind == DBG_FUNC {
            // `g:` is how the user may spell a global function; the table
            // does not carry it.
            let bare = if cstr::starts_with(p, b"g:") {
                p.offset(2)
            } else {
                p
            };
            xstrdup(bare)
        } else if here {
            xstrdup((*curbuf.get()).b_ffname)
        } else if kind == DBG_EXPR {
            let expr = xstrdup(p);
            // `eval_expr_no_emsg` reads the entry's `dbg_name`, so the
            // expression has to be stored before it can be evaluated -- and
            // its first value is the baseline the next check compares to.
            (*bp).dbg_name = expr;
            (*bp).dbg_val = eval_expr_no_emsg(bp);
            expr
        } else {
            // Expand the file name the way `do_source` does -- twice, so that
            // `$DIR/file` expands when `$DIR` is itself `~/dir`.
            let once = expand_env_save(p);
            if once.is_null() {
                return Err(Failed);
            }
            let twice = expand_env_save(once);
            xfree(once.cast());
            if twice.is_null() {
                return Err(Failed);
            }
            if *twice as c_int != '*' as c_int {
                let fixed = fix_fname(twice);
                xfree(twice.cast());
                fixed
            } else {
                twice
            }
        }
    };
    // SAFETY: `bp` is this frame's entry; `name` is owned or null.
    unsafe { (*bp).dbg_name = name };
    if name.is_null() {
        Err(Failed)
    } else {
        Ok(entry)
    }
}

/// `:breakadd`, and `:profile func`/`:profile file`.
///
/// # Safety
/// `eap` must be the live `exarg_T`.
pub unsafe fn ex_breakadd(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let (list, arg, forceit) = unsafe { (BreakList::of(&*eap), (*eap).arg, (*eap).forceit) };
    // SAFETY: `arg` is the NUL-terminated argument.
    let Ok(mut bp) = (unsafe { dbg_parsearg(arg, list) }) else {
        return;
    };
    bp.dbg_forceit = forceit;

    if bp.dbg_type == DBG_EXPR {
        last_breakp.set(last_breakp.get() + 1);
        bp.dbg_nr = last_breakp.get();
        list.push(bp);
        debug_tick.set(debug_tick.get() + 1);
        if list == BreakList::Debug {
            has_expr_breakpoint.set(true);
        }
        return;
    }

    // A name is matched as a file glob, so it is compiled the way `:next
    // *.c` would be, not as a regexp the user wrote.
    // SAFETY: `dbg_name` is the owned NUL-terminated name the parser left.
    let compiled = unsafe {
        let pat = file_pat_to_reg_pat(bp.dbg_name, ptr::null(), ptr::null_mut(), 0);
        if !pat.is_null() {
            bp.dbg_prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            xfree(pat.cast());
        }
        !pat.is_null() && !bp.dbg_prog.is_null()
    };
    if !compiled {
        // SAFETY: the name is this function's to free; the entry is dropped.
        unsafe { xfree(bp.dbg_name.cast()) };
        return;
    }

    if bp.dbg_lnum == 0 as linenr_T {
        // The default line number is the first.
        bp.dbg_lnum = 1 as linenr_T;
    }
    // A profiling point is not numbered and does not bump `debug_tick`:
    // nothing lists or deletes it by number.
    if list == BreakList::Debug {
        last_breakp.set(last_breakp.get() + 1);
        bp.dbg_nr = last_breakp.get();
        debug_tick.set(debug_tick.get() + 1);
    }
    list.push(bp);
}

/// Recompute [`has_expr_breakpoint`] after the list changed.
fn update_has_expr_breakpoint() {
    let list = BreakList::Debug;
    let any = (0..list.len()).any(|i| {
        // SAFETY: `i` is below `ga_len`.
        unsafe { (*list.entry(i)).dbg_type == DBG_EXPR }
    });
    has_expr_breakpoint.set(any);
}

/// `:breakdel` and `:profdel`.
///
/// # Safety
/// `eap` must be the live `exarg_T`.
pub unsafe fn ex_breakdel(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let (list, arg, cmdidx) = unsafe { (BreakList::of(&*eap), (*eap).arg, (*eap).cmdidx) };
    // SAFETY: `arg` is NUL-terminated.
    let first = unsafe { *arg as c_int };

    let mut del_all = false;
    let todel = if ascii_isdigit(first) {
        // `:breakdel {nr}`
        // SAFETY: `arg` is NUL-terminated.
        let nr = unsafe { atoi(arg) };
        // SAFETY: `i` is below `ga_len`.
        (0..list.len()).find(|&i| unsafe { (*list.entry(i)).dbg_nr == nr })
    } else if first == '*' as c_int {
        del_all = true;
        Some(0)
    } else {
        // `:breakdel {func|file|expr} [lnum] {name}` -- parse it and look
        // for the closest match.
        // SAFETY: `arg` is NUL-terminated.
        let Ok(bp) = (unsafe { dbg_parsearg(arg, list) }) else {
            return;
        };
        let mut best_lnum = 0 as linenr_T;
        let mut found = None;
        for i in 0..list.len() {
            // SAFETY: `i` is below the list's length, and `bp` is this
            // frame's; both names are owned and NUL-terminated.
            let matches = unsafe {
                let bpi = list.entry(i);
                bp.dbg_type == (*bpi).dbg_type
                    && cstr::eq(bp.dbg_name, (*bpi).dbg_name)
                    && (bp.dbg_lnum == (*bpi).dbg_lnum
                        || (bp.dbg_lnum == 0 as linenr_T
                            && (best_lnum == 0 as linenr_T || (*bpi).dbg_lnum < best_lnum)))
            };
            if matches {
                found = Some(i);
                // SAFETY: as above.
                best_lnum = unsafe { (*list.entry(i)).dbg_lnum };
            }
        }
        // SAFETY: the parsed entry is discarded either way.
        unsafe { xfree(bp.dbg_name.cast()) };
        found
    };

    let Some(todel) = todel else {
        // SAFETY: `arg` is NUL-terminated.
        let arg = unsafe { c_str(arg) };
        semsg!("E161: Breakpoint not found: {arg}");
        return;
    };

    while !list.is_empty() {
        // `todel` is below the list's length, and the entry taken out of it
        // owns its name, its compiled pattern and (for a watch) its last
        // value.
        let bp = list.remove(todel);
        // SAFETY: all three are this entry's own allocations.
        unsafe { xfree(bp.dbg_name.cast()) };
        if bp.dbg_type == DBG_EXPR && !bp.dbg_val.is_null() {
            unsafe { tv_free(bp.dbg_val) };
        }
        unsafe { vim_regfree(bp.dbg_prog) };
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
///
/// # Safety
/// `eap` is unused, but the signature is the Ex-command one.
pub unsafe fn ex_breaklist(_eap: *mut exarg_T) {
    let list = BreakList::Debug;
    if list.is_empty() {
        smsg!(0, "No breakpoints defined");
        return;
    }
    // Where `home_replace` shortens each file name; upstream shares
    // `NameBuff`, which the message it feeds writes again.
    let mut shortened = [0 as c_char; MAXPATHL as usize];
    let namebuff = shortened.as_mut_ptr();

    for i in 0..list.len() {
        // SAFETY: `i` is below `ga_len`; the entry's name is owned and
        // NUL-terminated, and the messages print bytes verbatim.
        let bp = unsafe { list.entry(i) };
        let kind = unsafe { (*bp).dbg_type };
        if kind == DBG_FILE {
            unsafe {
                home_replace(
                    ptr::null::<buf_T>(),
                    (*bp).dbg_name,
                    namebuff,
                    MAXPATHL as size_t,
                    true,
                )
            };
        }
        if kind == DBG_EXPR {
            // SAFETY: `bp` is a live breakpoint of the editor's own.
            let (nr, name) = unsafe { ((*bp).dbg_nr, c_str((*bp).dbg_name)) };
            smsg!(0, "{nr:3}  expr {name}");
        } else {
            let (label, shown) = if kind == DBG_FUNC {
                (c"func".as_ptr(), unsafe { (*bp).dbg_name })
            } else {
                (c"file".as_ptr(), namebuff)
            };
            // SAFETY: `bp` is a live breakpoint of the editor's own, and both
            // strings are NUL-terminated.
            let (nr, lnum) = unsafe { ((*bp).dbg_nr, (*bp).dbg_lnum as int64_t) };
            let (label, shown) = unsafe { (c_str(label), c_str(shown)) };
            smsg!(0, "{nr:3}  {label} {shown}  line {lnum}");
        }
    }
}

// -- Lookups ---------------------------------------------------------------

/// The line to break on in `fname`, or 0 when nothing matches.
///
/// # Safety
/// `fname` must be NUL-terminated.
pub unsafe fn dbg_find_breakpoint(file: bool, fname: *mut c_char, after: linenr_T) -> linenr_T {
    // SAFETY: caller contract.
    unsafe { debuggy_find(file, fname, after, BreakList::Debug, ptr::null_mut()) }
}

/// Whether profiling is on for a function or sourced file, and through `fp`
/// whether it was defined with `!`.
///
/// # Safety
/// `fname` must be NUL-terminated; `fp` null or writable.
pub unsafe fn has_profiling(file: bool, fname: *mut c_char, fp: *mut bool) -> bool {
    // SAFETY: caller contract.
    unsafe { debuggy_find(file, fname, 0 as linenr_T, BreakList::Profiling, fp) != 0 as linenr_T }
}

/// The shared body of [`dbg_find_breakpoint`] and [`has_profiling`]: the
/// lowest line above `after` that a name entry matches, or -- for a watch
/// expression whose value just changed -- `after` itself.
///
/// # Safety
/// As [`dbg_find_breakpoint`].
unsafe fn debuggy_find(
    file: bool,
    fname: *mut c_char,
    after: linenr_T,
    list: BreakList,
    fp: *mut bool,
) -> linenr_T {
    if list.is_empty() {
        return 0 as linenr_T;
    }

    // A script-local function arrives with `K_SNR` in front of its name; the
    // patterns are written against the `<SNR>` spelling.
    // SAFETY: caller contract.
    let name = unsafe {
        if !file && *fname as uint8_t as c_int == K_SPECIAL {
            let owned: *mut c_char = xmalloc(cstr::bytes_at(fname).len() + 3).cast();
            strcpy(owned, c"<SNR>".as_ptr());
            strcpy(owned.offset(5), fname.offset(3));
            owned
        } else {
            fname
        }
    };

    let mut lnum = 0 as linenr_T;
    for i in 0..list.len() {
        // SAFETY: `i` is below `ga_len`. Re-read every pass, because a watch
        // expression below can grow the array.
        let bp = unsafe { list.entry(i) };
        // SAFETY: as above.
        let kind = unsafe { (*bp).dbg_type };
        // Skip entries of the wrong kind, and ones for a line beyond a
        // breakpoint already found. Every profiling entry is a candidate:
        // profiling is per file, not per line.
        // SAFETY: as above.
        let candidate = unsafe {
            (kind == DBG_FILE) == file
                && kind != DBG_EXPR
                && (list == BreakList::Profiling
                    || ((*bp).dbg_lnum > after && (lnum == 0 as linenr_T || (*bp).dbg_lnum < lnum)))
        };

        if candidate {
            // A previous interruption must not cancel the match; only a
            // CTRL-C typed while matching should.
            let prev_got_int = got_int.get();
            got_int.set(false);
            // SAFETY: `dbg_prog` is this entry's compiled pattern and `name`
            // is NUL-terminated.
            if unsafe { vim_regexec_prog(&raw mut (*bp).dbg_prog, false, name, 0 as colnr_T) } {
                lnum = unsafe { (*bp).dbg_lnum };
                if !fp.is_null() {
                    unsafe { *fp = (*bp).dbg_forceit != 0 };
                }
            }
            got_int.set(got_int.get() | prev_got_int);
        } else if kind == DBG_EXPR {
            // SAFETY: `bp` is a live watch entry.
            if unsafe { watch_changed(bp) } {
                lnum = if after > 0 as linenr_T {
                    after
                } else {
                    1 as linenr_T
                };
                break;
            }
        }
    }

    if name != fname {
        // SAFETY: the `<SNR>` copy is ours.
        unsafe { xfree(name.cast()) };
    }
    lnum
}

/// Re-evaluate a watch expression and answer whether its value moved,
/// recording the before and after for the prompt banner when it did.
///
/// # Safety
/// `bp` must point at a live `DBG_EXPR` entry.
unsafe fn watch_changed(bp: *mut debuggy) -> bool {
    // SAFETY: caller contract throughout. Evaluating the expression runs
    // arbitrary Vimscript, so `bp` outliving the call rests on the same
    // assumption the C makes -- that a watch does not itself add a
    // breakpoint, which would grow the array and move every entry.
    let tv = unsafe { eval_expr_no_emsg(bp) };
    let previous = unsafe { (*bp).dbg_val };

    if tv.is_null() {
        // The expression stopped evaluating at all, which counts as a
        // change -- but only if there was a value to change from.
        if previous.is_null() {
            return false;
        }
        unsafe { set_oldval(previous) };
        unsafe { set_newval(ptr::null_mut()) };
        unsafe { tv_free(previous) };
        unsafe { (*bp).dbg_val = ptr::null_mut() };
        return true;
    }

    if previous.is_null() {
        // First evaluation: the baseline, with no old value to show.
        unsafe { set_oldval(ptr::null_mut()) };
        unsafe { (*bp).dbg_val = tv };
        unsafe { set_newval(tv) };
        return true;
    }

    // `EXPR_IS` answers "is the same value"; a false answer is a change.
    let changed = unsafe { typval_compare(tv, previous, EXPR_IS, false) }.is_ok()
        && unsafe { (*tv).vval.v_number } == 0;
    if changed {
        // Render the old value before re-evaluating, because evaluating
        // can reach whatever the old value refers to.
        unsafe { set_oldval(previous) };
        // `typval_compare` overwrote `tv`, so the new value has to be
        // evaluated a second time before it can be shown.
        let fresh = unsafe { eval_expr_no_emsg(bp) };
        unsafe { set_newval(fresh) };
        unsafe { tv_free(previous) };
        unsafe { (*bp).dbg_val = fresh };
    }
    unsafe { tv_free(tv) };
    changed
}

/// Record the "before" value the prompt banner prints, freeing whatever an
/// earlier change left. A null typval renders as the empty value.
///
/// # Safety
/// `tv` must be null or a live typval.
unsafe fn set_oldval(tv: *mut typval_T) {
    // SAFETY: caller contract; the cell owns what it holds.
    unsafe { xfree(debug_oldval.get().cast()) };
    debug_oldval.set(unsafe { typval_tostring(tv, true) });
}

/// [`set_oldval`] for the "after" value.
///
/// # Safety
/// As [`set_oldval`].
unsafe fn set_newval(tv: *mut typval_T) {
    // SAFETY: as `set_oldval`.
    unsafe { xfree(debug_newval.get().cast()) };
    debug_newval.set(unsafe { typval_tostring(tv, true) });
}
