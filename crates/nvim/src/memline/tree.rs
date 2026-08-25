//! Walking the block tree, and reading a line out of it.
//!
//! A memline is a B-tree: pointer blocks branch by line count, and data blocks
//! hold the text. `ml_find_line` is the walk every read and write starts from,
//! and the one place the block stack in `ml_locked`/`ml_stack` is built.
//! `ml_get_buf_impl` is the read on top of it, and `ml_flush_line` the write
//! back of a line that `ml_replace` handed out.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::siemsg_c;
use crate::types::MAXPATHL;
use core::ffi::{c_char, c_int, c_uint};

use super::*;

/// `ML_SIMPLE(action)` upstream. The actions that may be answered straight
/// from the currently locked block; `ML_FLUSH` deliberately is not one, and
/// neither is anything that has to rebuild the stack.
const ML_SIMPLE: c_int = 0x10;

/// The line-offset array that follows a data block's header.
///
/// `db_index` is declared `[c_uint; 0]` — the block is a whole page and the
/// array runs to the end of it — so the pointer has to be derived from the
/// block pointer with `&raw mut`. An autoref (`.as_mut_ptr()`) would carry
/// provenance over zero bytes and every access past it would be out of
/// bounds.
///
/// Entry `i` is the offset from the start of the block at which line
/// `ml_locked_low + i`'s text begins, with [`DB_MARKED`] possibly set in the
/// top bit. Lines are stored back to front: entry 0 is the *last* line's
/// text, at the highest offset.
///
/// # Safety
/// `dp` must point at a data block.
pub(crate) unsafe fn db_index(dp: *mut DataBlock) -> *mut c_uint {
    unsafe { (&raw mut (*dp).db_index).cast::<c_uint>() }
}

/// Where line `idx` of the block starts, with the mark bit stripped.
///
/// # Safety
/// `dp` must point at a data block holding more than `idx` lines.
pub(crate) unsafe fn db_line_start(dp: *mut DataBlock, idx: c_int) -> c_uint {
    unsafe { *db_index(dp).offset(idx as isize) & DB_INDEX_MASK }
}

/// The entry array that follows a pointer block's header. Same
/// zero-length-array provenance rule as [`db_index`].
///
/// # Safety
/// `pp` must point at a pointer block.
pub(crate) unsafe fn pb_entries(pp: *mut PointerBlock) -> *mut PointerEntry {
    unsafe { (&raw mut (*pp).pb_pointer).cast::<PointerEntry>() }
}

impl PointerBlock {
    /// How many entries fit in a pointer block of this page size. It is
    /// stored in the block itself (`pb_count_max`), so that a swap file
    /// written by a build with a different page size is recognisable.
    pub(crate) fn count_max(page_size: c_uint) -> uint16_t {
        let header = core::mem::offset_of!(PointerBlock, pb_pointer);
        ((page_size as usize - header) / size_of::<PointerEntry>()) as uint16_t
    }
}

/// The placeholder `ml_get` hands back when it cannot read the line: an
/// invalid line number, or a tree walk that found no block.
///
/// It is a mutable static rather than a literal because callers are handed a
/// `*mut c_char` and upstream rewrites the buffer on every call.
static questions: GlobalCell<[c_char; 4]> = GlobalCell::new([0; 4]);

/// Suppresses a second complaint from a redraw triggered by the first.
static ml_get_recursive: GlobalCell<c_int> = GlobalCell::new(0);

/// # Safety
/// `buf` must point at a buffer.
unsafe fn ml_get_placeholder(buf: *mut buf_T, lnum: linenr_T) -> *mut c_char {
    unsafe {
        questions.set([b'?' as c_char, b'?' as c_char, b'?' as c_char, 0]);
        (*buf).b_ml.ml_line_textlen = 4;
        (*buf).b_ml.ml_line_lnum = lnum;
        questions.ptr().cast::<c_char>()
    }
}

