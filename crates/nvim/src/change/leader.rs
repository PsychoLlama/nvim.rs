//! Measuring a 'comments' leader on a line.
//!
//! [`get_leader_len`] answers how many bytes at the start of a line are a
//! comment leader, and [`get_last_leader_offset`] where the *last* leader on a
//! line begins -- the one a trailing `//` comment starts at.
//!
//! Both walk 'comments' item by item. One item is `flags:leader`, and both
//! functions isolate the two halves by writing a NUL over the `:` **in their
//! own copy** of the item, so the flag letters are a NUL-terminated string
//! and the leader is what follows.
//!
//! The `flags` out-parameter of both points back into the 'comments' option
//! itself -- at the start of the matching item -- so it stays valid as long as
//! the option does. Note what [`get_leader_len`] does with it on failure.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;
use crate::winlayer::Buf;

/// A NUL-terminated string being walked byte by byte.
///
/// The scans here index a line and a leader from a base that never moves, and
/// never step past the terminating NUL -- so the one unchecked step is
/// *taking* the pointer, and every read after it is ordinary code.
#[derive(Clone, Copy)]
struct Scan(*mut c_char);

impl Scan {
    /// # Safety
    /// `s` must stay a NUL-terminated string for as long as the value is
    /// used.
    unsafe fn new(s: *mut c_char) -> Self {
        Self(s)
    }

    /// The byte `at` bytes in, as the C's `int`.
    fn at(self, at: c_int) -> c_int {
        // SAFETY: the constructor's promise. No walk below passes the
        // terminating NUL, so `at` stays inside the string.
        c_int::from(unsafe { *self.0.offset(at as isize) })
    }

    /// Whether the byte `at` bytes in is white space.
    fn white_at(self, at: c_int) -> bool {
        ascii_iswhite(self.at(at))
    }

    /// How many bytes of white space start at `at`.
    fn white_run(self, at: c_int) -> c_int {
        let mut n = 0;
        while self.white_at(at + n) {
            n += 1;
        }
        n
    }

    /// The string, `at` bytes in.
    fn from(self, at: c_int) -> *mut c_char {
        self.0.wrapping_offset(at as isize)
    }

    /// The string's length, `strlen`.
    fn len(self) -> c_int {
        // SAFETY: the constructor's promise.
        unsafe { strlen(self.0) as c_int }
    }
}

/// Whether `list` still names an item of 'comments' rather than its final NUL.
fn more_items(list: *mut c_char) -> bool {
    // SAFETY: `list` walks the buffer's own NUL-terminated 'comments' value,
    // which `copy_option_part` never advances past its NUL.
    unsafe { *list != 0 }
}

/// One 'comments' item, copied out of the option.
///
/// `COM_MAX_LEN` bytes is upstream's `part_buf`; `copy_option_part` truncates
/// into it.
struct ComItem {
    buf: [c_char; COM_MAX_LEN as usize],
}

impl ComItem {
    fn new() -> ComItem {
        ComItem {
            buf: [0; COM_MAX_LEN as usize],
        }
    }

    fn as_ptr(&mut self) -> *mut c_char {
        self.buf.as_mut_ptr()
    }

    /// Copy the item `*list` points at into `self` and advance `list` past it.
    ///
    /// Answers where the leader text starts -- everything after the `:`, which
    /// is overwritten with a NUL so that `self` is just the flag letters --
    /// or `None` for an item with no `:` at all.
    ///
    /// # Safety
    /// `list` must point at a position inside a NUL-terminated 'comments'
    /// value.
    unsafe fn take(&mut self, list: *mut *mut c_char) -> Option<*mut c_char> {
        let into = self.as_ptr();
        let sep = c",".as_ptr().cast_mut();
        // SAFETY: the caller's position in 'comments', and `buf` has room for
        // the `COM_MAX_LEN` bytes `copy_option_part` is told about.
        unsafe { copy_option_part(list, into, COM_MAX_LEN as size_t, sep) };
        // SAFETY: `copy_option_part` NUL-terminates what it wrote.
        let colon = unsafe { vim_strchr(into, ':' as c_int) };
        if colon.is_null() {
            return None;
        }
        // SAFETY: `colon` points at the `:` inside `buf`, so the byte after
        // it is inside `buf` too.
        unsafe { *colon = NUL as c_char };
        Some(unsafe { colon.add(1) })
    }

    /// Whether the item's flag letters contain `flag`.
    fn has(&mut self, flag: c_int) -> bool {
        // SAFETY: `buf` is NUL-terminated -- `copy_option_part` writes the
        // NUL, and `take` only turns the `:` inside it into another one.
        unsafe { !vim_strchr(self.as_ptr(), flag).is_null() }
    }
}

