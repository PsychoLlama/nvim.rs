//! One list, and the entries in it.
//!
//! [`qf_new_list`] pushes a list onto a stack and [`qf_add_entry`] appends
//! an entry to it. A list's entries are a `Vec`, with the list's `cursor`
//! and `index` marking the one `:cc` would jump to.
//!
//! [`copy_loclist`] is what makes a location list follow a window that was
//! split, and [`qf_mark_adjust`] moves entry line numbers when the buffer
//! they point into is edited.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::XString;
use crate::path::{fixed_fname, try_shorten_fname};
use crate::types::VAR_UNKNOWN;
use core::ffi::{CStr, c_char, c_int};

/// Report that the list moved under a command that was in the middle of
/// using it: E925 for a quickfix list, E926 for a location list.
pub(crate) fn emsg_list_changed(kind: QfListType) {
    if kind == QFLT_QUICKFIX {
        emsg(gettext(E_QUICKFIX_LIST_CHANGED));
    } else {
        emsg(gettext(E_LOCATION_LIST_CHANGED));
    }
}

/// Push a new, empty list onto the stack and make it current.
///
/// Lists newer than the current one are dropped first, so that `:colder`
/// followed by a fresh `:grep` browses like a tree rather than growing a
/// second branch. When the stack is full the oldest list goes instead.
pub(crate) fn qf_new_list(mut qi: Qi, title: Option<&CStr>) {
    while qi.list_count > qi.current + 1 {
        qi.list_count -= 1;
        qf_free(qi.slot(qi.list_count));
    }
    if qi.list_count == qi.max_count() {
        pop_stack(qi, false);
        qi.current = qi.list_count - 1;
    } else {
        qi.current = qi.list_count;
        qi.list_count += 1;
    }
    let kind = qi.kind;
    let mut qfl = qi.current_slot();
    *qfl = QfList::new();
    qfl.title = title.map(XString::from_cstr);
    qfl.kind = kind;
    qfl.id = next_list_id();
}

/// Everything one new entry is made of.
///
/// This is the borrowed form: the strings are the caller's and are copied
/// into the entry. [`Fields::entry`] builds one from a parsed line; the
/// other producers — `:vimgrep`, `:helpgrep` and `setqflist()` — fill it in
/// themselves.
pub(crate) struct NewEntry<'a> {
    /// The directory `fname` is relative to, from a `%D` line.
    pub(crate) dir: Option<&'a CStr>,
    /// The file the entry names. Ignored when `bufnum` is set.
    pub(crate) fname: Option<&'a CStr>,
    /// The module name to show instead of the file name.
    pub(crate) module: Option<&'a CStr>,
    /// The buffer the entry names, or 0 to resolve `dir`/`fname` instead.
    pub(crate) bufnum: c_int,
    /// The text shown for the entry.
    pub(crate) mesg: &'a CStr,
    pub(crate) lnum: LineNr,
    pub(crate) end_lnum: LineNr,
    pub(crate) col: c_int,
    pub(crate) end_col: c_int,
    /// Non-zero when the column is a screen column, not a byte index.
    /// Wider than a bool because `setqflist()` stores whatever number the
    /// caller gave and `getqflist()` reports it back.
    pub(crate) vis_col: c_char,
    /// A search pattern to find the position with, instead of `lnum`.
    pub(crate) pattern: Option<&'a CStr>,
    /// The error number, from `%n`.
    pub(crate) nr: c_int,
    /// The error type: `e`, `w`, `i`, `n`, or 1 for a help entry.
    pub(crate) kind: c_char,
    /// Arbitrary value a `setqflist()` caller attached.
    pub(crate) user_data: Option<&'a TypVal>,
    /// The entry names a real position and can be jumped to.
    pub(crate) valid: bool,
}

