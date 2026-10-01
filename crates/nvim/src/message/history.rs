//! The message history behind `:messages` and `'messagesopt'`.
//!
//! A queue of [`Entry`]s capped at `'messagesopt'`'s `history:` count.
//! [`msg_hist_add`] appends and evicts, [`ex_messages`] prints (or, under
//! `ext_messages`, emits) the tail of it.
//!
//! Upstream's list was doubly linked and addressed by pointer from three
//! cursors (first, last, and the `g<` mark), and `:messages` held one of
//! them across `msg_multihl`, which can pump the event loop and run
//! autocommands that add to the history and evict from it. Here every entry
//! has a sequence number instead, the `g<` mark is a sequence number, and
//! `:messages` walks by number: it copies one entry out, shows it with the
//! history unborrowed, and asks for the next number after it -- so an
//! eviction under its feet skips what is gone rather than reading it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::option::vars::P_MOPT;
use crate::options::OptMoptFlags;
use crate::types::Failed;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use std::collections::VecDeque;

/// One message in the history. Owns its chunks.
pub(crate) struct Entry {
    /// Where the entry is in the order of every message ever added.
    seq: u64,
    msg: HlMessage,
    /// The `ext_messages` kind this message was shown under.
    /// [`String_0::NULL`] is "no kind", which is not the empty kind: a UI
    /// reading the history sees the difference.
    kind: String_0,
    /// Only `g<` shows it; the next real message displaces it.
    temp: bool,
    append: bool,
}

impl Drop for Entry {
    fn drop(&mut self) {
        let msg = core::mem::replace(&mut self.msg, EMPTY_HL_MESSAGE);
        // SAFETY: the entry owns its chunks, and nothing else holds them.
        unsafe { hl_msg_free(msg) };
    }
}

/// The history itself.
pub(crate) struct History {
    entries: VecDeque<Entry>,
    /// The sequence number the next entry gets.
    next_seq: u64,
    /// The oldest entry `g<` may still show: the first one added since the
    /// temporary entries were last dropped. `None` until one is.
    temp_from: Option<u64>,
    /// Number of non-temporary entries, which is what `history:` caps.
    len: c_int,
    /// `'messagesopt'`'s `history:` count.
    max: c_int,
}

static HISTORY: GlobalCell<History> = GlobalCell::new(History {
    entries: VecDeque::new(),
    next_seq: 0,
    temp_from: None,
    len: 0,
    max: 500,
});

impl History {
    /// Delete the oldest messages until `keep` non-temporary ones remain.
    /// `keep` of zero empties the list, temporary entries included.
    fn clear(&mut self, keep: c_int) {
        while self.len > keep || (keep == 0 && !self.entries.is_empty()) {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.len -= c_int::from(!oldest.temp);
        }
    }

    /// Drop every temporary (`g<`-only) entry.
    fn clear_temp(&mut self) {
        if let Some(from) = self.temp_from.take() {
            self.entries.retain(|entry| entry.seq < from || !entry.temp);
        }
    }
}

/// The newest message in the history, for the unit specs: its sequence
/// number and the text of its first chunk.
pub fn last_message() -> Option<(u64, Vec<u8>)> {
    HISTORY.with(|history| {
        history.entries.back().map(|entry| {
            let text = if entry.msg.size == 0 {
                Vec::new()
            } else {
                // SAFETY: the entry owns `size` live chunks.
                unsafe { (*entry.msg.items).text.as_bytes().to_vec() }
            };
            (entry.seq, text)
        })
    })
}

/// The `'messagesopt'` items, spelled as [`messagesopt_changed`] matches them.
const OPT_HIT_ENTER: &CStr = c"hit-enter";
const OPT_WAIT: &CStr = c"wait:";
const OPT_HISTORY: &CStr = c"history:";
const OPT_PROGRESS: &CStr = c"progress:";

/// Free a message's chunks and the array holding them.
///
/// # Safety
/// `hl_msg` must own its chunks; nothing else may hold them afterwards.
pub unsafe fn hl_msg_free(hl_msg: HlMessage) {
    for i in 0..hl_msg.size {
        unsafe { xfree((*hl_msg.items.add(i)).text.data().cast()) };
    }
    unsafe { xfree(hl_msg.items.cast()) };
}

/// Add `bytes` to the history, as one chunk in highlight `hl_id`.
pub(crate) fn msg_hist_add(bytes: &[u8], hl_id: c_int) {
    // Remove leading and trailing newlines.
    let text = bytes
        .iter()
        .position(|&byte| byte != b'\n')
        .map_or(&[][..], |first| {
            let last = bytes
                .iter()
                .rposition(|&byte| byte != b'\n')
                .unwrap_or(first);
            &bytes[first..=last]
        });
    if text.is_empty() {
        return;
    }

    let text = String_0::from_bytes(text);
    let mut msg = EMPTY_HL_MESSAGE;
    // SAFETY: `msg` is a live, empty message.
    unsafe { hl_msg_push(&mut msg, HlMessageChunk { text, hl_id }) };
    // SAFETY: `msg` owns its one chunk.
    unsafe { msg_hist_add_multihl(msg, false, ptr::null_mut()) };
}

