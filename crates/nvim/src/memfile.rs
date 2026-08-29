//! Blocks of memory that can be parked in a file — the swap file's
//! allocator, and the only thing that ever touches a `.swp` page.
//!
//! A memfile is a sequence of fixed-size pages. [`memline`] asks for a block
//! of one or more pages, fills it, and hands it back; the memfile decides
//! whether that block lives in memory, in the file, or both. Everything
//! above this file addresses blocks by number and never sees an offset.
//!
//! [`memline`]: crate::memline
//!
//! # Block numbers
//!
//! A non-negative block number *is* the page number in the file: block `n`
//! starts at `n * mf_page_size`. Negative numbers are blocks that have never
//! been written, handed out counting down from −1. When such a block finally
//! reaches the file it gets a fresh non-negative number, and the old-to-new
//! mapping is remembered in `mf_trans` until its one reader —
//! [`mf_trans_del`] — collects it. That is why a pointer block on disk can
//! still name a negative child: the parent was written first.
//!
//! # What lives where
//!
//! * `mf_used` — every block currently in memory, owned. Blocks are pinned
//!   ([`Box`]), because callers hold a `*mut bhdr_T` across further memfile
//!   calls; the table only ever moves the box, never the block.
//! * `mf_free` — page runs inside the file that no block uses any more, as
//!   a stack. Upstream threads these through the `bh_data` field of the
//!   headers it keeps alive; there is nothing in a free block but its
//!   number and length, so this keeps only that.
//! * `mf_trans` — the negative-to-positive renumbering above.
//!
//! Block *data* is still `xmalloc`ed rather than owned by a Rust container:
//! [`memline`] reinterprets `bh_data` as its own on-disk structs, which need
//! more alignment than a byte buffer promises. [`bhdr_T`]'s [`Drop`] is what
//! guarantees it is released exactly once.
//!
//! # Order is load-bearing
//!
//! `mf_used` iterates in insertion order and removal swaps the last block
//! into the hole — the same dense-array behaviour the khash-derived
//! `PMap(int64_t)` this replaced had. [`mf_sync`] writes blocks in that
//! order (and the order decides which block's bytes fill a gap in the file,
//! so it is visible in the swap file), [`mf_get`] re-appends the block it
//! hands out, and [`mf_release_all`] walks the table by index while removing
//! from it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::hash::{BuildHasherDefault, Hasher};
use std::collections::HashMap;
use std::ffi::CString;

use crate::fileio::{read_eintr, write_eintr};
use crate::main::{did_swapwrite_msg, e_swapclose, got_int, main_loop};
use crate::memline::{ml_get_buf, ml_open_file};
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::os::fs::{
    os_fileinfo_blocksize, os_fileinfo_fd, os_fileinfo_link, os_fsync, os_open, os_remove,
    os_set_cloexec,
};
use crate::os::input::{os_breakcheck, os_char_avail};
use crate::path::full_name_save;
use crate::types::{FileInfo, blocknr_T, buf_T, off_T};
use crate::winlayer::buffers;
use ::libc::{__errno_location, close, lseek, strerror};

/// A swap-file operation that did not complete.
///
/// It carries nothing because there is nothing left to say: every reason
/// worth a message has already produced one (E294 seek-on-read, E295 read,
/// E296 seek-on-write, E297 write, and `mf_close`'s E314 for the descriptor).
/// The two silent reasons are the same answer for the same purpose — there
/// is no swap file, so the block cannot go anywhere and the caller must keep
/// it in memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SwapFailed;

/// Whether a memfile has blocks that are not on disk, and whether they may
/// be written yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum MfDirty {
    /// Everything is on disk.
    No,
    /// Something is not, and `ml_sync_all` should write it.
    Yes,
    /// Something is not, but writing it now would put a half-finished
    /// change in the swap file — `:preserve` clears this first.
    YesNoSync,
}

/// Sync every block, not just the ones with a file block number.
pub(crate) const MFS_ALL: c_int = 1;
/// Stop as soon as a character is typed (but write at least one block).
pub(crate) const MFS_STOP: c_int = 2;
/// Push the file to the platter, so it survives a crash.
pub(crate) const MFS_FLUSH: c_int = 4;
/// Write block zero and nothing else.
pub(crate) const MFS_ZERO: c_int = 8;