/// Whether `leader` matches `line` at byte `at`, and how many bytes of it did.
///
/// The comparison is exact, except that a leader *starting* with white space
/// only needs some white space in front of it in the line -- the amount need
/// not match, since a line may mix tabs and spaces.
fn leader_match(line: Scan, at: c_int, leader: Scan) -> Option<c_int> {
    let mut skip = 0;
    if leader.white_at(0) {
        if at == 0 || !line.white_at(at - 1) {
            return None; // missing white space
        }
        skip = leader.white_run(0);
    }
    let mut j = 0;
    while leader.at(skip + j) != NUL && leader.at(skip + j) == line.at(at + j) {
        j += 1;
    }
    if leader.at(skip + j) != NUL {
        return None; // the leader ran past what the line has
    }
    Some(j)
}

/// Whether the `b` flag is satisfied at `at`: there must be white space or the
/// end of the line after the leader.
fn blank_after(line: Scan, at: c_int) -> bool {
    line.white_at(at) || line.at(at) == NUL
}

/// How many bytes at the start of `line` are a comment leader, 0 for none.
///
/// `backward` is set by the `O` command, which skips items carrying the `O`
/// flag. `include_space` adds the white space after the leader to the answer.
///
/// **`flags` is written even when nothing matches.** Every item the scan
/// *tries* overwrites it (until a nested comment has been found), so after a
/// line with no leader it names the last item of 'comments' rather than
/// nothing. `format_lines` reads it in exactly that state -- its `://` test --
/// so clearing it would be a behaviour change.
///
/// # Safety
/// `line` must be NUL-terminated. `flags` must be null or writable.
pub unsafe fn get_leader_len(
    line: *mut c_char,
    flags: *mut *mut c_char,
    backward: bool,
    include_space: bool,
) -> c_int {
    // SAFETY: the caller's NUL-terminated line.
    let line = unsafe { Scan::new(line) };
    let set_flags = |at: *mut c_char| {
        if !flags.is_null() {
            // SAFETY: the caller's out-parameter, and it is not null.
            unsafe { *flags = at };
        }
    };
    let mut got_com = false;
    let mut item = ComItem::new();
    // A middle-part match is remembered rather than taken, because it may
    // be a substring of the *end* part, whose flags are the better answer.
    let mut middle_match_len = 0;
    let mut saved_flags: *mut c_char = ::core::ptr::null_mut();

    let mut result = 0;
    let mut i = line.white_run(0); // leading white space is ignored

    // Repeat to match several nested comment strings.
    while line.at(i) != NUL {
        let mut found_one = false;
        let mut list = cur_buf().b_p_com;
        while more_items(list) {
            if !got_com {
                set_flags(list); // where this item's flags started
            }
            let prev_list = list;
            // SAFETY: `list` is a position inside 'comments'.
            let taken = unsafe { item.take(&raw mut list) };
            let Some(leader) = taken else {
                continue; // no ':' in the item: ignore it
            };
            // SAFETY: `take` answers a NUL-terminated tail of its own buffer.
            let leader = unsafe { Scan::new(leader) };

            // A middle match is already in hand and this item can neither
            // extend nor end it, so stop and use the middle match.
            if middle_match_len != 0 && !item.has(COM_MIDDLE) && !item.has(COM_END) {
                break;
            }
            // Inside a nested comment, only further nested items count.
            if got_com && !item.has(COM_NEST) {
                continue;
            }
            // The `O` flag means "not for the O command".
            if backward && item.has(COM_NOBACK) {
                continue;
            }

            let Some(j) = leader_match(line, i, leader) else {
                continue;
            };
            if item.has(COM_BLANK) && !blank_after(line, i + j) {
                continue;
            }

            if item.has(COM_MIDDLE) {
                // Keep looking: an end item matching more bytes is the
                // better answer, and carries better flags.
                if middle_match_len == 0 {
                    middle_match_len = j;
                    saved_flags = prev_list;
                }
                continue;
            }
            if middle_match_len != 0 && j > middle_match_len {
                // A longer, and so better, match than the middle one.
                middle_match_len = 0;
            }
            if middle_match_len == 0 {
                i += j;
            }
            found_one = true;
            break;
        }

        if middle_match_len != 0 {
            // Fall back on the middle match, no end item having matched.
            if !got_com {
                set_flags(saved_flags);
            }
            i += middle_match_len;
            found_one = true;
        }
        if !found_one {
            break;
        }

        result = i;
        i += line.white_run(i);
        if include_space {
            result = i;
        }

        got_com = true;
        // `item` still holds the item that matched: stop unless it nests.
        if !item.has(COM_NEST) {
            break;
        }
    }
    result
}

