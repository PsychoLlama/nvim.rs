#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type CharsOption = ::core::ffi::c_uint;

/// The set of characters 'breakat' allows a line to be broken before, one
/// bit per byte value.
///
/// A bitset rather than the 256-entry `char[]` upstream keeps, because
/// `vim_isbreak` asks this once per character of every drawn line and wants
/// [`GlobalCell::get`](crate::global_cell::GlobalCell::get) — the cheap
/// accessor — rather than `with`, whose debug-build borrow tracking costs
/// far more than the lookup, or a 256-byte copy per character.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct BreakAt([u64; 4]);

impl BreakAt {
    /// No character may be broken before.
    pub(crate) const NONE: Self = Self([0; 4]);

    /// Whether a line may be broken before `byte`.
    pub(crate) fn has(self, byte: u8) -> bool {
        self.0[usize::from(byte) / 64] >> (byte % 64) & 1 != 0
    }

    /// Allow a break before `byte`.
    pub(crate) fn insert(&mut self, byte: u8) {
        self.0[usize::from(byte) / 64] |= 1 << (byte % 64);
    }
}
