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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::cstr;
use crate::memfile::BlockId;
use crate::memline::state::MemLine;
use crate::message_fmt::msg_cstr;
use crate::narrow::len_as_int;
use crate::siemsg;
use crate::statusline::trans_bufname;
use crate::winlayer::MlRef;
use core::ffi::{c_int, c_uint};

use super::*;

/// `ML_SIMPLE(action)` upstream. The actions that may be answered straight
/// from the currently locked block; `ML_FLUSH` deliberately is not one, and
/// neither is anything that has to rebuild the stack.
const ML_SIMPLE: c_int = 0x10;

/// Suppresses a second complaint from a redraw triggered by the first.
static ML_GET_RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

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
pub(crate) fn fetch_line(buffer: Buf, lnum: LineNr, will_change: bool) {
    let (mut held, cached) = fetch_line_held(buffer, lnum);
    if will_change && cached == Cached::Line {
        held.locked_has_moved();
        book_deleted_line(buffer, held);
    }
}

/// [`fetch_line`] for a read, answering the walk's loan of the memline
/// still held: the caller lends the cached line out of it
/// ([`Buf::lend_line`]) instead of asking the memline again --
/// a search and a `Lines` read do this once per line.
#[inline]
pub(crate) fn fetch_line_lent<'a>(buffer: Buf, lnum: LineNr) -> MlRef<'a> {
    fetch_line_held(buffer, lnum).0
}

/// What [`fetch_line_held`] left in the line cache.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cached {
    /// The line itself.
    Line,
    /// A stand-in: there are no lines, or not this one.
    Placeholder,
}

/// The walk behind [`fetch_line`] and [`fetch_line_lent`]: one loan of the
/// memline, let go before anything that is handed the buffer or can run a
/// message and taken again after, answered still held.
#[inline]
fn fetch_line_held<'a>(buffer: Buf, lnum: LineNr) -> (MlRef<'a>, Cached) {
    let mut held = buffer.ml();
    let ml = &mut *held;
    if !ml.is_open() {
        // There are no lines at all.
        ml.cache_placeholder(c"", LineNr::ZERO);
        return (held, Cached::Placeholder);
    }

    if lnum > ml.ml_line_count {
        drop(held);
        return (invalid_lnum(buffer, lnum), Cached::Placeholder);
    }

    // Pretend line 0 is line 1.
    let lnum = lnum.max(LineNr::ONE);

    // If it is the line handed out last time, it is already unpacked;
    // otherwise the one that was may need flushing first.
    if ml.cached_lnum() == lnum {
        return (held, Cached::Line);
    }
    // The common case, a run of reads through one block: the line is in
    // the block already locked and nothing is waiting to be written back,
    // so the flush below would only forget the cached line and the walk
    // would only answer the locked block. Both are done here without the
    // two calls.
    if !ml.line_is_dirty()
        && let Some(id) = ml.locked_id()
        && ml.locked_low() <= lnum
        && lnum <= ml.locked_high()
    {
        if ml.cached_lnum() != LineNr::ZERO {
            ml.clear_cache();
        }
        let idx = (lnum - ml.locked_low()).cast_unsigned() as usize;
        ml.cache_block_line(id, idx, lnum);
        return (held, Cached::Line);
    }
    drop(held);
    fetch_line_by_walk(buffer, lnum)
}

/// [`fetch_line_held`] past the locked block: flush, then find the block
/// through the tree.
#[cold]
#[inline(never)]
fn fetch_line_by_walk<'a>(buffer: Buf, lnum: LineNr) -> (MlRef<'a>, Cached) {
    ml_flush_line(buffer);

    // Find the data block holding the line. This also fills the stack with
    // the blocks from the root down and releases any block that was locked.
    let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
        line_not_found(buffer, lnum);
        return (buffer.ml(), Cached::Placeholder);
    };
    let mut held = buffer.ml();
    let ml = &mut *held;
    let idx = (lnum - ml.locked_low()).cast_unsigned() as usize;
    ml.cache_block_line(id, idx, lnum);
    (held, Cached::Line)
}

