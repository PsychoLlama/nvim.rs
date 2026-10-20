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

#![forbid(unsafe_code)]

use super::*;
use crate::getchar::typeahead;
use crate::memory::XString;
use crate::option::vars::P_EI;
use crate::types::Candidate;
use crate::types::{Failed, OptionSetFlags};

/// The [`EVENT_NAMES`] row an event number names.
///
/// Every caller has an event that came out of the table, so the index is
/// in range; `event_nr2name` is the one place a caller can be holding
/// something else, and it bounds-checks for itself.
pub(super) fn event_row(event: AutoEvent) -> &'static EventName {
    &EVENT_NAMES[event.index()]
}

/// The event named by the head of `text`: up to the first whitespace,
/// comma, bar or NUL. The second answer is how far to step past the name
/// and its comma, so a caller can walk a list; `None` means no event is
/// spelled that way, which upstream said with `NUM_EVENTS`.
pub fn event_name2nr(text: &[u8]) -> (Option<AutoEvent>, usize) {
    let len = text
        .iter()
        .position(|&c| {
            c == 0 || ascii_iswhite(::core::ffi::c_int::from(c)) || c == b',' || c == b'|'
        })
        .unwrap_or(text.len());
    let found = event_name_index(&text[..len]);
    // A separating comma belongs to the name just measured, not to the
    // next one; anything else the caller has to step over itself.
    let consumed = len + usize::from(text.get(len) == Some(&b','));
    (found.map(|at| EVENT_NAMES[at].event), consumed)
}

/// [`event_name2nr`] over a counted string, which is the whole name.
pub fn event_name2nr_str(str: &String_0) -> Option<AutoEvent> {
    event_name_index(str.as_bytes()).map(|at| EVENT_NAMES[at].event)
}

/// The canonical name of `event`.
///
/// Upstream took an `int` here and answered `"Unknown"` for a number that
/// was not an event; every event is one now, so the fallback is gone.
pub fn event_nr2name(event: AutoEvent) -> &'static ::core::ffi::CStr {
    EVENT_NAMES[event.index()].name
}

/// Whether `ei` -- a value of 'eventignore' or 'eventignorewin' -- names
/// `event`.
///
/// The list is scanned left to right and the *last* mention wins, except
/// that a `-name` exclusion answers immediately.  `all` in 'eventignorewin'
/// covers only the window-local events, which is what a row's `win_local`
/// records.
///
/// `window_local` says which of the two options `ei` is, which upstream
/// asked by comparing the pointer against `p_ei` -- a question the option's
/// value cannot answer now that it owns its storage, and that the caller
/// knew all along.
pub fn event_ignored(event: AutoEvent, ei: &[u8], window_local: bool) -> bool {
    let mut ignored = false;
    let mut at = 0;
    while at < ei.len() && ei[at] != 0 {
        let unignore = ei[at] == b'-';
        at += usize::from(unignore);
        if let Some(all) = skip_all(&ei[at..]) {
            ignored = !window_local || event_row(event).win_local;
            at += all;
        } else {
            let (found, consumed) = event_name2nr(&ei[at..]);
            at += consumed;
            if found == Some(event) {
                if unignore {
                    return false;
                }
                ignored = true;
            }
        }
    }
    ignored
}

/// `Ok` when `ei` -- a value of 'eventignore' or 'eventignorewin' -- is a
/// list of event names, `Err` otherwise.
///
/// `win` says the value is 'eventignorewin', which accepts only the
/// window-local events.
pub fn check_ei(value: &CStr, win: bool) -> Result<(), Failed> {
    let ei = value.to_bytes();
    let mut at = 0;
    while at < ei.len() {
        if let Some(all) = skip_all(&ei[at..]) {
            at += all;
        } else {
            at += usize::from(ei[at] == b'-');
            let (found, consumed) = event_name2nr(&ei[at..]);
            at += consumed;
            let Some(event) = found else {
                return Err(Failed);
            };
            if win && !event_row(event).win_local {
                return Err(Failed);
            }
        }
    }
    Ok(())
}

/// How far past a leading `all` item of `ei` (and its comma) the rest
/// starts, or `None` when there is not one.
///
/// Upstream compares with `strncasecmp`, the locale's; `all` is plain
/// ASCII, which no locale folds differently.
fn skip_all(ei: &[u8]) -> Option<usize> {
    if ei.len() < 3 || !ei[..3].eq_ignore_ascii_case(b"all") {
        return None;
    }
    match ei.get(3) {
        None | Some(0) => Some(3),
        Some(b',') => Some(4),
        Some(_) => None,
    }
}

