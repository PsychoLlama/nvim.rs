//! Deciding what comment leader the new line gets, and building it.
//!
//! Two phases. [`plan_leader`] reads the flag letters of the 'comments' item
//! that matched the *old* line and decides what the new line should carry:
//! nothing at all, the same leader, the item's *middle* part, or an empty
//! leader. [`build_leader`] then allocates the leader and edits it into shape
//! -- which is where 'comments' `l`/`r` (left/right adjusted) and the numeric
//! indent offset are honoured.
//!
//! The subtlety throughout is that a leader is measured in *screen cells*,
//! not bytes: replacing `/*` with ` *` has to keep the following text where
//! it was, and a double-width character being blanked out becomes two spaces.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{c_char, c_int, c_void};

use super::super::*;

/// What the new line's leader should be built from.
pub(crate) enum Repl {
    /// Keep the old line's leader as it is.
    Same,
    /// Replace it with the 'comments' item's middle part, held in
    /// [`LeaderPlan::middle`].
    Middle(c_int),
    /// Replace it with this text, which points into 'comments' itself.
    Text(*mut c_char, c_int),
    /// Replace it with nothing (the `f` flag: leader on the first line only).
    Blank,
}

/// [`plan_leader`]'s answer.
pub(crate) struct LeaderPlan {
    /// Bytes of the old line to take as the leader; 0 means "no leader".
    pub(crate) lead_len: c_int,
    pub(crate) repl: Repl,
    /// Storage for [`Repl::Middle`]; `copy_option_part` writes into it.
    pub(crate) middle: [c_char; COM_MAX_LEN as usize],
    /// Append a space after the leader.
    pub(crate) extra_space: bool,
    /// Where the *end* of a comment was found on the old line, if the leader
    /// was dropped because of it.
    pub(crate) comment_end: *mut c_char,
}

impl Repl {
    /// The replacement as a pointer and a length, or `None` for
    /// [`Repl::Same`].
    ///
    /// `middle` must be the buffer of the [`LeaderPlan`] this came out of:
    /// [`Repl::Middle`] points into it.
    fn resolve(&self, middle: &mut [c_char; COM_MAX_LEN as usize]) -> Option<(*mut c_char, c_int)> {
        match *self {
            Repl::Same => None,
            Repl::Middle(len) => Some((middle.as_mut_ptr(), len)),
            Repl::Text(p, len) => Some((p, len)),
            Repl::Blank => Some((c"".as_ptr().cast_mut(), 0)),
        }
    }
}

/// Step `p` to just past the next `:` of the 'comments' value, noting the `b`
/// flag on the way.
///
/// # Safety
/// `p` must point inside a NUL-terminated 'comments' value.
unsafe fn skip_flags(p: &mut *mut c_char, require_blank: &mut bool) {
    unsafe {
        while **p != 0 && c_int::from(*p.offset(-1)) != ':' as c_int {
            if c_int::from(**p) == COM_BLANK {
                *require_blank = true;
            }
            *p = p.add(1);
        }
    }
}

/// Step `p` to just past the next `:`, noting the `x` flag on the way.
///
/// The `x` flag means "closing this comment automatically is allowed", and is
/// recorded by setting `end_comment_pending` to -1 -- the "we want to set it"
/// marker the caller then replaces with the last character of the end leader.
///
/// # Safety
/// `p` must point inside a NUL-terminated 'comments' value.
unsafe fn skip_flags_noting_auto_end(p: &mut *mut c_char) {
    unsafe {
        while **p != 0 && c_int::from(*p.offset(-1)) != ':' as c_int {
            if c_int::from(**p) == COM_AUTO_END {
                end_comment_pending.set(-1);
            }
            *p = p.add(1);
        }
    }
}

