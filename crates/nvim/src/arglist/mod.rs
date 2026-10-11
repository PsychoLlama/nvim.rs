//! The argument list: `:args`, `:argadd`, `:argedit`, `:argdelete`,
//! `:argdedupe`, the `:next`/`:previous`/`:first`/`:last` family that walks
//! it, and — in the two submodules — `:all`/`:sall` and the
//! `argc()`/`argidx()`/`arglistid()`/`argv()` builtins.
//!
//! Every window names an argument list by its [`ArgListId`], and the lists
//! themselves live in the registry in [`state`]: the global one, the one
//! nvim starts with and `:argglobal` returns a window to, and each local one
//! `:arglocal` made, refcounted so that several windows can share it. A list
//! is reached through [`with_list`] for the length of one borrow, and each
//! entry owns its name.
//!
//! Almost every step here can run autocommands (`buflist_add`, `do_ecmd`,
//! `win_close`), and an autocommand can run `:args` again. The list is
//! therefore "locked" while it is being changed, and every entry point that
//! changes it refuses to re-enter: see [`arglist_is_locked`]. What runs
//! user code is handed copies, and a list is found again by its id, and an
//! entry by its index, once the user code has returned.

#![forbid(unsafe_code)]

mod all;
mod command;
mod eval;
pub(crate) mod state;

use crate::arglist::state::{ARG_HAD_LAST, ARG_LISTS, MAX_ALIST_ID};
use crate::ascii::ascii_isspace;
use crate::autocmd::is_aucmd_win;
use crate::buffer::{
    buf_hide, buf_is_empty, buflist_add_name, curbuf_reusable, find_buf, maketitle, otherfile,
};
use crate::eval::typval::{tv_get_number, tv_get_number_chk, tv_list_alloc_ret};
use crate::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::{autowrite, check_changed};
use crate::ex_docmd::state::cmdmod;
use crate::ex_getln::gotocmdline;
use crate::fileio::file_pat_to_regpat;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mark::{setmark, setpcmark};
use crate::memory::XString;
use crate::normal::reset_visual_and_resel;
use crate::option::magic_isset;
use crate::option::vars::{P_EA, p_ea, p_fic, p_tpm};
use crate::os::input::os_breakcheck;
use crate::path::{
    ExpandFlags, expand_wildcards_of, fixed_fname, full_name_of, gen_expand_wildcards_of,
    path_fnamecmp, path_full_compare_of,
};
use crate::regexp::{OwnedMatch, RE_MAGIC, vim_regexec};
use crate::types::{Failed, *};
use crate::ui::state::Columns;
use crate::undo::buf_is_changed;
use crate::version::list_in_columns;
use crate::window::{
    check_can_set_curbuf_forceit, goto_tabpage_tp, lastwin_nofloating, tabpage_index,
    valid_tabpage, win_close, win_enter, win_move_after, win_split, win_valid,
};
use crate::winlayer::graph::{cmdwin_type, firstwin, lastused_tabpage, lastwin};
use crate::winlayer::{Buf, Win, tab_windows};
use core::ffi::{CStr, c_int, c_uint};

pub(crate) use all::arg_all_bytes;
pub use all::ex_all;
pub use command::{
    do_argfile, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind, get_arglist_name,
};
pub use eval::{f_argc, f_argidx, f_arglistid, f_argv};

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
    pub(super) const EQUAL_FILES: FileComparison = 1;
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

/// A list length as the `c_int` the window's index is kept in.
fn count_of(n: usize) -> c_int {
    c_int::try_from(n).unwrap_or(c_int::MAX)
}

// ---------------------------------------------------------------------------
// Reaching the entries.

/// Run `f` over the list `id` names, or answer `None` when no window holds
/// it any more. The registry is lent for the call, so `f` must not reach
/// anything that runs user code or asks for a list itself.
fn with_list<R>(id: ArgListId, f: impl FnOnce(&mut ArgList) -> R) -> Option<R> {
    ARG_LISTS.with_mut(|lists| lists.get_mut(id).map(f))
}

/// `ALIST_COUNT(al)`: how many entries list `id` holds.
pub(crate) fn alist_count(id: ArgListId) -> c_int {
    with_list(id, |al| count_of(al.entries.len())).unwrap_or(0)
}

/// `WARGCOUNT(wp)`: how many files `window`'s argument list holds.
pub(crate) fn wargcount(window: Win) -> c_int {
    alist_count(window.w_alist)
}

/// The current window's argument list.
fn cur_alist() -> ArgListId {
    Win::current().w_alist
}

/// `ARGCOUNT`: how many files the current window's argument list holds.
pub(crate) fn argcount() -> c_int {
    alist_count(cur_alist())
}

