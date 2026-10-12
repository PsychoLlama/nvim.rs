//! The stuff/read/record buffers: [`KeyBuffer`] and its blocks.
//!
//! A `KeyBuffer` holds a byte string of already-escaped keys, appended to at
//! the insertion point ([`KeyBuffer::add`]) and consumed from the front
//! ([`KeyBuffer::read`]). Five of them exist — the two read buffers behind
//! [`stuff_readbuf`], the record buffer behind `q`, and the redo pair — and
//! every one of them is filled by the methods here.
//!
//! # What changed against upstream
//!
//! Upstream's `buffheader_T` is a linked list of `buffblock_T`s, each a
//! flexible array member, with an *inline* sentinel block in the header that
//! the insertion cursor `bh_curr` may point at -- that is, into the header
//! itself. Here the blocks are owned `Vec<u8>`s in a `VecDeque`, and the
//! cursor is an [`InsertPoint`] that names a block by its index or spells
//! "before everything" as a value. The *shape* of the chain is kept, block
//! for block: a block still takes `MINIMAL_SIZE` bytes or the first add's
//! length, whichever is more, and an add that does not fit starts the next
//! one -- because [`KeyBuffer::delete_tail`] can only take back bytes that
//! are all in the last block, and `ungetchars` observes exactly that.
//!
//! `KeyBuffer` is deliberately **not `Copy`**: moving one out of its cell
//! ([`GlobalCell::take`](crate::global_cell::GlobalCell)) is how
//! `save_redobuff` and `save_typeahead` set one aside.

#![forbid(unsafe_code)]

use super::*;
use crate::keycodes::{Ctrl_O, Ctrl_V, key_escape};
use crate::mbyte::{char_at, char_len, encode_char};
use crate::types::{MB_MAXBYTES, NUL};
use core::ffi::{c_int, c_uint};
use std::collections::VecDeque;

/// Smallest block [`KeyBuffer::add`] will start; upstream's
/// `MINIMAL_SIZE`.
const MINIMAL_SIZE: usize = 20;

/// Where a [`KeyBuffer`] puts the bytes it is given next.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InsertPoint {
    /// Before everything the buffer already holds. What [`KeyBuffer::restart`]
    /// sets while the buffer is being read, so that text stuffed mid-read is
    /// read *before* the remainder — upstream's `bh_curr = &bh_first`.
    Front,
    /// At the end of the block at this index, while it still has
    /// [`KeyBuffer::space`].
    Block(usize),
    /// Nowhere: the buffer was freed and has no cursor. Adding to it is the
    /// internal error `E222`.
    Gone,
}

/// One key buffer: a byte string of already-escaped keys.
pub(crate) struct KeyBuffer {
    /// The blocks, front first.
    blocks: VecDeque<Vec<u8>>,
    /// Where the next bytes go.
    at: InsertPoint,
    /// How far [`KeyBuffer::read`] has got into the first block.
    index: usize,
    /// Bytes still unused in the block `at` names: what upstream's
    /// allocation had left, which decides when the next block starts.
    space: usize,
    /// Start a new block on the next add rather than fill `space`.
    create_newblock: bool,
}