/// The `s`/`m` arm: the old line carries the *start* or the *middle* of a
/// three-part comment, so the new line gets the middle part.
///
/// # Safety
/// `p` must point at the `s` or `m` in a NUL-terminated 'comments' value, and
/// `saved_line` must be the old line.
unsafe fn plan_start_or_middle(
    plan: &mut LeaderPlan,
    mut p: *mut c_char,
    dir: c_int,
    saved_line: *mut c_char,
    p_extra: *mut c_char,
    mut require_blank: bool,
) {
    unsafe {
        let current_flag = c_int::from(*p as u8);
        if current_flag == COM_START {
            if dir == BACKWARD {
                // `O` on the start of a comment inserts no leader.
                plan.lead_len = 0;
                return;
            }
            // Step over the start item to reach the middle one.
            copy_option_part(
                &raw mut p,
                plan.middle.as_mut_ptr(),
                COM_MAX_LEN as size_t,
                c",".as_ptr().cast_mut(),
            );
            require_blank = false;
        }

        // Isolate the middle leader, then the end leader.
        skip_flags(&mut p, &mut require_blank);
        let middle_len = copy_option_part(
            &raw mut p,
            plan.middle.as_mut_ptr(),
            COM_MAX_LEN as size_t,
            c",".as_ptr().cast_mut(),
        ) as c_int;

        let mut lead_end: [c_char; COM_MAX_LEN as usize] = [0; COM_MAX_LEN as usize];
        skip_flags_noting_auto_end(&mut p);
        let n = copy_option_part(
            &raw mut p,
            lead_end.as_mut_ptr(),
            COM_MAX_LEN as size_t,
            c",".as_ptr().cast_mut(),
        );
        if end_comment_pending.get() == -1 {
            end_comment_pending.set(c_int::from(lead_end[n.wrapping_sub(1) as usize] as u8));
        }

        // The comment already ends on this line, so it needs no leader.
        if dir == FORWARD {
            let mut q = saved_line.offset(plan.lead_len as isize);
            while *q != 0 {
                if strncmp(q, lead_end.as_ptr(), n) == 0 {
                    plan.comment_end = q;
                    plan.lead_len = 0;
                    return;
                }
                q = q.add(1);
            }
        }

        if plan.lead_len > 0 {
            if current_flag == COM_START {
                // `o` on the start of a comment inserts the middle leader.
                plan.repl = Repl::Middle(middle_len);
            }
            // <CR> immediately after the start leader wants a space after the
            // middle leader on the new line -- as does the `b` flag.
            let after_leader = c_int::from(*saved_line.offset(plan.lead_len as isize));
            if !ascii_iswhite(c_int::from(
                *saved_line.offset((plan.lead_len - 1) as isize),
            )) && (!p_extra.is_null() && (*curwin.get()).w_cursor.col == plan.lead_len
                || p_extra.is_null() && after_leader == NUL
                || require_blank)
            {
                plan.extra_space = true;
            }
        }
    }
}

/// The `e` arm: the old line carries the *end* of a comment.
///
/// # Safety
/// `p` must point at the `e` in `curbuf->b_p_com`, and `saved_line` be the old
/// line.
unsafe fn plan_end(plan: &mut LeaderPlan, mut p: *mut c_char, dir: c_int, saved_line: *mut c_char) {
    unsafe {
        if dir == FORWARD {
            // `o` on the end of a comment inserts no leader. Remember where
            // the end is: it may be wanted to find the start (C comments).
            plan.comment_end = skipwhite(saved_line);
            plan.lead_len = 0;
            return;
        }

        // `O` on the end of a comment inserts the middle leader, which is the
        // item before this one -- so search backwards for it.
        while p > (*curbuf.get()).b_p_com && c_int::from(*p) != ',' as c_int {
            p = p.offset(-1);
        }
        let mut repl = p;
        while repl > (*curbuf.get()).b_p_com && c_int::from(*repl.offset(-1)) != ':' as c_int {
            repl = repl.offset(-1);
        }
        plan.repl = Repl::Text(repl, p.offset_from(repl) as c_int);

        // A space after the middle leader is always right for `O` on an end.
        plan.extra_space = true;

        // Is automatic ending of the comment allowed?
        let mut p2 = p;
        while *p2 != 0 && c_int::from(*p2) != ':' as c_int {
            if c_int::from(*p2) == COM_AUTO_END {
                end_comment_pending.set(-1);
            }
            p2 = p2.add(1);
        }
        if end_comment_pending.get() == -1 {
            // The last character of the end leader is what will close it.
            while *p2 != 0 && c_int::from(*p2) != ',' as c_int {
                p2 = p2.add(1);
            }
            end_comment_pending.set(c_int::from(*p2.offset(-1) as u8));
        }
    }
}