/// Append an already-chunked message to the history, taking ownership of it.
///
/// A `temp` entry is one only `g<` shows; the next real message displaces it.
///
/// # Safety
/// `msg` must own its chunks.
pub(crate) unsafe fn msg_hist_add_multihl(msg: HlMessage, temp: bool, _msg_data: *mut MessageData) {
    if do_clear_hist_temp.get() {
        HISTORY.with_mut(History::clear_temp);
        do_clear_hist_temp.set(false);
    }

    if msg_hist_off.get() || msg_silent.get() != 0 {
        unsafe { hl_msg_free(msg) };
        return;
    }

    let kind = msg_ext_kind.with(String_0::clone);
    // NOTE: this does not encode whether the message was actually appended
    // to the previous history entry. `append` is currently only true for
    // `:echon`, which is stored as a temporary entry for `g<`, where it is
    // guaranteed to follow the entry it was appended to.
    let append = msg_ext_append.get();
    HISTORY.with_mut(|history| {
        let seq = history.next_seq;
        history.next_seq += 1;
        history.entries.push_back(Entry {
            seq,
            msg,
            kind,
            temp,
            append,
        });
        history.temp_from.get_or_insert(seq);
        history.len += c_int::from(!temp);
        history.clear(history.max);
    });
    msg_ext_history.set(true);
}

/// Delete the oldest messages until `keep` non-temporary ones remain.
///
/// `keep` of zero empties the list, temporary entries included.
fn msg_hist_clear(keep: c_int) {
    HISTORY.with_mut(|history| history.clear(keep));
}

/// Does `p` start with `word`, and with a digit after it if `digit` is set?
///
/// # Safety
/// `p` must be a valid C string.
unsafe fn at_opt(p: *const c_char, word: &CStr, digit: bool) -> bool {
    let matched = unsafe { strnequal(p, word.as_ptr(), word.count_bytes()) };
    matched && (!digit || ascii_isdigit(unsafe { *p.add(word.count_bytes()) as c_int }))
}

/// `'messagesopt'` was set: validate it and adopt it.
///
/// Answers `Err` without changing anything if the value is not usable.
pub fn messagesopt_changed() -> Result<(), Failed> {
    let mut flags: OptMoptFlags = 0;
    let mut wait = 0;
    let mut history = 0;
    let mut progress_target = 0;

    // A copy: the cursor below walks past the end of a projection's borrow.
    let mopt = P_MOPT.get();
    let mut p = mopt.as_ptr().cast_mut();
    while unsafe { *p } != 0 {
        if unsafe { at_opt(p, OPT_HIT_ENTER, false) } {
            p = unsafe { p.add(OPT_HIT_ENTER.count_bytes()) };
            flags |= kOptMoptFlagHitEnter;
        } else if unsafe { at_opt(p, OPT_WAIT, true) } {
            p = unsafe { p.add(OPT_WAIT.count_bytes()) };
            wait = unsafe { getdigits_int(&raw mut p, false, INT_MAX) };
            flags |= kOptMoptFlagWait;
        } else if unsafe { at_opt(p, OPT_HISTORY, true) } {
            p = unsafe { p.add(OPT_HISTORY.count_bytes()) };
            history = unsafe { getdigits_int(&raw mut p, false, INT_MAX) };
            flags |= kOptMoptFlagHistory;
        } else if unsafe { at_opt(p, OPT_PROGRESS, false) } {
            p = unsafe { p.add(OPT_PROGRESS.count_bytes()) };
            flags |= kOptMoptFlagProgress;
            if unsafe { *p } == b'c' as c_char {
                progress_target |= PROGRESS_TARGET_CMD;
                p = unsafe { p.add(1) };
            }
        }
        // An unrecognised item leaves `p` where it was, so this rejects it.
        if unsafe { *p } != b',' as c_char && unsafe { *p } != 0 {
            return Err(Failed);
        }
        if unsafe { *p } == b',' as c_char {
            p = unsafe { p.add(1) };
        }
    }

    // Either "wait" or "hit-enter" is required.
    if flags & (kOptMoptFlagHitEnter | kOptMoptFlagWait) == 0 {
        return Err(Failed);
    }
    // "history" must be set, and both counts must be <= 10000.
    if flags & kOptMoptFlagHistory == 0 {
        return Err(Failed);
    }
    debug_assert!(history >= 0);
    if history > 10000 {
        return Err(Failed);
    }
    debug_assert!(wait >= 0);
    if wait > 10000 {
        return Err(Failed);
    }

    msg_flags.set(flags);
    msg_wait.set(wait);
    progress_msg_target.set(progress_target);

    HISTORY.with_mut(|entries| {
        entries.max = history;
        entries.clear(history);
    });

    Ok(())
}

