//! Inserting and deleting a line: the two operations that change
//! the tree's shape.
//!
//! Everything else in memline reads the tree or rewrites a line in place.
//! These two are where a data block runs out of room and has to be split (and
//! then, possibly, the pointer block above it, all the way to the root), and
//! where a data block empties and has to be unhooked again.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_long, c_uint};

#[allow(unused_imports)]
use super::*;

/// How a data block split came out, in the terms the pointer block above it
/// has to be told: two blocks side by side, each with a block number, a line
/// count and a page count.
///
/// The pointer-block walk mutates this as it climbs — once a pointer block
/// itself splits, *it* becomes the left/right pair its own parent must
/// describe.
struct SplitBlocks {
    bnum_left: blocknr_T,
    bnum_right: blocknr_T,
    line_count_left: c_int,
    line_count_right: c_int,
    page_count_left: c_int,
    page_count_right: c_int,
    /// `pe_old_lnum`: which line the block held first when the file was read,
    /// which is all `:recover` has to order blocks by. Zero means "leave the
    /// entry's alone".
    lnum_left: linenr_T,
    lnum_right: linenr_T,
    /// Lines added to the old block that the pointer blocks above it have not
    /// been told about yet.
    lineadd: c_int,
}

/// The line being inserted, as `ml_append_int` was handed it.
#[derive(Clone, Copy)]
struct NewLine {
    /// `len` bytes of text, the last of which is the NUL standing for the
    /// line's newline.
    text: *mut c_char,
    len: colnr_T,
    /// [`ML_APPEND_MARK`] and [`ML_APPEND_NEW`].
    flags: c_int,
}

/// Where in the data block `ml_find_line` returned the new line goes.
#[derive(Clone, Copy)]
struct InsertAt {
    /// Index of the line being appended *after*, within the block. Negative
    /// means "in front of the block's first line", which is the `lnum == 0`
    /// case and the "insert at the front of the next block" case.
    db_idx: c_int,
    /// Number of index entries in the block before the insertion.
    line_count: c_int,
}

