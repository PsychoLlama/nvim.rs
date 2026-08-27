//! Placements: a sign in a buffer, which is an extmark carrying a
//! [`DecorSignHighlight`] copied out of the definition.
//!
//! The drawing code never sees a definition, only these copies -- so
//! everything here is marktree work, and every walk goes through
//! [`Cursor`], whose construction is the promise that the tree and the
//! iterator belong together.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// The `"*"` group's first byte: the "all groups" filter.
const STAR: c_char = b'*'.cast_signed();

/// Places or replaces the sign extmark `*id` in `buf` at `lnum`.
///
/// Writes `*id` back when it was zero: `extmark_set` allocates one.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_set_sign(
    buf: *mut buf_T,
    id: *mut uint32_t,
    group: *const c_char,
    prio: c_int,
    lnum: linenr_T,
    def: Sign,
) {
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };
    // The definition is copied out: the store and the extmark below can both
    // move the entry it lives in.
    let def = *def;
    // SAFETY: the caller's group name.
    let ns = if group.is_null() {
        0
    } else {
        // SAFETY: as above.
        let ns = unsafe { namespace_of(group) };
        u32::try_from(ns).expect("a namespace id fits its own handle type")
    };

    let mut sign = DECOR_SIGN_HIGHLIGHT_INIT;
    sign.flags |= kSHIsSign;
    sign.text = def.sn_text;
    // SAFETY: a definition's name is a NUL-terminated string it owns.
    sign.sign_name = unsafe { xstrdup(def.sn_name) };
    sign.hl_id = def.sn_text_hl;
    sign.line_hl_id = def.sn_line_hl;
    sign.number_hl_id = def.sn_num_hl;
    sign.cursorline_hl_id = def.sn_cul_hl;
    // Upstream's `(DecorPriority)prio`: a `:sign define priority=` value is
    // whatever `atoi` read, and the low 16 bits of it are what a placement
    // carries.
    let [lo, hi, ..] = prio.cast_unsigned().to_le_bytes();
    sign.priority = DecorPriority::from_le_bytes([lo, hi]);

    let has_hl = def.sn_line_hl != 0 || def.sn_num_hl != 0 || def.sn_cul_hl != 0;
    let text_flag = if def.sn_text[0] != 0 {
        MT_FLAG_DECOR_SIGNTEXT
    } else {
        0
    };
    let hl_flag = if has_hl { MT_FLAG_DECOR_SIGNHL } else { 0 };
    let decor_flags =
        u16::try_from(text_flag | hl_flag).expect("both sign flags fit a mark's flag word");

    let decor = DecorInline {
        ext: true,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: decor_put_sh(sign),
                vt: ::core::ptr::null_mut::<DecorVirtText>(),
            },
        },
    };
    let row = buf.line_count().min(lnum) - 1;
    // SAFETY: a live buffer, and `id` is the caller's writable out-parameter.
    unsafe {
        extmark_set(
            buf.raw(),
            ns,
            id,
            row,
            0,
            -1,
            -1,
            decor,
            decor_flags,
            true,
            false,
            true,
            true,
            ::core::ptr::null_mut::<Error>(),
        )
    };
}

/// Re-places the existing sign `*id` where it already is, so that a
/// `:sign place {id} name=...` with no `line=` changes its type or priority.
///
/// Answers its line number, or zero when there is no such sign.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_mod_sign(
    buf: *mut buf_T,
    id: *mut uint32_t,
    group: *const c_char,
    prio: c_int,
    def: Sign,
) -> linenr_T {
    // SAFETY: the caller's group name.
    let Some(ns) = (unsafe { placed_ns(group) }) else {
        return 0;
    };
    // SAFETY: the caller's buffer and out-parameter.
    let mark = unsafe { lookup_ns(Buf::new(buf), ns, *id, false) };
    if mark.pos.row >= 0 {
        // SAFETY: the caller's buffer, group and out-parameter.
        unsafe { buf_set_sign(buf, id, group, prio, mark.pos.row + 1, def) };
    }
    mark.pos.row + 1
}

