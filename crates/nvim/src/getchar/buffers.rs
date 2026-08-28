//! The stuff/read/record buffers: [`KeyBuffer`] and its chain.
//!
//! A `KeyBuffer` is a linked list of [`KeyBlock`]s holding a byte string that
//! is appended to at the insertion point ([`KeyBuffer::add`]) and consumed
//! from the front ([`KeyBuffer::read`]).  Five of them exist — the two read
//! buffers behind [`stuff_readbuf`], the record buffer behind `q`, and the
//! redo pair — and every one of them is filled by the methods here.
//!
//! `KeyBlock::bytes` is a flexible array member: the type declares one byte,
//! the allocation holds as many as the block was sized for. Every read of it
//! goes through [`block_str`], which forms the pointer from the field's
//! address so that it inherits the whole allocation's provenance — taking a
//! slice of the declared `[c_char; 1]` would cover one byte.
//!
//! # What changed against upstream
//!
//! Upstream's `buffheader_T` starts with an *inline* block, `bh_first`, whose
//! only live field is `b_next`; the insertion cursor `bh_curr` then points
//! either at a real block or at that inline sentinel — that is, **into the
//! header itself**. A struct holding its own address cannot be moved, and
//! moving it is exactly what `save_redobuff` and `save_typeahead` do. So the
//! sentinel is gone here: the head is a plain `first` pointer and the cursor
//! is an [`InsertPoint`], which spells "before everything" as a *value*
//! rather than as the sentinel's address. `KeyBuffer` is therefore
//! position-independent, and deliberately **not `Copy`** — a copy of one is
//! two owners of one block chain, which is the bug the phase exists to
//! retire. [`GlobalCell::take`](crate::global_cell::GlobalCell) moves one out
//! of its cell instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_O, Ctrl_V, key_escape};
use crate::types::{MB_MAXBYTES, NUL};
use core::ffi::{c_char, c_int, c_uint};
use core::mem::offset_of;
use core::ptr;

/// Smallest block [`KeyBuffer::add`] will allocate; upstream's
/// `MINIMAL_SIZE`.
const MINIMAL_SIZE: usize = 20;

/// One block of a [`KeyBuffer`]'s byte string.
///
/// The layout is pinned, and load-bearing: `bytes` is a flexible array member
/// — the type declares one byte, the allocation holds as many as the block
/// was sized for, and [`KeyBuffer::add`] sizes it as
/// `offset_of!(KeyBlock, bytes) + len + 1`. That arithmetic only describes
/// the allocation when `bytes` is the *last* field, so every append past the
/// first byte would otherwise land on another field. `#[repr(C)]` is what
/// guarantees declaration order here; [`KeyBuffer::add`] carries the matching
/// compile-time assertion.
#[repr(C)]
pub(crate) struct KeyBlock {
    pub(crate) next: *mut KeyBlock,
    pub(crate) len: usize,
    pub(crate) bytes: [c_char; 1],
}

/// Where a [`KeyBuffer`] puts the bytes it is given next.
///
/// Upstream keeps this as a `buffblock_T *` that may be the address of the
/// header's own inline sentinel; naming the three cases makes the two
/// non-block ones say what they mean, and makes "write into the sentinel"
/// unrepresentable rather than merely unreachable.
#[derive(Clone, Copy)]
enum InsertPoint {
    /// Before everything the buffer already holds. What [`KeyBuffer::restart`]
    /// sets while the buffer is being read, so that text stuffed mid-read is
    /// read *before* the remainder — upstream's `bh_curr = &bh_first`.
    Front,
    /// At the end of this block, while it still has [`KeyBuffer::space`].
    Block(*mut KeyBlock),
    /// Nowhere: the buffer was freed and has no cursor. Adding to it is the
    /// internal error `E222`.
    Gone,
}

