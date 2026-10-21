//! Measuring a 'comments' leader on a line.
//!
//! [`get_leader_len`] answers how many bytes at the start of a line are a
//! comment leader, and [`get_last_leader_offset`] where the *last* leader on a
//! line begins -- the one a trailing `//` comment starts at.
//!
//! Both walk 'comments' item by item. One item is `flags:leader`; a copy of
//! it ([`ComItem`]) holds the two halves on either side of its first `:`.
//!
//! The item that matched is answered as a [`ComFlags`]: where it starts in
//! the current buffer's 'comments' value. Upstream hands back a pointer into
//! the option for the same purpose; an offset is that pointer without the
//! promise that the option is not written in the meantime -- read after a
//! change it names whatever is there now, never freed bytes. Note what
//! [`get_leader_len`] answers for it on failure.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::narrow::len_as_int;
use core::ffi::c_int;

use super::*;
use crate::option::split_option_part_with;
use crate::optionstr::OptStringRef;
use crate::types::NUL;
use crate::winlayer::Buf;

/// A 'comments' item of the current buffer, by the byte offset in the option
/// value at which it starts -- its flag letters come first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ComFlags(usize);

impl ComFlags {
    /// Where the item starts in 'comments'.
    pub(crate) const fn offset(self) -> usize {
        self.0
    }

    /// `f` of the rest of the current buffer's 'comments' from this item on
    /// -- upstream's `char *flags` read as a string. Empty when the option
    /// has since become shorter than the offset.
    pub(crate) fn with_rest<R>(self, f: impl FnOnce(&[u8]) -> R) -> R {
        let buffer = Buf::current();
        let com = buffer.b_p_com.bytes();
        f(com.get(self.0..).unwrap_or_default())
    }

    /// The item's flag letters: the bytes up to its `:`.
    pub fn letters(self) -> Vec<u8> {
        self.with_rest(|rest| {
            let end = rest.iter().position(|&b| b == b':').unwrap_or(rest.len());
            rest[..end].to_vec()
        })
    }

    /// Whether the item's flag letters contain `flag`; only the current item
    /// is searched -- its `:` ends the letters.
    pub(crate) fn has(self, flag: c_int) -> bool {
        self.with_rest(|rest| {
            rest.iter()
                .take_while(|&&b| b != b':')
                .any(|&b| c_int::from(b) == flag)
        })
    }

    /// The first byte from the item's start that is one of `stops`, or NUL
    /// when the option ends first.
    pub(crate) fn first_of(self, stops: &[u8]) -> u8 {
        self.with_rest(|rest| {
            rest.iter()
                .copied()
                .find(|b| stops.contains(b))
                .unwrap_or(0)
        })
    }
}

/// One 'comments' item, copied out of the option.
///
/// `COM_MAX_LEN` bytes is upstream's `part_buf`, and the copy is cut to fit
/// it the way `copy_option_part` cuts it: at most `COM_MAX_LEN - 1` bytes.
struct ComItem {
    buf: [u8; COM_MAX_LEN as usize],
    len: usize,
    /// Where the first `:` is, or `None` for an item with no `:` at all.
    colon: Option<usize>,
}

impl ComItem {
    fn new() -> ComItem {
        ComItem {
            buf: [0; COM_MAX_LEN as usize],
            len: 0,
            colon: None,
        }
    }

    /// Copy the item at the start of `list` into `self` and step `list` past
    /// it -- and the separator and blanks after it.
    ///
    /// Answers whether the item has a `:`, that is, whether it has a leader.
    fn take(&mut self, list: &mut &[u8]) -> bool {
        let (buf, len) = (&mut self.buf, &mut self.len);
        *len = 0;
        *list = split_option_part_with(list, COM_MAX_LEN as usize, b",", |run| {
            buf[*len..*len + run.len()].copy_from_slice(run);
            *len += run.len();
        });
        self.colon = self.buf[..self.len].iter().position(|&b| b == b':');
        self.colon.is_some()
    }

    /// The item's flag letters: everything before the `:`.
    fn flags(&self) -> &[u8] {
        &self.buf[..self.colon.unwrap_or(self.len)]
    }

    /// The leader text: everything after the `:`.
    fn leader(&self) -> &[u8] {
        self.colon
            .map_or(&[], |colon| &self.buf[colon + 1..self.len])
    }

    /// Whether the item's flag letters contain `flag`.
    fn has(&self, flag: c_int) -> bool {
        self.flags().iter().any(|&b| c_int::from(b) == flag)
    }
}

/// The byte at `at` of `text` as the C's `int`, NUL past its end.
fn at(text: &[u8], at: usize) -> c_int {
    c_int::from(byte_at(text, at))
}

/// How many bytes of white space start at `from`.
fn white_run(text: &[u8], from: usize) -> usize {
    let mut n = 0;
    while ascii_iswhite(at(text, from + n)) {
        n += 1;
    }
    n
}

/// Whether `leader` matches `line` at byte `i`, and how many bytes of it did.
///
/// The comparison is exact, except that a leader *starting* with white space
/// only needs some white space in front of it in the line -- the amount need
/// not match, since a line may mix tabs and spaces.
fn leader_match(line: &[u8], i: usize, leader: &[u8]) -> Option<usize> {
    let mut skip = 0;
    if ascii_iswhite(at(leader, 0)) {
        if i == 0 || !ascii_iswhite(at(line, i - 1)) {
            return None; // missing white space
        }
        skip = white_run(leader, 0);
    }
    let mut j = 0;
    while at(leader, skip + j) != NUL && at(leader, skip + j) == at(line, i + j) {
        j += 1;
    }
    if at(leader, skip + j) != NUL {
        return None; // the leader ran past what the line has
    }
    Some(j)
}