/// The buffer number of entry `n` of list `id`, if there is such an entry.
fn entry_fnum(id: ArgListId, n: c_int) -> Option<c_int> {
    let n = usize::try_from(n).ok()?;
    with_list(id, |al| al.entries.get(n).map(|entry| entry.fnum)).flatten()
}

/// The buffer number of entry `n` of `window`'s argument list, if there is
/// such an entry.
pub(crate) fn warg_fnum(window: Win, n: c_int) -> Option<c_int> {
    entry_fnum(window.w_alist, n)
}

/// `alist_name()`: run `f` over the name `entry` shows -- its buffer's name
/// when it has one, since that is what the user renamed it to, and its own
/// otherwise.
fn with_entry_name<R>(entry: &ArgEntry, f: impl FnOnce(&CStr) -> R) -> R {
    match find_buf(entry.fnum).filter(|buf| !buf.name.is_unnamed()) {
        Some(buf) => f(buf.name.shown().expect("a named buffer")),
        None => f(entry.name.as_cstr()),
    }
}

/// A copy of the name entry `n` of list `id` shows, for a caller that runs
/// user code with it or keeps it.
pub(crate) fn arg_name(id: ArgListId, n: c_int) -> Option<XString> {
    let n = usize::try_from(n).ok()?;
    with_list(id, |al| {
        al.entries
            .get(n)
            .map(|entry| with_entry_name(entry, XString::from_cstr))
    })
    .flatten()
}

/// Copies of the names every entry of list `id` shows.
pub(crate) fn arg_names(id: ArgListId) -> Vec<XString> {
    with_list(id, |al| {
        al.entries
            .iter()
            .map(|entry| with_entry_name(entry, XString::from_cstr))
            .collect()
    })
    .unwrap_or_default()
}

/// `curwin->w_arg_idx`: which argument the current window is on. It is not
/// an index into anything in particular — it can point past the end after a
/// deletion, which is what [`check_arg_idx`] exists to notice.
fn cur_arg_idx() -> c_int {
    Win::current().w_arg_idx
}

fn set_cur_arg_idx(idx: c_int) {
    Win::current().w_arg_idx = idx;
}

/// Do `name` and `fname` resolve to the same file? A buffer with no name of
/// its own (`fname` `None`) matches nothing.
fn same_file(name: &CStr, fname: Option<&CStr>) -> bool {
    fname.is_some_and(|fname| path_full_compare_of(name, fname, true, true) & EQUAL_FILES != 0)
}

