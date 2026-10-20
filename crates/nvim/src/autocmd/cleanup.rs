//! Deleting autocommands, and the deferred cleanup that makes it safe.
//!
//! The table itself lives here. An `AutoCmd` row never moves while a
//! firing walk might be standing on it: [`aucmd_del`] marks it and sets
//! `au_need_clean`, and [`au_cleanup`] does the actual removal once no walk
//! is live.  The
//! `aupat_*` half is the buffer-local pattern format -- `<buffer=N>` --
//! which is a pattern in the same table but matched by buffer number, and
//! [`aubuflocal_remove`] is what a wiped-out buffer's patterns go through.

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
use crate::message_fmt::msg_cstr;
use crate::smsg;
use crate::types::{AutoPat, EventCmds};
use crate::winlayer::Buf;

/// The autocommands defined for each event, indexed by event number.
///
/// Reached only through [`with_event_cmds`] and [`with_event_cmds_mut`],
/// one closure at a time: a handler run by a walk can define and delete
/// autocommands, so nothing holds a borrow of the table across user code --
/// a walk holds row indices and a [`PatId`](crate::types::PatId) instead.
static autocmds: GlobalCell<[EventCmds; AutoEvent::COUNT]> =
    GlobalCell::new([EventCmds::EMPTY; AutoEvent::COUNT]);

/// Run `f` over `event`'s autocommands.
pub(crate) fn with_event_cmds<R>(event: AutoEvent, f: impl FnOnce(&EventCmds) -> R) -> R {
    autocmds.with(|table| f(&table[event.index()]))
}

/// [`with_event_cmds`], to edit them.
pub(super) fn with_event_cmds_mut<R>(event: AutoEvent, f: impl FnOnce(&mut EventCmds) -> R) -> R {
    autocmds.with_mut(|table| f(&mut table[event.index()]))
}

/// Mark row `idx` of `event` deleted: release everything it owns and clear
/// its pattern, which is the mark [`au_cleanup`] compacts on.
///
/// The row itself stays in place. A walk may be standing on it, and the
/// list it is walking is indexed, so the row may not move until
/// `autocmd_busy` is clear.
pub(crate) fn aucmd_del(event: AutoEvent, idx: usize) {
    with_event_cmds_mut(event, |cmds| cmds.delete(idx));
    au_need_clean.set(true);
}

/// Delete every live autocommand of `event` whose pattern `doomed` picks,
/// answering how many went.
pub(crate) fn aucmd_del_where(event: AutoEvent, doomed: impl Fn(&AutoPat) -> bool) -> usize {
    let deleted = with_event_cmds_mut(event, |cmds| {
        let mut deleted = 0;
        for idx in 0..cmds.len() {
            if cmds.cmds[idx].pat.is_some_and(|id| doomed(cmds.pat(id))) {
                cmds.delete(idx);
                deleted += 1;
            }
        }
        deleted
    });
    if deleted > 0 {
        au_need_clean.set(true);
    }
    deleted
}

/// Delete every autocommand `group` defined for `event`.
pub fn aucmd_del_for_event_and_group(event: AutoEvent, group: ::core::ffi::c_int) {
    aucmd_del_where(event, |pat| pat.group == group);
    au_cleanup();
}

/// Compact the deleted rows out of every event's list.
///
/// This is the deferred half of [`aucmd_del`] and it may only run while no
/// walk is live: `autocmd_busy` says one is, and then the rows have to stay
/// where a walk's index can still reach them. Safe to call anywhere,
/// because that is the first thing it checks -- a call made during a walk
/// returns having done nothing, and the compaction waits for the point
/// where the editor is between autocommands.
pub(crate) fn au_cleanup() {
    if autocmd_busy.get() || !au_need_clean.get() {
        return;
    }
    autocmds.with_mut(|table| table.iter_mut().for_each(EventCmds::compact));
    au_need_clean.set(false);
}

/// Run `f` over the autocommand list for `event`, for the API's readers.
/// Nothing may run user code inside `f`.
pub(crate) fn with_autocmds_for_event<R>(event: AutoEvent, f: impl FnOnce(&EventCmds) -> R) -> R {
    with_event_cmds(event, f)
}

