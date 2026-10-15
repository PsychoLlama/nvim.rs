//! Inserting and deleting a line: the two operations that change
//! the tree's shape.
//!
//! Everything else in memline reads the tree or rewrites a line in place.
//! These two are where a data block runs out of room and has to be split (and
//! then, possibly, the pointer block above it, all the way to the root), and
//! where a data block empties and has to be unhooked again.
//!
//! Blocks are held by [`BlockId`] across every call that can reach the
//! memfile — a split allocates while the block it splits is locked — and
//! read through views made after the last such call.

#![forbid(unsafe_code)]

use core::ffi::{c_int, c_long, c_uint};

use super::*;
use crate::memfile::BlockId;
use crate::types::Failed;

/// How a data block split came out, in the terms the pointer block above it
/// has to be told: two blocks side by side, each with a block number, a line
/// count and a page count.
///
/// The pointer-block walk mutates this as it climbs — once a pointer block
/// itself splits, *it* becomes the left/right pair its own parent must
/// describe.
struct SplitBlocks {
    bnum_left: BlockNr,
    bnum_right: BlockNr,
    line_count_left: c_int,
    line_count_right: c_int,
    page_count_left: c_int,
    page_count_right: c_int,
    /// `pe_old_lnum`: which line the block held first when the file was read,
    /// which is all `:recover` has to order blocks by. Zero means "leave the
    /// entry's alone".
    lnum_left: LineNr,
    lnum_right: LineNr,
    /// Lines added to the old block that the pointer blocks above it have not
    /// been told about yet.
    lineadd: c_int,
}

/// The line being inserted, as `ml_append_int` was handed it.
struct NewLine<'a> {
    /// The text, the last byte of which is the NUL standing for the line's
    /// newline.
    text: &'a [u8],
    /// [`ML_APPEND_MARK`] and [`ML_APPEND_NEW`].
    flags: c_int,
}

impl NewLine<'_> {
    fn len(&self) -> c_uint {
        c_uint::try_from(self.text.len()).expect("a line shorter than 4 GiB")
    }

    fn marked(&self) -> bool {
        self.flags & ML_APPEND_MARK != 0
    }
}

/// Where in the data block `ml_find_line` returned the new line goes.
struct InsertAt {
    /// Index of the line being appended *after*, within the block. Negative
    /// means "in front of the block's first line", which is the `lnum == 0`
    /// case and the "insert at the front of the next block" case.
    db_idx: c_int,
    /// Number of index entries in the block before the insertion.
    line_count: c_int,
}

/// Insert `line` as a new line just after `lnum` (or as line 1, if `lnum` is
/// 0). `line` holds the text and the NUL standing for its line break.
///
/// `flags` are [`ML_APPEND_MARK`] and [`ML_APPEND_NEW`].
pub(crate) fn ml_append_int(
    mut buffer: Buf,
    lnum: LineNr,
    line: &[u8],
    flags: c_int,
) -> Result<(), Failed> {
    if lnum > buffer.b_ml.ml_line_count || !buffer.b_ml.is_open() {
        return Err(Failed); // lnum out of range
    }

    if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
        lowest_marked.set(lnum + 1);
    }

    let new = NewLine { text: line, flags };
    // Space needed for the text, and then for its index entry too.
    let space_needed = int64_t::from(new.len()) + INDEX_SIZE as int64_t;

    // Find the data block holding the previous line. This also fills the
    // stack with the blocks from the root down, and releases any block
    // that was locked.
    let Some(mut id) = ml_find_line(buffer, if lnum == 0 { 1 } else { lnum }, ML_INSERT) else {
        return Err(Failed);
    };
    buffer.b_ml.ml_flags.clear(MlFlags::EMPTY);

    // Index of `lnum` within the block. Negative when the block found was
    // line one's rather than `lnum`'s, which is the `lnum == 0` case.
    let mut db_idx = if lnum == 0 {
        -1
    } else {
        lnum - buffer.b_ml.locked_low()
    };
    // Number of index entries in the block before the insertion.
    let mut line_count = buffer.b_ml.locked_high() - buffer.b_ml.locked_low();
    let mut free = DataBlock(buffer.b_ml.page(id)).free();

    // If there is no room here, and this is the last line of the block,
    // and it is not the last line of the file, insert at the front of the
    // *next* block instead — that one may have room, and this keeps a
    // sequential insert from splitting a block per line.
    if int64_t::from(free) < space_needed
        && db_idx == line_count - 1
        && lnum < buffer.b_ml.ml_line_count
    {
        // The line is not going into the block ml_find_line just charged
        // it to, so take it back off through the block's `lineadd`.
        buffer.b_ml.shift_locked(-1);
        let Some(next) = ml_find_line(buffer, lnum + 1, ML_INSERT) else {
            return Err(Failed);
        };
        id = next;
        db_idx = -1;
        line_count = buffer.b_ml.locked_high() - buffer.b_ml.locked_low();
        free = DataBlock(buffer.b_ml.page(id)).free();
    }

    if buffer.b_prev_line_count == 0 {
        buffer.b_prev_line_count = buffer.b_ml.ml_line_count;
    }
    buffer.b_ml.ml_line_count += 1;

    let at = InsertAt { db_idx, line_count };
    if int64_t::from(free) >= space_needed {
        ml_insert_in_block(buffer, id, at, &new, space_needed);
    } else {
        let mut split = ml_split_data_block(buffer, id, at, lnum, &new, space_needed);
        if !ml_insert_pointer(buffer, &mut split) {
            return Err(Failed);
        }
    }

    // The line was inserted below `lnum`.
    buffer.update_line_chunks(lnum + 1, new.len() as c_int, ML_CHNK_ADDLINE);
    Ok(())
}