/// One key buffer: a byte string of already-escaped keys.
pub(crate) struct KeyBuffer {
    /// The first block, or null when the buffer holds nothing.
    first: *mut KeyBlock,
    /// Where the next bytes go.
    at: InsertPoint,
    /// How far [`KeyBuffer::read`] has got into the first block.
    index: usize,
    /// Bytes still unused in the block `at` names.
    space: usize,
    /// Start a new block on the next add rather than fill `space`.
    create_newblock: bool,
}

impl KeyBuffer {
    /// The state all five buffers start in, and the one `GlobalCell::take`
    /// leaves behind: no blocks, so nothing to free. A `const` as well as a
    /// [`Default`], because the cells themselves are `static`s.
    pub(crate) const EMPTY: Self = KeyBuffer {
        first: ptr::null_mut(),
        at: InsertPoint::Front,
        index: 0,
        space: 0,
        create_newblock: false,
    };
}

impl Default for KeyBuffer {
    fn default() -> Self {
        KeyBuffer::EMPTY
    }
}

/// The bytes of one block, as a pointer over the whole allocation.
///
/// # Safety
/// `block` must point at a live block allocated by [`KeyBuffer::add`].
pub(crate) unsafe fn block_str(block: *mut KeyBlock) -> *mut c_char {
    // SAFETY (this body): the caller's promise -- a live block, whose `bytes`
    // is the flexible array member the allocation was sized for.
    unsafe { (&raw mut (*block).bytes).cast::<c_char>() }
}

impl KeyBuffer {
    /// The first block, for the redo walk. Null when the buffer is empty.
    fn head(&self) -> *mut KeyBlock {
        self.first
    }

    /// Whether the buffer holds nothing.
    fn is_empty(&self) -> bool {
        self.first.is_null()
    }

    /// Free every block and leave the buffer with no insertion point.
    ///
    /// # Safety
    /// Nothing may hold a pointer into the chain.
    unsafe fn free(&mut self) {
        let mut block = self.first;
        while !block.is_null() {
            // SAFETY (this body): the caller's promise -- nothing else points
            // into this chain, so each block is ours to unlink and free
            // exactly once.
            let next = unsafe { (*block).next };
            unsafe { xfree(block.cast()) };
            block = next;
        }
        self.first = ptr::null_mut();
        self.at = InsertPoint::Gone;
    }

    /// The contents as one `xmalloc`ed NUL-terminated string, with its
    /// length. `K_SPECIAL` in the answer is still escaped.
    ///
    /// Answers a null pointer when the buffer is empty and `dozero` is false.
    ///
    /// # Safety
    /// The answer must be freed with `xfree`.
    pub(crate) unsafe fn contents(&self, dozero: bool) -> (*mut c_char, usize) {
        let mut count = 0;
        let mut block = self.first;
        while !block.is_null() {
            // SAFETY (this body): every block on the chain is live and its
            // bytes are NUL-terminated, so the walk stops inside each one;
            // `xmalloc` never returns null and `count + 1` is the total of
            // every block's length plus the terminator.
            count += unsafe { (*block).len };
            block = unsafe { (*block).next };
        }
        if count == 0 && !dozero {
            return (ptr::null_mut(), 0);
        }

        let out = unsafe { xmalloc(count + 1) }.cast::<c_char>();
        let mut at = 0;
        let mut block = self.first;
        while !block.is_null() {
            // Copy up to the block's own terminator rather than up to
            // `len`: `delete_tail` shortens a block by moving the NUL,
            // and upstream reads the NUL here too.
            let str = unsafe { block_str(block) };
            let mut i = 0;
            while unsafe { *str.add(i) } != 0 {
                unsafe { *out.add(at) = *str.add(i) };
                at += 1;
                i += 1;
            }
            block = unsafe { (*block).next };
        }
        unsafe { *out.add(at) = 0 };
        (out, at)
    }

