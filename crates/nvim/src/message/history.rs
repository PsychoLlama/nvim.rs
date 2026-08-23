//! The message history behind `:messages` and `'messagesopt'`.
//!
//! A doubly-linked list of [`MessageHistoryEntry`] capped at
//! `'messagesopt'`'s `history:` count. [`msg_hist_add`] appends and evicts,
//! [`ex_messages`] prints (or, under `ext_messages`, emits) the tail of it.
//!
//! The list stays a raw `repr(C)` chain rather than becoming a `Vec`: every
//! entry is addressed by pointer from three cursors at once (first, last and
//! the `g<` mark), and [`ex_messages`] holds one across [`msg_multihl`],
//! which can run autocommands that add to the history.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Oldest entry in the history.
static msg_hist_first: GlobalCell<*mut MessageHistoryEntry> = GlobalCell::new(ptr::null_mut());

/// Newest entry in the history. Exported for the unit specs.
pub static msg_hist_last: GlobalCell<*mut MessageHistoryEntry> = GlobalCell::new(ptr::null_mut());

/// Oldest entry `g<` may still show: the temporary entries start here.
static msg_hist_temp: GlobalCell<*mut MessageHistoryEntry> = GlobalCell::new(ptr::null_mut());

/// Number of non-temporary entries, which is what `history:` caps.
static msg_hist_len: GlobalCell<c_int> = GlobalCell::new(0);

/// `'messagesopt'`'s `history:` count.
static msg_hist_max: GlobalCell<c_int> = GlobalCell::new(500);

/// Drop the temporary entries before adding the next message.
pub(crate) static do_clear_hist_temp: GlobalCell<bool> = GlobalCell::new(true);

/// `'messagesopt'`'s flag set.
pub(crate) static msg_flags: GlobalCell<c_int> = GlobalCell::new(
    kOptMoptFlagHitEnter as c_int | kOptMoptFlagHistory as c_int | kOptMoptFlagProgress as c_int,
);

/// `'messagesopt'`'s `wait:` delay, in milliseconds.
pub(crate) static msg_wait: GlobalCell<c_int> = GlobalCell::new(0);

/// Where `'messagesopt'`'s `progress:` sends progress messages.
pub(crate) static progress_msg_target: GlobalCell<c_int> = GlobalCell::new(PROGRESS_TARGET_CMD);

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
    unsafe {
        for i in 0..hl_msg.size {
            xfree((*hl_msg.items.add(i)).text.data().cast());
        }
        xfree(hl_msg.items.cast());
    }
}

/// Add `s` (of `len` bytes, or -1 for the whole string) to the history.
///
/// # Safety
/// `s` must be a valid C string, readable for `len` bytes when that is not
/// negative.
pub(crate) unsafe fn msg_hist_add(s: *const c_char, len: c_int, hl_id: c_int) {
    unsafe {
        let mut start = s;
        let mut size = if len < 0 { strlen(s) } else { len as size_t };
        // Remove leading and trailing newlines.
        while size > 0 && *start == b'\n' as c_char {
            size -= 1;
            start = start.add(1);
        }
        while size > 0 && *start.add(size - 1) == b'\n' as c_char {
            size -= 1;
        }
        if size == 0 {
            return;
        }

        let text = String_0::from_raw_parts(xmemdupz(start.cast(), size).cast(), size);
        let mut msg = EMPTY_HL_MESSAGE;
        hl_msg_push(&mut msg, HlMessageChunk { text, hl_id });
        msg_hist_add_multihl(msg, false, ptr::null_mut());
    }
}