/// Put the new line into data block `id`, which has room for it, shifting
/// the lines that follow (which live at *lower* offsets) down by its length.
fn ml_insert_in_block(mut b: Buf, id: BlockId, at: InsertAt, new: &NewLine, space_needed: int64_t) {
    let InsertAt { db_idx, line_count } = at;
    let len = new.len();
    let mut dp = DataBlock(b.b_ml.page_mut(id));
    let txt_start = dp.txt_start().wrapping_sub(len);
    dp.set_txt_start(txt_start);
    dp.set_free(dp.free().wrapping_sub(space_needed as c_uint));
    dp.set_line_count(dp.line_count() + 1);

    // Where the new line's index entry goes.
    let slot = (db_idx + 1) as usize;
    if line_count > db_idx + 1 {
        // There are following lines: move their text to the front and
        // shift their index entries up one slot. `offset` is the start of
        // the previous line, which becomes the byte just past the new one.
        let offset = if db_idx < 0 {
            dp.txt_end()
        } else {
            dp.line_start(db_idx as usize)
        };
        let start = txt_start as usize;
        let n = (offset as usize).wrapping_sub(start + len as usize);
        dp.move_bytes(start + len as usize, start, n);
        let following = (line_count - db_idx - 1) as usize;
        dp.move_index(slot, slot + 1, following, len.wrapping_neg());
        dp.set_index(slot, offset.wrapping_sub(len));
    } else {
        // Add at the end, which is the start of the text.
        dp.set_index(slot, txt_start);
    }

    let to = dp.index(slot) as usize;
    dp.write(to, new.text);
    if new.marked() {
        dp.set_marked(slot, true);
    }

    b.b_ml.locked_is_dirty();
    if new.flags & ML_APPEND_NEW == 0 {
        b.b_ml.locked_has_moved();
    }
}

