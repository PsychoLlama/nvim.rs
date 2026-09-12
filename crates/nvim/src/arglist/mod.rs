//! The argument list: `:args`, `:argadd`, `:argedit`, `:argdelete`,
//! `:argdedupe`, the `:next`/`:previous`/`:first`/`:last` family that walks
//! it, and — in the two submodules — `:all`/`:sall` and the
//! `argc()`/`argidx()`/`arglistid()`/`argv()` builtins.
//!
//! Every window points at an `ArgList`, refcounted so that several windows
//! can share one. [`global_arglist`] is the list nvim starts with, the one
//! `:argglobal` returns a window to; `:arglocal` gives a window a private
//! copy. Entries are `ArgEntry` in a garray, so index arithmetic — a
//! `memmove` to close a hole, a `w_arg_idx` fixup after it — is the shape of
//! most of this file.
//!
//! Almost every step here can run autocommands (`buflist_add`, `do_ecmd`,
//! `win_close`), and an autocommand can run `:args` again. The list is
//! therefore "locked" while it is being changed, and every entry point that
//! changes it refuses to re-enter: see [`arglist_is_locked`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod all;
mod command;
mod eval;
pub(crate) mod state;

use crate::arglist::state::{arg_had_last, global_alist, max_alist_id};
use crate::ascii::ascii_isspace;
use crate::autocmd::is_aucmd_win;
use crate::buffer::{
    buf_hide, buf_is_empty, buf_set_name, buflist_add, curbuf_reusable, find_buf, maketitle,
    otherfile,
};
use crate::cstr;
use crate::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_string,
};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::{autowrite, check_changed};
use crate::ex_docmd::state::cmdmod;
use crate::ex_getln::gotocmdline;
use crate::fileio::file_pat_to_reg_pat;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mark::{setmark, setpcmark};
use crate::memory::{xcalloc, xfree, xstrdup};
use crate::normal::reset_visual_and_resel;
use crate::option::magic_isset;
use crate::option::vars::{p_ea, p_fic, p_tpm};
use crate::os::input::os_breakcheck;
use crate::path::{
    ExpandFlags, expand_wildcards, fix_fname, full_name_save, gen_expand_wildcards, path_fnamecmp,
    path_full_compare,
};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::types::{Failed, *};
use crate::ui::state::Columns;
use crate::undo::buf_is_changed;
use crate::version::list_in_columns;
use crate::window::{
    check_can_set_curbuf_forceit, goto_tabpage_tp, lastwin_nofloating, tabpage_index,
    valid_tabpage, win_close, win_enter, win_move_after, win_split, win_valid,
};
use crate::winlayer::graph::{cmdwin_type, firstwin, lastused_tabpage, lastwin};
use crate::winlayer::{Buf, Ea, Live, Win, tab_windows};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

pub use all::{arg_all, ex_all};
pub use command::{
    do_argfile, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind, get_arglist_name,
};
pub use eval::{f_argc, f_argidx, f_arglistid, f_argv};

/// An argument list — the global one, or a window's own copy — the same
/// promise as [`Ea`]. A list is reference counted and outlives any command
/// walking it.
pub(crate) type Al = Live<ArgList>;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{BlnFlags, FileComparison, c_uint};

    /// `buflist_new` flags.
    pub(super) const BLN_CURBUF: BlnFlags = 1;
    pub(super) const BLN_LISTED: BlnFlags = 2;

    /// `check_changed` flags.
    pub(super) const CCGD_AW: c_uint = 1;
    pub(super) const CCGD_MULTWIN: c_uint = 2;
    pub(super) const CCGD_FORCEIT: c_uint = 4;
    pub(super) const CCGD_EXCMD: c_uint = 16;

    /// `path_full_compare` result bit meaning "the same file".
    pub(super) const kEqualFiles: FileComparison = 1;
}
use flag::*;

/// `flag` when `cond`, and no bits otherwise — the `cond ? FLAG : 0` the
/// flag arguments of `buflist_add`, `check_changed` and `do_ecmd` are
/// assembled from.
fn flag_if(cond: bool, flag: c_uint) -> c_int {
    if cond { flag as c_int } else { 0 }
}

