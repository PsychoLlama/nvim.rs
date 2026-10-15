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
//! * `used` — every block currently in memory, owned. A block is reached by
//!   a [`BlockId`], a handle that stays good while the block is in memory —
//!   across further [`MemFile::get`]s, across renumbering, across the table
//!   reordering itself — and that names *no* block once it has been freed or
//!   released: using it then is a panic, not a read of whatever took its
//!   place. Callers that hold a block across calls hold the handle, never
//!   the bytes.
//! * `free` — page runs inside the file that no block uses any more, as
//!   a stack. Upstream threads these through the `bh_data` field of the
//!   headers it keeps alive; there is nothing in a free block but its
//!   number and length, so this keeps only that.
//! * `trans` — the negative-to-positive renumbering above.
//!
//! A block's pages are a `Box<[u8]>`, and [`MemFile::page`] and
//! [`MemFile::page_mut`] are how anyone sees them: as bytes, which
//! [`memline`]'s typed views read and write field by field. Nothing outside
//! this file is handed a page's address.
//!
//! # Order is load-bearing
//!
//! `used` iterates in insertion order and removal swaps the last block
//! into the hole — the same dense-array behaviour the khash-derived
//! `PMap(int64_t)` this replaced had. [`MemFile::sync`] writes blocks in that
//! order (and the order decides which block's bytes fill a gap in the file,
//! so it is visible in the swap file), [`MemFile::get`] moves the block it
//! hands out to the end, and [`mf_release_all`] walks the table by index
//! while removing from it.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memfile.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod swapfile;
mod table;
use swapfile::perror_msg;
pub(crate) use swapfile::*;
pub(crate) use table::BlockHdr;
pub(crate) use table::BlockId;
use table::BlockTable;

use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint};
use core::hash::{BuildHasherDefault, Hasher};
use std::collections::HashMap;
use std::ffi::CString;

use crate::event::r#loop::main_loop_is_recursing;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::memline::{ml_get_buf, ml_open_file};
use crate::message::e_swapclose;
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::os::fs::{os_fileinfo_fd, os_fsync, os_open, os_remove};
use crate::os::fs::{read_eintr, write_eintr};
use crate::os::input::{os_breakcheck, os_char_avail};
use crate::types::{BlockNr, FileInfo, FileOffset};
use crate::winlayer::buffers;
use ::libc::{close, lseek};

pub(crate) static did_swapwrite_msg: GlobalCell<bool> = GlobalCell::new(false);

/// A swap-file operation that did not complete.
///
/// It carries nothing because there is nothing left to say: every reason
/// worth a message has already produced one (E294 seek-on-read, E295 read,
/// E296 seek-on-write, E297 write, and `close`'s E314 for the descriptor).
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
const BH_LOCKED: c_uint = 2;
/// The block has changed since it was last written.
const BH_DIRTY: c_uint = 1;

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

/// A run of pages in the file that no block uses.
struct FreeBlock {
    bnum: BlockNr,
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

type BlockMap<V> = HashMap<BlockNr, V, BuildHasherDefault<BlockNrHasher>>;

/// A memory file: the blocks of one buffer's swap file, and the file.
///
/// Owned by the buffer whose lines it holds (`MemLine::ml_mfp`), and
/// dropped by [`MemFile::close`], which is the only way one ends.
pub struct MemFile {
    /// Name of the swap file, as given, or `None` for memory only. Private:
    /// readers go through [`MemFile::fname`], which is all any of them need.
    fname: Option<CString>,
    /// The same name, as a full path. Only [`MemFile::make_name_full`]
    /// ever reads it.
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
    trans: BlockMap<BlockNr>,
    /// Highest file block number handed out, plus one.
    pub mf_blocknr_max: BlockNr,
    /// Lowest memory-only block number handed out, minus one.
    pub mf_blocknr_min: BlockNr,
    /// How many memory-only block numbers are outstanding.
    pub mf_neg_count: BlockNr,
    /// How many pages the file holds.
    pub mf_infile_count: BlockNr,
    pub mf_page_size: c_uint,
    pub mf_dirty: MfDirty,
}

impl MemFile {
    /// A memory-only memfile, with no swap file: what every memline starts
    /// as (upstream's `mf_open(NULL, 0)`, which cannot fail).
    pub(crate) fn in_memory() -> Box<MemFile> {
        Box::new(MemFile {
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
        })
    }