/// Line `lnum` is past the end: complain (not from the redraw the
/// complaint causes) and cache the `???` placeholder.
#[cold]
#[inline(never)]
fn invalid_lnum<'a>(buffer: Buf, lnum: LineNr) -> MlRef<'a> {
    if ML_GET_RECURSIVE.get() == 0 {
        // Avoid giving this message for a recursive call, which
        // happens when the redraw it triggers reads the same line.
        ML_GET_RECURSIVE.set(1);
        siemsg!("E315: ml_get: Invalid lnum: {}", int64_t::from(lnum));
        ML_GET_RECURSIVE.set(0);
    }
    ml_flush_line(buffer);
    let mut held = buffer.ml();
    held.cache_placeholder(c"???", lnum);
    held
}

/// Book the cached line, which the caller is about to change, as deleted
/// text (`ml_add_deleted_len_buf`).
#[inline]
fn book_deleted_line(buffer: Buf, held: MlRef<'_>) {
    let text = buffer.lend_line(held, false);
    let counts = deleted_counts(buffer, &text, text.len());
    drop(text);
    book_deleted(buffer, counts);
}

/// The tree has no block for line `lnum`, which the line count says it
/// has: complain (once, not again from the redraw the complaint causes) and
/// cache the `???` placeholder.
#[cold]
fn line_not_found(buffer: Buf, lnum: LineNr) {
    if ML_GET_RECURSIVE.get() == 0 {
        ML_GET_RECURSIVE.set(1);
        let mut name = trans_bufname(buffer);
        shorten_dir(&mut name);
        // The missing space before "in buffer" is upstream's.
        let name = cstr::owned(&name);
        let name = msg_cstr(&name);
        siemsg!(
            "E316: ml_get: Cannot find line {}in buffer {} {name}",
            int64_t::from(lnum),
            buffer.handle()
        );
        ML_GET_RECURSIVE.set(0);
    }
    buffer.ml().cache_placeholder(c"???", lnum);
}

/// Set while a line is being written back: the write can append and delete
/// lines, which call back in here, and the line is already off the books by
/// then, so there is nothing left to do.
static FLUSHING: GlobalCell<bool> = GlobalCell::new(false);

/// Write the line `ml_replace` left in the line cache back into its data
/// block, and forget it.
pub(crate) fn ml_flush_line(buffer: Buf) {
    let ml = buffer.ml();
    let idle = ml.cached_lnum() == LineNr::ZERO || !ml.is_open();
    drop(ml);
    if idle {
        return; // nothing to do
    }

    if buffer.ml().line_is_dirty() {
        if FLUSHING.get() {
            return;
        }
        let lnum = buffer.ml().cached_lnum();
        let len = usize::try_from(buffer.ml().cached_len()).unwrap_or(0);
        let new_line = buffer.ml().take_replacement().unwrap_or_default();
        ml_flush_text(buffer, lnum, &new_line[..len.min(new_line.len())]);
    }

    buffer.ml().clear_cache();
}