/// Read line `lnum` of `buf`, as a NUL-terminated pointer into the data
/// block holding it (or into `ml_line_ptr` if a `ml_replace` is pending).
///
/// `will_change` says the caller is about to write through the pointer, which
/// dirties the block and books the old text as deleted.
///
/// Never returns NULL: the failure paths answer "???".
///
/// # Safety
/// `buf` must point at a buffer.
pub(crate) unsafe fn ml_get_buf_impl(
    buf: *mut buf_T,
    lnum: linenr_T,
    will_change: bool,
) -> *mut c_char {
    // Where the E316 report's buffer name goes; upstream shares `NameBuff`.
    let mut name = [0 as c_char; MAXPATHL as usize];
    unsafe {
        if (*buf).b_ml.ml_mfp.is_null() {
            // There are no lines at all.
            (*buf).b_ml.ml_line_textlen = 1;
            return c"".as_ptr().cast_mut();
        }

        if lnum > (*buf).b_ml.ml_line_count {
            if ml_get_recursive.get() == 0 {
                // Avoid giving this message for a recursive call, which
                // happens when the redraw it triggers reads the same line.
                ml_get_recursive.set(1);
                siemsg_c!(
                    gettext(c"E315: ml_get: Invalid lnum: %ld".as_ptr()),
                    lnum as int64_t,
                );
                ml_get_recursive.set(0);
            }
            ml_flush_line(buf, false);
            return ml_get_placeholder(buf, lnum);
        }

        // Pretend line 0 is line 1.
        let lnum = lnum.max(1);

        // If it is the line handed out last time, it is already unpacked;
        // otherwise the one that was may need flushing first.
        if (*buf).b_ml.ml_line_lnum != lnum {
            ml_flush_line(buf, false);

            // Find the data block holding the line. This also fills the
            // stack with the blocks from the root down and releases any
            // block that was locked.
            let hp = ml_find_line(buf, lnum, ML_FIND);
            if hp.is_null() {
                if ml_get_recursive.get() == 0 {
                    ml_get_recursive.set(1);
                    get_trans_bufname(buf, &mut name);
                    shorten_dir(name.as_mut_ptr());
                    // The missing space before "in buffer" is upstream's.
                    siemsg_c!(
                        gettext(c"E316: ml_get: Cannot find line %ldin buffer %d %s".as_ptr()),
                        lnum as int64_t,
                        (*buf).handle,
                        name.as_ptr(),
                    );
                    ml_get_recursive.set(0);
                }
                return ml_get_placeholder(buf, lnum);
            }

            let dp = (*hp).bh_data as *mut DataBlock;
            let idx = lnum - (*buf).b_ml.ml_locked_low;
            let start = db_line_start(dp, idx);
            // The text ends where the previous line starts; the first line
            // of the block ends at the end of the block.
            let end = if idx == 0 {
                (*dp).db_txt_end
            } else {
                db_line_start(dp, idx - 1)
            };

            (*buf).b_ml.ml_line_ptr = (dp as *mut c_char).offset(start as isize);
            (*buf).b_ml.ml_line_textlen = end.wrapping_sub(start) as colnr_T;
            (*buf).b_ml.ml_line_lnum = lnum;
            (*buf).b_ml.forget_line();
        }

        if will_change {
            (*buf).b_ml.ml_flags |= MlFlags::LOCKED_DIRTY | MlFlags::LOCKED_POS;
            ml_add_deleted_len_buf(buf, (*buf).b_ml.ml_line_ptr, -1);
        }
        (*buf).b_ml.ml_line_ptr
    }
}

/// Write the line `ml_replace` left in `ml_line_ptr` back into its data
/// block, and forget it.
///
/// `noalloc` says the caller owns the line's memory and this must not free
/// it; it is only ever set together with [`MlFlags::LINE_DIRTY`].
///
/// # Safety
/// `buf` must point at a buffer.
pub(crate) unsafe fn ml_flush_line(buf: *mut buf_T, noalloc: bool) {
    unsafe {
        // ml_append_int/ml_delete_int below call back in here; the line is
        // already off the books by then, so there is nothing left to do.
        static entered: GlobalCell<bool> = GlobalCell::new(false);

        if (*buf).b_ml.ml_line_lnum == 0 || (*buf).b_ml.ml_mfp.is_null() {
            return; // nothing to do
        }

        if (*buf).b_ml.ml_flags.has(MlFlags::LINE_DIRTY) {
            if entered.get() {
                return;
            }
            entered.set(true);
            (*buf).flush_count += 1;

            let lnum = (*buf).b_ml.ml_line_lnum;
            let new_line = (*buf).b_ml.ml_line_ptr;

            let hp = ml_find_line(buf, lnum, ML_FIND);
            if hp.is_null() {
                siemsg_c!(
                    gettext(c"E320: Cannot find line %ld".as_ptr()),
                    lnum as int64_t,
                );
            } else {
                ml_store_line(buf, hp, lnum, new_line);
            }

            if !noalloc {
                xfree(new_line.cast());
            }
            entered.set(false);
        } else if (*buf).b_ml.ml_flags.has(MlFlags::ALLOCATED) {
            // The caller must set `MlFlags::LINE_DIRTY` along with noalloc, which
            // the branch above handles.
            debug_assert!(!noalloc);
            xfree((*buf).b_ml.ml_line_ptr.cast());
        }

        (*buf).b_ml.forget_line();
        (*buf).b_ml.ml_line_lnum = 0;
        (*buf).b_ml.ml_line_offset = 0;
    }
}

