//! The argument list: `:args`, `:argadd`, `:argedit`, `:argdelete`,
//! `:argdedupe`, the `:next`/`:previous`/`:first`/`:last` family that walks
//! it, and — in the two submodules — `:all`/`:sall` and the
//! `argc()`/`argidx()`/`arglistid()`/`argv()` builtins.
//!
//! Every window points at an `alist_T`, refcounted so that several windows
//! can share one. [`global_arglist`] is the list nvim starts with, the one
//! `:argglobal` returns a window to; `:arglocal` gives a window a private
//! copy. Entries are `aentry_T` in a garray, so index arithmetic — a
//! `memmove` to close a hole, a `w_arg_idx` fixup after it — is the shape of
//! most of this file.
//!
//! Almost every step here can run autocommands (`buflist_add`, `do_ecmd`,
//! `win_close`), and an autocommand can run `:args` again. The list is
//! therefore "locked" while it is being changed, and every entry point that
//! changes it refuses to re-enter: see [`arglist_is_locked`].

#![deny(unsafe_op_in_unsafe_fn)]

mod all;
mod command;
mod eval;

use crate::ascii::ascii_isspace;
use crate::autocmd::is_aucmd_win;
use crate::buffer::{
    buf_hide, buf_is_empty, buf_set_name, buflist_add, buflist_findnr, bufref_valid,
    curbuf_reusable, maketitle, otherfile, set_bufref,
};
use crate::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_string,
};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::{autowrite, check_changed};
use crate::ex_getln::gotocmdline;
use crate::fileio::file_pat_to_reg_pat;
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{
    Columns, arg_had_last, autocmd_no_enter, autocmd_no_leave, cmdmod, cmdwin_type, curbuf, curtab,
    curwin, first_tabpage, firstwin, global_alist, got_int, lastused_tabpage, lastwin,
    max_alist_id, p_ea, p_fic, p_tpm, tabpage_move_disallowed,
};
use crate::mark::{setmark, setpcmark};
use crate::memory::{xcalloc, xfree, xmalloc, xstrdup};
use crate::normal::reset_VIsual_and_resel;
use crate::option::magic_isset;
use crate::os::cshim::memmove;
use crate::os::input::os_breakcheck;
use crate::path::{
    ExpandFlags, expand_wildcards, fix_fname, full_name_save, gen_expand_wildcards, path_fnamecmp,
    path_full_compare,
};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::types::{CMD_argdo, CMD_argglobal, CMD_arglocal, CMD_args, CMD_snext, *};
use crate::undo::bufIsChanged;
use crate::version::list_in_columns;
use crate::window::{
    check_can_set_curbuf_forceit, goto_tabpage_tp, lastwin_nofloating, tabpage_index,
    valid_tabpage, win_close, win_enter, win_move_after, win_split, win_valid,
};
use ::libc::strlen;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

pub use all::{arg_all, ex_all};
pub use command::{
    do_argfile, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind, get_arglist_name,
};
pub use eval::{f_argc, f_argidx, f_arglistid, f_argv};

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{bln_values, c_int, c_uint, file_comparison};

    /// `buflist_new` flags.
    pub const BLN_CURBUF: bln_values = 1;
    pub const BLN_LISTED: bln_values = 2;

    /// `check_changed` flags.
    pub const CCGD_AW: c_uint = 1;
    pub const CCGD_MULTWIN: c_uint = 2;
    pub const CCGD_FORCEIT: c_uint = 4;
    pub const CCGD_EXCMD: c_uint = 16;

    /// `do_ecmd` flags and its `lnum` sentinels.
    pub const ECMD_HIDE: c_uint = 1;
    pub const ECMD_OLDBUF: c_uint = 4;
    pub const ECMD_FORCEIT: c_uint = 8;
    pub const ECMD_ONE: c_int = 1;
    pub const ECMD_LAST: c_int = -1;

    /// `path_full_compare` result bit meaning "the same file".
    pub const kEqualFiles: file_comparison = 1;
}
use flag::*;

pub const ML_EMPTY: c_int = 0x1;

/// `flag` when `cond`, and no bits otherwise — the `cond ? FLAG : 0` the
/// flag arguments of `buflist_add`, `check_changed` and `do_ecmd` are
/// assembled from.
fn flag_if(cond: bool, flag: c_uint) -> c_int {
    if cond { flag as c_int } else { 0 }
}

/// A zeroed garray, ready for `ga_init`.
const EMPTY_GARRAY: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ptr::null_mut(),
};

// ---------------------------------------------------------------------------
// Reaching the entries.