/// The block is locked in memory: a caller is holding it.
pub(crate) const BH_LOCKED: c_uint = 2;
/// The block has changed since it was last written.
pub(crate) const BH_DIRTY: c_uint = 1;

/// Page size to use when the device will not say.
pub(crate) const MEMFILE_PAGE_SIZE: c_uint = 4096;
/// A device block size outside this range is not believed.
pub(crate) const MIN_SWAP_PAGE_SIZE: u64 = 1048;
pub(crate) const MAX_SWAP_PAGE_SIZE: u64 = 50000;

const O_RDWR: c_int = 0o2;
const O_CREAT: c_int = 0o100;
const O_EXCL: c_int = 0o200;
const O_TRUNC: c_int = 0o1000;
const O_NOFOLLOW: c_int = 0o400000;
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;
/// `S_IREAD | S_IWRITE`: the mode a swap file is created with.
const SWAPFILE_MODE: c_int = 0o400 | 0o200;

/// One block: a run of `bh_page_count` pages, and the memory holding them.
///
/// `bh_data` is an `xmalloc`ed buffer of `page_count * mf_page_size` bytes
/// that [`memline`](crate::memline) casts to its own block
/// structs. It is released by [`Drop`], so a block is freed by dropping the
/// box that owns it and no other way.
pub struct bhdr_T {
    /// The block number, which is also the key it is filed under.
    pub bh_bnum: blocknr_T,
    pub bh_data: *mut c_void,
    pub bh_page_count: c_uint,
    /// [`BH_DIRTY`] and/or [`BH_LOCKED`].
    pub bh_flags: c_uint,
}

impl bhdr_T {
    /// A block of `page_count` pages, zeroed.
    ///
    /// The zeroing is not just hygiene: a page written to the swap file
    /// straight from `malloc` would carry whatever the last owner of that
    /// memory held — up to and including the contents of a file the user
    /// read earlier in the session.
    fn new(page_size: c_uint, page_count: c_uint) -> Box<Self> {
        let bytes = page_size as usize * page_count as usize;
        // SAFETY: xmalloc either answers `bytes` writable bytes or exits,
        // and the buffer is untyped, so any bit pattern is valid in it.
        let data = unsafe {
            let data = xmalloc(bytes);
            data.write_bytes(0, bytes);
            data
        };
        Box::new(bhdr_T {
            bh_bnum: 0,
            bh_data: data,
            bh_page_count: page_count,
            bh_flags: 0,
        })
    }
}

impl Drop for bhdr_T {
    fn drop(&mut self) {
        // SAFETY: `bh_data` came from `xmalloc` in `new` and nothing else
        // frees it — the field is never reassigned.
        unsafe { xfree(self.bh_data) };
    }
}

/// A run of pages in the file that no block uses.
#[derive(Clone, Copy)]
struct FreeBlock {
    bnum: blocknr_T,
    page_count: c_uint,
}

/// Mixes a block number so the top bits — which the hash table probes
/// first — depend on all of it. Block numbers are small and consecutive,
/// which the identity hash spreads badly and SipHash costs too much for.
#[derive(Default)]
struct BlockNrHasher(u64);

