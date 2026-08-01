//! One list, and the entries in it.
//!
//! [`qf_new_list`] pushes a list onto a stack and [`qf_add_entry`] appends
//! an entry to it. The entries are a doubly linked list of `qfline_T`
//! hanging off `qf_start`/`qf_last`, with `qf_ptr`/`qf_index` marking the
//! one `:cc` would jump to.
//!
//! [`copy_loclist`] is what makes a location list follow a window that was
//! split, and [`qf_mark_adjust`] moves entry line numbers when the buffer
//! they point into is edited.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// The id the next list created is given. Ids are never reused, so a
/// caller that saved one can tell whether the list it saw is still there.
static last_qf_id: GlobalCell<c_uint> = GlobalCell::new(0);

/// A list slot that has never been used.
pub(crate) fn empty_list() -> qf_list_T {
    // SAFETY: every field is an integer, a bool, a raw pointer, or the
    // `Callback` whose zero discriminant is `kCallbackNone` and whose union
    // is a pointer either way.
    unsafe { core::mem::zeroed() }
}

/// Whether the list holds no entries. A null list counts as empty.
#[inline]
pub(crate) unsafe fn qf_list_empty(qfl: *const qf_list_T) -> bool {
    // SAFETY: the caller's list, which may be null.
    unsafe { qfl.is_null() || (*qfl).qf_count <= 0 }
}

/// Whether the list holds at least one entry naming a real position.
#[inline]
pub(crate) unsafe fn qf_list_has_valid_entries(qfl: *const qf_list_T) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe { !qf_list_empty(qfl) && !(*qfl).qf_nonevalid }
}

/// Note that the list changed, so that a command holding a pointer into it
/// can tell it has to start over.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn qf_list_changed(qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe { (*qfl).qf_changedtick += 1 };
}

/// Report that the list moved under a command that was in the middle of
/// using it: E925 for a quickfix list, E926 for a location list.
pub(crate) unsafe fn emsg_list_changed(qfl_type: qfltype_T) {
    // SAFETY: both messages are static NUL-terminated strings.
    unsafe {
        if qfl_type == QFLT_QUICKFIX {
            emsg(gettext(E_QUICKFIX_LIST_CHANGED.as_ptr()));
        } else {
            emsg(gettext(E_LOCATION_LIST_CHANGED.as_ptr()));
        }
    }
}

/// Push a new, empty list onto the stack and make it current.
///
/// Lists newer than the current one are dropped first, so that `:colder`
/// followed by a fresh `:grep` browses like a tree rather than growing a
/// second branch. When the stack is full the oldest list goes instead.
///
/// # Safety
///
/// `qi` must be a live stack, and `qf_title` null or NUL-terminated.
pub(crate) unsafe fn qf_new_list(qi: *mut qf_info_T, qf_title: *const c_char) {
    // SAFETY: forwarded from the caller.
    unsafe {
        while (*qi).qf_listcount > (*qi).qf_curlist + 1 {
            (*qi).qf_listcount -= 1;
            qf_free(qf_get_list(qi, (*qi).qf_listcount));
        }
        if (*qi).qf_listcount == (*qi).max_count() {
            qf_pop_stack(qi, false);
            (*qi).qf_curlist = (*qi).qf_listcount - 1;
        } else {
            (*qi).qf_curlist = (*qi).qf_listcount;
            (*qi).qf_listcount += 1;
        }
        let qfl = qf_get_curlist(qi);
        *qfl = empty_list();
        qf_store_title(qfl, qf_title);
        (*qfl).qfl_type = (*qi).qfl_type;
        last_qf_id.set(last_qf_id.get().wrapping_add(1));
        (*qfl).qf_id = last_qf_id.get();
        (*qfl).qf_has_user_data = false;
    }
}