/// One history entry, copied out so it can be shown with the history
/// unborrowed.
struct Shown {
    seq: u64,
    /// Deep: the chunks' strings are the copy's own.
    chunks: Vec<HlMessageChunk>,
    kind: String_0,
    temp: bool,
    append: bool,
}

/// The first entry numbered `seq` or later, copied.
fn entry_from(seq: u64) -> Option<Shown> {
    HISTORY.with(|history| {
        let at = history.entries.partition_point(|entry| entry.seq < seq);
        history.entries.get(at).map(|entry| Shown {
            seq: entry.seq,
            // SAFETY: the entry owns `size` live chunks.
            chunks: (0..entry.msg.size)
                .map(|i| unsafe { (*entry.msg.items.add(i)).clone() })
                .collect(),
            kind: entry.kind.clone(),
            temp: entry.temp,
            append: entry.append,
        })
    })
}

/// One history entry as the `msg_history_show` UI event carries it:
/// `[kind, [[attr, text, hl_id], ..], append]`.
fn entry_to_event(entry: &Shown) -> Object {
    let mut content = EMPTY_ARRAY;
    let mut out = EMPTY_ARRAY;
    for chunk in &entry.chunks {
        let attr = if chunk.hl_id != 0 {
            syn_id2attr(chunk.hl_id)
        } else {
            0
        };
        let mut content_entry = EMPTY_ARRAY;
        content_entry.push(Object::integer(attr.into()));
        content_entry.push(Object::string(chunk.text.clone()));
        content_entry.push(Object::integer(chunk.hl_id.into()));
        content.push(Object::array(content_entry));
    }

    out.push(Object::string(entry.kind.clone()));
    out.push(Object::array(content));
    out.push(Object::boolean(entry.append));
    Object::array(out)
}

/// `:messages`.
pub fn ex_messages(excmd: &mut ExArg) {
    if excmd.line.arg() == b"clear" {
        let keep = if excmd.addr_count != 0 {
            excmd.line2 as c_int
        } else {
            0
        };
        msg_hist_clear(keep);
        return;
    }
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        emsg(gettext(e_invarg));
        return;
    }

    let mut entries = EMPTY_ARRAY;
    // `g<` starts at its mark, which may be unset; `:messages` at the start.
    let mut next = if excmd.skip {
        HISTORY.with(|history| history.temp_from)
    } else {
        Some(0)
    };
    let mut skip = if excmd.addr_count != 0 {
        HISTORY.with(|history| history.len) - excmd.line2 as c_int
    } else {
        0
    };

    while let Some(entry) = next.and_then(entry_from) {
        next = Some(entry.seq + 1);
        // Skip over count or temporary "g<" messages. The decrement sits
        // inside the short circuit: a temporary entry does not consume one
        // of the counted lines.
        let temporary = entry.temp && !excmd.skip;
        let counted_out = !temporary && {
            let remaining = skip;
            skip -= 1;
            remaining > 0
        };
        if temporary || counted_out {
            continue;
        }
        if ui_has(kUIMessages) && msg_silent.get() == 0 {
            entries.push(entry_to_event(&entry));
        }
        if redirecting() || !ui_has(kUIMessages) {
            // Under ext_messages the text has already gone to the UI
            // above; this pass exists only to feed the redirection, so
            // silence the display half of it.  `ui_has` is asked twice,
            // as upstream does, and deliberately not hoisted into a
            // local: `msg_multihl` can reach `wait_return`, which pumps
            // the event loop, which can service a UI attach or detach.
            msg_silent.set(msg_silent.get() + c_int::from(ui_has(kUIMessages)));
            let mut needs_clear = false;
            let mut text = EMPTY_HL_MESSAGE;
            for chunk in entry.chunks {
                // SAFETY: `text` started empty and is only pushed to here.
                unsafe { hl_msg_push(&mut text, chunk) };
            }
            let clear = &raw mut needs_clear;
            let kind = (!entry.kind.is_null()).then(|| entry.kind.as_cstr());
            // SAFETY: `text` is this frame's own copy, and `clear` a live
            // `bool`.
            unsafe {
                msg_multihl(
                    Object::Nil,
                    text.clone(),
                    kind,
                    false,
                    false,
                    ptr::null_mut(),
                    clear,
                )
            };
            // SAFETY: not kept (`history` is false), so still this frame's.
            unsafe { hl_msg_free(text) };
            msg_silent.set(msg_silent.get() - c_int::from(ui_has(kUIMessages)));
        }
    }

    if !entries.is_empty() {
        ui_call_msg_history_show(entries, excmd.skip);
    }
}