/// Insert `line` as a new line just after `lnum` (or as line 1, if `lnum` is
/// 0).
///
/// `len` is the length including the NUL, or 0 to measure it. `flags` are
/// [`ML_APPEND_MARK`] and [`ML_APPEND_NEW`].
///
/// # Safety
/// `buf` must point at a buffer, and `line` at `len` readable bytes (or at a
/// NUL-terminated string, if `len` is 0).
pub(crate) unsafe fn ml_append_int(
    buf: *mut buf_T,
    lnum: linenr_T,
    line: *mut c_char,
    len_arg: colnr_T,
    flags: c_int,
) -> c_int {
    unsafe {
        if lnum > (*buf).b_ml.ml_line_count || (*buf).b_ml.ml_mfp.is_null() {
            return FAIL; // lnum out of range
        }

        if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
            lowest_marked.set(lnum + 1);
        }

        // Space needed for the text, and then for its index entry too.
        let len = if len_arg == 0 {
            strlen(line) as colnr_T + 1
        } else {
            len_arg
        };
        let space_needed = len as int64_t + INDEX_SIZE as int64_t;

        let mfp = (*buf).b_ml.ml_mfp;

        // Find the data block holding the previous line. This also fills the
        // stack with the blocks from the root down, and releases any block
        // that was locked.
        let mut hp = ml_find_line(buf, if lnum == 0 { 1 } else { lnum }, ML_INSERT);
        if hp.is_null() {
            return FAIL;
        }
        (*buf).b_ml.ml_flags &= !ML_EMPTY;

        // Index of `lnum` within the block. Negative when the block found was
        // line one's rather than `lnum`'s, which is the `lnum == 0` case.
        let mut db_idx = if lnum == 0 {
            -1
        } else {
            lnum - (*buf).b_ml.ml_locked_low
        };
        // Number of index entries in the block before the insertion.
        let mut line_count = (*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low;
        let mut dp = (*hp).bh_data as *mut DataBlock;

        // If there is no room here, and this is the last line of the block,
        // and it is not the last line of the file, insert at the front of the
        // *next* block instead — that one may have room, and this keeps a
        // sequential insert from splitting a block per line.
        if ((*dp).db_free as int64_t) < space_needed
            && db_idx == line_count - 1
            && lnum < (*buf).b_ml.ml_line_count
        {
            // The line is not going into the block ml_find_line just charged
            // it to, so take it back off through ml_locked_lineadd.
            (*buf).b_ml.ml_locked_lineadd -= 1;
            (*buf).b_ml.ml_locked_high -= 1;
            hp = ml_find_line(buf, lnum + 1, ML_INSERT);
            if hp.is_null() {
                return FAIL;
            }
            db_idx = -1;
            line_count = (*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low;
            dp = (*hp).bh_data as *mut DataBlock;
        }

        if (*buf).b_prev_line_count == 0 {
            (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count;
        }
        (*buf).b_ml.ml_line_count += 1;

        let at = InsertAt { db_idx, line_count };
        let new = NewLine {
            text: line,
            len,
            flags,
        };
        if (*dp).db_free as int64_t >= space_needed {
            ml_insert_in_block(buf, dp, &at, &new, space_needed);
        } else {
            let mut split = ml_split_data_block(buf, hp, &at, lnum, &new, space_needed);
            if !ml_insert_pointer(buf, mfp, &mut split) {
                return FAIL;
            }
        }

        // The line was inserted below `lnum`.
        ml_updatechunk(buf, lnum + 1, len, ML_CHNK_ADDLINE);
        OK
    }
}

/// Put the new line into a data block that has room for it, shifting the
/// lines that follow (which live at *lower* offsets) down by `len`.
///
/// # Safety
/// `dp` must be the locked data block, with at least `space_needed` bytes free.
unsafe fn ml_insert_in_block(
    buf: *mut buf_T,
    dp: *mut DataBlock,
    at: &InsertAt,
    new: &NewLine,
    space_needed: int64_t,
) {
    unsafe {
        let InsertAt { db_idx, line_count } = *at;
        let NewLine {
            text: line,
            len,
            flags,
        } = *new;
        (*dp).db_txt_start = (*dp).db_txt_start.wrapping_sub(len as c_uint);
        (*dp).db_free = (*dp).db_free.wrapping_sub(space_needed as c_uint);
        (*dp).db_line_count += 1;

        if line_count > db_idx + 1 {
            // There are following lines: move their text to the front and
            // shift their index entries up one slot. `offset` is the start of
            // the previous line, which becomes the byte just past the new one.
            let offset = if db_idx < 0 {
                (*dp).db_txt_end as c_int
            } else {
                db_line_start(dp, db_idx) as c_int
            };
            let txt_start = (*dp).db_txt_start as isize;
            core::ptr::copy(
                (dp as *mut c_char).offset(txt_start + len as isize),
                (dp as *mut c_char).offset(txt_start),
                (offset as usize).wrapping_sub(((*dp).db_txt_start as usize) + len as usize),
            );
            let mut i = line_count - 1;
            while i > db_idx {
                *db_index(dp).offset((i + 1) as isize) =
                    (*db_index(dp).offset(i as isize)).wrapping_sub(len as c_uint);
                i -= 1;
            }
            *db_index(dp).offset((db_idx + 1) as isize) = (offset as colnr_T - len) as c_uint;
        } else {
            // Add at the end, which is the start of the text.
            *db_index(dp).offset((db_idx + 1) as isize) = (*dp).db_txt_start;
        }

        let slot = db_index(dp).offset((db_idx + 1) as isize);
        core::ptr::copy(
            line,
            (dp as *mut c_char).offset(*slot as isize),
            len as usize,
        );
        if flags & ML_APPEND_MARK != 0 {
            *slot |= DB_MARKED;
        }

        (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
        if flags & ML_APPEND_NEW == 0 {
            (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
        }
    }
}

/// There is no room: allocate a second data block and share the lines
/// between the two, then hand back what the pointer block above has to be
/// told.
///
/// The new block goes to the left of the existing one when the insert is at
/// the very front (`db_idx < 0`); otherwise to the right, taking the lines
/// that follow the insertion point with it. The new line itself goes in
/// whichever of the two has room after that — preferring the left, because
/// that is what makes inserting a run of lines at one place cheap.
///
/// # Safety
/// `hp` must be the locked data block, and `new.text` must hold `new.len`
/// bytes.
unsafe fn ml_split_data_block(
    buf: *mut buf_T,
    hp: *mut bhdr_T,
    at: &InsertAt,
    lnum: linenr_T,
    new: &NewLine,
    space_needed_arg: int64_t,
) -> SplitBlocks {
    unsafe {
        let InsertAt { db_idx, line_count } = *at;
        let NewLine {
            text: line,
            len,
            flags,
        } = *new;
        let dp = (*hp).bh_data as *mut DataBlock;
        let mfp = (*buf).b_ml.ml_mfp;
        let page_size = (*mfp).mf_page_size as int64_t;
        let mut space_needed = space_needed_arg;

        let mut data_moved = 0;
        let mut total_moved = 0;
        let lines_moved;
        let in_left;
        if db_idx < 0 {
            // Left block is new, right block is the existing one; nothing
            // moves, and space_needed does not change.
            lines_moved = 0;
            in_left = true;
        } else {
            lines_moved = line_count - db_idx - 1;
            if lines_moved == 0 {
                in_left = false; // put the new line in the right block
            } else {
                data_moved = (db_line_start(dp, db_idx).wrapping_sub((*dp).db_txt_start)) as c_int;
                total_moved = data_moved + lines_moved * INDEX_SIZE as c_int;
                if (*dp).db_free as int64_t + total_moved as int64_t >= space_needed {
                    in_left = true;
                    space_needed = total_moved as int64_t;
                } else {
                    in_left = false;
                    space_needed += total_moved as int64_t;
                }
            }
        }

        let page_count = (space_needed + HEADER_SIZE as int64_t + page_size - 1) / page_size;
        let hp_new = ml_new_data(mfp, flags & ML_APPEND_NEW != 0, page_count);

        let (hp_left, hp_right, mut line_count_left, mut line_count_right) = if db_idx < 0 {
            (hp_new, hp, 0, line_count)
        } else {
            (hp, hp_new, line_count, 0)
        };
        let dp_right = (*hp_right).bh_data as *mut DataBlock;
        let dp_left = (*hp_left).bh_data as *mut DataBlock;
        let bnum_left = (*hp_left).bh_bnum;
        let bnum_right = (*hp_right).bh_bnum;
        let page_count_left = (*hp_left).bh_page_count as c_int;
        let page_count_right = (*hp_right).bh_page_count as c_int;

        // The new line may go into the right/new block.
        if !in_left {
            (*dp_right).db_txt_start = (*dp_right).db_txt_start.wrapping_sub(len as c_uint);
            (*dp_right).db_free = (*dp_right)
                .db_free
                .wrapping_sub(len as c_uint + INDEX_SIZE as c_uint);
            let slot = db_index(dp_right);
            *slot = (*dp_right).db_txt_start;
            if flags & ML_APPEND_MARK != 0 {
                *slot |= DB_MARKED;
            }
            core::ptr::copy(
                line,
                (dp_right as *mut c_char).offset((*dp_right).db_txt_start as isize),
                len as usize,
            );
            line_count_right += 1;
        }

        // Lines after the insertion point move from the left/old block to the
        // right/new one.
        if lines_moved != 0 {
            (*dp_right).db_txt_start = (*dp_right).db_txt_start.wrapping_sub(data_moved as c_uint);
            (*dp_right).db_free = (*dp_right).db_free.wrapping_sub(total_moved as c_uint);
            core::ptr::copy(
                (dp_left as *mut c_char).offset((*dp_left).db_txt_start as isize),
                (dp_right as *mut c_char).offset((*dp_right).db_txt_start as isize),
                data_moved as usize,
            );
            // How far the text shifted, and so how much every index entry
            // that came with it has to shift.
            let offset = (*dp_right)
                .db_txt_start
                .wrapping_sub((*dp_left).db_txt_start) as c_int;
            (*dp_left).db_txt_start = (*dp_left).db_txt_start.wrapping_add(data_moved as c_uint);
            (*dp_left).db_free = (*dp_left).db_free.wrapping_add(total_moved as c_uint);

            let mut to = line_count_right;
            let mut from = db_idx + 1;
            while from < line_count_left {
                *db_index(dp_right).offset(to as isize) =
                    (*db_index(dp).offset(from as isize)).wrapping_add(offset as c_uint);
                from += 1;
                to += 1;
            }
            line_count_right += lines_moved;
            line_count_left -= lines_moved;
        }

        // The new line may go into the left (old or new) block.
        if in_left {
            (*dp_left).db_txt_start = (*dp_left).db_txt_start.wrapping_sub(len as c_uint);
            (*dp_left).db_free = (*dp_left)
                .db_free
                .wrapping_sub(len as c_uint + INDEX_SIZE as c_uint);
            let slot = db_index(dp_left).offset(line_count_left as isize);
            *slot = (*dp_left).db_txt_start;
            if flags & ML_APPEND_MARK != 0 {
                *slot |= DB_MARKED;
            }
            core::ptr::copy(
                line,
                (dp_left as *mut c_char).offset((*dp_left).db_txt_start as isize),
                len as usize,
            );
            line_count_left += 1;
        }

        let (lnum_left, lnum_right) = if db_idx < 0 {
            (lnum + 1, 0) // left block is new
        } else if in_left {
            (0, lnum + 2) // right block is new
        } else {
            (0, lnum + 1)
        };

        (*dp_left).db_line_count = line_count_left as c_long;
        (*dp_right).db_line_count = line_count_right as c_long;

        // Release the two data blocks. The new one already has a correct
        // block number; the old one (still in ml_locked) gets a positive one
        // if it changed and this is not a file being read in for the first
        // time.
        if lines_moved != 0 || in_left {
            (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
        }
        if flags & ML_APPEND_NEW == 0 && db_idx >= 0 && in_left {
            (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
        }
        mf_put(mfp, hp_new, true, false);

        // Flush the old data block. ml_locked_lineadd goes to zero because
        // the pointer blocks are about to be updated by hand.
        let lineadd = (*buf).b_ml.ml_locked_lineadd;
        (*buf).b_ml.ml_locked_lineadd = 0;
        ml_find_line(buf, 0, ML_FLUSH);

        SplitBlocks {
            bnum_left,
            bnum_right,
            line_count_left,
            line_count_right,
            page_count_left,
            page_count_right,
            lnum_left,
            lnum_right,
            lineadd,
        }
    }
}

/// Walk back up the stack replacing the entry that described the block that
/// was split with the two that describe its halves, splitting pointer blocks
/// on the way whenever one is full.
///
/// Returns false if a block could not be read, in which case `ml_append_int`
/// fails.
///
/// # Safety
/// `buf`'s stack must be the path ml_find_line left, and `split` must
/// describe two blocks that exist.
unsafe fn ml_insert_pointer(buf: *mut buf_T, mfp: *mut memfile_T, split: &mut SplitBlocks) -> bool {
    unsafe {
        let mut stack_idx = (*buf).b_ml.ml_stack_top - 1;
        while stack_idx >= 0 {
            // The stack can be reallocated by ml_add_stack, but nothing here
            // pushes onto it, so this stays valid for the iteration.
            let ip = (*buf).b_ml.ml_stack.offset(stack_idx as isize);
            let pb_idx = (*ip).ip_index;
            let mut hp = mf_get(mfp, (*ip).ip_bnum, 1);
            if hp.is_null() {
                return false;
            }
            let mut pp = (*hp).bh_data as *mut PointerBlock;
            if (*pp).pb_id as c_int != PTR_ID as c_int {
                iemsg(gettext(c"E317: Pointer block id wrong 3".as_ptr()));
                mf_put(mfp, hp, false, false);
                return false;
            }

            if ((*pp).pb_count as c_int) < (*pp).pb_count_max as c_int {
                ml_pointer_add_entry(buf, hp, pb_idx, split, stack_idx);
                return true;
            }

            // The pointer block is full: split it, and go round again to give
            // *its* parent the two halves.
            let hp_new =
                match ml_split_pointer_block(buf, mfp, &mut hp, &mut pp, ip, &mut stack_idx) {
                    Some(hp_new) => hp_new,
                    None => return false,
                };
            let pp_new = (*hp_new).bh_data as *mut PointerBlock;

            // Move the entries after the current one into the new block; if
            // there are none, the new entry itself goes there.
            let total_moved = (*pp).pb_count as c_int - pb_idx - 1;
            if total_moved != 0 {
                core::ptr::copy(
                    pb_entries(pp).offset((pb_idx + 1) as isize),
                    pb_entries(pp_new),
                    total_moved as usize,
                );
                (*pp_new).pb_count = total_moved as uint16_t;
                (*pp).pb_count = ((*pp).pb_count as c_int - (total_moved - 1)) as uint16_t;
                let right = pb_entries(pp).offset((pb_idx + 1) as isize);
                (*right).pe_bnum = split.bnum_right;
                (*right).pe_line_count = split.line_count_right;
                (*right).pe_page_count = split.page_count_right;
                if split.lnum_right != 0 {
                    (*right).pe_old_lnum = split.lnum_right;
                }
            } else {
                (*pp_new).pb_count = 1;
                let right = pb_entries(pp_new);
                (*right).pe_bnum = split.bnum_right;
                (*right).pe_line_count = split.line_count_right;
                (*right).pe_page_count = split.page_count_right;
                (*right).pe_old_lnum = split.lnum_right;
            }
            let left = pb_entries(pp).offset(pb_idx as isize);
            (*left).pe_bnum = split.bnum_left;
            (*left).pe_line_count = split.line_count_left;
            (*left).pe_page_count = split.page_count_left;
            if split.lnum_left != 0 {
                (*left).pe_old_lnum = split.lnum_left;
            }
            split.lnum_left = 0;
            split.lnum_right = 0;

            // The two pointer blocks are now the pair the level above has to
            // describe, so recount their lines.
            split.line_count_right = pb_line_total(pp_new);
            split.line_count_left = pb_line_total(pp);
            split.bnum_left = (*hp).bh_bnum;
            split.bnum_right = (*hp_new).bh_bnum;
            split.page_count_left = 1;
            split.page_count_right = 1;
            mf_put(mfp, hp, true, false);
            mf_put(mfp, hp_new, true, false);
            stack_idx -= 1;
        }

        // Fallen off the bottom of the stack.
        iemsg(gettext(c"E318: Updated too many blocks?".as_ptr()));
        (*buf).b_ml.ml_stack_top = 0; // invalidate the stack
        true
    }
}

/// Sum of the line counts of every entry in a pointer block.
///
/// # Safety
/// `pp` must point at a pointer block.
unsafe fn pb_line_total(pp: *mut PointerBlock) -> c_int {
    unsafe {
        let mut total = 0;
        for i in 0..(*pp).pb_count as c_int {
            total += (*pb_entries(pp).offset(i as isize)).pe_line_count;
        }
        total
    }
}

/// The pointer block has room: replace entry `pb_idx` with the two the split
/// produced, and truncate the stack here.
///
/// # Safety
/// `hp` must be the pointer block at `stack_idx`, with room for one more
/// entry.
unsafe fn ml_pointer_add_entry(
    buf: *mut buf_T,
    hp: *mut bhdr_T,
    pb_idx: c_int,
    split: &SplitBlocks,
    stack_idx: c_int,
) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        let pp = (*hp).bh_data as *mut PointerBlock;
        if pb_idx + 1 < (*pp).pb_count as c_int {
            core::ptr::copy(
                pb_entries(pp).offset((pb_idx + 1) as isize),
                pb_entries(pp).offset((pb_idx + 2) as isize),
                ((*pp).pb_count as c_int - pb_idx - 1) as usize,
            );
        }
        (*pp).pb_count = (*pp).pb_count.wrapping_add(1);

        let left = pb_entries(pp).offset(pb_idx as isize);
        (*left).pe_line_count = split.line_count_left;
        (*left).pe_bnum = split.bnum_left;
        (*left).pe_page_count = split.page_count_left;
        let right = pb_entries(pp).offset((pb_idx + 1) as isize);
        (*right).pe_line_count = split.line_count_right;
        (*right).pe_bnum = split.bnum_right;
        (*right).pe_page_count = split.page_count_right;

        if split.lnum_left != 0 {
            (*left).pe_old_lnum = split.lnum_left;
        }
        if split.lnum_right != 0 {
            (*right).pe_old_lnum = split.lnum_right;
        }

        mf_put(mfp, hp, true, false);
        (*buf).b_ml.ml_stack_top = stack_idx + 1; // truncate the stack

        if split.lineadd != 0 {
            (*buf).b_ml.ml_stack_top -= 1;
            // Fix the line count in the rest of the blocks on the stack, and
            // then the stack entry itself.
            ml_lineadd(buf, split.lineadd);
            (*(*buf)
                .b_ml
                .ml_stack
                .offset((*buf).b_ml.ml_stack_top as isize))
            .ip_high += split.lineadd;
            (*buf).b_ml.ml_stack_top += 1;
        }
    }
}

/// Allocate the pointer block that `*hp`'s entries will be shared with.
///
/// Block 1 is the root and cannot move, so when *it* fills the tree gains a
/// level instead: block 1's entries go into a fresh block, block 1 is left
/// pointing at that one alone, and the new block is what gets split. That can
/// only happen at the bottom of the stack, and `stack_idx` is bumped so the
/// loop comes back to block 1 afterwards.
///
/// Returns None if a block could not be allocated.
///
/// # Safety
/// `*hp`/`*pp` must be a full pointer block, `ip` its stack entry.
unsafe fn ml_split_pointer_block(
    buf: *mut buf_T,
    mfp: *mut memfile_T,
    hp: &mut *mut bhdr_T,
    pp: &mut *mut PointerBlock,
    ip: *mut infoptr_T,
    stack_idx: &mut c_int,
) -> Option<*mut bhdr_T> {
    unsafe {
        let page_size = (*mfp).mf_page_size as usize;
        loop {
            let hp_new = ml_new_ptr(mfp);
            if hp_new.is_null() {
                return None;
            }
            let pp_new = (*hp_new).bh_data as *mut PointerBlock;
            if (**hp).bh_bnum != 1 {
                return Some(hp_new);
            }

            core::ptr::copy_nonoverlapping((*pp).cast::<u8>(), pp_new.cast::<u8>(), page_size);
            (**pp).pb_count = 1;
            let root = pb_entries(*pp);
            (*root).pe_bnum = (*hp_new).bh_bnum;
            (*root).pe_line_count = (*buf).b_ml.ml_line_count;
            (*root).pe_old_lnum = 1;
            (*root).pe_page_count = 1;
            mf_put(mfp, *hp, true, false); // release block 1
            *hp = hp_new; // the new block is the one to split
            *pp = pp_new;
            (*ip).ip_index = 0;
            *stack_idx += 1; // do block 1 again later
        }
    }
}

/// Delete line `lnum`.
///
/// `flags` is [`ML_DEL_MESSAGE`], which asks for "--No lines in buffer--" if
/// the buffer ends up empty.
///
/// # Safety
/// `buf` must point at a buffer holding line `lnum`.
pub(crate) unsafe fn ml_delete_int(buf: *mut buf_T, lnum: linenr_T, flags: c_int) -> c_int {
    unsafe {
        if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
            lowest_marked.set(lowest_marked.get() - 1);
        }

        // If the file becomes empty the last line is replaced by an empty one.
        if (*buf).b_ml.ml_line_count == 1 {
            if flags & ML_DEL_MESSAGE != 0 {
                set_keep_msg(gettext(no_lines_msg.ptr().cast::<c_char>()), 0);
            }
            let i = ml_replace_buf(buf, 1, c"".as_ptr().cast_mut(), true, false);
            (*buf).b_ml.ml_flags |= ML_EMPTY;
            return i;
        }

        // Find the data block holding the line. This also fills the stack
        // with the blocks from the root down, and releases any locked block.
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return FAIL;
        }
        let hp = ml_find_line(buf, lnum, ML_DELETE);
        if hp.is_null() {
            return FAIL;
        }

        let dp = (*hp).bh_data as *mut DataBlock;
        // Number of index entries in the block before the delete. The +2 (not
        // +1) is because ML_DELETE already took one off ml_locked_high.
        let count = (*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low + 2;
        let idx = lnum - (*buf).b_ml.ml_locked_low;

        if (*buf).b_prev_line_count == 0 {
            (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count;
        }
        (*buf).b_ml.ml_line_count -= 1;

        let line_start = db_line_start(dp, idx) as c_int;
        let line_size = if idx == 0 {
            // First line in the block, so its text is at the end.
            (*dp).db_txt_end.wrapping_sub(line_start as c_uint) as c_int
        } else {
            db_line_start(dp, idx - 1).wrapping_sub(line_start as c_uint) as c_int
        };

        // A line always holds its terminating NL internally, as a NUL, even
        // with 'noeol'.
        debug_assert!(line_size >= 1);
        ml_add_deleted_len_buf(
            buf,
            (dp as *mut c_char).offset(line_start as isize),
            (line_size - 1) as ssize_t,
        );

        if count == 1 {
            // The block held only this line, so it goes away, and with it the
            // entry pointing at it — and if that empties its pointer block,
            // that one too, up to the root if need be.
            if !ml_free_data_block(buf, mfp, hp) {
                return FAIL;
            }
        } else {
            // Move the text of the following lines forward over the deleted
            // one, and their index entries back over its entry.
            let text_start = (*dp).db_txt_start as isize;
            core::ptr::copy(
                (dp as *mut c_char).offset(text_start),
                (dp as *mut c_char).offset(text_start + line_size as isize),
                (line_start - text_start as c_int) as usize,
            );
            for i in idx..count - 1 {
                *db_index(dp).offset(i as isize) =
                    (*db_index(dp).offset((i + 1) as isize)).wrapping_add(line_size as c_uint);
            }

            (*dp).db_free = (*dp)
                .db_free
                .wrapping_add(line_size as c_uint + INDEX_SIZE as c_uint);
            (*dp).db_txt_start = (*dp).db_txt_start.wrapping_add(line_size as c_uint);
            (*dp).db_line_count -= 1;

            // Mark the block dirty, and make sure it reaches the file so a
            // recovery sees the delete.
            (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
        }

        ml_updatechunk(buf, lnum, line_size, ML_CHNK_DELLINE);
        OK
    }
}

/// The data block `hp` has emptied: free it and unhook it from the pointer
/// block above, recursing upwards while that empties pointer blocks too.
///
/// Returns false if a block could not be read.
///
/// # Safety
/// `hp` must be the locked data block, and `buf`'s stack the path to it.
unsafe fn ml_free_data_block(buf: *mut buf_T, mfp: *mut memfile_T, hp: *mut bhdr_T) -> bool {
    unsafe {
        mf_free(mfp, hp); // free the data block
        (*buf).b_ml.ml_locked = core::ptr::null_mut();

        let mut stack_idx = (*buf).b_ml.ml_stack_top - 1;
        while stack_idx >= 0 {
            (*buf).b_ml.ml_stack_top = 0; // the stack is invalid if this fails
            let ip = (*buf).b_ml.ml_stack.offset(stack_idx as isize);
            let idx = (*ip).ip_index;
            let hp = mf_get(mfp, (*ip).ip_bnum, 1);
            if hp.is_null() {
                return false;
            }
            let pp = (*hp).bh_data as *mut PointerBlock;
            if (*pp).pb_id as c_int != PTR_ID as c_int {
                iemsg(gettext(c"E317: Pointer block id wrong 4".as_ptr()));
                mf_put(mfp, hp, false, false);
                return false;
            }
            (*pp).pb_count = (*pp).pb_count.wrapping_sub(1);
            let count = (*pp).pb_count as c_int;
            if count == 0 {
                // This pointer block is empty now too.
                mf_free(mfp, hp);
                stack_idx -= 1;
                continue;
            }

            if count != idx {
                // Move the entries after the deleted one down.
                core::ptr::copy(
                    pb_entries(pp).offset((idx + 1) as isize),
                    pb_entries(pp).offset(idx as isize),
                    (count - idx) as usize,
                );
            }
            mf_put(mfp, hp, true, false);

            (*buf).b_ml.ml_stack_top = stack_idx; // truncate the stack
            // Fix the line count in the rest of the blocks on the stack.
            if (*buf).b_ml.ml_locked_lineadd != 0 {
                ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
                (*(*buf)
                    .b_ml
                    .ml_stack
                    .offset((*buf).b_ml.ml_stack_top as isize))
                .ip_high += (*buf).b_ml.ml_locked_lineadd;
            }
            (*buf).b_ml.ml_stack_top += 1;
            break;
        }
        true
    }
}
