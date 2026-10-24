//! Walking the block tree, and reading a line out of it.
//!
//! A memline is a B-tree: pointer blocks branch by line count, and data blocks
//! hold the text. `ml_find_line` is the walk every read and write starts from,
//! and the one place the locked block and the block stack are built.
//! `fetch_line` is the read on top of it, and `ml_flush_line` the write
//! back of a line that `ml_replace` handed out.
//!
//! Every block is reached through the memfile by its [`BlockId`] and read
//! through a view ([`DataBlock`], [`PointerBlock`]) made fresh after any call
//! that can reach the memfile again: a view is a borrow of the page, and
//! nothing holds one across a `get`, a `new_block` or a split.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::memfile::BlockId;
use crate::message_fmt::msg_cstr;
use crate::siemsg;
use crate::statusline::trans_bufname;
use crate::types::MemLine;
use core::ffi::{c_int, c_uint};

use super::*;

/// `ML_SIMPLE(action)` upstream. The actions that may be answered straight
/// from the currently locked block; `ML_FLUSH` deliberately is not one, and
/// neither is anything that has to rebuild the stack.
const ML_SIMPLE: c_int = 0x10;

/// Suppresses a second complaint from a redraw triggered by the first.
static ml_get_recursive: GlobalCell<c_int> = GlobalCell::new(0);

impl MemLine {
    /// Block `id`'s bytes.
    #[inline]
    pub(crate) fn page(&self, id: BlockId) -> &[u8] {
        self.ml_mfp.as_deref().expect("an open memline").page(id)
    }

    /// Block `id`'s bytes, writable.
    #[inline]
    pub(crate) fn page_mut(&mut self, id: BlockId) -> &mut [u8] {
        self.memfile().page_mut(id)
    }

    /// Give block `id` back to the memfile.
    pub(crate) fn put(&mut self, id: BlockId, dirty: bool) {
        self.memfile().put(id, dirty, false);
    }
}

/// Unpack line `lnum` of `buffer` into the line cache: the line in the data
/// block holding it, or the replacement a pending `ml_replace` left.
///
/// `will_change` says the caller is about to write into the line, which
/// dirties the block and books the old text as deleted.
///
/// Never fails: the failure paths cache the `???` placeholder.
pub(crate) fn fetch_line(mut buffer: Buf, lnum: LineNr, will_change: bool) {
    if !buffer.b_ml.is_open() {
        // There are no lines at all.
        buffer.b_ml.cache_placeholder(c"", 0);
        return;
    }

    if lnum > buffer.b_ml.ml_line_count {
        if ml_get_recursive.get() == 0 {
            // Avoid giving this message for a recursive call, which
            // happens when the redraw it triggers reads the same line.
            ml_get_recursive.set(1);
            siemsg!("E315: ml_get: Invalid lnum: {}", int64_t::from(lnum));
            ml_get_recursive.set(0);
        }
        ml_flush_line(buffer);
        buffer.b_ml.cache_placeholder(c"???", lnum);
        return;
    }

    // Pretend line 0 is line 1.
    let lnum = lnum.max(1);

    // If it is the line handed out last time, it is already unpacked;
    // otherwise the one that was may need flushing first.
    if buffer.b_ml.cached_lnum() != lnum {
        // The common case, a run of reads through one block: the line is in
        // the block already locked and nothing is waiting to be written
        // back, so the flush below would only forget the cached line and
        // the walk would only answer the locked block. Both are done here
        // without the two calls.
        if !buffer.b_ml.line_is_dirty()
            && let Some(id) = buffer.b_ml.locked_id()
            && buffer.b_ml.locked_low() <= lnum
            && lnum <= buffer.b_ml.locked_high()
        {
            if buffer.b_ml.cached_lnum() != 0 {
                buffer.b_ml.clear_cache();
            }
            let idx = (lnum - buffer.b_ml.locked_low()) as usize;
            buffer.b_ml.cache_block_line(id, idx, lnum);
            if will_change {
                buffer.b_ml.locked_has_moved();
                let text = buffer.b_ml.cached_bytes();
                ml_add_deleted_len_buf(buffer, text, text.len());
            }
            return;
        }
        ml_flush_line(buffer);

        // Find the data block holding the line. This also fills the
        // stack with the blocks from the root down and releases any
        // block that was locked.
        let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
            line_not_found(buffer, lnum);
            return;
        };
        let idx = (lnum - buffer.b_ml.locked_low()) as usize;
        buffer.b_ml.cache_block_line(id, idx, lnum);
    }

    if will_change {
        buffer.b_ml.locked_has_moved();
        let text = buffer.b_ml.cached_bytes();
        ml_add_deleted_len_buf(buffer, text, text.len());
    }
}

