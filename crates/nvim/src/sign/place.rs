//! Placements: a sign in a buffer, which is an extmark carrying a
//! [`DecorSignHighlight`] copied out of the definition.
//!
//! The drawing code never sees a definition, only these copies -- so
//! everything here is marktree work, and every walk goes through
//! [`Cursor`], whose construction is the promise that the tree and the
//! iterator belong together.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::decoration::decor_sign;
use crate::memory::ThinCString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::Failed;

/// Places or replaces the sign extmark `*id` in `buffer` at `lnum`.
///
/// Writes `*id` back when it was zero: `extmark_set` allocates one.
fn buf_set_sign(
    buffer: Buf,
    id: &mut uint32_t,
    group: Option<&[u8]>,
    prio: c_int,
    lnum: LineNr,
    def: &Sign,
) {
    let ns = group.map_or(0, |group| {
        u32::try_from(namespace_of(group)).expect("a namespace id fits its own handle type")
    });

    let mut sign = DECOR_SIGN_HIGHLIGHT_INIT;
    sign.flags |= kSHIsSign;
    sign.text = def.text;
    // The decoration store frees it with the placement.
    sign.sign_name = ThinCString::from_cstr(&def.name).into_raw();
    sign.hl_id = def.text_hl;
    sign.line_hl_id = def.line_hl;
    sign.number_hl_id = def.num_hl;
    sign.cursorline_hl_id = def.cul_hl;
    // Upstream's `(DecorPriority)prio`: a `:sign define priority=` value is
    // whatever `atoi` read, and the low 16 bits of it are what a placement
    // carries.
    let [lo, hi, ..] = prio.cast_unsigned().to_le_bytes();
    sign.priority = DecorPriority::from_le_bytes([lo, hi]);

    let has_hl = def.line_hl != 0 || def.num_hl != 0 || def.cul_hl != 0;
    let decor_flags =
        MtFlags::DECOR_SIGNTEXT.when(def.text[0] != 0) | MtFlags::DECOR_SIGNHL.when(has_hl);

    let decor = DecorInline {
        ext: true,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: decor_put_sh(sign),
                vt: ::core::ptr::null_mut::<DecorVirtText>(),
            },
        },
    };
    let row = buffer.line_count().min(lnum) - 1;
    extmark_set(
        buffer,
        ns,
        Some(id),
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
    );
}

/// Re-places the existing sign `*id` where it already is, so that a
/// `:sign place {id} name=...` with no `line=` changes its type or priority.
///
/// Answers its line number, or zero when there is no such sign.
fn buf_mod_sign(
    buffer: Buf,
    id: &mut uint32_t,
    group: Option<&[u8]>,
    prio: c_int,
    def: &Sign,
) -> LineNr {
    let Some(ns) = placed_ns(group) else {
        return 0;
    };
    let mark = lookup_ns(buffer, ns, *id, false);
    if mark.pos.row >= 0 {
        buf_set_sign(buffer, id, group, prio, mark.pos.row + 1, def);
    }
    mark.pos.row + 1
}

/// The one namespace a placement lookup runs in, or `None` for a group that
/// names none.
///
/// The two callers below both reject "no such group" *and* a named group
/// that resolved to the global namespace, which is the same fact twice: only
/// the global group (`None`) may be namespace 0.
fn placed_ns(group: Option<&[u8]>) -> Option<uint32_t> {
    let ns = group_get_ns(group);
    if ns < 0 || (group.is_some() && ns == 0) {
        return None;
    }
    Some(u32::try_from(ns).expect("a namespace id fits its own handle type"))
}

/// The line the sign `id` sits on in `group`, or zero when there is none.
///
/// Zero rather than an error, so that `:sign jump` still loads the file.
fn buf_findsign(buffer: Buf, id: c_int, group: Option<&[u8]>) -> c_int {
    let Some(ns) = placed_ns(group) else {
        return 0;
    };
    lookup_ns(buffer, ns, id.cast_unsigned(), false).pos.row + 1
}

