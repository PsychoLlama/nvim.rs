//! The stuff/read/record buffers: [`buffheader_T`] and its chain.
//!
//! A `buffheader_T` is a linked list of [`buffblock_T`]s holding a byte string
//! that is appended to at the tail ([`add_buff`]) and consumed from the head
//! ([`read_readbuf`]).  Five of them exist — the two read buffers behind
//! `stuffReadbuff`, the record buffer behind `q`, and the redo pair — and
//! every one of them is filled by the functions here.
//!
//! `buffblock_T::b_str` is a flexible array member: the type declares one
//! byte, the allocation holds as many as the block was sized for. Every read
//! of it goes through [`block_str`], which forms the pointer from the field's
//! address so that it inherits the whole allocation's provenance — taking a
//! slice of the declared `[c_char; 1]` would cover one byte.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_O, Ctrl_V, key_escape};
use crate::types::MB_MAXBYTES;
use core::ffi::{c_char, c_int, c_uint};
use core::mem::offset_of;
use core::ptr;

/// Smallest block `add_buff` will allocate; upstream's `MINIMAL_SIZE`.
const MINIMAL_SIZE: usize = 20;

/// The bytes of one block, as a pointer over the whole allocation.
///
/// # Safety
/// `block` must point at a live block allocated by [`add_buff`].
pub(crate) unsafe fn block_str(block: *mut buffblock_T) -> *mut c_char {
    unsafe { (&raw mut (*block).b_str).cast::<c_char>() }
}

/// Walk the blocks of `buf`, head first.
///
/// Each block's successor is read *before* the block is handed out, so a
/// caller may free what it is given — which is what [`free_buff`] does.
///
/// # Safety
/// `buf` must point at a live buffer whose chain nothing else is mutating.
unsafe fn blocks(buf: *const buffheader_T) -> impl Iterator<Item = *mut buffblock_T> {
    let mut next = unsafe { (*buf).bh_first.b_next };
    core::iter::from_fn(move || {
        let block = next;
        if block.is_null() {
            return None;
        }
        next = unsafe { (*block).b_next };
        Some(block)
    })
}

/// Free and clear a buffer.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn free_buff(buf: *mut buffheader_T) {
    unsafe {
        for block in blocks(buf) {
            xfree(block.cast());
        }
        (*buf).bh_first.b_next = ptr::null_mut();
        (*buf).bh_curr = ptr::null_mut();
    }
}

/// The contents of a buffer as one `xmalloc`ed NUL-terminated string, with
/// its length. `K_SPECIAL` in the answer is escaped.
///
/// Answers a null pointer when the buffer is empty and `dozero` is false.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn buff_contents(buf: *mut buffheader_T, dozero: bool) -> (*mut c_char, usize) {
    unsafe {
        let mut count = 0;
        for block in blocks(buf) {
            count += (*block).b_strlen;
        }
        if count == 0 && !dozero {
            return (ptr::null_mut(), 0);
        }

        let out = xmalloc(count + 1).cast::<c_char>();
        let mut at = 0;
        for block in blocks(buf) {
            // Copy up to the block's own terminator rather than up to
            // `b_strlen`: `delete_buff_tail` shortens a block by moving the
            // NUL, and upstream reads the NUL here too.
            let str = block_str(block);
            let mut i = 0;
            while *str.add(i) != 0 {
                *out.add(at) = *str.add(i);
                at += 1;
                i += 1;
            }
        }
        *out.add(at) = 0;
        (out, at)
    }
}

/// The contents of the record buffer as one string, clearing the buffer.
///
/// `K_SPECIAL` in the answer is escaped. The caller owns the string.
///
/// # Safety
/// Callable at any time; the answer must be freed with `xfree`.
pub unsafe fn get_recorded() -> *mut c_char {
    unsafe {
        let (recorded, mut len) = buff_contents(recordbuff.ptr(), true);
        if recorded.is_null() {
            return ptr::null_mut();
        }
        free_buff(recordbuff.ptr());

        // Drop the characters added last, which must be the (possibly mapped)
        // keys that stopped the recording.
        if len >= last_recorded_len.get() {
            len -= last_recorded_len.get();
            *recorded.add(len) = 0;
        }
        // Stopping a recording from Insert mode with CTRL-O q also leaves the
        // CTRL-O behind.
        if len > 0 && restart_edit.get() != 0 && c_int::from(*recorded.add(len - 1)) == Ctrl_O {
            *recorded.add(len - 1) = 0;
        }
        recorded
    }
}