/// The body of [`ml_flush_line`] once the block is found: overwrite line
/// `lnum` in `hp` with `new_line`, in place if the new text fits.
///
/// # Safety
/// `hp` must be the block `ml_find_line(buf, lnum, ML_FIND)` returned, still
/// locked, and `new_line` must hold `ml_line_textlen` readable bytes.
unsafe fn ml_store_line(buf: *mut buf_T, hp: *mut bhdr_T, lnum: linenr_T, new_line: *mut c_char) {
    unsafe {
        let dp = (*hp).bh_data as *mut DataBlock;
        let idx = lnum - (*buf).b_ml.ml_locked_low;
        let start = db_line_start(dp, idx) as c_int;
        let old_line = (dp as *mut c_char).offset(start as isize);
        let old_len = if idx == 0 {
            // Line is last in the block, so its text runs to the end.
            (*dp).db_txt_end as c_int - start
        } else {
            // The text of the previous line follows it.
            db_line_start(dp, idx - 1) as c_int - start
        };
        let new_len = (*buf).b_ml.ml_line_textlen;
        // Negative if the line got smaller.
        let extra = new_len - old_len;

        if ((*dp).db_free as c_int) < extra {
            // It does not fit: delete and append instead. Append first,
            // because ml_delete_int cannot delete the last line of a
            // buffer, which is trouble for a buffer that has only one. The
            // mark has to come along.
            let marked = *db_index(dp).offset(idx as isize) & DB_MARKED != 0;
            ml_append_int(
                buf,
                lnum,
                new_line,
                new_len,
                if marked { ML_APPEND_MARK as c_int } else { 0 },
            );
            ml_delete_int(buf, lnum, 0);
            return;
        }

        let count = (*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low + 1;
        if extra != 0 && idx < count - 1 {
            // Move the text of the lines that follow, and adjust their
            // offsets. (Lines are stored back to front, so "following"
            // lines sit at *lower* offsets.)
            let txt_start = (*dp).db_txt_start as isize;
            core::ptr::copy(
                (dp as *mut c_char).offset(txt_start),
                (dp as *mut c_char).offset(txt_start - extra as isize),
                (start - (*dp).db_txt_start as c_int) as usize,
            );
            for i in idx + 1..count {
                let slot = db_index(dp).offset(i as isize);
                *slot = (*slot).wrapping_sub(extra as c_uint);
            }
        }
        let slot = db_index(dp).offset(idx as isize);
        *slot = (*slot).wrapping_sub(extra as c_uint);

        (*dp).db_free = (*dp).db_free.wrapping_sub(extra as c_uint);
        (*dp).db_txt_start = (*dp).db_txt_start.wrapping_sub(extra as c_uint);

        core::ptr::copy(
            new_line,
            old_line.offset(-(extra as isize)),
            new_len as usize,
        );
        (*buf).b_ml.ml_flags |= MlFlags::LOCKED_DIRTY | MlFlags::LOCKED_POS;
        // The `extra == 0` case is already covered by the insert and delete.
        if extra != 0 {
            ml_updatechunk(buf, lnum, extra, ML_CHNK_UPDLINE);
        }
    }
}

/// A new, empty data block of `page_count` pages.
///
/// `negative` asks the memfile for a negative block number, which is what an
/// as-yet-unwritten block gets.
///
/// # Safety
/// `mfp` must point at a memfile.
pub(crate) unsafe fn ml_new_data(
    mfp: *mut memfile_T,
    negative: bool,
    page_count: int64_t,
) -> *mut bhdr_T {
    debug_assert!(page_count >= 0);
    unsafe {
        let hp = mf_new(mfp, negative, page_count as c_uint);
        let dp = (*hp).bh_data as *mut DataBlock;
        (*dp).db_id = DATA_ID as uint16_t;
        (*dp).db_txt_end = (page_count as c_uint).wrapping_mul((*mfp).mf_page_size);
        (*dp).db_txt_start = (*dp).db_txt_end;
        (*dp).db_free = (*dp).db_txt_start.wrapping_sub(HEADER_SIZE as c_uint);
        (*dp).db_line_count = 0;
        hp
    }
}

/// A new, empty pointer block.
///
/// # Safety
/// `mfp` must point at a memfile.
pub(crate) unsafe fn ml_new_ptr(mfp: *mut memfile_T) -> *mut bhdr_T {
    unsafe {
        let hp = mf_new(mfp, false, 1);
        let pp = (*hp).bh_data as *mut PointerBlock;
        (*pp).pb_id = PTR_ID as uint16_t;
        (*pp).pb_count = 0;
        (*pp).pb_count_max = PointerBlock::count_max((*mfp).mf_page_size);
        hp
    }
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
/// been updated yet — except that while `ml_locked` is set,
/// `ml_locked_lineadd` still has to be added to it.
///
/// # Safety
/// `buf` must point at a buffer whose memline is open.
pub(crate) unsafe fn ml_find_line(buf: *mut buf_T, lnum: linenr_T, action: c_int) -> *mut bhdr_T {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;

        // If a block is locked, see whether the wanted line is in it. Not
        // for ML_FLUSH (the point of which is to release it), and not for
        // the actions that have to rebuild the stack.
        if !(*buf).b_ml.ml_locked.is_null() {
            if action & ML_SIMPLE != 0
                && (*buf).b_ml.ml_locked_low <= lnum
                && (*buf).b_ml.ml_locked_high >= lnum
            {
                // Remember to update the pointer blocks and the stack later.
                if action == ML_INSERT as c_int {
                    (*buf).b_ml.ml_locked_lineadd += 1;
                    (*buf).b_ml.ml_locked_high += 1;
                } else if action == ML_DELETE as c_int {
                    (*buf).b_ml.ml_locked_lineadd -= 1;
                    (*buf).b_ml.ml_locked_high -= 1;
                }
                return (*buf).b_ml.ml_locked;
            }

            mf_put(
                mfp,
                (*buf).b_ml.ml_locked,
                (*buf).b_ml.ml_flags.has(MlFlags::LOCKED_DIRTY),
                (*buf).b_ml.ml_flags.has(MlFlags::LOCKED_POS),
            );
            (*buf).b_ml.ml_locked = core::ptr::null_mut();

            // Lines were added or deleted in the block that was locked, so
            // the counts in the pointer blocks above it are stale.
            if (*buf).b_ml.ml_locked_lineadd != 0 {
                ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
            }
        }

        if action == ML_FLUSH as c_int {
            return core::ptr::null_mut(); // nothing else to do
        }

        let mut bnum: blocknr_T = 1; // start at the root of the tree
        let mut page_count: c_int = 1;
        let mut low: linenr_T = 1;
        let mut high: linenr_T = (*buf).b_ml.ml_line_count;

        if action == ML_FIND as c_int {
            // The previous walk's stack usually still covers this line —
            // reads come in runs. Restart from the deepest entry that does.
            let mut top = (*buf).b_ml.ml_stack_top - 1;
            while top >= 0 {
                let ip = (*buf).b_ml.ml_stack.offset(top as isize);
                if (*ip).ip_low <= lnum && (*ip).ip_high >= lnum {
                    bnum = (*ip).ip_bnum;
                    low = (*ip).ip_low;
                    high = (*ip).ip_high;
                    (*buf).b_ml.ml_stack_top = top; // truncate at the entry above
                    break;
                }
                top -= 1;
            }
            if top < 0 {
                (*buf).b_ml.ml_stack_top = 0; // not found, start at the root
            }
        } else {
            // ML_DELETE or ML_INSERT: the whole path has to be rewritten.
            (*buf).b_ml.ml_stack_top = 0;
        }

        // Search downwards until a data block is found.
        loop {
            let hp = mf_get(mfp, bnum, page_count as c_uint);
            if hp.is_null() {
                break;
            }

            if action == ML_INSERT as c_int {
                high += 1;
            } else if action == ML_DELETE as c_int {
                high -= 1;
            }

            let dp = (*hp).bh_data as *mut DataBlock;
            if (*dp).db_id as c_int == DATA_ID as c_int {
                (*buf).b_ml.ml_locked = hp;
                (*buf).b_ml.ml_locked_low = low;
                (*buf).b_ml.ml_locked_high = high;
                (*buf).b_ml.ml_locked_lineadd = 0;
                (*buf).b_ml.block_is_clean();
                return hp;
            }

            // Anything that is not a data block must be a pointer block.
            let pp = dp as *mut PointerBlock;
            if (*pp).pb_id as c_int != PTR_ID as c_int {
                iemsg(gettext(c"E317: Pointer block id wrong".as_ptr()));
                mf_put(mfp, hp, false, false);
                break;
            }

            // ml_add_stack may reallocate ml_stack, so `ip` has to be taken
            // after it and not held across another push.
            let top = ml_add_stack(buf);
            let ip = (*buf).b_ml.ml_stack.offset(top as isize);
            (*ip).ip_bnum = bnum;
            (*ip).ip_low = low;
            (*ip).ip_high = high;
            (*ip).ip_index = -1; // index not known yet

            let mut dirty = false;
            let count = (*pp).pb_count as c_int;
            let mut idx = 0;
            while idx < count {
                let entry = pb_entries(pp).offset(idx as isize);
                let t = (*entry).pe_line_count;
                low += t;
                if low > lnum {
                    (*ip).ip_index = idx;
                    bnum = (*entry).pe_bnum;
                    page_count = (*entry).pe_page_count;
                    high = low - 1;
                    low -= t;

                    // A negative block number is one that has not been
                    // written yet; it may since have been given a real one.
                    if bnum < 0 {
                        let bnum2 = mf_trans_del(mfp, bnum);
                        if bnum != bnum2 {
                            bnum = bnum2;
                            (*entry).pe_bnum = bnum;
                            dirty = true;
                        }
                    }
                    break;
                }
                idx += 1;
            }

            if idx >= count {
                // Past the end: the tree disagrees with the line count.
                if lnum > (*buf).b_ml.ml_line_count {
                    siemsg_c!(
                        gettext(c"E322: Line number out of range: %ld past the end".as_ptr()),
                        lnum as int64_t - (*buf).b_ml.ml_line_count as int64_t,
                    );
                } else {
                    siemsg_c!(
                        gettext(c"E323: Line count wrong in block %ld".as_ptr()),
                        bnum,
                    );
                }
                mf_put(mfp, hp, false, false);
                break;
            }

            let entry = pb_entries(pp).offset(idx as isize);
            if action == ML_DELETE as c_int {
                (*entry).pe_line_count -= 1;
                dirty = true;
            } else if action == ML_INSERT as c_int {
                (*entry).pe_line_count += 1;
                dirty = true;
            }
            mf_put(mfp, hp, dirty, false);
        }

        // The walk failed. For ML_DELETE/ML_INSERT the counts on the way
        // down were already adjusted for a line that will not be
        // inserted/deleted after all, so put them back.
        if action == ML_DELETE as c_int {
            ml_lineadd(buf, 1);
        } else if action == ML_INSERT as c_int {
            ml_lineadd(buf, -1);
        }
        (*buf).b_ml.ml_stack_top = 0;
        core::ptr::null_mut()
    }
}

/// Push an entry onto the info-pointer stack, growing it if needed, and
/// return its index.
///
/// # Safety
/// `buf` must point at a buffer.
pub(crate) unsafe fn ml_add_stack(buf: *mut buf_T) -> c_int {
    unsafe {
        let top = (*buf).b_ml.ml_stack_top;
        if top == (*buf).b_ml.ml_stack_size {
            (*buf).b_ml.ml_stack_size += STACK_INCR;
            let new_size = size_of::<infoptr_T>() * (*buf).b_ml.ml_stack_size as usize;
            (*buf).b_ml.ml_stack = xrealloc((*buf).b_ml.ml_stack.cast(), new_size).cast();
        }
        (*buf).b_ml.ml_stack_top += 1;
        top
    }
}

/// Add `count` (negative to subtract) to the line count of every pointer
/// block on the stack, and to the stack entries themselves.
///
/// This is the repair path: when an insert or delete fails part way down,
/// the pointer blocks have already been updated for it.
///
/// # Safety
/// `buf` must point at a buffer whose memline is open.
pub(crate) unsafe fn ml_lineadd(buf: *mut buf_T, count: c_int) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        let mut idx = (*buf).b_ml.ml_stack_top - 1;
        while idx >= 0 {
            let ip = (*buf).b_ml.ml_stack.offset(idx as isize);
            let hp = mf_get(mfp, (*ip).ip_bnum, 1);
            if hp.is_null() {
                break;
            }
            // Must be a pointer block: it is on the stack.
            let pp = (*hp).bh_data as *mut PointerBlock;
            if (*pp).pb_id as c_int != PTR_ID as c_int {
                mf_put(mfp, hp, false, false);
                iemsg(gettext(c"E317: Pointer block id wrong 2".as_ptr()));
                break;
            }
            let entry = pb_entries(pp).offset((*ip).ip_index as isize);
            (*entry).pe_line_count += count;
            (*ip).ip_high += count;
            mf_put(mfp, hp, true, false);
            idx -= 1;
        }
    }
}