/// Decide what leader the new line should carry, from the flag letters of the
/// 'comments' item that matched the old line.
///
/// `lead_flags` points into 'comments' at the start of that item, so the scan
/// runs up to the item's `:`.  Only the first of `s`/`m`/`e`/`f` decides;
/// `b` seen before it is remembered.
///
/// Sets `end_comment_pending`, the global that lets the *next* thing typed
/// close the comment automatically ('comments' `x`).
///
/// # Safety
/// `lead_flags` must point into `curbuf->b_p_com` and `saved_line` be the old
/// line, NUL-terminated.
pub(crate) unsafe fn plan_leader(
    dir: c_int,
    lead_len: c_int,
    lead_flags: *mut c_char,
    saved_line: *mut c_char,
    p_extra: *mut c_char,
) -> LeaderPlan {
    unsafe {
        let mut plan = LeaderPlan {
            lead_len,
            repl: Repl::Same,
            middle: [0; COM_MAX_LEN as usize],
            extra_space: false,
            comment_end: ::core::ptr::null_mut(),
        };
        let mut require_blank = false;

        let mut p = lead_flags;
        while *p != 0 && c_int::from(*p) != ':' as c_int {
            let flag = c_int::from(*p);
            if flag == COM_BLANK {
                require_blank = true;
            } else if flag == COM_START || flag == COM_MIDDLE {
                plan_start_or_middle(&mut plan, p, dir, saved_line, p_extra, require_blank);
                break;
            } else if flag == COM_END {
                plan_end(&mut plan, p, dir, saved_line);
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
            p = p.add(1);
        }
        plan
    }
}

/// The `l`/`r` alignment letter and the numeric indent offset of a 'comments'
/// item.
///
/// # Safety
/// `lead_flags` must point into a NUL-terminated 'comments' value.
unsafe fn leader_alignment(lead_flags: *mut c_char) -> (c_int, c_int) {
    unsafe {
        let mut align = 0;
        let mut off = 0;
        let mut p = lead_flags;
        while c_int::from(*p) != NUL && c_int::from(*p) != ':' as c_int {
            if c_int::from(*p) == COM_RIGHT || c_int::from(*p) == COM_LEFT {
                align = c_int::from(*p as u8);
                p = p.add(1);
            } else if ascii_isdigit(c_int::from(*p)) || c_int::from(*p) == '-' as c_int {
                off = getdigits_int(&raw mut p, true, 0);
            } else {
                p = p.add(1);
            }
        }
        (align, off)
    }
}

/// Overwrite the *right* end of `leader` with `repl`, keeping the text that
/// follows in the same screen column.
///
/// Answers the new leader length.
///
/// # Safety
/// `leader` must hold `lead_len` bytes with room for `repl_len` more.
unsafe fn right_adjust(
    leader: *mut c_char,
    mut lead_len: c_int,
    repl: *mut c_char,
    repl_len: c_int,
) -> c_int {
    unsafe {
        // Line up with the last non-white character of the old leader.
        let mut p = leader.offset((lead_len - 1) as isize);
        while p > leader && ascii_iswhite(c_int::from(*p)) {
            p = p.offset(-1);
        }
        p = p.add(1);

        // Walk back over as many *cells* as the replacement takes.
        let repl_size = vim_strnsize(repl, repl_len);
        let mut old_size = 0;
        let endp = p;
        while old_size < repl_size && p > leader {
            p = p.offset(-((utf_head_off(leader, p.offset(-1)) + 1) as isize));
            old_size += ptr2cells(p);
        }
        let shift = repl_len - endp.offset_from(p) as c_int;
        if shift != 0 {
            memmove(
                endp.offset(shift as isize) as *mut c_void,
                endp as *const c_void,
                leader.offset(lead_len as isize).offset_from(endp) as size_t,
            );
        }
        lead_len += shift;
        memmove(p as *mut c_void, repl as *const c_void, repl_len as size_t);
        if p.offset(repl_len as isize) > leader.offset(lead_len as isize) {
            *p.offset(repl_len as isize) = NUL as c_char;
        }

        // Blank out whatever is left of the old leader in front of it, a
        // double-width character becoming two spaces so the columns hold.
        loop {
            p = p.offset(-1);
            if p < leader {
                break;
            }
            let mut l = utf_head_off(leader, p);
            if l > 1 {
                p = p.offset(-(l as isize));
                if ptr2cells(p) > 1 {
                    *p.add(1) = b' ' as c_char;
                    l -= 1;
                }
                memmove(
                    p.add(1) as *mut c_void,
                    p.offset(l as isize).add(1) as *const c_void,
                    leader
                        .offset(lead_len as isize)
                        .offset_from(p.offset(l as isize).add(1)) as size_t,
                );
                lead_len -= l;
                *p = b' ' as c_char;
            } else if !ascii_iswhite(c_int::from(*p)) {
                *p = b' ' as c_char;
            }
        }
        lead_len
    }
}

/// Overwrite the *left* end of `leader` with `repl`, keeping the text that
/// follows in the same screen column.
///
/// Answers the new leader length.
///
/// # Safety
/// `leader` must hold `lead_len` bytes with room for `repl_len` more.
unsafe fn left_adjust(
    leader: *mut c_char,
    mut lead_len: c_int,
    repl: *mut c_char,
    repl_len: c_int,
) -> c_int {
    unsafe {
        let mut p = skipwhite(leader);

        // How many bytes of the old leader take as many cells as the
        // replacement; move the rest out of the way.
        let repl_size = vim_strnsize(repl, repl_len);
        let mut i = 0;
        while i < lead_len && c_int::from(*p.offset(i as isize)) != NUL {
            let l = utfc_ptr2len(p.offset(i as isize));
            if vim_strnsize(p, i + l) > repl_size {
                break;
            }
            i += l;
        }
        if i != repl_len {
            memmove(
                p.offset(repl_len as isize) as *mut c_void,
                p.offset(i as isize) as *const c_void,
                ((lead_len - i) as isize - p.offset_from(leader)) as size_t,
            );
            lead_len += repl_len - i;
        }
        memmove(p as *mut c_void, repl as *const c_void, repl_len as size_t);

        // Blank out the rest of the old leader, keeping tabs so the indent
        // stays the same width.
        p = p.offset(repl_len as isize);
        while p < leader.offset(lead_len as isize) {
            if !ascii_iswhite(c_int::from(*p)) {
                if p.add(1) < leader.offset(lead_len as isize) && c_int::from(*p.add(1)) == TAB {
                    // Don't put a space in front of a TAB: drop the byte.
                    lead_len -= 1;
                    memmove(
                        p as *mut c_void,
                        p.add(1) as *const c_void,
                        leader.offset(lead_len as isize).offset_from(p) as size_t,
                    );
                } else {
                    let mut l = utfc_ptr2len(p);
                    if l > 1 {
                        if ptr2cells(p) > 1 {
                            // A double-width character becomes two spaces.
                            l -= 1;
                            *p = b' ' as c_char;
                            p = p.add(1);
                        }
                        memmove(
                            p.add(1) as *mut c_void,
                            p.offset(l as isize) as *const c_void,
                            leader.offset(lead_len as isize).offset_from(p) as size_t,
                        );
                        lead_len -= l - 1;
                    }
                    *p = b' ' as c_char;
                }
            }
            p = p.add(1);
        }
        *p = NUL as c_char;
        lead_len
    }
}

/// What [`build_leader`] produced.
pub(crate) struct BuiltLeader {
    /// The leader text, which may have been advanced past its own indent --
    /// so this is *not* what has to be freed.
    pub(crate) leader: *mut c_char,
    /// The allocation, for `xfree`.
    pub(crate) allocated: *mut c_char,
    pub(crate) lead_len: c_int,
    pub(crate) newcol: colnr_T,
    pub(crate) newindent: c_int,
}

/// What the line being split contributes to the leader's size and shape.
pub(crate) struct LeaderContext {
    /// Where the leader starts in the old line when it is a comment *after*
    /// code (`code(); // why`); everything before it is blanked out so that
    /// the new line still lines up under the comment.
    pub(crate) comment_start: c_int,
    /// Bytes of the old line that will be appended after the leader.
    pub(crate) extra_len: c_int,
    /// The 'formatlistpat' padding the caller may add, to the same block.
    pub(crate) second_line_indent: c_int,
}

/// Allocate the new line's leader and edit it into shape.
///
/// The buffer is deliberately over-allocated: the caller appends `p_extra`,
/// and 'comments' `n` list padding, to the same block.
///
/// # Safety
/// `saved_line` must hold at least `plan.lead_len` bytes and `lead_flags`
/// point into 'comments'.
pub(crate) unsafe fn build_leader(
    mut plan: LeaderPlan,
    lead_flags: *mut c_char,
    saved_line: *mut c_char,
    ctx: LeaderContext,
    do_si: bool,
    mut newindent: c_int,
) -> BuiltLeader {
    let LeaderContext {
        comment_start,
        extra_len,
        second_line_indent,
    } = ctx;
    unsafe {
        let repl = plan.repl.resolve(&mut plan.middle);
        let repl_len = repl.map_or(0, |(_, len)| len);
        let mut lead_len = plan.lead_len;
        let mut extra_space = plan.extra_space;

        // Room for the leader, its replacement, the space, what moves to the
        // new line, and the 'comments' `n` padding the caller may add.
        let bytes = lead_len
            + repl_len
            + c_int::from(extra_space)
            + extra_len
            + second_line_indent.max(0)
            + 1;
        assert!(bytes >= 0);
        let leader = xmalloc(bytes as size_t) as *mut c_char;
        let allocated = leader;
        xmemcpyz(
            leader as *mut c_void,
            saved_line as *const c_void,
            lead_len as size_t,
        );

        // A leader found *after* code (`code(); // why`) keeps only its white
        // space, so that the new line lines up under the comment.
        // TODO(vim): handle multi-byte and double width chars
        for li in 0..comment_start {
            if !ascii_iswhite(c_int::from(*leader.offset(li as isize))) {
                *leader.offset(li as isize) = b' ' as c_char;
            }
        }

        let mut leader = leader;
        if let Some((repl, repl_len)) = repl {
            let (align, mut off) = leader_alignment(lead_flags);
            lead_len = if align == COM_RIGHT {
                right_adjust(leader, lead_len, repl, repl_len)
            } else {
                left_adjust(leader, lead_len, repl, repl_len)
            };

            // The indent may have changed with the leader.
            if (*curbuf.get()).b_p_ai != 0 || do_si {
                newindent = indent_size_ts(
                    leader,
                    (*curbuf.get()).b_p_ts,
                    (*curbuf.get()).b_p_vts_array,
                );
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
            while off > 0
                && lead_len > 0
                && c_int::from(*leader.offset((lead_len - 1) as isize)) == ' ' as c_int
            {
                if !vim_strchr(skipwhite(leader), '\t' as c_int).is_null() {
                    break;
                }
                lead_len -= 1;
                off -= 1;
            }

            // A leader already ending in white space needs no extra space.
            if lead_len > 0 && ascii_iswhite(c_int::from(*leader.offset((lead_len - 1) as isize))) {
                extra_space = false;
            }
            *leader.offset(lead_len as isize) = NUL as c_char;
        }

        if extra_space {
            *leader.offset(lead_len as isize) = b' ' as c_char;
            lead_len += 1;
            *leader.offset(lead_len as isize) = NUL as c_char;
        }

        let mut newcol = lead_len as colnr_T;

        // An indent is about to be set below, so drop the one the leader
        // carries -- advancing past it rather than moving the text.
        if newindent != 0 || did_si.get() {
            while lead_len != 0 && ascii_iswhite(c_int::from(*leader)) {
                lead_len -= 1;
                newcol -= 1;
                leader = leader.add(1);
            }
        }
        can_si.set(false);
        did_si.set(false);

        BuiltLeader {
            leader,
            allocated,
            lead_len,
            newcol,
            newindent,
        }
    }
}

/// A comment ended on the old line and its leader was dropped: if it was a C
/// comment and 'autoindent'/'smartindent' is on, line the new line up with
/// the line the comment *started* on.
///
/// # Safety
/// `comment_end` must point into `saved_line`.
pub(crate) unsafe fn indent_after_comment_end(
    comment_end: *mut c_char,
    saved_line: *mut c_char,
    do_si: bool,
    newindent: c_int,
) -> c_int {
    unsafe {
        if c_int::from(*comment_end) != '*' as c_int
            || c_int::from(*comment_end.add(1)) != '/' as c_int
            || ((*curbuf.get()).b_p_ai == 0 && !do_si)
        {
            return newindent;
        }
        let old_cursor = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.col = comment_end.offset_from(saved_line) as colnr_T;
        let pos = findmatch(::core::ptr::null_mut(), NUL);
        let newindent = if pos.is_null() {
            newindent
        } else {
            (*curwin.get()).w_cursor.lnum = (*pos).lnum;
            get_indent()
        };
        (*curwin.get()).w_cursor = old_cursor;
        newindent
    }
}
