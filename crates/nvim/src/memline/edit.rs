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
use crate::types::{Failed, MemLine};

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
    /// `text.len()`, measured once.
    len: c_uint,
}

impl<'a> NewLine<'a> {
    fn new(text: &'a [u8], flags: c_int) -> Self {
        let len = c_uint::try_from(text.len()).expect("a line shorter than 4 GiB");
        NewLine { text, flags, len }
    }

    fn len(&self) -> c_uint {
        self.len
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

    let new = NewLine::new(line, flags);
    // Space needed for the text, and then for its index entry too.
    let space_needed = int64_t::from(new.len()) + INDEX_SIZE as int64_t;

    if !append_to_locked(&mut buffer.b_ml, lnum, &new, space_needed) {
        return append_by_walk(buffer, lnum, &new, space_needed);
    }
    buffer.b_ml.ml_flags.clear(MlFlags::EMPTY);
    count_new_line(buffer);
    // The line was inserted below `lnum`.
    buffer.update_line_chunks(lnum + 1, new.len() as c_int, ML_CHNK_ADDLINE);
    Ok(())
}

/// The buffer holds one more line.
fn count_new_line(mut buffer: Buf) {
    if buffer.b_prev_line_count == 0 {
        buffer.b_prev_line_count = buffer.b_ml.ml_line_count;
    }
    buffer.b_ml.ml_line_count += 1;
}

/// The append nearly every line of a file read makes: into the block that
/// is already locked, which has room for it. This is `ml_find_line`'s
/// locked-block answer and the insert without a split in one, with the
/// page looked up once. Answers false, having changed nothing, when the
/// walk is needed.
#[inline]
fn append_to_locked(ml: &mut MemLine, lnum: LineNr, new: &NewLine, space_needed: int64_t) -> bool {
    let (Some(locked), Some(memfile)) = (ml.ml_locked.as_mut(), ml.ml_mfp.as_deref_mut()) else {
        return false;
    };
    // The line `ml_find_line` would be asked for.
    let wanted = if lnum == 0 { 1 } else { lnum };
    if wanted < locked.low || wanted > locked.high {
        return false;
    }
    let mut data = DataBlock(memfile.page_mut(locked.id));
    let header = data.header();
    if int64_t::from(header.free) < space_needed {
        return false;
    }
    let at = InsertAt {
        db_idx: if lnum == 0 { -1 } else { lnum - locked.low },
        line_count: locked.high + 1 - locked.low,
    };
    insert_line(&mut data, header, at, new, space_needed);
    // What `ml_find_line`'s answer and the insert tell the locked block.
    locked.lineadd += 1;
    locked.high += 1;
    locked.dirty = true;
    if new.flags & ML_APPEND_NEW == 0 {
        locked.moved = true;
    }
    true
}

/// [`ml_append_int`] the long way: find the block through the tree, and
/// split it when it has no room. Out of line, so that the common case stays
/// a short function.
#[cold]
#[inline(never)]
fn append_by_walk(
    mut buffer: Buf,
    lnum: LineNr,
    new: &NewLine,
    space_needed: int64_t,
) -> Result<(), Failed> {
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

    count_new_line(buffer);

    let at = InsertAt { db_idx, line_count };
    if int64_t::from(free) >= space_needed {
        let mut data = DataBlock(buffer.b_ml.page_mut(id));
        let header = data.header();
        insert_line(&mut data, header, at, new, space_needed);
        buffer.b_ml.locked_is_dirty();
        if new.flags & ML_APPEND_NEW == 0 {
            buffer.b_ml.locked_has_moved();
        }
    } else {
        let mut split = ml_split_data_block(buffer, id, at, lnum, new, space_needed);
        if !ml_insert_pointer(buffer, &mut split) {
            return Err(Failed);
        }
    }

    // The line was inserted below `lnum`.
    buffer.update_line_chunks(lnum + 1, new.len() as c_int, ML_CHNK_ADDLINE);
    Ok(())
}

/// Put the new line into the data block `data`, whose header is `header`
/// and which has room for it, shifting the lines that follow (which live
/// at *lower* offsets) down by its length.
#[inline(always)]
fn insert_line(
    data: &mut DataBlock<&mut [u8]>,
    mut header: DataHeader,
    at: InsertAt,
    new: &NewLine,
    space_needed: int64_t,
) {
    let InsertAt { db_idx, line_count } = at;
    let len = new.len();
    let old_start = header.txt_start;
    header.txt_start = old_start.wrapping_sub(len);
    header.free = header.free.wrapping_sub(space_needed as c_uint);
    header.line_count += 1;
    data.set_header(&header);

    // The lines after the new one, whose text moves to the front and whose
    // index entries shift up one slot.
    let following = (line_count - db_idx - 1).max(0) as usize;
    // The new line ends where the line it follows starts — the end of the
    // text for the block's first line — or, with no line after it, at the
    // old start of the text.
    let end = if following == 0 {
        old_start
    } else if db_idx < 0 {
        header.txt_end
    } else {
        data.line_start(db_idx as usize)
    };
    data.insert_text(header.txt_start as usize, end as usize, new.text);
    let start = end.wrapping_sub(len);
    let entry = if new.marked() {
        start | DB_MARKED
    } else {
        start
    };
    data.insert_index((db_idx + 1) as usize, following, len.wrapping_neg(), entry);
}

/// Put `new` into the data block `data` as its line `slot`, at the front of
/// its text, which has room for it.
fn add_new_line(data: &mut DataBlock<&mut [u8]>, slot: usize, new: &NewLine) {
    let start = data.txt_start().wrapping_sub(new.len());
    data.set_txt_start(start);
    let room = data.free().wrapping_sub(new.len() + INDEX_SIZE as c_uint);
    data.set_free(room);
    data.set_index(slot, start);
    if new.marked() {
        data.set_marked(slot, true);
    }
    data.write(start as usize, new.text);
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
    let page_size = int64_t::from(buffer.b_ml.memfile().mf_page_size);
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
            let data = DataBlock(buffer.b_ml.page(id));
            data_moved = data
                .line_start(db_idx as usize)
                .wrapping_sub(data.txt_start()) as c_int;
            total_moved = data_moved + lines_moved * INDEX_SIZE as c_int;
            if int64_t::from(data.free()) + int64_t::from(total_moved) >= space_needed {
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
    let new_id = ml_new_data(buffer.b_ml.memfile(), negative, page_count);

    let (id_left, id_right, mut line_count_left, mut line_count_right) = if db_idx < 0 {
        (new_id, id, 0, line_count)
    } else {
        (id, new_id, line_count, 0)
    };
    let memfile = buffer.b_ml.memfile();
    let (bnum_left, bnum_right) = (memfile.block_nr(id_left), memfile.block_nr(id_right));
    let page_count_left = memfile.page_count(id_left) as c_int;
    let page_count_right = memfile.page_count(id_right) as c_int;

    {
        let (left_page, right_page) = memfile.pages_mut(id_left, id_right);
        let (mut left, mut right) = (DataBlock(left_page), DataBlock(right_page));

        // The new line may go into the right/new block.
        if !in_left {
            add_new_line(&mut right, 0, new);
            line_count_right += 1;
        }

        // Lines after the insertion point move from the left/old block to
        // the right/new one.
        if lines_moved != 0 {
            let room = right.txt_start().wrapping_sub(data_moved as c_uint);
            right.set_txt_start(room);
            right.set_free(right.free().wrapping_sub(total_moved as c_uint));
            let (from, to) = (left.txt_start() as usize, right.txt_start() as usize);
            let n = data_moved as usize;
            right.raw_mut()[to..to + n].copy_from_slice(&left.raw_mut()[from..from + n]);
            // How far the text shifted, and so how much every index entry
            // that came with it has to shift.
            let offset = right.txt_start().wrapping_sub(left.txt_start());
            left.set_txt_start(left.txt_start().wrapping_add(data_moved as c_uint));
            left.set_free(left.free().wrapping_add(total_moved as c_uint));

            let (from, to) = ((db_idx + 1) as usize, line_count_right as usize);
            right.copy_index_from(&left, from, to, lines_moved as usize, offset);
            line_count_right += lines_moved;
            line_count_left -= lines_moved;
        }

        // The new line may go into the left (old or new) block.
        if in_left {
            add_new_line(&mut left, line_count_left as usize, new);
            line_count_left += 1;
        }

        left.set_line_count(c_long::from(line_count_left));
        right.set_line_count(c_long::from(line_count_right));
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
    buffer.b_ml.put(new_id, true);

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
        let Some(mut id) = buffer.b_ml.memfile().get(ip.ip_bnum, 1) else {
            return false;
        };
        let pointers = PointerBlock(buffer.b_ml.page(id));
        if !pointers.is_pointer() {
            iemsg(gettext(c"E317: Pointer block id wrong 3"));
            buffer.b_ml.put(id, false);
            return false;
        }

        if pointers.count() < pointers.count_max() {
            ml_pointer_add_entry(buffer, id, pb_idx as usize, split, stack_idx as usize);
            return true;
        }

        // The pointer block is full: split it, and go round again to give
        // *its* parent the two halves.
        let new_id = ml_split_pointer_block(buffer, &mut id, &mut stack_idx);

        // Move the entries after the current one into the new block; if
        // there are none, the new entry itself goes there.
        let pb_idx = pb_idx as usize;
        let memfile = buffer.b_ml.memfile();
        let (page, page_new) = memfile.pages_mut(id, new_id);
        let (mut pointers, mut new_pointers) = (PointerBlock(page), PointerBlock(page_new));
        let total_moved = usize::from(pointers.count()) - pb_idx - 1;
        if total_moved != 0 {
            new_pointers.write_entries(0, pointers.entry_bytes(pb_idx + 1, total_moved));
            new_pointers.set_count(total_moved as u16);
            pointers.set_count(pointers.count() - (total_moved as u16 - 1));
            let (bnum, lines, pages) = (
                split.bnum_right,
                split.line_count_right,
                split.page_count_right,
            );
            pointers.set_child(pb_idx + 1, bnum, lines, pages, split.lnum_right);
        } else {
            new_pointers.set_count(1);
            new_pointers.set_child(
                0,
                split.bnum_right,
                split.line_count_right,
                split.page_count_right,
                0,
            );
            new_pointers.set_old_lnum(0, split.lnum_right);
        }
        let (bnum, lines, pages) = (
            split.bnum_left,
            split.line_count_left,
            split.page_count_left,
        );
        pointers.set_child(pb_idx, bnum, lines, pages, split.lnum_left);
        split.lnum_left = 0;
        split.lnum_right = 0;

        // The two pointer blocks are now the pair the level above has to
        // describe, so recount their lines.
        split.line_count_right = new_pointers.line_total();
        split.line_count_left = pointers.line_total();
        split.bnum_left = memfile.block_nr(id);
        split.bnum_right = memfile.block_nr(new_id);
        split.page_count_left = 1;
        split.page_count_right = 1;
        memfile.put(id, true, false);
        memfile.put(new_id, true, false);
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
    let mut pointers = PointerBlock(buffer.b_ml.page_mut(id));
    let count = usize::from(pointers.count());
    if pb_idx + 1 < count {
        pointers.move_entries(pb_idx + 1, pb_idx + 2, count - pb_idx - 1);
    }
    pointers.set_count(pointers.count().wrapping_add(1));

    let (bnum, lines, pages) = (
        split.bnum_left,
        split.line_count_left,
        split.page_count_left,
    );
    pointers.set_child(pb_idx, bnum, lines, pages, split.lnum_left);
    let (bnum, lines, pages) = (
        split.bnum_right,
        split.line_count_right,
        split.page_count_right,
    );
    pointers.set_child(pb_idx + 1, bnum, lines, pages, split.lnum_right);

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
        let new_id = ml_new_ptr(buffer.b_ml.memfile());
        let memfile = buffer.b_ml.memfile();
        if memfile.block_nr(*id) != 1 {
            return new_id;
        }

        // Both blocks are whole pages of the same size, and the new one
        // cannot be block 1.
        let bnum_new = memfile.block_nr(new_id);
        let (root_page, new_page) = memfile.pages_mut(*id, new_id);
        new_page.copy_from_slice(root_page);
        let mut root = PointerBlock(root_page);
        root.set_count(1);
        // Block 1 now holds exactly one entry, and this is it.
        let line_count = buffer.b_ml.ml_line_count;
        let memfile = buffer.b_ml.memfile();
        let mut root = PointerBlock(memfile.page_mut(*id));
        root.set_bnum(0, bnum_new);
        root.set_line_count(0, line_count);
        root.set_old_lnum(0, 1);
        root.set_page_count(0, 1);
        memfile.put(*id, true, false); // release block 1
        *id = new_id; // the new block is the one to split
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
    // Line 0 is line 1's block, where upstream went on to delete whatever
    // index entry -1 -- the block's own header -- said; it is refused.
    if !buffer.b_ml.is_open() || lnum < 1 {
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

    let data = DataBlock(buffer.b_ml.page(id));
    let line_start = data.line_start(idx);
    // The text runs to where the line before it in the block starts, or
    // to the end of the block for its first line.
    let line_size = data.line_end(idx).wrapping_sub(line_start) as c_int;

    // A line always holds its terminating NL internally, as a NUL, even
    // with 'noeol'.
    debug_assert!(line_size >= 1);
    let text = data.text(
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
        let mut data = DataBlock(buffer.b_ml.page_mut(id));
        let mut header = data.header();
        let text_start = header.txt_start;
        let size = line_size as c_uint;
        data.move_bytes(
            text_start as usize,
            (text_start + size) as usize,
            (line_start - text_start) as usize,
        );
        data.move_index(idx + 1, idx, count - 1 - idx, size);

        header.free = header.free.wrapping_add(size + INDEX_SIZE as c_uint);
        header.txt_start = text_start.wrapping_add(size);
        header.line_count -= 1;
        data.set_header(&header);

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
    buffer.b_ml.memfile().free_block(id);
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
        let Some(id) = buffer.b_ml.memfile().get(ip.ip_bnum, 1) else {
            buffer.b_ml.stack_clear();
            return false;
        };
        let mut pointers = PointerBlock(buffer.b_ml.page_mut(id));
        if !pointers.is_pointer() {
            iemsg(gettext(c"E317: Pointer block id wrong 4"));
            buffer.b_ml.put(id, false);
            buffer.b_ml.stack_clear();
            return false;
        }
        let count = pointers.count().wrapping_sub(1);
        pointers.set_count(count);
        let count = usize::from(count);
        if count == 0 {
            // This pointer block is empty now too.
            buffer.b_ml.memfile().free_block(id);
            continue;
        }

        if count != idx {
            // Move the entries after the deleted one down.
            pointers.move_entries(idx + 1, idx, count - idx);
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