impl Hasher for BlockNrHasher {
    fn write_i64(&mut self, n: i64) {
        // splitmix64's finalizer.
        let mut z = n as u64;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.0 = z ^ (z >> 31);
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_i64((self.0 as i64).wrapping_mul(31).wrapping_add(b as i64));
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

/// The blocks that are in memory, in insertion order, found by number.
///
/// See the module docs: both the order and the swap-into-the-hole removal
/// are observable, so this is an index map rather than a plain [`HashMap`].
#[derive(Default)]
struct BlockTable {
    // Boxed because a block's address escapes: every caller works through the
    // `*mut bhdr_T` this hands out, and those must survive the vector growing
    // or swapping entries around.
    #[allow(clippy::vec_box)]
    blocks: Vec<Box<bhdr_T>>,
    index: HashMap<blocknr_T, u32, BuildHasherDefault<BlockNrHasher>>,
}

impl BlockTable {
    fn len(&self) -> usize {
        self.blocks.len()
    }

    /// The `i`-th block in iteration order. Callers walk by index because
    /// the table can change under them.
    #[inline]
    fn at(&mut self, i: usize) -> *mut bhdr_T {
        &raw mut *self.blocks[i]
    }

    #[inline]
    fn get(&mut self, nr: blocknr_T) -> Option<*mut bhdr_T> {
        let i = *self.index.get(&nr)? as usize;
        Some(&raw mut *self.blocks[i])
    }

    /// File `block` under its own number, at the end of the order.
    #[inline]
    fn insert(&mut self, block: Box<bhdr_T>) -> *mut bhdr_T {
        let nr = block.bh_bnum;
        self.blocks.push(block);
        let i = self.blocks.len() - 1;
        let previous = self.index.insert(nr, i as u32);
        debug_assert!(previous.is_none(), "memfile: block {nr} filed twice");
        &raw mut *self.blocks[i]
    }

    /// Take the block numbered `nr` out, moving the last one into its slot.
    #[inline]
    fn remove(&mut self, nr: blocknr_T) -> Option<Box<bhdr_T>> {
        let i = self.index.remove(&nr)? as usize;
        let block = self.blocks.swap_remove(i);
        if let Some(moved) = self.blocks.get(i) {
            self.index.insert(moved.bh_bnum, i as u32);
        }
        Some(block)
    }
}

/// A memory file: the blocks of one buffer's swap file, and the file.
pub struct memfile_T {
    /// Name of the swap file, as given, or `None` for memory only. Private:
    /// readers go through [`mf_fname`], which is all any of them need.
    fname: Option<CString>,
    /// The same name, as a full path. Only [`mf_fullname`] ever reads it.
    ffname: Option<CString>,
    /// Descriptor of the open swap file, or −1 for memory only.
    pub mf_fd: c_int,
    /// The flags `mf_fd` was opened with, for reopening it.
    pub mf_flags: c_int,
    /// `mf_fd` was lost and should be reopened on the next write.
    pub mf_reopen: bool,
    /// Blocks in memory. See the module docs.
    used: BlockTable,
    /// Unused page runs in the file, most recently freed last.
    free: Vec<FreeBlock>,
    /// Negative block number to the file block number it was given.
    trans: HashMap<blocknr_T, blocknr_T, BuildHasherDefault<BlockNrHasher>>,
    /// Highest file block number handed out, plus one.
    pub mf_blocknr_max: blocknr_T,
    /// Lowest memory-only block number handed out, minus one.
    pub mf_blocknr_min: blocknr_T,
    /// How many memory-only block numbers are outstanding.
    pub mf_neg_count: blocknr_T,
    /// How many pages the file holds.
    pub mf_infile_count: blocknr_T,
    pub mf_page_size: c_uint,
    pub mf_dirty: MfDirty,
}

/// Open a new or existing memory file.
///
/// `fname` is the swap file to use, or null for memory only. It must be
/// allocated, and is consumed either way — including when opening fails,
/// which answers null.
pub(crate) unsafe fn mf_open(fname: *mut c_char, flags: c_int) -> *mut memfile_T {
    let mfp = Box::into_raw(Box::new(memfile_T {
        fname: None,
        ffname: None,
        mf_fd: -1,
        mf_flags: 0,
        mf_reopen: false,
        used: BlockTable::default(),
        free: Vec::new(),
        trans: HashMap::default(),
        mf_blocknr_max: 0,
        mf_blocknr_min: -1,
        mf_neg_count: 0,
        mf_infile_count: 0,
        mf_page_size: MEMFILE_PAGE_SIZE,
        mf_dirty: MfDirty::No,
    }));

    unsafe {
        if !fname.is_null() && !mf_do_open(mfp, fname, flags) {
            drop(Box::from_raw(mfp));
            return core::ptr::null_mut();
        }

        // Match the page size to the device's block size when we can: it
        // makes every read and write a whole number of device blocks.
        let mut file_info: FileInfo = core::mem::zeroed();
        if (*mfp).mf_fd >= 0 && os_fileinfo_fd((*mfp).mf_fd, &raw mut file_info) {
            let blocksize = os_fileinfo_blocksize(&raw mut file_info);
            if (MIN_SWAP_PAGE_SIZE..=MAX_SWAP_PAGE_SIZE).contains(&blocksize) {
                (*mfp).mf_page_size = blocksize as c_uint;
            }
        }

        // When recovering, the real page size comes out of block zero later
        // (`ml_recover` calls `mf_new_page_size`), so the size used here may
        // be too small and the block count is rounded up.
        let size = if (*mfp).mf_fd < 0 || flags & (O_TRUNC | O_EXCL) != 0 {
            0
        } else {
            lseek((*mfp).mf_fd, 0, SEEK_END) as off_T
        };
        (*mfp).mf_blocknr_max = if size <= 0 {
            0
        } else {
            assert!(
                (*mfp).mf_page_size > 0 && (*mfp).mf_page_size as off_T - 1 <= off_T::MAX - size,
                "memfile: swap file too large for its page size"
            );
            (size + (*mfp).mf_page_size as blocknr_T - 1) / (*mfp).mf_page_size as blocknr_T
        };
        (*mfp).mf_infile_count = (*mfp).mf_blocknr_max;
    }

    mfp
}

/// Give an existing memory file a swap file, as `'updatecount'` going from
/// zero to non-zero does. `fname` is consumed as in [`mf_open`].
pub(crate) unsafe fn mf_open_file(
    mfp: *mut memfile_T,
    fname: *mut c_char,
) -> Result<(), SwapFailed> {
    unsafe {
        if mf_do_open(mfp, fname, O_RDWR | O_CREAT | O_EXCL) {
            (*mfp).mf_dirty = MfDirty::Yes;
            Ok(())
        } else {
            Err(SwapFailed)
        }
    }
}

/// Close a memory file, releasing every block, and delete the swap file if
/// `del_file`.
pub(crate) unsafe fn mf_close(mfp: *mut memfile_T, del_file: bool) {
    if mfp.is_null() {
        return;
    }
    unsafe {
        if (*mfp).mf_fd >= 0 && close((*mfp).mf_fd) < 0 {
            emsg(gettext(e_swapclose));
        }
        if del_file && !mf_fname(mfp).is_null() {
            os_remove(mf_fname(mfp));
        }
        mf_free_fnames(mfp);
        // Dropping the memfile drops every block it still owns.
        drop(Box::from_raw(mfp));
    }
}

/// Close and delete the swap file of `buf`, keeping the memory file. Used
/// when `'swapfile'` is reset.
///
/// `getlines` first pulls every line into memory — clumsy, but the blocks
/// still in the file are about to become unreachable.
pub(crate) unsafe fn mf_close_file(buf: *mut buf_T, getlines: bool) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() || (*mfp).mf_fd < 0 {
            return;
        }

        if getlines {
            for lnum in 1..=(*buf).b_ml.ml_line_count {
                ml_get_buf(buf, lnum);
            }
        }

        if close((*mfp).mf_fd) < 0 {
            emsg(gettext(e_swapclose));
        }
        (*mfp).mf_fd = -1;

        if !mf_fname(mfp).is_null() {
            os_remove(mf_fname(mfp));
            mf_free_fnames(mfp);
        }
    }
}

