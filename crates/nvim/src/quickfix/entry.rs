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
//! `qf_get_*_idx` pair are what `:cdo`/`:cfdo` count with.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::buffer::BufRef;
use crate::file_search::Name;
use crate::types::{CMD_cdo, CMD_cfdo, CMD_ldo, CMD_lfdo};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// The file name the previous entry was filed under, and the buffer it
/// named. Consecutive entries usually name the same file, so remembering
/// the last answer saves a `buflist_new` lookup per entry.
static last_bufname: GlobalCell<Option<Name>> = GlobalCell::new(None);
static last_bufref: GlobalCell<BufRef> = GlobalCell::new(BufRef::NONE);

/// Throw the cache away. The buffer it names may have been wiped out since,
/// and a stale hit would file entries under a dead buffer.
pub(crate) fn forget_last_buffer() {
    last_bufname.with_mut(|name| *name = None);
}

/// The buffer for `bufname`, from the cache or freshly listed.
///
/// # Safety
///
/// `bufname` must be NUL-terminated.
unsafe fn buffer_for(bufname: *mut c_char) -> *mut buf_T {
    // SAFETY: forwarded from the caller; `bufref_valid` only reads.
    unsafe {
        let cached = last_bufname.with(|name| match name {
            Some(name) => strcmp(bufname, name.as_ptr()) == 0,
            None => false,
        });
        if cached && last_bufref.get().valid() {
            return last_bufref.get().raw();
        }
        let buf = buflist_new(bufname, ptr::null_mut(), 0, BLN_NOOPT as c_int);
        let name = Name::from_ptr(bufname);
        last_bufname.with_mut(|slot| *slot = Some(name));
        last_bufref.set(BufRef::of_raw(buf));
        buf
    }
}

/// The buffer number for the file an entry names, listing the buffer if it
/// is not listed yet. Answers 0 when the entry names no file.
///
/// # Safety
///
/// `qfl` must be a live list; `directory` and `fname` null or
/// NUL-terminated.
pub(crate) unsafe fn qf_get_fnum(
    qfl: *mut qf_list_T,
    directory: *mut c_char,
    fname: *mut c_char,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        if fname.is_null() || *fname == 0 {
            return 0;
        }
        // Owned only when the name had to be joined to a directory.
        let mut joined: *mut c_char = ptr::null_mut();
        let bufname = if !directory.is_null() && !vim_is_abs_name(fname) {
            joined = concat_fnames(directory, fname, true);
            // The file should be there. If it is not, `make` changed
            // directory without a "leaving directory" message and the
            // directory stack has to be re-guessed.
            if !os_path_exists(joined) {
                xfree(joined.cast());
                let guess = qf_guess_filepath(qfl, fname);
                joined = if guess.is_null() {
                    xstrdup(fname)
                } else {
                    concat_fnames(guess, fname, true)
                };
            }
            joined
        } else {
            fname
        };

        let buf = buffer_for(bufname);
        xfree(joined.cast());
        if buf.is_null() {
            return 0;
        }
        (*buf).b_has_qf_entry = has_entry_flag(qfl);
        (*buf).handle as c_int
    }
}

impl DirStack {
    /// The directory most recently pushed, or null when the stack is empty.
    fn top(&mut self) -> *mut c_char {
        match self.dirs.last_mut() {
            Some(dir) => dir.as_mut_ptr(),
            None => ptr::null_mut(),
        }
    }

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

/// Push a directory, answering the one that is now on top.
///
/// A relative directory names a subdirectory of one already on the stack:
/// the stack is searched from the top down for the one it exists under, and
/// the directories passed on the way are dropped. On the file stack, and
/// for the first directory pushed, the name is taken as given.
///
/// # Safety
///
/// `slot` must be a live `DirStack` pointer, and `dirbuf` NUL-terminated.
pub(crate) unsafe fn qf_push_dir(
    dirbuf: *mut c_char,
    slot: *mut *mut DirStack,
    is_file_stack: bool,
) -> *mut c_char {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*slot).is_null() {
            *slot = Box::into_raw(Box::new(DirStack { dirs: Vec::new() }));
        }
        let stack = &mut **slot;
        let name = Name::from_ptr(dirbuf);
        let plain = vim_is_abs_name(dirbuf) || stack.dirs.is_empty() || is_file_stack;
        stack.dirs.push(name);
        if plain {
            return stack.top();
        }