/// Append an already-chunked message to the history, taking ownership of it.
///
/// A `temp` entry is one only `g<` shows; the next real message displaces it.
///
/// # Safety
/// `msg` must own its chunks.
pub(crate) unsafe fn msg_hist_add_multihl(msg: HlMessage, temp: bool, _msg_data: *mut MessageData) {
    unsafe {
        if do_clear_hist_temp.get() {
            msg_hist_clear_temp();
            do_clear_hist_temp.set(false);
        }

        if msg_hist_off.get() || msg_silent.get() != 0 {
            hl_msg_free(msg);
            return;
        }

        let entry: *mut MessageHistoryEntry =
            xmalloc(::core::mem::size_of::<MessageHistoryEntry>()).cast();
        (*entry).msg = msg;
        (*entry).temp = temp;
        (*entry).kind = if msg_ext_kind.get().is_null() {
            ptr::null_mut()
        } else {
            xstrdup(msg_ext_kind.get())
        };
        (*entry).prev = msg_hist_last.get();
        (*entry).next = ptr::null_mut();
        // NOTE: this does not encode whether the message was actually appended
        // to the previous history entry. `append` is currently only true for
        // `:echon`, which is stored as a temporary entry for `g<`, where it is
        // guaranteed to follow the entry it was appended to.
        (*entry).append = msg_ext_append.get();

        if msg_hist_first.get().is_null() {
            msg_hist_first.set(entry);
        }
        if !msg_hist_last.get().is_null() {
            (*msg_hist_last.get()).next = entry;
        }
        if msg_hist_temp.get().is_null() {
            msg_hist_temp.set(entry);
        }

        msg_hist_len.set(msg_hist_len.get() + c_int::from(!temp));
        msg_hist_last.set(entry);
        msg_ext_history.set(true);

        msg_hist_clear(msg_hist_max.get());
    }
}

/// Unlink `entry` from the list and free it.
///
/// # Safety
/// `entry` must be in the history list.
unsafe fn msg_hist_free_msg(entry: *mut MessageHistoryEntry) {
    unsafe {
        if (*entry).next.is_null() {
            msg_hist_last.set((*entry).prev);
        } else {
            (*(*entry).next).prev = (*entry).prev;
        }
        if (*entry).prev.is_null() {
            msg_hist_first.set((*entry).next);
        } else {
            (*(*entry).prev).next = (*entry).next;
        }
        if entry == msg_hist_temp.get() {
            msg_hist_temp.set((*entry).next);
        }
        hl_msg_free((*entry).msg.clone());
        xfree((*entry).kind.cast());
        xfree(entry.cast());
    }
}

/// Delete the oldest messages until `keep` non-temporary ones remain.
///
/// `keep` of zero empties the list, temporary entries included.
///
/// # Safety
/// Only that the history list is well formed.
unsafe fn msg_hist_clear(keep: c_int) {
    unsafe {
        while msg_hist_len.get() > keep || (keep == 0 && !msg_hist_first.get().is_null()) {
            msg_hist_len.set(msg_hist_len.get() - c_int::from(!(*msg_hist_first.get()).temp));
            msg_hist_free_msg(msg_hist_first.get());
        }
    }
}

/// Drop every temporary (`g<`-only) entry.
///
/// # Safety
/// Only that the history list is well formed.
unsafe fn msg_hist_clear_temp() {
    unsafe {
        while !msg_hist_temp.get().is_null() {
            let next = (*msg_hist_temp.get()).next;
            if (*msg_hist_temp.get()).temp {
                msg_hist_free_msg(msg_hist_temp.get());
            }
            msg_hist_temp.set(next);
        }
    }
}

/// Does `p` start with `word`, and with a digit after it if `digit` is set?
///
/// # Safety
/// `p` must be a valid C string.
unsafe fn at_opt(p: *const c_char, word: &CStr, digit: bool) -> bool {
    unsafe {
        strnequal(p, word.as_ptr(), word.count_bytes())
            && (!digit || ascii_isdigit(*p.add(word.count_bytes()) as c_int))
    }
}