/// Set the page size, once block zero of an existing swap file has said what
/// it really is.
pub(crate) unsafe fn mf_new_page_size(mfp: *mut memfile_T, new_size: c_uint) {
    unsafe { (*mfp).mf_page_size = new_size };
}

/// Allocate a new block of `page_count` pages and lock it.
///
/// `negative` asks for a memory-only block number, which is what data
/// blocks get: they are written last, so their numbers are handed out last.
pub(crate) unsafe fn mf_new(
    mfp: *mut memfile_T,
    negative: bool,
    page_count: c_uint,
) -> *mut bhdr_T {
    unsafe {
        let page_size = (*mfp).mf_page_size;
        let mut block;
        // Reuse a free run if one is long enough, otherwise take the next
        // number from whichever end this block belongs to.
        if let Some(free) = (*mfp).free.last_mut()
            && !negative
            && free.page_count >= page_count
        {
            block = bhdr_T::new(page_size, page_count);
            block.bh_bnum = free.bnum;
            if free.page_count > page_count {
                free.bnum += page_count as blocknr_T;
                free.page_count -= page_count;
            } else {
                (*mfp).free.pop();
            }
        } else {
            block = bhdr_T::new(page_size, page_count);
            if negative {
                block.bh_bnum = (*mfp).mf_blocknr_min;
                (*mfp).mf_blocknr_min -= 1;
                (*mfp).mf_neg_count += 1;
            } else {
                block.bh_bnum = (*mfp).mf_blocknr_max;
                (*mfp).mf_blocknr_max += page_count as blocknr_T;
            }
        }

        block.bh_flags = BH_LOCKED | BH_DIRTY; // a new block is always dirty
        (*mfp).mf_dirty = MfDirty::Yes;
        (*mfp).used.insert(block)
    }
}