/// Put `new` into the data block `dp` as its line `slot`, at the front of
/// its text, which has room for it.
fn add_new_line(dp: &mut DataBlock<&mut [u8]>, slot: usize, new: &NewLine) {
    let start = dp.txt_start().wrapping_sub(new.len());
    dp.set_txt_start(start);
    let room = dp.free().wrapping_sub(new.len() + INDEX_SIZE as c_uint);
    dp.set_free(room);
    dp.set_index(slot, start);
    if new.marked() {
        dp.set_marked(slot, true);
    }
    dp.write(start as usize, new.text);
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
/// `id` is the locked data block, and stays locked across the allocation
/// of the new one: the memfile neither moves nor drops a locked block, and
/// the handle is what is held, not the bytes.
fn ml_split_data_block(
    mut buffer: Buf,
    id: BlockId,
    at: InsertAt,
    lnum: LineNr,
    new: &NewLine,
    space_needed_arg: int64_t,
) -> SplitBlocks {
    let InsertAt { db_idx, line_count } = at;
    let page_size = int64_t::from(buffer.b_ml.mfp().mf_page_size);
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
            let dp = DataBlock(buffer.b_ml.page(id));
            data_moved = dp.line_start(db_idx as usize).wrapping_sub(dp.txt_start()) as c_int;
            total_moved = data_moved + lines_moved * INDEX_SIZE as c_int;
            if int64_t::from(dp.free()) + int64_t::from(total_moved) >= space_needed {
                in_left = true;
                space_needed = int64_t::from(total_moved);
            } else {
                in_left = false;
                space_needed += int64_t::from(total_moved);
            }
        }
    }

    let page_count = (space_needed + HEADER_SIZE as int64_t + page_size - 1) / page_size;
    let negative = new.flags & ML_APPEND_NEW != 0;
    let id_new = ml_new_data(buffer.b_ml.mfp(), negative, page_count);

    let (id_left, id_right, mut line_count_left, mut line_count_right) = if db_idx < 0 {
        (id_new, id, 0, line_count)
    } else {
        (id, id_new, line_count, 0)
    };
    let mfp = buffer.b_ml.mfp();
    let (bnum_left, bnum_right) = (mfp.block_nr(id_left), mfp.block_nr(id_right));
    let page_count_left = mfp.page_count(id_left) as c_int;
    let page_count_right = mfp.page_count(id_right) as c_int;

    {
        let (left, right) = mfp.pages_mut(id_left, id_right);
        let (mut dp_left, mut dp_right) = (DataBlock(left), DataBlock(right));

        // The new line may go into the right/new block.
        if !in_left {
            add_new_line(&mut dp_right, 0, new);
            line_count_right += 1;
        }

        // Lines after the insertion point move from the left/old block to
        // the right/new one.
        if lines_moved != 0 {
            let room = dp_right.txt_start().wrapping_sub(data_moved as c_uint);
            dp_right.set_txt_start(room);
            dp_right.set_free(dp_right.free().wrapping_sub(total_moved as c_uint));
            let (from, to) = (dp_left.txt_start() as usize, dp_right.txt_start() as usize);
            let n = data_moved as usize;
            dp_right.raw_mut()[to..to + n].copy_from_slice(&dp_left.raw_mut()[from..from + n]);
            // How far the text shifted, and so how much every index entry
            // that came with it has to shift.
            let offset = dp_right.txt_start().wrapping_sub(dp_left.txt_start());
            dp_left.set_txt_start(dp_left.txt_start().wrapping_add(data_moved as c_uint));
            dp_left.set_free(dp_left.free().wrapping_add(total_moved as c_uint));

            let (from, to) = ((db_idx + 1) as usize, line_count_right as usize);
            dp_right.copy_index_from(&dp_left, from, to, lines_moved as usize, offset);
            line_count_right += lines_moved;
            line_count_left -= lines_moved;
        }

        // The new line may go into the left (old or new) block.
        if in_left {
            add_new_line(&mut dp_left, line_count_left as usize, new);
            line_count_left += 1;
        }

        dp_left.set_line_count(c_long::from(line_count_left));
        dp_right.set_line_count(c_long::from(line_count_right));
    }

    let (lnum_left, lnum_right) = if db_idx < 0 {
        (lnum + 1, 0) // left block is new
    } else if in_left {
        (0, lnum + 2) // right block is new
    } else {
        (0, lnum + 1)
    };

    // Release the two data blocks. The new one already has a correct
    // block number; the old one (still locked) gets a positive one
    // if it changed and this is not a file being read in for the first
    // time.
    if lines_moved != 0 || in_left {
        buffer.b_ml.locked_is_dirty();
    }
    if new.flags & ML_APPEND_NEW == 0 && db_idx >= 0 && in_left {
        buffer.b_ml.locked_has_moved();
    }
    buffer.b_ml.put(id_new, true);

    // Flush the old data block. The lines it owes the pointer blocks
    // above it go with it, because they are about to be paid by hand.
    let lineadd = buffer.b_ml.take_locked_lineadd();
    let _ = ml_find_line(buffer, 0, ML_FLUSH);

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