/// Whether the `b` flag is satisfied at `i`: there must be white space or the
/// end of the line after the leader.
fn blank_after(line: &[u8], i: usize) -> bool {
    ascii_iswhite(at(line, i)) || at(line, i) == NUL
}

/// [`get_leader_len`] when only the length is wanted.
pub(crate) fn leader_len_of(line: &[u8], backward: bool, include_space: bool) -> usize {
    usize::try_from(get_leader_len(line, backward, include_space).0).unwrap_or(0)
}

/// How many bytes at the start of `line` are a comment leader, 0 for none,
/// and the 'comments' item the scan settled on.
///
/// `backward` is set by the `O` command, which skips items carrying the `O`
/// flag. `include_space` adds the white space after the leader to the answer.
///
/// **The item is answered even when nothing matches.** Every item the scan
/// *tries* is noted (until a nested comment has been found), so after a line
/// with no leader the answer names the last item of 'comments' rather than
/// nothing; `None` means no item was tried at all -- a line that is empty or
/// only white space. Upstream writes its `flags` out-parameter the same way,
/// and `format_lines` reads it in exactly that state -- its `://` test -- so
/// a caller carrying an older answer keeps it on `None`.
pub fn get_leader_len(
    line: &[u8],
    backward: bool,
    include_space: bool,
) -> (c_int, Option<ComFlags>) {
    let buffer = Buf::current();
    let com = buffer.b_p_com.bytes();
    let mut flags = None;
    let mut got_com = false;
    let mut item = ComItem::new();
    // A middle-part match is remembered rather than taken, because it may
    // be a substring of the *end* part, whose flags are the better answer.
    let mut middle_match_len = 0;
    let mut saved_flags = None;

    let mut result = 0;
    let mut i = white_run(line, 0); // leading white space is ignored

    // Repeat to match several nested comment strings.
    while at(line, i) != NUL {
        let mut found_one = false;
        let mut list = com;
        while !list.is_empty() {
            let here = ComFlags(com.len() - list.len());
            if !got_com {
                flags = Some(here); // where this item's flags started
            }
            if !item.take(&mut list) {
                continue; // no ':' in the item: ignore it
            }

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

            let Some(j) = leader_match(line, i, item.leader()) else {
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
                    saved_flags = Some(here);
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
                flags = saved_flags;
            }
            i += middle_match_len;
            found_one = true;
        }
        if !found_one {
            break;
        }

        result = i;
        i += white_run(line, i);
        if include_space {
            result = i;
        }

        got_com = true;
        // `item` still holds the item that matched: stop unless it nests.
        if !item.has(COM_NEST) {
            break;
        }
    }
    (len_as_int(result), flags)
}

/// Where the last comment on `line` starts, or -1 if there is none, and the
/// 'comments' item that matched there.
///
/// This is the *trailing* comment question -- `code(); // why` -- so the scan
/// runs backwards from the end of the line. `lower_check_bound` is what stops
/// it: once a leader is found at `i`, no leader can start earlier than the
/// longest overlap any *other* 'comments' item has with it.
///
/// The item is the one that matched last, kept when a later round finds
/// nothing; `None` when nothing ever matched.
pub fn get_last_leader_offset(line: &[u8]) -> (c_int, Option<ComFlags>) {
    let buffer = Buf::current();
    let com = buffer.b_p_com.bytes();
    let mut result = -1;
    let mut found_flags = None;
    let mut lower_check_bound = 0;
    let mut item = ComItem::new();

    let mut i = line.len();
    loop {
        if i == 0 {
            break;
        }
        i -= 1;
        if i < lower_check_bound {
            break;
        }

        // Scan 'comments' for an item whose leader starts at `i`.
        let mut found_one = false;
        let mut com_flags = None;
        let mut list = com;
        while !list.is_empty() {
            let flags_save = ComFlags(com.len() - list.len());
            if !item.take(&mut list) {
                continue; // cannot happen for a well-formed 'comments'
            }

            let Some(j) = leader_match(line, i, item.leader()) else {
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
                while k <= i && ascii_iswhite(at(line, k)) {
                    k += 1;
                }
                if k < i {
                    continue;
                }
            }

            found_one = true;
            found_flags = Some(flags_save);
            com_flags = Some(flags_save);
            break;
        }
        if !found_one {
            continue;
        }

        result = len_as_int(i);
        // A nesting comment can have another one in front of it.
        if item.has(COM_NEST) {
            continue;
        }
        lower_check_bound = i;

        // The leader found may be the *tail* of a longer one belonging to
        // another item -- `#` inside `#if`, say. Pull `lower_check_bound`
        // back far enough that the next round can find that longer one.
        let found_leader = item.leader();
        let com_leader = &found_leader[white_run(found_leader, 0)..];
        let len1 = com_leader.len();

        let mut other = ComItem::new();
        let mut list = com;
        while !list.is_empty() {
            let flags_save = ComFlags(com.len() - list.len());
            let taken = other.take(&mut list);
            if Some(flags_save) == com_flags {
                continue;
            }
            // Upstream does not test for a missing ':' here; a
            // well-formed 'comments' always has one.
            if !taken {
                continue;
            }
            let leader = other.leader();
            let leader = &leader[white_run(leader, 0)..];
            let len2 = leader.len();
            if len2 == 0 {
                continue;
            }

            // Does this item's leader end with a prefix of `com_leader`?
            let mut off = len2.min(i);
            while off > 0 && off + len1 > len2 {
                off -= 1;
                if com_leader.starts_with(&leader[off..]) {
                    lower_check_bound = lower_check_bound.min(i - off);
                }
            }
        }
    }
    (result, found_flags)
}
