//! The postfix form the parser emits and [`super::build`] consumes.
//!
//! Upstream keeps this as three raw cursors (`post_start`, `post_ptr`,
//! `post_end`) over one `xmalloc`ed `int` array, with the "append, growing
//! if full" dance open-coded at every one of its ~90 uses. It is a stack of
//! `int`s and nothing more: the items are opcodes (the negative `NFA_*`
//! constants), literal code points, and the odd inline operand. A `Vec`
//! says the same thing and takes the pointer arithmetic with it.
//!
//! The parser also rewinds the stack — `\{n,m}` re-parses its atom and
//! throws away what the speculative pass emitted — which is [`len`] plus
//! [`truncate`] here.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::src::nvim::global_cell::GlobalCell;

static POSTFIX: GlobalCell<Vec<c_int>> = GlobalCell::new(Vec::new());

/// Start a fresh program for a pattern `pattern_len` bytes long.
///
/// The reservation is upstream's guess at how many items a pattern of that
/// length can produce; it is only a capacity, so an underestimate costs a
/// reallocation rather than a failure.
pub(crate) fn start(pattern_len: usize) {
    POSTFIX.with_mut(|items| {
        items.clear();
        items.reserve((pattern_len + 1) * 25 + 1000);
    });
}

/// Release the program. Called once a compile has read it back.
pub(crate) fn finish() {
    POSTFIX.with_mut(|items| *items = Vec::new());
}

/// Append one item. The transpiled form of upstream's `EMIT` macro.
pub(crate) fn emit(item: c_int) {
    POSTFIX.with_mut(|items| items.push(item));
}

/// Append `item` followed by the `NFA_CONCAT` that joins it to what came
/// before — the shape most of the collection parser emits in.
pub(crate) fn emit_concat(item: c_int) {
    emit(item);
    emit(super::NFA_CONCAT as c_int);
}

/// How many items have been emitted; a handle [`truncate`] can rewind to.
pub(crate) fn len() -> usize {
    POSTFIX.with(|items| items.len())
}

/// Rewind to a [`len`] handle, dropping everything emitted since.
pub(crate) fn truncate(mark: usize) {
    POSTFIX.with_mut(|items| items.truncate(mark));
}

/// Drop the last item. `[a-z]` uses this to reclaim the range's start
/// character, which it emitted before it knew a `-` followed.
pub(crate) fn drop_last() {
    POSTFIX.with_mut(|items| {
        items.pop();
    });
}

/// Read the program back.
///
/// The callback must not emit: this is the compile's read phase, and the
/// borrow tracking in [`GlobalCell`] turns a violation into a panic rather
/// than the invalidated cursor upstream would have had.
pub(crate) fn with_items<R>(f: impl FnOnce(&[c_int]) -> R) -> R {
    POSTFIX.with(|items| f(items))
}
