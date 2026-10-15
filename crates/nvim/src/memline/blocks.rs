//! The three kinds of block a swap file holds, as typed views over the bytes
//! the memfile hands out.
//!
//! A page is `&[u8]` (or `&mut [u8]`) — [`MemFile::page`] is the only thing
//! that hands one out — and a view reads and writes the block's fields at the
//! offsets its C layout gives them, in native byte order, every access
//! checked against the page's length. The layouts themselves are the
//! `#[repr(C)]` structs below; nothing ever constructs one, they exist so
//! that every offset is `offset_of!` of a field the format names rather than
//! a number, and so that the format's padding stays the compiler's word.
//!
//! * A **data block** is a header, then an index of line starts growing up
//!   from the header, then the lines' text growing down from the end of the
//!   block — the first line at the very end. An index entry's top bit is
//!   [`DB_MARKED`].
//! * A **pointer block** is a header and an array of [`PointerEntry`]s, one
//!   per child, in line order.
//! * **Block zero** is the swap file's header: [`ZeroBlockLayout`].
//!
//! [`MemFile::page`]: crate::memfile::MemFile::page

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_long, c_uint};
use core::mem::{offset_of, size_of};

use crate::types::{BlockNr, LineNr};

/// The data block layout: a header and the index that follows it.
#[repr(C)]
pub(crate) struct DataBlockLayout {
    pub db_id: u16,
    /// Bytes free between the end of the index and the start of the text.
    pub db_free: c_uint,
    /// Where the text starts: the last line's first byte.
    pub db_txt_start: c_uint,
    /// Where the text ends: the block's length.
    pub db_txt_end: c_uint,
    pub db_line_count: c_long,
    pub db_index: [c_uint; 0],
}

/// `db_id` of a data block: `"da"`.
pub(crate) const DATA_ID: u16 = 25697;
/// The marker `:global` sets on a line it still has to visit.
pub(crate) const DB_MARKED: c_uint = 1 << (c_uint::BITS - 1);
/// An index entry without its [`DB_MARKED`] bit: the line's start.
pub(crate) const DB_INDEX_MASK: c_uint = !DB_MARKED;
/// One index entry.
pub(crate) const INDEX_SIZE: usize = size_of::<c_uint>();
/// The data block header: where the index starts.
pub(crate) const HEADER_SIZE: usize = offset_of!(DataBlockLayout, db_index);

/// One child of a pointer block.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PointerEntry {
    /// The child's block number.
    pub pe_bnum: BlockNr,
    /// How many lines the child holds.
    pub pe_line_count: LineNr,
    /// Which line the child held first when the file was read: all
    /// `:recover` has to order a block it can only find in the file by.
    pub pe_old_lnum: LineNr,
    /// How many pages the child spans.
    pub pe_page_count: c_int,
}

/// The pointer block layout: a header and the entries that follow it.
#[repr(C)]
pub(crate) struct PointerBlockLayout {
    pub pb_id: u16,
    pub pb_count: u16,
    /// How many entries fit: stored, so a swap file written with another
    /// page size is recognisable.
    pub pb_count_max: u16,
    pub pb_pointer: [PointerEntry; 0],
}

/// `pb_id` of a pointer block: `"tp"`.
pub(crate) const PTR_ID: u16 = 28788;

pub const B0_HNAME_SIZE: c_uint = 40;
pub const B0_UNAME_SIZE: c_uint = 40;
pub const B0_FNAME_SIZE_ORG: c_uint = 900;
pub const B0_FNAME_SIZE_NOCRYPT: c_uint = 898;
pub const B0_FNAME_SIZE_CRYPT: c_uint = 890;
pub const B0_MAGIC_CHAR: u8 = 85;
pub const B0_MAGIC_SHORT: c_uint = 269_554_195;
pub const B0_MAGIC_INT: c_uint = 539_042_339;
pub const B0_MAGIC_LONG: c_uint = 808_530_483;
pub const BLOCK0_ID1: u8 = b'0';
pub const BLOCK0_ID0: u8 = b'b';
pub(crate) const B0_DIRTY: u8 = 0x55;
pub(crate) const B0_FF_MASK: c_int = 3;
pub(crate) const B0_SAME_DIR: c_int = 4;
pub(crate) const B0_HAS_FENC: c_int = 8;