/// The contents of the redo buffer as one string, with `K_SPECIAL` escaped.
///
/// # Safety
/// Callable at any time; the answer owns its bytes.
pub unsafe fn get_inserted() -> String_0 {
    let (data, size) = unsafe { buff_contents(redobuff.ptr(), false) };
    String_0 { data, size }
}

/// Append `s` to `buf` after its current block.
///
/// `K_SPECIAL` must have been escaped already. `slen` is the length, or -1 for
/// a NUL-terminated string.
///
/// # Safety
/// `buf` must point at a live buffer and `s` at `slen` readable bytes.
pub(crate) unsafe fn add_buff(buf: *mut buffheader_T, s: *const c_char, slen: ptrdiff_t) {
    unsafe {
        let slen = if slen < 0 { strlen(s) } else { slen as usize };
        if slen == 0 {
            return; // don't add empty strings
        }

        if (*buf).bh_first.b_next.is_null() {
            // First add to the list.
            (*buf).bh_curr = &raw mut (*buf).bh_first;
            (*buf).bh_create_newblock = true;
        } else if (*buf).bh_curr.is_null() {
            iemsg(gettext(c"E222: Add to read buffer".as_ptr()));
            return;
        } else if (*buf).bh_index != 0 {
            // Reclaim what has already been read out of the head block.
            let head = (*buf).bh_first.b_next;
            let str = block_str(head);
            let kept = (*head).b_strlen - (*buf).bh_index;
            ptr::copy(str.add((*buf).bh_index), str, kept + 1);
            (*head).b_strlen = kept;
            (*buf).bh_space += (*buf).bh_index;
        }
        (*buf).bh_index = 0;

        if !(*buf).bh_create_newblock && (*buf).bh_space >= slen {
            let curr = (*buf).bh_curr;
            xmemcpyz(block_str(curr).add((*curr).b_strlen).cast(), s.cast(), slen);
            (*curr).b_strlen += slen;
            (*buf).bh_space -= slen;
        } else {
            let len = MINIMAL_SIZE.max(slen);
            let block = xmalloc(offset_of!(buffblock_T, b_str) + len + 1).cast::<buffblock_T>();
            xmemcpyz(block_str(block).cast(), s.cast(), slen);
            (*block).b_strlen = slen;
            (*buf).bh_space = len - slen;
            (*buf).bh_create_newblock = false;

            (*block).b_next = (*(*buf).bh_curr).b_next;
            (*(*buf).bh_curr).b_next = block;
            (*buf).bh_curr = block;
        }
    }
}

/// Delete `slen` bytes from the end of `buf`. Only works when they were just
/// added.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn delete_buff_tail(buf: *mut buffheader_T, slen: c_int) {
    unsafe {
        let curr = (*buf).bh_curr;
        if curr.is_null() || (*curr).b_strlen < slen as usize {
            return; // nothing to delete
        }
        (*curr).b_strlen -= slen as usize;
        *block_str(curr).add((*curr).b_strlen) = 0;
        (*buf).bh_space += slen as usize;
    }
}

/// Append the decimal spelling of `n` to `buf`.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn add_num_buff(buf: *mut buffheader_T, n: c_int) {
    let mut number = [0u8; 32];
    let len = write_int(&mut number, n);
    unsafe { add_buff(buf, number.as_ptr().cast(), len as ptrdiff_t) };
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