/// Find block `nr`, reading it from the file if it is not in memory, and
/// lock it. Answers null if there is no such block.
///
/// A negative `nr` must go through [`mf_trans_del`] first.
pub(crate) unsafe fn mf_get(mfp: *mut memfile_T, nr: blocknr_T, page_count: c_uint) -> *mut bhdr_T {
    unsafe {
        if nr >= (*mfp).mf_blocknr_max || nr <= (*mfp).mf_blocknr_min {
            return core::ptr::null_mut();
        }

        // Taking it out and putting it back is what moves the block to the
        // end of the sync order.
        let mut block = match (*mfp).used.remove(nr) {
            Some(block) => block,
            None => {
                // Not in memory. Only a block the file holds can be read
                // back; a memory-only one is simply gone.
                if nr < 0 || nr >= (*mfp).mf_infile_count || page_count == 0 {
                    return core::ptr::null_mut();
                }
                let mut block = bhdr_T::new((*mfp).mf_page_size, page_count);
                block.bh_bnum = nr;
                if mf_read(mfp, &mut block).is_err() {
                    return core::ptr::null_mut();
                }
                block
            }
        };

        block.bh_flags |= BH_LOCKED;
        (*mfp).used.insert(block)
    }
}

/// The block numbered `nr` if it is already in memory, or null.
///
/// Unlike [`mf_get`] this neither reads the file, nor locks the block, nor
/// moves it in the sync order. `ml_setflags` uses it to amend block zero
/// where it lies.
pub(crate) unsafe fn mf_find(mfp: *mut memfile_T, nr: blocknr_T) -> *mut bhdr_T {
    unsafe { (*mfp).used.get(nr).unwrap_or(core::ptr::null_mut()) }
}

/// Release a block held by [`mf_get`] or [`mf_new`].
///
/// `dirty` says it was changed and has to reach the file; `infile` asks for
/// its file block number to be settled now, which recovery needs so that a
/// block already on disk can name it.
pub(crate) unsafe fn mf_put(mfp: *mut memfile_T, hp: *mut bhdr_T, dirty: bool, infile: bool) {
    unsafe {
        let mut flags = (*hp).bh_flags;
        if flags & BH_LOCKED == 0 {
            iemsg(gettext(c"E293: Block was not locked"));
        }
        flags &= !BH_LOCKED;
        if dirty {
            flags |= BH_DIRTY;
            if (*mfp).mf_dirty != MfDirty::YesNoSync {
                (*mfp).mf_dirty = MfDirty::Yes;
            }
        }
        (*hp).bh_flags = flags;
        if infile {
            mf_trans_add(mfp, hp);
        }
    }
}

/// Give up a block for good. Its pages in the file, if it has any, go back
/// on the free list.
pub(crate) unsafe fn mf_free(mfp: *mut memfile_T, hp: *mut bhdr_T) {
    unsafe {
        let bnum = (*hp).bh_bnum;
        let page_count = (*hp).bh_page_count;
        // Dropping the block frees its data; `hp` dangles from here on.
        let block = (*mfp).used.remove(bnum);
        debug_assert!(block.is_some(), "memfile: freeing a block not in memory");
        drop(block);
        if bnum < 0 {
            // Memory-only numbers are never reused, so they do not belong
            // on the free list.
            (*mfp).mf_neg_count -= 1;
        } else {
            (*mfp).free.push(FreeBlock { bnum, page_count });
        }
    }
}

