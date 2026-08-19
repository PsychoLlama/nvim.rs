//! The syntax-item flags.
//!
//! One bitmask, shared by three carriers: `synpat_T::sp_flags` (what a
//! `:syntax match`/`region` item was defined with), `keyentry::flags` (the
//! same for a `:syntax keyword`) and `stateitem_T::si_flags` (what the state
//! machine has since worked out about the item it is standing in). A handful
//! are only ever set on one of the three — [`SynFlags::MATCHCONT`] and
//! [`SynFlags::TRANS_CONT`] are state-machine deductions with no `:syntax`
//! keyword behind them, and `SYNC_HERE`/`SYNC_THERE` only mean anything on
//! a `:syntax sync` item.
//!
//! Named `HL_*` upstream, and unrelated to the attribute flags of the same
//! prefix in [`crate::highlight`] — in particular [`SynFlags::CONCEAL`] here
//! (a syntax item that `'conceallevel'` may hide) is not
//! `HlAttrFlags::CONCEALED` (the SGR "concealed" attribute). Sharing one C
//! prefix for two families is exactly what the two newtypes now prevent.
//! The values are `v0.12.4:src/nvim/syntax.h`'s anonymous enum.

#![forbid(unsafe_code)]

crate::flag_set! {
    /// One bitmask, shared by three carriers -- see the module docs.
    pub struct SynFlags;

    /// Not matched at the top level: only inside an item that `contains=` it.
    const CONTAINED = 0x01;

    /// Carries no highlighting of its own — the enclosing item's shows through.
    const TRANSP = 0x02;

    /// A region that must start and end on the same line.
    const ONELINE = 0x04;

    /// The end pattern matched `$`, so the region reaches the end of the line.
    const HAS_EOL = 0x08;

    /// `:syntax sync` item: the sync point is after this item.
    const SYNC_HERE = 0x10;

    /// `:syntax sync` item: the sync point is at the current line.
    const SYNC_THERE = 0x20;

    /// Highlight with the `matchgroup=` id rather than the item's own.
    const MATCH = 0x40;

    /// `nextgroup` may skip a newline to find its match.
    const SKIPNL = 0x80;

    /// `nextgroup` may skip white space to find its match.
    const SKIPWHITE = 0x100;

    /// `nextgroup` may skip empty lines to find its match.
    const SKIPEMPTY = 0x200;

    /// This item's end match wins over a contained item's, which cannot extend
    /// past it.
    const KEEPEND = 0x400;

    /// A trailing newline in the pattern is not part of the match.
    const EXCLUDENL = 0x800;

    /// Only used when drawing; skipped while syncing.
    const DISPLAY = 0x1000;

    /// The item defines a fold.
    const FOLD = 0x2000;

    /// Ignore an enclosing item's [`KEEPEND`](Self::KEEPEND).
    const EXTEND = 0x4000;

    /// State-machine deduction: this item's match was continued from the previous
    /// line rather than started on this one.
    const MATCHCONT = 0x8000;

    /// State-machine deduction: [`TRANSP`](Self::TRANSP) and the item has no
    /// `contains=`, so
    /// it takes both the highlighting and the containment of its parent.
    const TRANS_CONT = 0x1_0000;

    /// The item may be hidden by `'conceallevel'`.
    const CONCEAL = 0x2_0000;

    /// A region's start and end matches may be hidden by `'conceallevel'`.
    const CONCEALENDS = 0x4_0000;

    /// A top-level item of a `:syntax include`d syntax, which `contains=TOP`
    /// still admits.
    const INCLUDED_TOPLEVEL = 0x8_0000;
}
