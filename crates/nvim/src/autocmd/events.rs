//! The event table's readers: name to number, number to name,
//! 'eventignore'.
//!
//! [`event_name2nr`] is the one parse -- it measures a name up to the first
//! whitespace, comma or bar and binary searches [`EVENT_NAMES`] for it, and
//! every other spelling of a lookup goes through it or its `String_0`
//! twin.  [`event_ignored`] and [`check_ei`] are the 'eventignore' half:
//! whether an event is currently suppressed, and whether an option value
//! is a list of real event names.  [`au_event_disable`]/[`au_event_restore`]
//! are the save-and-restore pair the rest of the editor brackets a
//! side-effecting operation with.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK, OptionSetFlags};

/// The [`EVENT_NAMES`] row an event number names.
///
/// Every caller has an event that came out of the table, so the index is
/// in range; `event_nr2name` is the one place a caller can be holding
/// something else, and it bounds-checks for itself.
pub(super) fn event_row(event: event_T) -> &'static EventName {
    &EVENT_NAMES[event as usize]
}

/// The event named by the head of `start`: up to the first whitespace,
/// comma or bar.  `*end` is left just past the name and its comma, so a
/// caller can walk a list; `NUM_EVENTS` means no event is spelled that way.
pub unsafe fn event_name2nr(
    start: *const ::core::ffi::c_char,
    end: *mut *mut ::core::ffi::c_char,
) -> event_T {
    unsafe {
        let rest = CStr::from_ptr(start).to_bytes();
        let len = rest
            .iter()
            .position(|&c| ascii_iswhite(c as ::core::ffi::c_int) || c == b',' || c == b'|')
            .unwrap_or(rest.len());
        let found = event_name_index(&rest[..len]);
        // A separating comma belongs to the name just measured, not to the
        // next one; anything else the caller has to step over itself.
        let consumed = len + usize::from(rest.get(len) == Some(&b','));
        *end = start.add(consumed).cast_mut();
        match found {
            Some(at) => EVENT_NAMES[at].event.unsigned_abs(),
            None => NUM_EVENTS,
        }
    }
}

/// [`event_name2nr`] over a counted string, which is the whole name.
pub unsafe fn event_name2nr_str(str: String_0) -> event_T {
    // An empty API string has a null `data`, which is not a valid pointer
    // even for a zero-length slice.
    let wanted: &[u8] = if str.size == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(str.data.cast::<u8>(), str.size) }
    };
    match event_name_index(wanted) {
        Some(at) => EVENT_NAMES[at].event.unsigned_abs(),
        None => NUM_EVENTS,
    }
}

/// The canonical name of `event`, or `"Unknown"` for an event number that
/// is not one.
///
/// Upstream also asks `event >= 0`, which `event_T` being unsigned makes
/// vacuous; the answers are the same either way, so the test stays gone.
pub fn event_nr2name(event: event_T) -> *const ::core::ffi::c_char {
    let name = match EVENT_NAMES.get(event as usize) {
        Some(row) => row.name,
        None => c"Unknown",
    };
    name.as_ptr()
}

/// Whether `ei` -- a value of 'eventignore' or 'eventignorewin' -- names
/// `event`.
///
/// The list is scanned left to right and the *last* mention wins, except
/// that a `-name` exclusion answers immediately.  `all` in 'eventignorewin'
/// covers only the window-local events, which is what the sign of a row's
/// `event` records.
pub unsafe fn event_ignored(event: event_T, mut ei: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut ignored = false;
        while *ei != 0 {
            let unignore = *ei == b'-' as ::core::ffi::c_char;
            ei = ei.add(usize::from(unignore));
            if let Some(after_all) = skip_all(ei) {
                ignored = ei == p_ei.get() || event_row(event).event <= 0;
                ei = after_all;
            } else if event_name2nr(ei, &raw mut ei) == event {
                if unignore {
                    return false;
                }
                ignored = true;
            }
        }
        ignored
    }
}

