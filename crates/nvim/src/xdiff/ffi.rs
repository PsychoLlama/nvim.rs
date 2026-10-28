//! The C boundary. Everything in `xdiff/` that is not safe Rust lives here.
//!
//! `xdiff/` is vendored upstream (git's diff engine by way of libxdiff). Its
//! callers hold their texts as slices and take the results through [`Emit`]:
//! [`diff_hunks`] for the hunk extents `:diff` wants, [`diff_with`] for
//! `vim.diff()`, which may want a unified diff's lines instead.
//!
//! There is one reason this module is not safe: [`is_space`], which is
//! `isspace(3)` and therefore reads the C locale's table. That last one is kept rather than replaced with an ASCII
//! test because `setlocale(LC_ALL, "")` at startup leaves `LC_CTYPE` under
//! the user's control, so which bytes count as whitespace is observable
//! through `'diffopt'`'s `iwhite` family.
//!
//! The interface ported here is LibXDiff's `xdiff.h`, by Davide Libenzi
//! (File Differential Library), Copyright (C) 2003 Davide Libenzi. LibXDiff
//! is LGPL-2.1-or-later, and this port stays under that license (text:
//! licenses/LGPL-2.1.txt).

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `xdiff/` row in docs/perimeter.md.
#![allow(unsafe_code)]

#[cfg(not(miri))]
use crate::os::cshim::__ctype_b_loc;
use crate::xdiff::xdiffi::diff;
use crate::xdiff::xtypes::{Aborted, EmitConf, Params, XdResult};

/// glibc's `_ISspace` bit in the `__ctype_b_loc` table.
#[cfg(not(miri))]
const IS_SPACE: u16 = 8192;

/// `XDL_ISSPACE`: `isspace((unsigned char) c)`, locale and all.
///
/// Safe to call: the table `__ctype_b_loc` returns is 384 entries wide and
/// indexed `-128 ..= 255`, so every `u8` is in range.
///
/// Under Miri it is the `"C"` locale's answer spelled out — Miri cannot call
/// a foreign function, and the lane runs with no locale set anyway, so this
/// *is* what libc would say there.
pub(crate) fn is_space(byte: u8) -> bool {
    #[cfg(miri)]
    {
        matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
    }
    #[cfg(not(miri))]
    unsafe {
        *(*__ctype_b_loc()).offset(byte as isize) & IS_SPACE != 0
    }
}

/// Where the engine reports a result: the lines of a unified diff, or each
/// hunk's extent.
pub(crate) enum Emit<'a> {
    /// A unified diff, a line at a time, each line in up to three parts.
    Lines(&'a mut dyn FnMut(&[&[u8]]) -> XdResult),
    /// Hunk extents only, never a line.
    Hunks(&'a mut dyn FnMut(i64, i64, i64, i64) -> XdResult),
}

impl Emit<'_> {
    /// Is there a hunk consumer? That is what picks between the
    /// unified-diff writer and the hunk-extent walk, and `:diffupdate`,
    /// the inline refiner and `vim.diff{on_hunk=}` all take it.
    pub(crate) fn has_hunk_func(&self) -> bool {
        matches!(self, Emit::Hunks(_))
    }

    /// One line of a unified diff, in parts.
    pub(crate) fn line(&mut self, parts: &[&[u8]]) -> XdResult {
        match self {
            Emit::Lines(on_line) => on_line(parts),
            Emit::Hunks(_) => Err(Aborted),
        }
    }

    /// One hunk's extent: zero-based starts and counts on either side.
    pub(crate) fn hunk(
        &mut self,
        start_a: i64,
        count_a: i64,
        start_b: i64,
        count_b: i64,
    ) -> XdResult {
        match self {
            Emit::Hunks(on_hunk) => on_hunk(start_a, count_a, start_b, count_b),
            Emit::Lines(_) => Err(Aborted),
        }
    }
}

/// Diff `text1` against `text2` with the `XDF_*` bits `flags` and no context,
/// reporting each hunk's zero-based start and count on either side to
/// `on_hunk`.
pub(crate) fn diff_hunks(
    text1: &[u8],
    text2: &[u8],
    flags: u64,
    on_hunk: &mut dyn FnMut(i64, i64, i64, i64) -> XdResult,
) -> XdResult {
    diff_with(text1, text2, flags, 0, 0, &mut Emit::Hunks(on_hunk))
}

/// Diff `text1` against `text2` with the `XDF_*` bits `flags`, reporting
/// through `emit`: `ctxlen` unchanged lines either side of a hunk, and hunks
/// joined when `interhunkctxlen` or fewer unchanged lines separate them.
pub(crate) fn diff_with(
    text1: &[u8],
    text2: &[u8],
    flags: u64,
    ctxlen: i64,
    interhunkctxlen: i64,
    emit: &mut Emit<'_>,
) -> XdResult {
    let params = Params {
        flags,
        anchors: Vec::new(),
    };
    let conf = EmitConf {
        // A negative context length is not "less than none": it makes
        // `xdl_emit_diff` compute a hunk header describing a range that is
        // nowhere near the body it then writes, because the body's context
        // loops simply do not run for a negative count while the header
        // arithmetic still applies it. Clamping here is what makes the two
        // agree; `vim.diff{ctxlen = -1}` is now `ctxlen = 0`. See O-B15-1.
        ctxlen: ctxlen.max(0),
        // `interhunkctxlen` needs no clamp: it only shrinks the run of
        // unchanged lines two hunks may share, and a negative one is a
        // coherent (if pointless) request to never join two hunks.
        interhunkctxlen,
        flags: 0,
    };
    diff(text1, text2, &params, &conf, emit)
}
