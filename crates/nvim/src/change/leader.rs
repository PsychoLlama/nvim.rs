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
        unsafe {
            copy_option_part(
                list,
                self.as_ptr(),
                COM_MAX_LEN as size_t,
                c",".as_ptr().cast_mut(),
            );
            let colon = vim_strchr(self.as_ptr(), ':' as c_int);
            if colon.is_null() {
                return None;
            }
            *colon = NUL as c_char;
            Some(colon.add(1))
        }
    }

    /// Whether the item's flag letters contain `flag`.
    ///
    /// # Safety
    /// `self` must hold a NUL-terminated flag string.
    unsafe fn has(&mut self, flag: c_int) -> bool {
        unsafe { !vim_strchr(self.as_ptr(), flag).is_null() }
    }
}

/// Whether `leader` matches `line` at byte `at`, and how many bytes of it did.
///
/// The comparison is exact, except that a leader *starting* with white space
/// only needs some white space in front of it in the line -- the amount need
/// not match, since a line may mix tabs and spaces.
///
/// # Safety
/// `line` must be NUL-terminated and `at` a byte offset into it; `leader` must
/// be NUL-terminated.
unsafe fn leader_match(line: *mut c_char, at: c_int, mut leader: *mut c_char) -> Option<c_int> {
    unsafe {
        if ascii_iswhite(c_int::from(*leader)) {
            if at == 0 || !ascii_iswhite(c_int::from(*line.offset((at - 1) as isize))) {
                return None; // missing white space
            }
            while ascii_iswhite(c_int::from(*leader)) {
                leader = leader.add(1);
            }
        }
        let mut j = 0;
        while c_int::from(*leader.offset(j as isize)) != NUL
            && c_int::from(*leader.offset(j as isize))
                == c_int::from(*line.offset((at + j) as isize))
        {
            j += 1;
        }
        if c_int::from(*leader.offset(j as isize)) != NUL {
            return None; // the leader ran past what the line has
        }
        Some(j)
    }
}

/// Whether the `b` flag is satisfied at `at + j`: there must be white space or
/// the end of the line after the leader.
///
/// # Safety
/// `line` must be NUL-terminated and `at + j` a byte offset into it.
unsafe fn blank_after(line: *mut c_char, at: c_int) -> bool {
    unsafe {
        ascii_iswhite(c_int::from(*line.offset(at as isize)))
            || c_int::from(*line.offset(at as isize)) == NUL
    }
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
    unsafe {
        let mut got_com = false;
        let mut item = ComItem::new();
        // A middle-part match is remembered rather than taken, because it may
        // be a substring of the *end* part, whose flags are the better answer.
        let mut middle_match_len = 0;
        let mut saved_flags: *mut c_char = ::core::ptr::null_mut();

        let mut result = 0;
        let mut i = 0;
        while ascii_iswhite(c_int::from(*line.offset(i as isize))) {
            i += 1; // leading white space is ignored
        }

        // Repeat to match several nested comment strings.
        while c_int::from(*line.offset(i as isize)) != NUL {
            let mut found_one = false;
            let mut list = (*curbuf.get()).b_p_com;
            while *list != 0 {
                if !got_com && !flags.is_null() {
                    *flags = list; // remember where this item's flags started
                }
                let prev_list = list;
                let Some(leader) = item.take(&raw mut list) else {
                    continue; // no ':' in the item: ignore it
                };

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
                if !got_com && !flags.is_null() {
                    *flags = saved_flags;
                }
                i += middle_match_len;
                found_one = true;
            }
            if !found_one {
                break;
            }

            result = i;
            while ascii_iswhite(c_int::from(*line.offset(i as isize))) {
                i += 1;
            }
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
    unsafe {
        let mut result = -1;
        let mut lower_check_bound = 0;
        let mut item = ComItem::new();

        let mut i = strlen(line) as c_int;
        loop {
            i -= 1;
            if i < lower_check_bound {
                break;
            }

            // Scan 'comments' for an item whose leader starts at `i`.
            let mut found_one = false;
            let mut com_leader: *mut c_char = ::core::ptr::null_mut();
            let mut com_flags: *mut c_char = ::core::ptr::null_mut();
            let mut list = (*curbuf.get()).b_p_com;
            while *list != 0 {
                let flags_save = list;
                let Some(leader) = item.take(&raw mut list) else {
                    continue; // cannot happen for a well-formed 'comments'
                };
                com_leader = leader;

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
                    while k <= i && ascii_iswhite(c_int::from(*line.offset(k as isize))) {
                        k += 1;
                    }
                    if k < i {
                        continue;
                    }
                }

                found_one = true;
                if !flags.is_null() {
                    *flags = flags_save;
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
            while ascii_iswhite(c_int::from(*com_leader)) {
                com_leader = com_leader.add(1);
            }
            let len1 = strlen(com_leader) as c_int;

            let mut other = ComItem::new();
            let mut list = (*curbuf.get()).b_p_com;
            while *list != 0 {
                let flags_save = list;
                // `take` writes the NUL that isolates the flags, so the leader
                // has to be taken before the identity test short-circuits it.
                let leader = other.take(&raw mut list);
                if flags_save == com_flags {
                    continue;
                }
                // Upstream does not test for a missing ':' here; a
                // well-formed 'comments' always has one.
                let Some(mut leader) = leader else {
                    continue;
                };
                while ascii_iswhite(c_int::from(*leader)) {
                    leader = leader.add(1);
                }
                let len2 = strlen(leader) as c_int;
                if len2 == 0 {
                    continue;
                }

                // Does this item's leader end with a prefix of `com_leader`?
                let mut off = len2.min(i);
                while off > 0 && off + len1 > len2 {
                    off -= 1;
                    if strncmp(
                        leader.offset(off as isize),
                        com_leader,
                        (len2 - off) as size_t,
                    ) == 0
                    {
                        lower_check_bound = lower_check_bound.min(i - off);
                    }
                }
            }
        }
        result
    }
}