/// The one namespace a placement lookup runs in, or `None` for a group that
/// names none.
///
/// The two callers below both reject "no such group" *and* a named group
/// that resolved to the global namespace, which is the same fact twice: only
/// a null `group` may be namespace 0.
///
/// # Safety
/// `group` must be null or a NUL-terminated string.
unsafe fn placed_ns(group: *const c_char) -> Option<uint32_t> {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 || (!group.is_null() && ns == 0) {
        return None;
    }
    Some(u32::try_from(ns).expect("a namespace id fits its own handle type"))
}

/// The line the sign `id` sits on in `group`, or zero when there is none.
///
/// Zero rather than an error, so that `:sign jump` still loads the file.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_findsign(buf: *mut buf_T, id: c_int, group: *const c_char) -> c_int {
    // SAFETY: the caller's group name.
    let Some(ns) = (unsafe { placed_ns(group) }) else {
        return 0;
    };
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };
    lookup_ns(buf, ns, id.cast_unsigned(), false).pos.row + 1
}

/// Orders marks the way `:sign` reports them and removes them: by row, then
/// by [`sign_item_cmp`] — priority, then mark id, then placement serial, all
/// newest first.
///
/// A stable sort is provably the permutation the `qsort` upstream uses
/// produced: `buf_put_decor_sh` hands every placed sign a distinct
/// `sign_add_id`, so the comparator is a total order and no two entries tie.
///
/// # Safety
/// Every mark must carry a live sign decoration.
pub(crate) unsafe fn sort_signs(signs: &mut [MTKey]) {
    signs.sort_by(|a, b| {
        if a.pos.row != b.pos.row {
            return a.pos.row.cmp(&b.pos.row);
        }
        let (sh1, sh2) = (decor_find_sign(mt_decor(*a)), decor_find_sign(mt_decor(*b)));
        assert!(!sh1.is_null() && !sh2.is_null(), "sign mark without a sign");
        let (ia, ib) = (
            SignItem { sh: sh1, id: a.id },
            SignItem { sh: sh2, id: b.id },
        );
        // SAFETY: the caller's marks, whose sign items the store just named.
        unsafe { sign_item_cmp(&ia, &ib) }
    });
}

/// Whether `mark` is a sign placement this walk should report, in `ns`.
fn wanted_sign(mark: MTKey, ns: int64_t) -> bool {
    !mt_end(mark) && mt_decor_sign(mark) && (ns == ALL_GROUPS || int64_t::from(mark.ns) == ns)
}

/// Every sign placement in `buf` from `first_row` on, in marktree order.
///
/// The walk `:sign place`, `sign_getplaced()` and `getbufinfo()` share.
/// `keep` narrows it further; it sees only marks that are already signs in
/// the right group.
pub(super) fn placed_signs(
    buf: Buf,
    first_row: int32_t,
    ns: int64_t,
    mut keep: impl FnMut(MTKey) -> Keep,
) -> Vec<MTKey> {
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);
    walk.seek(first_row, 0);
    let mut out = Vec::new();
    for mark in walk.marks() {
        match keep(mark) {
            Keep::Stop => break,
            Keep::Yes if wanted_sign(mark, ns) => out.push(mark),
            _ => {}
        }
    }
    out
}

/// What [`placed_signs`]' filter says about one mark.
pub(super) enum Keep {
    /// Report it, if it is a sign in the right group.
    Yes,
    /// Skip it and carry on.
    No,
    /// End the walk here; the tree is in row order, so nothing further can
    /// match either.
    Stop,
}