/// Block zero's layout. The strings are NUL-terminated inside their field
/// when the block is sane; the four magic numbers are one value per C width
/// so that a build with another endianness or another type size reads them
/// as a mismatch.
#[repr(C)]
pub struct ZeroBlockLayout {
    pub b0_id: [u8; 2],
    pub b0_version: [u8; 10],
    pub b0_page_size: [u8; 4],
    pub b0_mtime: [u8; 4],
    pub b0_ino: [u8; 4],
    pub b0_pid: [u8; 4],
    pub b0_uname: [u8; B0_UNAME_SIZE as usize],
    pub b0_hname: [u8; B0_HNAME_SIZE as usize],
    pub b0_fname: [u8; B0_FNAME_SIZE_ORG as usize],
    pub b0_magic_long: c_long,
    pub b0_magic_int: c_int,
    pub b0_magic_short: i16,
    pub b0_magic_char: u8,
}

/// How many bytes of a page block zero uses.
pub(crate) const B0_SIZE: usize = size_of::<ZeroBlockLayout>();

/// `N` bytes of `bytes` from `at`.
///
/// # Panics
/// When they run past the end: a field outside its page.
#[inline]
fn field<const N: usize>(bytes: &[u8], at: usize) -> &[u8; N] {
    <&[u8; N]>::try_from(&bytes[at..at + N]).expect("N bytes")
}

#[inline]
fn field_mut<const N: usize>(bytes: &mut [u8], at: usize) -> &mut [u8; N] {
    <&mut [u8; N]>::try_from(&mut bytes[at..at + N]).expect("N bytes")
}

#[inline]
fn get_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_ne_bytes(*field(bytes, at))
}

#[inline]
fn set_u16(bytes: &mut [u8], at: usize, value: u16) {
    *field_mut(bytes, at) = value.to_ne_bytes();
}

#[inline]
fn get_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_ne_bytes(*field(bytes, at))
}

#[inline]
fn set_u32(bytes: &mut [u8], at: usize, value: u32) {
    *field_mut(bytes, at) = value.to_ne_bytes();
}

#[inline]
fn get_i32(bytes: &[u8], at: usize) -> i32 {
    i32::from_ne_bytes(*field(bytes, at))
}

#[inline]
fn set_i32(bytes: &mut [u8], at: usize, value: i32) {
    *field_mut(bytes, at) = value.to_ne_bytes();
}

#[inline]
fn get_i64(bytes: &[u8], at: usize) -> i64 {
    i64::from_ne_bytes(*field(bytes, at))
}

#[inline]
fn set_i64(bytes: &mut [u8], at: usize, value: i64) {
    *field_mut(bytes, at) = value.to_ne_bytes();
}

/// Store a number in block zero: four bytes, little-endian.
///
/// Only the low 32 bits survive — the format has no room for more, and a
/// timestamp or inode wider than that is simply truncated, as upstream's
/// `(unsigned)n >> 8` stepping did.
pub(crate) fn b0_store_number(n: c_long, dest: &mut [u8; 4]) {
    *dest = n.to_le_bytes()[..4].try_into().expect("four bytes");
}

/// Read back a number stored by [`b0_store_number`]. The result is the
/// unsigned 32-bit value widened, never negative.
pub(crate) fn b0_read_number(src: &[u8; 4]) -> c_long {
    c_long::from(u32::from_le_bytes(*src))
}

/// A data block, over its page.
pub(crate) struct DataBlock<P>(pub(crate) P);

impl<P: AsRef<[u8]>> DataBlock<P> {
    #[inline]
    fn bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// The whole block's length.
    pub(crate) fn len(&self) -> usize {
        self.bytes().len()
    }

    pub(crate) fn id(&self) -> u16 {
        get_u16(self.bytes(), offset_of!(DataBlockLayout, db_id))
    }

    /// Whether this is a data block, rather than a pointer block or garbage.
    pub(crate) fn is_data(&self) -> bool {
        self.id() == DATA_ID
    }

    pub(crate) fn free(&self) -> c_uint {
        get_u32(self.bytes(), offset_of!(DataBlockLayout, db_free))
    }

    pub(crate) fn txt_start(&self) -> c_uint {
        get_u32(self.bytes(), offset_of!(DataBlockLayout, db_txt_start))
    }