        // Look for a directory on the stack that `dirbuf` is under.
        let mut joined: *mut c_char = ptr::null_mut();
        let found = stack.keep_matching(|dir| {
            xfree(joined.cast());
            joined = concat_fnames(dir.as_ptr(), dirbuf, true);
            os_isdir(joined)
        });
        if found.is_some() {
            // Under a known directory: keep the two joined.
            *stack.dirs.last_mut().unwrap() = Name::from_ptr(joined);
        }
        // Nothing matched, so it must be a top-level directory: the name
        // pushed above is already the right one. Upstream ends with a
        // "dirname is still NULL, pop the entry and answer NULL" branch,
        // which no path can reach — every branch above stores a name.
        xfree(joined.cast());
        stack.top()
    }
}

/// Drop the top directory, answering the one below it.
///
/// # Safety
///
/// `slot` must be a live `DirStack` pointer.
pub(crate) unsafe fn qf_pop_dir(slot: *mut *mut DirStack) -> *mut c_char {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*slot).is_null() {
            return ptr::null_mut();
        }
        let stack = &mut **slot;
        stack.dirs.pop();
        stack.top()
    }
}

/// Throw the whole stack away.
///
/// # Safety
///
/// `slot` must be a live `DirStack` pointer.
pub(crate) unsafe fn qf_clean_dir_stack(slot: *mut *mut DirStack) {
    // SAFETY: forwarded from the caller; the pointer was made by
    // `Box::into_raw` in `qf_push_dir`.
    unsafe {
        if !(*slot).is_null() {
            drop(Box::from_raw(*slot));
            *slot = ptr::null_mut();
        }
    }
}

/// Which directory on the stack a file can actually be found in, dropping
/// the ones the output has already left. Answers null when none has it.
///
/// This is what recovers from `make` entering two sibling directories in a
/// row without saying it left the first: the pushed directory is wrong, but
/// one further down the stack holds the file.
///
/// # Safety
///
/// `qfl` must be a live list and `filename` NUL-terminated.
pub(crate) unsafe fn qf_guess_filepath(qfl: *mut qf_list_T, filename: *mut c_char) -> *mut c_char {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*qfl).qf_dir_stack.is_null() {
            return ptr::null_mut();
        }
        let stack = &mut *(*qfl).qf_dir_stack;
        if stack.dirs.is_empty() {
            return ptr::null_mut();
        }
        let mut joined: *mut c_char = ptr::null_mut();
        let found = stack.keep_matching(|dir| {
            xfree(joined.cast());
            joined = concat_fnames(dir.as_ptr(), filename, true);
            os_path_exists(joined)
        });
        xfree(joined.cast());
        match found {
            Some(at) => stack.dirs[at].as_mut_ptr(),
            None => ptr::null_mut(),
        }
    }
}

/// Whether a list with the given id is still on the window's stack — or,
/// for a null window, on the quickfix stack.
///
/// # Safety
///
/// `wp` must be null or a window that was live; it is checked.
pub(crate) unsafe fn qflist_valid(wp: *mut win_T, qf_id: c_uint) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = if wp.is_null() {
            ql_info.get()
        } else if win_valid(wp) {
            win_loclist(wp)
        } else {
            return false;
        };
        !qi.is_null() && qf_id2nr(qi, qf_id) != INVALID_QFIDX
    }
}

/// Whether an entry is still in the list.
///
/// Loading a file from the quickfix list runs autocommands, which may have
/// replaced the list under the command that was walking it.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn is_qf_entry_present(qfl: *mut qf_list_T, qf_ptr: *mut qfline_T) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut i = 1;
        let mut qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if qfp == qf_ptr {
                break;
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        i <= (*qfl).qf_count
    }
}