/// Deletes signs from `buf`.
///
/// `id` of zero means any id and `group` selects a namespace (see
/// [`group_get_ns`]). `atlnum` above zero narrows to one line — where,
/// unlike every other combination, only the **highest priority** sign goes.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_delete_signs(
    buf: *mut buf_T,
    group: *const c_char,
    id: c_int,
    atlnum: linenr_T,
) -> c_int {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 {
        return FAIL;
    }
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };

    let mut itr = MarkTreeIter::default();
    let row = if atlnum > 0 { atlnum - 1 } else { 0 };
    let mut signs: Vec<MTKey> = Vec::new();

    // The walk continues below as a bare iterator: `extmark_del` steps it
    // itself, which a `Cursor` cannot express.
    {
        let mut walk = Cursor::in_buffer(buf, &mut itr);
        if atlnum > 0 {
            // Signs that *started* above this row but still cover it.
            if !walk.seek_overlap(row, 0) {
                return FAIL;
            }
            while let Some(pair) = walk.step_overlap() {
                if (ns == ALL_GROUPS || int64_t::from(pair.start.ns) == ns)
                    && mt_decor_sign(pair.start)
                {
                    signs.push(pair.start);
                }
            }
        } else {
            walk.seek(0, 0);
        }
    }

    let tree = tree_of(buf);
    while !itr.x.is_null() {
        // SAFETY: the iterator is positioned in this buffer's live tree.
        let mark = unsafe { marktree_itr_current(&mut itr) };
        if row != 0 && mark.pos.row > row {
            break;
        }
        let wanted = wanted_sign(mark, ns) && (id == 0 || mark.id.cast_signed() == id);
        if wanted && atlnum <= 0 {
            // `extmark_del` advances the iterator itself.
            // SAFETY: as above, plus a live buffer.
            unsafe { extmark_del(buf.raw(), &raw mut itr, mark, true) };
            continue;
        }
        if wanted {
            signs.push(mark);
        }
        // SAFETY: as above.
        unsafe { marktree_itr_next(&mut *tree, &mut itr) };
    }

    if signs.is_empty() {
        // Only the single-line form treats "nothing matched" as failure; the
        // sweeping forms are content to have deleted nothing.
        return if atlnum > 0 { FAIL } else { OK };
    }
    // SAFETY: every mark collected above carries a live sign decoration.
    unsafe { sort_signs(&mut signs) };
    // SAFETY: a live buffer.
    unsafe { extmark_del_id(buf.raw(), signs[0].ns, signs[0].id) };
    OK
}

/// Whether `buf` carries any sign at all — text or highlight.
///
/// # Safety
/// `buf` must be live.
pub(crate) unsafe fn buf_has_signs(buf: *const buf_T) -> bool {
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf.cast_mut()) };
    buf.meta_total(kMTMetaSignHL) + buf.meta_total(kMTMetaSignText) != 0
}

/// Places the sign `name` in `buf`, or changes the existing sign `*id`.
///
/// `lnum` above zero places; zero re-places the existing sign where it is,
/// which is how `:sign place {id} name=X buffer=N` changes a sign's type.
/// `prio` of `-1` takes the definition's, or [`SIGN_DEF_PRIO`].
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated; `name` must
/// be NUL-terminated.
pub(crate) unsafe fn sign_place(
    id: *mut uint32_t,
    group: *const c_char,
    name: *mut c_char,
    buf: *mut buf_T,
    lnum: linenr_T,
    prio: c_int,
) -> c_int {
    // `*` is the "all groups" filter, not a group one can place into.
    // SAFETY: the caller's group name, null or NUL-terminated.
    if !group.is_null() && unsafe { *group == STAR || *group == 0 } {
        return FAIL;
    }

    // SAFETY: the caller's name.
    let Some(def) = (unsafe { sign_find(name) }) else {
        // SAFETY: as above, and a format the message takes.
        unsafe { semsg_c!(gettext(c"E155: Unknown sign: %s".as_ptr()), name) };
        return FAIL;
    };
    let prio = match (prio, def.sn_priority) {
        (-1, -1) => SIGN_DEF_PRIO,
        (-1, own) => own,
        (given, _) => given,
    };

    // SAFETY: the caller's buffer, group and out-parameter.
    let lnum = unsafe {
        if lnum > 0 {
            buf_set_sign(buf, id, group, prio, lnum, def);
            lnum
        } else {
            buf_mod_sign(buf, id, group, prio, def)
        }
    };
    if lnum <= 0 {
        // SAFETY: the caller's name.
        unsafe {
            semsg_c!(
                gettext(c"E885: Not possible to change sign %s".as_ptr()),
                name,
            )
        };
        return FAIL;
    }
    OK
}

