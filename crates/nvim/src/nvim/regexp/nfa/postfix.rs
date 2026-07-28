//! The postfix form the parser emits and [`super::build`] consumes.
//!
//! Upstream keeps this as three raw cursors (`post_start`, `post_ptr`,
//! `post_end`) over one `xmalloc`ed `int` array, with the "append, growing
//! if full" dance open-coded at every one of its ~100 uses. It is a stack of
//! `int`s and nothing more: the items are opcodes (the negative `NFA_*`
//! constants), literal code points, and the odd inline operand. This module
//! is that stack behind a checked API — [`emit`], [`len`], [`truncate`],
//! [`drop_last`], [`with_items`] — so that the parsers above it hold no
//! pointers at all.
//!
//! The parser rewinds the stack as well as appending to it: `\{n,m}`
//! re-parses its atom and throws away what the speculative pass emitted,
//! which is [`len`] plus [`truncate`].
//!
//! # Why this is not simply a `Vec`
//!
//! It was, and it cost a factor of three. Appending happens once per emitted
//! item, and the compile-speed test's
//! `\v(((((Nxxxxxxx&&xxxx){179})+)+)+){179}` emits eight million of them:
//! the `\{n,m}` expansion re-parses its atom once per repetition and the
//! repetitions nest. At opt-level 0 — which is what the test suites run —
//! none of `Vec`'s accessors inline, so `push` is a chain of half a dozen
//! calls where upstream's macro was six instructions. `Vec` still owns the
//! allocation; the length and the write pointer are kept beside it as plain
//! fields so the hot path touches neither.
//!
//! For the same reason the accessors read the cell through `ptr` rather than
//! `with_mut`, whose debug borrow table is an outlined call and a hash
//! insert. That is the rule the match context follows too — see
//! [`super::super::context`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::regexp::NFA_CONCAT;

struct Program {
    /// Owns the allocation. Its own length stays 0; `len` below is the real
    /// one, so that appending never calls into `Vec`.
    buf: Vec<c_int>,
    /// `buf.as_mut_ptr()`, refreshed whenever `buf` reallocates.
    items: *mut c_int,
    /// `buf.capacity()`, likewise.
    cap: usize,
    len: usize,
}

impl Program {
    /// Make room for at least `want` items, keeping the cached pointer and
    /// capacity in step with `buf`.
    fn reserve(&mut self, want: usize) {
        if want <= self.cap {
            return;
        }
        self.buf.reserve_exact(want);
        self.items = self.buf.as_mut_ptr();
        self.cap = self.buf.capacity();
    }
}

static POSTFIX: GlobalCell<Program> = GlobalCell::new(Program {
    buf: Vec::new(),
    items: core::ptr::null_mut(),
    cap: 0,
    len: 0,
});

/// Start a fresh program for a pattern `pattern_len` bytes long.
///
/// The reservation is upstream's guess at how many items a pattern of that
/// length can produce; it is only a capacity, so an underestimate costs a
/// reallocation rather than a failure.
pub(crate) fn start(pattern_len: usize) {
    POSTFIX.with_mut(|program| {
        program.len = 0;
        program.reserve((pattern_len + 1) * 25 + 1000);
    });
}

/// Release the program. Called once a compile has read it back.
pub(crate) fn finish() {
    POSTFIX.with_mut(|program| {
        program.buf = Vec::new();
        program.items = core::ptr::null_mut();
        program.cap = 0;
        program.len = 0;
    });
}

/// Grow the program. Out of line: appending is the hot path and this runs a
/// handful of times per compile.
#[inline(never)]
#[cold]
fn grow() {
    POSTFIX.with_mut(|program| {
        // Upstream's `realloc_post_list` grows by half again.
        let want = program.cap + program.cap / 2 + 1;
        program.reserve(want);
    });
}

/// Append one item. The transpiled form of upstream's `EMIT` macro.
#[inline(always)]
pub(crate) fn emit(item: c_int) {
    // SAFETY: `len` is at most `cap`, and `items` addresses `cap` items of
    // the allocation `buf` owns. No reference into the program is live —
    // the only one handed out is `with_items`', and the read phase does not
    // emit.
    unsafe {
        let program = POSTFIX.ptr();
        if (*program).len == (*program).cap {
            grow();
        }
        *(*program).items.add((*program).len) = item;
        (*program).len += 1;
    }
}

/// Append `item` followed by the `NFA_CONCAT` that joins it to what came
/// before — the shape most of the collection parser emits in.
#[inline(always)]
pub(crate) fn emit_concat(item: c_int) {
    emit(item);
    emit(NFA_CONCAT);
}

/// How many items have been emitted; a handle [`truncate`] can rewind to.
#[inline(always)]
pub(crate) fn len() -> usize {
    // SAFETY: as `emit`.
    unsafe { (*POSTFIX.ptr()).len }
}

/// Rewind to a [`len`] handle, dropping everything emitted since.
#[inline(always)]
pub(crate) fn truncate(mark: usize) {
    // SAFETY: as `emit`; `mark` came from `len`, so it is at most `len`.
    unsafe { (*POSTFIX.ptr()).len = mark };
}

/// Drop the last item. `[a-z]` uses this to reclaim the `NFA_CONCAT` that
/// followed the range's start character, which it emitted before it knew a
/// `-` came next.
#[inline(always)]
pub(crate) fn drop_last() {
    // SAFETY: as `emit`.
    unsafe {
        let program = POSTFIX.ptr();
        (*program).len = (*program).len.saturating_sub(1);
    }
}

/// Read the program back.
///
/// The callback must not emit: this is the compile's read phase, and
/// appending could reallocate under the slice — exactly as it would have
/// invalidated upstream's cursor.
///
/// `ptr`, not `with`, and for a sharper reason than the accessors above: the
/// callback here is the whole of `post2nfa`, and a live `GlobalCell` borrow
/// makes *every* `get`/`set` in the program — every other cell, not just
/// this one — take the borrow table's slow path for as long as it is held.
/// Wrapping `post2nfa` in `with` measured nine times slower than upstream.
pub(crate) fn with_items<R>(f: impl FnOnce(&[c_int]) -> R) -> R {
    // SAFETY: `items` addresses `len` initialised items, and the read phase
    // does not emit, so nothing reallocates under the slice.
    unsafe {
        let program = POSTFIX.ptr();
        // An empty program has no allocation yet, and a slice needs a
        // non-null base even for a length of zero.
        let base = if (*program).items.is_null() {
            core::ptr::NonNull::dangling().as_ptr()
        } else {
            (*program).items
        };
        f(core::slice::from_raw_parts(base, (*program).len))
    }
}
