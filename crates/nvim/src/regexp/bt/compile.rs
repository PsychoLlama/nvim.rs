//! Emitting the program: the node writer and the `regtail`/`reginsert`
//! surgery the parser performs on what it has already written.
//!
//! A node is three bytes — an opcode and a big-endian 16-bit offset to the
//! next node — followed by whatever operand the opcode carries. The offset
//! is relative, so a node is position-independent and [`reginsert`] can slide
//! the tail of the program along to open a gap in front of one.
//!
//! The parser runs twice. The first pass only measures: [`regcode`] holds the
//! [`JUST_CALC_SIZE`] sentinel, every write becomes an addition to
//! [`regsize`], and every node handle the parser gets back is that same
//! sentinel, which makes all the patching below a no-op. The second pass
//! writes into the block sized by the first.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use crate::main::rc_did_emsg;
use crate::mbyte::{utf_char2bytes, utf_char2len, utf_iscomposing_legacy};
use crate::message::emsg;
use crate::os::libc::gettext;
use crate::regexp::{
    BACK, BRACE_COMPLEX, BRANCH, JUST_CALC_SIZE, MAGIC_OFF, MAGIC_ON, NOT_MULTI, NUL, RE_MAGIC,
    RE_STRICT, RE_STRING, had_endbrace, had_eol, initchr, num_complex_braces, peekchr, re_has_z,
    re_multi_type, refresh_cpo_flags, reg_magic, reg_strict, reg_string, reg_toolong, regcode,
    regflags, regnpar, regnzpar, regparse, regsize,
};
use crate::types::{int64_t, uint8_t, uint32_t};

/// The fixed part of a node: the opcode plus the offset to the next one.
const NODE_HDR: usize = 3;

/// Is this the sizing pass rather than the writing one?
fn sizing() -> bool {
    regcode.get() == JUST_CALC_SIZE
}

/// Charge `n` bytes to the size the sizing pass is accumulating.
fn charge(n: usize) {
    regsize.set(regsize.get() + n as int64_t);
}

/// Reset the compiler's per-pattern state. Called once per pass.
pub(crate) fn regcomp_start(expr: *mut uint8_t, re_flags: c_int) {
    // SAFETY: `expr` is the caller's NUL-terminated pattern.
    unsafe { initchr(expr.cast()) };
    reg_magic.set(if re_flags & RE_MAGIC != 0 {
        MAGIC_ON
    } else {
        MAGIC_OFF
    });
    reg_string.set(re_flags & RE_STRING);
    reg_strict.set(re_flags & RE_STRICT);
    refresh_cpo_flags();

    num_complex_braces.set(0);
    regnpar.set(1);
    had_endbrace.set([0; 10]);
    regnzpar.set(1);
    re_has_z.set(0);
    regsize.set(0);
    reg_toolong.set(0);
    regflags.set(0);
    had_eol.set(0);
}

/// Should `c` be emitted as a `MULTIBYTECODE` node rather than as bytes?
///
/// Only when a multi follows it or it can carry combining characters —
/// otherwise the multibyte character is just its bytes, and matching it
/// byte-wise is faster.
pub(crate) fn use_multibytecode(c: c_int) -> bool {
    utf_char2len(c) > 1 && (re_multi_type(peekchr()) != NOT_MULTI || utf_iscomposing_legacy(c))
}

/// Emit one byte of program.
pub(crate) fn regc(b: c_int) {
    if sizing() {
        charge(1);
        return;
    }
    // SAFETY: the writing pass sized the block with the same call sequence,
    // so there is room for this byte.
    unsafe {
        let at = regcode.get();
        regcode.set(at.add(1));
        *at = b as uint8_t;
    }
}

/// Emit one character of program, as its UTF-8 bytes.
pub(crate) fn regmbc(c: c_int) {
    if sizing() {
        charge(utf_char2len(c) as usize);
        return;
    }
    // SAFETY: as `regc`; `utf_char2bytes` writes at most `utf_char2len(c)`
    // bytes, which is what the sizing pass charged.
    unsafe {
        let at = regcode.get();
        regcode.set(at.add(utf_char2bytes(c, at.cast()) as usize));
    }
}