/// The tree has no block for line `lnum`, which the line count says it
/// has: complain (once, not again from the redraw the complaint causes) and
/// cache the `???` placeholder.
#[cold]
fn line_not_found(mut buffer: Buf, lnum: LineNr) {
    if ml_get_recursive.get() == 0 {
        ml_get_recursive.set(1);
        let mut name = trans_bufname(buffer);
        shorten_dir(&mut name);
        // The missing space before "in buffer" is upstream's.
        let name = cstr::owned(&name);
        let name = msg_cstr(&name);
        siemsg!(
            "E316: ml_get: Cannot find line {}in buffer {} {name}",
            int64_t::from(lnum),
            buffer.handle
        );
        ml_get_recursive.set(0);
    }
    buffer.b_ml.cache_placeholder(c"???", lnum);
}

/// Set while a line is being written back: the write can append and delete
/// lines, which call back in here, and the line is already off the books by
/// then, so there is nothing left to do.
static flushing: GlobalCell<bool> = GlobalCell::new(false);

/// Write the line `ml_replace` left in the line cache back into its data
/// block, and forget it.
pub(crate) fn ml_flush_line(mut buffer: Buf) {
    if buffer.b_ml.cached_lnum() == 0 || !buffer.b_ml.is_open() {
        return; // nothing to do
    }

    if buffer.b_ml.line_is_dirty() {
        if flushing.get() {
            return;
        }
        let lnum = buffer.b_ml.cached_lnum();
        let len = usize::try_from(buffer.b_ml.cached_len()).unwrap_or(0);
        let new_line = buffer.b_ml.take_replacement().unwrap_or_default();
        ml_flush_text(buffer, lnum, &new_line[..len.min(new_line.len())]);
    }

    buffer.b_ml.clear_cache();
}

/// Write `text` (its NUL included) over line `lnum` of `buffer`, as
/// flushing the line cache does. The cache's line number and length are
/// what the chunk index reads while it happens.
pub(crate) fn ml_flush_text(mut buffer: Buf, lnum: LineNr, text: &[u8]) {
    if flushing.get() {
        return;
    }
    flushing.set(true);
    buffer.flush_count += 1;
    ml_write_line(buffer, lnum, text);
    flushing.set(false);
}

/// Write `new_line` (its NUL included) over line `lnum` of `buffer`.
fn ml_write_line(buffer: Buf, lnum: LineNr, new_line: &[u8]) {
    match ml_find_line(buffer, lnum, ML_FIND) {
        None => siemsg!("E320: Cannot find line {}", int64_t::from(lnum)),
        Some(id) => ml_store_line(buffer, id, lnum, new_line),
    }
}

/// The body of [`ml_flush_line`] once the block is found: overwrite line
/// `lnum` in block `id` with `new_line`, in place if the new text fits.
fn ml_store_line(mut buffer: Buf, id: BlockId, lnum: LineNr, new_line: &[u8]) {
    let idx = (lnum - buffer.b_ml.locked_low()) as usize;
    let count = (buffer.b_ml.locked_high() - buffer.b_ml.locked_low() + 1) as usize;
    let data = DataBlock(buffer.b_ml.page(id));
    let start = data.line_start(idx);
    // The text of the previous line follows it, or the block ends.
    let old_len = data.line_end(idx) as c_int - start as c_int;
    let (free, txt_start, marked) = (data.free(), data.txt_start(), data.is_marked(idx));
    let new_len = c_int::try_from(new_line.len()).expect("a line shorter than INT_MAX");
    // Negative if the line got smaller.
    let extra = new_len - old_len;

    if (free as c_int) < extra {
        // It does not fit: delete and append instead. Append first,
        // because ml_delete_int cannot delete the last line of a
        // buffer, which is trouble for a buffer that has only one. The
        // mark has to come along.
        let mark = if marked { ML_APPEND_MARK } else { 0 };
        let _ = ml_append_int(buffer, lnum, new_line, mark);
        let _ = ml_delete_int(buffer, lnum, 0);
        return;
    }

    let shift = extra as c_uint;
    let mut data = DataBlock(buffer.b_ml.page_mut(id));
    if extra != 0 && idx + 1 < count {
        // Move the text of the lines that follow, and adjust their
        // offsets. (Lines are stored back to front, so "following"
        // lines sit at *lower* offsets.)
        let to = (txt_start as isize - extra as isize) as usize;
        data.move_bytes(txt_start as usize, to, (start - txt_start) as usize);
        data.shift_index(idx + 1, count, shift.wrapping_neg());
    }
    data.set_index(idx, data.index(idx).wrapping_sub(shift));
    data.set_free(free.wrapping_sub(shift));
    data.set_txt_start(txt_start.wrapping_sub(shift));
    data.write((start as isize - extra as isize) as usize, new_line);

    buffer.b_ml.locked_has_moved();
    // The `extra == 0` case is already covered by the insert and delete.
    if extra != 0 {
        buffer.update_line_chunks(lnum, extra, ML_CHNK_UPDLINE);
    }
}