/// Where the last comment on `line` starts, or -1 if there is none.
///
/// This is the *trailing* comment question -- `code(); // why` -- so the scan
/// runs backwards from the end of the line. `lower_check_bound` is what stops
/// it: once a leader is found at `i`, no leader can start earlier than the
/// longest overlap any *other* 'comments' item has with it.
///
/// `flags`, when not null, is set to the matching item's flag letters.
///
/// # Safety
/// `line` must be NUL-terminated. `flags` must be null or writable.
pub unsafe fn get_last_leader_offset(line: *mut c_char, flags: *mut *mut c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated line.
    let line = unsafe { Scan::new(line) };
    let mut result = -1;
    let mut lower_check_bound = 0;
    let mut item = ComItem::new();

    let mut i = line.len();
    loop {
        i -= 1;
        if i < lower_check_bound {
            break;
        }

        // Scan 'comments' for an item whose leader starts at `i`.
        let mut found_one = false;
        let mut com_leader = None;
        let mut com_flags: *mut c_char = ::core::ptr::null_mut();
        let mut list = cur_buf().b_p_com;
        while more_items(list) {
            let flags_save = list;
            // SAFETY: `list` is a position inside 'comments'.
            let taken = unsafe { item.take(&raw mut list) };
            let Some(leader) = taken else {
                continue; // cannot happen for a well-formed 'comments'
            };
            // SAFETY: `take` answers a NUL-terminated tail of its own buffer.
            let leader = unsafe { Scan::new(leader) };
            com_leader = Some(leader);

            let Some(j) = leader_match(line, i, leader) else {
                continue;
            };
            if item.has(COM_BLANK) && !blank_after(line, i + j) {
                continue;
            }
            if item.has(COM_MIDDLE) {
                // A middle part only counts when everything in front of it
                // is white space: otherwise C's `*` would look like the
                // middle of a comment wherever it appears.
                let mut k = 0;
                while k <= i && line.white_at(k) {
                    k += 1;
                }
                if k < i {
                    continue;
                }
            }

            found_one = true;
            if !flags.is_null() {
                // SAFETY: the caller's out-parameter, and it is not null.
                unsafe { *flags = flags_save };
            }
            com_flags = flags_save;
            break;
        }
        if !found_one {
            continue;
        }

        result = i;
        // A nesting comment can have another one in front of it.
        if item.has(COM_NEST) {
            continue;
        }
        lower_check_bound = i;

        // The leader found may be the *tail* of a longer one belonging to
        // another item -- `#` inside `#if`, say. Pull `lower_check_bound`
        // back far enough that the next round can find that longer one.
        let Some(com_leader) = com_leader else {
            continue;
        };
        let com_start = com_leader.white_run(0);
        // SAFETY: white space is inside the leader, so the byte after a run
        // of it is too, and the string stays NUL-terminated.
        let com_leader = unsafe { Scan::new(com_leader.from(com_start)) };
        let len1 = com_leader.len();

        let mut other = ComItem::new();
        let mut list = cur_buf().b_p_com;
        while more_items(list) {
            let flags_save = list;
            // `take` writes the NUL that isolates the flags, so the leader
            // has to be taken before the identity test short-circuits it.
            //
            // SAFETY: `list` is a position inside 'comments'.
            let taken = unsafe { other.take(&raw mut list) };
            if flags_save == com_flags {
                continue;
            }
            // Upstream does not test for a missing ':' here; a
            // well-formed 'comments' always has one.
            let Some(leader) = taken else {
                continue;
            };
            // SAFETY: as for `com_leader` above.
            let leader = unsafe { Scan::new(leader) };
            let leader = unsafe { Scan::new(leader.from(leader.white_run(0))) };
            let len2 = leader.len();
            if len2 == 0 {
                continue;
            }

            // Does this item's leader end with a prefix of `com_leader`?
            let mut off = len2.min(i);
            while off > 0 && off + len1 > len2 {
                off -= 1;
                let tail = leader.from(off);
                let n = (len2 - off) as size_t;
                // SAFETY: `tail` is `off` bytes into a string of `len2`, and
                // `n` is the rest of it; `com_leader` is NUL-terminated.
                if unsafe { strncmp(tail, com_leader.from(0), n) } == 0 {
                    lower_check_bound = lower_check_bound.min(i - off);
                }
            }
        }
    }
    result
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