/// Everything one new entry is made of.
///
/// This is the borrowed form: the strings are the caller's and are copied
/// into the entry, so nothing here is freed. [`Fields::entry`] builds one
/// from a parsed line; the other producers — `:vimgrep`, `:helpgrep`,
/// `setqflist()` and [`copy_loclist_entries`] — fill it in themselves,
/// because their names and messages come from buffers and dictionaries
/// rather than from the fixed field buffers a parse writes into.
pub(crate) struct NewEntry {
    /// The directory `fname` is relative to, from a `%D` line; may be null.
    pub(crate) dir: *mut c_char,
    /// The file the entry names, or null. Ignored when `bufnum` is set.
    pub(crate) fname: *mut c_char,
    /// The module name to show instead of the file name; may be null.
    pub(crate) module: *mut c_char,
    /// The buffer the entry names, or 0 to resolve `dir`/`fname` instead.
    pub(crate) bufnum: c_int,
    /// The text shown for the entry. Never null.
    pub(crate) mesg: *mut c_char,
    pub(crate) lnum: linenr_T,
    pub(crate) end_lnum: linenr_T,
    pub(crate) col: c_int,
    pub(crate) end_col: c_int,
    /// Non-zero when the column is a screen column, not a byte index.
    /// Wider than a bool because `setqflist()` stores whatever number the
    /// caller gave and `getqflist()` reports it back.
    pub(crate) vis_col: c_char,
    /// A search pattern to find the position with, instead of `lnum`.
    pub(crate) pattern: *mut c_char,
    /// The error number, from `%n`.
    pub(crate) nr: c_int,
    /// The error type: `e`, `w`, `i`, `n`, or 1 for a help entry.
    pub(crate) kind: c_char,
    /// Arbitrary value a `setqflist()` caller attached; may be null.
    pub(crate) user_data: *mut typval_T,
    /// The entry names a real position and can be jumped to.
    pub(crate) valid: bool,
}

impl NewEntry {
    /// An entry naming nothing, for the callers that set only a few fields.
    pub(crate) fn new(mesg: *mut c_char) -> NewEntry {
        NewEntry {
            dir: ptr::null_mut(),
            fname: ptr::null_mut(),
            module: ptr::null_mut(),
            bufnum: 0,
            mesg,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            vis_col: 0,
            pattern: ptr::null_mut(),
            nr: 0,
            kind: 0,
            user_data: ptr::null_mut(),
            valid: true,
        }
    }
}

/// Append an entry to the end of a list.
///
/// The first entry that names a real position becomes the current one, so
/// that a bare `:cc` after `:make` lands on the first error rather than on
/// a compiler banner.
///
/// # Safety
///
/// `qfl` must be a live list, and every string in `new` null or
/// NUL-terminated.
pub(crate) unsafe fn qf_add_entry(qfl: *mut qf_list_T, new: &NewEntry) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qfp: *mut qfline_T = xmalloc(size_of::<qfline_T>()).cast();
        let buf = if new.bufnum != 0 {
            (*qfp).qf_fnum = new.bufnum;
            let buf = buflist_findnr(new.bufnum);
            if !buf.is_null() {
                (*buf).b_has_qf_entry |= has_entry_flag(qfl);
            }
            buf
        } else {
            (*qfp).qf_fnum = qf_get_fnum(qfl, new.dir, new.fname);
            buflist_findnr((*qfp).qf_fnum)
        };

        // The entry shows a shortened name only when it differs from the
        // buffer's own, which is what the quickfix window would print.
        (*qfp).qf_fname = ptr::null_mut();
        let fullname = if new.fname.is_null() {
            ptr::null_mut()
        } else {
            fix_fname(new.fname)
        };
        if !buf.is_null() && !(*buf).b_ffname.is_null() && !fullname.is_null() {
            if path_fnamecmp(fullname, (*buf).b_ffname) != 0 {
                let short = path_try_shorten_fname(fullname);
                if !short.is_null() {
                    (*qfp).qf_fname = xstrdup(short);
                }
            }
        }
        xfree(fullname.cast());

        (*qfp).qf_text = xstrdup(new.mesg);
        (*qfp).qf_lnum = new.lnum;
        (*qfp).qf_end_lnum = new.end_lnum;
        (*qfp).qf_col = new.col;
        (*qfp).qf_end_col = new.end_col;
        (*qfp).qf_viscol = new.vis_col;
        if new.user_data.is_null() || (*new.user_data).v_type == VAR_UNKNOWN {
            (*qfp).qf_user_data.v_type = VAR_UNKNOWN;
        } else {
            tv_copy(new.user_data, &raw mut (*qfp).qf_user_data);
            (*qfl).qf_has_user_data = true;
        }
        (*qfp).qf_pattern = dup_unless_empty(new.pattern);
        (*qfp).qf_module = dup_unless_empty(new.module);
        (*qfp).qf_nr = new.nr;
        // 1 marks a help entry; anything else that cannot be printed is
        // reported as no type at all.
        (*qfp).qf_type = if new.kind != 1 && !vim_isprintc(new.kind as c_int) {
            0
        } else {
            new.kind
        };
        (*qfp).qf_valid = new.valid as c_char;
        (*qfp).qf_next = ptr::null_mut();
        (*qfp).qf_cleared = false as c_char;

        if qf_list_empty(qfl) {
            (*qfl).qf_start = qfp;
            (*qfl).qf_ptr = qfp;
            (*qfl).qf_index = 0;
            (*qfp).qf_prev = ptr::null_mut();
        } else {
            let last = (*qfl).qf_last;
            debug_assert!(!last.is_null());
            (*qfp).qf_prev = last;
            (*last).qf_next = qfp;
        }
        (*qfl).qf_last = qfp;
        (*qfl).qf_count += 1;
        if (*qfl).qf_index == 0 && (*qfp).qf_valid != 0 {
            (*qfl).qf_index = (*qfl).qf_count;
            (*qfl).qf_ptr = qfp;
        }
    }
}

