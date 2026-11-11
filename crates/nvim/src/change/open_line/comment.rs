//! Deciding what comment leader the new line gets, and building it.
//!
//! Two phases. [`plan_leader`] reads the flag letters of the 'comments' item
//! that matched the *old* line and decides what the new line should carry:
//! nothing at all, the same leader, the item's *middle* part, or an empty
//! leader. [`build_leader`] then copies the leader and edits it into shape
//! -- which is where 'comments' `l`/`r` (left/right adjusted) and the numeric
//! indent offset are honoured.
//!
//! The subtlety throughout is that a leader is measured in *screen cells*,
//! not bytes: replacing `/*` with ` *` has to keep the following text where
//! it was, and a double-width character being blanked out becomes two spaces.
//!
//! The leader is edited in a buffer sized the way upstream sizes its
//! allocation, zero-filled, with every `memmove` a `copy_within` -- the walks
//! read the terminator after the leader and, in two places, a few bytes past
//! it, and the zeros answer what the C reads there.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::{getdigits_int_at, ptr2cells_at};
use crate::cstr::{byte_at, until_nul};
use crate::mbyte::{c_cluster_len, c_display_width, head_off};
use crate::narrow::len_as_int;
use core::ffi::c_int;

use crate::change::*;
use crate::option::split_option_part;
use crate::optionstr::OptStringRef;
use crate::winlayer::{Buf, Win};

/// The byte at `at` of `text` as the C's `int`, NUL past its end.
fn at(text: &[u8], at: usize) -> u8 {
    byte_at(text, at)
}

/// The byte before `at`, which at the very start of 'comments' is none --
/// upstream reads the byte in front of the option there.
fn before(text: &[u8], at: usize) -> u8 {
    at.checked_sub(1).map_or(0, |prev| text[prev])
}

/// How many bytes of white space start `text`.
fn white_run(text: &[u8]) -> usize {
    text.iter()
        .take_while(|&&b| ascii_iswhite(c_int::from(b)))
        .count()
}

/// `copy_option_part` one item of `com` from byte `*p`, stepping `*p` past
/// it: the item, cut to `COM_MAX_LEN - 1` bytes.
fn copy_part(com: &[u8], p: &mut usize) -> Vec<u8> {
    let (part, rest) = split_option_part(&com[(*p).min(com.len())..], COM_MAX_LEN, b",");
    *p = com.len() - rest.len();
    part
}

/// What the new line's leader should be built from.
#[derive(Debug)]
pub(crate) enum Repl {
    /// Keep the old line's leader as it is.
    Same,
    /// Replace it with this text: the 'comments' item's middle part.
    Text(Vec<u8>),
    /// Replace it with nothing (the `f` flag: leader on the first line only).
    Blank,
}

/// [`plan_leader`]'s answer.
#[derive(Debug)]
pub(crate) struct LeaderPlan {
    /// Bytes of the old line to take as the leader; 0 means "no leader".
    pub(crate) lead_len: c_int,
    pub(crate) repl: Repl,
    /// Append a space after the leader.
    pub(crate) extra_space: bool,
    /// Where the *end* of a comment was found on the old line, if the leader
    /// was dropped because of it.
    pub(crate) comment_end: Option<usize>,
}

impl Repl {
    /// The replacement, or `None` for [`Repl::Same`].
    fn text(&self) -> Option<&[u8]> {
        match self {
            Repl::Same => None,
            Repl::Text(text) => Some(text),
            Repl::Blank => Some(b""),
        }
    }
}

/// Step `p` to just past the next `:` of the 'comments' value, noting the `b`
/// flag on the way.
fn skip_flags(com: &[u8], p: &mut usize, require_blank: &mut bool) {
    while at(com, *p) != 0 && before(com, *p) != b':' {
        if at(com, *p) == COM_BLANK {
            *require_blank = true;
        }
        *p += 1;
    }
}