/// An entry count as an index. Every count here came out of the list
/// itself, or out of a range already checked against it.
fn as_count(n: c_int) -> usize {
    usize::try_from(n).expect("an argument count is never negative")
}

// ---------------------------------------------------------------------------
// Reaching the entries.

/// The global argument list — `GARGLIST`/`GARGCOUNT` upstream. It is the one
/// list that is never freed, which is why several tests here compare against
/// its address rather than a flag.
pub(crate) fn global_arglist() -> *mut ArgList {
    global_alist.ptr()
}

/// `AARGLIST(al)` and `ALIST_COUNT(al)`: an argument list's entries and length.
/// The transpiler spelled these out at every use; the C had them as macros.
fn alist_entries(al: *mut ArgList) -> (*mut ArgEntry, c_int) {
    // SAFETY: the caller's promise -- a live `ArgList`. The pointer is
    // derived from the vector itself on every call, so a growth between two
    // calls cannot leave a stale one behind.
    let entries = unsafe { &mut (*al).al_ga };
    (entries.as_mut_ptr(), entries.len() as c_int)
}

fn alist_arg(al: *mut ArgList, n: c_int) -> *mut ArgEntry {
    alist_entries(al).0.wrapping_add(n as usize)
}

fn alist_count(al: *mut ArgList) -> c_int {
    alist_entries(al).1
}

/// `WARGLIST(wp)[n]` and `WARGCOUNT(wp)`: a window's argument list.
///
/// Every window always has one, so this is total.
fn win_alist(window: Win) -> *mut ArgList {
    window.w_alist
}

fn warg(window: Win, n: c_int) -> *mut ArgEntry {
    alist_arg(win_alist(window), n)
}

fn wargcount(window: Win) -> c_int {
    alist_count(win_alist(window))
}

/// `ARGLIST[n]` and `ARGCOUNT`: the current window's argument list.
fn arg(n: c_int) -> *mut ArgEntry {
    warg(Win::current(), n)
}

fn argcount() -> c_int {
    wargcount(Win::current())
}

/// The current window's argument list, for the handful of places that
/// change it in place.
///
/// # Safety
///
/// The borrow must not outlive a call that can replace the window's list.
unsafe fn cur_arglist<'a>() -> &'a mut Vec<ArgEntry> {
    // SAFETY: the current window always has an argument list.
    unsafe { &mut (*win_alist(Win::current())).al_ga }
}

/// `curwin->w_arg_idx`: which argument the current window is on. It is not
/// an index into anything in particular — it can point past the end after a
/// deletion, which is what [`check_arg_idx`] exists to notice.
fn cur_arg_idx() -> c_int {
    // SAFETY: curwin is always valid.
    Win::current().w_arg_idx
}

fn set_cur_arg_idx(idx: c_int) {
    // SAFETY: curwin is always valid.
    Win::current().w_arg_idx = idx;
}

/// `alist_name(ARGLIST + n)`: the n-th argument's file name.
fn arg_name(n: c_int) -> *mut c_char {
    // SAFETY: callers only ask for indexes in range.
    unsafe { alist_name(arg(n)) }
}

/// Do `name` and `fname` resolve to the same file? A buffer with no name of
/// its own (`fname` null) matches nothing.
///
/// # Safety
///
/// `name` must be a NUL-terminated file name and `fname` that or null.
unsafe fn same_file(name: *mut c_char, fname: *mut c_char) -> bool {
    // SAFETY: caller contract; `path_full_compare` only reads both names.
    !fname.is_null()
        && unsafe { path_full_compare(name, fname, true, true) } as c_uint & kEqualFiles as c_uint
            != 0
}

// ---------------------------------------------------------------------------
// Building and tearing down lists.

/// Set while the argument list is being changed and something that might
/// trigger an autocommand is called.
static ARGLIST_LOCKED: GlobalCell<bool> = GlobalCell::new(false);

/// Is the argument list locked against re-entry? Reports E1156 when it is,
/// so callers are just `if arglist_is_locked() { return; }`.
fn arglist_is_locked() -> bool {
    if ARGLIST_LOCKED.get() {
        crate::semsg!("E1156: Cannot change the argument list recursively");
        return true;
    }
    false
}