/// Append `what` (which starts with a comma) to 'eventignore', and answer
/// the old value for [`au_event_restore`].
pub(crate) fn au_event_disable(what: &CStr) -> XString {
    let saved = P_EI.get();
    let appended = if saved.is_empty() && what.to_bytes().first() == Some(&b',') {
        // Nothing to join to, so the leading comma would open the list with
        // an empty entry.
        XString::from_bytes(&what.to_bytes()[1..])
    } else {
        let mut joined = saved.clone();
        joined.push_cstr(what);
        joined
    };
    set_option_eventignore(appended.as_cstr());
    saved
}

/// Put back what [`au_event_disable`] saved.
pub(crate) fn au_event_restore(old_ignored: Option<XString>) {
    if let Some(value) = old_ignored {
        set_option_eventignore(value.as_cstr());
    }
}

/// Set 'eventignore', which copies what it is given.
fn set_option_eventignore(value: &CStr) {
    let string = String_0::from_bytes(value.to_bytes());
    set_option_direct(
        kOptEventignore,
        OptVal::string(string),
        OptionSetFlags::NONE,
        SID_NONE,
    );
}

/// Whether any autocommand is defined for `event` -- counting the deleted
/// rows a running walk still keeps, as upstream's `kv_size` did.
pub fn has_event(event: AutoEvent) -> bool {
    with_event_cmds(event, |cmds| !cmds.is_empty())
}

/// Whether a `CursorHold` autocommand exists for the mode we are in.
///
/// Safe: it only asks the event table and the editor's mode, both of which
/// are live for as long as the editor is.
fn has_cursorhold() -> bool {
    has_event(if get_real_state() == MODE_NORMAL_BUSY {
        AutoEvent::CursorHold
    } else {
        AutoEvent::CursorHoldI
    })
}

/// Whether `CursorHold` should fire now: one is defined, nothing else is
/// pending, and we are in a mode that has one.
///
/// Safe: every question it asks is of the editor's own state -- the event
/// table, the typeahead, the recording register -- and takes no pointer
/// from the caller.
pub fn trigger_cursorhold() -> bool {
    if did_cursorhold.get()
        || !has_cursorhold()
        || reg_recording.get() != 0
        || !typeahead().is_empty()
        || ins_compl_active()
    {
        return false;
    }
    let state = get_real_state();
    state == MODE_NORMAL_BUSY || state & MODE_INSERT != 0
}

/// Completion source for `:autocmd`'s event argument: the augroup names
/// first (when [`autocmd_include_groups`] is set), then every event name.
pub fn expand_get_event_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let idx = ::core::ffi::c_int::try_from(idx).ok()?;
    // `augroup_name` answers `None` once `idx` walks past the last group.
    if let Some(name) = augroup_name(idx + 1) {
        // Skip the group, but keep it in the numbering: the caller
        // walks `idx` up until this answers `None`.
        if !autocmd_include_groups.get() || augroup_is_deleted(idx + 1) {
            return Some(Candidate::Borrowed(c""));
        }
        return Some(Candidate::Owned(name.as_cstr().to_owned()));
    }
    let row = usize::try_from(idx - next_augroup_id.get()).ok()?;
    EVENT_NAMES
        .get(row)
        .map(|row| Candidate::Borrowed(row.name))
}

/// Completion source for an 'eventignore' item: every event name, or --
/// for 'eventignorewin' (`win`) -- only the window-local ones.
pub fn get_event_name_no_group(idx: usize, win: bool) -> Option<&'static CStr> {
    if !win {
        return EVENT_NAMES.get(idx).map(|row| row.name);
    }
    EVENT_NAMES
        .iter()
        .filter(|row| row.win_local)
        .nth(idx)
        .map(|row| row.name)
}

/// Whether `event` is an event nvim has.
pub fn autocmd_supported(event: &::core::ffi::CStr) -> bool {
    event_name2nr(event.to_bytes()).0.is_some()
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

    /// Every row's index is the [`AutoEvent`] variant of that *name*, which
    /// is what `event_nr2name` indexes with, and its `event` is the
    /// canonical event that name resolves to.
    #[test]
    fn every_name_finds_its_own_row() {
        assert_eq!(EVENT_NAMES.len(), AutoEvent::COUNT);
        for (i, row) in EVENT_NAMES.iter().enumerate() {
            assert_eq!(event_name_index(row.name.to_bytes()), Some(i));
            assert!(AutoEvent::at_row(i).is_some());
            assert!(row.event.index() < AutoEvent::COUNT);
        }
    }
}
