//! Finding an entry, and resolving the file it names.
//!
//! [`qf_get_fnum`] turns the file name a parsed entry carries into a buffer
//! number. A relative name is resolved against the [`DirStack`] that
//! `%D`/`%X` maintain — and, when that directory turns out to be wrong,
//! against the rest of the stack ([`qf_guess_filepath`]), because `make`
//! can change directory without printing a message about it.
//!
//! The `*_valid_entry` walkers are how `:cnext` and friends skip entries
//! that name no real position, and [`qf_get_nth_valid_entry`] and the
//! `qf_get_*_idx` pair are what `:cdo`/`:cfdo` count with. They walk a
//! `&QfList` and answer positions: nothing they do can run user code.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::{BufRef, buflist_add_name};
use crate::file_search::Name;
use crate::memory::XString;
use crate::os::fs::{dir_exists, path_exists};
use crate::path::join_fnames;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_int};

/// The file name the previous entry was filed under, and the buffer it
/// named. Consecutive entries usually name the same file, so remembering
/// the last answer saves a `buflist_new` lookup per entry.
static LAST_BUFNAME: GlobalCell<Option<Name>> = GlobalCell::new(None);
static LAST_BUFREF: GlobalCell<BufRef> = GlobalCell::new(BufRef::NONE);

/// Throw the cache away. The buffer it names may have been wiped out since,
/// and a stale hit would file entries under a dead buffer.
pub(crate) fn forget_last_buffer() {
    LAST_BUFNAME.with_mut(|name| *name = None);
}

/// The buffer for `bufname`, from the cache or freshly listed.
fn buffer_for(bufname: &CStr) -> Option<Buf> {
    let cached = LAST_BUFNAME.with(|name| {
        name.as_ref()
            .is_some_and(|name| name.bytes() == bufname.to_bytes())
    });
    if cached && LAST_BUFREF.get().valid() {
        return LAST_BUFREF.get().get();
    }
    let buf = buflist_add_name(Some(bufname), 0, BLN_NOOPT.cast_signed());
    LAST_BUFNAME.with_mut(|slot| *slot = Some(Name::from_bytes(bufname.to_bytes())));
    LAST_BUFREF.set(BufRef::of_opt(buf));
    buf
}

/// The buffer number for the file an entry names, listing the buffer if it
/// is not listed yet. Answers 0 when the entry names no file.
///
/// Listing a buffer fires `BufNew`, so `qfl` is a view, and `directory` and
/// `fname` must not borrow from the list.
pub(crate) fn qf_get_fnum(qfl: Qfl, directory: Option<&CStr>, fname: Option<&CStr>) -> c_int {
    let Some(fname) = fname.filter(|fname| !fname.is_empty()) else {
        return 0;
    };
    let joined;
    let bufname = match directory {
        Some(dir) if !vim_is_abs_name(fname) => {
            let mut path = join_fnames(dir, fname, true);
            // The file should be there. If it is not, `make` changed
            // directory without a "leaving directory" message and the
            // directory stack has to be re-guessed.
            if !path_exists(path.as_cstr()) {
                path = match qf_guess_filepath(qfl, fname) {
                    Some(guess) => join_fnames(guess.as_cstr(), fname, true),
                    None => XString::from_cstr(fname),
                };
            }
            joined = path;
            joined.as_cstr()
        }
        _ => fname,
    };

    let kind = qfl.kind;
    let Some(mut buf) = buffer_for(bufname) else {
        return 0;
    };
    buf.b_has_qf_entry = has_entry_flag(kind);
    buf.handle as c_int
}

impl DirStack {
    /// Find, searching down from just below the top, the first directory
    /// `wanted` accepts, and drop everything between it and the top — those
    /// are directories the output has already left. Nothing accepted means
    /// the whole stack below the top goes.
    ///
    /// Answers where the accepted directory now is; the drain only removes
    /// entries above it, so its index does not move.
    fn keep_matching(&mut self, wanted: impl FnMut(&Name) -> bool) -> Option<usize> {
        let below = self.dirs.len() - 1;
        let found = self.dirs[..below].iter().rposition(wanted);
        self.dirs.drain(found.map_or(0, |at| at + 1)..below);
        found
    }
}

/// Push a directory.
///
/// A relative directory names a subdirectory of one already on the stack:
/// the stack is searched from the top down for the one it exists under, and
/// the directories passed on the way are dropped. On the file stack, and
/// for the first directory pushed, the name is taken as given.
pub(crate) fn qf_push_dir(dirbuf: &CStr, stack: &mut DirStack, is_file_stack: bool) {
    let plain = vim_is_abs_name(dirbuf) || stack.dirs.is_empty() || is_file_stack;
    stack.dirs.push(Name::from_bytes(dirbuf.to_bytes()));
    if plain {
        return;
    }

    // Look for a directory on the stack that `dirbuf` is under.
    let mut joined = None;
    let found = stack.keep_matching(|dir| {
        let path = join_fnames(dir.as_cstr(), dirbuf, true);
        let isdir = dir_exists(path.as_cstr());
        joined = Some(path);
        isdir
    });
    if found.is_some()
        && let Some(joined) = joined
        && let Some(top) = stack.dirs.last_mut()
    {
        // Under a known directory: keep the two joined.
        *top = Name::from_bytes(&joined);
    }
    // Nothing matched, so it must be a top-level directory: the name pushed
    // above is already the right one.
}

