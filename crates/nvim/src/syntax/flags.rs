//! The syntax-item flags.
//!
//! One bitmask, shared by three carriers: `synpat_T::sp_flags` (what a
//! `:syntax match`/`region` item was defined with), `keyentry::flags` (the
//! same for a `:syntax keyword`) and `stateitem_T::si_flags` (what the state
//! machine has since worked out about the item it is standing in). A handful
//! are only ever set on one of the three — [`HL_MATCHCONT`] and
//! [`HL_TRANS_CONT`] are state-machine deductions with no `:syntax` keyword
//! behind them, and [`HL_SYNC_HERE`]/[`HL_SYNC_THERE`] only mean anything on
//! a `:syntax sync` item.
//!
//! Named `HL_*` upstream, and unrelated to the attribute flags of the same
//! prefix in [`crate::nvim::highlight`] — in particular [`HL_CONCEAL`] here
//! (a syntax item that `'conceallevel'` may hide) is not
//! `highlight::HL_CONCEALED` (the SGR "concealed" attribute). The values are
//! `v0.12.4:src/nvim/syntax.h`'s anonymous enum.
//!
//! Typed `c_int` because that is what all three carriers are.

#![forbid(unsafe_code)]

use core::ffi::c_int;

/// Not matched at the top level: only inside an item that `contains=` it.
pub const HL_CONTAINED: c_int = 0x01;
/// Carries no highlighting of its own — the enclosing item's shows through.
pub const HL_TRANSP: c_int = 0x02;
/// A region that must start and end on the same line.
pub const HL_ONELINE: c_int = 0x04;
/// The end pattern matched `$`, so the region reaches the end of the line.
pub const HL_HAS_EOL: c_int = 0x08;
/// `:syntax sync` item: the sync point is after this item.
pub const HL_SYNC_HERE: c_int = 0x10;
/// `:syntax sync` item: the sync point is at the current line.
pub const HL_SYNC_THERE: c_int = 0x20;
/// Highlight with the `matchgroup=` id rather than the item's own.
pub const HL_MATCH: c_int = 0x40;
/// `nextgroup` may skip a newline to find its match.
pub const HL_SKIPNL: c_int = 0x80;
/// `nextgroup` may skip white space to find its match.
pub const HL_SKIPWHITE: c_int = 0x100;
/// `nextgroup` may skip empty lines to find its match.
pub const HL_SKIPEMPTY: c_int = 0x200;
/// This item's end match wins over a contained item's, which cannot extend
/// past it.
pub const HL_KEEPEND: c_int = 0x400;
/// A trailing newline in the pattern is not part of the match.
pub const HL_EXCLUDENL: c_int = 0x800;
/// Only used when drawing; skipped while syncing.
pub const HL_DISPLAY: c_int = 0x1000;
/// The item defines a fold.
pub const HL_FOLD: c_int = 0x2000;
/// Ignore an enclosing item's [`HL_KEEPEND`].
pub const HL_EXTEND: c_int = 0x4000;
/// State-machine deduction: this item's match was continued from the previous
/// line rather than started on this one.
pub const HL_MATCHCONT: c_int = 0x8000;
/// State-machine deduction: [`HL_TRANSP`] and the item has no `contains=`, so
/// it takes both the highlighting and the containment of its parent.
pub const HL_TRANS_CONT: c_int = 0x1_0000;
/// The item may be hidden by `'conceallevel'`.
pub const HL_CONCEAL: c_int = 0x2_0000;
/// A region's start and end matches may be hidden by `'conceallevel'`.
pub const HL_CONCEALENDS: c_int = 0x4_0000;
/// A top-level item of a `:syntax include`d syntax, which `contains=TOP`
/// still admits.
pub const HL_INCLUDED_TOPLEVEL: c_int = 0x8_0000;
