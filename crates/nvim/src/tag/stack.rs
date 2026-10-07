//! The tag stack.
//!
//! Every window remembers where each tag jump started, so `CTRL-T` and
//! `:pop` can walk back. [`TagStack`] is that stack — the entries, how many
//! are in use and which one the walk stands on. [`do_tags`] prints it,
//! [`get_tagstack`] and [`set_tagstack`] are the Vimscript views.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::highlight_group::HLF_D;
use crate::memory::ThinCString;
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::types::{Failed, IOSIZE, ListRef, VAR_LIST};
use crate::vim_snprintf;
use crate::winlayer::{Buf, Live, Win};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// One entry of a window's tag stack, whose caller has promised it outlives
/// the value.
///
/// The tag code passes `*mut Taggy` around because `'tagfunc'` can close
/// the window the entries live in, so no borrow may outlive one field
/// access — which is what [`Live`]'s `Deref` gives.
pub(crate) type Tagg = Live<Taggy>;

/// `emsg(_(msg))`: report one of this family's messages.
///
/// The argument is a [`CStr`] rather than a pointer, so nothing here is a
/// promise: the family's error messages are all string constants.
pub(crate) fn tag_emsg(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// How many entries a window's stack holds before the oldest is dropped.
const TAGSTACKSIZE: usize = super::TAGSTACKSIZE as usize;

/// The view a stack entry's mark starts with: no remembered scroll position.
const NO_VIEW: FileMarkView = FileMarkView {
    topline_offset: MAXLNUM,
    skipcol: 0,
};

/// One window's tag stack.
///
/// The entries live in the window itself — `w_tagstack`, of which
/// `w_tagstacklen` are in use, with `w_tagstackidx` the one `CTRL-T` would
/// pop next. This borrows the three together so the bookkeeping stays in
/// one place.
pub(crate) struct TagStack {
    win: Win,
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
    pub(crate) mark: Pos,
    pub(crate) fnum: c_int,
    /// Whatever `'tagfunc'` attached to the match. The entry takes
    /// ownership.
    pub(crate) user_data: *mut c_char,
}

impl TagStack {
    /// Borrow the tag stack of `window`.
    pub(crate) fn of(window: Win) -> Self {
        TagStack { win: window }
    }

    /// How many entries hold anything.
    pub(crate) fn len(&self) -> usize {
        // Upstream never lets the count past the array, but clamp anyway:
        // every index below is taken from this.
        (self.win.w_tagstacklen as usize).min(TAGSTACKSIZE)
    }

    /// The entry `CTRL-T` would pop next; one past the end at the top.
    pub(crate) fn curidx(&self) -> c_int {
        self.win.w_tagstackidx
    }

    /// The entries in use, oldest first.
    pub(crate) fn entries(&mut self) -> &mut [Taggy] {
        let len = self.len();
        &mut self.win.w_tagstack[..len]
    }

    /// Move the current index, clamped to the entries that exist.
    ///
    /// The stack length is a valid index: it means "at the top", where
    /// `CTRL-T` pops the newest entry.
    pub(crate) fn set_curidx(&mut self, curidx: c_int) {
        let len = self.len() as c_int;
        self.win.w_tagstackidx = curidx.clamp(0, len);
    }

    /// Throw the whole stack away.
    pub(crate) fn clear(&mut self) {
        self.truncate(0);
        self.win.w_tagstackidx = 0;
    }

    /// Drop every entry from `len` on, leaving the index alone.
    pub(crate) fn truncate(&mut self, len: usize) {
        for item in &mut self.entries()[len..] {
            tagstack_clear_entry(item);
        }
        // `len` is no larger than the count it replaces.
        self.win.w_tagstacklen = len as c_int;
    }

    /// Drop the oldest entry, shifting the rest down to free the top.
    fn shift(&mut self) {
        let entries = self.entries();
        tagstack_clear_entry(&mut entries[0]);
        entries.rotate_left(1);
        // The count was at least one.
        self.win.w_tagstacklen -= 1;
    }

    /// Put a new entry on top, dropping the oldest if the stack is full.
    pub(crate) fn push(&mut self, item: Push) {
        if self.len() >= TAGSTACKSIZE {
            self.shift();
        }
        let idx = self.len();
        // `idx` is now within the array, because `shift` made room for it.
        self.win.w_tagstacklen += 1;
        // Field by field, not a whole `Taggy`: the timestamp and the
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

    /// Add every dict in `items` that describes a jump, oldest first.
    fn push_items(&mut self, items: &ListRef) {
        // An index and a handle on each dict rather than a borrow of the
        // list: nothing here should hold into a user container across the
        // writes to the stack.
        let mut at = 0;
        while at < items.len() {
            let item = items.items()[at].li_tv.dict_handle();
            at += 1;

            // Skip anything that is not a dict describing a jump.
            let Some(item) = item else {
                continue;
            };
            let Some(from) = dict_find(Some(&*item), b"from") else {
                continue;
            };
            let mut mark = Pos::default();
            let mut fnum = 0;
            if list2fpos(&from.di_tv, &mut mark, Some(&mut fnum), None, false).is_err() {
                continue;
            }
            let Some(tagname) = dict_get_string_alloc(Some(&*item), b"tagname") else {
                continue;
            };
            // The dict counts columns from one, the mark from zero.
            if mark.col > 0 {
                mark.col -= 1;
            }
            self.push(Push {
                tagname: tagname.into_raw(),
                cur_fnum: number(&item, b"bufnr"),
                cur_match: number(&item, b"matchnr") - 1,
                mark,
                fnum,
                user_data: dict_get_string_alloc(Some(&*item), b"user_data")
                    .map_or(ptr::null_mut(), ThinCString::into_raw),
            });
        }
    }
}

/// Free what one stack entry owns, and forget it.
pub fn tagstack_clear_entry(item: &mut Taggy) {
    // SAFETY: the caller promises both fields are ours to free.
    unsafe { xfree(item.tagname.cast()) };
    unsafe { xfree(item.user_data.cast()) };
    item.tagname = ptr::null_mut();
    item.user_data = ptr::null_mut();
}

/// `:tags` — print the tag stack of the current window.
pub fn do_tags(_excmd: &mut ExArg) {
    let mut row = [0 as c_char; IOSIZE as usize];
    let mut stack = TagStack::of(Win::current());
    let curidx = stack.curidx();
    let len = stack.len();

    msg_title(gettext(c"\n  # TO tag         FROM line  in file/text"));
    for (i, item) in stack.entries().iter_mut().enumerate() {
        if item.tagname.is_null() {
            continue;
        }
        let name = unsafe { fm_getname(&raw mut item.fmark, 30) };
        if name.is_null() {
            // The file the jump came from is gone.
            continue;
        }
        msg_putchar('\n' as c_int);
        // Formatted rather than built up: a tag name longer than the
        // buffer is truncated, as upstream truncates it.
        let str_m = IOSIZE as size_t;
        // Two trailing spaces: the file name that follows is a separate
        // `msg_display`, and the gap is part of this format.
        let fmt = c"%c%2d %2d %-15s %5d  ".as_ptr();
        let args = if i as c_int == curidx { '>' } else { ' ' } as c_int;
        let arg5 = i as c_int + 1;
        let arg6 = item.cur_match + 1;
        let tagname2 = item.tagname;
        let lnum2 = item.fmark.mark.lnum;
        unsafe {
            vim_snprintf!(
                row.as_mut_ptr(),
                str_m,
                fmt,
                args,
                arg5,
                arg6,
                tagname2,
                lnum2,
            )
        };
        msg_display(cstr::in_chars(&row), 0, false);
        let hl = if item.fmark.fnum == Buf::current().handle {
            HLF_D
        } else {
            0
        };
        msg_display(unsafe { cstr::at(name) }, hl, false);
        unsafe { xfree(name.cast()) };
    }
    if curidx as usize == len {
        // Nothing has been popped: show where the next CTRL-T lands.
        msg_str(c"\n>");
    }
}

/// Describe one stack entry the way `gettagstack()` answers it.
fn tag_details(tag: &Taggy, retdict: &mut Dict) {
    add_str(retdict, b"tagname", tag.tagname);
    let _ = retdict.add_number(b"matchnr", (tag.cur_match + 1) as VarNumber);
    let _ = retdict.add_number(b"bufnr", tag.cur_fnum as VarNumber);
    if !tag.user_data.is_null() {
        add_str(retdict, b"user_data", tag.user_data);
    }

    let pos = tv_list_alloc(4);
    let _ = retdict.add_list(b"from", Some(pos.clone()));
    let mark = &tag.fmark;
    let str_m = if mark.fnum != -1 {
        mark.fnum as VarNumber
    } else {
        0
    };
    pos.edit().push_number(str_m);
    pos.edit().push_number(mark.mark.lnum as VarNumber);
    // Columns are counted from one outside, except for the "past the
    // end of the line" sentinel, which is passed through.
    let n2 = if mark.mark.col == MAXCOL as ColNr {
        MAXCOL as VarNumber
    } else {
        (mark.mark.col + 1) as VarNumber
    };
    pos.edit().push_number(n2);
    pos.edit().push_number(mark.mark.coladd as VarNumber);
}

/// `gettagstack()` — describe the tag stack of `window` into `retdict`.
pub fn get_tagstack(window: Win, retdict: &mut Dict) {
    let mut stack = TagStack::of(window);
    let _ = retdict.add_number(b"length", stack.len() as VarNumber);
    let _ = retdict.add_number(b"curidx", (stack.curidx() + 1) as VarNumber);

    let items = tv_list_alloc(2);
    let _ = retdict.add_list(b"items", Some(items.clone()));
    for entry in stack.entries() {
        let d = tv_dict_alloc();
        items.edit().push_dict(Some(d.clone()));
        tag_details(entry, d.edit());
    }
}

/// `settagstack()` — replace, append to or truncate the tag stack of `window`.
///
/// `action` is `'a'` to append, `'r'` to replace and `'t'` to truncate.
/// Answers `Err` with the error already reported.
pub fn set_tagstack(window: Win, d: &Dict, action: c_int) -> Result<(), Failed> {
    if tfu_in_use.get() {
        // 'tagfunc' is running: it is the tag stack's own contents
        // that are being computed.
        tag_emsg(c"E986: Cannot modify the tag stack within tagfunc");
        return Err(Failed);
    }

    let mut items = None;
    if let Some(di) = dict_find(Some(d), b"items") {
        if di.di_tv.v_type() != VAR_LIST {
            emsg(gettext(e_listreq));
            return Err(Failed);
        }
        items = di.di_tv.list_handle();
    }

    let mut stack = TagStack::of(window);
    if let Some(di) = dict_find(Some(d), b"curidx") {
        stack.set_curidx(tv_get_number(&di.di_tv) as c_int - 1);
    }

    if action == 't' as c_int {
        // Drop everything above the current entry.
        let keep = stack.curidx().max(0) as usize;
        if keep < stack.len() {
            stack.truncate(keep);
        }
    }

    if let Some(items) = items {
        if action == 'r' as c_int {
            stack.clear();
        }
        stack.push_items(&items);
        // Leave the index above the last entry, as a fresh jump would.
        stack.set_curidx(stack.len() as c_int);
    }
    Ok(())
}

/// [`Dict::add_str`] of an entry's own string, which may be null.
fn add_str(d: &mut Dict, key: &[u8], val: *const c_char) {
    // SAFETY: a stack entry's strings are null or NUL-terminated
    // allocations the entry owns, live for the call.
    let _ = d.add_str(key, unsafe { cstr::at_opt(val) });
}

/// A number field of a dict, zero when it is missing.
fn number(d: &Dict, key: &[u8]) -> c_int {
    dict_get_number(Some(d), key) as c_int
}
