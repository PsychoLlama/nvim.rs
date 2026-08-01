//! The tag stack.
//!
//! Every window remembers where each tag jump started, so `CTRL-T` and
//! `:pop` can walk back. [`TagStack`] is that stack — the entries, how many
//! are in use and which one the walk stands on. [`do_tags`] prints it,
//! [`get_tagstack`] and [`set_tagstack`] are the Vimscript views.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// How many entries a window's stack holds before the oldest is dropped.
const TAGSTACKSIZE: usize = super::TAGSTACKSIZE as usize;

/// The view a stack entry's mark starts with: no remembered scroll position.
const NO_VIEW: fmarkv_T = fmarkv_T {
    topline_offset: MAXLNUM as linenr_T,
    skipcol: 0,
};

/// One window's tag stack.
///
/// The entries live in the window itself — `w_tagstack`, of which
/// `w_tagstacklen` are in use, with `w_tagstackidx` the one `CTRL-T` would
/// pop next. This borrows the three together so the bookkeeping stays in
/// one place.
pub(crate) struct TagStack {
    win: *mut win_T,
}

/// A new entry for [`TagStack::push`].
pub(crate) struct Push {
    /// The tag being jumped to. The entry takes ownership.
    pub(crate) tagname: *mut c_char,
    /// The buffer the jump landed in, or 0 when that is not known yet.
    pub(crate) cur_fnum: c_int,
    /// Which of the matches was taken, counted from zero.
    pub(crate) cur_match: c_int,
    /// Where the cursor was before the jump, and in which buffer.
    pub(crate) mark: pos_T,
    pub(crate) fnum: c_int,
    /// Whatever `'tagfunc'` attached to the match. The entry takes
    /// ownership.
    pub(crate) user_data: *mut c_char,
}

impl TagStack {
    /// Borrow the tag stack of `wp`.
    ///
    /// # Safety
    /// `wp` must be a live window, and nothing else may be reaching into
    /// its tag stack for as long as this lives.
    pub(crate) unsafe fn of(wp: *mut win_T) -> Self {
        TagStack { win: wp }
    }

    /// How many entries hold anything.
    pub(crate) fn len(&self) -> usize {
        // SAFETY: the window is live, and upstream never lets the count
        // past the array.
        unsafe { ((*self.win).w_tagstacklen as usize).min(TAGSTACKSIZE) }
    }

    /// The entry `CTRL-T` would pop next; one past the end at the top.
    pub(crate) fn curidx(&self) -> c_int {
        // SAFETY: the window is live.
        unsafe { (*self.win).w_tagstackidx }
    }

    /// The entries in use, oldest first.
    pub(crate) fn entries(&mut self) -> &mut [taggy_T] {
        let len = self.len();
        // SAFETY: `w_tagstack` is an inline array of `TAGSTACKSIZE`
        // entries, and `len` is clamped to it.
        unsafe {
            core::slice::from_raw_parts_mut(
                (&raw mut (*self.win).w_tagstack).cast::<taggy_T>(),
                len,
            )
        }
    }

    /// Move the current index, clamped to the entries that exist.
    ///
    /// The stack length is a valid index: it means "at the top", where
    /// `CTRL-T` pops the newest entry.
    pub(crate) fn set_curidx(&mut self, curidx: c_int) {
        let len = self.len() as c_int;
        // SAFETY: the window is live.
        unsafe { (*self.win).w_tagstackidx = curidx.clamp(0, len) };
    }

    /// Throw the whole stack away.
    pub(crate) fn clear(&mut self) {
        self.truncate(0);
        // SAFETY: the window is live.
        unsafe { (*self.win).w_tagstackidx = 0 };
    }

    /// Drop every entry from `len` on, leaving the index alone.
    pub(crate) fn truncate(&mut self, len: usize) {
        for item in &mut self.entries()[len..] {
            // SAFETY: an entry in use owns its name and user data.
            unsafe { tagstack_clear_entry(item) };
        }
        // SAFETY: the window is live, and `len` is no larger than the
        // count it replaces.
        unsafe { (*self.win).w_tagstacklen = len as c_int };
    }

    /// Drop the oldest entry, shifting the rest down to free the top.
    fn shift(&mut self) {
        let entries = self.entries();
        // SAFETY: entry 0 is in use, so it owns its name and user data.
        unsafe { tagstack_clear_entry(&mut entries[0]) };
        entries.rotate_left(1);
        // SAFETY: the window is live, and the count was at least one.
        unsafe { (*self.win).w_tagstacklen -= 1 };
    }

