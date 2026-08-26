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

use core::ffi::{c_char, c_int, c_void};

use crate::change::*;
use crate::types::NUL;
use crate::winlayer::{Buf, Win};

/// `memmove` inside the leader buffer [`build_leader`] allocated.
///
/// The block is over-allocated for the replacement, the extra space and the
/// text that moves to the new line, which is what makes every move below fit.
fn move_bytes(dst: *mut c_char, src: *const c_char, n: size_t) {
    // SAFETY: both ends are inside the leader's own allocation, which was
    // sized for the longest form it takes; the two may overlap.
    unsafe { memmove(dst.cast::<c_void>(), src.cast::<c_void>(), n) };
}

/// How many bytes are between `to` and `from`, both inside one leader.
fn gap(to: *const c_char, from: *const c_char) -> size_t {
    // SAFETY: both pointers are inside the same allocation.
    unsafe { to.offset_from(from) as size_t }
}

/// A cursor into the NUL-terminated `'comments'` value.
///
/// Every walk below tests the byte under the cursor before stepping, so the
/// cursor never passes the terminating NUL -- which is what makes reading
/// through it ordinary code rather than an unchecked dereference. Taking the
/// cursor in the first place is the one unchecked step.
#[derive(Clone, Copy)]
struct Com(*mut c_char);

impl Com {
    /// # Safety
    /// `p` must point inside `curbuf->b_p_com`, which is NUL-terminated, and
    /// stay valid for as long as the cursor is used.
    unsafe fn new(p: *mut c_char) -> Self {
        Self(p)
    }

    /// The byte under the cursor.
    fn byte(self) -> c_int {
        // SAFETY: the constructor's promise, and no walk here steps past the
        // terminating NUL.
        c_int::from(unsafe { *self.0 })
    }

    /// The byte `off` bytes from the cursor.
    fn byte_at(self, off: isize) -> c_int {
        // SAFETY: as [`Com::byte`]; `off` only ever names a byte the walk has
        // already passed or is about to reach.
        c_int::from(unsafe { *self.0.offset(off) })
    }

    /// The byte under the cursor as an unsigned value -- the form the flag
    /// letters are compared and stored as.
    fn flag(self) -> c_int {
        // SAFETY: as [`Com::byte`].
        c_int::from(unsafe { *self.0 } as u8)
    }