    /// Append `s` at the insertion point.
    ///
    /// `K_SPECIAL` must have been escaped already. `slen` is the length, or
    /// -1 for a NUL-terminated string. Answers false when the buffer has been
    /// read out and has nowhere to put the bytes, which is upstream's `E222`;
    /// the message is the caller's to emit, because it reaches the message
    /// machinery and this runs inside the cell's borrow.
    ///
    /// # Safety
    /// `s` must point at `slen` readable bytes.
    unsafe fn add(&mut self, s: *const c_char, slen: ptrdiff_t) -> bool {
        let slen = if slen < 0 {
            // SAFETY (this body): `s` is `slen` readable bytes by the caller's
            // promise, and every block reached here is one this buffer
            // allocated -- sized `offset_of!(KeyBlock, bytes) + len + 1`,
            // which is what makes the writes past `bytes` in range.
            unsafe { strlen(s) }
        } else {
            slen as usize
        };
        if slen == 0 {
            return true; // don't add empty strings
        }

        if self.first.is_null() {
            // First add to the list.
            self.at = InsertPoint::Front;
            self.create_newblock = true;
        } else if matches!(self.at, InsertPoint::Gone) {
            return false;
        } else if self.index != 0 {
            // Reclaim what has already been read out of the head block.
            let head = self.first;
            let str = unsafe { block_str(head) };
            let kept = unsafe { (*head).len } - self.index;
            unsafe { ptr::copy(str.add(self.index), str, kept + 1) };
            unsafe { (*head).len = kept };
            self.space += self.index;
        }
        self.index = 0;

        match self.at {
            InsertPoint::Block(curr) if !self.create_newblock && self.space >= slen => {
                unsafe { xmemcpyz(block_str(curr).add((*curr).len).cast(), s.cast(), slen) };
                unsafe { (*curr).len += slen };
                self.space -= slen;
            }
            at => {
                let len = MINIMAL_SIZE.max(slen);
                let block =
                    unsafe { xmalloc(offset_of!(KeyBlock, bytes) + len + 1) }.cast::<KeyBlock>();
                unsafe { xmemcpyz(block_str(block).cast(), s.cast(), slen) };
                unsafe { (*block).len = slen };
                self.space = len - slen;
                self.create_newblock = false;

                // Link it in *after* the insertion point, which is what
                // makes `Front` mean "read this before the rest".
                match at {
                    InsertPoint::Block(curr) => {
                        unsafe { (*block).next = (*curr).next };
                        unsafe { (*curr).next = block };
                    }
                    // `Gone` is impossible: the chain is either empty,
                    // and the first branch above reset `at`, or it is not
                    // and `Gone` returned early.
                    InsertPoint::Front | InsertPoint::Gone => {
                        unsafe { (*block).next = self.first };
                        self.first = block;
                    }
                }
                self.at = InsertPoint::Block(block);
            }
        }
        true
    }

    /// Delete `slen` bytes from the end. Only works when they were just
    /// added: a block boundary in between and nothing happens, which is how
    /// `ungetchars` can fail to take back a key longer than the last one.
    fn delete_tail(&mut self, slen: c_int) {
        let InsertPoint::Block(curr) = self.at else {
            return; // nothing to delete from
        };
        // SAFETY: `at` names a live block of this buffer's own chain.
        if unsafe { (*curr).len } < slen as usize {
            return; // the bytes are not all in the last block
        }
        unsafe { (*curr).len -= slen as usize };
        // SAFETY (this body): `at` names a live block of this buffer's own
        // chain, and the test above showed it holds at least `slen` bytes.
        unsafe { *block_str(curr).add((*curr).len) = 0 };
        self.space += slen as usize;
    }

    /// One byte from the front, advancing past it when `advance` is set.
    ///
    /// # Safety
    /// Nothing may hold a pointer into the block being consumed.
    unsafe fn read(&mut self, advance: bool) -> c_int {
        let curr = self.first;
        if curr.is_null() {
            return NUL; // buffer is empty
        }

        // SAFETY (this body): `curr` is the live head block, whose bytes are
        // NUL-terminated, and `index` is inside it; the block is only freed
        // once its NUL is reached.
        let str = unsafe { block_str(curr) };
        let c = unsafe { *str.add(self.index) } as u8;
        if advance {
            self.index += 1;
            if c_int::from(unsafe { *str.add(self.index) }) == NUL {
                self.first = unsafe { (*curr).next };
                if matches!(self.at, InsertPoint::Block(at) if at == curr) {
                    // Upstream leaves `bh_curr` pointing at the block it
                    // just freed, and the next add writes through it.
                    // Fall back to the front, which is where `restart`
                    // would have put the cursor anyway.
                    self.at = InsertPoint::Front;
                    self.create_newblock = true;
                }
                unsafe { xfree(curr.cast()) };
                self.index = 0;
            }
        }
        c_int::from(c)
    }