    pub(crate) fn txt_end(&self) -> c_uint {
        get_u32(self.bytes(), offset_of!(DataBlockLayout, db_txt_end))
    }

    pub(crate) fn line_count(&self) -> c_long {
        get_i64(self.bytes(), offset_of!(DataBlockLayout, db_line_count))
    }

    /// Where index entry `i` is.
    #[inline]
    pub(crate) fn index_at(i: usize) -> usize {
        HEADER_SIZE + i * INDEX_SIZE
    }

    /// Index entry `i`, mark bit and all.
    #[inline]
    pub(crate) fn index(&self, i: usize) -> c_uint {
        get_u32(self.bytes(), Self::index_at(i))
    }

    /// Where line `i` of the block starts.
    #[inline]
    pub(crate) fn line_start(&self, i: usize) -> c_uint {
        self.index(i) & DB_INDEX_MASK
    }

    /// Where line `i` of the block ends, its NUL included: where the line
    /// before it in the block starts — lines are stored back to front — or
    /// the end of the text for the first.
    #[inline]
    pub(crate) fn line_end(&self, i: usize) -> c_uint {
        if i == 0 {
            self.txt_end()
        } else {
            self.line_start(i - 1)
        }
    }

    pub(crate) fn is_marked(&self, i: usize) -> bool {
        self.index(i) & DB_MARKED != 0
    }

    /// The bytes from `from` to `to`.
    #[inline]
    pub(crate) fn text(&self, from: usize, to: usize) -> &[u8] {
        &self.bytes()[from..to]
    }
}

impl<P: AsRef<[u8]> + AsMut<[u8]>> DataBlock<P> {
    #[inline]
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }

    /// Make the page an empty data block filling all of it.
    pub(crate) fn init(&mut self) {
        let end = c_uint::try_from(self.len()).expect("a block shorter than 4 GiB");
        set_u16(
            self.bytes_mut(),
            offset_of!(DataBlockLayout, db_id),
            DATA_ID,
        );
        self.set_txt_end(end);
        self.set_txt_start(end);
        let header = c_uint::try_from(HEADER_SIZE).expect("a 24-byte header");
        self.set_free(end.wrapping_sub(header));
        self.set_line_count(0);
    }

    pub(crate) fn set_free(&mut self, value: c_uint) {
        set_u32(
            self.bytes_mut(),
            offset_of!(DataBlockLayout, db_free),
            value,
        );
    }

    pub(crate) fn set_txt_start(&mut self, value: c_uint) {
        set_u32(
            self.bytes_mut(),
            offset_of!(DataBlockLayout, db_txt_start),
            value,
        );
    }

    pub(crate) fn set_txt_end(&mut self, value: c_uint) {
        set_u32(
            self.bytes_mut(),
            offset_of!(DataBlockLayout, db_txt_end),
            value,
        );
    }

    pub(crate) fn set_line_count(&mut self, value: c_long) {
        set_i64(
            self.bytes_mut(),
            offset_of!(DataBlockLayout, db_line_count),
            value,
        );
    }

    #[inline]
    pub(crate) fn set_index(&mut self, i: usize, value: c_uint) {
        set_u32(self.bytes_mut(), Self::index_at(i), value);
    }

    pub(crate) fn set_marked(&mut self, i: usize, marked: bool) {
        let entry = self.index(i);
        let entry = if marked {
            entry | DB_MARKED
        } else {
            entry & DB_INDEX_MASK
        };
        self.set_index(i, entry);
    }

    /// Move `len` bytes from `from` to `to` inside the block; the two may
    /// overlap.
    pub(crate) fn move_bytes(&mut self, from: usize, to: usize, len: usize) {
        self.bytes_mut().copy_within(from..from + len, to);
    }

    /// Write `text` at `at`.
    pub(crate) fn write(&mut self, at: usize, text: &[u8]) {
        self.bytes_mut()[at..at + text.len()].copy_from_slice(text);
    }

    /// The whole page, for copying a run of text out of or into it.
    pub(crate) fn raw_mut(&mut self) -> &mut [u8] {
        self.bytes_mut()
    }
}

/// A pointer block, over its page.
pub(crate) struct PointerBlock<P>(pub(crate) P);

const PE_AT: usize = offset_of!(PointerBlockLayout, pb_pointer);
const PE_SIZE: usize = size_of::<PointerEntry>();