/// The sign decoration `mark` carries.
///
/// # Panics
/// When it carries none: every walk here only keeps sign marks.
pub(super) fn sign_of(mark: MTKey) -> Sh {
    decor_sign(mt_decor(mark)).expect("sign mark without a sign")
}

/// Orders marks the way `:sign` reports them and removes them: by row, then
/// by [`sign_item_cmp`] — priority, then mark id, then placement serial, all
/// newest first.
///
/// A stable sort is provably the permutation the `qsort` upstream uses
/// produced: `buf_put_decor_sh` hands every placed sign a distinct
/// `sign_add_id`, so the comparator is a total order and no two entries tie.
pub(crate) fn sort_signs(signs: &mut [MTKey]) {
    signs.sort_by(|a, b| {
        if a.pos.row != b.pos.row {
            return a.pos.row.cmp(&b.pos.row);
        }
        let (ia, ib) = (
            SignItem {
                sh: sign_of(*a).raw(),
                id: a.id,
            },
            SignItem {
                sh: sign_of(*b).raw(),
                id: b.id,
            },
        );
        sign_item_cmp(&ia, &ib)
    });
}

/// Whether `mark` is a sign placement this walk should report, in `ns`.
fn wanted_sign(mark: MTKey, ns: int64_t) -> bool {
    !mt_end(mark) && mt_decor_sign(mark) && (ns == ALL_GROUPS || int64_t::from(mark.ns) == ns)
}

/// Every sign placement in `buffer` from `first_row` on, in marktree order.
///
/// The walk `:sign place`, `sign_getplaced()` and `getbufinfo()` share.
/// `keep` narrows it further; it sees only marks that are already signs in
/// the right group.
pub(super) fn placed_signs(
    buffer: Buf,
    first_row: int32_t,
    ns: int64_t,
    mut keep: impl FnMut(MTKey) -> Keep,
) -> Vec<MTKey> {
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buffer, &mut itr);
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