/// The next entry worth jumping to, searching forward from `qf_ptr`.
///
/// With `FORWARD_FILE` that means the next entry in a *different* file.
/// Answers null at the end of the list, leaving `qf_index` alone.
///
/// # Safety
///
/// `qfl` must be a live list and `qf_ptr` an entry in it.
unsafe fn get_next_valid_entry(
    qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    qf_index: &mut c_int,
    dir: c_int,
) -> *mut qfline_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut idx = *qf_index;
        let old_fnum = (*qf_ptr).qf_fnum;
        loop {
            if idx == (*qfl).qf_count || (*qf_ptr).qf_next.is_null() {
                return ptr::null_mut();
            }
            idx += 1;
            qf_ptr = (*qf_ptr).qf_next;
            if wanted_entry(qfl, qf_ptr, old_fnum, dir, FORWARD_FILE as c_int) {
                break;
            }
        }
        *qf_index = idx;
        qf_ptr
    }
}

/// The next entry worth jumping to, searching backward from `qf_ptr`.
///
/// # Safety
///
/// `qfl` must be a live list and `qf_ptr` an entry in it.
unsafe fn get_prev_valid_entry(
    qfl: *mut qf_list_T,
    mut qf_ptr: *mut qfline_T,
    qf_index: &mut c_int,
    dir: c_int,
) -> *mut qfline_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut idx = *qf_index;
        let old_fnum = (*qf_ptr).qf_fnum;
        loop {
            if idx == 1 || (*qf_ptr).qf_prev.is_null() {
                return ptr::null_mut();
            }
            idx -= 1;
            qf_ptr = (*qf_ptr).qf_prev;
            if wanted_entry(qfl, qf_ptr, old_fnum, dir, BACKWARD_FILE as c_int) {
                break;
            }
        }
        *qf_index = idx;
        qf_ptr
    }
}

/// Whether the walk should stop at this entry: it names a real position (or
/// the list has none that do), and — when the walk is per file — it is not
/// in the file it started from.
///
/// # Safety
///
/// `qfl` must be a live list and `qf_ptr` an entry in it.
unsafe fn wanted_entry(
    qfl: *mut qf_list_T,
    qf_ptr: *mut qfline_T,
    old_fnum: c_int,
    dir: c_int,
    per_file: c_int,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !(*qfl).qf_nonevalid && (*qf_ptr).qf_valid == 0 {
            return false;
        }
        !(dir == per_file && (*qf_ptr).qf_fnum == old_fnum)
    }
}

/// The `errornr`th entry worth jumping to from the current one, in `dir`.
///
/// Reports E553 and answers null when there is not even one; running out
/// part way is not an error, and stops on the last one found.
///
/// # Safety
///
/// `qfl` must be a live list holding entries.
unsafe fn get_nth_valid_entry(
    qfl: *mut qf_list_T,
    mut errornr: c_int,
    dir: c_int,
    new_qfidx: &mut c_int,
) -> *mut qfline_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qf_ptr = (*qfl).qf_ptr;
        let mut qf_idx = (*qfl).qf_index;
        let mut first = true;
        while errornr != 0 {
            errornr -= 1;
            let prev_ptr = qf_ptr;
            let prev_idx = qf_idx;
            qf_ptr = if dir == FORWARD as c_int || dir == FORWARD_FILE as c_int {
                get_next_valid_entry(qfl, qf_ptr, &mut qf_idx, dir)
            } else {
                get_prev_valid_entry(qfl, qf_ptr, &mut qf_idx, dir)
            };
            if qf_ptr.is_null() {
                qf_ptr = prev_ptr;
                qf_idx = prev_idx;
                if first {
                    emsg(gettext(E_NO_MORE_ITEMS.as_ptr()));
                    return ptr::null_mut();
                }
                break;
            }
            first = false;
        }
        *new_qfidx = qf_idx;
        qf_ptr
    }
}

/// The entry numbered `errornr`, or the nearest one the list has.
///
/// # Safety
///
/// `qfl` must be a live list holding entries.
pub(crate) unsafe fn get_nth_entry(
    qfl: *mut qf_list_T,
    errornr: c_int,
    new_qfidx: &mut c_int,
) -> *mut qfline_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qf_ptr = (*qfl).qf_ptr;
        let mut qf_idx = (*qfl).qf_index;
        while errornr < qf_idx && qf_idx > 1 && !(*qf_ptr).qf_prev.is_null() {
            qf_idx -= 1;
            qf_ptr = (*qf_ptr).qf_prev;
        }
        while errornr > qf_idx && qf_idx < (*qfl).qf_count && !(*qf_ptr).qf_next.is_null() {
            qf_idx += 1;
            qf_ptr = (*qf_ptr).qf_next;
        }
        *new_qfidx = qf_idx;
        qf_ptr
    }
}