impl<P: AsRef<[u8]>> PointerBlock<P> {
    #[inline]
    fn bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// How many entries fit in a pointer block of this page size. It is
    /// stored in the block itself (`pb_count_max`), so that a swap file
    /// written by a build with a different page size is recognisable.
    pub(crate) fn count_max_for(page_size: c_uint) -> u16 {
        let room = (page_size as usize).saturating_sub(PE_AT) / PE_SIZE;
        u16::try_from(room).unwrap_or(u16::MAX)
    }

    pub(crate) fn id(&self) -> u16 {
        get_u16(self.bytes(), offset_of!(PointerBlockLayout, pb_id))
    }

    /// Whether this is a pointer block.
    pub(crate) fn is_pointer(&self) -> bool {
        self.id() == PTR_ID
    }

    pub(crate) fn count(&self) -> u16 {
        get_u16(self.bytes(), offset_of!(PointerBlockLayout, pb_count))
    }

    pub(crate) fn count_max(&self) -> u16 {
        get_u16(self.bytes(), offset_of!(PointerBlockLayout, pb_count_max))
    }

    #[inline]
    fn entry_at(i: usize) -> usize {
        PE_AT + i * PE_SIZE
    }

    /// Entry `i`.
    #[inline]
    pub(crate) fn entry(&self, i: usize) -> PointerEntry {
        let at = Self::entry_at(i);
        let bytes = self.bytes();
        PointerEntry {
            pe_bnum: get_i64(bytes, at + offset_of!(PointerEntry, pe_bnum)),
            pe_line_count: get_i32(bytes, at + offset_of!(PointerEntry, pe_line_count)),
            pe_old_lnum: get_i32(bytes, at + offset_of!(PointerEntry, pe_old_lnum)),
            pe_page_count: get_i32(bytes, at + offset_of!(PointerEntry, pe_page_count)),
        }
    }

    /// The line count of entry `i`, without reading the rest of it.
    #[inline]
    pub(crate) fn line_count(&self, i: usize) -> LineNr {
        get_i32(
            self.bytes(),
            Self::entry_at(i) + offset_of!(PointerEntry, pe_line_count),
        )
    }

    /// The lines every entry accounts for, added up.
    pub(crate) fn line_total(&self) -> c_int {
        (0..usize::from(self.count()))
            .map(|i| self.line_count(i))
            .sum()
    }

    /// The bytes of entries `from..from + count`, for copying them into
    /// another block.
    pub(crate) fn entry_bytes(&self, from: usize, count: usize) -> &[u8] {
        &self.bytes()[Self::entry_at(from)..Self::entry_at(from + count)]
    }
}

impl<P: AsRef<[u8]> + AsMut<[u8]>> PointerBlock<P> {
    #[inline]
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }

    /// Make the page an empty pointer block.
    pub(crate) fn init(&mut self, page_size: c_uint) {
        set_u16(
            self.bytes_mut(),
            offset_of!(PointerBlockLayout, pb_id),
            PTR_ID,
        );
        self.set_count(0);
        self.set_count_max(Self::count_max_for(page_size));
    }

    pub(crate) fn set_count(&mut self, count: u16) {
        set_u16(
            self.bytes_mut(),
            offset_of!(PointerBlockLayout, pb_count),
            count,
        );
    }

    pub(crate) fn set_count_max(&mut self, count: u16) {
        set_u16(
            self.bytes_mut(),
            offset_of!(PointerBlockLayout, pb_count_max),
            count,
        );
    }

    pub(crate) fn set_bnum(&mut self, i: usize, bnum: BlockNr) {
        set_i64(
            self.bytes_mut(),
            Self::entry_at(i) + offset_of!(PointerEntry, pe_bnum),
            bnum,
        );
    }

    pub(crate) fn set_line_count(&mut self, i: usize, count: LineNr) {
        let at = Self::entry_at(i) + offset_of!(PointerEntry, pe_line_count);
        set_i32(self.bytes_mut(), at, count);
    }

    pub(crate) fn add_line_count(&mut self, i: usize, delta: LineNr) {
        let count = self.line_count(i);
        self.set_line_count(i, count + delta);
    }

    pub(crate) fn set_old_lnum(&mut self, i: usize, lnum: LineNr) {
        let at = Self::entry_at(i) + offset_of!(PointerEntry, pe_old_lnum);
        set_i32(self.bytes_mut(), at, lnum);
    }

    pub(crate) fn set_page_count(&mut self, i: usize, count: c_int) {
        let at = Self::entry_at(i) + offset_of!(PointerEntry, pe_page_count);
        set_i32(self.bytes_mut(), at, count);
    }

    /// Point entry `i` at a child: its number, line count and page count,
    /// and — when `old_lnum` is not zero — the line it started at.
    pub(crate) fn set_child(
        &mut self,
        i: usize,
        bnum: BlockNr,
        lines: c_int,
        pages: c_int,
        old_lnum: LineNr,
    ) {
        self.set_bnum(i, bnum);
        self.set_line_count(i, lines);
        self.set_page_count(i, pages);
        if old_lnum != 0 {
            self.set_old_lnum(i, old_lnum);
        }
    }

    /// Move entries `from..from + count` to start at `to`; the ranges may
    /// overlap.
    pub(crate) fn move_entries(&mut self, from: usize, to: usize, count: usize) {
        let (src, dst) = (Self::entry_at(from), Self::entry_at(to));
        self.bytes_mut()
            .copy_within(src..src + count * PE_SIZE, dst);
    }

    /// Write `entries` (whole entries' bytes) starting at entry `at`.
    pub(crate) fn write_entries(&mut self, at: usize, entries: &[u8]) {
        let from = Self::entry_at(at);
        self.bytes_mut()[from..from + entries.len()].copy_from_slice(entries);
    }
}