    /// Put a new entry on top, dropping the oldest if the stack is full.
    pub(crate) fn push(&mut self, item: Push) {
        if self.len() >= TAGSTACKSIZE {
            self.shift();
        }
        let idx = self.len();
        // SAFETY: the window is live, and `idx` is now within the array
        // because `shift` made room for it.
        unsafe { (*self.win).w_tagstacklen += 1 };
        // Field by field, not a whole `taggy_T`: the timestamp and the
        // additional data of the slot are deliberately left as they were.
        let entry = &mut self.entries()[idx];
        entry.tagname = item.tagname;
        entry.cur_fnum = item.cur_fnum;
        // A match number below zero would index the wrong way on the way
        // back out.
        entry.cur_match = item.cur_match.max(0);
        entry.fmark.mark = item.mark;
        entry.fmark.fnum = item.fnum;
        entry.fmark.view = NO_VIEW;
        entry.user_data = item.user_data;
    }

    /// Add every dict in `l` that describes a jump, oldest first.
    ///
    /// # Safety
    /// `l` must be a live list.
    unsafe fn push_items(&mut self, l: *mut list_T) {
        // SAFETY: the list and its items are live for the whole walk, and
        // the two strings taken out of each dict are freshly allocated
        // copies the new entry takes over.
        unsafe {
            let mut li = tv_list_first(l);
            while !li.is_null() {
                let tv = &raw mut (*li).li_tv;
                li = (*li).li_next;

                // Skip anything that is not a dict describing a jump.
                if (*tv).v_type != VAR_DICT || (*tv).vval.v_dict.is_null() {
                    continue;
                }
                let item = (*tv).vval.v_dict;
                let Some(from) = find(item, c"from") else {
                    continue;
                };
                let mut mark = pos_T::default();
                let mut fnum = 0;
                if list2fpos(
                    &raw mut (*from).di_tv,
                    &raw mut mark,
                    &raw mut fnum,
                    ptr::null_mut(),
                    false,
                ) != OK
                {
                    continue;
                }
                let tagname = tv_dict_get_string(item, c"tagname".as_ptr(), true);
                if tagname.is_null() {
                    continue;
                }
                // The dict counts columns from one, the mark from zero.
                if mark.col > 0 {
                    mark.col -= 1;
                }
                self.push(Push {
                    tagname,
                    cur_fnum: number(item, c"bufnr"),
                    cur_match: number(item, c"matchnr") - 1,
                    mark,
                    fnum,
                    user_data: tv_dict_get_string(item, c"user_data".as_ptr(), true),
                });
            }
        }
    }
}

/// Free what one stack entry owns, and forget it.
///
/// # Safety
/// The entry's `tagname` and `user_data` must be NULL or allocations the
/// entry owns.
pub unsafe fn tagstack_clear_entry(item: &mut taggy_T) {
    // SAFETY: the caller promises both fields are ours to free.
    unsafe {
        xfree(item.tagname.cast());
        xfree(item.user_data.cast());
    }
    item.tagname = ptr::null_mut();
    item.user_data = ptr::null_mut();
}

/// `:tags` — print the tag stack of the current window.
///
/// # Safety
/// Must be called with a live `curwin`.
pub unsafe fn do_tags(_eap: *mut exarg_T) {
    // SAFETY: `curwin` is live, and `fm_getname` answers an allocation we
    // free again below.
    unsafe {
        let mut stack = TagStack::of(curwin.get());
        let curidx = stack.curidx();
        let len = stack.len();

        msg_puts_title(gettext(
            c"\n  # TO tag         FROM line  in file/text".as_ptr(),
        ));
        for (i, item) in stack.entries().iter_mut().enumerate() {
            if item.tagname.is_null() {
                continue;
            }
            let name = fm_getname(&raw mut item.fmark, 30);
            if name.is_null() {
                // The file the jump came from is gone.
                continue;
            }
            msg_putchar('\n' as c_int);
            // Kept in `IObuff` rather than built here: a tag name longer
            // than the buffer is truncated, as upstream truncates it.
            vim_snprintf(
                IObuff.ptr().cast(),
                IOSIZE as size_t,
                c"%c%2d %2d %-15s %5d  ".as_ptr(),
                if i as c_int == curidx { '>' } else { ' ' } as c_int,
                i as c_int + 1,
                item.cur_match + 1,
                item.tagname,
                item.fmark.mark.lnum,
            );
            msg_outtrans(IObuff.ptr().cast(), 0, false);
            let hl = if item.fmark.fnum == (*curbuf.get()).handle {
                HLF_D as c_int
            } else {
                0
            };
            msg_outtrans(name, hl, false);
            xfree(name.cast());
        }
        if curidx as usize == len {
            // Nothing has been popped: show where the next CTRL-T lands.
            msg_puts(c"\n>".as_ptr());
        }
    }
}