/// The global argument list — `GARGLIST`/`GARGCOUNT` upstream. It is the one
/// list that is never freed, which is why several tests here compare against
/// its address rather than a flag.
fn global_arglist() -> *mut alist_T {
    global_alist.ptr()
}

/// `AARGLIST(al)` and `ALIST_COUNT(al)`: an argument list's entries and length.
/// The transpiler spelled these out at every use; the C had them as macros.
fn alist_entries(al: *mut alist_T) -> (*mut aentry_T, c_int) {
    // SAFETY: every caller holds a valid argument list.
    let ga = unsafe { &(*al).al_ga };
    (ga.ga_data as *mut aentry_T, ga.ga_len)
}

fn alist_arg(al: *mut alist_T, n: c_int) -> *mut aentry_T {
    alist_entries(al).0.wrapping_add(n as usize)
}

fn alist_count(al: *mut alist_T) -> c_int {
    alist_entries(al).1
}

/// `WARGLIST(wp)[n]` and `WARGCOUNT(wp)`: a window's argument list.
fn win_alist(wp: *mut win_T) -> *mut alist_T {
    // SAFETY: every window always has an argument list.
    unsafe { (*wp).w_alist }
}

fn warg(wp: *mut win_T, n: c_int) -> *mut aentry_T {
    alist_arg(win_alist(wp), n)
}

fn wargcount(wp: *mut win_T) -> c_int {
    alist_count(win_alist(wp))
}

/// `ARGLIST[n]` and `ARGCOUNT`: the current window's argument list.
fn arg(n: c_int) -> *mut aentry_T {
    warg(curwin.get(), n)
}

fn argcount() -> c_int {
    wargcount(curwin.get())
}

/// `ARGCOUNT` as an assignable place.
fn set_argcount(count: c_int) {
    // SAFETY: the current window always has an argument list.
    unsafe { (*win_alist(curwin.get())).al_ga.ga_len = count };
}

/// `curwin->w_arg_idx`: which argument the current window is on. It is not
/// an index into anything in particular — it can point past the end after a
/// deletion, which is what [`check_arg_idx`] exists to notice.
fn cur_arg_idx() -> c_int {
    // SAFETY: curwin is always valid.
    unsafe { (*curwin.get()).w_arg_idx }
}