/// Which of a buffer's `b_has_qf_entry` bits a list's entries set.
///
/// # Safety
///
/// `qfl` must be a live list.
#[inline]
pub(crate) unsafe fn has_entry_flag(qfl: *const qf_list_T) -> c_int {
    // SAFETY: forwarded from the caller.
    if unsafe { (*qfl).qfl_type } == QFLT_QUICKFIX {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    }
}

/// A copy of the string, or null when it is null or empty.
///
/// # Safety
///
/// `s` must be null or NUL-terminated.
#[inline]
unsafe fn dup_unless_empty(s: *const c_char) -> *mut c_char {
    // SAFETY: forwarded from the caller.
    unsafe {
        if s.is_null() || *s == 0 {
            ptr::null_mut()
        } else {
            xstrdup(s)
        }
    }
}

/// Copy every entry of one list into another.
///
/// `qf_add_entry` cannot work out the buffer number, because the file name
/// is not passed on; it is copied field by field afterwards instead.
///
/// # Safety
///
/// Both lists must be live, and `to_qfl` empty.
unsafe fn copy_loclist_entries(from_qfl: *const qf_list_T, to_qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut i = 1;
        let mut from = (*from_qfl).qf_start;
        while !got_int.get() && i <= (*from_qfl).qf_count && !from.is_null() {
            qf_add_entry(
                to_qfl,
                &NewEntry {
                    module: (*from).qf_module,
                    lnum: (*from).qf_lnum,
                    end_lnum: (*from).qf_end_lnum,
                    col: (*from).qf_col,
                    end_col: (*from).qf_end_col,
                    vis_col: (*from).qf_viscol,
                    pattern: (*from).qf_pattern,
                    nr: (*from).qf_nr,
                    user_data: &raw mut (*from).qf_user_data,
                    valid: (*from).qf_valid != 0,
                    ..NewEntry::new((*from).qf_text)
                },
            );
            let copy = (*to_qfl).qf_last;
            (*copy).qf_fnum = (*from).qf_fnum;
            (*copy).qf_type = (*from).qf_type;
            if (*from_qfl).qf_ptr == from {
                (*to_qfl).qf_ptr = copy;
            }
            i += 1;
            from = (*from).qf_next;
        }
    }
}

/// Copy one location list, entries and all.
///
/// # Safety
///
/// Both lists must be live, and `to_qfl` an unused slot.
pub(crate) unsafe fn copy_loclist(from_qfl: *mut qf_list_T, to_qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        // The entry fields are filled in by `qf_add_entry`.
        (*to_qfl).qfl_type = (*from_qfl).qfl_type;
        (*to_qfl).qf_nonevalid = (*from_qfl).qf_nonevalid;
        (*to_qfl).qf_has_user_data = (*from_qfl).qf_has_user_data;
        (*to_qfl).qf_count = 0;
        (*to_qfl).qf_index = 0;
        (*to_qfl).qf_start = ptr::null_mut();
        (*to_qfl).qf_last = ptr::null_mut();
        (*to_qfl).qf_ptr = ptr::null_mut();
        (*to_qfl).qf_title = if (*from_qfl).qf_title.is_null() {
            ptr::null_mut()
        } else {
            xstrdup((*from_qfl).qf_title)
        };
        (*to_qfl).qf_ctx = if (*from_qfl).qf_ctx.is_null() {
            ptr::null_mut()
        } else {
            let ctx: *mut typval_T = xcalloc(1, size_of::<typval_T>()).cast();
            tv_copy((*from_qfl).qf_ctx, ctx);
            ctx
        };
        callback_copy(
            &raw mut (*to_qfl).qf_qftf_cb,
            &raw mut (*from_qfl).qf_qftf_cb,
        );

        if (*from_qfl).qf_count != 0 {
            copy_loclist_entries(from_qfl, to_qfl);
        }

        (*to_qfl).qf_index = (*from_qfl).qf_index;
        last_qf_id.set(last_qf_id.get().wrapping_add(1));
        (*to_qfl).qf_id = last_qf_id.get();
        (*to_qfl).qf_changedtick = 0;
        // With nothing valid to point at, the current entry is the first.
        if (*to_qfl).qf_nonevalid {
            (*to_qfl).qf_ptr = (*to_qfl).qf_start;
            (*to_qfl).qf_index = 1;
        }
    }
}