/// `'messagesopt'` was set: validate it and adopt it.
///
/// Answers `FAIL` without changing anything if the value is not usable.
///
/// # Safety
/// Only that `p_mopt` holds a valid string.
pub unsafe fn messagesopt_changed() -> c_int {
    unsafe {
        let mut flags = 0;
        let mut wait = 0;
        let mut history = 0;
        let mut progress_target = 0;

        let mut p = p_mopt.get();
        while *p != 0 {
            if at_opt(p, OPT_HIT_ENTER, false) {
                p = p.add(OPT_HIT_ENTER.count_bytes());
                flags |= kOptMoptFlagHitEnter as c_int;
            } else if at_opt(p, OPT_WAIT, true) {
                p = p.add(OPT_WAIT.count_bytes());
                wait = getdigits_int(&raw mut p, false, INT_MAX);
                flags |= kOptMoptFlagWait as c_int;
            } else if at_opt(p, OPT_HISTORY, true) {
                p = p.add(OPT_HISTORY.count_bytes());
                history = getdigits_int(&raw mut p, false, INT_MAX);
                flags |= kOptMoptFlagHistory as c_int;
            } else if at_opt(p, OPT_PROGRESS, false) {
                p = p.add(OPT_PROGRESS.count_bytes());
                flags |= kOptMoptFlagProgress as c_int;
                if *p == b'c' as c_char {
                    progress_target |= PROGRESS_TARGET_CMD;
                    p = p.add(1);
                }
            }
            // An unrecognised item leaves `p` where it was, so this rejects it.
            if *p != b',' as c_char && *p != 0 {
                return FAIL;
            }
            if *p == b',' as c_char {
                p = p.add(1);
            }
        }

        // Either "wait" or "hit-enter" is required.
        if flags & (kOptMoptFlagHitEnter as c_int | kOptMoptFlagWait as c_int) == 0 {
            return FAIL;
        }
        // "history" must be set, and both counts must be <= 10000.
        if flags & kOptMoptFlagHistory as c_int == 0 {
            return FAIL;
        }
        debug_assert!(history >= 0);
        if history > 10000 {
            return FAIL;
        }
        debug_assert!(wait >= 0);
        if wait > 10000 {
            return FAIL;
        }

        msg_flags.set(flags);
        msg_wait.set(wait);
        progress_msg_target.set(progress_target);

        msg_hist_max.set(history);
        msg_hist_clear(msg_hist_max.get());

        OK
    }
}

/// One history entry as the `msg_history_show` UI event carries it:
/// `[kind, [[attr, text, hl_id], ..], append]`.
///
/// # Safety
/// `entry` must be in the history list.
unsafe fn entry_to_event(entry: *mut MessageHistoryEntry) -> Object {
    unsafe {
        let mut content = EMPTY_ARRAY;
        for i in 0..(*entry).msg.size {
            let chunk = (*(*entry).msg.items.add(i)).clone();
            let attr = if chunk.hl_id != 0 {
                syn_id2attr(chunk.hl_id)
            } else {
                0
            };
            let mut content_entry = EMPTY_ARRAY;
            array_push(&mut content_entry, Object::integer(attr.into()));
            array_push(
                &mut content_entry,
                Object::string(copy_string(chunk.text, ptr::null_mut())),
            );
            array_push(&mut content_entry, Object::integer(chunk.hl_id.into()));
            array_push(&mut content, Object::array(content_entry));
        }

        let mut out = EMPTY_ARRAY;
        array_push(&mut out, Object::string(cstr_to_string((*entry).kind)));
        array_push(&mut out, Object::array(content));
        array_push(&mut out, Object::boolean((*entry).append));
        Object::array(out)
    }
}

/// `:messages`.
///
/// # Safety
/// `eap` must point at a valid command argument block.
pub unsafe fn ex_messages(eap: *mut exarg_T) {
    unsafe {
        if strcmp((*eap).arg, c"clear".as_ptr()) == 0 {
            msg_hist_clear(if (*eap).addr_count != 0 {
                (*eap).line2 as c_int
            } else {
                0
            });
            return;
        }
        if *(*eap).arg != 0 {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }

        let mut entries = EMPTY_ARRAY;
        let mut p = if (*eap).skip != 0 {
            msg_hist_temp.get()
        } else {
            msg_hist_first.get()
        };
        let mut skip = if (*eap).addr_count != 0 {
            msg_hist_len.get() - (*eap).line2 as c_int
        } else {
            0
        };

        while !p.is_null() {
            // Skip over count or temporary "g<" messages. The decrement sits
            // inside the short circuit: a temporary entry does not consume one
            // of the counted lines.
            let temporary = (*p).temp && (*eap).skip == 0;
            let counted_out = !temporary && {
                let remaining = skip;
                skip -= 1;
                remaining > 0
            };
            if !temporary && !counted_out {
                if ui_has(kUIMessages) && msg_silent.get() == 0 {
                    array_push(&mut entries, entry_to_event(p));
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
                    msg_multihl(
                        Object::NIL,
                        (*p).msg.clone(),
                        (*p).kind,
                        false,
                        false,
                        ptr::null_mut(),
                        &raw mut needs_clear,
                    );
                    msg_silent.set(msg_silent.get() - c_int::from(ui_has(kUIMessages)));
                }
            }
            p = (*p).next;
        }

        if entries.size > 0 {
            ui_call_msg_history_show(entries, (*eap).skip != 0);
            api_free_array(entries);
        }
    }
}