/// Write out every dirty block, newest first.
///
/// `flags` is a set of [`MFS_ALL`], [`MFS_STOP`], [`MFS_FLUSH`] and
/// [`MFS_ZERO`]. Fails when there is no file or a write failed — which on a
/// full disk is the common case, so after the first failure only blocks that
/// already have a place in the file are attempted.
pub(crate) unsafe fn mf_sync(mfp: *mut memfile_T, flags: c_int) -> Result<(), SwapFailed> {
    unsafe {
        let got_int_save = got_int.get();

        if (*mfp).mf_fd < 0 {
            (*mfp).mf_dirty = MfDirty::No;
            return Err(SwapFailed);
        }

        // Only a CTRL-C typed *while* writing interrupts this, not one
        // typed earlier.
        got_int.set(false);

        // Last to first, which makes a half-written file likelier to be
        // consistent. The last block is typically early in the table.
        let mut status = Ok(());
        let mut visited = false;
        let mut i = 0;
        while i < (*mfp).used.len() {
            let hp = (*mfp).used.at(i);
            visited = true;
            let syncable = (flags & MFS_ALL != 0 || (*hp).bh_bnum >= 0)
                && (*hp).bh_flags & BH_DIRTY != 0
                && (status.is_ok()
                    || ((*hp).bh_bnum >= 0 && (*hp).bh_bnum < (*mfp).mf_infile_count));
            if syncable && !(flags & MFS_ZERO != 0 && (*hp).bh_bnum != 0) {
                if mf_write(mfp, hp).is_err() {
                    if status.is_err() {
                        break; // a second failure: give up
                    }
                    status = Err(SwapFailed);
                }
                if flags & MFS_STOP != 0 {
                    if os_char_avail() {
                        break;
                    }
                } else if (*main_loop.ptr()).recursive == 0 {
                    // May be reached on OOM inside a libuv callback, where
                    // the event loop must not be re-entered.
                    os_breakcheck();
                }
                if got_int.get() {
                    break;
                }
            }
            i += 1;
        }

        // Everything written means nothing is dirty. So does a failure: the
        // flag is cleared to stop us retrying on every keystroke.
        if !visited || status.is_err() {
            (*mfp).mf_dirty = MfDirty::No;
        }

        if flags & MFS_FLUSH != 0 && os_fsync((*mfp).mf_fd) != 0 {
            status = Err(SwapFailed);
        }

        got_int.set(got_int.get() || got_int_save);
        status
    }
}

/// Mark every block that has a place in the file dirty, so a freshly
/// created swap file gets all of them.
pub(crate) unsafe fn mf_set_dirty(mfp: *mut memfile_T) {
    unsafe {
        for i in 0..(*mfp).used.len() {
            let hp = (*mfp).used.at(i);
            if (*hp).bh_bnum > 0 {
                (*hp).bh_flags |= BH_DIRTY;
            }
        }
        (*mfp).mf_dirty = MfDirty::Yes;
    }
}