/// The entry a jump command asked for: the `errornr`th in `dir` when there
/// is a direction, the entry numbered `errornr` when there is not, and the
/// current one when neither was given.
///
/// # Safety
///
/// `qfl` must be a live list holding entries.
pub(crate) unsafe fn qf_get_entry(
    qfl: *mut qf_list_T,
    errornr: c_int,
    dir: c_int,
    new_qfidx: &mut c_int,
) -> *mut qfline_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        *new_qfidx = (*qfl).qf_index;
        if dir != 0 {
            get_nth_valid_entry(qfl, errornr, dir, new_qfidx)
        } else if errornr != 0 {
            get_nth_entry(qfl, errornr, new_qfidx)
        } else {
            (*qfl).qf_ptr
        }
    }
}

/// How many entries the current list holds. Zero when there is no list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_get_size(eap: *mut exarg_T) -> size_t {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if qi.is_null() {
            return 0;
        }
        (*qf_get_curlist(qi)).qf_count as size_t
    }
}

/// How many entries `:cdo`/`:ldo` would visit, or how many files
/// `:cfdo`/`:lfdo` would.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_get_valid_size(eap: *mut exarg_T) -> size_t {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if qi.is_null() {
            return 0;
        }
        let per_entry = (*eap).cmdidx as c_int == CMD_cdo as c_int
            || (*eap).cmdidx as c_int == CMD_ldo as c_int;
        let qfl = qf_get_curlist(qi);
        let mut prev_fnum = 0;
        let mut size: size_t = 0;
        let mut i = 1;
        let mut qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if per_entry {
                    size += 1;
                } else if (*qfp).qf_fnum > 0 && (*qfp).qf_fnum != prev_fnum {
                    size += 1;
                    prev_fnum = (*qfp).qf_fnum;
                }
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        size
    }
}

/// Which entry of the current list is current. Zero when there is no list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_get_cur_idx(eap: *mut exarg_T) -> size_t {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if qi.is_null() {
            return 0;
        }
        (*qf_get_curlist(qi)).qf_index as size_t
    }
}

/// Which entry is current, counting only the entries `:cdo` would visit —
/// or, for `:cfdo`/`:lfdo`, only the files. One when there are none.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_get_cur_valid_idx(eap: *mut exarg_T) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if qi.is_null() {
            return 1;
        }
        let qfl = qf_get_curlist(qi);
        if !qf_list_has_valid_entries(qfl) {
            return 1;
        }
        let per_file = (*eap).cmdidx as c_int == CMD_cfdo as c_int
            || (*eap).cmdidx as c_int == CMD_lfdo as c_int;
        let mut prev_fnum = 0;
        let mut eidx = 0;
        let mut i = 1;
        let mut qfp = (*qfl).qf_start;
        while i <= (*qfl).qf_index && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if !per_file {
                    eidx += 1;
                } else if (*qfp).qf_fnum > 0 && (*qfp).qf_fnum != prev_fnum {
                    eidx += 1;
                    prev_fnum = (*qfp).qf_fnum;
                }
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        if eidx != 0 { eidx } else { 1 }
    }
}

/// Which entry the `n`th thing `:cdo` and friends visit is, counting
/// entries for `:cdo`/`:ldo` and files for `:cfdo`/`:lfdo`. One when the
/// list runs out first.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn qf_get_nth_valid_entry(qfl: *mut qf_list_T, n: size_t, fdo: bool) -> size_t {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !qf_list_has_valid_entries(qfl) {
            return 1;
        }
        let mut prev_fnum = 0;
        let mut eidx: size_t = 0;
        let mut i = 1;
        let mut qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                if !fdo {
                    eidx += 1;
                } else if (*qfp).qf_fnum > 0 && (*qfp).qf_fnum != prev_fnum {
                    eidx += 1;
                    prev_fnum = (*qfp).qf_fnum;
                }
            }
            if eidx == n {
                break;
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        if i <= (*qfl).qf_count { i as size_t } else { 1 }
    }
}