fn set_cur_arg_idx(idx: c_int) {
    // SAFETY: curwin is always valid.
    unsafe { (*curwin.get()).w_arg_idx = idx };
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
unsafe fn alist_clear(al: *mut alist_T) {
    if arglist_is_locked() {
        return;
    }
    // SAFETY: caller contract; each entry owns its `ae_fname`.
    unsafe {
        let (entries, len) = alist_entries(al);
        if !entries.is_null() {
            for i in 0..len {
                xfree((*entries.offset(i as isize)).ae_fname as *mut c_void);
            }
        }
        ga_clear(&raw mut (*al).al_ga);
    }
}

/// Initialise an argument list to no entries.
///
/// # Safety
///
/// `al` must point at storage for an `alist_T`.
pub unsafe fn alist_init(al: *mut alist_T) {
    // SAFETY: caller contract.
    unsafe { ga_init(&raw mut (*al).al_ga, size_of::<aentry_T>() as c_int, 5) };
}

/// Drop a reference to an argument list, freeing it once no window holds it.
/// The global list is never freed.
///
/// # Safety
///
/// `al` must be a valid argument list.
pub unsafe fn alist_unlink(al: *mut alist_T) {
    if al == global_arglist() {
        return;
    }
    // SAFETY: caller contract; the list is ours to free once its last
    // reference goes.
    unsafe {
        (*al).al_refcount -= 1;
        if (*al).al_refcount <= 0 {
            alist_clear(al);
            xfree(al as *mut c_void);
        }
    }
}

/// Give the current window a fresh, empty argument list of its own.
unsafe fn alist_new() {
    max_alist_id.set(max_alist_id.get() + 1);
    // SAFETY: curwin is valid; the new list starts out owned by it alone.
    unsafe {
        let al = xmalloc(size_of::<alist_T>()) as *mut alist_T;
        (*curwin.get()).w_alist = al;
        (*al).al_refcount = 1;
        (*al).id = max_alist_id.get();
        alist_init(al);
    }
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
    al: *mut alist_T,
    count: c_int,
    files: *mut *mut c_char,
    use_curbuf: bool,
    fnum_list: *mut c_int,
    fnum_len: c_int,
) {
    if arglist_is_locked() {
        return;
    }
    // SAFETY: caller contract; `ga_grow` reserves every slot the loop fills,
    // and each name is handed to the entry that takes it over.
    unsafe {
        alist_clear(al);
        ga_grow(&raw mut (*al).al_ga, count);
        for i in 0..count {
            if got_int.get() {
                // Adding many buffers can take a long time, so the user can
                // interrupt; the names not yet added are dropped.
                for j in i..count {
                    xfree(*files.offset(j as isize) as *mut c_void);
                }
                break;
            }
            if !fnum_list.is_null() && i < fnum_len {
                // Name a buffer previously used for the argument list, so
                // that `alist_add` re-uses it.
                ARGLIST_LOCKED.set(true);
                buf_set_name(*fnum_list.offset(i as isize), *files.offset(i as isize));
                ARGLIST_LOCKED.set(false);
            }
            alist_add(
                al,
                *files.offset(i as isize),
                if use_curbuf { 2 } else { 1 },
            );
            os_breakcheck();
        }
        xfree(files as *mut c_void);
    }
    if al == global_arglist() {
        arg_had_last.set(false);
    }
}

/// Append `fname` to `al`, taking over the name. The caller must have grown
/// the list. `set_fnum` 1 records the buffer number, 2 additionally lets the
/// current buffer be re-used. May trigger `Buf*` autocommands.
///
/// # Safety
///
/// `al` must be a valid argument list with room for one more entry, and
/// `fname` an owned name or null.
pub unsafe fn alist_add(al: *mut alist_T, fname: *mut c_char, set_fnum: c_int) {
    if fname.is_null() {
        // Don't add NULL file names.
        return;
    }
    if arglist_is_locked() {
        return;
    }
    let wp = curwin.get();
    ARGLIST_LOCKED.set(true);
    // SAFETY: caller contract; the slot at `ga_len` is the room the caller
    // grew, and `buflist_add` cannot move the list while it is locked.
    unsafe {
        (*wp).w_locked = true;
        let at = (*al).al_ga.ga_len;
        (*alist_arg(al, at)).ae_fname = fname;
        if set_fnum > 0 {
            let flags = BLN_LISTED as c_int | flag_if(set_fnum == 2, BLN_CURBUF);
            (*alist_arg(al, at)).ae_fnum = buflist_add(fname, flags);
        }
        (*al).al_ga.ga_len += 1;
        (*wp).w_locked = false;
    }
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

/// Split `str` into arguments in place, collecting a pointer to each in
/// `gap`. Without `escaped` the whole string is one argument.
///
/// # Safety
///
/// `str` must be a NUL-terminated writable string that outlives `gap`, and
/// `gap` uninitialised storage for a garray.
unsafe fn get_arglist(gap: *mut garray_T, str: *mut c_char, escaped: bool) {
    // SAFETY: caller contract. One `strlen` up front rather than one per
    // argument: `split_one_arg` only ever shortens what it is given, so the
    // original length still bounds it.
    unsafe {
        ga_init(gap, size_of::<*mut c_char>() as c_int, 20);
        let total = strlen(str) as usize;
        let buf = core::slice::from_raw_parts_mut(str.cast::<u8>(), total + 1);
        let mut at = 0;
        while at < total && buf[at] != 0 {
            ga_grow(gap, 1);
            *((*gap).ga_data as *mut *mut c_char).offset((*gap).ga_len as isize) = str.add(at);
            (*gap).ga_len += 1;
            if !escaped {
                return;
            }
            at += split_one_arg(&mut buf[at..]);
        }
    }
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

pub unsafe fn get_arglist_exp(
    str: *mut c_char,
    fcountp: *mut c_int,
    fnamesp: *mut *mut *mut c_char,
    wig: bool,
) -> c_int {
    const EXPAND: ExpandFlags = ExpandFlags::FILE
        .or(ExpandFlags::NOTFOUND)
        .or(ExpandFlags::NOTWILD);
    let mut ga = EMPTY_GARRAY;
    // SAFETY: caller contract; `ga` holds pointers into `str`, which outlives
    // the expansion, and the garray is cleared before returning.
    unsafe {
        get_arglist(&raw mut ga, str, true);
        let names = ga.ga_data as *mut *mut c_char;
        let result = if wig {
            expand_wildcards(ga.ga_len, names, fcountp, fnamesp, EXPAND)
        } else {
            gen_expand_wildcards(ga.ga_len, names, fcountp, fnamesp, EXPAND)
        };
        ga_clear(&raw mut ga);
        result
    }
}

/// Re-check `w_arg_idx` in every window sharing the current window's list.
unsafe fn alist_check_arg_idx() {
    let alist = win_alist(curwin.get());
    let mut tp = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        // SAFETY: the tab page and window lists are well formed, and
        // `check_arg_idx` does not change them.
        tp = unsafe {
            let mut win = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !win.is_null() {
                if (*win).w_alist == alist {
                    check_arg_idx(win);
                }
                win = (*win).w_next;
            }
            (*tp).tp_next as *mut tabpage_T
        };
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
    let wp = curwin.get();
    let flags = BLN_LISTED as c_int | flag_if(will_edit, BLN_CURBUF);
    // SAFETY: `after` is clamped into the list, the `memmove` opens exactly
    // the `count` slots `ga_grow` reserved, and the list cannot move while it
    // is locked.
    unsafe {
        ga_grow(&raw mut (*win_alist(wp)).al_ga, count);
        let after = after.clamp(0, argcount());
        if after < argcount() {
            memmove(
                arg(after + count) as *mut c_void,
                arg(after) as *const c_void,
                ((argcount() - after) as size_t).wrapping_mul(size_of::<aentry_T>()),
            );
        }
        ARGLIST_LOCKED.set(true);
        (*wp).w_locked = true;
        for i in 0..count {
            let name = *files.offset(i as isize);
            (*arg(after + i)).ae_fname = name;
            (*arg(after + i)).ae_fnum = buflist_add(name, flags);
        }
        ARGLIST_LOCKED.set(false);
        (*wp).w_locked = false;
        (*win_alist(wp)).al_ga.ga_len += count;
        if old_argcount > 0 && (*wp).w_arg_idx >= after {
            (*wp).w_arg_idx += count;
        }
    }
}

/// Free entry `idx`'s name and close the gap it leaves. The caller fixes
/// `w_arg_idx` itself: `:argdelete` and `:argdedupe` disagree about where a
/// cursor sitting on the removed entry should land.
///
/// # Safety
///
/// `idx` must be an entry of the current window's argument list.
unsafe fn remove_arg(idx: c_int) {
    // SAFETY: caller contract; the tail being moved down is
    // `argcount() - idx - 1` entries long.
    unsafe {
        xfree((*arg(idx)).ae_fname as *mut c_void);
        memmove(
            arg(idx) as *mut c_void,
            arg(idx + 1) as *const c_void,
            ((argcount() - idx - 1) as size_t).wrapping_mul(size_of::<aentry_T>()),
        );
    }
    set_argcount(argcount() - 1);
}

/// Delete every argument whose name `regmatch` matches, and report whether
/// any did.
///
/// # Safety
///
/// `regmatch` must hold a compiled program.
unsafe fn delete_matching_args(regmatch: *mut regmatch_T) -> bool {
    let mut didone = false;
    let mut i = 0;
    while i < argcount() {
        // SAFETY: caller contract; `i` is in range and the entry's name is
        // NUL-terminated.
        if !unsafe { vim_regexec(regmatch, arg_name(i), 0 as colnr_T) } {
            i += 1;
            continue;
        }
        didone = true;
        // SAFETY: `i` is in range; the removal shifts the tail down one
        // slot, so `i` stays put to re-examine what moved into it.
        unsafe { remove_arg(i) };
        if cur_arg_idx() > i {
            set_cur_arg_idx(cur_arg_idx() - 1);
        }
    }
    didone
}

/// Delete the arguments matching each file pattern in `alist_ga`. A pattern
/// that matches nothing reports E480 and the rest still run.
///
/// # Safety
///
/// `alist_ga` must be a garray of NUL-terminated patterns.
unsafe fn arglist_del_files(alist_ga: *mut garray_T) {
    let mut regmatch = regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        // Ignore case when 'fileignorecase' is set.
        rm_ic: p_fic.get() != 0,
    };
    // SAFETY: caller contract; the garray holds `ga_len` NUL-terminated
    // patterns, and nothing below changes it.
    let (patterns, len) = unsafe { ((*alist_ga).ga_data as *mut *mut c_char, (*alist_ga).ga_len) };
    let mut i = 0;
    while i < len && !got_int.get() {
        // SAFETY: `i` is in range; the translated pattern and the compiled
        // program are freed on every path out of the body.
        let (pattern, regexp) = unsafe {
            let pattern = *patterns.offset(i as isize);
            (
                pattern,
                file_pat_to_reg_pat(pattern, ptr::null(), ptr::null_mut(), 0),
            )
        };
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
        let didone = unsafe {
            let didone = delete_matching_args(&raw mut regmatch);
            vim_regfree(regmatch.regprog);
            xfree(regexp as *mut c_void);
            didone
        };
        if !didone {
            // SAFETY: the pattern is NUL-terminated and still alive.
            let pattern = unsafe { CStr::from_ptr(pattern).to_string_lossy() };
            crate::semsg!("E480: No match: {pattern}");
        }
        i += 1;
    }
    // SAFETY: caller contract; the garray's items point into the caller's
    // own string, so only the array itself is freed.
    unsafe { ga_clear(alist_ga) };
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
    unsafe {
        // ":argadd" with no argument adds the current file.
        if op == ArgListOp::Add && *str as c_int == NUL {
            if (*curbuf.get()).b_ffname.is_null() {
                return false;
            }
            str = (*curbuf.get()).b_fname;
            arg_escaped = false;
        }
    }
    // Collect all the file name arguments.
    let mut new_ga = EMPTY_GARRAY;
    // SAFETY: `str` is writable and outlives `new_ga`.
    unsafe { get_arglist(&raw mut new_ga, str, arg_escaped) };
    if op == ArgListOp::Delete {
        // SAFETY: `new_ga` holds NUL-terminated patterns pointing into `str`.
        unsafe { arglist_del_files(&raw mut new_ga) };
    } else {
        let mut exp_count: c_int = 0;
        let mut exp_files: *mut *mut c_char = ptr::null_mut();
        // SAFETY: the expansion reads `new_ga`'s names and hands back an
        // owned array of owned names, which the arms below take over.
        let expanded = unsafe {
            let result = expand_wildcards(
                new_ga.ga_len,
                new_ga.ga_data as *mut *mut c_char,
                &raw mut exp_count,
                &raw mut exp_files,
                ANY_NAME,
            );
            ga_clear(&raw mut new_ga);
            result != FAIL && exp_count != 0
        };
        if !expanded {
            crate::semsg!("E479: No match");
            return false;
        }
        // SAFETY: `exp_files` holds `exp_count` owned names.
        unsafe {
            if op == ArgListOp::Add {
                alist_add_list(exp_count, exp_files, after, will_edit);
                xfree(exp_files as *mut c_void);
            } else {
                alist_set(
                    win_alist(curwin.get()),
                    exp_count,
                    exp_files,
                    will_edit,
                    ptr::null_mut(),
                    0,
                );
            }
        }
    }
    // SAFETY: walks the tab page and window lists.
    unsafe { alist_check_arg_idx() };
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
/// `win` must be a valid window.
pub unsafe fn editing_arg_idx(win: *mut win_T) -> bool {
    // SAFETY: caller contract; the window is valid.
    let idx = unsafe { (*win).w_arg_idx };
    if idx >= wargcount(win) {
        return false;
    }
    let entry = warg(win, idx);
    // SAFETY: `entry` is in range and the window's buffer is valid.
    unsafe {
        let buf = (*win).w_buffer;
        (*buf).handle == (*entry).ae_fnum || same_file(alist_name(entry), (*buf).b_ffname)
    }
}

/// Refresh `win`'s "am I on the argument I think I am" state, and remember
/// when the last argument has been reached — `arg_had_last` is how `:next`
/// knows there is nothing after it.
///
/// # Safety
///
/// `win` must be a valid window.
pub unsafe fn check_arg_idx(win: *mut win_T) {
    // SAFETY: caller contract; the window's buffer and list are valid.
    let (editing, idx) = unsafe { (editing_arg_idx(win), (*win).w_arg_idx) };
    if wargcount(win) <= 1 || editing {
        // Editing the current entry: `arg_had_last` if it is the last.
        // SAFETY: caller contract.
        unsafe { (*win).w_arg_idx_invalid = c_int::from(false) };
        if idx == wargcount(win) - 1 && win_alist(win) == global_arglist() {
            arg_had_last.set(true);
        }
        return;
    }
    // Not editing the current entry, so `arg_had_last` only if this buffer
    // is the *last* global argument.
    // SAFETY: caller contract.
    unsafe { (*win).w_arg_idx_invalid = c_int::from(true) };
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
    // SAFETY: `last` is the final entry of the global list, and the window's
    // buffer is valid.
    let holds_last = unsafe {
        let buf = (*win).w_buffer;
        (*buf).handle == (*last).ae_fnum || same_file(alist_name(last), (*buf).b_ffname)
    };
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
pub unsafe fn alist_name(aep: *mut aentry_T) -> *mut c_char {
    // SAFETY: caller contract; a found buffer outlives this call.
    unsafe {
        let bp = buflist_findnr((*aep).ae_fnum);
        if bp.is_null() || (*bp).b_fname.is_null() {
            (*aep).ae_fname
        } else {
            (*bp).b_fname
        }
    }
}