/// Block zero, over its page (or over the bytes read from a swap file's
/// start).
pub(crate) struct ZeroBlock<P>(pub(crate) P);

impl ZeroBlock<()> {
    /// Upstream keeps two more fields in the last two bytes of the name:
    /// `#define b0_dirty b0_fname[B0_FNAME_SIZE_ORG - 1]` and
    /// `#define b0_flags b0_fname[B0_FNAME_SIZE_ORG - 2]`. That is why the
    /// name itself is only ever written up to `B0_FNAME_SIZE_NOCRYPT`.
    const DIRTY: usize = B0_FNAME_SIZE_ORG as usize - 1;
    const FLAGS: usize = B0_FNAME_SIZE_ORG as usize - 2;
}

macro_rules! b0_field {
    ($get:ident, $get_mut:ident, $field:ident, $n:expr) => {
        pub(crate) fn $get(&self) -> &[u8; $n] {
            field(self.bytes(), offset_of!(ZeroBlockLayout, $field))
        }

        pub(crate) fn $get_mut(&mut self) -> &mut [u8; $n]
        where
            P: AsMut<[u8]>,
        {
            field_mut(self.0.as_mut(), offset_of!(ZeroBlockLayout, $field))
        }
    };
}

impl<P: AsRef<[u8]>> ZeroBlock<P> {
    #[inline]
    fn bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    b0_field!(id, id_mut, b0_id, 2);
    b0_field!(version, version_mut, b0_version, 10);
    b0_field!(page_size_field, page_size_mut, b0_page_size, 4);
    b0_field!(mtime_field, mtime_mut, b0_mtime, 4);
    b0_field!(ino_field, ino_mut, b0_ino, 4);
    b0_field!(pid_field, pid_mut, b0_pid, 4);
    b0_field!(uname, uname_mut, b0_uname, B0_UNAME_SIZE as usize);
    b0_field!(hname, hname_mut, b0_hname, B0_HNAME_SIZE as usize);
    b0_field!(fname, fname_mut, b0_fname, B0_FNAME_SIZE_ORG as usize);

    /// The page size the file was written with.
    pub(crate) fn page_size(&self) -> c_long {
        b0_read_number(self.page_size_field())
    }

    /// The edited file's modification time, low 32 bits.
    pub(crate) fn mtime(&self) -> c_long {
        b0_read_number(self.mtime_field())
    }

    /// The edited file's inode, low 32 bits.
    pub(crate) fn ino(&self) -> c_long {
        b0_read_number(self.ino_field())
    }

    /// The PID recorded by the Nvim that last wrote this block.
    pub(crate) fn pid(&self) -> c_long {
        b0_read_number(self.pid_field())
    }

    pub(crate) fn magic_long(&self) -> c_long {
        get_i64(self.bytes(), offset_of!(ZeroBlockLayout, b0_magic_long))
    }