/// Clear an argument list: free every file name and reset it to no entries.
///
/// # Safety
///
/// `al` must be a valid argument list.
unsafe fn alist_clear(al: *mut ArgList) {
    if arglist_is_locked() {
        return;
    }
    // SAFETY: the caller's promise -- a live `ArgList`, each of whose
    // entries owns its `ae_fname`.
    let entries = unsafe { &mut (*al).al_ga };
    for entry in entries.drain(..) {
        unsafe { xfree(entry.ae_fname.cast()) };
    }
}

/// Empty an argument list, without freeing the names in it -- as upstream's
/// `ga_init` over a live array leaks what it held. Only the global list,
/// which starts out empty, reaches here.
///
/// # Safety
///
/// `al` must be a live `ArgList`.
pub unsafe fn alist_init(al: *mut ArgList) {
    // SAFETY: the caller's promise -- a live `ArgList`.
    unsafe { (*al).al_ga = Vec::new() };
}

/// Drop a reference to an argument list, freeing it once no window holds it.
/// The global list is never freed.
///
/// # Safety
///
/// `al` must be a valid argument list.
pub unsafe fn alist_unlink(al: *mut ArgList) {
    // SAFETY: the caller's promise -- a live `ArgList`.
    let mut al = unsafe { Al::new(al) };
    if ptr::eq(al.raw(), global_arglist()) {
        return;
    }
    // SAFETY: caller contract; the list is ours to free once its last
    // reference goes.
    if al.al_refcount.release() <= 0 {
        unsafe { alist_clear(al.raw()) };
        drop(unsafe { Box::from_raw(al.raw()) });
    }
}

/// Give the current window a fresh, empty argument list of its own.
///
/// Safe: `curwin` is set from startup to exit, and the new list starts out
/// owned by it alone.
fn alist_new() {
    max_alist_id.set(max_alist_id.get() + 1);
    // The new list starts out owned by the current window alone; it is
    // released, and its box reclaimed, by `alist_unlink`.
    Win::current().w_alist = Box::into_raw(Box::new(ArgList {
        al_ga: Vec::new(),
        al_refcount: Refcount::ONE,
        id: max_alist_id.get(),
    }));
}

/// Replace `al`'s entries with `files`, taking over the array and the names
/// in it. `use_curbuf` lets an added name re-use the current buffer;
/// `fnum_list` names buffers previously used for the argument list, so that
/// [`alist_add`] finds and re-uses them.
///
/// # Safety
///
/// `al` must be a valid argument list, `files` an owned array of `count`
/// owned names, and `fnum_list` — when non-null — `fnum_len` buffer numbers.
unsafe fn alist_set(
    al: *mut ArgList,
    count: c_int,
    files: *mut *mut c_char,
    use_curbuf: bool,
    fnum_list: *mut c_int,
    fnum_len: c_int,
) {
    // SAFETY: the caller's promise -- a live `ArgList`.
    let mut al = unsafe { Al::new(al) };
    if arglist_is_locked() {
        return;
    }
    // SAFETY: caller contract; `ga_grow` reserves every slot the loop fills,
    // and each name is handed to the entry that takes it over.
    unsafe { alist_clear(al.raw()) };
    al.al_ga.reserve(as_count(count));
    for i in 0..count {
        if got_int.get() {
            // Adding many buffers can take a long time, so the user can
            // interrupt; the names not yet added are dropped.
            for j in i..count {
                unsafe { xfree(*files.offset(j as isize) as *mut c_void) };
            }
            break;
        }
        if !fnum_list.is_null() && i < fnum_len {
            // Name a buffer previously used for the argument list, so
            // that `alist_add` re-uses it.
            ARGLIST_LOCKED.set(true);
            unsafe { buf_set_name(*fnum_list.offset(i as isize), *files.offset(i as isize)) };
            ARGLIST_LOCKED.set(false);
        }
        let al2 = al.raw();
        let fname2 = unsafe { *files.offset(i as isize) };
        let set_fnum2 = if use_curbuf { 2 } else { 1 };
        unsafe { alist_add(al2, fname2, set_fnum2) };
        os_breakcheck();
    }
    unsafe { xfree(files as *mut c_void) };
    if ptr::eq(al.raw(), global_arglist()) {
        arg_had_last.set(false);
    }
}