    /// Prepare the buffer for reading, if it holds anything.
    fn restart(&mut self) {
        if !self.first.is_null() {
            self.at = InsertPoint::Front;
            // Force a new block to be created; see `add`.
            self.create_newblock = true;
        }
    }
}

/// [`KeyBuffer::add`] sizes a block as `offset_of!(KeyBlock, bytes) + len +
/// 1`, which only describes the allocation while `bytes` is the last field —
/// a layout that put it anywhere else would make every append overwrite
/// `next`/`len`. `#[repr(C)]` on [`KeyBlock`] is what holds that. Drop the
/// attribute and rustc is free to reorder, but only *some* layouts it then
/// picks are wrong — so this fails the build on exactly the ones that are,
/// which is what `-Zrandomize-layout` shakes out (verified: without the
/// attribute, that flag trips this assertion rather than the suite).
const _: () = {
    let tail = offset_of!(KeyBlock, bytes);
    assert!(tail > offset_of!(KeyBlock, next));
    assert!(tail > offset_of!(KeyBlock, len));
};

/// One of the five key buffers, *named* rather than pointed at.
///
/// `Copy`, and holding a `&'static` to the cell rather than a pointer into
/// it, so making one needs no `unsafe` and every operation takes its own
/// short borrow. Each method below is a leaf — the one call-out, `E222`'s
/// message, is deliberately made after the borrow ends.
#[derive(Clone, Copy)]
pub(crate) struct KeyBufferRef(&'static GlobalCell<KeyBuffer>);

/// The redo buffer: the keys `.` replays.
pub(crate) fn redobuff() -> KeyBufferRef {
    KeyBufferRef(&REDOBUFF)
}

/// The redo buffer before this command, which `CTRL-O .` replays.
pub(crate) fn old_redobuff() -> KeyBufferRef {
    KeyBufferRef(&OLD_REDOBUFF)
}

/// The register being recorded into by `q`.
pub(crate) fn recordbuff() -> KeyBufferRef {
    KeyBufferRef(&RECORDBUFF)
}

/// First read-ahead buffer, for translated commands.
pub(crate) fn readbuf1() -> KeyBufferRef {
    KeyBufferRef(&READBUF1)
}

/// Second read-ahead buffer, for redo.
pub(crate) fn readbuf2() -> KeyBufferRef {
    KeyBufferRef(&READBUF2)
}

impl KeyBufferRef {
    /// Append `s`, which must already have `K_SPECIAL` escaped. `slen` is the
    /// length, or -1 for a NUL-terminated string.
    ///
    /// # Safety
    /// `s` must point at `slen` readable bytes.
    pub(crate) unsafe fn add(self, s: *const c_char, slen: ptrdiff_t) {
        // SAFETY: the caller's obligation, forwarded.
        if !self.0.with_mut(|buf| unsafe { buf.add(s, slen) }) {
            // Outside the borrow: `iemsg` reaches the message machinery.
            // SAFETY: a static string.
            unsafe { iemsg(gettext(c"E222: Add to read buffer".as_ptr())) };
        }
    }

    /// Append the decimal spelling of `n`.
    pub(crate) fn add_num(self, n: c_int) {
        let mut number = [0u8; 32];
        let len = write_int(&mut number, n);
        // SAFETY: `number[..len]` is what `write_int` just filled in.
        unsafe { self.add(number.as_ptr().cast(), len as ptrdiff_t) };
    }

    /// Append byte or special key `c`, escaping special keys, NUL and
    /// `K_SPECIAL`.
    pub(crate) fn add_byte(self, c: c_int) {
        let mut temp = [0u8; 4];
        let templen = if c < 0 || c == K_SPECIAL || c == NUL {
            temp[..3].copy_from_slice(&key_escape(c));
            3
        } else {
            temp[0] = c as u8;
            1
        };
        // SAFETY: `temp[..templen]` was just filled in.
        unsafe { self.add(temp.as_ptr().cast(), templen) };
    }

    /// Append character `c`, escaping special keys, NUL and `K_SPECIAL` and
    /// splitting a codepoint into its UTF-8 bytes.
    pub(crate) fn add_char(self, c: c_int) {
        if c < 0 {
            // A special key is one unit; it has no UTF-8 spelling.
            self.add_byte(c);
            return;
        }
        let mut bytes = [0u8; MB_MAXBYTES + 1];
        // SAFETY: `bytes` is the buffer `utf_char2bytes` documents.
        let len = unsafe { utf_char2bytes(c, bytes.as_mut_ptr().cast()) } as usize;
        for &byte in &bytes[..len] {
            self.add_byte(c_int::from(byte));
        }
    }

    /// Delete `slen` bytes from the end; see [`KeyBuffer::delete_tail`].
    pub(crate) fn delete_tail(self, slen: c_int) {
        self.0.with_mut(|buf| buf.delete_tail(slen));
    }

    /// Free every block.
    ///
    /// # Safety
    /// Nothing may hold a pointer into the chain.
    pub(crate) unsafe fn free(self) {
        // SAFETY: the caller's obligation, forwarded.
        self.0.with_mut(|buf| unsafe { buf.free() });
    }

    /// The contents as one `xmalloc`ed string; see [`KeyBuffer::contents`].
    ///
    /// # Safety
    /// The answer must be freed with `xfree`.
    pub(crate) unsafe fn contents(self, dozero: bool) -> (*mut c_char, usize) {
        // SAFETY: the caller's obligation, forwarded.
        self.0.with(|buf| unsafe { buf.contents(dozero) })
    }

    /// One byte from the front; see [`KeyBuffer::read`].
    ///
    /// # Safety
    /// Nothing may hold a pointer into the block being consumed.
    pub(crate) unsafe fn read(self, advance: bool) -> c_int {
        // SAFETY: the caller's obligation, forwarded.
        self.0.with_mut(|buf| unsafe { buf.read(advance) })
    }

    /// Prepare the buffer for reading, if it holds anything.
    pub(crate) fn restart(self) {
        self.0.with_mut(KeyBuffer::restart);
    }

    /// Whether the buffer holds nothing.
    pub(crate) fn is_empty(self) -> bool {
        self.0.with(KeyBuffer::is_empty)
    }

    /// The first block, for the redo walk.
    pub(crate) fn head(self) -> *mut KeyBlock {
        self.0.with(KeyBuffer::head)
    }

    /// Move the buffer out of its cell, leaving an empty one behind.
    pub(crate) fn take(self) -> KeyBuffer {
        self.0.take()
    }

    /// Install `buf`. Whatever the cell held is dropped on the floor, so it
    /// must have been freed or moved out first.
    pub(crate) fn set(self, buf: KeyBuffer) {
        self.0.with_mut(|slot| *slot = buf);
    }
}

/// The contents of the record buffer as one string, clearing the buffer.
///
/// `K_SPECIAL` in the answer is escaped. The caller owns the string.
///
/// # Safety
/// Callable at any time; the answer must be freed with `xfree`.
pub unsafe fn get_recorded() -> *mut c_char {
    // SAFETY (this body): `contents` answers an `xmalloc`ed string of `len`
    // bytes plus a NUL, which this frame owns and hands to the caller.
    let (recorded, mut len) = unsafe { recordbuff().contents(true) };
    if recorded.is_null() {
        return ptr::null_mut();
    }
    unsafe { recordbuff().free() };

    // Drop the characters added last, which must be the (possibly mapped)
    // keys that stopped the recording.
    if len >= last_recorded_len.get() {
        len -= last_recorded_len.get();
        unsafe { *recorded.add(len) = 0 };
    }
    // Stopping a recording from Insert mode with CTRL-O q also leaves the
    // CTRL-O behind.
    if len > 0
        && restart_edit.get() != 0
        && c_int::from(unsafe { *recorded.add(len - 1) }) == Ctrl_O
    {
        unsafe { *recorded.add(len - 1) = 0 };
    }
    recorded
}

/// The contents of the redo buffer as one string, with `K_SPECIAL` escaped.
///
/// # Safety
/// Callable at any time; the answer owns its bytes.
pub unsafe fn get_inserted() -> String_0 {
    // SAFETY (this body): as [`get_recorded`] -- the answer owns its bytes.
    let (data, size) = unsafe { redobuff().contents(false) };
    String_0::from_raw_parts(data, size)
}

/// Write `n` into `out` as decimal digits and answer how many there are.
///
/// `out` is upstream's 32-byte scratch array, which no `c_int` can overrun.
fn write_int(out: &mut [u8; 32], n: c_int) -> usize {
    let negative = n < 0;
    // Build the digits backwards, on the magnitude as a u32 so that
    // `c_int::MIN` is representable.
    let mut magnitude = n.unsigned_abs();
    let mut at = out.len();
    loop {
        at -= 1;
        out[at] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if negative {
        at -= 1;
        out[at] = b'-';
    }
    let len = out.len() - at;
    out.copy_within(at.., 0);
    len
}

/// One byte from the read buffers, `readbuf1` first. No translation is done,
/// so `K_SPECIAL` is still escaped.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn read_readbuffers(advance: bool) -> c_int {
    // SAFETY (this body): the two read buffers are statics, and nothing holds
    // a pointer into the block a read may free.
    let c = unsafe { readbuf1().read(advance) };
    if c == NUL {
        unsafe { readbuf2().read(advance) }
    } else {
        c
    }
}

/// Prepare the read buffers for reading, if they hold anything.
pub(crate) fn start_stuff() {
    readbuf1().restart();
    readbuf2().restart();
}

/// Whether the stuff buffer is empty.
pub fn stuff_empty() -> bool {
    readbuf1().is_empty() && readbuf2().is_empty()
}

/// Whether `readbuf1` is empty. There may still be redo characters in
/// `readbuf2`.
pub fn readbuf1_empty() -> bool {
    readbuf1().is_empty()
}

/// Set a typeahead character that `flush_buffers` will not throw away.
pub fn typeahead_noflush(c: c_int) {
    typeahead_char.set(c);
}

/// Throw away the stuff buffer and the mapped characters in the typeahead
/// buffer, as an error does.
///
/// `FLUSH_INPUT` additionally drains everything the OS has for us, which is
/// what a CTRL-C wants: an escape sequence arrives one byte at a time and
/// leaving half of it behind would make the rest read as literal keys.
///
/// # Safety
/// Callable at any time; may block briefly reading input.
pub unsafe fn flush_buffers(flush_typeahead: flush_buffers_T) {
    init_typebuf();

    start_stuff();
    // SAFETY (this body): the typeahead is initialised just above, and
    // `inchar` is given its own storage and the room left in it.
    while unsafe { read_readbuffers(true) } != NUL {}

    if flush_typeahead == FLUSH_INPUT {
        // Drain what the OS has for us as well, before the typeahead's
        // bounds move under `inchar`.
        let tb = typeahead();
        while unsafe { inchar(tb.storage(), tb.buflen() - 1, 10) } != 0 {}
    }
    flush_typebuf(flush_typeahead == FLUSH_MINIMAL);
}

/// Flush the map and typeahead buffers and beep about an error.
///
/// Safe: the only promise is that the editor exists.
pub fn beep_flush() {
    if emsg_silent.get() == 0 {
        // SAFETY (this body): both callees only read the editor's own state.
        unsafe { flush_buffers(FLUSH_MINIMAL) };
        unsafe { vim_beep(kOptBoFlagError as c_uint) };
    }
}

/// Stuff a NUL-terminated string into `readbuf1`, to be read back as keys.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuff_readbuf(s: *const c_char) {
    // SAFETY (this body): the caller's promise -- a NUL-terminated string,
    // which `add` copies.
    unsafe { readbuf1().add(s, -1) };
}