    pub(crate) fn magic_int(&self) -> c_int {
        get_i32(self.bytes(), offset_of!(ZeroBlockLayout, b0_magic_int))
    }

    pub(crate) fn magic_short(&self) -> i16 {
        i16::from_ne_bytes(*field(
            self.bytes(),
            offset_of!(ZeroBlockLayout, b0_magic_short),
        ))
    }

    pub(crate) fn magic_char(&self) -> u8 {
        self.bytes()[offset_of!(ZeroBlockLayout, b0_magic_char)]
    }

    /// [`B0_SAME_DIR`], [`B0_HAS_FENC`] and the `'fileformat'` in
    /// [`B0_FF_MASK`].
    pub(crate) fn flags(&self) -> c_int {
        c_int::from(self.fname()[ZeroBlock::FLAGS])
    }

    /// Whether the buffer had unsaved changes when this was last written.
    /// It is what makes `:recover` worth offering.
    pub(crate) fn dirty(&self) -> bool {
        self.fname()[ZeroBlock::DIRTY] != 0
    }
}

impl<P: AsRef<[u8]> + AsMut<[u8]>> ZeroBlock<P> {
    pub(crate) fn set_page_size(&mut self, size: c_long) {
        b0_store_number(size, self.page_size_mut());
    }

    pub(crate) fn set_mtime(&mut self, mtime: c_long) {
        b0_store_number(mtime, self.mtime_mut());
    }

    pub(crate) fn set_ino(&mut self, ino: c_long) {
        b0_store_number(ino, self.ino_mut());
    }

    pub(crate) fn set_pid(&mut self, pid: c_long) {
        b0_store_number(pid, self.pid_mut());
    }

    /// Write the id and the four magic numbers that make a page block zero.
    pub(crate) fn set_identity(&mut self) {
        *self.id_mut() = [BLOCK0_ID0, BLOCK0_ID1];
        let bytes = self.0.as_mut();
        set_i64(
            bytes,
            offset_of!(ZeroBlockLayout, b0_magic_long),
            c_long::from(B0_MAGIC_LONG),
        );
        set_i32(
            bytes,
            offset_of!(ZeroBlockLayout, b0_magic_int),
            B0_MAGIC_INT.cast_signed(),
        );
        // Truncated to 16 bits, as upstream's store into an `int16_t` is.
        let short = (B0_MAGIC_SHORT & 0xffff) as u16;
        set_u16(bytes, offset_of!(ZeroBlockLayout, b0_magic_short), short);
        bytes[offset_of!(ZeroBlockLayout, b0_magic_char)] = B0_MAGIC_CHAR;
    }

    pub(crate) fn set_flags(&mut self, flags: c_int) {
        // A byte of the name field: only the low eight bits are kept.
        self.fname_mut()[ZeroBlock::FLAGS] = flags.to_le_bytes()[0];
    }

    pub(crate) fn set_flag(&mut self, flag: c_int, on: bool) {
        let flags = self.flags();
        self.set_flags(if on { flags | flag } else { flags & !flag });
    }