/// Emit a node with opcode `op` and an unset next-offset, and hand back a
/// handle to it — or [`JUST_CALC_SIZE`] during the sizing pass.
pub(crate) fn regnode(op: c_int) -> *mut uint8_t {
    let node = regcode.get();
    if sizing() {
        charge(NODE_HDR);
        return node;
    }
    // SAFETY: as `regc`, three bytes' worth.
    unsafe {
        regcode.set(node.add(NODE_HDR));
        *node = op as uint8_t;
        *node.add(1) = NUL as uint8_t;
        *node.add(2) = NUL as uint8_t;
    }
    node
}

/// Store `val` big-endian at `p` and return the byte after it. `p` must have
/// four writable bytes, which every caller here has just charged for.
fn put_uint32(p: *mut uint8_t, val: uint32_t) -> *mut uint8_t {
    // SAFETY: the four bytes were charged by the sizing pass.
    unsafe {
        p.copy_from_nonoverlapping(val.to_be_bytes().as_ptr(), 4);
        p.add(4)
    }
}

/// Change an already-emitted node's opcode. `[]` uses this to widen an
/// `ANYOF` into its newline-accepting form once it sees a `\n` inside.
pub(crate) fn set_opcode(node: *mut uint8_t, op: c_int) {
    // SAFETY: `node` is a node in the program under construction.
    unsafe { *node = op as uint8_t };
}

/// Emit a node's 32-bit operand, big-endian.
pub(crate) fn regnr(val: uint32_t) {
    if sizing() {
        charge(4);
        return;
    }
    regcode.set(put_uint32(regcode.get(), val));
}

/// The node `p` points at, or null if it is the last one.
pub(crate) fn regnext(p: *mut uint8_t) -> *mut uint8_t {
    if p == JUST_CALC_SIZE || reg_toolong.get() != 0 {
        return core::ptr::null_mut();
    }
    // SAFETY: `p` is a node in the program, so its two offset bytes are
    // readable.
    unsafe {
        let offset = usize::from(u16::from_be_bytes([*p.add(1), *p.add(2)]));
        if offset == 0 {
            core::ptr::null_mut()
        } else if *p as c_int == BACK {
            p.sub(offset)
        } else {
            p.add(offset)
        }
    }
}

/// Point the last node of the chain starting at `p` at `val`.
///
/// A `BACK` node's offset counts backwards, which is how the compiler builds
/// the loop in a non-simple `*`.
pub(crate) fn regtail(p: *mut uint8_t, val: *const uint8_t) {
    if p == JUST_CALC_SIZE {
        return;
    }
    let mut scan = p;
    loop {
        let next = regnext(scan);
        if next.is_null() {
            break;
        }
        scan = next;
    }
    // SAFETY: `scan` is a node in the program and `val` another node in the
    // same allocation, so the difference is well defined.
    unsafe {
        let offset = if *scan as c_int == BACK {
            scan.offset_from(val)
        } else {
            val.offset_from(scan)
        };
        // A 16-bit offset cannot reach: the pattern is too long. The caller
        // notices via `reg_toolong` and gives up on the whole program.
        if offset > 0xffff {
            reg_toolong.set(1);
        } else {
            let bytes = (offset as u16).to_be_bytes();
            *scan.add(1) = bytes[0];
            *scan.add(2) = bytes[1];
        }
    }
}

/// [`regtail`] on the *operand* of `p`, for the node kinds whose operand is
/// itself a chain: a `BRANCH` and the ten `BRACE_COMPLEX` slots.
pub(crate) fn regoptail(p: *mut uint8_t, val: *mut uint8_t) {
    if p.is_null() || p == JUST_CALC_SIZE {
        return;
    }
    // SAFETY: `p` is a node in the program, so its opcode is readable and a
    // node of either kind is followed by its operand.
    unsafe {
        let op = *p as c_int;
        if op != BRANCH && !(BRACE_COMPLEX..=BRACE_COMPLEX + 9).contains(&op) {
            return;
        }
        regtail(p.add(NODE_HDR), val);
    }
}