/// Walk back up the stack replacing the entry that described the block that
/// was split with the two that describe its halves, splitting pointer blocks
/// on the way whenever one is full.
///
/// Returns false if a block could not be read, in which case `ml_append_int`
/// fails.
fn ml_insert_pointer(mut buffer: Buf, split: &mut SplitBlocks) -> bool {
    let mut stack_idx = buffer.b_ml.stack_len() as c_int - 1;
    while stack_idx >= 0 {
        let ip = buffer.b_ml.stack_at(stack_idx as usize);
        let pb_idx = ip.ip_index;
        let Some(mut id) = buffer.b_ml.mfp().get(ip.ip_bnum, 1) else {
            return false;
        };
        let pp = PointerBlock(buffer.b_ml.page(id));
        if !pp.is_pointer() {
            iemsg(gettext(c"E317: Pointer block id wrong 3"));
            buffer.b_ml.put(id, false);
            return false;
        }

        if pp.count() < pp.count_max() {
            ml_pointer_add_entry(buffer, id, pb_idx as usize, split, stack_idx as usize);
            return true;
        }

        // The pointer block is full: split it, and go round again to give
        // *its* parent the two halves.
        let id_new = ml_split_pointer_block(buffer, &mut id, &mut stack_idx);

        // Move the entries after the current one into the new block; if
        // there are none, the new entry itself goes there.
        let pb_idx = pb_idx as usize;
        let mfp = buffer.b_ml.mfp();
        let (page, page_new) = mfp.pages_mut(id, id_new);
        let (mut pp, mut pp_new) = (PointerBlock(page), PointerBlock(page_new));
        let total_moved = usize::from(pp.count()) - pb_idx - 1;
        if total_moved != 0 {
            pp_new.write_entries(0, pp.entry_bytes(pb_idx + 1, total_moved));
            pp_new.set_count(total_moved as u16);
            pp.set_count(pp.count() - (total_moved as u16 - 1));
            let (bnum, lines, pages) = (
                split.bnum_right,
                split.line_count_right,
                split.page_count_right,
            );
            pp.set_child(pb_idx + 1, bnum, lines, pages, split.lnum_right);
        } else {
            pp_new.set_count(1);
            pp_new.set_child(
                0,
                split.bnum_right,
                split.line_count_right,
                split.page_count_right,
                0,
            );
            pp_new.set_old_lnum(0, split.lnum_right);
        }
        let (bnum, lines, pages) = (
            split.bnum_left,
            split.line_count_left,
            split.page_count_left,
        );
        pp.set_child(pb_idx, bnum, lines, pages, split.lnum_left);
        split.lnum_left = 0;
        split.lnum_right = 0;

        // The two pointer blocks are now the pair the level above has to
        // describe, so recount their lines.
        split.line_count_right = pp_new.line_total();
        split.line_count_left = pp.line_total();
        split.bnum_left = mfp.block_nr(id);
        split.bnum_right = mfp.block_nr(id_new);
        split.page_count_left = 1;
        split.page_count_right = 1;
        mfp.put(id, true, false);
        mfp.put(id_new, true, false);
        stack_idx -= 1;
    }

    // Fallen off the bottom of the stack.
    iemsg(gettext(c"E318: Updated too many blocks?"));
    buffer.b_ml.stack_clear(); // invalidate the stack
    true
}