/// Append byte or special key `c` to `buf`, escaping special keys, NUL and
/// `K_SPECIAL`.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn add_byte_buff(buf: *mut buffheader_T, c: c_int) {
    let mut temp = [0u8; 4];
    let templen = if c < 0 || c == K_SPECIAL || c == NUL {
        temp[..3].copy_from_slice(&key_escape(c));
        3
    } else {
        temp[0] = c as u8;
        1
    };
    unsafe { add_buff(buf, temp.as_ptr().cast(), templen) };
}

/// Append character `c` to `buf`, escaping special keys, NUL, `K_SPECIAL` and
/// splitting a codepoint into its UTF-8 bytes.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn add_char_buff(buf: *mut buffheader_T, c: c_int) {
    unsafe {
        if c < 0 {
            // A special key is one unit; it has no UTF-8 spelling.
            add_byte_buff(buf, c);
            return;
        }
        let mut bytes = [0u8; MB_MAXBYTES + 1];
        let len = utf_char2bytes(c, bytes.as_mut_ptr().cast()) as usize;
        for &byte in &bytes[..len] {
            add_byte_buff(buf, c_int::from(byte));
        }
    }
}

/// One byte from the read buffers, `readbuf1` first. No translation is done,
/// so `K_SPECIAL` is still escaped.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn read_readbuffers(advance: bool) -> c_int {
    unsafe {
        let c = read_readbuf(readbuf1.ptr(), advance);
        if c == NUL {
            read_readbuf(readbuf2.ptr(), advance)
        } else {
            c
        }
    }
}

/// One byte from `buf`, advancing past it when `advance` is set.
///
/// # Safety
/// `buf` must point at a live buffer.
pub(crate) unsafe fn read_readbuf(buf: *mut buffheader_T, advance: bool) -> c_int {
    unsafe {
        let curr = (*buf).bh_first.b_next;
        if curr.is_null() {
            return NUL; // buffer is empty
        }

        let str = block_str(curr);
        let c = *str.add((*buf).bh_index) as u8;
        if advance {
            (*buf).bh_index += 1;
            if c_int::from(*str.add((*buf).bh_index)) == NUL {
                (*buf).bh_first.b_next = (*curr).b_next;
                xfree(curr.cast());
                (*buf).bh_index = 0;
            }
        }
        c_int::from(c)
    }
}

/// Prepare the read buffers for reading, if they hold anything.
///
/// # Safety
/// Callable at any time.
pub(crate) unsafe fn start_stuff() {
    unsafe {
        for buf in [readbuf1.ptr(), readbuf2.ptr()] {
            if !(*buf).bh_first.b_next.is_null() {
                (*buf).bh_curr = &raw mut (*buf).bh_first;
                // Force a new block to be created; see `add_buff`.
                (*buf).bh_create_newblock = true;
            }
        }
    }
}

/// Whether the stuff buffer is empty.
///
/// # Safety
/// Callable at any time.
pub unsafe fn stuff_empty() -> bool {
    unsafe {
        (*readbuf1.ptr()).bh_first.b_next.is_null() && (*readbuf2.ptr()).bh_first.b_next.is_null()
    }
}

/// Whether `readbuf1` is empty. There may still be redo characters in
/// `readbuf2`.
///
/// # Safety
/// Callable at any time.
pub unsafe fn readbuf1_empty() -> bool {
    unsafe { (*readbuf1.ptr()).bh_first.b_next.is_null() }
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
    unsafe {
        init_typebuf();

        start_stuff();
        while read_readbuffers(true) != NUL {}

        let tb = typebuf.ptr();
        if flush_typeahead == FLUSH_MINIMAL {
            // Remove the mapped characters at the start only, and only when
            // that leaves enough room in typebuf.
            if (*tb).tb_off + (*tb).tb_maplen >= (*tb).tb_buflen {
                (*tb).tb_off = MAXMAPLEN as c_int;
                (*tb).tb_len = 0;
            } else {
                (*tb).tb_off += (*tb).tb_maplen;
                (*tb).tb_len -= (*tb).tb_maplen;
            }
            if (*tb).tb_len == 0 {
                typebuf_was_filled.set(false);
            }
        } else {
            if flush_typeahead == FLUSH_INPUT {
                while inchar((*tb).tb_buf, (*tb).tb_buflen - 1, 10) != 0 {}
            }
            (*tb).tb_off = MAXMAPLEN as c_int;
            (*tb).tb_len = 0;
            // Text received from a client or from feedkeys() is gone with it.
            typebuf_was_filled.set(false);
        }
        (*tb).tb_maplen = 0;
        (*tb).tb_silent = 0;
        cmd_silent.set(false);
        (*tb).tb_no_abbr_cnt = 0;
        (*tb).tb_change_cnt += 1;
        if (*tb).tb_change_cnt == 0 {
            (*tb).tb_change_cnt = 1;
        }
    }
}