/// `OK` when `ei` -- a value of 'eventignore' or 'eventignorewin' -- is a
/// list of event names, `FAIL` otherwise.
///
/// 'eventignorewin' is the value that is not `p_ei`, and it accepts only
/// the window-local events.
pub unsafe fn check_ei(mut ei: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let win = ei != p_ei.get();
        while *ei != 0 {
            if let Some(after_all) = skip_all(ei) {
                ei = after_all;
            } else {
                ei = ei.add(usize::from(*ei == b'-' as ::core::ffi::c_char));
                let event = event_name2nr(ei, &raw mut ei);
                if event == NUM_EVENTS || (win && event_row(event).event > 0) {
                    return FAIL;
                }
            }
        }
        OK
    }
}

/// The rest of `ei` past a leading `all` item, or `None` when there is not
/// one.
///
/// `strncasecmp` rather than an ASCII fold because upstream's is the
/// locale's, and it stops at the first mismatch -- so a string shorter than
/// `all` is rejected without `ei[3]` ever being read.
unsafe fn skip_all(ei: *mut ::core::ffi::c_char) -> Option<*mut ::core::ffi::c_char> {
    unsafe {
        if strncasecmp(ei, c"all".as_ptr(), 3) != 0 {
            return None;
        }
        let after = *ei.add(3);
        if after != 0 && after != b',' as ::core::ffi::c_char {
            return None;
        }
        Some(ei.add(3 + usize::from(after == b',' as ::core::ffi::c_char)))
    }
}

/// Append `what` (which starts with a comma) to 'eventignore', and answer
/// the old value in allocated memory for [`au_event_restore`].
pub unsafe fn au_event_disable(what: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe {
        let p_ei_len = strlen(p_ei.get());
        let save_ei = xmemdupz(p_ei.get().cast::<::core::ffi::c_void>(), p_ei_len)
            .cast::<::core::ffi::c_char>();
        let new_ei = xstrnsave(p_ei.get(), p_ei_len.wrapping_add(strlen(what)));
        if *what == b',' as ::core::ffi::c_char && *p_ei.get() == 0 {
            strcpy(new_ei, what.add(1));
        } else {
            strcpy(new_ei.add(p_ei_len), what);
        }
        set_option_eventignore(new_ei);
        xfree(new_ei.cast::<::core::ffi::c_void>());
        save_ei
    }
}

/// Put back what [`au_event_disable`] saved, and free it.
pub unsafe fn au_event_restore(old_ei: *mut ::core::ffi::c_char) {
    unsafe {
        if !old_ei.is_null() {
            set_option_eventignore(old_ei);
            xfree(old_ei.cast::<::core::ffi::c_void>());
        }
    }
}

/// Set 'eventignore' to a NUL-terminated string, without an owner.
unsafe fn set_option_eventignore(value: *mut ::core::ffi::c_char) {
    unsafe {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(value),
                },
            },
            OptionSetFlags::NONE,
            SID_NONE,
        );
    }
}

/// Whether any autocommand is defined for `event`.
pub unsafe fn has_event(event: event_T) -> bool {
    unsafe { (*autocmds.ptr())[event as usize].size != 0 }
}

/// Whether a `CursorHold` autocommand exists for the mode we are in.
unsafe fn has_cursorhold() -> bool {
    unsafe {
        has_event(if get_real_state() == MODE_NORMAL_BUSY {
            EVENT_CURSORHOLD
        } else {
            EVENT_CURSORHOLDI
        })
    }
}

/// Whether `CursorHold` should fire now: one is defined, nothing else is
/// pending, and we are in a mode that has one.
pub unsafe fn trigger_cursorhold() -> bool {
    unsafe {
        if did_cursorhold.get()
            || !has_cursorhold()
            || reg_recording.get() != 0
            || (*typebuf.ptr()).tb_len != 0
            || ins_compl_active()
        {
            return false;
        }
        let state = get_real_state();
        state == MODE_NORMAL_BUSY || state & MODE_INSERT != 0
    }
}