/// Step `p` to just past the next `:`, noting the `x` flag on the way.
///
/// The `x` flag means "closing this comment automatically is allowed": the
/// answer says whether it was there, and the caller then records the last
/// character of the end leader in `END_COMMENT_PENDING`.
fn skip_flags_noting_auto_end(com: &[u8], p: &mut usize) -> bool {
    let mut auto_end = false;
    while at(com, *p) != 0 && before(com, *p) != b':' {
        if at(com, *p) == COM_AUTO_END {
            auto_end = true;
        }
        *p += 1;
    }
    auto_end
}

/// The `s`/`m` arm: the old line carries the *start* or the *middle* of a
/// three-part comment, so the new line gets the middle part.
///
/// `saved_line` is the old line (cut at the cursor when text moves to the new
/// line, which `has_extra` says).
fn plan_start_or_middle(
    plan: &mut LeaderPlan,
    com: &[u8],
    mut p: usize,
    dir: c_int,
    saved_line: &[u8],
    has_extra: bool,
    mut require_blank: bool,
) {
    let current_flag = at(com, p);
    if current_flag == COM_START {
        if dir == BACKWARD {
            // `O` on the start of a comment inserts no leader.
            plan.lead_len = 0;
            return;
        }
        // Step over the start item to reach the middle one.
        let _ = copy_part(com, &mut p);
        require_blank = false;
    }

    // Isolate the middle leader, then the end leader.
    skip_flags(com, &mut p, &mut require_blank);
    let middle = copy_part(com, &mut p);

    let auto_end = skip_flags_noting_auto_end(com, &mut p);
    let lead_end = copy_part(com, &mut p);
    if auto_end {
        // An empty end part has no last character; upstream reads the byte
        // in front of its buffer.
        END_COMMENT_PENDING.set(lead_end.last().copied().filter(|&b| b != 0));
    }

    let lead_len = usize::try_from(plan.lead_len).expect("non-negative");

    // The comment already ends on this line, so it needs no leader.
    if dir == FORWARD {
        let mut q = lead_len;
        while at(saved_line, q) != 0 {
            if saved_line[q..].starts_with(&lead_end) {
                plan.comment_end = Some(q);
                plan.lead_len = 0;
                return;
            }
            q += 1;
        }
    }

    if plan.lead_len > 0 {
        let middle_part = current_flag == COM_START;
        // <CR> immediately after the start leader wants a space after the
        // middle leader on the new line -- as does the `b` flag.
        let after_leader = at(saved_line, lead_len);
        let last_of_leader = at(saved_line, lead_len - 1);
        if !ascii_iswhite(c_int::from(last_of_leader))
            && (has_extra && Win::current().w_cursor.col == ColNr::new(plan.lead_len)
                || !has_extra && after_leader == 0
                || require_blank)
        {
            plan.extra_space = true;
        }
        if middle_part {
            // `o` on the start of a comment inserts the middle leader.
            plan.repl = Repl::Text(middle);
        }
    }
}

/// The `e` arm: the old line carries the *end* of a comment.
fn plan_end(plan: &mut LeaderPlan, com: &[u8], mut p: usize, dir: c_int, saved_line: &[u8]) {
    if dir == FORWARD {
        // `o` on the end of a comment inserts no leader. Remember where
        // the end is: it may be wanted to find the start (C comments).
        plan.comment_end = Some(white_run(saved_line));
        plan.lead_len = 0;
        return;
    }

    // `O` on the end of a comment inserts the middle leader, which is the
    // item before this one -- so search backwards for it.
    while p > 0 && at(com, p) != b',' {
        p -= 1;
    }
    let mut repl = p;
    while repl > 0 && before(com, repl) != b':' {
        repl -= 1;
    }
    plan.repl = Repl::Text(com[repl..p].to_vec());

    // A space after the middle leader is always right for `O` on an end.
    plan.extra_space = true;

    // Is automatic ending of the comment allowed?
    let mut p2 = p;
    let mut auto_end = false;
    while at(com, p2) != 0 && at(com, p2) != b':' {
        if at(com, p2) == COM_AUTO_END {
            auto_end = true;
        }
        p2 += 1;
    }
    if auto_end {
        // The last character of the end leader is what will close it.
        while at(com, p2) != 0 && at(com, p2) != b',' {
            p2 += 1;
        }
        END_COMMENT_PENDING.set(Some(before(com, p2)).filter(|&b| b != 0));
    }
}

