//! Emitting the program: the node writer and the `tail`/`insert`
//! surgery the parser performs on what it has already written.
//!
//! A node is three bytes — an opcode and a big-endian 16-bit offset to the
//! next node — followed by whatever operand the opcode carries. The offset
//! is relative, so a node is position-independent and
//! [`BtEmitter::insert`] can slide the tail of the program along to open a
//! gap in front of one.
//!
//! Upstream parsed every pattern twice: once only to measure, with the
//! output cursor holding a `-1` sentinel and every patch a no-op, so that
//! it could allocate the program, and once to write into that allocation.
//! The writer here appends to a `Vec`, which needs no measuring, so the
//! parser runs once and [`super::piece::bt_regcomp`] copies the text into
//! the program block. The bytes are the writing pass's. A node handle is a
//! [`Node`], the node's offset into the text.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::regexp::RegCompiler;
use core::ffi::{c_int, c_uint};
use core::mem::offset_of;

use super::op::BtOp;
use crate::mbyte::{encode_char, utf_char2len, utf_iscomposing_legacy};
use crate::memory::xmalloc;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{BtRegProg, NOT_MULTI, REX_SET, Rex, peekchr, re_multi_type};
use crate::types::{NUL, RegEngine, RegProg, int64_t, uint8_t, uint32_t};

/// The fixed part of a node: the opcode plus the offset to the next one.
const NODE_HDR: usize = 3;

/// A compiled backtracking program.
///
/// One `xmalloc` block: a fixed head, then the `REGMAGIC` stamp, then the
/// nodes. The head opens with the same five fields `RegProg` and the NFA's
/// `NfaRegProg` open with — C's single inheritance, and what lets
/// `vim_regexec` hand any program to any engine and lets each engine cast the
/// pointer back to its own shape. **That prefix is the layout and stays
/// exactly as it is**: this is a handle round the pointer, not a new
/// representation of what is behind it.
///
/// What it buys is the same thing [`Rex`](crate::regexp::Rex) buys for the
/// match context. Each field is named once, here, with the promise that comes
/// with reading it stated once; the compiler and the matcher stop spelling
/// out `(*prog).` inside `unsafe` regions of their own, and several of their
/// functions stop being one `unsafe` block from brace to brace.
#[derive(Clone, Copy)]
pub(crate) struct BtProg(*mut BtRegProg);

impl BtProg {
    /// The program the running match is for, or `None` if the caller handed
    /// in a match structure with no program.
    #[inline(always)]
    pub(crate) fn of_match(rex: Rex) -> Option<BtProg> {
        let prog = rex.regprog();
        (!prog.is_null()).then(|| BtProg(prog.cast()))
    }

    /// A fresh program with room for `nodes` bytes of program text.
    ///
    /// Only the fields [`bt_regcomp`](super::piece::bt_regcomp) does not go
    /// on to write are initialised here.
    fn alloc(nodes: usize) -> BtProg {
        // SAFETY: `xmalloc` returns a block of the size asked for or does not
        // return, so the head is inside it.
        let prog = unsafe { xmalloc(offset_of!(BtRegProg, program) + nodes) }.cast::<BtRegProg>();
        unsafe { (*prog).re_in_use = false };
        BtProg(prog)
    }

    /// A fresh program holding `text`, which starts with the `REGMAGIC`
    /// stamp. Only the fields `bt_regcomp` does not go on to write are
    /// initialised.
    pub(crate) fn with_text(text: &[u8]) -> BtProg {
        let prog = BtProg::alloc(text.len());
        // SAFETY: `alloc` made room for `text.len()` bytes of program.
        unsafe {
            prog.text()
                .copy_from_nonoverlapping(text.as_ptr(), text.len())
        };
        prog
    }

    /// Hand the block back to the caller of `bt_regcomp`, as the engine's
    /// shared shape.
    pub(crate) fn into_regprog(self) -> *mut RegProg {
        self.0.cast()
    }

    // ------------------------------------------------- what the compiler wrote