    pub(crate) fn set_dirty(&mut self, dirty: bool) {
        self.fname_mut()[ZeroBlock::DIRTY] = if dirty { B0_DIRTY } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: usize = 4096;

    fn page() -> Vec<u8> {
        vec![0; PAGE]
    }

    #[test]
    fn the_layouts_are_the_formats() {
        // The offsets the swap file format has always had on a 64-bit
        // build; a change here is a change of format.
        assert_eq!(HEADER_SIZE, 24);
        assert_eq!(PE_AT, 8);
        assert_eq!(PE_SIZE, 24);
        assert_eq!(offset_of!(ZeroBlockLayout, b0_fname), 108);
        assert_eq!(offset_of!(ZeroBlockLayout, b0_magic_long), 1008);
        assert_eq!(B0_SIZE, 1024);
    }

    #[test]
    fn a_fresh_data_block_is_empty_and_fills_its_page() {
        let mut block = DataBlock(page());
        block.init();
        assert!(block.is_data());
        assert_eq!(block.txt_end(), 4096);
        assert_eq!(block.txt_start(), 4096);
        assert_eq!(block.free(), 4096 - 24);
        assert_eq!(block.line_count(), 0);
        // In native order, at the C offsets.
        assert_eq!(&block.0[..2], &DATA_ID.to_ne_bytes());
        assert_eq!(&block.0[12..16], &4096u32.to_ne_bytes());
    }

    #[test]
    fn lines_go_in_back_to_front_and_read_back() {
        let mut block = DataBlock(page());
        block.init();
        // Two lines, as `ml_append_int` lays them out: the first at the end.
        for (i, text) in [&b"one\0"[..], b"three\0"].into_iter().enumerate() {
            let len = c_uint::try_from(text.len()).unwrap();
            let start = block.txt_start() - len;
            block.write(start as usize, text);
            block.set_index(i, start);
            block.set_txt_start(start);
            block.set_free(block.free() - len - 4);
            block.set_line_count(block.line_count() + 1);
        }
        let line = |b: &DataBlock<Vec<u8>>, i: usize| {
            b.text(b.line_start(i) as usize, b.line_end(i) as usize - 1)
                .to_vec()
        };
        assert_eq!(line(&block, 0), b"one");
        assert_eq!(line(&block, 1), b"three");

        block.set_marked(1, true);
        assert!(block.is_marked(1));
        assert_eq!(
            line(&block, 1),
            b"three",
            "the mark is not part of the start"
        );
        block.set_marked(1, false);
        assert!(!block.is_marked(1));

        // Moving the text keeps every byte.
        let start = block.txt_start() as usize;
        block.move_bytes(start, start - 2, 10);
        assert_eq!(&block.0[start - 2..start + 4], b"three\0");
    }

    #[test]
    #[should_panic(expected = "range end index")]
    fn an_index_past_the_page_is_refused() {
        let block = DataBlock(page());
        let _ = block.index((PAGE - HEADER_SIZE) / INDEX_SIZE);
    }

    #[test]
    fn pointer_entries_read_back_and_move() {
        let mut block = PointerBlock(page());
        block.init(4096);
        assert!(block.is_pointer());
        assert_eq!(block.count_max(), 170);
        block.set_count(3);
        for i in 0..3 {
            let n = c_int::try_from(i).unwrap();
            block.set_child(i, BlockNr::from(n) - 5, 10 * n, 1, n + 1);
        }
        assert_eq!(block.line_total(), 30);
        assert_eq!(
            block.entry(2),
            PointerEntry {
                pe_bnum: -3,
                pe_line_count: 20,
                pe_old_lnum: 3,
                pe_page_count: 1
            }
        );
        // `set_child` with no line leaves the old one.
        block.set_child(0, 9, 1, 2, 0);
        assert_eq!(block.entry(0).pe_old_lnum, 1);

        block.move_entries(1, 2, 2);
        assert_eq!(block.entry(3).pe_line_count, 20);
        let mut other = PointerBlock(page());
        other.write_entries(0, block.entry_bytes(2, 2));
        assert_eq!(other.entry(1), block.entry(3));
        block.add_line_count(3, -5);
        assert_eq!(block.line_count(3), 15);
    }

    #[test]
    fn block_zero_round_trips_its_fields() {
        let mut b0 = ZeroBlock(page());
        b0.set_identity();
        b0.set_page_size(4096);
        b0.set_pid(0x1_2345_6789);
        b0.set_flags(B0_SAME_DIR | 2);
        b0.set_flag(B0_HAS_FENC, true);
        b0.set_dirty(true);
        assert_eq!(b0.id(), b"b0");
        assert_eq!(b0.page_size(), 4096);
        // Four little-endian bytes: the top of a wider number is gone.
        assert_eq!(b0.pid(), 0x2345_6789);
        assert_eq!(b0.pid_field(), &[0x89, 0x67, 0x45, 0x23]);
        assert_eq!(b0.flags(), B0_SAME_DIR | B0_HAS_FENC | 2);
        assert!(b0.dirty());
        assert_eq!(b0.magic_long(), c_long::from(B0_MAGIC_LONG));
        assert_eq!(b0.magic_int(), B0_MAGIC_INT.cast_signed());
        assert_eq!(b0.magic_short(), 0x1213);
        assert_eq!(b0.magic_char(), 0x55);
        b0.fname_mut()[..3].copy_from_slice(b"abc");
        assert_eq!(&b0.0[108..111], b"abc");
    }
}