/// Drop the top directory.
pub(crate) fn qf_pop_dir(stack: &mut DirStack) {
    stack.dirs.pop();
}

/// Which directory on the stack a file can actually be found in, dropping
/// the ones the output has already left. `None` when none has it.
///
/// This is what recovers from `make` entering two sibling directories in a
/// row without saying it left the first: the pushed directory is wrong, but
/// one further down the stack holds the file.
pub(crate) fn qf_guess_filepath(mut qfl: Qfl, filename: &CStr) -> Option<Name> {
    // Nothing in the walk can run user code.
    let stack = &mut qfl.dir_stack;
    if stack.dirs.is_empty() {
        return None;
    }
    let found = stack.keep_matching(|dir| {
        let path = join_fnames(dir.as_cstr(), filename, true);
        path_exists(path.as_cstr())
    })?;
    Some(stack.dirs[found].clone())
}

/// Whether the entry a command was working on is still in the list.
///
/// Loading a file from the quickfix list runs autocommands, which may have
/// replaced the list under the command that was walking it; the caller has
/// checked the list's identity and change tick already.
pub(crate) fn is_qf_entry_present(qfl: &QfList, at: usize) -> bool {
    at < qfl.entries.len()
}

/// Why a walk found no entry to go to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum NoEntry {
    /// The list has no current entry at all.
    Empty,
    /// Not even one step could be taken: E553.
    NoMoreItems,
}

/// The next entry worth jumping to, searching forward from `at`, whose
/// number is `*idx`.
///
/// With `FORWARD_FILE` that means the next entry in a *different* file.
/// Answers `None` at the end of the list, leaving `*idx` alone.
fn get_next_valid_entry(qfl: &QfList, mut at: usize, idx: &mut c_int, dir: c_int) -> Option<usize> {
    let mut nr = *idx;
    let old_fnum = qfl.entries[at].fnum;
    loop {
        if nr == qfl.count() || at + 1 >= qfl.entries.len() {
            return None;
        }
        nr += 1;
        at += 1;
        if wanted_entry(qfl, &qfl.entries[at], old_fnum, dir, FORWARD_FILE as c_int) {
            break;
        }
    }
    *idx = nr;
    Some(at)
}

/// The next entry worth jumping to, searching backward from `at`.
fn get_prev_valid_entry(qfl: &QfList, mut at: usize, idx: &mut c_int, dir: c_int) -> Option<usize> {
    let mut nr = *idx;
    let old_fnum = qfl.entries[at].fnum;
    loop {
        if nr == 1 || at == 0 {
            return None;
        }
        nr -= 1;
        at -= 1;
        if wanted_entry(qfl, &qfl.entries[at], old_fnum, dir, BACKWARD_FILE as c_int) {
            break;
        }
    }
    *idx = nr;
    Some(at)
}

/// Whether the walk should stop at this entry: it names a real position (or
/// the list has none that do), and — when the walk is per file — it is not
/// in the file it started from.
fn wanted_entry(
    qfl: &QfList,
    entry: &QfEntry,
    old_fnum: c_int,
    dir: c_int,
    per_file: c_int,
) -> bool {
    if !qfl.no_valid && !entry.valid {
        return false;
    }
    !(dir == per_file && entry.fnum == old_fnum)
}

/// The `errornr`th entry worth jumping to from the current one, in `dir`.
///
/// Running out part way is not an error, and stops on the last one found;
/// not finding even one is [`NoEntry::NoMoreItems`].
fn get_nth_valid_entry(
    qfl: &QfList,
    mut errornr: c_int,
    dir: c_int,
    new_qfidx: &mut c_int,
) -> Result<usize, NoEntry> {
    if qfl.current().is_none() {
        return Err(NoEntry::Empty);
    }
    let mut at = qfl.cursor;
    let mut qf_idx = qfl.index;
    let mut first = true;
    while errornr != 0 {
        errornr -= 1;
        let next = if dir == FORWARD as c_int || dir == FORWARD_FILE as c_int {
            get_next_valid_entry(qfl, at, &mut qf_idx, dir)
        } else {
            get_prev_valid_entry(qfl, at, &mut qf_idx, dir)
        };
        let Some(next) = next else {
            if first {
                return Err(NoEntry::NoMoreItems);
            }
            break;
        };
        at = next;
        first = false;
    }
    *new_qfidx = qf_idx;
    Ok(at)
}