/// Append `fname` to `al`, taking over the name. `set_fnum` 1 records the
/// buffer number, 2 additionally lets the current buffer be re-used. May
/// trigger `Buf*` autocommands.
///
/// The entry is built before it joins the list, so the autocommands
/// `buflist_add` may run see the list as it was -- which is what upstream's
/// "fill the slot at `ga_len`, bump `ga_len` afterwards" amounted to.
///
/// # Safety
///
/// `al` must be a valid argument list and `fname` an owned name or null.
pub unsafe fn alist_add(al: *mut ArgList, fname: *mut c_char, set_fnum: c_int) {
    if fname.is_null() {
        // Don't add NULL file names.
        return;
    }
    if arglist_is_locked() {
        return;
    }
    let mut wp = Win::current();
    ARGLIST_LOCKED.set(true);
    wp.w_locked = true;
    // SAFETY: caller contract -- `fname` is NUL-terminated, and the list
    // cannot be changed under us while it is locked.
    let ae_fnum = if set_fnum > 0 {
        let flags = BLN_LISTED as c_int | flag_if(set_fnum == 2, BLN_CURBUF);
        unsafe { buflist_add(fname, flags) }
    } else {
        0
    };
    unsafe {
        (*al).al_ga.push(ArgEntry {
            ae_fname: fname,
            ae_fnum,
        })
    };
    wp.w_locked = false;
    ARGLIST_LOCKED.set(false);
}

/// Terminates the first argument in `s` and answers where the next one starts.
///
/// A backslash escapes the byte after it — both are kept, `alist_add`'s caller
/// strips them later — and a backtick suspends the "whitespace ends the
/// argument" rule, so `` `shell command` `` stays one argument.
///
/// The C walked a read and a write pointer in step and copied every byte over
/// itself; since neither branch ever consumes more bytes than it writes, the
/// only lasting change is the terminator, which lands on the whitespace that
/// ended the argument (or on the string's own NUL, where it is a no-op).
pub fn split_one_arg(s: &mut [u8]) -> usize {
    let mut inbacktick = false;
    let mut end = 0;
    while end < s.len() && s[end] != 0 {
        // `rem_backslash`: a backslash is only an escape if something follows.
        if s[end] == b'\\' && end + 1 < s.len() && s[end + 1] != 0 {
            end += 1;
        } else {
            if !inbacktick && ascii_isspace(s[end] as c_int) {
                break;
            }
            if s[end] == b'`' {
                inbacktick = !inbacktick;
            }
        }
        end += 1;
    }
    // `skipwhite`: space and tab only.
    let mut next = end;
    while next < s.len() && (s[next] == b' ' || s[next] == b'\t') {
        next += 1;
    }
    if end < s.len() {
        s[end] = 0;
    }
    next
}

/// Split `str` into arguments in place, answering a pointer to each. The
/// pointers are into `str` itself, so they live exactly as long as it does.
/// Without `escaped` the whole string is one argument.
///
/// # Safety
///
/// `str` must be a NUL-terminated writable string that outlives the answer.
unsafe fn get_arglist(str: *mut c_char, escaped: bool) -> Vec<*mut c_char> {
    // SAFETY: caller contract. One `strlen` up front rather than one per
    // argument: `split_one_arg` only ever shortens what it is given, so the
    // original length still bounds it.
    let total = unsafe { cstr::bytes_at(str) }.len();
    let buf = unsafe { core::slice::from_raw_parts_mut(str.cast::<u8>(), total + 1) };
    let mut args = Vec::new();
    let mut at = 0;
    while at < total && buf[at] != 0 {
        args.push(unsafe { str.add(at) });
        if !escaped {
            break;
        }
        at += split_one_arg(&mut buf[at..]);
    }
    args
}