    /// The pattern's own `\c`/`\C`/`\Z` and the `RF_*` findings.
    #[inline(always)]
    pub(crate) fn regflags(self) -> c_uint {
        // SAFETY: the handle is a live program of this engine — see
        // `of_match`. Every accessor below reads or writes one field of that
        // block and this note covers all of them.
        unsafe { (*self.0).regflags }
    }

    #[inline(always)]
    pub(crate) fn add_regflags(self, bits: c_uint) {
        unsafe { (*self.0).regflags |= bits };
    }

    #[inline(always)]
    pub(crate) fn set_regflags(self, flags: c_uint) {
        unsafe { (*self.0).regflags = flags };
    }

    /// Can the pattern only match at the start of the line?
    #[inline(always)]
    pub(crate) fn is_anchored(self) -> bool {
        unsafe { (*self.0).reganch != 0 }
    }

    #[inline(always)]
    pub(crate) fn set_anchored(self, anchored: bool) {
        unsafe { (*self.0).reganch = uint8_t::from(anchored) };
    }

    /// The character the pattern must start with, or `NUL` if unknown.
    #[inline(always)]
    pub(crate) fn regstart(self) -> c_int {
        unsafe { (*self.0).regstart }
    }

    #[inline(always)]
    pub(crate) fn set_regstart(self, c: c_int) {
        unsafe { (*self.0).regstart = c };
    }

    /// A literal run the line must hold somewhere for the pattern to match,
    /// or null.
    #[inline(always)]
    pub(crate) fn regmust(self) -> *mut uint8_t {
        unsafe { (*self.0).regmust }
    }

    /// How long that run is. The matcher *writes* it back: `cstrncmp` reports
    /// the byte length it actually compared, which differs from the byte
    /// length of the pattern under 'ignorecase' folding.
    #[inline(always)]
    pub(crate) fn regmlen(self) -> c_int {
        unsafe { (*self.0).regmlen }
    }

    #[inline(always)]
    pub(crate) fn set_regmlen(self, len: c_int) {
        unsafe { (*self.0).regmlen = len };
    }

    #[inline(always)]
    pub(crate) fn set_regmust(self, run: *mut uint8_t, len: c_int) {
        unsafe { (*self.0).regmust = run };
        self.set_regmlen(len);
    }

    /// Does the pattern have `\z(` groups?
    #[inline(always)]
    pub(crate) fn has_z(self) -> bool {
        unsafe { (*self.0).reghasz as c_int == REX_SET }
    }

    #[inline(always)]
    pub(crate) fn set_reghasz(self, hasz: uint8_t) {
        unsafe { (*self.0).reghasz = hasz };
    }

    #[inline(always)]
    pub(crate) fn set_engine(self, engine: *mut RegEngine) {
        unsafe { (*self.0).engine = engine };
    }

    // ---------------------------------------------------------- the program

    /// Where the program text begins, `REGMAGIC` stamp and all. What the
    /// emitter's output cursor is aimed at.
    #[inline(always)]
    pub(crate) fn text(self) -> *mut uint8_t {
        // SAFETY: `program` is the flexible array the block ends with; taking
        // its address reads nothing.
        unsafe { (&raw mut (*self.0).program).cast::<uint8_t>() }
    }

    /// The stamp the block opens with, which is how a program compiled by the
    /// other engine is spotted.
    #[inline(always)]
    pub(crate) fn magic(self) -> c_int {
        // SAFETY: the block always holds at least the stamp.
        unsafe { *self.text() as c_int }
    }

    /// The first node, one byte past the stamp.
    #[inline(always)]
    pub(crate) fn first_node(self) -> *mut uint8_t {
        // SAFETY: as `magic`; a program always has an `END` node after it.
        unsafe { self.text().add(1) }
    }
}

/// A node of the program being written: its offset into the program text,
/// whose first byte is the `REGMAGIC` stamp.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Node(usize);

impl Node {
    /// Where the node's operand starts, after its three-byte header. The
    /// operand of a `BRANCH` or a complex brace is itself a chain.
    pub(crate) fn operand(self) -> Node {
        Node(self.0 + NODE_HDR)
    }
}