/// Decide what leader the new line should carry, from the flag letters of the
/// 'comments' item that matched the old line.
///
/// `lead_flags` names that item, so the scan runs up to the item's `:`.
/// Only the first of `s`/`m`/`e`/`f` decides; `b` seen before it is
/// remembered. `saved_line` is the old line, cut where text moves to the new
/// one (`has_extra`).
///
/// Sets `END_COMMENT_PENDING`, the global that lets the *next* thing typed
/// close the comment automatically ('comments' `x`).
pub(crate) fn plan_leader(
    dir: c_int,
    lead_len: c_int,
    lead_flags: ComFlags,
    saved_line: &[u8],
    has_extra: bool,
) -> LeaderPlan {
    let mut plan = LeaderPlan {
        lead_len,
        repl: Repl::Same,
        extra_space: false,
        comment_end: None,
    };
    let mut require_blank = false;

    let buffer = Buf::current();
    let com = buffer.b_p_com.bytes();
    let mut p = lead_flags.offset();
    while at(com, p) != 0 && at(com, p) != b':' {
        let flag = at(com, p);
        if flag == COM_BLANK {
            require_blank = true;
        } else if flag == COM_START || flag == COM_MIDDLE {
            plan_start_or_middle(&mut plan, com, p, dir, saved_line, has_extra, require_blank);
            break;
        } else if flag == COM_END {
            plan_end(&mut plan, com, p, dir, saved_line);
            break;
        } else if flag == COM_FIRST {
            // The leader belongs on the first line only: `O` gets none,
            // `o` gets it blanked out so the text still lines up.
            if dir == BACKWARD {
                plan.lead_len = 0;
            } else {
                plan.repl = Repl::Blank;
            }
            break;
        }
        p += 1;
    }
    plan
}

/// The `l`/`r` alignment letter and the numeric indent offset of a 'comments'
/// item.
fn leader_alignment(lead_flags: ComFlags) -> (u8, c_int) {
    let mut align = 0;
    let mut off = 0;
    lead_flags.with_rest(|com| {
        let mut p = 0;
        while at(com, p) != 0 && at(com, p) != b':' {
            let c = at(com, p);
            if c == COM_RIGHT || c == COM_LEFT {
                align = c;
                p += 1;
            } else if c.is_ascii_digit() || c == b'-' {
                let (n, past) = getdigits_int_at(com, p, true, 0);
                off = n;
                // A `-` with no digits after it is no number: upstream reads
                // nothing and stands still here for ever.
                p = past.max(p + 1);
            } else {
                p += 1;
            }
        }
    });
    (align, off)
}

/// Overwrite the *right* end of the leader in `buf` with `repl`, keeping the
/// text that follows in the same screen column.
///
/// Answers the new leader length. `buf` holds `lead_len` bytes of leader
/// with room for `repl` after them.
fn right_adjust(buf: &mut [u8], mut lead_len: usize, repl: &[u8]) -> usize {
    let repl_len = repl.len();
    // Line up with the last non-white character of the old leader.
    let mut p = lead_len - 1;
    while p > 0 && ascii_iswhite(c_int::from(buf[p])) {
        p -= 1;
    }
    p += 1;

    // Walk back over as many *cells* as the replacement takes.
    let repl_size = c_display_width(repl, len_as_int(repl_len));
    let mut old_size = 0;
    let endp = p;
    while old_size < repl_size && p > 0 {
        p -= head_off(buf, p - 1) + 1;
        old_size += ptr2cells_at(&buf[p..]);
    }
    let gap = endp - p;
    if repl_len != gap {
        buf.copy_within(endp..lead_len, endp + repl_len - gap);
    }
    lead_len = lead_len + repl_len - gap;
    buf[p..p + repl_len].copy_from_slice(repl);
    if p + repl_len > lead_len {
        buf[p + repl_len] = b'\0';
    }

    // Blank out whatever is left of the old leader in front of it, a
    // double-width character becoming two spaces so the columns hold.
    while p > 0 {
        p -= 1;
        let mut l = head_off(buf, p);
        if l > 1 {
            p -= l;
            if ptr2cells_at(&buf[p..]) > 1 {
                buf[p + 1] = b' ';
                l -= 1;
            }
            let from = p + l + 1;
            buf.copy_within(from..from.max(lead_len), p + 1);
            lead_len -= l;
            buf[p] = b' ';
        } else if !ascii_iswhite(c_int::from(buf[p])) {
            buf[p] = b' ';
        }
    }
    lead_len
}