/// Free every entry in a list, leaving its title and context alone.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn qf_free_items(qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut stop = false;
        while (*qfl).qf_count != 0 && !(*qfl).qf_start.is_null() {
            let qfp = (*qfl).qf_start;
            let next = (*qfp).qf_next;
            if !stop {
                xfree((*qfp).qf_fname.cast());
                xfree((*qfp).qf_module.cast());
                xfree((*qfp).qf_text.cast());
                xfree((*qfp).qf_pattern.cast());
                tv_clear(&raw mut (*qfp).qf_user_data);
                stop = qfp == next;
                xfree(qfp.cast());
                if stop {
                    // `qf_count` can be wrong; setting it to one here stops
                    // the loop rather than walking off the freed entry.
                    // TODO(vim): Avoid qf_count being incorrect.
                    (*qfl).qf_count = 1;
                } else {
                    (*qfl).qf_start = next;
                }
            }
            (*qfl).qf_count -= 1;
        }
        (*qfl).qf_start = ptr::null_mut();
        (*qfl).qf_last = ptr::null_mut();
        (*qfl).qf_ptr = ptr::null_mut();
        (*qfl).qf_index = 0;
        (*qfl).qf_nonevalid = true;

        qf_clean_dir_stack(&raw mut (*qfl).qf_dir_stack);
        (*qfl).qf_directory = ptr::null_mut();
        qf_clean_dir_stack(&raw mut (*qfl).qf_file_stack);
        (*qfl).qf_currfile = ptr::null_mut();
        (*qfl).qf_multiline = false;
        (*qfl).qf_multiignore = false;
        (*qfl).qf_multiscan = false;
    }
}

/// Free a list: its entries, its title and its context.
///
/// # Safety
///
/// `qfl` must be a live list.
pub(crate) unsafe fn qf_free(qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        qf_free_items(qfl);
        xfree((*qfl).qf_title.cast());
        (*qfl).qf_title = ptr::null_mut();
        tv_free((*qfl).qf_ctx);
        (*qfl).qf_ctx = ptr::null_mut();
        callback_free(&raw mut (*qfl).qf_qftf_cb);
        (*qfl).qf_id = 0;
        (*qfl).qf_changedtick = 0;
    }
}

/// Move the line numbers of every entry naming `buf` after an edit.
///
/// `buf` is the buffer that changed; `wp` names the window whose location
/// list to walk, or is null for the quickfix stack. Answers whether any
/// entry named the buffer at all — the caller clears the buffer's
/// "has entries" flag when none did.
///
/// # Safety
///
/// `buf` must be a live buffer and `wp` null or a live window.
pub unsafe fn qf_mark_adjust(
    buf: *mut buf_T,
    wp: *mut win_T,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let wanted = if wp.is_null() {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        if (*buf).b_has_qf_entry & wanted == 0 {
            return false;
        }
        let qi = if wp.is_null() {
            ql_info.get()
        } else if (*wp).w_llist.is_null() {
            return false;
        } else {
            (*wp).w_llist
        };

        let mut found_one = false;
        for idx in 0..(*qi).qf_listcount {
            let qfl = qf_get_list(qi, idx);
            let mut i = 1;
            let mut qfp = (*qfl).qf_start;
            while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
                if (*qfp).qf_fnum == (*buf).handle {
                    found_one = true;
                    if (*qfp).qf_lnum >= line1 && (*qfp).qf_lnum <= line2 {
                        if amount == MAXLNUM as linenr_T {
                            (*qfp).qf_cleared = true as c_char;
                        } else {
                            (*qfp).qf_lnum += amount;
                        }
                    } else if amount_after != 0 && (*qfp).qf_lnum > line2 {
                        (*qfp).qf_lnum += amount_after;
                    }
                }
                i += 1;
                qfp = (*qfp).qf_next;
            }
        }
        found_one
    }
}