/// Deletes signs from `buffer`.
///
/// `id` of zero means any id and `group` selects a namespace (see
/// [`group_get_ns`]). `atlnum` above zero narrows to one line — where,
/// unlike every other combination, only the **highest priority** sign goes.
fn buf_delete_signs(buffer: Buf, group: Option<&[u8]>, id: c_int, atlnum: LineNr) -> c_int {
    let ns = group_get_ns(group);
    if ns < 0 {
        return FAIL;
    }

    let mut itr = MarkTreeIter::default();
    let row = if atlnum > 0 { atlnum - 1 } else { 0 };
    let mut signs: Vec<MTKey> = Vec::new();

    {
        let mut walk = Cursor::in_buffer(buffer, &mut itr);
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

    // The walk goes on with a cursor per step: `extmark::del` steps the
    // iterator itself, which one cursor held across the loop cannot let it.
    while !itr.x.is_null() {
        let mark = marktree_itr_current(&mut itr);
        if row != 0 && mark.pos.row > row {
            break;
        }
        let wanted = wanted_sign(mark, ns) && (id == 0 || mark.id.cast_signed() == id);
        if wanted && atlnum <= 0 {
            // `extmark::del` advances the iterator itself.
            extmark::del(buffer, &mut itr, mark, true);
            continue;
        }
        if wanted {
            signs.push(mark);
        }
        Cursor::in_buffer(buffer, &mut itr).step();
    }

    if signs.is_empty() {
        // Only the single-line form treats "nothing matched" as failure; the
        // sweeping forms are content to have deleted nothing.
        return if atlnum > 0 { FAIL } else { OK };
    }
    sort_signs(&mut signs);
    extmark_del_id(buffer, signs[0].ns, signs[0].id);
    OK
}

/// Whether `buffer` carries any sign at all — text or highlight.
pub(crate) fn buf_has_signs(buffer: Buf) -> bool {
    buffer.meta_total(kMTMetaSignHL) + buffer.meta_total(kMTMetaSignText) != 0
}

/// Places the sign `name` in `buffer`, or changes the existing sign `*id`.
///
/// `lnum` above zero places; zero re-places the existing sign where it is,
/// which is how `:sign place {id} name=X buffer=N` changes a sign's type.
/// `prio` of `-1` takes the definition's, or [`SIGN_DEF_PRIO`].
pub(crate) fn sign_place(
    id: &mut uint32_t,
    group: Option<&[u8]>,
    name: &[u8],
    buffer: Buf,
    lnum: LineNr,
    prio: c_int,
) -> Result<(), Failed> {
    // `*` is the "all groups" filter, not a group one can place into.
    if group.is_some_and(|group| matches!(group.first(), None | Some(b'*'))) {
        return Err(Failed);
    }

    let Some(def) = sign_find(name) else {
        semsg!("E155: Unknown sign: {}", msg_bytes(name));
        return Err(Failed);
    };
    let prio = match (prio, def.priority) {
        (-1, -1) => SIGN_DEF_PRIO,
        (-1, own) => own,
        (given, _) => given,
    };

    let lnum = if lnum > 0 {
        buf_set_sign(buffer, id, group, prio, lnum, &def);
        lnum
    } else {
        buf_mod_sign(buffer, id, group, prio, &def)
    };
    if lnum <= 0 {
        semsg!("E885: Not possible to change sign {}", msg_bytes(name));
        return Err(Failed);
    }
    Ok(())
}

/// [`sign_unplace`] for one buffer.
fn sign_unplace_inner(buffer: Buf, id: c_int, group: Option<&[u8]>, atlnum: LineNr) -> c_int {
    if !buf_has_signs(buffer) {
        return FAIL;
    }
    let all_groups = group.is_some_and(|group| group.first() == Some(&b'*'));
    if id == 0 || atlnum > 0 || all_groups {
        return buf_delete_signs(buffer, group, id, atlnum);
    }
    let ns = group_get_ns(group);
    if ns < 0 {
        return FAIL;
    }
    let ns = u32::try_from(ns).expect("a namespace id fits its own handle type");
    if !extmark_del_id(buffer, ns, id.cast_unsigned()) {
        return FAIL;
    }
    OK
}

/// Removes signs from `buffer`, or from every buffer when it is `None`.
pub(crate) fn sign_unplace(
    buffer: Option<Buf>,
    id: c_int,
    group: Option<&[u8]>,
    atlnum: LineNr,
) -> c_int {
    if let Some(buffer) = buffer {
        return sign_unplace_inner(buffer, id, group, atlnum);
    }
    let mut retval = OK;
    for cbuf in buffers() {
        if sign_unplace_inner(cbuf, id, group, atlnum) == FAIL {
            retval = FAIL;
        }
    }
    retval
}

/// Moves the cursor to the sign `id`, opening `buffer` if no window shows it.
///
/// Answers the line jumped to, or `-1`. Runs autocommands -- the window
/// change, or the `:edit` -- so it holds nothing of the caller's across them.
pub(crate) fn sign_jump(id: c_int, group: Option<&[u8]>, buffer: Buf) -> LineNr {
    let lnum = buf_findsign(buffer, id, group);
    if lnum <= 0 {
        semsg!("E157: Invalid sign ID: {}", id);
        return -1;
    }
    if buf_jump_open_win(buffer).is_some() {
        let mut win = Win::current();
        win.w_cursor.lnum = lnum;
        check_cursor_lnum(win);
        beginline(BeginlineOpts::WHITE);
    } else {
        let Some(name) = buffer.name.shown() else {
            emsg(gettext(
                c"E934: Cannot jump to a buffer that does not have a name",
            ));
            return -1;
        };
        let mut cmd = format!("e +{lnum} ").into_bytes();
        cmd.extend_from_slice(name.to_bytes());
        let cmd = CString::new(cmd).expect("a buffer name is a C string");
        let _ = do_cmdline_cmd(&cmd);
    }

    fold_open_cursor();
    lnum
}