/// Overwrite the *left* end of the leader in `buf` with `repl`, keeping the
/// text that follows in the same screen column.
///
/// Answers the new leader length. `buf` holds `lead_len` bytes of leader
/// with room for `repl` after them.
fn left_adjust(buf: &mut [u8], mut lead_len: usize, repl: &[u8]) -> usize {
    let repl_len = repl.len();
    let mut p = white_run(until_nul(buf));

    // How many bytes of the old leader take as many cells as the
    // replacement; move the rest out of the way.
    let repl_size = c_display_width(repl, len_as_int(repl_len));
    let mut i = 0;
    while i < lead_len && buf[p + i] != b'\0' {
        let l = c_cluster_len(&buf[p + i..]);
        if c_display_width(&buf[p..], len_as_int(i + l)) > repl_size {
            break;
        }
        i += l;
    }
    if i != repl_len {
        let rest = (lead_len - i).saturating_sub(p);
        buf.copy_within(p + i..p + i + rest, p + repl_len);
        lead_len = lead_len + repl_len - i;
    }
    buf[p..p + repl_len].copy_from_slice(repl);

    // Blank out the rest of the old leader, keeping tabs so the indent
    // stays the same width.
    p += repl_len;
    while p < lead_len {
        if !ascii_iswhite(c_int::from(buf[p])) {
            if p + 1 < lead_len && buf[p + 1] == TAB {
                // Don't put a space in front of a TAB: drop the byte.
                lead_len -= 1;
                buf.copy_within(p + 1..lead_len + 1, p);
            } else {
                let mut l = c_cluster_len(&buf[p..]);
                if l > 1 {
                    if ptr2cells_at(&buf[p..]) > 1 {
                        // A double-width character becomes two spaces.
                        l -= 1;
                        buf[p] = b' ';
                        p += 1;
                    }
                    // Upstream moves `lead_len - p` bytes from `p + l`,
                    // which reads a little past the leader; what lands
                    // there is behind the terminator written below.
                    let end = (lead_len + l).min(buf.len());
                    buf.copy_within(p + l..end, p + 1);
                    lead_len -= l - 1;
                }
                buf[p] = b' ';
            }
        }
        p += 1;
    }
    buf[p] = b'\0';
    lead_len
}

/// What [`build_leader`] produced.
#[derive(Debug)]
pub(crate) struct BuiltLeader {
    /// The leader text, without the indent it may have carried.
    pub(crate) text: Vec<u8>,
    pub(crate) lead_len: c_int,
    pub(crate) newcol: ColNr,
    pub(crate) newindent: c_int,
}

/// What the line being split contributes to the leader's size and shape.
#[derive(Debug)]
pub(crate) struct LeaderContext {
    /// Where the leader starts in the old line when it is a comment *after*
    /// code (`code(); // why`); everything before it is blanked out so that
    /// the new line still lines up under the comment.
    pub(crate) comment_start: c_int,
    /// Bytes of the old line that will be appended after the leader.
    pub(crate) extra_len: usize,
    /// The 'formatlistpat' padding the caller may add.
    pub(crate) second_line_indent: c_int,
}