/// Split `str` into file names and expand them into `fnamesp[fcountp]`.
/// With `wig`, names matching 'wildignore' are dropped.
///
/// # Safety
///
/// `str` must be a NUL-terminated writable string; `fcountp` and `fnamesp`
/// must be valid out-parameters.
/// Everything a name can be, with a directory marked by its separator and
/// a pattern that matched nothing answered as itself.
const ANY_NAME: ExpandFlags = ExpandFlags::DIR
    .or(ExpandFlags::FILE)
    .or(ExpandFlags::ADDSLASH)
    .or(ExpandFlags::NOTFOUND);

/// # Safety
///
/// `str` must point at a NUL-terminated string, unaliased for the call.
/// `fcountp` must point at a writable `int` the caller owns. `fnamesp` must
/// point at a writable `*mut *mut c_char` slot the caller owns for the call.
pub unsafe fn get_arglist_exp(
    str: *mut c_char,
    fcountp: *mut c_int,
    fnamesp: *mut *mut *mut c_char,
    wig: bool,
) -> Result<(), Failed> {
    const EXPAND: ExpandFlags = ExpandFlags::FILE
        .or(ExpandFlags::NOTFOUND)
        .or(ExpandFlags::NOTWILD);
    // SAFETY: caller contract; the names point into `str`, which outlives
    // the expansion.
    let mut args = unsafe { get_arglist(str, true) };
    let count = args.len() as c_int;
    let names = args.as_mut_ptr();
    if wig {
        unsafe { expand_wildcards(count, names, fcountp, fnamesp, EXPAND) }
    } else {
        unsafe { gen_expand_wildcards(count, names, fcountp, fnamesp, EXPAND) }
    }
}

/// Re-check `w_arg_idx` in every window sharing the current window's list.
///
/// Safe: the tab page and window lists are the editor's own, and
/// `check_arg_idx` only reads and writes `w_arg_idx`.
fn alist_check_arg_idx() {
    let alist = win_alist(Win::current());
    for win in tab_windows().filter(|win| win.w_alist == alist) {
        check_arg_idx(win);
    }
}

/// Insert `files` into the current window's argument list after entry
/// `after`, taking over the names (but not the array itself). `will_edit`
/// says one of them is about to be edited.
///
/// # Safety
///
/// `files` must hold `count` owned names.
unsafe fn alist_add_list(count: c_int, files: *mut *mut c_char, after: c_int, will_edit: bool) {
    let old_argcount = argcount();
    if arglist_is_locked() {
        return;
    }
    let mut wp = Win::current();
    let flags = BLN_LISTED as c_int | flag_if(will_edit, BLN_CURBUF);
    let after = after.clamp(0, argcount());
    ARGLIST_LOCKED.set(true);
    wp.w_locked = true;
    // SAFETY: caller contract -- `files` holds `count` owned names. The
    // entries are built before any of them joins the list, so the
    // autocommands `buflist_add` may run see the list they started with.
    let added: Vec<ArgEntry> = (0..count)
        .map(|i| {
            let name = unsafe { *files.offset(i as isize) };
            ArgEntry {
                ae_fname: name,
                ae_fnum: unsafe { buflist_add(name, flags) },
            }
        })
        .collect();
    ARGLIST_LOCKED.set(false);
    wp.w_locked = false;
    // SAFETY: every window has an argument list.
    let entries = unsafe { &mut (*win_alist(wp)).al_ga };
    let at = as_count(after);
    entries.splice(at..at, added);
    if old_argcount > 0 && wp.w_arg_idx >= after {
        wp.w_arg_idx += count;
    }
}

/// Free entry `idx`'s name and close the gap it leaves. The caller fixes
/// `w_arg_idx` itself: `:argdelete` and `:argdedupe` disagree about where a
/// cursor sitting on the removed entry should land.
fn remove_arg(idx: c_int) {
    // SAFETY: caller contract -- `idx` names an entry, which owns its name.
    let gone = unsafe { cur_arglist() }.remove(as_count(idx));
    unsafe { xfree(gone.ae_fname.cast()) };
}