    /// Open an existing swap file, or create one, as upstream's `mf_open`
    /// with a name: `None` when it cannot be opened.
    pub(crate) fn open(fname: CString, flags: c_int) -> Option<Box<MemFile>> {
        let mut memfile = MemFile::in_memory();
        if !memfile.do_open(fname, flags) {
            return None;
        }

        // Match the page size to the device's block size when we can: it
        // makes every read and write a whole number of device blocks.
        // SAFETY: a plain-old-data out-parameter of this frame, and the
        // memfile's own descriptor.
        let mut file_info: FileInfo = unsafe { core::mem::zeroed() };
        if memfile.mf_fd >= 0 && unsafe { os_fileinfo_fd(memfile.mf_fd, &raw mut file_info) } {
            let blocksize = file_info.stat.st_blksize;
            if (MIN_SWAP_PAGE_SIZE..=MAX_SWAP_PAGE_SIZE).contains(&blocksize) {
                memfile.mf_page_size = blocksize as c_uint;
            }
        }

        // When recovering, the real page size comes out of block zero later
        // (`ml_recover` calls `set_page_size`), so the size used here may
        // be too small and the block count is rounded up.
        let size = if memfile.mf_fd < 0 || flags & (O_TRUNC | O_EXCL) != 0 {
            0
        } else {
            memfile.file_size()
        };
        memfile.mf_blocknr_max = pages_in_file(size, memfile.mf_page_size);
        memfile.mf_infile_count = memfile.mf_blocknr_max;
        Some(memfile)
    }

    /// How long the swap file is, or −1 when it cannot be told.
    pub(crate) fn file_size(&self) -> FileOffset {
        // SAFETY: the memfile's own descriptor; `lseek` reads nothing.
        unsafe { lseek(self.mf_fd, 0, SEEK_END) }
    }

    /// Give a memory-only memfile a swap file, as `'updatecount'` going from
    /// zero to non-zero does.
    pub(crate) fn open_file(&mut self, fname: CString) -> Result<(), SwapFailed> {
        if self.do_open(fname, O_RDWR | O_CREAT | O_EXCL) {
            self.mf_dirty = MfDirty::Yes;
            Ok(())
        } else {
            Err(SwapFailed)
        }
    }

    /// Close the memfile, releasing every block, and delete the swap file if
    /// `del_file`.
    pub(crate) fn close(mut self: Box<Self>, del_file: bool) {
        // SAFETY: the memfile's own descriptor, closed once: it is
        // forgotten here, so the drop below does not close it again.
        if self.mf_fd >= 0 && unsafe { close(self.mf_fd) } < 0 {
            emsg(gettext(e_swapclose));
        }
        self.mf_fd = -1;
        if del_file && let Some(fname) = self.fname() {
            os_remove(fname);
        }
        // Dropping the memfile drops every block it still owns.
    }

    /// Set the page size, once block zero of an existing swap file has said
    /// what it really is.
    pub(crate) fn set_page_size(&mut self, new_size: c_uint) {
        self.mf_page_size = new_size;
    }

    /// Allocate a new block of `page_count` pages and lock it.
    ///
    /// `negative` asks for a memory-only block number, which is what data
    /// blocks get: they are written last, so their numbers are handed out last.
    pub(crate) fn new_block(&mut self, negative: bool, page_count: c_uint) -> BlockId {
        let mut block = BlockHdr::new(self.mf_page_size, page_count);
        // Reuse a free run if one is long enough, otherwise take the next
        // number from whichever end this block belongs to.
        if let Some(free) = self.free.last_mut()
            && !negative
            && free.page_count >= page_count
        {
            block.bnum = free.bnum;
            if free.page_count > page_count {
                free.bnum += page_count as BlockNr;
                free.page_count -= page_count;
            } else {
                self.free.pop();
            }
        } else if negative {
            block.bnum = self.mf_blocknr_min;
            self.mf_blocknr_min -= 1;
            self.mf_neg_count += 1;
        } else {
            block.bnum = self.mf_blocknr_max;
            self.mf_blocknr_max += page_count as BlockNr;
        }

        block.flags = BH_LOCKED | BH_DIRTY; // a new block is always dirty
        self.mf_dirty = MfDirty::Yes;
        self.used.insert(block)
    }