/// Completion source for `:autocmd`'s event argument: the augroup names
/// first (when [`autocmd_include_groups`] is set), then every event name.
pub unsafe fn expand_get_event_name(
    _xp: *mut expand_T,
    idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let name = augroup_name(idx + 1);
        if !name.is_null() {
            // Skip the group, but keep it in the numbering: the caller
            // walks `idx` up until this answers null.
            if !autocmd_include_groups.get() || name.cast_const() == get_deleted_augroup() {
                return c"".as_ptr().cast_mut();
            }
            return name;
        }
        match usize::try_from(idx - next_augroup_id.get()) {
            Ok(i) if i < EVENT_NAMES.len() => EVENT_NAMES[i].name.as_ptr().cast_mut(),
            _ => ::core::ptr::null_mut(),
        }
    }
}

/// Completion source for an 'eventignore' item: every event name, or --
/// for 'eventignorewin' (`win`) -- only the window-local ones.
pub fn get_event_name_no_group(
    _xp: *mut expand_T,
    idx: ::core::ffi::c_int,
    win: bool,
) -> *mut ::core::ffi::c_char {
    let Ok(idx) = usize::try_from(idx) else {
        return ::core::ptr::null_mut();
    };
    if idx >= EVENT_NAMES.len() {
        return ::core::ptr::null_mut();
    }
    if !win {
        return EVENT_NAMES[idx].name.as_ptr().cast_mut();
    }
    match EVENT_NAMES.iter().filter(|row| row.event <= 0).nth(idx) {
        Some(row) => row.name.as_ptr().cast_mut(),
        None => ::core::ptr::null_mut(),
    }
}

/// Whether `event` -- a NUL-terminated name -- is an event nvim has.
pub unsafe fn autocmd_supported(event: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        event_name2nr(event, &raw mut end) != NUM_EVENTS
    }
}

/// ASCII-case-folded comparison, the order [`EVENT_NAMES`] is in.
fn cmp_ignore_ascii_case(a: &[u8], b: &[u8]) -> Ordering {
    a.iter()
        .map(u8::to_ascii_lowercase)
        .cmp(b.iter().map(u8::to_ascii_lowercase))
}

/// The index into [`EVENT_NAMES`] of the event spelled `wanted`, folding
/// ASCII case.  The whole name has to match -- a string that merely starts
/// with an event's name is not one.
///
/// Upstream reaches the same rows through a generated perfect hash
/// (`v0.12.4`'s `src/gen/gen_events.lua` calls `gen.hashy`, which emits a
/// ~400-line `switch` on the name's length and one discriminating byte,
/// plus a permutation table `event_hash` to undo the bucket order).  None
/// of that is needed: `gen_events.lua` sorts the names by `name:lower()`
/// before it numbers the enum, so [`EVENT_NAMES`] is *already* the sorted
/// table and a binary search over it answers the same lookup.
fn event_name_index(wanted: &[u8]) -> Option<usize> {
    EVENT_NAMES
        .binary_search_by(|row| cmp_ignore_ascii_case(row.name.to_bytes(), wanted))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The binary search is only a lookup while the table is sorted the way
    /// `gen_events.lua` emitted it.  Nothing else asserts this: the rows are
    /// hand-transcribed, and a misplaced one would silently stop being
    /// findable rather than fail to compile.
    #[test]
    fn event_names_are_sorted_case_insensitively() {
        for pair in EVENT_NAMES.windows(2) {
            let [a, b] = pair else { unreachable!() };
            assert_eq!(
                cmp_ignore_ascii_case(a.name.to_bytes(), b.name.to_bytes()),
                Ordering::Less,
                "{:?} must sort before {:?}",
                a.name,
                b.name
            );
        }
    }

    /// Every row's index is the `EVENT_*` constant of that *name*, which is
    /// what `event_nr2name` indexes with, and its `event` is the canonical
    /// event that name resolves to.
    #[test]
    fn every_name_finds_its_own_row() {
        for (i, row) in EVENT_NAMES.iter().enumerate() {
            assert_eq!(event_name_index(row.name.to_bytes()), Some(i));
            assert!(row.event.unsigned_abs() < NUM_EVENTS);
        }
    }
}