/// Delete every argument whose name `regmatch` matches, and report whether
/// any did.
///
/// # Safety
///
/// `regmatch` must hold a compiled program.
unsafe fn delete_matching_args(regmatch: *mut RegMatch) -> bool {
    let mut didone = false;
    let mut i = 0;
    while i < argcount() {
        // SAFETY: caller contract; `i` is in range and the entry's name is
        // NUL-terminated.
        if !unsafe { vim_regexec(regmatch, arg_name(i), 0 as ColNr) } {
            i += 1;
            continue;
        }
        didone = true;
        remove_arg(i);
        if cur_arg_idx() > i {
            set_cur_arg_idx(cur_arg_idx() - 1);
        }
    }
    didone
}

/// Delete the arguments matching each file pattern in `patterns`. A pattern
/// that matches nothing reports E480 and the rest still run.
///
/// # Safety
///
/// Every pattern must be NUL-terminated and stay alive for the call.
unsafe fn arglist_del_files(patterns: &[*mut c_char]) {
    let mut regmatch = RegMatch {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        // Ignore case when 'fileignorecase' is set.
        rm_ic: p_fic.get() != 0,
    };
    for &pattern in patterns {
        if got_int.get() {
            break;
        }
        // SAFETY: caller contract -- every pattern is NUL-terminated. The
        // translated pattern and the compiled program are freed on every
        // path out of the body.
        let regexp = unsafe { file_pat_to_reg_pat(pattern, ptr::null(), ptr::null_mut(), 0) };
        if regexp.is_null() {
            break;
        }
        // SAFETY: `regexp` is a NUL-terminated pattern owned here.
        regmatch.regprog = unsafe { vim_regcomp(regexp, if magic_isset() { RE_MAGIC } else { 0 }) };
        if regmatch.regprog.is_null() {
            // SAFETY: `regexp` is ours to free.
            unsafe { xfree(regexp as *mut c_void) };
            break;
        }
        // SAFETY: the program was just compiled and is freed right after.
        // SAFETY: `regmatch` is this frame's and `regexp` is ours to free
        // once the walk that reads it has finished.
        let didone = unsafe { delete_matching_args(&raw mut regmatch) };
        unsafe { vim_regfree(regmatch.regprog) };
        unsafe { xfree(regexp.cast()) };
        if !didone {
            // SAFETY: the pattern is NUL-terminated and still alive.
            let pattern = unsafe { CStr::from_ptr(pattern) }.to_string_lossy();
            crate::semsg!("E480: No match: {pattern}");
        }
    }
}

/// What [`do_arglist`] should do with the names it parses.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ArgListOp {
    /// `:args {file}` — redefine the list.
    Set,
    /// `:argadd`/`:argedit` — insert after a given entry.
    Add,
    /// `:argdelete {pat}` — remove the entries matching each pattern.
    Delete,
}

/// Parse `str` into file names and set, add or delete them. `after` is where
/// [`ArgListOp::Add`] inserts (0 meaning before the first), and `will_edit`
/// says one of the added names is about to be edited.
///
/// # Safety
///
/// `str` must be a NUL-terminated writable command argument.
unsafe fn do_arglist(str: *mut c_char, op: ArgListOp, after: c_int, will_edit: bool) -> bool {
    if arglist_is_locked() {
        return false;
    }
    let mut str = str;
    let mut arg_escaped = true;
    // SAFETY: caller contract; curbuf is valid and its name outlives the
    // expansion below.
    // ":argadd" with no argument adds the current file.
    if op == ArgListOp::Add && unsafe { *str } as c_int == NUL {
        if Buf::current().b_ffname.is_null() {
            return false;
        }
        str = Buf::current().b_fname;
        arg_escaped = false;
    }
    // Collect all the file name arguments.
    // SAFETY: `str` is writable and outlives the names taken out of it.
    let mut new_args = unsafe { get_arglist(str, arg_escaped) };
    if op == ArgListOp::Delete {
        // SAFETY: the names are NUL-terminated patterns pointing into `str`.
        unsafe { arglist_del_files(&new_args) };
    } else {
        let mut exp_count: c_int = 0;
        let mut exp_files: *mut *mut c_char = ptr::null_mut();
        // SAFETY: the expansion reads the collected names and hands back an
        // owned array of owned names, which the arms below take over.
        let count = new_args.len() as c_int;
        let names = new_args.as_mut_ptr();
        let countp = &raw mut exp_count;
        let filesp = &raw mut exp_files;
        let result = unsafe { expand_wildcards(count, names, countp, filesp, ANY_NAME) };
        drop(new_args);
        let expanded = result.is_ok() && exp_count != 0;
        if !expanded {
            crate::semsg!("E479: No match");
            return false;
        }
        // SAFETY: `exp_files` holds `exp_count` owned names.
        if op == ArgListOp::Add {
            unsafe { alist_add_list(exp_count, exp_files, after, will_edit) };
            unsafe { xfree(exp_files as *mut c_void) };
        } else {
            let al2 = win_alist(Win::current());
            let fnum_list2 = ptr::null_mut();
            unsafe { alist_set(al2, exp_count, exp_files, will_edit, fnum_list2, 0) };
        }
    }
    alist_check_arg_idx();
    true
}