impl<'a> NewEntry<'a> {
    /// An entry naming nothing, for the callers that set only a few fields.
    pub(crate) fn new(mesg: &'a CStr) -> NewEntry<'a> {
        NewEntry {
            dir: None,
            fname: None,
            module: None,
            bufnum: 0,
            mesg,
            lnum: 0,
            end_lnum: 0,
            col: 0,
            end_col: 0,
            vis_col: 0,
            pattern: None,
            nr: 0,
            kind: 0,
            user_data: None,
            valid: true,
        }
    }

    /// The entry, with every string copied — but not yet the buffer it
    /// names, which takes listing a buffer and so running autocommands.
    fn to_entry(&self) -> QfEntry {
        let mut user_data = TypVal::Unknown;
        if let Some(data) = self.user_data.filter(|data| data.v_type() != VAR_UNKNOWN) {
            tv_copy(data, &mut user_data);
        }
        QfEntry {
            lnum: self.lnum,
            end_lnum: self.end_lnum,
            fnum: 0,
            col: self.col,
            end_col: self.end_col,
            nr: self.nr,
            module: dup_unless_empty(self.module),
            fname: None,
            pattern: dup_unless_empty(self.pattern),
            text: XString::from_cstr(self.mesg),
            viscol: self.vis_col,
            cleared: false,
            // 1 marks a help entry; anything else that cannot be printed is
            // reported as no type at all.
            kind: if self.kind != 1 && !vim_isprintc(c_int::from(self.kind)) {
                0
            } else {
                self.kind
            },
            user_data,
            valid: self.valid,
        }
    }
}

/// A copy of the string, or nothing when it is absent or empty.
fn dup_unless_empty(s: Option<&CStr>) -> Option<XString> {
    s.filter(|s| !s.is_empty()).map(XString::from_cstr)
}

/// Which of a buffer's `b_has_qf_entry` bits a list's entries set.
pub(crate) fn has_entry_flag(kind: QfListType) -> c_int {
    if kind == QFLT_QUICKFIX {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    }
}

/// Append an entry to the end of a list.
///
/// The first entry that names a real position becomes the current one, so
/// that a bare `:cc` after `:make` lands on the first error rather than on
/// a compiler banner.
///
/// Naming the file lists a buffer, which fires `BufNew`; everything the
/// entry is made of is copied first, and the list is written only once that
/// is over.
pub(crate) fn qf_add_entry(mut qfl: Qfl, new: &NewEntry) {
    let mut entry = new.to_entry();
    let flag = has_entry_flag(qfl.kind);
    let buf = if new.bufnum != 0 {
        entry.fnum = new.bufnum;
        let buf = find_buf(new.bufnum);
        if let Some(mut buf) = buf {
            buf.b_has_qf_entry |= flag;
        }
        buf
    } else {
        entry.fnum = qf_get_fnum(qfl, new.dir, new.fname);
        find_buf(entry.fnum)
    };

    // The entry shows a shortened name only when it differs from the
    // buffer's own, which is what the quickfix window would print.
    if let Some(fname) = new.fname
        && let Some(full) = fixed_fname(fname)
        && let Some(buf_ffname) = buf.and_then(|buf| buf.name.full().map(CStr::to_owned))
        && path_fnamecmp(full.as_cstr(), &buf_ffname) != 0
    {
        entry.fname = Some(XString::from_cstr(try_shorten_fname(full.as_cstr())));
    }

    let has_user_data = new
        .user_data
        .is_some_and(|data| data.v_type() != VAR_UNKNOWN);
    let valid = entry.valid;
    if qfl.is_empty() {
        qfl.cursor = 0;
        qfl.index = 0;
    }
    qfl.entries.push(entry);
    if has_user_data {
        qfl.has_user_data = true;
    }
    if qfl.index == 0 && valid {
        qfl.index = qfl.count();
        qfl.cursor = qfl.entries.len() - 1;
    }
}

/// A copy of an entry for another window's location list: what
/// `qf_add_entry` would make of it, which leaves out the shortened file
/// name, since that is not passed on.
fn copy_entry(from: &QfEntry) -> QfEntry {
    let mut user_data = TypVal::Unknown;
    if from.user_data.v_type() != VAR_UNKNOWN {
        tv_copy(&from.user_data, &mut user_data);
    }
    QfEntry {
        lnum: from.lnum,
        end_lnum: from.end_lnum,
        fnum: from.fnum,
        col: from.col,
        end_col: from.end_col,
        nr: from.nr,
        module: from.module.clone(),
        fname: None,
        pattern: from.pattern.clone(),
        text: from.text.clone(),
        viscol: from.viscol,
        cleared: false,
        kind: from.kind,
        user_data,
        valid: from.valid,
    }
}

/// Copy one location list, entries and all, into an unused slot.
pub(crate) fn copy_loclist(from: &QfList, to: &mut QfList) {
    to.kind = from.kind;
    to.no_valid = from.no_valid;
    to.has_user_data = from.has_user_data;
    to.title = from.title.clone();
    to.context = from.context.as_ref().map(|ctx| {
        let mut copy = Box::new(TypVal::Unknown);
        tv_copy(ctx, &mut copy);
        copy
    });
    to.text_func = from.text_func.duplicate();

    to.entries = Vec::with_capacity(from.entries.len());
    to.cursor = 0;
    for (at, entry) in from.entries.iter().enumerate() {
        if got_int.get() {
            break;
        }
        to.entries.push(copy_entry(entry));
        if from.cursor == at {
            to.cursor = at;
        }
    }

    to.index = from.index;
    to.id = next_list_id();
    to.changedtick = 0;
    // With nothing valid to point at, the current entry is the first.
    if to.no_valid {
        to.cursor = 0;
        to.index = 1;
    }
}

/// Free every entry in a list, leaving its title and context alone.
pub(crate) fn qf_free_items(qfl: &mut QfList) {
    qfl.entries = Vec::new();
    qfl.cursor = 0;
    qfl.index = 0;
    qfl.no_valid = true;
    qfl.dir_stack.dirs.clear();
    qfl.file_stack.dirs.clear();
    qfl.multiline = false;
    qfl.multiignore = false;
    qfl.multiscan = false;
}

/// Free a list: its entries, its title and its context. What kind of list
/// the slot held is kept, for a command that reports the list went away.
pub(crate) fn qf_free(mut qfl: Qfl) {
    let kind = qfl.kind;
    let old = core::mem::take(&mut *qfl);
    qfl.kind = kind;
    drop(old);
}

impl Buf {
    /// Move the line numbers of every quickfix entry naming this buffer after
    /// an edit: upstream's `qf_mark_adjust`.
    ///
    /// `window` names the window whose location list to walk, or is `None`
    /// for the quickfix stack. Answers whether any entry named the buffer at
    /// all — the caller clears the buffer's "has entries" flag when none did.
    ///
    /// Every edit asks, once for the quickfix stack and once per window, so
    /// the flag test is the inlined part and the walk is not.
    #[inline]
    pub(crate) fn adjust_quickfix_entries(
        self,
        window: Option<Win>,
        line1: LineNr,
        line2: LineNr,
        amount: LineNr,
        amount_after: LineNr,
    ) -> bool {
        let wanted = if window.is_none() {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        self.b_has_qf_entry & wanted != 0
            && adjust_entries(self, window, line1, line2, amount, amount_after)
    }
}

/// [`Buf::adjust_quickfix_entries`]'s walk, for a buffer that has entries.
#[inline(never)]
fn adjust_entries(
    buffer: Buf,
    window: Option<Win>,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) -> bool {
    let mut qi = match window {
        None => Qi::global(),
        Some(wp) => match wp.w_llist {
            Some(id) => id.stack(),
            None => return false,
        },
    };

    // Nothing below can run user code, so the stack is borrowed whole.
    let stack: &mut QfStack = &mut qi;
    let count = usize::try_from(stack.list_count).unwrap_or(0);
    let mut found_one = false;
    for list in &mut stack.lists[..count] {
        for entry in &mut list.entries {
            if got_int.get() {
                break;
            }
            if entry.fnum != buffer.handle {
                continue;
            }
            found_one = true;
            if entry.lnum >= line1 && entry.lnum <= line2 {
                if amount == MAXLNUM {
                    entry.cleared = true;
                } else {
                    entry.lnum += amount;
                }
            } else if amount_after != 0 && entry.lnum > line2 {
                entry.lnum += amount_after;
            }
        }
    }
    found_one
}