/// Drop as many cached blocks as possible, for when memory has run out.
/// Answers whether anything was released.
pub(crate) unsafe fn mf_release_all() -> bool {
    let mut released = false;
    for buf in buffers() {
        let mfp = buf.b_ml.ml_mfp;
        if mfp.is_null() {
            continue;
        }
        // SAFETY: the buffer is a live one from the editor's own list, and
        // `mfp` is the memfile it owns.
        unsafe {
            // Nothing can be released without somewhere to put it.
            if (*mfp).mf_fd < 0 && buf.b_may_swap {
                ml_open_file(buf.raw());
            }

            if (*mfp).mf_fd >= 0 {
                let mut i = 0;
                while i < (*mfp).used.len() {
                    let hp = (*mfp).used.at(i);
                    if (*hp).bh_flags & BH_LOCKED == 0
                        && ((*hp).bh_flags & BH_DIRTY == 0 || mf_write(mfp, hp).is_ok())
                    {
                        // Dropping it releases the block's pages.
                        drop((*mfp).used.remove((*hp).bh_bnum));
                        released = true;
                        // Stay at `i`: removal moved another block into
                        // this slot (or that was the last one).
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
    released
}

/// Read a block's pages from the file.
unsafe fn mf_read(mfp: *mut memfile_T, hp: &mut bhdr_T) -> Result<(), SwapFailed> {
    unsafe {
        if (*mfp).mf_fd < 0 {
            return Err(SwapFailed); // there is no file to read
        }

        let page_size = (*mfp).mf_page_size;
        let offset = (page_size as blocknr_T * hp.bh_bnum) as off_T;
        if lseek((*mfp).mf_fd, offset, SEEK_SET) != offset {
            perror_msg(c"E294: Seek error in swap file read");
            return Err(SwapFailed);
        }
        assert!(
            hp.bh_page_count <= c_uint::MAX / page_size,
            "memfile: block longer than the address space"
        );
        let size = page_size * hp.bh_page_count;
        if read_eintr((*mfp).mf_fd, hp.bh_data, size as usize) as c_uint != size {
            perror_msg(c"E295: Read error in swap file");
            return Err(SwapFailed);
        }

        Ok(())
    }
}

/// Write a block's pages to the file, extending it as needed.
///
/// The file must have no holes, so writing a block past the end first fills
/// the space in front of it — with the blocks that belong there, or, where
/// one of those has been freed, with a copy of this block's bytes as
/// filler.
unsafe fn mf_write(mfp: *mut memfile_T, hp: *mut bhdr_T) -> Result<(), SwapFailed> {
    unsafe {
        if (*mfp).mf_fd < 0 && !(*mfp).mf_reopen {
            return Err(SwapFailed); // there is no file and there never was
        }

        // A memory-only block must be given its place in the file first.
        if (*hp).bh_bnum < 0 {
            mf_trans_add(mfp, hp);
        }

        let page_size = (*mfp).mf_page_size;
        loop {
            let mut nr = (*hp).bh_bnum;
            let hp2 = if nr > (*mfp).mf_infile_count {
                nr = (*mfp).mf_infile_count;
                // Null when the block that belongs here was freed.
                (*mfp).used.get(nr).unwrap_or(core::ptr::null_mut())
            } else {
                hp
            };

            let offset = (page_size as blocknr_T * nr) as off_T;
            let page_count = if hp2.is_null() {
                1
            } else {
                (*hp2).bh_page_count
            };
            // Upstream lets this wrap, and a line long enough to overflow it
            // would have overflowed `ml_append_int`'s arithmetic first.
            let size = page_size.wrapping_mul(page_count);

            for attempt in 1..=2 {
                if (*mfp).mf_fd >= 0 {
                    if lseek((*mfp).mf_fd, offset, SEEK_SET) != offset {
                        perror_msg(c"E296: Seek error in swap file write");
                        return Err(SwapFailed);
                    }
                    let data = if hp2.is_null() {
                        (*hp).bh_data
                    } else {
                        (*hp2).bh_data
                    };
                    if write_eintr((*mfp).mf_fd, data, size as usize) as c_uint == size {
                        break;
                    }
                }

                if attempt == 1 {
                    // A swap file on a network drive survives the
                    // connection dropping if we reopen it.
                    if (*mfp).mf_fd >= 0 {
                        close((*mfp).mf_fd);
                    }
                    (*mfp).mf_fd = os_open(mf_fname(mfp), (*mfp).mf_flags, SWAPFILE_MODE);
                    (*mfp).mf_reopen = (*mfp).mf_fd < 0;
                }
                if attempt == 2 || (*mfp).mf_fd < 0 {
                    // Usually a full disk. Keep trying in case space turns
                    // up, but say so only once, until a write succeeds or
                    // the user hits a key.
                    if !did_swapwrite_msg.get() {
                        emsg(gettext(c"E297: Write error in swap file"));
                    }
                    did_swapwrite_msg.set(true);
                    return Err(SwapFailed);
                }
            }

            did_swapwrite_msg.set(false);
            if !hp2.is_null() {
                (*hp2).bh_flags &= !BH_DIRTY; // wrote a real block, not filler
            }
            if nr + page_count as blocknr_T > (*mfp).mf_infile_count {
                (*mfp).mf_infile_count = nr + page_count as blocknr_T;
            }
            if nr == (*hp).bh_bnum {
                break; // wrote the block we came for
            }
        }

        Ok(())
    }
}

/// Give a memory-only block a place in the file, and remember the
/// renumbering for [`mf_trans_del`].
///
/// Cannot fail: a number is always available, and the page run it comes from
/// is either recycled off the free list or taken past the end of the file.
/// Upstream returned `OK`/`FAIL` here and never answered `FAIL`.
unsafe fn mf_trans_add(mfp: *mut memfile_T, hp: *mut bhdr_T) {
    unsafe {
        if (*hp).bh_bnum >= 0 {
            return; // already has one
        }

        // Reuse a free run if one is long enough, as `mf_new` does.
        let page_count = (*hp).bh_page_count;
        let new_bnum;
        if let Some(free) = (*mfp).free.last_mut()
            && free.page_count >= page_count
        {
            new_bnum = free.bnum;
            if free.page_count > page_count {
                free.bnum += page_count as blocknr_T;
                free.page_count -= page_count;
            } else {
                (*mfp).free.pop();
            }
        } else {
            new_bnum = (*mfp).mf_blocknr_max;
            (*mfp).mf_blocknr_max += page_count as blocknr_T;
        }

        let old_bnum = (*hp).bh_bnum;
        // Refiling moves the box, never the block, so `hp` stays good.
        let mut block = (*mfp)
            .used
            .remove(old_bnum)
            .expect("memfile: renumbering a block that is not in memory");
        block.bh_bnum = new_bnum;
        (*mfp).used.insert(block);
        (*mfp).trans.insert(old_bnum, new_bnum);
    }
}

/// The file block number a memory-only block was given, consuming the
/// record of it. Answers `old_nr` unchanged if there is none.
pub(crate) unsafe fn mf_trans_del(mfp: *mut memfile_T, old_nr: blocknr_T) -> blocknr_T {
    unsafe {
        match (*mfp).trans.remove(&old_nr) {
            Some(new_bnum) => {
                (*mfp).mf_neg_count -= 1;
                new_bnum
            }
            None => old_nr,
        }
    }
}

/// The swap file's name as it was given, or null when the memfile is memory
/// only. It stays valid until the name is changed or the memfile closed.
///
/// # Safety
/// `mfp` must point at a memfile.
pub(crate) unsafe fn mf_fname(mfp: *const memfile_T) -> *const c_char {
    match unsafe { &(*mfp).fname } {
        Some(fname) => fname.as_ptr(),
        None => core::ptr::null(),
    }
}

/// Take over an allocated C string, which is released.
unsafe fn take_cstring(p: *mut c_char) -> CString {
    unsafe {
        let owned = CStr::from_ptr(p).to_owned();
        xfree(p.cast::<c_void>());
        owned
    }
}

/// Release the swap file's names.
pub(crate) unsafe fn mf_free_fnames(mfp: *mut memfile_T) {
    unsafe {
        (*mfp).fname = None;
        (*mfp).ffname = None;
    }
}

/// Name the swap file. `fname` must be allocated, and is consumed.
///
/// Only called when creating or renaming it, so the full path is always
/// worked out afresh.
pub(crate) unsafe fn mf_set_fnames(mfp: *mut memfile_T, fname: *mut c_char) {
    unsafe {
        let full = full_name_save(fname, false);
        (*mfp).fname = Some(take_cstring(fname));
        (*mfp).ffname = (!full.is_null()).then(|| take_cstring(full));
    }
}

/// Make the swap file's name absolute — before a `:cd` makes the relative
/// one mean something else.
pub(crate) unsafe fn mf_fullname(mfp: *mut memfile_T) {
    unsafe {
        if mfp.is_null() || (*mfp).fname.is_none() || (*mfp).ffname.is_none() {
            return;
        }
        (*mfp).fname = (*mfp).ffname.take();
    }
}

/// Whether any block still owes the file a number.
pub(crate) unsafe fn mf_need_trans(mfp: *mut memfile_T) -> bool {
    unsafe { (*mfp).fname.is_some() && (*mfp).mf_neg_count > 0 }
}

/// Open the swap file. `fname` must be allocated, and is consumed — also
/// when this fails, in which case the memfile stays memory-only.
unsafe fn mf_do_open(mfp: *mut memfile_T, fname: *mut c_char, mut flags: c_int) -> bool {
    unsafe {
        // `fname` has to have been allocated.
        mf_set_fnames(mfp, fname);
        debug_assert!(!mf_fname(mfp).is_null());

        // A swap file being created really should not exist yet. If it does
        // and it is a symlink, this is most likely an attack.
        let mut file_info: FileInfo = core::mem::zeroed();
        if flags & O_CREAT != 0 && os_fileinfo_link(mf_fname(mfp), &raw mut file_info) {
            (*mfp).mf_fd = -1;
            emsg(gettext(c"E300: Swap file already exists (symlink attack?)"));
        } else {
            flags |= O_NOFOLLOW;
            (*mfp).mf_flags = flags;
            (*mfp).mf_fd = os_open(mf_fname(mfp), flags, SWAPFILE_MODE);
        }

        if (*mfp).mf_fd < 0 {
            mf_free_fnames(mfp);
            return false;
        }

        os_set_cloexec((*mfp).mf_fd);
        true
    }
}

/// `PERROR`: an error message with the failing call's `strerror` after it.
unsafe fn perror_msg(message: &'static CStr) {
    unsafe {
        semsg_c!(
            c"%s: %s".as_ptr(),
            gettext(message).as_ptr(),
            strerror(*__errno_location()),
        );
    }
}
