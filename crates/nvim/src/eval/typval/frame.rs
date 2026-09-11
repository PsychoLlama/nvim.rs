//! [`CallFrame`]: the argument list a call is spliced into.
//!
//! Upstream builds a C array per call site and reasons in prose about which
//! of its slots have to be freed; this is that reasoning written down.  The
//! one step that is not safe code -- duplicating a value the caller keeps --
//! lives beside `TypVal::bit_copy` in `access.rs`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{TV_INITIAL_VALUE, TypVal};
use core::mem::ManuallyDrop;

/// One call's argument list, assembled where the caller's own array had no
/// room: in front of it, or around a `base->Method()` base.
///
/// A slot is one of two things. Either the frame *owns* the value -- a
/// partial's bound argument, copied because the call may free the partial --
/// and releases it when the frame goes out of scope, or the slot is a bit
/// copy of a value the caller owns for the length of the call and the frame
/// releases nothing. That is the whole reason the slots are `ManuallyDrop`
/// and the ownership rides beside them in a bitmask rather than in the type:
/// splicing a base in front of the caller's arguments must not duplicate a
/// string or take a reference, and a `Vec<TypVal>` built by `clone()` would
/// do both.
/// The slots are a [`ManuallyDrop`] for the reason ruling (p30-a) gives:
/// a frame releases exactly the `len` slots it holds, and a droppable field
/// would have the compiler walk all `N` of them a second time afterwards --
/// twenty-one tag tests on every call the interpreter makes, whatever the
/// arity.
pub(crate) struct CallFrame<const N: usize> {
    slots: ManuallyDrop<[TypVal; N]>,
    len: usize,
    /// Bit `i` set: slot `i` is this frame's to release.
    owned: u32,
}

impl<const N: usize> CallFrame<N> {
    /// An empty frame.
    pub(crate) const fn new() -> Self {
        const { assert!(N <= 32, "the ownership mask is a u32") };
        Self {
            slots: ManuallyDrop::new([TV_INITIAL_VALUE; N]),
            len: 0,
            owned: 0,
        }
    }

    /// A frame of values the caller keeps: every slot names a payload
    /// something else owns and releases, and the frame releases none of
    /// them.
    pub(crate) fn naming(tvs: [TypVal; N]) -> Self {
        let mut frame = Self::new();
        for tv in tvs {
            frame.push_naming(tv);
        }
        frame
    }

    /// How many arguments the frame holds.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Whether another slot would overflow the frame.
    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len == N
    }

    /// Append a value that *names* what it does not own -- a caller's
    /// string, a container somebody else gives the reference back for, a
    /// static message.  The frame releases nothing for this slot.
    pub(crate) fn push_naming(&mut self, tv: TypVal) {
        self.slots[self.len] = tv;
        self.len += 1;
    }

    /// Append a value the frame takes over and releases.
    pub(crate) fn push_owned(&mut self, tv: TypVal) {
        self.owned |= 1 << self.len;
        self.push_naming(tv);
    }

    /// Move the last slot to the front, taking the ownership bits with it.
    pub(super) fn rotate_last_to_front(&mut self) {
        self.slots[..self.len].rotate_right(1);
        // The slot that moved to the front carries a clear bit, so shifting
        // the whole mask up one is what the rotation did to the values.
        self.owned <<= 1;
    }

    /// Claim slot `idx`: the callee took the value over, or handed one back
    /// that nothing else names, so the frame releases it after all.
    pub(crate) fn own(&mut self, idx: usize) {
        assert!(idx < self.len, "no such argument");
        self.owned |= 1 << idx;
    }

    /// Drop the arguments past `n`: the ones the frame owns are released,
    /// the rest are only disowned.
    pub(crate) fn truncate(&mut self, n: usize) {
        while self.len > n {
            self.len -= 1;
            if self.owned & (1 << self.len) == 0 {
                self.slots[self.len].disown();
            } else {
                // The assignment releases what is there, which for an owned
                // slot is exactly right.
                self.slots[self.len] = TypVal::Unknown;
                self.owned &= !(1 << self.len);
            }
        }
    }

    /// Take the next slot, which the frame owns and releases, for a caller
    /// that fills one value at a time.  It arrives `VAR_UNKNOWN`, so a
    /// caller that fails to fill it releases nothing.
    pub(crate) fn claim(&mut self) -> &mut TypVal {
        assert!(self.len < N, "no room for another argument");
        self.owned |= 1 << self.len;
        self.len += 1;
        &mut self.slots[self.len - 1]
    }

    /// The slots past the ones in use, for a caller that fills them in
    /// place -- an argument parser writing into the frame it will then hand
    /// to the call.  Every slot it fills is the frame's, which
    /// [`fill`](Self::fill) records.
    #[inline]
    pub(crate) fn room(&mut self) -> &mut [TypVal] {
        &mut self.slots[self.len..]
    }

    /// Take over the first `n` slots of the last [`room`](Self::room): the
    /// frame owns them and releases them.
    pub(crate) fn fill(&mut self, n: usize) {
        assert!(self.len + n <= N, "no room for that many arguments");
        for _ in 0..n {
            self.owned |= 1 << self.len;
            self.len += 1;
        }
    }

    /// The arguments, as the slice a call reads them through.
    ///
    /// The borrow is the point: a callee reads its arguments and releases
    /// none, which is exactly what a shared slice says.
    #[inline]
    pub(crate) fn args(&self) -> &[TypVal] {
        &self.slots[..self.len]
    }
}

impl<const N: usize> Drop for CallFrame<N> {
    fn drop(&mut self) {
        self.truncate(0);
    }
}