/// Stuff a NUL-terminated string into the redo read buffer.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuff_redo_readbuf(s: *const c_char) {
    // SAFETY (this body): as [`stuff_readbuf`].
    unsafe { readbuf2().add(s, -1) };
}

/// Stuff `len` bytes into `readbuf1`.
///
/// # Safety
/// `s` must point at `len` readable bytes.
pub unsafe fn stuff_readbuf_len(s: *const c_char, len: ptrdiff_t) {
    // SAFETY (this body): the caller's promise -- `len` readable bytes, which
    // `add` copies.
    unsafe { readbuf1().add(s, len) };
}

/// Stuff a string into `readbuf1`, replacing the characters that would end a
/// command line — CR, NL and ESC — with a space.
///
/// Used for `:normal` and for the `@` register: an embedded CR would
/// terminate whatever is reading the stuffed text.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuff_readbuf_one_line(mut s: *const c_char) {
    // SAFETY (this body): the caller's promise -- `s` is NUL-terminated -- and
    // the walk stops at its NUL, so the two bytes after a `K_SPECIAL` are only
    // read once the one before them is known non-NUL.
    while c_int::from(unsafe { *s }) != NUL {
        if c_int::from(unsafe { *s } as u8) == K_SPECIAL
            && c_int::from(unsafe { *s.add(1) }) != NUL
            && c_int::from(unsafe { *s.add(2) }) != NUL
        {
            // Copy an escaped key code through untouched.
            unsafe { stuff_readbuf_len(s, 3) };
            s = unsafe { s.add(3) };
        } else {
            let c = unsafe { mb_cptr2char_adv(&raw mut s) };
            stuff_readbuf_char(if c == CAR || c == NL || c == ESC {
                ' ' as c_int
            } else {
                c
            });
        }
    }
}