    /// Find block `nr`, reading it from the file if it is not in memory, and
    /// lock it. `None` if there is no such block.
    ///
    /// A negative `nr` must go through [`MemFile::trans_del`] first.
    pub(crate) fn get(&mut self, nr: BlockNr, page_count: c_uint) -> Option<BlockId> {
        if nr >= self.mf_blocknr_max || nr <= self.mf_blocknr_min {
            return None;
        }

        // Handing a block out moves it to the end of the sync order.
        let id = match self.used.find(nr) {
            Some(id) => {
                self.used.move_to_end(id);
                id
            }
            None => {
                // Not in memory. Only a block the file holds can be read
                // back; a memory-only one is simply gone.
                if nr < 0 || nr >= self.mf_infile_count || page_count == 0 {
                    return None;
                }
                let mut block = BlockHdr::new(self.mf_page_size, page_count);
                block.bnum = nr;
                self.read(&mut block).ok()?;
                self.used.insert(block)
            }
        };
        self.used.block_mut(id).flags |= BH_LOCKED;
        Some(id)
    }

    /// The block numbered `nr` if it is already in memory.
    ///
    /// Unlike [`MemFile::get`] this neither reads the file, nor locks the
    /// block, nor moves it in the sync order. `ml_setflags` uses it to amend
    /// block zero where it lies.
    pub(crate) fn find(&self, nr: BlockNr) -> Option<BlockId> {
        self.used.find(nr)
    }

    /// The bytes of block `id`: `page_count` pages, starting at the block's
    /// first byte.
    ///
    /// # Panics
    /// When `id` names a block that is no longer in memory.
    #[inline]
    pub(crate) fn page(&self, id: BlockId) -> &[u8] {
        &self.used.block(id).data
    }

    /// [`MemFile::page`], writable.
    #[inline]
    pub(crate) fn page_mut(&mut self, id: BlockId) -> &mut [u8] {
        &mut self.used.block_mut(id).data
    }

    /// Two different blocks' bytes at once, for copying from one into the
    /// other.
    ///
    /// # Panics
    /// When `a` and `b` are the same block, or either is not in memory.
    pub(crate) fn pages_mut(&mut self, a: BlockId, b: BlockId) -> (&mut [u8], &mut [u8]) {
        let (a, b) = self.used.block_pair_mut(a, b);
        (&mut a.data, &mut b.data)
    }

    /// Block `id`'s number: its page number in the file, or a negative one
    /// if it has never been written.
    pub(crate) fn block_nr(&self, id: BlockId) -> BlockNr {
        self.used.block(id).bnum
    }

    /// How many pages block `id` spans.
    pub(crate) fn page_count(&self, id: BlockId) -> c_uint {
        self.used.block(id).page_count
    }

    /// Say block `id` changed without taking it out: what `ml_setflags` does
    /// to block zero before a sync.
    pub(crate) fn mark_dirty(&mut self, id: BlockId) {
        self.used.block_mut(id).flags |= BH_DIRTY;
    }

    /// Grow (or shrink) block `id` to `bytes`, keeping what fits: block zero
    /// of a swap file being recovered was read at a guessed page size.
    pub(crate) fn resize_page(&mut self, id: BlockId, bytes: usize) {
        let block = self.used.block_mut(id);
        let mut data = vec![0u8; bytes].into_boxed_slice();
        let keep = bytes.min(block.data.len());
        data[..keep].copy_from_slice(&block.data[..keep]);
        block.data = data;
    }

    /// Release a block held by [`MemFile::get`] or [`MemFile::new_block`].
    ///
    /// `dirty` says it was changed and has to reach the file; `infile` asks
    /// for its file block number to be settled now, which recovery needs so
    /// that a block already on disk can name it.
    pub(crate) fn put(&mut self, id: BlockId, dirty: bool, infile: bool) {
        let block = self.used.block_mut(id);
        let mut flags = block.flags;
        if flags & BH_LOCKED == 0 {
            iemsg(gettext(c"E293: Block was not locked"));
        }
        flags &= !BH_LOCKED;
        if dirty {
            flags |= BH_DIRTY;
            if self.mf_dirty != MfDirty::YesNoSync {
                self.mf_dirty = MfDirty::Yes;
            }
        }
        self.used.block_mut(id).flags = flags;
        if infile {
            self.trans_add(id);
        }
    }

    /// Give up a block for good. Its pages in the file, if it has any, go
    /// back on the free list, and `id` names nothing from here on.
    pub(crate) fn free_block(&mut self, id: BlockId) {
        let block = self.used.remove(id);
        let (bnum, page_count) = (block.bnum, block.page_count);
        drop(block);
        if bnum < 0 {
            // Memory-only numbers are never reused, so they do not belong
            // on the free list.
            self.mf_neg_count -= 1;
        } else {
            self.free.push(FreeBlock { bnum, page_count });
        }
    }