/// Is `buffer` the file `entry` names -- by number, or by resolving both
/// names?
fn holds_entry(buffer: Buf, entry: &ArgEntry) -> bool {
    buffer.handle == entry.fnum
        || with_entry_name(entry, |name| same_file(name, buffer.name.full()))
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

/// Clear list `id`: drop every entry.
fn alist_clear(id: ArgListId) {
    if arglist_is_locked() {
        return;
    }
    with_list(id, |al| al.entries.clear());
}

/// One more holder of list `id` -- a window that now shares it, or a walk
/// that must keep it alive.
pub(crate) fn alist_retain(id: ArgListId) {
    with_list(id, |al| al.refcount.retain());
}

/// Drop a reference to list `id`, freeing it once nothing holds it. The
/// global list is never freed.
pub fn alist_unlink(id: ArgListId) {
    if id == ArgListId::GLOBAL {
        return;
    }
    if with_list(id, |al| al.refcount.release()).is_some_and(|left| left <= 0) {
        // Upstream clears the list first, and its clear refuses with E1156
        // while the list is locked -- then frees the list anyway.
        let _ = arglist_is_locked();
        let gone = ARG_LISTS.with_mut(|lists| lists.remove(id));
        drop(gone);
    }
}

/// Give the current window a fresh, empty argument list of its own.
fn alist_new() {
    MAX_ALIST_ID.set(MAX_ALIST_ID.get() + 1);
    let id = ArgListId(MAX_ALIST_ID.get());
    let list = ArgList {
        entries: Vec::new(),
        refcount: Refcount::ONE,
    };
    ARG_LISTS.with_mut(|lists| lists.insert(id, list));
    Win::current().w_alist = id;
}

/// Replace list `id`'s entries with `files`. `use_curbuf` lets an added name
/// re-use the current buffer.
fn alist_set(id: ArgListId, files: Vec<XString>, use_curbuf: bool) {
    if arglist_is_locked() {
        return;
    }
    alist_clear(id);
    with_list(id, |al| al.entries.reserve(files.len()));
    for name in files {
        if got_int.get() {
            // Adding many buffers can take a long time, so the user can
            // interrupt; the names not yet added are dropped.
            break;
        }
        alist_add(id, name, if use_curbuf { 2 } else { 1 });
        os_breakcheck();
    }
    if id == ArgListId::GLOBAL {
        ARG_HAD_LAST.set(false);
    }
}

/// Append `name` to list `id`. `set_fnum` 1 records the buffer number, 2
/// additionally lets the current buffer be re-used. May trigger `Buf*`
/// autocommands.
///
/// The entry is built before it joins the list, so the autocommands
/// `buflist_add` may run see the list as it was -- which is what upstream's
/// "fill the slot at `ga_len`, bump `ga_len` afterwards" amounted to. The
/// list is found again by its id afterwards; one that no window holds any
/// more takes nothing.
pub(crate) fn alist_add(id: ArgListId, name: XString, set_fnum: c_int) {
    if arglist_is_locked() {
        return;
    }
    let mut wp = Win::current();
    ARGLIST_LOCKED.set(true);
    wp.w_locked = true;
    let fnum = if set_fnum > 0 {
        let flags = BLN_LISTED as c_int | flag_if(set_fnum == 2, BLN_CURBUF);
        buflist_add_name(Some(name.as_cstr()), 0, flags).map_or(0, |buf| buf.handle)
    } else {
        0
    };
    wp.w_locked = false;
    ARGLIST_LOCKED.set(false);
    with_list(id, |al| al.entries.push(ArgEntry { name, fnum }));
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

/// Split `text` into arguments, each an owned copy -- upstream's
/// `get_arglist`, which cut its string in place. Without `escaped` the whole
/// text is one argument.
fn split_arglist(text: &[u8], escaped: bool) -> Vec<XString> {
    // A terminated copy for the splitter to cut. The text is read as the C
    // string it was: up to a NUL.
    let total = text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len());
    let mut buf = Vec::with_capacity(total + 1);
    buf.extend_from_slice(&text[..total]);
    buf.push(0);
    let mut args = Vec::new();
    let mut at = 0;
    while at < total && buf[at] != 0 {
        let start = at;
        if !escaped {
            args.push(XString::from_bytes(&buf[start..total]));
            break;
        }
        at += split_one_arg(&mut buf[at..]);
        let end = buf[start..]
            .iter()
            .position(|&byte| byte == 0)
            .map_or(buf.len(), |len| start + len);
        args.push(XString::from_bytes(&buf[start..end]));
    }
    args
}

/// Everything a name can be, with a directory marked by its separator and
/// a pattern that matched nothing answered as itself.
const ANY_NAME: ExpandFlags = ExpandFlags::DIR
    .or(ExpandFlags::FILE)
    .or(ExpandFlags::ADDSLASH)
    .or(ExpandFlags::NOTFOUND);

/// Split `text` into file names and expand them -- upstream's
/// `get_arglist_exp`. With `wildignore`, names matching 'wildignore' are
/// dropped and those matching 'suffixes' sorted last.
pub(crate) fn expand_arglist(text: &[u8], wildignore: bool) -> Result<Vec<XString>, Failed> {
    const EXPAND: ExpandFlags = ExpandFlags::FILE
        .or(ExpandFlags::NOTFOUND)
        .or(ExpandFlags::NOTWILD);
    let args = split_arglist(text, true);
    if wildignore {
        expand_wildcards_of(&args, EXPAND)
    } else {
        gen_expand_wildcards_of(&args, EXPAND)
    }
}

/// Re-check `w_arg_idx` in every window sharing the current window's list.
fn alist_check_arg_idx() {
    let alist = cur_alist();
    for win in tab_windows().filter(|win| win.w_alist == alist) {
        check_arg_idx(win);
    }
}

/// Insert `files` into the current window's argument list after entry
/// `after`. `will_edit` says one of them is about to be edited.
fn alist_add_list(files: Vec<XString>, after: c_int, will_edit: bool) {
    let old_argcount = argcount();
    if arglist_is_locked() {
        return;
    }
    let mut wp = Win::current();
    let flags = BLN_LISTED as c_int | flag_if(will_edit, BLN_CURBUF);
    let after = after.clamp(0, argcount());
    let count = count_of(files.len());
    ARGLIST_LOCKED.set(true);
    wp.w_locked = true;
    // The entries are built before any of them joins the list, so the
    // autocommands `buflist_add` may run see the list they started with.
    let added: Vec<ArgEntry> = files
        .into_iter()
        .map(|name| {
            let fnum = buflist_add_name(Some(name.as_cstr()), 0, flags).map_or(0, |buf| buf.handle);
            ArgEntry { name, fnum }
        })
        .collect();
    ARGLIST_LOCKED.set(false);
    wp.w_locked = false;
    // The window's list, found again now that the autocommands are done.
    let at = as_count(after);
    with_list(wp.w_alist, |al| {
        let at = at.min(al.entries.len());
        al.entries.splice(at..at, added);
    });
    if old_argcount > 0 && wp.w_arg_idx >= after {
        wp.w_arg_idx += count;
    }
}

/// Drop entry `idx` and close the gap it leaves. The caller fixes
/// `w_arg_idx` itself: `:argdelete` and `:argdedupe` disagree about where a
/// cursor sitting on the removed entry should land.
fn remove_arg(idx: c_int) {
    let idx = as_count(idx);
    with_list(cur_alist(), |al| al.entries.remove(idx));
}

/// Delete every argument whose name `regex_match` matches, and report
/// whether any did.
fn delete_matching_args(regex_match: &mut RegMatch) -> bool {
    let mut didone = false;
    let mut i = 0;
    while let Some(name) = arg_name(cur_alist(), i) {
        if !vim_regexec(regex_match, name.as_cstr(), 0) {
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
fn arglist_del_files(patterns: &[XString]) {
    for pattern in patterns {
        if got_int.get() {
            break;
        }
        let Some(regexp) = file_pat_to_regpat(pattern.as_cstr()) else {
            break;
        };
        let re_flags = if magic_isset() { RE_MAGIC } else { 0 };
        // Ignore case when 'fileignorecase' is set.
        let Some(mut regmatch) = OwnedMatch::compile(regexp.as_cstr(), re_flags, p_fic()) else {
            break;
        };
        if !delete_matching_args(&mut regmatch) {
            let pattern = pattern.as_cstr().to_string_lossy();
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

/// Parse `arg` into file names and set, add or delete them. `after` is where
/// [`ArgListOp::Add`] inserts (0 meaning before the first), and `will_edit`
/// says one of the added names is about to be edited.
fn do_arglist(arg: &[u8], op: ArgListOp, after: c_int, will_edit: bool) -> bool {
    if arglist_is_locked() {
        return false;
    }
    // ":argadd" with no argument adds the current file, unsplit.
    let new_args = if op == ArgListOp::Add && arg.first().is_none_or(|&byte| byte == 0) {
        let buffer = Buf::current();
        let (Some(_), Some(shown)) = (buffer.name.full(), buffer.name.shown()) else {
            return false;
        };
        split_arglist(shown.to_bytes(), false)
    } else {
        split_arglist(arg, true)
    };
    if op == ArgListOp::Delete {
        arglist_del_files(&new_args);
    } else {
        let files = match expand_wildcards_of(&new_args, ANY_NAME) {
            Ok(files) if !files.is_empty() => files,
            _ => {
                crate::semsg!("E479: No match");
                return false;
            }
        };
        if op == ArgListOp::Add {
            alist_add_list(files, after, will_edit);
        } else {
            alist_set(cur_alist(), files, will_edit);
        }
    }
    alist_check_arg_idx();
    true
}

/// Redefine the current window's argument list from a command argument
/// (`:drop`'s, start-up's `-p`/`-o`).
pub(crate) fn set_arglist(arg: &[u8]) {
    do_arglist(arg, ArgListOp::Set, 0, true);
}

// ---------------------------------------------------------------------------
// Where the window sits in its list.

/// Is `win` editing the file at its own argument index?
pub fn editing_arg_idx(win: Win) -> bool {
    let Ok(idx) = usize::try_from(win.w_arg_idx) else {
        return false;
    };
    let buf = win.buffer();
    with_list(win.w_alist, |al| {
        al.entries
            .get(idx)
            .is_some_and(|entry| holds_entry(buf, entry))
    })
    .unwrap_or(false)
}

/// Refresh `win`'s "am I on the argument I think I am" state, and remember
/// when the last argument has been reached — `ARG_HAD_LAST` is how `:next`
/// knows there is nothing after it.
pub fn check_arg_idx(mut win: Win) {
    let (editing, idx) = (editing_arg_idx(win), win.w_arg_idx);
    if wargcount(win) <= 1 || editing {
        // Editing the current entry: `ARG_HAD_LAST` if it is the last.
        win.w_arg_idx_invalid = false;
        if idx == wargcount(win) - 1 && win.w_alist == ArgListId::GLOBAL {
            ARG_HAD_LAST.set(true);
        }
        return;
    }
    // Not editing the current entry, so `ARG_HAD_LAST` only if this buffer
    // is the *last* global argument.
    win.w_arg_idx_invalid = true;
    let gcount = alist_count(ArgListId::GLOBAL);
    if idx == wargcount(win) - 1
        || ARG_HAD_LAST.get()
        || win.w_alist != ArgListId::GLOBAL
        || gcount <= 0
        || idx >= gcount
    {
        return;
    }
    let buf = win.buffer();
    let holds_last = with_list(ArgListId::GLOBAL, |al| {
        al.entries
            .last()
            .is_some_and(|entry| holds_entry(buf, entry))
    });
    if holds_last == Some(true) {
        ARG_HAD_LAST.set(true);
    }
}