/// Write `text` (its NUL included) over line `lnum` of `buffer`, as
/// flushing the line cache does. The cache's line number and length are
/// what the chunk index reads while it happens.
pub(crate) fn ml_flush_text(buffer: Buf, lnum: LineNr, text: &[u8]) {
    if FLUSHING.get() {
        return;
    }
    FLUSHING.set(true);
    buffer.ml().flush_count += 1;
    ml_write_line(buffer, lnum, text);
    FLUSHING.set(false);
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
fn ml_store_line(buffer: Buf, id: BlockId, lnum: LineNr, new_line: &[u8]) {
    // One edit for the block's rewrite, ended before the calls that are
    // handed the buffer.
    let mut held = buffer.ml();
    let ml = &mut *held;
    let idx = (lnum - ml.locked_low()).cast_unsigned() as usize;
    let count = (ml.locked_high() - ml.locked_low() + 1).cast_unsigned() as usize;
    let data = DataBlock(ml.page(id));
    let start = data.line_start(idx);
    // The text of the previous line follows it, or the block ends.
    let old_len = data.line_end(idx).cast_signed() - start.cast_signed();
    let (free, txt_start, marked) = (data.free(), data.txt_start(), data.is_marked(idx));
    let new_len = c_int::try_from(new_line.len()).expect("a line shorter than INT_MAX");
    // Negative if the line got smaller.
    let extra = new_len - old_len;

    if free.cast_signed() < extra {
        // It does not fit: delete and append instead. Append first,
        // because ml_delete_int cannot delete the last line of a
        // buffer, which is trouble for a buffer that has only one. The
        // mark has to come along.
        let mark = if marked { ML_APPEND_MARK } else { 0 };
        drop(held);
        let _ = ml_append_int(buffer, lnum, new_line, mark);
        let _ = ml_delete_int(buffer, lnum, 0);
        return;
    }

    let shift = extra.cast_unsigned();
    let mut data = DataBlock(ml.page_mut(id));
    if extra != 0 && idx + 1 < count {
        // Move the text of the lines that follow, and adjust their
        // offsets. (Lines are stored back to front, so "following"
        // lines sit at *lower* offsets.)
        let to = txt_start.wrapping_sub(shift) as usize;
        data.move_bytes(txt_start as usize, to, (start - txt_start) as usize);
        data.shift_index(idx + 1, count, shift.wrapping_neg());
    }
    data.set_index(idx, data.index(idx).wrapping_sub(shift));
    data.set_free(free.wrapping_sub(shift));
    data.set_txt_start(txt_start.wrapping_sub(shift));
    data.write(start.wrapping_sub(shift) as usize, new_line);
    ml.locked_has_moved();
    drop(held);

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
    let id = memfile.new_block(
        negative,
        c_uint::try_from(page_count).expect("a block's pages"),
    );
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
/// [`LockedBlock::lineadd`]: crate::memline::state::LockedBlock::lineadd
pub(crate) fn ml_find_line(buffer: Buf, lnum: LineNr, action: c_int) -> Option<BlockId> {
    // One loan of the memline for the walk (this is every read's path),
    // taken again only after a call that is handed the buffer, and ended
    // before a message.
    let mut ml = buffer.ml();

    // If a block is locked, see whether the wanted line is in it. Not
    // for ML_FLUSH (the point of which is to release it), and not for
    // the actions that have to rebuild the stack.
    if ml.is_locked() {
        if action & ML_SIMPLE != 0 && ml.locked_low() <= lnum && ml.locked_high() >= lnum {
            // Remember to update the pointer blocks and the stack later.
            if action == ML_INSERT {
                ml.shift_locked(1);
            } else if action == ML_DELETE {
                ml.shift_locked(-1);
            }
            return ml.locked_id();
        }

        // Give it back, telling the memfile what was done to it, and
        // then repair the line counts in the pointer blocks above: lines
        // were added or deleted in it and they were never told.
        if let Some(locked) = ml.unlock() {
            ml.memfile().put(locked.id, locked.dirty, locked.moved);
            if locked.lineadd != 0 {
                drop(ml);
                ml_lineadd(buffer, locked.lineadd);
                ml = buffer.ml();
            }
        }
    }

    if action == ML_FLUSH {
        return None; // nothing else to do
    }

    let mut bnum: BlockNr = 1; // start at the root of the tree
    let mut page_count: c_int = 1;
    let mut low: LineNr = LineNr::ONE;
    let mut high: LineNr = ml.ml_line_count;

    if action == ML_FIND {
        // The previous walk's stack usually still covers this line —
        // reads come in runs. Restart from the deepest entry that does.
        let mut top = ml.stack_len();
        let mut resumed = false;
        while top > 0 {
            top -= 1;
            let ip = ml.stack_at(top);
            if ip.ip_low <= lnum && ip.ip_high >= lnum {
                bnum = ip.ip_bnum;
                low = ip.ip_low;
                high = ip.ip_high;
                ml.stack_truncate(top); // drop the entry itself
                resumed = true;
                break;
            }
        }
        if !resumed {
            ml.stack_clear(); // not found, start at the root
        }
    } else {
        // ML_DELETE or ML_INSERT: the whole path has to be rewritten.
        ml.stack_clear();
    }

    // Search downwards until a data block is found.
    while let Some(id) = ml.memfile().get(bnum, page_count.cast_unsigned()) {
        if action == ML_INSERT {
            high += 1;
        } else if action == ML_DELETE {
            high -= 1;
        }

        if DataBlock(ml.page(id)).is_data() {
            ml.lock(id, low, high);
            return Some(id);
        }

        // Anything that is not a data block must be a pointer block. The
        // block goes back before the message, which may redraw.
        if !PointerBlock(ml.page(id)).is_pointer() {
            ml.put(id, false);
            drop(ml);
            iemsg(gettext(c"E317: Pointer block id wrong"));
            return find_line_failed(buffer, action);
        }

        let top = ml.stack_push();
        let frame = InfoPtr {
            ip_bnum: bnum,
            ip_low: low,
            ip_high: high,
            ip_index: -1, // index not known yet
        };
        ml.stack_set(top, frame);

        let mut dirty = false;
        let pointers = PointerBlock(ml.page(id));
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
            let line_count = ml.ml_line_count;
            ml.put(id, false);
            drop(ml);
            if lnum > line_count {
                siemsg!(
                    "E322: Line number out of range: {} past the end",
                    int64_t::from(lnum) - int64_t::from(line_count)
                );
            } else {
                siemsg!("E323: Line count wrong in block {}", bnum);
            }
            return find_line_failed(buffer, action);
        }

        let entry = pointers.entry(idx);
        ml.stack_set_index(top, len_as_int(idx));
        bnum = entry.pe_bnum;
        page_count = entry.pe_page_count;
        high = low - 1;
        low -= entry.pe_line_count;

        // A negative block number is one that has not been written yet;
        // it may since have been given a real one.
        if bnum < 0 {
            let renumbered = ml.memfile().trans_del(bnum);
            if bnum != renumbered {
                bnum = renumbered;
                PointerBlock(ml.page_mut(id)).set_bnum(idx, bnum);
                dirty = true;
            }
        }

        if action == ML_DELETE {
            PointerBlock(ml.page_mut(id)).add_line_count(idx, -1);
            dirty = true;
        } else if action == ML_INSERT {
            PointerBlock(ml.page_mut(id)).add_line_count(idx, 1);
            dirty = true;
        }
        ml.put(id, dirty);
    }

    drop(ml);
    find_line_failed(buffer, action)
}

/// The walk failed. For ML_DELETE/ML_INSERT the counts on the way down were
/// already adjusted for a line that will not be inserted/deleted after all,
/// so put them back.
#[cold]
fn find_line_failed(buffer: Buf, action: c_int) -> Option<BlockId> {
    if action == ML_DELETE {
        ml_lineadd(buffer, 1);
    } else if action == ML_INSERT {
        ml_lineadd(buffer, -1);
    }
    buffer.ml().stack_clear();
    None
}

/// Push an entry onto the info-pointer stack and return its index. The
/// entry is left blank; every caller fills it in.
pub(crate) fn ml_add_stack(b: Buf) -> usize {
    b.ml().stack_push()
}

/// Add `count` (negative to subtract) to the line count of every pointer
/// block on the stack, and to the stack entries themselves.
///
/// This is the repair path: when an insert or delete fails part way down,
/// the pointer blocks have already been updated for it.
pub(crate) fn ml_lineadd(buffer: Buf, count: c_int) {
    let arg = buffer.ml().stack_len();
    ml_lineadd_depth(buffer, count, arg);
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
pub(crate) fn ml_lineadd_depth(b: Buf, count: c_int, depth: usize) {
    // One edit for the climb, ended before the message.
    let mut held = b.ml();
    let ml = &mut *held;
    let mut idx = depth;
    while idx > 0 {
        idx -= 1;
        let ip = ml.stack_at(idx);
        let Some(id) = ml.memfile().get(ip.ip_bnum, 1) else {
            break;
        };
        // Must be a pointer block: it is on the stack.
        let mut pointers = PointerBlock(ml.page_mut(id));
        if !pointers.is_pointer() {
            ml.put(id, false);
            drop(held);
            iemsg(gettext(c"E317: Pointer block id wrong 2"));
            return;
        }
        pointers.add_line_count(ip.ip_index.cast_unsigned() as usize, count);
        ml.stack_add_high(idx, count);
        ml.put(id, true);
    }
}