/// The pointer block `id` has room: replace entry `pb_idx` with the two the
/// split produced, and truncate the stack here.
fn ml_pointer_add_entry(
    mut buffer: Buf,
    id: BlockId,
    pb_idx: usize,
    split: &SplitBlocks,
    stack_idx: usize,
) {
    let mut pp = PointerBlock(buffer.b_ml.page_mut(id));
    let count = usize::from(pp.count());
    if pb_idx + 1 < count {
        pp.move_entries(pb_idx + 1, pb_idx + 2, count - pb_idx - 1);
    }
    pp.set_count(pp.count().wrapping_add(1));

    let (bnum, lines, pages) = (
        split.bnum_left,
        split.line_count_left,
        split.page_count_left,
    );
    pp.set_child(pb_idx, bnum, lines, pages, split.lnum_left);
    let (bnum, lines, pages) = (
        split.bnum_right,
        split.line_count_right,
        split.page_count_right,
    );
    pp.set_child(pb_idx + 1, bnum, lines, pages, split.lnum_right);

    buffer.b_ml.put(id, true);
    buffer.b_ml.stack_truncate(stack_idx + 1); // truncate the stack

    if split.lineadd != 0 {
        // Fix the line count in the rest of the blocks on the stack, and
        // then the stack entry itself -- which stays on the stack, so
        // the walk has to stop below it.
        ml_lineadd_depth(buffer, split.lineadd, stack_idx);
        buffer.b_ml.stack_add_high(stack_idx, split.lineadd);
    }
}

/// Allocate the pointer block that `*id`'s entries will be shared with.
///
/// Block 1 is the root and cannot move, so when *it* fills the tree gains a
/// level instead: block 1's entries go into a fresh block, block 1 is left
/// pointing at that one alone, and the new block is what gets split. That can
/// only happen at the bottom of the stack, and `stack_idx` is bumped so the
/// loop comes back to block 1 afterwards.
fn ml_split_pointer_block(mut buffer: Buf, id: &mut BlockId, stack_idx: &mut c_int) -> BlockId {
    // Where `*id`'s own stack entry is; `stack_idx` moves below.
    let ip = *stack_idx as usize;
    loop {
        let id_new = ml_new_ptr(buffer.b_ml.mfp());
        let mfp = buffer.b_ml.mfp();
        if mfp.block_nr(*id) != 1 {
            return id_new;
        }

        // Both blocks are whole pages of the same size, and the new one
        // cannot be block 1.
        let bnum_new = mfp.block_nr(id_new);
        let (root, page_new) = mfp.pages_mut(*id, id_new);
        page_new.copy_from_slice(root);
        let mut root = PointerBlock(root);
        root.set_count(1);
        // Block 1 now holds exactly one entry, and this is it.
        let line_count = buffer.b_ml.ml_line_count;
        let mfp = buffer.b_ml.mfp();
        let mut root = PointerBlock(mfp.page_mut(*id));
        root.set_bnum(0, bnum_new);
        root.set_line_count(0, line_count);
        root.set_old_lnum(0, 1);
        root.set_page_count(0, 1);
        mfp.put(*id, true, false); // release block 1
        *id = id_new; // the new block is the one to split
        buffer.b_ml.stack_set_index(ip, 0);
        *stack_idx += 1; // do block 1 again later
    }
}