/// A new, empty data block of `page_count` pages.
///
/// `negative` asks the memfile for a negative block number, which is what an
/// as-yet-unwritten block gets.
pub(crate) fn ml_new_data(memfile: &mut MemFile, negative: bool, page_count: int64_t) -> BlockId {
    debug_assert!(page_count >= 0);
    let id = memfile.new_block(negative, page_count as c_uint);
    DataBlock(memfile.page_mut(id)).init();
    id
}

/// A new, empty pointer block.
pub(crate) fn ml_new_ptr(memfile: &mut MemFile) -> BlockId {
    let id = memfile.new_block(false, 1);
    let page_size = memfile.mf_page_size;
    PointerBlock(memfile.page_mut(id)).init(page_size);
    id
}

/// Find the data block holding line `lnum`, locking it and leaving the path
/// from the root in `ml_stack`.
///
/// `action` is `ML_FIND` to just look, `ML_INSERT`/`ML_DELETE` to also adjust
/// the line counts on the way down for a line about to be added or removed,
/// or `ML_FLUSH` to only release the locked block.
///
/// `ip_high` in each stack entry reflects the last line in that block *after*
/// the insert or delete, even though the pointer block itself may not have
/// been updated yet — except that while a block is locked, the lines it owes
/// its parents ([`LockedBlock::lineadd`]) still have to be added to it.
///
/// [`LockedBlock::lineadd`]: crate::types::memline::LockedBlock::lineadd
pub(crate) fn ml_find_line(mut buffer: Buf, lnum: LineNr, action: c_int) -> Option<BlockId> {
    // If a block is locked, see whether the wanted line is in it. Not
    // for ML_FLUSH (the point of which is to release it), and not for
    // the actions that have to rebuild the stack.
    if buffer.b_ml.is_locked() {
        if action & ML_SIMPLE != 0
            && buffer.b_ml.locked_low() <= lnum
            && buffer.b_ml.locked_high() >= lnum
        {
            // Remember to update the pointer blocks and the stack later.
            if action == ML_INSERT {
                buffer.b_ml.shift_locked(1);
            } else if action == ML_DELETE {
                buffer.b_ml.shift_locked(-1);
            }
            return buffer.b_ml.locked_id();
        }

        // Give it back, telling the memfile what was done to it, and
        // then repair the line counts in the pointer blocks above: lines
        // were added or deleted in it and they were never told.
        if let Some(locked) = buffer.b_ml.unlock() {
            buffer
                .b_ml
                .memfile()
                .put(locked.id, locked.dirty, locked.moved);
            if locked.lineadd != 0 {
                ml_lineadd(buffer, locked.lineadd);
            }
        }
    }

    if action == ML_FLUSH {
        return None; // nothing else to do
    }

    let mut bnum: BlockNr = 1; // start at the root of the tree
    let mut page_count: c_int = 1;
    let mut low: LineNr = 1;
    let mut high: LineNr = buffer.b_ml.ml_line_count;

    if action == ML_FIND {
        // The previous walk's stack usually still covers this line —
        // reads come in runs. Restart from the deepest entry that does.
        let mut top = buffer.b_ml.stack_len();
        let mut resumed = false;
        while top > 0 {
            top -= 1;
            let ip = buffer.b_ml.stack_at(top);
            if ip.ip_low <= lnum && ip.ip_high >= lnum {
                bnum = ip.ip_bnum;
                low = ip.ip_low;
                high = ip.ip_high;
                buffer.b_ml.stack_truncate(top); // drop the entry itself
                resumed = true;
                break;
            }
        }
        if !resumed {
            buffer.b_ml.stack_clear(); // not found, start at the root
        }
    } else {
        // ML_DELETE or ML_INSERT: the whole path has to be rewritten.
        buffer.b_ml.stack_clear();
    }

    // Search downwards until a data block is found.
    while let Some(id) = buffer.b_ml.memfile().get(bnum, page_count as c_uint) {
        if action == ML_INSERT {
            high += 1;
        } else if action == ML_DELETE {
            high -= 1;
        }

        if DataBlock(buffer.b_ml.page(id)).is_data() {
            buffer.b_ml.lock(id, low, high);
            return Some(id);
        }

        // Anything that is not a data block must be a pointer block.
        if !PointerBlock(buffer.b_ml.page(id)).is_pointer() {
            iemsg(gettext(c"E317: Pointer block id wrong"));
            buffer.b_ml.put(id, false);
            break;
        }

        let top = ml_add_stack(buffer);
        let frame = InfoPtr {
            ip_bnum: bnum,
            ip_low: low,
            ip_high: high,
            ip_index: -1, // index not known yet
        };
        buffer.b_ml.stack_set(top, frame);

        let mut dirty = false;
        let pointers = PointerBlock(buffer.b_ml.page(id));
        let count = usize::from(pointers.count());
        let mut idx = 0;
        for t in pointers.line_counts() {
            low += t;
            if low > lnum {
                break;
            }
            idx += 1;
        }

        if idx >= count {
            // Past the end: the tree disagrees with the line count.
            if lnum > buffer.b_ml.ml_line_count {
                siemsg!(
                    "E322: Line number out of range: {} past the end",
                    int64_t::from(lnum) - int64_t::from(buffer.b_ml.ml_line_count)
                );
            } else {
                siemsg!("E323: Line count wrong in block {}", bnum);
            }
            buffer.b_ml.put(id, false);
            break;
        }

        let entry = pointers.entry(idx);
        buffer.b_ml.stack_set_index(top, idx as c_int);
        bnum = entry.pe_bnum;
        page_count = entry.pe_page_count;
        high = low - 1;
        low -= entry.pe_line_count;

        // A negative block number is one that has not been written yet;
        // it may since have been given a real one.
        if bnum < 0 {
            let renumbered = buffer.b_ml.memfile().trans_del(bnum);
            if bnum != renumbered {
                bnum = renumbered;
                PointerBlock(buffer.b_ml.page_mut(id)).set_bnum(idx, bnum);
                dirty = true;
            }
        }

        if action == ML_DELETE {
            PointerBlock(buffer.b_ml.page_mut(id)).add_line_count(idx, -1);
            dirty = true;
        } else if action == ML_INSERT {
            PointerBlock(buffer.b_ml.page_mut(id)).add_line_count(idx, 1);
            dirty = true;
        }
        buffer.b_ml.put(id, dirty);
    }

    // The walk failed. For ML_DELETE/ML_INSERT the counts on the way
    // down were already adjusted for a line that will not be
    // inserted/deleted after all, so put them back.
    if action == ML_DELETE {
        ml_lineadd(buffer, 1);
    } else if action == ML_INSERT {
        ml_lineadd(buffer, -1);
    }
    buffer.b_ml.stack_clear();
    None
}