/// The backtracker's program writer.
pub(crate) struct BtEmitter {
    /// The program text so far, `REGMAGIC` stamp first.
    text: Vec<u8>,
    /// A node offset did not fit in 16 bits: the pattern is too long.
    pub(crate) too_long: bool,
}

impl BtEmitter {
    pub(crate) const fn new() -> BtEmitter {
        BtEmitter {
            text: Vec::new(),
            too_long: false,
        }
    }

    /// Make room for `bytes` of program up front, so that the writes do not
    /// reallocate as the text grows.
    pub(crate) fn reserve(&mut self, bytes: usize) {
        self.text.reserve(bytes);
    }

    /// The program written so far, leaving the emitter empty.
    pub(crate) fn take_text(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.text)
    }

    /// Emit one byte of program. `b` is a byte, or a character below 256.
    #[inline]
    pub(crate) fn byte(&mut self, b: c_int) {
        self.text.push(b as uint8_t);
    }

    /// Emit one character of program, as its UTF-8 bytes.
    #[inline]
    pub(crate) fn char(&mut self, c: c_int) {
        if (0..0x80).contains(&c) {
            return self.byte(c);
        }
        let mut bytes = [0; 6];
        let len = encode_char(c, &mut bytes);
        self.text.extend_from_slice(&bytes[..len]);
    }

    /// Emit a node with opcode `op` and an unset next-offset.
    #[inline]
    pub(crate) fn node(&mut self, op: BtOp) -> Node {
        self.node_nl(op, false)
    }

    /// [`BtEmitter::node`], for the class opcodes that have a `\_x` form:
    /// with `crosses_lines` the node also matches a line break.
    #[inline]
    pub(crate) fn node_nl(&mut self, op: BtOp, crosses_lines: bool) -> Node {
        let node = Node(self.text.len());
        self.text
            .extend_from_slice(&[op.encode(crosses_lines), NUL as uint8_t, NUL as uint8_t]);
        node
    }

    /// Emit a node's 32-bit operand, big-endian.
    #[inline]
    pub(crate) fn number(&mut self, val: uint32_t) {
        self.text.extend_from_slice(&val.to_be_bytes());
    }

    /// Change an already-emitted node's opcode. `[]` uses this to widen an
    /// `ANYOF` into its newline-accepting form once it sees a `\n` inside.
    pub(crate) fn set_opcode(&mut self, node: Node, op: BtOp, crosses_lines: bool) {
        self.text[node.0] = op.encode(crosses_lines);
    }

    /// The opcode byte of `node`.
    pub(crate) fn opcode_at(&self, node: Node) -> uint8_t {
        self.text[node.0]
    }

    /// The node after `p` in its chain. `None` at the end of the chain, and
    /// once an offset has overflowed, since the chain can no longer be
    /// trusted.
    #[inline]
    pub(crate) fn next(&self, p: Node) -> Option<Node> {
        if self.too_long {
            return None;
        }
        let text = &self.text;
        let offset = usize::from(u16::from_be_bytes([text[p.0 + 1], text[p.0 + 2]]));
        if offset == 0 {
            None
        } else if text[p.0] == BtOp::Back.code() as uint8_t {
            Some(Node(p.0 - offset))
        } else {
            Some(Node(p.0 + offset))
        }
    }

    /// Point the last node of the chain starting at `p` at `val`.
    ///
    /// A `BACK` node's offset counts backwards, which is how the compiler
    /// builds the loop in a non-simple `*`.
    #[inline]
    pub(crate) fn tail(&mut self, p: Node, val: Node) {
        let too_long = self.too_long;
        let text = &mut self.text;
        let back = BtOp::Back.code() as uint8_t;
        // Walk to the end of the chain, as `next` does; an overflowed chain
        // is not walked, and the patch lands on `p` itself.
        let mut scan = p.0;
        if !too_long {
            loop {
                let step = usize::from(u16::from_be_bytes([text[scan + 1], text[scan + 2]]));
                if step == 0 {
                    break;
                }
                scan = if text[scan] == back {
                    scan - step
                } else {
                    scan + step
                };
            }
        }
        let (at, val) = (scan as isize, val.0 as isize);
        let offset = if text[scan] == back {
            at - val
        } else {
            val - at
        };
        // A 16-bit offset cannot reach: the pattern is too long. The caller
        // notices and gives up on the whole program.
        if offset > 0xffff {
            self.too_long = true;
        } else {
            let bytes = (offset as u16).to_be_bytes();
            text[scan + 1] = bytes[0];
            text[scan + 2] = bytes[1];
        }
    }

    /// [`BtEmitter::tail`] on the *operand* of `p`, for the node kinds whose
    /// operand is itself a chain: a `BRANCH` and the ten `BRACE_COMPLEX`
    /// slots.
    pub(crate) fn op_tail(&mut self, p: Node, val: Node) {
        let Ok((op, _)) = BtOp::decode(self.opcode_at(p)) else {
            return;
        };
        if op == BtOp::Branch || op.is_complex_brace() {
            self.tail(p.operand(), val);
        }
    }

    /// Open a node for `op` in front of `opnd`, sliding everything written
    /// since along, with `operand` after its header.
    #[inline]
    fn insert_with(&mut self, op: BtOp, opnd: Node, operand: &[u8]) {
        let width = NODE_HDR + operand.len();
        let text = &mut self.text;
        let end = text.len();
        text.resize(end + width, 0);
        text.copy_within(opnd.0..end, opnd.0 + width);
        text[opnd.0] = op.code() as uint8_t;
        text[opnd.0 + 1] = NUL as uint8_t;
        text[opnd.0 + 2] = NUL as uint8_t;
        text[opnd.0 + NODE_HDR..opnd.0 + width].copy_from_slice(operand);
    }

    /// Insert an operand-less node in front of `opnd`.
    pub(crate) fn insert(&mut self, op: BtOp, opnd: Node) {
        self.insert_with(op, opnd, &[]);
    }

    /// Insert a node carrying one 32-bit number in front of `opnd`.
    pub(crate) fn insert_nr(&mut self, op: BtOp, val: int64_t, opnd: Node) {
        debug_assert!((0..=uint32_t::MAX as int64_t).contains(&val));
        self.insert_with(op, opnd, &(val as uint32_t).to_be_bytes());
    }

    /// Insert a `BRACE_LIMITS`-shaped node — two 32-bit numbers — in front
    /// of `opnd`, and point it at the end of itself so the matcher can find
    /// the braced atom.
    pub(crate) fn insert_limits(&mut self, op: BtOp, minval: int64_t, maxval: int64_t, opnd: Node) {
        debug_assert!((0..=uint32_t::MAX as int64_t).contains(&minval));
        debug_assert!((0..=uint32_t::MAX as int64_t).contains(&maxval));
        let mut operand = [0; 8];
        operand[..4].copy_from_slice(&(minval as uint32_t).to_be_bytes());
        operand[4..].copy_from_slice(&(maxval as uint32_t).to_be_bytes());
        self.insert_with(op, opnd, &operand);
        self.tail(opnd, Node(opnd.0 + NODE_HDR + operand.len()));
    }
}

