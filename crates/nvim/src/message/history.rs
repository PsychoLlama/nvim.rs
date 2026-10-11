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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::getdigits_int_at;
use crate::option::vars::P_MOPT;
use crate::options::OptMoptFlags;
use crate::types::Failed;
use core::ffi::{CStr, c_int};
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
            let text = entry
                .msg
                .first()
                .map_or_else(Vec::new, |chunk| chunk.text.as_bytes().to_vec());
            (entry.seq, text)
        })
    })
}

/// The `'messagesopt'` items, spelled as [`messagesopt_changed`] matches them.
const OPT_HIT_ENTER: &CStr = c"hit-enter";
const OPT_WAIT: &CStr = c"wait:";
const OPT_HISTORY: &CStr = c"history:";
const OPT_PROGRESS: &CStr = c"progress:";

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
    msg_hist_add_chunks([HlMessageChunk { text, hl_id }], false);
}

/// [`msg_hist_add_multihl`] for a message given as its chunks.
pub(crate) fn msg_hist_add_chunks(chunks: impl IntoIterator<Item = HlMessageChunk>, temp: bool) {
    msg_hist_add_multihl(chunks.into_iter().collect(), temp);
}

/// Append an already-chunked message to the history, taking ownership of it.
///
/// A `temp` entry is one only `g<` shows; the next real message displaces it.
pub(crate) fn msg_hist_add_multihl(msg: HlMessage, temp: bool) {
    if do_clear_hist_temp.get() {
        HISTORY.with_mut(History::clear_temp);
        do_clear_hist_temp.set(false);
    }

    if msg_hist_off.get() || msg_silent.get() != 0 {
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

/// Does `text` start with `word` at `at`, and with a digit after it if
/// `digit` is set?
fn item_at(text: &[u8], at: usize, word: &CStr, digit: bool) -> bool {
    let word = word.to_bytes();
    text[at..].starts_with(word)
        && (!digit || cstr::byte_at(text, at + word.len()).is_ascii_digit())
}

/// `'messagesopt'` was set: validate it and adopt it.
///
/// Answers `Err` without changing anything if the value is not usable.
pub fn messagesopt_changed() -> Result<(), Failed> {
    let mut flags: OptMoptFlags = 0;
    let mut wait = 0;
    let mut history = 0;
    let mut progress_target = 0;

    // A copy: the walk below would otherwise hold the option's borrow.
    let mopt = P_MOPT.get();
    let text = mopt.as_cstr().to_bytes();
    let mut at = 0;
    while at < text.len() {
        if item_at(text, at, OPT_HIT_ENTER, false) {
            at += OPT_HIT_ENTER.count_bytes();
            flags |= kOptMoptFlagHitEnter;
        } else if item_at(text, at, OPT_WAIT, true) {
            (wait, at) = getdigits_int_at(text, at + OPT_WAIT.count_bytes(), false, INT_MAX);
            flags |= kOptMoptFlagWait;
        } else if item_at(text, at, OPT_HISTORY, true) {
            (history, at) = getdigits_int_at(text, at + OPT_HISTORY.count_bytes(), false, INT_MAX);
            flags |= kOptMoptFlagHistory;
        } else if item_at(text, at, OPT_PROGRESS, false) {
            at += OPT_PROGRESS.count_bytes();
            flags |= kOptMoptFlagProgress;
            if cstr::byte_at(text, at) == b'c' {
                progress_target |= PROGRESS_TARGET_CMD;
                at += 1;
            }
        }
        // An unrecognised item leaves `at` where it was, so this rejects it.
        match cstr::byte_at(text, at) {
            b',' => at += 1,
            0 => {}
            _ => return Err(Failed),
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
            chunks: entry.msg.clone(),
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
            let kind = (!entry.kind.is_null()).then(|| entry.kind.as_cstr());
            let shown = &entry.chunks;
            msg_multihl(
                Object::Nil,
                shown,
                kind,
                false,
                false,
                None,
                &mut needs_clear,
            );
            msg_silent.set(msg_silent.get() - c_int::from(ui_has(kUIMessages)));
        }
    }

    if !entries.is_empty() {
        ui_call_msg_history_show(entries, excmd.skip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(entries: &[bool]) -> History {
        let mut history = History {
            entries: VecDeque::new(),
            next_seq: 0,
            temp_from: None,
            len: 0,
            max: 500,
        };
        for &temp in entries {
            let seq = history.next_seq;
            history.next_seq += 1;
            history.entries.push_back(Entry {
                seq,
                msg: Vec::new(),
                kind: String_0::NULL,
                temp,
                append: false,
            });
            history.temp_from.get_or_insert(seq);
            history.len += c_int::from(!temp);
        }
        history
    }

    fn seqs(history: &History) -> Vec<u64> {
        history.entries.iter().map(|entry| entry.seq).collect()
    }

    #[test]
    fn clearing_keeps_the_newest_counted_entries() {
        let mut history = history(&[false, true, false, false]);
        history.clear(2);
        // The oldest counted entry goes, and that is enough: the temporary
        // one now at the front is not counted, so it stays.
        assert_eq!(seqs(&history), [1, 2, 3]);
        assert_eq!(history.len, 2);
        history.clear(0);
        assert!(history.entries.is_empty());
        assert_eq!(history.len, 0);
    }

    #[test]
    fn the_temporary_entries_go_from_the_mark_on() {
        let mut history = history(&[false, true, false, true]);
        history.temp_from = Some(2);
        history.clear_temp();
        // Seq 1 is before the mark, so it stays; seq 3 is after it.
        assert_eq!(seqs(&history), [0, 1, 2]);
        assert_eq!(history.temp_from, None);
    }
}