/// Push an entry onto the info-pointer stack and return its index. The
/// entry is left blank; every caller fills it in.
pub(crate) fn ml_add_stack(mut b: Buf) -> usize {
    b.b_ml.stack_push()
}

/// Add `count` (negative to subtract) to the line count of every pointer
/// block on the stack, and to the stack entries themselves.
///
/// This is the repair path: when an insert or delete fails part way down,
/// the pointer blocks have already been updated for it.
pub(crate) fn ml_lineadd(buffer: Buf, count: c_int) {
    ml_lineadd_depth(buffer, count, buffer.b_ml.stack_len());
}

/// [`ml_lineadd`] over the bottom `depth` entries of the stack only.
///
/// The two callers that repair a *pointer* block have already put the entry
/// they are working on back on the stack, and it must not be corrected
/// twice; upstream expressed that by dropping `ml_stack_top` for the call
/// and reading the entry back from above it afterwards, which a `Vec` will
/// not do.
///
/// # Panics
/// When `depth` is more than the stack holds.
pub(crate) fn ml_lineadd_depth(mut b: Buf, count: c_int, depth: usize) {
    let mut idx = depth;
    while idx > 0 {
        idx -= 1;
        let ip = b.b_ml.stack_at(idx);
        let Some(id) = b.b_ml.memfile().get(ip.ip_bnum, 1) else {
            break;
        };
        // Must be a pointer block: it is on the stack.
        let mut pointers = PointerBlock(b.b_ml.page_mut(id));
        if !pointers.is_pointer() {
            b.b_ml.put(id, false);
            iemsg(gettext(c"E317: Pointer block id wrong 2"));
            break;
        }
        pointers.add_line_count(ip.ip_index as usize, count);
        b.b_ml.stack_add_high(idx, count);
        b.b_ml.put(id, true);
    }
}