    fn step(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    fn back(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    fn raw(self) -> *mut c_char {
        self.0
    }

    /// The cursor as `copy_option_part`'s in/out argument.
    fn as_list(&mut self) -> *mut *mut c_char {
        &raw mut self.0
    }
}

/// `copy_option_part` one item into `into`, advancing `p` past it.
fn copy_part(p: &mut Com, into: *mut c_char) -> size_t {
    let sep = c",".as_ptr().cast_mut();
    // SAFETY: `p` is a position inside 'comments', and every caller's `into`
    // has room for the `COM_MAX_LEN` bytes named here.
    unsafe { copy_option_part(p.as_list(), into, COM_MAX_LEN as size_t, sep) }
}

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
fn skip_flags(p: &mut Com, require_blank: &mut bool) {
    while p.byte() != 0 && p.byte_at(-1) != ':' as c_int {
        if p.byte() == COM_BLANK {
            *require_blank = true;
        }
        p.step();
    }
}

/// Step `p` to just past the next `:`, noting the `x` flag on the way.
///
/// The `x` flag means "closing this comment automatically is allowed", and is
/// recorded by setting `end_comment_pending` to -1 -- the "we want to set it"
/// marker the caller then replaces with the last character of the end leader.
fn skip_flags_noting_auto_end(p: &mut Com) {
    while p.byte() != 0 && p.byte_at(-1) != ':' as c_int {
        if p.byte() == COM_AUTO_END {
            end_comment_pending.set(-1);
        }
        p.step();
    }
}

/// The `s`/`m` arm: the old line carries the *start* or the *middle* of a
/// three-part comment, so the new line gets the middle part.
///
/// # Safety
/// `saved_line` must be the old line, NUL-terminated.
unsafe fn plan_start_or_middle(
    plan: &mut LeaderPlan,
    mut p: Com,
    dir: c_int,
    saved_line: *mut c_char,
    p_extra: *mut c_char,
    mut require_blank: bool,
) {
    let current_flag = p.flag();
    if current_flag == COM_START {
        if dir == BACKWARD {
            // `O` on the start of a comment inserts no leader.
            plan.lead_len = 0;
            return;
        }
        // Step over the start item to reach the middle one.
        copy_part(&mut p, plan.middle.as_mut_ptr());
        require_blank = false;
    }

    // Isolate the middle leader, then the end leader.
    skip_flags(&mut p, &mut require_blank);
    let middle_len = copy_part(&mut p, plan.middle.as_mut_ptr()) as c_int;

    let mut lead_end: [c_char; COM_MAX_LEN as usize] = [0; COM_MAX_LEN as usize];
    skip_flags_noting_auto_end(&mut p);
    let n = copy_part(&mut p, lead_end.as_mut_ptr());
    if end_comment_pending.get() == -1 {
        end_comment_pending.set(c_int::from(lead_end[n.wrapping_sub(1) as usize] as u8));
    }

    // SAFETY: the caller's NUL-terminated old line, and `lead_len` bytes of
    // it are the leader that matched.
    let old = unsafe { Com::new(saved_line) };

    // The comment already ends on this line, so it needs no leader.
    if dir == FORWARD {
        // SAFETY: `lead_len` bytes into the old line is inside it.
        let mut q = unsafe { Com::new(old.raw().wrapping_offset(plan.lead_len as isize)) };
        while q.byte() != 0 {
            // SAFETY: `lead_end` holds the `n` bytes just copied into it, and
            // `q` is inside the NUL-terminated old line.
            if unsafe { strncmp(q.raw(), lead_end.as_ptr(), n) } == 0 {
                plan.comment_end = q.raw();
                plan.lead_len = 0;
                return;
            }
            q.step();
        }
    }

    if plan.lead_len > 0 {
        if current_flag == COM_START {
            // `o` on the start of a comment inserts the middle leader.
            plan.repl = Repl::Middle(middle_len);
        }
        // <CR> immediately after the start leader wants a space after the
        // middle leader on the new line -- as does the `b` flag.
        let after_leader = old.byte_at(plan.lead_len as isize);
        let last_of_leader = old.byte_at((plan.lead_len - 1) as isize);
        if !ascii_iswhite(last_of_leader)
            && (!p_extra.is_null() && cur_win().w_cursor.col == plan.lead_len
                || p_extra.is_null() && after_leader == NUL
                || require_blank)
        {
            plan.extra_space = true;
        }
    }
}

/// The `e` arm: the old line carries the *end* of a comment.
///
/// # Safety
/// `saved_line` must be the old line, NUL-terminated.
unsafe fn plan_end(plan: &mut LeaderPlan, mut p: Com, dir: c_int, saved_line: *mut c_char) {
    if dir == FORWARD {
        // `o` on the end of a comment inserts no leader. Remember where
        // the end is: it may be wanted to find the start (C comments).
        // SAFETY: the caller's NUL-terminated old line.
        plan.comment_end = unsafe { skipwhite(saved_line) };
        plan.lead_len = 0;
        return;
    }

    // `O` on the end of a comment inserts the middle leader, which is the
    // item before this one -- so search backwards for it.
    let com = cur_buf().b_p_com;
    while p.raw() > com && p.byte() != ',' as c_int {
        p.back();
    }
    let mut repl = p;
    while repl.raw() > com && repl.byte_at(-1) != ':' as c_int {
        repl.back();
    }
    // SAFETY: both cursors are inside 'comments', with `repl` at or before
    // `p`.
    let len = unsafe { p.raw().offset_from(repl.raw()) } as c_int;
    plan.repl = Repl::Text(repl.raw(), len);

    // A space after the middle leader is always right for `O` on an end.
    plan.extra_space = true;

    // Is automatic ending of the comment allowed?
    let mut p2 = p;
    while p2.byte() != 0 && p2.byte() != ':' as c_int {
        if p2.byte() == COM_AUTO_END {
            end_comment_pending.set(-1);
        }
        p2.step();
    }
    if end_comment_pending.get() == -1 {
        // The last character of the end leader is what will close it.
        while p2.byte() != 0 && p2.byte() != ',' as c_int {
            p2.step();
        }
        p2.back();
        end_comment_pending.set(p2.flag());
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
    let mut plan = LeaderPlan {
        lead_len,
        repl: Repl::Same,
        middle: [0; COM_MAX_LEN as usize],
        extra_space: false,
        comment_end: ::core::ptr::null_mut(),
    };
    let mut require_blank = false;

    // SAFETY: the caller's position inside `curbuf->b_p_com`.
    let mut p = unsafe { Com::new(lead_flags) };
    while p.byte() != 0 && p.byte() != ':' as c_int {
        let flag = p.byte();
        if flag == COM_BLANK {
            require_blank = true;
        } else if flag == COM_START || flag == COM_MIDDLE {
            // SAFETY: the caller's old line.
            unsafe { plan_start_or_middle(&mut plan, p, dir, saved_line, p_extra, require_blank) };
            break;
        } else if flag == COM_END {
            // SAFETY: as above.
            unsafe { plan_end(&mut plan, p, dir, saved_line) };
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
        p.step();
    }
    plan
}

/// The `l`/`r` alignment letter and the numeric indent offset of a 'comments'
/// item.
///
/// # Safety
/// `lead_flags` must point into a NUL-terminated 'comments' value.
unsafe fn leader_alignment(lead_flags: *mut c_char) -> (c_int, c_int) {
    let mut align = 0;
    let mut off = 0;
    // SAFETY: the caller's position inside 'comments'.
    let mut p = unsafe { Com::new(lead_flags) };
    while p.byte() != NUL && p.byte() != ':' as c_int {
        if p.byte() == COM_RIGHT || p.byte() == COM_LEFT {
            align = p.flag();
            p.step();
        } else if ascii_isdigit(p.byte()) || p.byte() == '-' as c_int {
            // SAFETY: `p` is inside 'comments' and `getdigits_int` only walks
            // the digits it finds there.
            off = unsafe { getdigits_int(p.as_list(), true, 0) };
        } else {
            p.step();
        }
    }
    (align, off)
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
    // Line up with the last non-white character of the old leader.
    let mut p = unsafe { leader.offset((lead_len - 1) as isize) };
    while p > leader && ascii_iswhite(c_int::from(unsafe { *p })) {
        p = unsafe { p.offset(-1) };
    }
    p = unsafe { p.add(1) };

    // Walk back over as many *cells* as the replacement takes.
    let repl_size = unsafe { vim_strnsize(repl, repl_len) };
    let mut old_size = 0;
    let endp = p;
    while old_size < repl_size && p > leader {
        p = unsafe { p.offset(-((utf_head_off(leader, p.offset(-1)) + 1) as isize)) };
        old_size += unsafe { ptr2cells(p) };
    }
    let shift = repl_len - gap(endp, p) as c_int;
    if shift != 0 {
        let end = leader.wrapping_offset(lead_len as isize);
        move_bytes(endp.wrapping_offset(shift as isize), endp, gap(end, endp));
    }
    lead_len += shift;
    move_bytes(p, repl, repl_len as size_t);
    if unsafe { p.offset(repl_len as isize) } > unsafe { leader.offset(lead_len as isize) } {
        unsafe { *p.offset(repl_len as isize) = NUL as c_char };
    }

    // Blank out whatever is left of the old leader in front of it, a
    // double-width character becoming two spaces so the columns hold.
    loop {
        p = unsafe { p.offset(-1) };
        if p < leader {
            break;
        }
        let mut l = unsafe { utf_head_off(leader, p) };
        if l > 1 {
            p = unsafe { p.offset(-(l as isize)) };
            if unsafe { ptr2cells(p) } > 1 {
                unsafe { *p.add(1) = b' ' as c_char };
                l -= 1;
            }
            let end = leader.wrapping_offset(lead_len as isize);
            let from = p.wrapping_offset(l as isize).wrapping_add(1);
            move_bytes(p.wrapping_add(1), from, gap(end, from));
            lead_len -= l;
            unsafe { *p = b' ' as c_char };
        } else if !ascii_iswhite(c_int::from(unsafe { *p })) {
            unsafe { *p = b' ' as c_char };
        }
    }
    lead_len
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
    let mut p = unsafe { skipwhite(leader) };

    // How many bytes of the old leader take as many cells as the
    // replacement; move the rest out of the way.
    let repl_size = unsafe { vim_strnsize(repl, repl_len) };
    let mut i = 0;
    while i < lead_len && c_int::from(unsafe { *p.offset(i as isize) }) != NUL {
        let l = unsafe { utfc_ptr2len(p.offset(i as isize)) };
        if unsafe { vim_strnsize(p, i + l) } > repl_size {
            break;
        }
        i += l;
    }
    if i != repl_len {
        let rest = ((lead_len - i) as size_t).wrapping_sub(gap(p, leader));
        move_bytes(
            p.wrapping_offset(repl_len as isize),
            p.wrapping_offset(i as isize),
            rest,
        );
        lead_len += repl_len - i;
    }
    move_bytes(p, repl, repl_len as size_t);

    // Blank out the rest of the old leader, keeping tabs so the indent
    // stays the same width.
    p = unsafe { p.offset(repl_len as isize) };
    while p < unsafe { leader.offset(lead_len as isize) } {
        if !ascii_iswhite(c_int::from(unsafe { *p })) {
            if unsafe { p.add(1) } < unsafe { leader.offset(lead_len as isize) }
                && c_int::from(unsafe { *p.add(1) }) == TAB
            {
                // Don't put a space in front of a TAB: drop the byte.
                lead_len -= 1;
                let end = leader.wrapping_offset(lead_len as isize);
                move_bytes(p, p.wrapping_add(1), gap(end, p));
            } else {
                let mut l = unsafe { utfc_ptr2len(p) };
                if l > 1 {
                    if unsafe { ptr2cells(p) } > 1 {
                        // A double-width character becomes two spaces.
                        l -= 1;
                        unsafe { *p = b' ' as c_char };
                        p = unsafe { p.add(1) };
                    }
                    let end = leader.wrapping_offset(lead_len as isize);
                    move_bytes(
                        p.wrapping_add(1),
                        p.wrapping_offset(l as isize),
                        gap(end, p),
                    );
                    lead_len -= l - 1;
                }
                unsafe { *p = b' ' as c_char };
            }
        }
        p = unsafe { p.add(1) };
    }
    unsafe { *p = NUL as c_char };
    lead_len
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
    let repl = plan.repl.resolve(&mut plan.middle);
    let repl_len = repl.map_or(0, |(_, len)| len);
    let mut lead_len = plan.lead_len;
    let mut extra_space = plan.extra_space;

    // Room for the leader, its replacement, the space, what moves to the
    // new line, and the 'comments' `n` padding the caller may add.
    let bytes =
        lead_len + repl_len + c_int::from(extra_space) + extra_len + second_line_indent.max(0) + 1;
    debug_assert!(bytes >= 0);
    let leader = unsafe { xmalloc(bytes as size_t) } as *mut c_char;
    let allocated = leader;
    unsafe {
        xmemcpyz(
            leader as *mut c_void,
            saved_line as *const c_void,
            lead_len as size_t,
        )
    };

    // A leader found *after* code (`code(); // why`) keeps only its white
    // space, so that the new line lines up under the comment.
    // TODO(vim): handle multi-byte and double width chars
    for li in 0..comment_start {
        if !ascii_iswhite(c_int::from(unsafe { *leader.offset(li as isize) })) {
            unsafe { *leader.offset(li as isize) = b' ' as c_char };
        }
    }

    let mut leader = leader;
    if let Some((repl, repl_len)) = repl {
        let (align, mut off) = unsafe { leader_alignment(lead_flags) };
        lead_len = if align == COM_RIGHT {
            unsafe { right_adjust(leader, lead_len, repl, repl_len) }
        } else {
            unsafe { left_adjust(leader, lead_len, repl, repl_len) }
        };

        // The indent may have changed with the leader.
        if cur_buf().b_p_ai != 0 || do_si {
            newindent =
                unsafe { indent_size_ts(leader, cur_buf().b_p_ts, cur_buf().b_p_vts_array) };
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
            && c_int::from(unsafe { *leader.offset((lead_len - 1) as isize) }) == ' ' as c_int
        {
            if !unsafe { vim_strchr(skipwhite(leader), '\t' as c_int) }.is_null() {
                break;
            }
            lead_len -= 1;
            off -= 1;
        }

        // A leader already ending in white space needs no extra space.
        if lead_len > 0
            && ascii_iswhite(c_int::from(unsafe {
                *leader.offset((lead_len - 1) as isize)
            }))
        {
            extra_space = false;
        }
        unsafe { *leader.offset(lead_len as isize) = NUL as c_char };
    }

    if extra_space {
        unsafe { *leader.offset(lead_len as isize) = b' ' as c_char };
        lead_len += 1;
        unsafe { *leader.offset(lead_len as isize) = NUL as c_char };
    }

    let mut newcol = lead_len as colnr_T;

    // An indent is about to be set below, so drop the one the leader
    // carries -- advancing past it rather than moving the text.
    if newindent != 0 || did_si.get() {
        while lead_len != 0 && ascii_iswhite(c_int::from(unsafe { *leader })) {
            lead_len -= 1;
            newcol -= 1;
            leader = unsafe { leader.add(1) };
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
    if c_int::from(unsafe { *comment_end }) != '*' as c_int
        || c_int::from(unsafe { *comment_end.add(1) }) != '/' as c_int
        || (cur_buf().b_p_ai == 0 && !do_si)
    {
        return newindent;
    }
    let old_cursor = cur_win().w_cursor;
    cur_win().w_cursor.col = unsafe { comment_end.offset_from(saved_line) } as colnr_T;
    let newindent = match unsafe { findmatch(::core::ptr::null_mut(), NUL) } {
        None => newindent,
        Some(pos) => {
            cur_win().w_cursor.lnum = pos.lnum;
            get_indent()
        }
    };
    cur_win().w_cursor = old_cursor;
    newindent
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