/// Drop every `<buffer=N>` autocommand naming `buffer`, which is being freed.
pub fn aubuflocal_remove(buffer: Buf) {
    // A walk in progress may be about to match on this buffer number;
    // clear it rather than let it match a freed buffer.
    active_walk_bufnrs.with_mut(|nrs| {
        for nr in nrs.iter_mut().filter(|nr| **nr == buffer.handle) {
            *nr = 0;
        }
    });

    // One pass over the whole table: a buffer is wiped far more often than
    // it has autocommands of its own. The messages wait for the table.
    let mut deleted: Vec<(AutoEvent, usize)> = Vec::new();
    autocmds.with_mut(|table| {
        for (event, cmds) in AutoEvent::all().zip(table.iter_mut()) {
            let mut count = 0;
            for idx in 0..cmds.len() {
                if cmds.cmds[idx]
                    .pat
                    .is_some_and(|id| cmds.pat(id).buflocal_nr == buffer.handle)
                {
                    cmds.delete(idx);
                    count += 1;
                }
            }
            if count > 0 {
                deleted.push((event, count));
            }
        }
    });
    if !deleted.is_empty() {
        au_need_clean.set(true);
    }
    for (event, count) in deleted {
        if p_verbose() >= 6 {
            for _ in 0..count {
                verbose_enter();
                let arg0 = msg_cstr(event_nr2name(event));
                smsg!(
                    0,
                    "auto-removing autocommand: {arg0} <buffer={}>",
                    buffer.handle
                );
                verbose_leave();
            }
        }
    }
    au_cleanup();
}

/// Whether `pat` is one of the buffer-local pattern spellings:
/// `<buffer>`, `<buffer=N>` or `<buffer=abuf>`.
pub fn aupat_is_buflocal(pat: &[u8]) -> bool {
    pat.len() >= 8 && pat.starts_with(b"<buffer") && pat.last() == Some(&b'>')
}

/// The buffer number a buffer-local pattern names, or 0 when it names one
/// that cannot be resolved.
pub fn aupat_get_buflocal_nr(pat: &[u8]) -> ::core::ffi::c_int {
    debug_assert!(aupat_is_buflocal(pat));

    // "<buffer>"
    if pat.len() == 8 {
        return Buf::current().handle;
    }

    if pat.len() > 9 && pat[7] == b'=' {
        // "<buffer=abuf>"
        if pat.len() == 13 && pat.eq_ignore_ascii_case(b"<buffer=abuf>") {
            return autocmd_bufnr.get();
        }
        // "<buffer=123>": digits, and nothing but digits, up to the '>'.
        let digits = &pat[8..pat.len() - 1];
        if digits.iter().all(u8::is_ascii_digit) {
            return atoi_digits(digits);
        }
    }

    0
}

/// `atoi` over a run of ASCII digits: glibc's is `(int)strtol(..)`, which
/// clamps to `LONG_MAX` and then keeps the low 32 bits.
fn atoi_digits(digits: &[u8]) -> ::core::ffi::c_int {
    let long = digits.iter().fold(0i64, |acc, digit| {
        acc.saturating_mul(10)
            .saturating_add(i64::from(digit - b'0'))
    });
    // The truncation is the point: it is the C cast's.
    #[allow(clippy::cast_possible_truncation)]
    let int = long as ::core::ffi::c_int;
    int
}

/// The canonical `<buffer=N>` spelling of a buffer-local pattern.
pub fn aupat_normalize_buflocal_pat(pat: &[u8], buflocal_nr: ::core::ffi::c_int) -> XString {
    debug_assert!(aupat_is_buflocal(pat));
    let nr = if buflocal_nr == 0 {
        Buf::current().handle
    } else {
        buflocal_nr
    };
    XString::from_bytes(format!("<buffer={nr}>").as_bytes())
}

/// `pat` as it is stored: a buffer-local one normalised to `<buffer=N>`,
/// so every comparison against a stored pattern is a plain one.
pub(crate) fn normalized_pattern(pat: &[u8]) -> ::std::borrow::Cow<'_, [u8]> {
    if aupat_is_buflocal(pat) {
        let nr = aupat_get_buflocal_nr(pat);
        ::std::borrow::Cow::Owned(aupat_normalize_buflocal_pat(pat, nr).to_vec())
    } else {
        ::std::borrow::Cow::Borrowed(pat)
    }
}