impl KeyBuffer {
    /// The state all five buffers start in, and the one `GlobalCell::take`
    /// leaves behind: no blocks. A `const` as well as a [`Default`], because
    /// the cells themselves are `static`s.
    pub(crate) const EMPTY: Self = KeyBuffer {
        blocks: VecDeque::new(),
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

/// A block's bytes up to its first NUL: upstream's blocks are C strings,
/// and every reader stops at the terminator rather than at the length.
pub(crate) fn up_to_nul(block: &[u8]) -> &[u8] {
    block
        .iter()
        .position(|&b| b == 0)
        .map_or(block, |end| &block[..end])
}

impl KeyBuffer {
    /// Whether the buffer holds nothing.
    fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// The contents as owned bytes, `K_SPECIAL` still escaped, every block
    /// from its start (what has been read out of the first one included, as
    /// upstream's `get_buffcont` reads it).
    pub(crate) fn bytes(&self) -> Vec<u8> {
        // `save_redobuff` copies the redo buffer at every user function
        // call, nearly always an empty one: that costs no allocation, and a
        // full one gets room for a typical redo up front.
        if self.blocks.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(64);
        for block in &self.blocks {
            out.extend_from_slice(up_to_nul(block));
        }
        out
    }

    /// Drop every block and leave the buffer with no insertion point.
    fn free(&mut self) {
        self.blocks.clear();
        self.at = InsertPoint::Gone;
    }

    /// Append `s` at the insertion point. `K_SPECIAL` must have been escaped
    /// already.
    ///
    /// Answers false when the buffer has been read out and has nowhere to
    /// put the bytes, which is upstream's `E222`; the message is the caller's
    /// to emit, because it reaches the message machinery and this runs inside
    /// the cell's borrow.
    fn add(&mut self, s: &[u8]) -> bool {
        if s.is_empty() {
            return true; // don't add empty strings
        }

        if self.blocks.is_empty() {
            // First add to the list.
            self.at = InsertPoint::Front;
            self.create_newblock = true;
        } else if self.at == InsertPoint::Gone {
            return false;
        } else if self.index != 0 {
            // Reclaim what has already been read out of the head block.
            if let Some(head) = self.blocks.front_mut() {
                head.drain(..self.index.min(head.len()));
            }
            self.space += self.index;
        }
        self.index = 0;

        match self.at {
            InsertPoint::Block(curr) if !self.create_newblock && self.space >= s.len() => {
                self.blocks[curr].extend_from_slice(s);
                self.space -= s.len();
            }
            at => {
                let len = MINIMAL_SIZE.max(s.len());
                let mut block = Vec::with_capacity(len);
                block.extend_from_slice(s);
                self.space = len - s.len();
                self.create_newblock = false;

                // Link it in *after* the insertion point, which is what
                // makes `Front` mean "read this before the rest".
                let at = match at {
                    InsertPoint::Block(curr) => curr + 1,
                    // `Gone` is impossible: the chain is either empty, and
                    // the first branch above reset `at`, or it is not and
                    // `Gone` returned early.
                    InsertPoint::Front | InsertPoint::Gone => 0,
                };
                self.blocks.insert(at, block);
                self.at = InsertPoint::Block(at);
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
        let Ok(slen) = usize::try_from(slen) else {
            return;
        };
        let block = &mut self.blocks[curr];
        if block.len() < slen {
            return; // the bytes are not all in the last block
        }
        block.truncate(block.len() - slen);
        self.space += slen;
    }

    /// One byte from the front, advancing past it when `advance` is set.
    fn read(&mut self, advance: bool) -> c_int {
        let Some(head) = self.blocks.front() else {
            return NUL; // buffer is empty
        };
        let at = |i: usize| head.get(i).copied().unwrap_or(0);
        let c = at(self.index);
        if advance {
            self.index += 1;
            if at(self.index) == 0 {
                // The head block is used up: drop it, and move the cursor
                // with the blocks behind it.
                self.blocks.pop_front();
                self.at = match self.at {
                    // Upstream leaves `bh_curr` pointing at the block it
                    // just freed, and the next add writes through it. Fall
                    // back to the front, which is where `restart` would
                    // have put the cursor anyway.
                    InsertPoint::Block(0) => {
                        self.create_newblock = true;
                        InsertPoint::Front
                    }
                    InsertPoint::Block(i) => InsertPoint::Block(i - 1),
                    at => at,
                };
                self.index = 0;
            }
        }
        c_int::from(c)
    }

    /// Prepare the buffer for reading, if it holds anything.
    fn restart(&mut self) {
        if !self.blocks.is_empty() {
            self.at = InsertPoint::Front;
            // Force a new block to be created; see `add`.
            self.create_newblock = true;
        }
    }
}

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
    /// Append `bytes`, which must already have `K_SPECIAL` escaped.
    pub(crate) fn add_bytes(self, bytes: &[u8]) {
        if bytes.is_empty() {
            return; // what `add` does too, without the borrow
        }
        if !self.0.with_mut(|buf| buf.add(bytes)) {
            // Outside the borrow: `iemsg` reaches the message machinery.
            iemsg(gettext(c"E222: Add to read buffer"));
        }
    }

    /// Append the decimal spelling of `n`.
    pub(crate) fn add_num(self, n: c_int) {
        let mut number = [0u8; 32];
        let len = write_int(&mut number, n);
        self.add_bytes(&number[..len]);
    }

    /// Append byte or special key `c`, escaping special keys, NUL and
    /// `K_SPECIAL`.
    pub(crate) fn add_byte(self, c: c_int) {
        if c < 0 || c == K_SPECIAL || c == NUL {
            self.add_bytes(&key_escape(c));
        } else {
            self.add_bytes(&[c.to_le_bytes()[0]]);
        }
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
        let len = encode_char(c, &mut bytes);
        for &byte in &bytes[..len] {
            self.add_byte(c_int::from(byte));
        }
    }

    /// Delete `slen` bytes from the end; see [`KeyBuffer::delete_tail`].
    pub(crate) fn delete_tail(self, slen: c_int) {
        self.0.with_mut(|buf| buf.delete_tail(slen));
    }

    /// Drop every block.
    pub(crate) fn free(self) {
        self.0.with_mut(KeyBuffer::free);
    }

    /// One byte from the front; see [`KeyBuffer::read`].
    pub(crate) fn read(self, advance: bool) -> c_int {
        self.0.with_mut(|buf| buf.read(advance))
    }

    /// Prepare the buffer for reading, if it holds anything.
    pub(crate) fn restart(self) {
        self.0.with_mut(KeyBuffer::restart);
    }

    /// Whether the buffer holds nothing.
    pub(crate) fn is_empty(self) -> bool {
        self.0.with(KeyBuffer::is_empty)
    }

    /// The contents as owned bytes; see [`KeyBuffer::bytes`].
    pub(crate) fn bytes(self) -> Vec<u8> {
        self.0.with(KeyBuffer::bytes)
    }

    /// Move the buffer out of its cell, leaving an empty one behind.
    pub(crate) fn take(self) -> KeyBuffer {
        self.0.take()
    }

    /// Install `buf`, dropping whatever the cell held.
    pub(crate) fn set(self, buf: KeyBuffer) {
        self.0.with_mut(|slot| *slot = buf);
    }
}

/// The contents of the record buffer, clearing the buffer.
///
/// `K_SPECIAL` in the answer is escaped.
pub fn get_recorded() -> Vec<u8> {
    let mut recorded = recordbuff().bytes();
    recordbuff().free();

    // Drop the characters added last, which must be the (possibly mapped)
    // keys that stopped the recording.
    let mut len = recorded.len();
    if len >= last_recorded_len.get() {
        len -= last_recorded_len.get();
    }
    // Stopping a recording from Insert mode with CTRL-O q also leaves the
    // CTRL-O behind.
    if len > 0 && restart_edit.get() != 0 && c_int::from(recorded[len - 1]) == Ctrl_O {
        len -= 1;
    }
    recorded.truncate(len);
    recorded
}

/// The contents of the redo buffer, with `K_SPECIAL` escaped:
/// [`String_0::NULL`] when it is empty.
pub fn get_inserted() -> String_0 {
    let bytes = redobuff().bytes();
    if bytes.is_empty() {
        return String_0::NULL;
    }
    String_0::from_bytes(&bytes)
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
        out[at] = b'0' + u8::try_from(magnitude % 10).expect("a digit");
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
pub(crate) fn read_readbuffers(advance: bool) -> c_int {
    let c = readbuf1().read(advance);
    if c == NUL {
        readbuf2().read(advance)
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
pub fn flush_buffers(flush_typeahead: FlushBuffers) {
    init_typebuf();

    start_stuff();
    while read_readbuffers(true) != NUL {}

    if flush_typeahead == FLUSH_INPUT {
        // Drain what the OS has for us as well, into a buffer as long as
        // the typeahead's storage, which is what upstream reads into.
        let mut drain = vec![0; typeahead().buflen()];
        while inchar(&mut drain, 10) != 0 {}
    }
    flush_typebuf(flush_typeahead == FLUSH_MINIMAL);
}

/// Flush the map and typeahead buffers and beep about an error.
pub fn beep_flush() {
    if emsg_silent.get() == 0 {
        flush_buffers(FLUSH_MINIMAL);
        vim_beep(kOptBoFlagError as c_uint);
    }
}

/// Stuff `s` into `readbuf1`, to be read back as keys. It is read up to its
/// first NUL, as upstream's `char *` is.
pub fn stuff_readbuf(s: &[u8]) {
    readbuf1().add_bytes(up_to_nul(s));
}

/// Stuff `s` into the redo read buffer, up to its first NUL.
pub fn stuff_redo_readbuf(s: &[u8]) {
    readbuf2().add_bytes(up_to_nul(s));
}

/// Stuff `s` into `readbuf1` whole, NULs and all: upstream's
/// `stuffReadbuffLen`, for a caller that holds a length.
pub fn stuff_readbuf_len(s: &[u8]) {
    readbuf1().add_bytes(s);
}

/// Stuff `s` into `readbuf1`, replacing the characters that would end a
/// command line — CR, NL and ESC — with a space. `s` is read up to its
/// first NUL.
///
/// Used for `:normal` and for the `@` register: an embedded CR would
/// terminate whatever is reading the stuffed text.
pub fn stuff_readbuf_one_line(s: &[u8]) {
    let mut s = up_to_nul(s);
    while let Some(&first) = s.first() {
        if c_int::from(first) == K_SPECIAL && s.len() >= 3 {
            // Copy an escaped key code through untouched.
            stuff_readbuf_len(&s[..3]);
            s = &s[3..];
        } else {
            let c = char_at(s);
            s = &s[char_len(s)..];
            stuff_readbuf_char(if c == CAR || c == NL || c == ESC {
                c_int::from(b' ')
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

/// Stuff `arg` into `readbuf1`, one printable run at a time. `arg` is read
/// up to its first NUL.
///
/// With `literally` set, a control character is preceded by CTRL-V so that
/// whatever reads the keys inserts it rather than acting on it; `K_SPECIAL`
/// is then left alone as well, because the text really contains that byte.
pub fn stuffescaped(arg: &[u8], literally: bool) {
    let mut arg = up_to_nul(arg);
    while !arg.is_empty() {
        // Stuff a run of ordinary characters in one go.
        let run = arg
            .iter()
            .position(|&b| {
                !((b' '..DEL as u8).contains(&b) || (c_int::from(b) == K_SPECIAL && !literally))
            })
            .unwrap_or(arg.len());
        if run > 0 {
            stuff_readbuf_len(&arg[..run]);
        }
        arg = &arg[run..];

        // Then the character that stopped it, one at a time.
        if !arg.is_empty() {
            let c = char_at(arg);
            arg = &arg[char_len(arg)..];
            if literally && ((c < c_int::from(b' ') && c != TAB) || c == DEL) {
                stuff_readbuf_char(Ctrl_V);
            }
            stuff_readbuf_char(c);
        }
    }
}

#[cfg(test)]
mod tests {
    //! The key buffers' block shape, observed through the entry points the
    //! editor uses.

    use super::*;
    use crate::global_cell::{editor_state::Held, editor_state_lock};

    struct Fresh(#[allow(dead_code, reason = "held for its drop")] Held);

    impl Fresh {
        fn new() -> Self {
            let held = editor_state_lock();
            clear();
            Fresh(held)
        }
    }

    impl Drop for Fresh {
        fn drop(&mut self) {
            clear();
        }
    }

    fn clear() {
        start_stuff();
        while read_readbuffers(true) != NUL {}
        recordbuff().free();
        recordbuff().set(KeyBuffer::EMPTY);
    }

    fn drain() -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let c = read_readbuffers(true);
            if c == NUL {
                return out;
            }
            out.push(u8::try_from(c).expect("a read buffer holds bytes"));
        }
    }

    #[test]
    fn text_stuffed_mid_read_is_read_first() {
        let _fresh = Fresh::new();
        stuff_readbuf(b"abc");
        start_stuff();
        assert_eq!(read_readbuffers(true), c_int::from(b'a'));
        // Stuffed while reading: in front of what is left, in order.
        stuff_readbuf(b"XY");
        stuff_readbuf(b"Z");
        assert_eq!(read_readbuffers(false), c_int::from(b'X'));
        assert_eq!(drain(), b"XYZbc");
    }

    #[test]
    fn a_nul_ends_what_is_stuffed() {
        let _fresh = Fresh::new();
        stuff_readbuf(b"ab\0cd");
        stuff_redo_readbuf(b"r\0s");
        assert_eq!(drain(), b"abr");
    }

    #[test]
    fn delete_tail_only_reaches_the_last_block() {
        let _fresh = Fresh::new();
        // 19 bytes and then 3: the second add does not fit the first
        // block's 20 and starts a block of its own.
        recordbuff().add_bytes(&[b'a'; 19]);
        recordbuff().add_bytes(b"xyz");
        recordbuff().delete_tail(4);
        assert_eq!(recordbuff().bytes().len(), 22, "spans two blocks: kept");
        recordbuff().delete_tail(3);
        assert_eq!(recordbuff().bytes(), [b'a'; 19]);
        // Into the space the first block still has: one block.
        recordbuff().add_bytes(b"q");
        recordbuff().delete_tail(20);
        let mut kept = vec![b'a'; 19];
        kept.push(b'q');
        assert_eq!(recordbuff().bytes(), kept);
    }

    #[test]
    fn escaped_runs_and_one_line_stuffing() {
        let _fresh = Fresh::new();
        stuffescaped(b"ab\x01c\x80\xfe\x58", true);
        assert_eq!(drain(), b"ab\x16\x01c\xc2\x80\xfe\x58\xc3\xbeX");
        stuff_readbuf_one_line(b"a\rb\x1b\x80kl\n");
        assert_eq!(drain(), b"a b \x80kl ");
    }

    #[test]
    fn recorded_keys_drop_what_stopped_the_recording() {
        let _fresh = Fresh::new();
        recordbuff().add_bytes(b"ihello");
        recordbuff().add_bytes(b"q");
        last_recorded_len.set(1);
        assert_eq!(get_recorded(), b"ihello");
        // Read out: nothing recorded is still a string.
        last_recorded_len.set(0);
        assert_eq!(get_recorded(), b"");
    }
}