/// Should `c` be emitted as a `MULTIBYTECODE` node rather than as bytes?
///
/// Only when a multi follows it or it can carry combining characters —
/// otherwise the multibyte character is just its bytes, and matching it
/// byte-wise is faster.
pub(crate) fn use_multibytecode(rc: &mut RegCompiler, c: c_int) -> bool {
    utf_char2len(c) > 1 && (re_multi_type(peekchr(rc)) != NOT_MULTI || utf_iscomposing_legacy(c))
}

/// The node after `p` in a finished program's chain, or null if `p` is the
/// last one.
///
/// `p` must be a node of a finished program.
pub(crate) fn regnext(p: *mut uint8_t) -> *mut uint8_t {
    // SAFETY: `p` is a node in the program, so its two offset bytes are
    // readable.
    let offset = usize::from(u16::from_be_bytes([unsafe { *p.add(1) }, unsafe {
        *p.add(2)
    }]));
    if offset == 0 {
        core::ptr::null_mut()
    } else if unsafe { *p } == BtOp::Back.code() as uint8_t {
        unsafe { p.sub(offset) }
    } else {
        unsafe { p.add(offset) }
    }
}

/// Is a `\1`..`\9` back-reference to group `refnum` legal here?
///
/// Normally the group must already have closed. The exception is a
/// look-behind: `\(...\)\@<=` runs the group after the reference in the
/// program, so a reference forward into one is fine as long as some `\@<=`
/// or `\@<!` is still to come in the pattern.
pub(crate) fn seen_endbrace(rc: &RegCompiler, refnum: c_int) -> bool {
    if rc.closed_groups[refnum as usize] != 0 {
        return true;
    }
    // SAFETY: the cursor points into the NUL-terminated pattern, so the walk
    // stops at its end; the message is a static NUL-terminated string.
    let mut p = rc.cursor.cast::<uint8_t>();
    while unsafe { *p } as c_int != NUL {
        if unsafe { *p } as c_int == '@' as c_int
            && unsafe { *p.add(1) } as c_int == '<' as c_int
            && (unsafe { *p.add(2) } as c_int == '!' as c_int
                || unsafe { *p.add(2) } as c_int == '=' as c_int)
        {
            return true;
        }
        p = unsafe { p.add(1) };
    }
    emsg(gettext(c"E65: Illegal back reference"));
    rc_did_emsg.set(true);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(emit: impl Fn(&mut BtEmitter)) -> Vec<u8> {
        let mut code = BtEmitter::new();
        emit(&mut code);
        code.take_text()
    }

    #[test]
    fn a_tail_points_the_last_node_of_the_chain_at_its_target() {
        let text = write(|code| {
            code.byte(0o234);
            let first = code.node(BtOp::Branch);
            let second = code.node(BtOp::Nothing);
            let end = code.node(BtOp::End);
            code.tail(first, second);
            code.tail(first, end);
        });
        // magic, then BRANCH -> +3, NOTHING -> +3, END with no next.
        let (branch, nothing, end) = (
            BtOp::Branch.code() as u8,
            BtOp::Nothing.code() as u8,
            BtOp::End.code() as u8,
        );
        assert_eq!(text, [0o234, branch, 0, 3, nothing, 0, 3, end, 0, 0]);
    }

    #[test]
    fn a_back_node_counts_its_offset_backwards() {
        let mut code = BtEmitter::new();
        code.byte(0o234);
        let target = code.node(BtOp::Nothing);
        let back = code.node(BtOp::Back);
        code.tail(back, target);
        assert_eq!(code.next(back), Some(target));
        assert_eq!(&code.take_text()[4..], [BtOp::Back.code() as u8, 0, 3]);
    }

    #[test]
    fn an_insert_slides_the_operand_along_and_heads_it() {
        let text = write(|code| {
            code.byte(0o234);
            let atom = code.node(BtOp::Any);
            code.insert_nr(BtOp::Behind, 0x0102_0304, atom);
        });
        let behind = BtOp::Behind.code() as u8;
        let any = BtOp::Any.code() as u8;
        assert_eq!(text, [0o234, behind, 0, 0, 1, 2, 3, 4, any, 0, 0]);
    }

    #[test]
    fn limits_point_past_themselves_at_the_braced_atom() {
        let mut code = BtEmitter::new();
        code.byte(0o234);
        let atom = code.node(BtOp::Any);
        code.insert_limits(BtOp::BraceLimits, 1, 3, atom);
        // The limits node took the atom's place; the atom is 11 bytes on.
        assert_eq!(code.next(atom), Some(Node(atom.0 + 11)));
    }

    #[test]
    fn a_character_is_written_as_its_utf8_bytes() {
        let text = write(|code| {
            code.char(0x61);
            code.char(0x20ac);
        });
        assert_eq!(text, b"a\xe2\x82\xac");
    }

    #[test]
    fn an_offset_past_sixteen_bits_marks_the_program_too_long() {
        let mut code = BtEmitter::new();
        let first = code.node(BtOp::Branch);
        for _ in 0..0x10000 {
            code.byte(0);
        }
        let far = code.node(BtOp::End);
        code.tail(first, far);
        assert!(code.too_long);
        assert_eq!(code.next(first), None, "an overflowed chain is not walked");
    }
}