/// Open `len` bytes in front of `opnd`, sliding everything written since it
/// along, and lay a node header for `op` in the gap.
///
/// Returns the first byte after the header, which is where the caller writes
/// the new node's operand.
fn open_before(op: c_int, opnd: *mut uint8_t, len: usize) -> *mut uint8_t {
    // SAFETY: `opnd` is a node in the program and everything from it to
    // `regcode` was written by this pass; the sizing pass charged `len` extra
    // bytes for this call, so the destination is in the same allocation.
    unsafe {
        let mut src = regcode.get();
        regcode.set(src.add(len));
        let mut dst = regcode.get();
        while src > opnd {
            src = src.sub(1);
            dst = dst.sub(1);
            *dst = *src;
        }
        *opnd = op as uint8_t;
        *opnd.add(1) = NUL as uint8_t;
        *opnd.add(2) = NUL as uint8_t;
        opnd.add(NODE_HDR)
    }
}

/// Insert an operand-less node in front of `opnd`.
pub(crate) fn reginsert(op: c_int, opnd: *mut uint8_t) {
    if sizing() {
        charge(NODE_HDR);
        return;
    }
    open_before(op, opnd, NODE_HDR);
}

/// Insert a node carrying one 32-bit number in front of `opnd`.
pub(crate) fn reginsert_nr(op: c_int, val: int64_t, opnd: *mut uint8_t) {
    if sizing() {
        charge(NODE_HDR + 4);
        return;
    }
    let place = open_before(op, opnd, NODE_HDR + 4);
    debug_assert!((0..=uint32_t::MAX as int64_t).contains(&val));
    put_uint32(place, val as uint32_t);
}

/// Insert a `BRACE_LIMITS`-shaped node — two 32-bit numbers — in front of
/// `opnd`, and point it at the end of itself so the matcher can find the
/// braced atom.
pub(crate) fn reginsert_limits(op: c_int, minval: int64_t, maxval: int64_t, opnd: *mut uint8_t) {
    if sizing() {
        charge(NODE_HDR + 8);
        return;
    }
    let mut place = open_before(op, opnd, NODE_HDR + 8);
    debug_assert!((0..=uint32_t::MAX as int64_t).contains(&minval));
    debug_assert!((0..=uint32_t::MAX as int64_t).contains(&maxval));
    place = put_uint32(place, minval as uint32_t);
    place = put_uint32(place, maxval as uint32_t);
    regtail(opnd, place);
}

/// Is a `\1`..`\9` back-reference to group `refnum` legal here?
///
/// Normally the group must already have closed. The exception is a
/// look-behind: `\(...\)\@<=` runs the group after the reference in the
/// program, so a reference forward into one is fine as long as some `\@<=`
/// or `\@<!` is still to come in the pattern.
pub(crate) fn seen_endbrace(refnum: c_int) -> bool {
    if had_endbrace.get()[refnum as usize] != 0 {
        return true;
    }
    // SAFETY: `regparse` points into the NUL-terminated pattern, so the walk
    // stops at its end; the message is a static NUL-terminated string.
    unsafe {
        let mut p = regparse.get().cast::<uint8_t>();
        while *p as c_int != NUL {
            if *p as c_int == '@' as c_int
                && *p.add(1) as c_int == '<' as c_int
                && (*p.add(2) as c_int == '!' as c_int || *p.add(2) as c_int == '=' as c_int)
            {
                return true;
            }
            p = p.add(1);
        }
        emsg(gettext(c"E65: Illegal back reference".as_ptr()));
    }
    rc_did_emsg.set(true);
    false
}