/// Redefine the argument list from a command line (start-up's `-p`/`-o`).
///
/// # Safety
///
/// `str` must be a NUL-terminated writable command argument.
pub unsafe fn set_arglist(str: *mut c_char) {
    // SAFETY: caller contract.
    unsafe { do_arglist(str, ArgListOp::Set, 0, true) };
}

// ---------------------------------------------------------------------------
// Where the window sits in its list.

/// Is `win` editing the file at its own argument index?
///
/// # Safety
///
/// Safe: the index is checked against the window's own argument list before
/// anything reads an entry.
pub fn editing_arg_idx(win: Win) -> bool {
    let idx = win.w_arg_idx;
    if idx >= wargcount(win) {
        return false;
    }
    let entry = warg(win, idx);
    let buf = win.buffer();
    // SAFETY: `entry` is in range of the window's list, so it is one of its
    // own entries, and `b_ffname` is that buffer's name or NULL.
    unsafe { buf.handle == (*entry).ae_fnum || same_file(alist_name(entry), buf.b_ffname) }
}

/// Refresh `win`'s "am I on the argument I think I am" state, and remember
/// when the last argument has been reached — `arg_had_last` is how `:next`
/// knows there is nothing after it.
///
/// # Safety
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn check_arg_idx(mut win: Win) {
    let (editing, idx) = (editing_arg_idx(win), win.w_arg_idx);
    if wargcount(win) <= 1 || editing {
        // Editing the current entry: `arg_had_last` if it is the last.
        win.w_arg_idx_invalid = false;
        if idx == wargcount(win) - 1 && win_alist(win) == global_arglist() {
            arg_had_last.set(true);
        }
        return;
    }
    // Not editing the current entry, so `arg_had_last` only if this buffer
    // is the *last* global argument.
    win.w_arg_idx_invalid = true;
    let gcount = alist_count(global_arglist());
    if idx == wargcount(win) - 1
        || arg_had_last.get()
        || win_alist(win) != global_arglist()
        || gcount <= 0
        || idx >= gcount
    {
        return;
    }
    let last = alist_arg(global_arglist(), gcount - 1);
    let buf = win.buffer();
    // SAFETY: `last` is the final entry of the global list, and `b_ffname`
    // is the window's buffer's name or NULL.
    let holds_last =
        unsafe { buf.handle == (*last).ae_fnum || same_file(alist_name(last), buf.b_ffname) };
    if holds_last {
        arg_had_last.set(true);
    }
}

/// The file name of an argument list entry — the associated buffer's name
/// when it has one, since that is what the user renamed it to.
///
/// # Safety
///
/// `aep` must be a valid argument list entry.
pub unsafe fn alist_name(aep: *mut ArgEntry) -> *mut c_char {
    // SAFETY: caller contract; a found buffer outlives this call.
    let bp = find_buf(unsafe { (*aep).ae_fnum });
    match bp.filter(|bp| !bp.b_fname.is_null()) {
        None => unsafe { (*aep).ae_fname },
        Some(bp) => bp.b_fname,
    }
}