/// Copy the new line's leader out of `saved_line` and edit it into shape.
///
/// `saved_line` holds at least `plan.lead_len` bytes; `lead_flags` names the
/// 'comments' item that matched.
pub(crate) fn build_leader(
    plan: LeaderPlan,
    lead_flags: ComFlags,
    saved_line: &[u8],
    ctx: LeaderContext,
    do_si: bool,
    mut newindent: c_int,
) -> BuiltLeader {
    let LeaderContext {
        comment_start,
        extra_len,
        second_line_indent,
    } = ctx;
    let repl = plan.repl.text();
    let repl_len = repl.map_or(0, <[u8]>::len);
    let mut lead_len = usize::try_from(plan.lead_len).expect("non-negative");
    let mut extra_space = plan.extra_space;

    // Upstream's allocation: the leader, its replacement, the space, what
    // moves to the new line, and the 'comments' `n` padding the caller may
    // add.
    let bytes = lead_len
        + repl_len
        + usize::from(extra_space)
        + extra_len
        + usize::try_from(second_line_indent).unwrap_or(0)
        + 1;
    let mut buf = vec![0u8; bytes];
    buf[..lead_len].copy_from_slice(&saved_line[..lead_len]);

    // A leader found *after* code (`code(); // why`) keeps only its white
    // space, so that the new line lines up under the comment.
    // TODO(vim): handle multi-byte and double width chars
    for byte in &mut buf[..usize::try_from(comment_start).expect("a comment column")] {
        if !ascii_iswhite(c_int::from(*byte)) {
            *byte = b' ';
        }
    }

    if let Some(repl) = repl {
        let (align, mut off) = leader_alignment(lead_flags);
        lead_len = if align == COM_RIGHT {
            right_adjust(&mut buf, lead_len, repl)
        } else {
            left_adjust(&mut buf, lead_len, repl)
        };

        // The indent may have changed with the leader.
        if Buf::current().b_p_ai != 0 || do_si {
            newindent = Buf::current().indent_width(until_nul(&buf));
        }

        // Add the 'comments' numeric offset.
        if newindent + off < 0 {
            off = -newindent;
            newindent = 0;
        } else {
            newindent += off;
        }

        // Take the shift back out of the trailing spaces so the alignment
        // holds -- but not if a tab is in the way, which would change the
        // width by more than one column.
        while off > 0 && lead_len > 0 && buf[lead_len - 1] == b' ' {
            let text = until_nul(&buf);
            if text[white_run(text)..].contains(&b'\t') {
                break;
            }
            lead_len -= 1;
            off -= 1;
        }

        // A leader already ending in white space needs no extra space.
        if lead_len > 0 && ascii_iswhite(c_int::from(buf[lead_len - 1])) {
            extra_space = false;
        }
        buf[lead_len] = b'\0';
    }

    if extra_space {
        buf[lead_len] = b' ';
        lead_len += 1;
        buf[lead_len] = b'\0';
    }

    let mut newcol = ColNr::from_index(lead_len);

    // An indent is about to be set below, so drop the one the leader
    // carries -- starting past it rather than moving the text.
    let mut start = 0;
    if newindent != 0 || DID_SI.get() {
        while lead_len != 0 && ascii_iswhite(c_int::from(buf[start])) {
            lead_len -= 1;
            newcol -= 1;
            start += 1;
        }
    }
    CAN_SI.set(false);
    DID_SI.set(false);

    BuiltLeader {
        text: until_nul(&buf[start..]).to_vec(),
        lead_len: len_as_int(lead_len),
        newcol,
        newindent,
    }
}

/// A comment ended on the old line and its leader was dropped: if it was a C
/// comment and 'autoindent'/'smartindent' is on, line the new line up with
/// the line the comment *started* on.
///
/// `comment_end` is a byte offset into `saved_line`.
pub(crate) fn indent_after_comment_end(
    comment_end: usize,
    saved_line: &[u8],
    do_si: bool,
    newindent: c_int,
) -> c_int {
    if at(saved_line, comment_end) != b'*'
        || at(saved_line, comment_end + 1) != b'/'
        || (Buf::current().b_p_ai == 0 && !do_si)
    {
        return newindent;
    }
    let old_cursor = Win::current().w_cursor;
    Win::current().w_cursor.col = ColNr::from_index(comment_end);
    let newindent = match findmatch(None, 0) {
        None => newindent,
        Some(pos) => {
            Win::current().w_cursor.lnum = pos.lnum;
            get_indent()
        }
    };
    Win::current().w_cursor = old_cursor;
    newindent
}