/// The entry numbered `errornr`, or the nearest one the list has.
pub(crate) fn get_nth_entry(
    qfl: &QfList,
    errornr: c_int,
    new_qfidx: &mut c_int,
) -> Result<usize, NoEntry> {
    if qfl.current().is_none() {
        return Err(NoEntry::Empty);
    }
    let mut at = qfl.cursor;
    let mut qf_idx = qfl.index;
    while errornr < qf_idx && qf_idx > 1 && at > 0 {
        qf_idx -= 1;
        at -= 1;
    }
    while errornr > qf_idx && qf_idx < qfl.count() && at + 1 < qfl.entries.len() {
        qf_idx += 1;
        at += 1;
    }
    *new_qfidx = qf_idx;
    Ok(at)
}

/// The entry a jump command asked for: the `errornr`th in `dir` when there
/// is a direction, the entry numbered `errornr` when there is not, and the
/// current one when neither was given. Answers its position in the list,
/// and sets `*new_qfidx` to its number.
pub(crate) fn qf_get_entry(
    qfl: &QfList,
    errornr: c_int,
    dir: c_int,
    new_qfidx: &mut c_int,
) -> Result<usize, NoEntry> {
    *new_qfidx = qfl.index;
    if dir != 0 {
        get_nth_valid_entry(qfl, errornr, dir, new_qfidx)
    } else if errornr != 0 {
        get_nth_entry(qfl, errornr, new_qfidx)
    } else if qfl.current().is_some() {
        Ok(qfl.cursor)
    } else {
        Err(NoEntry::Empty)
    }
}

/// How many entries the current list holds. Zero when there is no list.
pub fn qf_get_size(cmdidx: CmdIdx) -> size_t {
    let Some(qi) = stack_for_cmd(cmdidx, false) else {
        return 0;
    };
    qi.current_list().entries.len()
}

/// How many entries `:cdo`/`:ldo` would visit, or how many files
/// `:cfdo`/`:lfdo` would.
pub fn qf_get_valid_size(cmdidx: CmdIdx) -> size_t {
    let Some(qi) = stack_for_cmd(cmdidx, false) else {
        return 0;
    };
    let per_entry = cmdidx == CmdIdx::cdo || cmdidx == CmdIdx::ldo;
    let qfl = qi.current_list();
    let mut prev_fnum = 0;
    let mut size: size_t = 0;
    for entry in &qfl.entries {
        if got_int.get() {
            break;
        }
        if entry.valid {
            if per_entry {
                size += 1;
            } else if entry.fnum > 0 && entry.fnum != prev_fnum {
                size += 1;
                prev_fnum = entry.fnum;
            }
        }
    }
    size
}

/// Which entry of the current list is current. Zero when there is no list.
pub fn qf_get_cur_idx(cmdidx: CmdIdx) -> size_t {
    let Some(qi) = stack_for_cmd(cmdidx, false) else {
        return 0;
    };
    size_t::try_from(qi.current_list().index).expect("the current entry is numbered from one")
}

/// Which entry is current, counting only the entries `:cdo` would visit —
/// or, for `:cfdo`/`:lfdo`, only the files. One when there are none.
pub fn qf_get_cur_valid_idx(cmdidx: CmdIdx) -> c_int {
    let Some(qi) = stack_for_cmd(cmdidx, false) else {
        return 1;
    };
    let qfl = qi.current_list();
    if !qfl.has_valid_entries() {
        return 1;
    }
    let per_file = cmdidx == CmdIdx::cfdo || cmdidx == CmdIdx::lfdo;
    let mut prev_fnum = 0;
    let mut eidx = 0;
    let upto = usize::try_from(qfl.index).unwrap_or(0);
    for entry in qfl.entries.iter().take(upto) {
        if entry.valid {
            if !per_file {
                eidx += 1;
            } else if entry.fnum > 0 && entry.fnum != prev_fnum {
                eidx += 1;
                prev_fnum = entry.fnum;
            }
        }
    }
    if eidx != 0 { eidx } else { 1 }
}

/// Which entry the `n`th thing `:cdo` and friends visit is, counting
/// entries for `:cdo`/`:ldo` and files for `:cfdo`/`:lfdo`. One when the
/// list runs out first.
pub(crate) fn qf_get_nth_valid_entry(qfl: &QfList, n: size_t, fdo: bool) -> size_t {
    if !qfl.has_valid_entries() {
        return 1;
    }
    let mut prev_fnum = 0;
    let mut eidx: size_t = 0;
    for (at, entry) in qfl.entries.iter().enumerate() {
        if got_int.get() {
            // Upstream's walk stops where it was interrupted and answers
            // that entry.
            return at + 1;
        }
        if entry.valid {
            if !fdo {
                eidx += 1;
            } else if entry.fnum > 0 && entry.fnum != prev_fnum {
                eidx += 1;
                prev_fnum = entry.fnum;
            }
        }
        if eidx == n {
            return at + 1;
        }
    }
    1
}