/// Delete line `lnum`.
///
/// `flags` is [`ML_DEL_MESSAGE`], which asks for "--No lines in buffer--" if
/// the buffer ends up empty.
pub(crate) fn ml_delete_int(mut buffer: Buf, lnum: LineNr, flags: c_int) -> Result<(), Failed> {
    if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
        lowest_marked.set(lowest_marked.get() - 1);
    }

    // If the file becomes empty the last line is replaced by an empty one.
    if buffer.b_ml.ml_line_count == 1 {
        if flags & ML_DEL_MESSAGE != 0 {
            set_keep_msg(Some(gettext(no_lines_msg)), 0);
        }
        let i = ml_replace_buf_text(buffer, 1, b"");
        buffer.b_ml.ml_flags |= MlFlags::EMPTY;
        return i;
    }

    // Find the data block holding the line. This also fills the stack
    // with the blocks from the root down, and releases any locked block.
    if !buffer.b_ml.is_open() {
        return Err(Failed);
    }
    let Some(id) = ml_find_line(buffer, lnum, ML_DELETE) else {
        return Err(Failed);
    };

    // Number of index entries in the block before the delete. The +2 (not
    // +1) is because ML_DELETE already took one off the block's `high`.
    let count = (buffer.b_ml.locked_high() - buffer.b_ml.locked_low() + 2) as usize;
    let idx = (lnum - buffer.b_ml.locked_low()) as usize;

    if buffer.b_prev_line_count == 0 {
        buffer.b_prev_line_count = buffer.b_ml.ml_line_count;
    }
    buffer.b_ml.ml_line_count -= 1;

    let dp = DataBlock(buffer.b_ml.page(id));
    let line_start = dp.line_start(idx);
    // The text runs to where the line before it in the block starts, or
    // to the end of the block for its first line.
    let line_size = dp.line_end(idx).wrapping_sub(line_start) as c_int;

    // A line always holds its terminating NL internally, as a NUL, even
    // with 'noeol'.
    debug_assert!(line_size >= 1);
    let text = dp.text(
        line_start as usize,
        (line_start as usize + line_size as usize) - 1,
    );
    ml_add_deleted_len_buf(buffer, text, text.len());

    if count == 1 {
        // The block held only this line, so it goes away, and with it the
        // entry pointing at it — and if that empties its pointer block,
        // that one too, up to the root if need be.
        if !ml_free_data_block(buffer, id) {
            return Err(Failed);
        }
    } else {
        // Move the text of the following lines forward over the deleted
        // one, and their index entries back over its entry.
        let mut dp = DataBlock(buffer.b_ml.page_mut(id));
        let text_start = dp.txt_start();
        let size = line_size as c_uint;
        dp.move_bytes(
            text_start as usize,
            (text_start + size) as usize,
            (line_start - text_start) as usize,
        );
        dp.move_index(idx + 1, idx, count - 1 - idx, size);

        dp.set_free(dp.free().wrapping_add(size + INDEX_SIZE as c_uint));
        dp.set_txt_start(text_start.wrapping_add(size));
        dp.set_line_count(dp.line_count() - 1);

        // Mark the block dirty, and make sure it reaches the file so a
        // recovery sees the delete.
        buffer.b_ml.locked_has_moved();
    }

    buffer.update_line_chunks(lnum, line_size, ML_CHNK_DELLINE);
    Ok(())
}

/// The data block `id` has emptied: free it and unhook it from the pointer
/// block above, recursing upwards while that empties pointer blocks too.
///
/// Returns false if a block could not be read.
fn ml_free_data_block(mut buffer: Buf, id: BlockId) -> bool {
    // Free the data block; the lines it owes the pointer blocks above it
    // survive it, and are paid below.
    buffer.b_ml.mfp().free_block(id);
    let locked_lineadd = buffer.b_ml.forget_locked();

    // The stack is invalid until one of the pointer blocks on it survives
    // the unhooking; upstream said so by dropping `ml_stack_top` to zero
    // on the way in and putting it back on the way out, which a `Vec`
    // cannot do without losing the entries it is still reading. The
    // length is only cut once the answer is known.
    let mut stack_idx = buffer.b_ml.stack_len();
    while stack_idx > 0 {
        stack_idx -= 1;
        let ip = buffer.b_ml.stack_at(stack_idx);
        let idx = ip.ip_index as usize;
        let Some(id) = buffer.b_ml.mfp().get(ip.ip_bnum, 1) else {
            buffer.b_ml.stack_clear();
            return false;
        };
        let mut pp = PointerBlock(buffer.b_ml.page_mut(id));
        if !pp.is_pointer() {
            iemsg(gettext(c"E317: Pointer block id wrong 4"));
            buffer.b_ml.put(id, false);
            buffer.b_ml.stack_clear();
            return false;
        }
        let count = pp.count().wrapping_sub(1);
        pp.set_count(count);
        let count = usize::from(count);
        if count == 0 {
            // This pointer block is empty now too.
            buffer.b_ml.mfp().free_block(id);
            continue;
        }

        if count != idx {
            // Move the entries after the deleted one down.
            pp.move_entries(idx + 1, idx, count - idx);
        }
        buffer.b_ml.put(id, true);

        // This block stays on the stack and everything below it is gone.
        buffer.b_ml.stack_truncate(stack_idx + 1);
        // Fix the line count in the rest of the blocks on the stack --
        // below this one, which is corrected by hand right after.
        if locked_lineadd != 0 {
            ml_lineadd_depth(buffer, locked_lineadd, stack_idx);
            buffer.b_ml.stack_add_high(stack_idx, locked_lineadd);
        }
        return true;
    }
    // Every pointer block on the way up emptied as well.
    buffer.b_ml.stack_clear();
    true
}