/// Flush the map and typeahead buffers and beep about an error.
///
/// # Safety
/// Callable at any time.
pub unsafe fn beep_flush() {
    unsafe {
        if emsg_silent.get() == 0 {
            flush_buffers(FLUSH_MINIMAL);
            vim_beep(kOptBoFlagError as c_uint);
        }
    }
}

/// Stuff a NUL-terminated string into `readbuf1`, to be read back as keys.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuffReadbuff(s: *const c_char) {
    unsafe { add_buff(readbuf1.ptr(), s, -1) };
}

/// Stuff a NUL-terminated string into the redo read buffer.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuffRedoReadbuff(s: *const c_char) {
    unsafe { add_buff(readbuf2.ptr(), s, -1) };
}

/// Stuff `len` bytes into `readbuf1`.
///
/// # Safety
/// `s` must point at `len` readable bytes.
pub unsafe fn stuffReadbuffLen(s: *const c_char, len: ptrdiff_t) {
    unsafe { add_buff(readbuf1.ptr(), s, len) };
}

/// Stuff a string into `readbuf1`, replacing the characters that would end a
/// command line — CR, NL and ESC — with a space.
///
/// Used for `:normal` and for the `@` register: an embedded CR would
/// terminate whatever is reading the stuffed text.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn stuffReadbuffSpec(mut s: *const c_char) {
    unsafe {
        while c_int::from(*s) != NUL {
            if c_int::from(*s as u8) == K_SPECIAL
                && c_int::from(*s.add(1)) != NUL
                && c_int::from(*s.add(2)) != NUL
            {
                // Copy an escaped key code through untouched.
                stuffReadbuffLen(s, 3);
                s = s.add(3);
            } else {
                let c = mb_cptr2char_adv(&raw mut s);
                stuffcharReadbuff(if c == CAR || c == NL || c == ESC {
                    ' ' as c_int
                } else {
                    c
                });
            }
        }
    }
}

/// Stuff one character into `readbuf1`.
///
/// # Safety
/// Callable at any time.
pub unsafe fn stuffcharReadbuff(c: c_int) {
    unsafe { add_char_buff(readbuf1.ptr(), c) };
}

/// Stuff the decimal spelling of `n` into `readbuf1`.
///
/// # Safety
/// Callable at any time.
pub unsafe fn stuffnumReadbuff(n: c_int) {
    unsafe { add_num_buff(readbuf1.ptr(), n) };
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
    unsafe {
        while c_int::from(*arg) != NUL {
            // Stuff a run of ordinary characters in one go.
            let start = arg;
            while (c_int::from(*arg) >= ' ' as c_int && c_int::from(*arg) < DEL)
                || (c_int::from(*arg as u8) == K_SPECIAL && !literally)
            {
                arg = arg.add(1);
            }
            if arg > start {
                stuffReadbuffLen(start, arg.offset_from(start));
            }

            // Then the character that stopped it, one at a time.
            if c_int::from(*arg) != NUL {
                let c = mb_cptr2char_adv(&raw mut arg);
                if literally && ((c < ' ' as c_int && c != TAB) || c == DEL) {
                    stuffcharReadbuff(Ctrl_V);
                }
                stuffcharReadbuff(c);
            }
        }
    }
}
