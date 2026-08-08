//! The C boundary. Everything in `xdiff/` that is not safe Rust lives here.
//!
//! `xdiff/` is vendored upstream (git's diff engine by way of libxdiff), and
//! the structs it exchanges with the rest of the tree — `mmfile_t`,
//! `mmbuffer_t`, `xpparam_t`, `xdemitconf_t` and `xdemitcb_t` — keep their C
//! layout because `diff.rs` and `lua/xdiff.rs` build them. This module reads
//! them once, at [`xdl_diff`], and hands the engine the safe mirrors in
//! [`super::xtypes`]; results go back out through [`Emit`].
//!
//! There are exactly three reasons this module is not safe: turning an
//! `mmfile_t` into a slice, calling a caller-supplied `extern "C"` callback,
//! and [`is_space`], which is `isspace(3)` and therefore reads the C
//! locale's table. That last one is kept rather than replaced with an ASCII
//! test because `setlocale(LC_ALL, "")` at startup leaves `LC_CTYPE` under
//! the user's control, so which bytes count as whitespace is observable
//! through `'diffopt'`'s `iwhite` family.
//!
//! The interface ported here is LibXDiff's `xdiff.h`, by Davide Libenzi
//! (File Differential Library), Copyright (C) 2003 Davide Libenzi. LibXDiff
//! is LGPL-2.1-or-later, and this port stays under that license (text:
//! licenses/LGPL-2.1.txt).

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_long};

#[cfg(not(miri))]
use crate::src::nvim::os::libc::__ctype_b_loc;
use crate::src::nvim::types::{
    mmbuffer_t, mmfile_t, xdemitcb_t, xdemitconf_t, xdl_emit_hunk_consume_func_t, xpparam_t,
};
use crate::src::xdiff::xdiffi::diff;
use crate::src::xdiff::xtypes::{Aborted, EmitConf, Params, XdResult};

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
pub fn is_space(byte: u8) -> bool {
    #[cfg(miri)]
    {
        matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
    }
    #[cfg(not(miri))]
    unsafe {
        *(*__ctype_b_loc()).offset(byte as isize) & IS_SPACE != 0
    }
}

/// The `mmfile_t`'s bytes.
///
/// # Safety
///
/// `mf` must point at an initialised `mmfile_t` whose `ptr`/`size` describe a
/// readable block that outlives `'a` and is not written during it.
unsafe fn file_bytes<'a>(mf: *const mmfile_t) -> &'a [u8] {
    unsafe {
        if (*mf).ptr.is_null() || (*mf).size <= 0 {
            return &[];
        }
        core::slice::from_raw_parts((*mf).ptr.cast::<u8>(), (*mf).size as usize)
    }
}

/// `xdemitcb_t` plus `xdemitconf_t.hunk_func`: the three ways the engine
/// reports a result, behind safe methods.
///
/// Holding the `xdemitcb_t` by shared reference is what keeps the function
/// pointers out of this module's own types. It is sound because nothing
/// writes the block during a diff — both callers build it on their stack and
/// hand the callbacks a *separate* `priv` pointer to accumulate into.
pub struct Emit<'a> {
    cb: &'a xdemitcb_t,
    hunk_func: xdl_emit_hunk_consume_func_t,
}