/// [`sign_unplace`] for one buffer.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn sign_unplace_inner(
    buf: *mut buf_T,
    id: c_int,
    group: *const c_char,
    atlnum: linenr_T,
) -> c_int {
    // SAFETY: the caller's buffer.
    if !unsafe { buf_has_signs(buf) } {
        return FAIL;
    }
    // SAFETY: the caller's group name, null or NUL-terminated.
    let all_groups = !group.is_null() && unsafe { *group } == STAR;
    if id == 0 || atlnum > 0 || all_groups {
        // SAFETY: the caller's buffer and group.
        return unsafe { buf_delete_signs(buf, group, id, atlnum) };
    }
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 {
        return FAIL;
    }
    let ns = u32::try_from(ns).expect("a namespace id fits its own handle type");
    // SAFETY: the caller's buffer.
    if !unsafe { extmark_del_id(buf, ns, id.cast_unsigned()) } {
        return FAIL;
    }
    OK
}

/// Removes signs from `buf`, or from every buffer when `buf` is null.
///
/// # Safety
/// `buf` must be null or live; `group` must be null or NUL-terminated.
pub(crate) unsafe fn sign_unplace(
    buf: *mut buf_T,
    id: c_int,
    group: *const c_char,
    atlnum: linenr_T,
) -> c_int {
    if !buf.is_null() {
        // SAFETY: the caller's buffer and group.
        return unsafe { sign_unplace_inner(buf, id, group, atlnum) };
    }
    let mut retval = OK;
    for cbuf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and the caller's
        // group name.
        if unsafe { sign_unplace_inner(cbuf.raw(), id, group, atlnum) } == FAIL {
            retval = FAIL;
        }
    }
    retval
}

/// Moves the cursor to the sign `id`, opening `buf` if no window shows it.
///
/// Answers the line jumped to, or `-1`.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
pub(crate) unsafe fn sign_jump(id: c_int, group: *const c_char, buf: *mut buf_T) -> linenr_T {
    // SAFETY: the caller's buffer and group.
    let lnum = unsafe { buf_findsign(buf, id, group) };
    if lnum <= 0 {
        // SAFETY: a format the message takes.
        unsafe { semsg_c!(gettext(c"E157: Invalid sign ID: %d".as_ptr()), id) };
        return -1;
    }
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };

    // SAFETY: a live buffer.
    if !unsafe { buf_jump_open_win(buf.raw()) }.is_null() {
        // SAFETY: `curwin` is live from startup to exit.
        let mut win = unsafe { Win::current() };
        win.w_cursor.lnum = lnum;
        check_cursor_lnum(win);
        beginline(BeginlineOpts::WHITE);
    } else {
        if buf.b_fname.is_null() {
            // SAFETY: a static message.
            unsafe {
                emsg(gettext(
                    c"E934: Cannot jump to a buffer that does not have a name".as_ptr(),
                ))
            };
            return -1;
        }
        // SAFETY: a live buffer's name is a NUL-terminated string it owns.
        let cmdlen = unsafe { strlen(buf.b_fname) } + 24;
        let mut cmd = vec![0 as c_char; cmdlen + 1];
        // SAFETY: as above; `cmd` has room for `cmdlen` bytes plus the NUL.
        unsafe {
            snprintf(
                cmd.as_mut_ptr(),
                cmdlen,
                c"e +%ld %s".as_ptr(),
                int64_t::from(lnum),
                buf.b_fname,
            );
            do_cmdline_cmd(cmd.as_mut_ptr());
        };
    }

    // SAFETY: the editor's own fold state.
    unsafe { fold_open_cursor() };
    lnum
}