/// Stuff one character into `readbuf1`.
pub fn stuff_readbuf_char(c: c_int) {
    readbuf1().add_char(c);
}

/// Stuff the decimal spelling of `n` into `readbuf1`.
pub fn stuff_readbuf_number(n: c_int) {
    readbuf1().add_num(n);
}

/// Stuff `arg` into `readbuf1`, one printable run at a time.
///
/// With `literally` set, a control character is preceded by CTRL-V so that
/// whatever reads the keys inserts it rather than acting on it; `K_SPECIAL`
/// is then left alone as well, because the text really contains that byte.
///
/// # Safety
/// `arg` must point at a NUL-terminated string.
pub unsafe fn stuffescaped(mut arg: *const c_char, literally: bool) {
    // SAFETY (this body): the caller's promise -- `arg` is NUL-terminated --
    // and both walks stop at its NUL, so `start..arg` is always a run inside
    // it.
    while c_int::from(unsafe { *arg }) != NUL {
        // Stuff a run of ordinary characters in one go.
        let start = arg;
        while (c_int::from(unsafe { *arg }) >= ' ' as c_int && c_int::from(unsafe { *arg }) < DEL)
            || (c_int::from(unsafe { *arg } as u8) == K_SPECIAL && !literally)
        {
            arg = unsafe { arg.add(1) };
        }
        if arg > start {
            unsafe { stuff_readbuf_len(start, arg.offset_from(start)) };
        }

        // Then the character that stopped it, one at a time.
        if c_int::from(unsafe { *arg }) != NUL {
            let c = unsafe { mb_cptr2char_adv(&raw mut arg) };
            if literally && ((c < ' ' as c_int && c != TAB) || c == DEL) {
                stuff_readbuf_char(Ctrl_V);
            }
            stuff_readbuf_char(c);
        }
    }
}