impl Emit<'_> {
    /// Did the caller install `xdemitconf_t.hunk_func`? That is what picks
    /// between the unified-diff writer and the hunk-extent walk, and both
    /// `:diffupdate` and `vim.diff{on_hunk=}` take it.
    pub fn has_hunk_func(&self) -> bool {
        self.hunk_func.is_some()
    }

    /// Did the caller install `xdemitcb_t.out_hunk`? Nothing in nvim does,
    /// but the field is public and a caller that sets it wants its headers.
    pub fn has_out_hunk(&self) -> bool {
        self.cb.out_hunk.is_some()
    }

    /// `ecb->out_line`, with each part as one `mmbuffer_t`.
    pub fn line(&mut self, parts: &[&[u8]]) -> XdResult {
        let mut bufs: [mmbuffer_t; 3] = [mmbuffer_t {
            ptr: core::ptr::null_mut(),
            size: 0,
        }; 3];
        for (buf, part) in bufs.iter_mut().zip(parts) {
            buf.ptr = part.as_ptr().cast::<c_char>().cast_mut();
            buf.size = part.len() as c_int;
        }
        // Upstream calls through the pointer unconditionally; every caller
        // that can reach an emit path installs it.
        let Some(out_line) = self.cb.out_line else {
            return Err(Aborted);
        };
        let rc = unsafe { out_line(self.cb.priv_0, bufs.as_mut_ptr(), parts.len() as c_int) };
        if rc < 0 { Err(Aborted) } else { Ok(()) }
    }

    /// `ecb->out_hunk`. The function-name arguments are always empty: the
    /// `XDL_EMIT_FUNCNAMES` machinery is `#if 0`-ed out of the vendored
    /// source, so `xdl_emit_diff`'s `func_line.len` never leaves zero.
    pub fn out_hunk(&mut self, s1: i64, c1: i64, s2: i64, c2: i64) -> XdResult {
        let Some(out_hunk) = self.cb.out_hunk else {
            return Err(Aborted);
        };
        let rc = unsafe {
            out_hunk(
                self.cb.priv_0,
                s1 as c_long,
                c1 as c_long,
                s2 as c_long,
                c2 as c_long,
                core::ptr::null(),
                0,
            )
        };
        if rc < 0 { Err(Aborted) } else { Ok(()) }
    }

    /// `xecfg->hunk_func`, called with `ecb->priv` as upstream does.
    pub fn hunk(&mut self, start_a: i64, count_a: i64, start_b: i64, count_b: i64) -> XdResult {
        let Some(hunk_func) = self.hunk_func else {
            return Err(Aborted);
        };
        let rc = unsafe {
            hunk_func(
                start_a as c_int,
                count_a as c_int,
                start_b as c_int,
                count_b as c_int,
                self.cb.priv_0,
            )
        };
        if rc < 0 { Err(Aborted) } else { Ok(()) }
    }
}

/// `xdl_diff`: diff `mf1` against `mf2` and report through `ecb`.
///
/// Returns 0, or -1 if a callback refused or the histogram engine hit one of
/// its hard limits — the only caller-visible failure.
///
/// # Safety
///
/// All five pointers must be non-null and point at initialised structs that
/// stay valid, and unwritten by anything but the callbacks, for the call;
/// `xpp->anchors` must hold `anchors_nr` NUL-terminated strings.
pub unsafe fn xdl_diff(
    mf1: *mut mmfile_t,
    mf2: *mut mmfile_t,
    xpp: *const xpparam_t,
    xecfg: *const xdemitconf_t,
    ecb: *mut xdemitcb_t,
) -> c_int {
    // One block for one obligation: the caller's five pointers are good.
    let (text1, text2, xpp, xecfg, cb) = unsafe {
        (
            file_bytes(mf1),
            file_bytes(mf2),
            &*xpp,
            &*xecfg,
            &*ecb.cast_const(),
        )
    };

    let mut anchors = Vec::with_capacity(xpp.anchors_nr);
    for i in 0..xpp.anchors_nr {
        // Each anchor is a `char *` the caller owns; `is_anchor` only wants
        // the bytes, so read them here and leave the pointer behind.
        anchors.push(unsafe { CStr::from_ptr(*xpp.anchors.add(i)) }.to_bytes());
    }

    let params = Params {
        flags: xpp.flags,
        anchors,
    };
    let conf = EmitConf {
        // A negative context length is not "less than none": it makes
        // `xdl_emit_diff` compute a hunk header describing a range that is
        // nowhere near the body it then writes, because the body's context
        // loops simply do not run for a negative count while the header
        // arithmetic still applies it. Clamping here is what makes the two
        // agree; `vim.diff{ctxlen = -1}` is now `ctxlen = 0`. See O-B15-1.
        ctxlen: xecfg.ctxlen.max(0),
        // `interhunkctxlen` needs no clamp: it only shrinks the run of
        // unchanged lines two hunks may share, and a negative one is a
        // coherent (if pointless) request to never join two hunks.
        interhunkctxlen: xecfg.interhunkctxlen,
        flags: xecfg.flags,
    };
    let mut emit = Emit {
        cb,
        hunk_func: xecfg.hunk_func,
    };

    match diff(text1, text2, &params, &conf, &mut emit) {
        Ok(()) => 0,
        Err(Aborted) => -1,
    }
}