/// Describe one stack entry the way `gettagstack()` answers it.
///
/// # Safety
/// Both pointers must be live.
unsafe fn tag_details(tag: &taggy_T, retdict: *mut dict_T) {
    // SAFETY: the dict is live, and the entry's strings are
    // NUL-terminated.
    unsafe {
        add_str(retdict, c"tagname", tag.tagname);
        add_nr(retdict, c"matchnr", (tag.cur_match + 1) as varnumber_T);
        add_nr(retdict, c"bufnr", tag.cur_fnum as varnumber_T);
        if !tag.user_data.is_null() {
            add_str(retdict, c"user_data", tag.user_data);
        }

        let pos = tv_list_alloc(4);
        tv_dict_add_list(retdict, c"from".as_ptr(), c"from".count_bytes(), pos);
        let mark = &tag.fmark;
        tv_list_append_number(
            pos,
            if mark.fnum != -1 {
                mark.fnum as varnumber_T
            } else {
                0
            },
        );
        tv_list_append_number(pos, mark.mark.lnum as varnumber_T);
        // Columns are counted from one outside, except for the "past the
        // end of the line" sentinel, which is passed through.
        tv_list_append_number(
            pos,
            if mark.mark.col == MAXCOL as colnr_T {
                MAXCOL as varnumber_T
            } else {
                (mark.mark.col + 1) as varnumber_T
            },
        );
        tv_list_append_number(pos, mark.mark.coladd as varnumber_T);
    }
}

/// `gettagstack()` — describe the tag stack of `wp` into `retdict`.
///
/// # Safety
/// Both pointers must be live.
pub unsafe fn get_tagstack(wp: *mut win_T, retdict: *mut dict_T) {
    // SAFETY: the window and the dict are live.
    unsafe {
        let mut stack = TagStack::of(wp);
        add_nr(retdict, c"length", stack.len() as varnumber_T);
        add_nr(retdict, c"curidx", (stack.curidx() + 1) as varnumber_T);

        let items = tv_list_alloc(2);
        tv_dict_add_list(retdict, c"items".as_ptr(), c"items".count_bytes(), items);
        for entry in stack.entries() {
            let d = tv_dict_alloc();
            tv_list_append_dict(items, d);
            tag_details(entry, d);
        }
    }
}

/// `settagstack()` — replace, append to or truncate the tag stack of `wp`.
///
/// `action` is `'a'` to append, `'r'` to replace and `'t'` to truncate.
/// Answers `OK`, or `FAIL` with the error already reported.
///
/// # Safety
/// Both pointers must be live.
pub unsafe fn set_tagstack(wp: *mut win_T, d: *const dict_T, action: c_int) -> c_int {
    // SAFETY: the window and the dict are live for the whole call.
    unsafe {
        if tfu_in_use.get() {
            // 'tagfunc' is running: it is the tag stack's own contents
            // that are being computed.
            emsg(gettext(
                c"E986: Cannot modify the tag stack within tagfunc".as_ptr(),
            ));
            return FAIL;
        }

        let mut items = ptr::null_mut::<list_T>();
        if let Some(di) = find(d, c"items") {
            if (*di).di_tv.v_type != VAR_LIST {
                emsg(gettext((&raw const e_listreq).cast()));
                return FAIL;
            }
            items = (*di).di_tv.vval.v_list;
        }

        let mut stack = TagStack::of(wp);
        if let Some(di) = find(d, c"curidx") {
            stack.set_curidx(tv_get_number(&raw mut (*di).di_tv) as c_int - 1);
        }

        if action == 't' as c_int {
            // Drop everything above the current entry.
            let keep = stack.curidx().max(0) as usize;
            if keep < stack.len() {
                stack.truncate(keep);
            }
        }

        if !items.is_null() {
            if action == 'r' as c_int {
                stack.clear();
            }
            stack.push_items(items);
            // Leave the index above the last entry, as a fresh jump would.
            stack.set_curidx(stack.len() as c_int);
        }
        OK
    }
}

/// [`tv_dict_find`] answering `None` rather than a NULL pointer.
///
/// # Safety
/// `d` must be live.
unsafe fn find(d: *const dict_T, key: &CStr) -> Option<*mut dictitem_T> {
    // SAFETY: the dict is live and the key is NUL-terminated.
    let di = unsafe { tv_dict_find(d, key.as_ptr(), -1) };
    (!di.is_null()).then_some(di)
}

/// [`tv_dict_add_nr`] with the key's length taken from the literal.
///
/// # Safety
/// `d` must be live.
unsafe fn add_nr(d: *mut dict_T, key: &CStr, nr: varnumber_T) {
    // SAFETY: the dict is live and the key is NUL-terminated.
    unsafe { tv_dict_add_nr(d, key.as_ptr(), key.count_bytes(), nr) };
}

/// [`tv_dict_add_str`] with the key's length taken from the literal.
///
/// # Safety
/// `d` must be live and `val` NUL-terminated.
unsafe fn add_str(d: *mut dict_T, key: &CStr, val: *const c_char) {
    // SAFETY: the dict is live, and both strings are NUL-terminated.
    unsafe { tv_dict_add_str(d, key.as_ptr(), key.count_bytes(), val) };
}

/// A number field of a dict, zero when it is missing.
///
/// # Safety
/// `d` must be live.
unsafe fn number(d: *const dict_T, key: &CStr) -> c_int {
    // SAFETY: the dict is live and the key is NUL-terminated.
    unsafe { tv_dict_get_number(d, key.as_ptr()) as c_int }
}
