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
use crate::types::ColNr;

impl Buf {
    /// Places or replaces the sign extmark `*id` in `self` at `lnum`.
    ///
    /// Writes `*id` back when it was zero: `extmark_set` allocates one.
    fn set_sign(
        self,
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
        // The decoration store drops it with the placement.
        sign.sign_name = Some(def.name.clone());
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

        let decor = DecorInline::Ext(DecorExt {
            sh_idx: decor_put_sh(sign),
            vt_idx: DECOR_ID_INVALID,
        });
        let row = self.line_count().min(lnum).get() - 1;
        self.set_extmark(
            ns,
            Some(id),
            row,
            ColNr::ZERO,
            -1,
            ColNr::NONE,
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
    fn modify_sign(
        self,
        id: &mut uint32_t,
        group: Option<&[u8]>,
        prio: c_int,
        def: &Sign,
    ) -> LineNr {
        let Some(ns) = placed_ns(group) else {
            return LineNr::ZERO;
        };
        let mark = lookup_ns(self, ns, *id, false);
        if mark.pos.row >= 0 {
            self.set_sign(id, group, prio, LineNr::new(mark.pos.row + 1), def);
        }
        LineNr::new(mark.pos.row + 1)
    }

    /// The line the sign `id` sits on in `group`, or zero when there is none.
    ///
    /// Zero rather than an error, so that `:sign jump` still loads the file.
    fn find_sign(self, id: c_int, group: Option<&[u8]>) -> c_int {
        let Some(ns) = placed_ns(group) else {
            return 0;
        };
        lookup_ns(self, ns, id.cast_unsigned(), false).pos.row + 1
    }

    /// Every sign placement in `self` from `first_row` on, in marktree order.
    ///
    /// The walk `:sign place`, `sign_getplaced()` and `getbufinfo()` share.
    /// `keep` narrows it further; it sees only marks that are already signs in
    /// the right group.
    pub(super) fn placed_signs(
        self,
        first_row: int32_t,
        ns: int64_t,
        mut keep: impl FnMut(MTKey) -> Keep,
    ) -> Vec<MTKey> {
        let mut itr = MarkTreeIter::default();
        let mut walk = Cursor::in_buffer(self, &mut itr);
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

    /// Deletes signs from `self`.
    ///
    /// `id` of zero means any id and `group` selects a namespace (see
    /// [`group_get_ns`]). `atlnum` above zero narrows to one line — where,
    /// unlike every other combination, only the **highest priority** sign goes.
    /// [`SignError::Quiet`] when the group names nothing, or when the one-line
    /// form found nothing to remove.
    fn delete_signs(
        self,
        group: Option<&[u8]>,
        id: c_int,
        atlnum: LineNr,
    ) -> Result<(), SignError> {
        let ns = group_get_ns(group);
        if ns < 0 {
            return Err(SignError::Quiet);
        }

        let mut itr = MarkTreeIter::default();
        let row = if atlnum > LineNr::ZERO {
            atlnum.get() - 1
        } else {
            0
        };
        let mut signs: Vec<MTKey> = Vec::new();

        {
            let mut walk = Cursor::in_buffer(self, &mut itr);
            if atlnum > LineNr::ZERO {
                // Signs that *started* above this row but still cover it.
                if !walk.seek_overlap(row, 0) {
                    return Err(SignError::Quiet);
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
        while itr.is_positioned() {
            let mark = Cursor::in_buffer(self, &mut itr).current();
            if row != 0 && mark.pos.row > row {
                break;
            }
            let wanted = wanted_sign(mark, ns) && (id == 0 || mark.id.cast_signed() == id);
            if wanted && atlnum <= LineNr::ZERO {
                // `extmark::del` advances the iterator itself.
                extmark::del(self, &mut itr, mark, true);
                continue;
            }
            if wanted {
                signs.push(mark);
            }
            Cursor::in_buffer(self, &mut itr).step();
        }

        if signs.is_empty() {
            // Only the single-line form treats "nothing matched" as failure; the
            // sweeping forms are content to have deleted nothing.
            return if atlnum > LineNr::ZERO {
                Err(SignError::Quiet)
            } else {
                Ok(())
            };
        }
        sort_signs(&mut signs);
        extmark_del_id(self, signs[0].ns, signs[0].id);
        Ok(())
    }

    /// [`sign_unplace`] for one buffer.
    fn unplace_sign(
        self,
        id: c_int,
        group: Option<&[u8]>,
        atlnum: LineNr,
    ) -> Result<(), SignError> {
        if !buf_has_signs(self) {
            return Err(SignError::Quiet);
        }
        let all_groups = group.is_some_and(|group| group.first() == Some(&b'*'));
        if id == 0 || atlnum > LineNr::ZERO || all_groups {
            return self.delete_signs(group, id, atlnum);
        }
        let ns = group_get_ns(group);
        if ns < 0 {
            return Err(SignError::Quiet);
        }
        let ns = u32::try_from(ns).expect("a namespace id fits its own handle type");
        if !extmark_del_id(self, ns, id.cast_unsigned()) {
            return Err(SignError::Quiet);
        }
        Ok(())
    }
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

/// A copy of the sign decoration `mark` carries.
///
/// # Panics
/// When it carries none: every walk here only keeps sign marks.
pub(super) fn sign_of(mark: MTKey) -> DecorSignHighlight {
    let idx = decor_sign(mt_decor(mark)).expect("sign mark without a sign");
    with_decor_item(idx, DecorSignHighlight::clone)
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
        let item = |mark: MTKey| sign_item(mark).expect("sign mark without a sign");
        sign_item_cmp(&item(*a), &item(*b))
    });
}

/// Whether `mark` is a sign placement this walk should report, in `ns`.
fn wanted_sign(mark: MTKey, ns: int64_t) -> bool {
    !mt_end(mark) && mt_decor_sign(mark) && (ns == ALL_GROUPS || int64_t::from(mark.ns) == ns)
}

/// What [`Buf::placed_signs`]' filter says about one mark.
#[derive(Debug)]
pub(super) enum Keep {
    /// Report it, if it is a sign in the right group.
    Yes,
    /// Skip it and carry on.
    No,
    /// End the walk here; the tree is in row order, so nothing further can
    /// match either.
    Stop,
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
) -> Result<(), SignError> {
    // `*` is the "all groups" filter, not a group one can place into.
    if group.is_some_and(|group| matches!(group.first(), None | Some(b'*'))) {
        return Err(SignError::Quiet);
    }

    let Some(def) = sign_find(name) else {
        return Err(SignError::UnknownSign(name.into()));
    };
    let prio = match (prio, def.priority) {
        (-1, -1) => SIGN_DEF_PRIO,
        (-1, own) => own,
        (given, _) => given,
    };

    let lnum = if lnum > LineNr::ZERO {
        buffer.set_sign(id, group, prio, lnum, &def);
        lnum
    } else {
        buffer.modify_sign(id, group, prio, &def)
    };
    if lnum <= LineNr::ZERO {
        return Err(SignError::CannotChange(name.into()));
    }
    Ok(())
}

/// Removes signs from `buffer`, or from every buffer when it is `None`.
///
/// [`SignError::Quiet`] when one of them had nothing to remove: every buffer
/// is still tried.
pub(crate) fn sign_unplace(
    buffer: Option<Buf>,
    id: c_int,
    group: Option<&[u8]>,
    atlnum: LineNr,
) -> Result<(), SignError> {
    if let Some(buffer) = buffer {
        return buffer.unplace_sign(id, group, atlnum);
    }
    let mut retval = Ok(());
    for cbuf in buffers() {
        if let Err(quiet) = cbuf.unplace_sign(id, group, atlnum) {
            retval = Err(quiet);
        }
    }
    retval
}

/// Moves the cursor to the sign `id`, opening `buffer` if no window shows it.
///
/// Answers the line jumped to. Runs autocommands -- the window change, or
/// the `:edit` -- so it holds nothing of the caller's across them.
pub(crate) fn sign_jump(id: c_int, group: Option<&[u8]>, buffer: Buf) -> Result<LineNr, SignError> {
    let lnum = buffer.find_sign(id, group);
    if lnum <= 0 {
        return Err(SignError::InvalidId(id));
    }
    if buf_jump_open_win(buffer).is_some() {
        let mut win = Win::current();
        win.w_cursor.lnum = LineNr::new(lnum);
        check_cursor_lnum(win);
        beginline(BeginlineOpts::WHITE);
    } else {
        let Some(name) = buffer.name.shown() else {
            return Err(SignError::NoBufferName);
        };
        let mut cmd = format!("e +{lnum} ").into_bytes();
        cmd.extend_from_slice(name.to_bytes());
        let cmd = CString::new(cmd).expect("a buffer name is a C string");
        let _ = do_cmdline_cmd(&cmd);
    }

    fold_open_cursor();
    Ok(LineNr::new(lnum))
}