    /// Write out every dirty block, newest first.
    ///
    /// `flags` is a set of [`MFS_ALL`], [`MFS_STOP`], [`MFS_FLUSH`] and
    /// [`MFS_ZERO`]. Fails when there is no file or a write failed — which
    /// on a full disk is the common case, so after the first failure only
    /// blocks that already have a place in the file are attempted.
    pub(crate) fn sync(&mut self, flags: c_int) -> Result<(), SwapFailed> {
        let got_int_save = got_int.get();

        if self.mf_fd < 0 {
            self.mf_dirty = MfDirty::No;
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
        while i < self.used.len() {
            let id = self.used.id_at(i);
            let (bnum, block_flags) = {
                let block = self.used.block(id);
                (block.bnum, block.flags)
            };
            visited = true;
            let syncable = (flags & MFS_ALL != 0 || bnum >= 0)
                && block_flags & BH_DIRTY != 0
                && (status.is_ok() || (bnum >= 0 && bnum < self.mf_infile_count));
            if syncable && !(flags & MFS_ZERO != 0 && bnum != 0) {
                if self.write(id).is_err() {
                    if status.is_err() {
                        break; // a second failure: give up
                    }
                    status = Err(SwapFailed);
                }
                if flags & MFS_STOP != 0 {
                    if os_char_avail() {
                        break;
                    }
                } else if !main_loop_is_recursing() {
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
            self.mf_dirty = MfDirty::No;
        }

        // SAFETY: the memfile's own descriptor.
        if flags & MFS_FLUSH != 0 && unsafe { os_fsync(self.mf_fd) } != 0 {
            status = Err(SwapFailed);
        }

        got_int.set(got_int.get() || got_int_save);
        status
    }

    /// Mark every block that has a place in the file dirty, so a freshly
    /// created swap file gets all of them.
    pub(crate) fn set_dirty(&mut self) {
        for i in 0..self.used.len() {
            let block = self.used.block_mut(self.used.id_at(i));
            if block.bnum > 0 {
                block.flags |= BH_DIRTY;
            }
        }
        self.mf_dirty = MfDirty::Yes;
    }

    /// Drop every block nobody holds, writing the dirty ones out first:
    /// what running out of memory does to each memfile. Answers whether
    /// anything was released. A block that is locked — the one a memline
    /// is holding — stays, and so does its handle.
    pub(crate) fn release_unlocked(&mut self) -> bool {
        let mut released = false;
        if self.mf_fd < 0 {
            return released; // nowhere to put them
        }
        let mut i = 0;
        while i < self.used.len() {
            let id = self.used.id_at(i);
            let flags = self.used.block(id).flags;
            if flags & BH_LOCKED == 0 && (flags & BH_DIRTY == 0 || self.write(id).is_ok()) {
                // Dropping it releases the block's pages.
                drop(self.used.remove(id));
                released = true;
                // Stay at `i`: removal moved another block into this slot
                // (or that was the last one).
            } else {
                i += 1;
            }
        }
        released
    }

    /// Read a block's pages from the file.
    fn read(&mut self, block: &mut BlockHdr) -> Result<(), SwapFailed> {
        if self.mf_fd < 0 {
            return Err(SwapFailed); // there is no file to read
        }

        let page_size = self.mf_page_size;
        let offset = (page_size as BlockNr * block.bnum) as FileOffset;
        // SAFETY: the memfile's own descriptor.
        if unsafe { lseek(self.mf_fd, offset, SEEK_SET) } != offset {
            perror_msg(c"E294: Seek error in swap file read");
            return Err(SwapFailed);
        }
        let size = block.data.len();
        if read_eintr(self.mf_fd, &mut block.data) as usize != size {
            perror_msg(c"E295: Read error in swap file");
            return Err(SwapFailed);
        }
        Ok(())
    }

    /// Write a block's pages to the file, extending it as needed.
    ///
    /// The file must have no holes, so writing a block past the end first
    /// fills the space in front of it — with the blocks that belong there,
    /// or, where one of those has been freed, with a copy of this block's
    /// bytes as filler.
    fn write(&mut self, id: BlockId) -> Result<(), SwapFailed> {
        if self.mf_fd < 0 && !self.mf_reopen {
            return Err(SwapFailed); // there is no file and there never was
        }

        // A memory-only block must be given its place in the file first.
        if self.used.block(id).bnum < 0 {
            self.trans_add(id);
        }

        let page_size = self.mf_page_size;
        loop {
            let target = self.used.block(id).bnum;
            let mut nr = target;
            // The block whose bytes go at `nr`: this one, the one that
            // belongs in front of it, or (`None`) filler, because that one
            // was freed.
            let at = if nr > self.mf_infile_count {
                nr = self.mf_infile_count;
                self.used.find(nr)
            } else {
                Some(id)
            };

            let offset = (page_size as BlockNr * nr) as FileOffset;
            let page_count = at.map_or(1, |at| self.used.block(at).page_count);
            // Upstream lets this wrap, and a line long enough to overflow it
            // would have overflowed `ml_append_int`'s arithmetic first.
            let size = page_size.wrapping_mul(page_count) as usize;

            for attempt in 1..=2 {
                if self.mf_fd >= 0 {
                    // SAFETY: the memfile's own descriptor.
                    if unsafe { lseek(self.mf_fd, offset, SEEK_SET) } != offset {
                        perror_msg(c"E296: Seek error in swap file write");
                        return Err(SwapFailed);
                    }
                    let data = &self.used.block(at.unwrap_or(id)).data;
                    if write_eintr(self.mf_fd, &data[..size.min(data.len())]) as usize == size {
                        break;
                    }
                }

                if attempt == 1 {
                    // A swap file on a network drive survives the
                    // connection dropping if we reopen it.
                    // SAFETY: the memfile's own descriptor, and its name,
                    // NUL-terminated.
                    unsafe {
                        if self.mf_fd >= 0 {
                            close(self.mf_fd);
                        }
                        let name = self
                            .fname
                            .as_deref()
                            .map_or(core::ptr::null(), CStr::as_ptr);
                        self.mf_fd = os_open(name, self.mf_flags, SWAPFILE_MODE);
                    }
                    self.mf_reopen = self.mf_fd < 0;
                }
                if attempt == 2 || self.mf_fd < 0 {
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
            if let Some(at) = at {
                self.used.block_mut(at).flags &= !BH_DIRTY; // a real block, not filler
            }
            if nr + page_count as BlockNr > self.mf_infile_count {
                self.mf_infile_count = nr + page_count as BlockNr;
            }
            if nr == target {
                break; // wrote the block we came for
            }
        }

        Ok(())
    }

    /// Give a memory-only block a place in the file, and remember the
    /// renumbering for [`MemFile::trans_del`].
    ///
    /// Cannot fail: a number is always available, and the page run it comes
    /// from is either recycled off the free list or taken past the end of
    /// the file. Upstream returned `OK`/`FAIL` here and never answered
    /// `FAIL`.
    fn trans_add(&mut self, id: BlockId) {
        let (old_bnum, page_count) = {
            let block = self.used.block(id);
            (block.bnum, block.page_count)
        };
        if old_bnum >= 0 {
            return; // already has one
        }

        // Reuse a free run if one is long enough, as `new_block` does.
        let new_bnum;
        if let Some(free) = self.free.last_mut()
            && free.page_count >= page_count
        {
            new_bnum = free.bnum;
            if free.page_count > page_count {
                free.bnum += page_count as BlockNr;
                free.page_count -= page_count;
            } else {
                self.free.pop();
            }
        } else {
            new_bnum = self.mf_blocknr_max;
            self.mf_blocknr_max += page_count as BlockNr;
        }

        // Refiling moves the block to the end of the order, as taking it
        // out and putting it back did; the handle stays good.
        self.used.renumber(id, new_bnum);
        self.trans.insert(old_bnum, new_bnum);
    }

    /// The file block number a memory-only block was given, consuming the
    /// record of it. Answers `old_nr` unchanged if there is none.
    pub(crate) fn trans_del(&mut self, old_nr: BlockNr) -> BlockNr {
        match self.trans.remove(&old_nr) {
            Some(new_bnum) => {
                self.mf_neg_count -= 1;
                new_bnum
            }
            None => old_nr,
        }
    }
}

impl Drop for MemFile {
    /// A memfile that goes without [`MemFile::close`] — the buffer holding
    /// it freed with its memline still open — still gives its descriptor
    /// back. The swap file itself stays, as it would after a crash.
    fn drop(&mut self) {
        if self.mf_fd >= 0 {
            // SAFETY: the memfile's own descriptor, which nothing else
            // closes once the memfile is gone.
            unsafe { close(self.mf_fd) };
        }
    }
}

/// How many `page_size` pages a swap file of `size` bytes holds, rounded up.
/// Zero for an empty file, and for one that was never opened.
fn pages_in_file(size: FileOffset, page_size: c_uint) -> BlockNr {
    if size <= 0 {
        return 0;
    }
    let page_size = BlockNr::from(page_size);
    assert!(
        page_size > 0 && page_size - 1 <= BlockNr::MAX - size,
        "memfile: swap file too large for its page size"
    );
    (size + page_size - 1) / page_size
}

/// Close and delete the swap file of `buffer`, keeping the memory file. Used
/// when `'swapfile'` is reset.
///
/// `getlines` first pulls every line into memory — clumsy, but the blocks
/// still in the file are about to become unreachable.
pub(crate) fn mf_close_file(mut buffer: Buf, getlines: bool) {
    if buffer
        .b_ml
        .ml_mfp
        .as_ref()
        .is_none_or(|memfile| memfile.mf_fd < 0)
    {
        return;
    }

    if getlines {
        for lnum in 1..=buffer.b_ml.ml_line_count {
            ml_get_buf(buffer, lnum);
        }
    }

    let Some(memfile) = buffer.b_ml.ml_mfp.as_deref_mut() else {
        return;
    };
    // SAFETY: the memfile's own descriptor, closed once and forgotten.
    if unsafe { close(memfile.mf_fd) } < 0 {
        emsg(gettext(e_swapclose));
    }
    memfile.mf_fd = -1;

    if let Some(fname) = memfile.fname() {
        os_remove(fname);
        memfile.free_fnames();
    }
}

/// Drop as many cached blocks as possible, for when memory has run out.
/// Answers whether anything was released.
pub(crate) fn mf_release_all() -> bool {
    let mut released = false;
    for mut buf in buffers() {
        let Some(memfile) = buf.b_ml.ml_mfp.as_deref() else {
            continue;
        };
        // Nothing can be released without somewhere to put it.
        if memfile.mf_fd < 0 && buf.b_may_swap {
            ml_open_file(buf);
        }
        if let Some(memfile) = buf.b_ml.ml_mfp.as_deref_mut() {
            released |= memfile.release_unlocked();
        }
    }
    released
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn a_memory_only_memfile_releases_nothing() {
        let mut memfile = MemFile::in_memory();
        let id = memfile.new_block(false, 1);
        memfile.put(id, true, false);
        // Nowhere to write it, so it has to stay.
        assert!(!memfile.release_unlocked());
        assert_eq!(memfile.find(0), Some(id));
        memfile.close(false);
    }

    #[test]
    #[cfg_attr(miri, ignore = "a real swap file: lseek and fcntl")]
    fn releasing_memory_keeps_the_locked_block_and_reads_the_rest_back() {
        let dir = std::env::temp_dir().join(format!("nvim-memfile-release-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let path = dir.join("x.swp");
        let name = CString::new(path.as_os_str().as_bytes()).expect("a path without NUL");
        let mut memfile = MemFile::open(name, O_RDWR | O_CREAT | O_EXCL).expect("a swap file");

        // Three blocks, numbered 0, 1 and 2; the middle one stays held, as
        // the memline's locked data block does while a line is cached.
        let ids: Vec<BlockId> = (0..3u8)
            .map(|n| {
                let id = memfile.new_block(false, 1);
                memfile.page_mut(id)[0] = n + 1;
                id
            })
            .collect();
        memfile.put(ids[0], true, false);
        memfile.put(ids[2], true, false);
        let held = ids[1];
        memfile.page_mut(held)[1] = 42;

        assert!(memfile.release_unlocked());
        // The held block, its handle and its bytes all survive.
        assert_eq!(memfile.find(1), Some(held));
        assert_eq!(&memfile.page(held)[..2], &[2, 42]);
        // The others went to the file and out of memory ...
        assert!(memfile.find(0).is_none());
        assert!(memfile.find(2).is_none());
        // ... and come back from it.
        let back = memfile.get(2, 1).expect("block 2 from the file");
        assert_eq!(memfile.page(back)[0], 3);
        assert_ne!(back, ids[2], "a released handle names nothing");
        memfile.put(back, false, false);
        memfile.put(held, false, false);

        memfile.close(true);
        assert!(
            !path.exists(),
            "closing with `del_file` removes the swap file"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
