//! Parsing `'diffopt'` and `'diffanchors'`.
//!
//! [`diffopt_changed`] is the option's validator *and* its effect: it parses
//! the whole comma-separated value into the `diff_flags`, `diff_algorithm`,
//! `linematch_lines` and `diff_context` cells, rejecting the value whole if
//! any item is unknown.  [`parse_diffanchors`] is the separate
//! `'diffanchors'` grammar, which names line ranges the diff must be split
//! at.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, OK};
use crate::winlayer::{Buf, Win, tabs, windows};
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// The `'diffopt'` items that are nothing but a flag bit.
///
/// **Order matters**: the parse takes the first item this text starts with,
/// so `iwhiteall` and `iwhiteeol` have to precede `iwhite`.  No other pair
/// here is a prefix of another, and none is a prefix of the items with
/// arguments below, so the rest of the order is upstream's for readability
/// only.
const FLAG_ITEMS: &[(&[u8], c_int)] = &[
    (b"filler", DIFF_FILLER),
    (b"anchor", DIFF_ANCHOR),
    (b"iblank", DIFF_IBLANK),
    (b"icase", DIFF_ICASE),
    (b"iwhiteall", DIFF_IWHITEALL),
    (b"iwhiteeol", DIFF_IWHITEEOL),
    (b"iwhite", DIFF_IWHITE),
    (b"horizontal", DIFF_HORIZONTAL),
    (b"vertical", DIFF_VERTICAL),
    (b"hiddenoff", DIFF_HIDDEN_OFF),
    (b"closeoff", DIFF_CLOSE_OFF),
    (b"followwrap", DIFF_FOLLOWWRAP),
    (b"internal", DIFF_INTERNAL),
];

/// `algorithm:` values, and the `XDF_*` bits they select.
const ALGORITHMS: &[(&[u8], u64)] = &[
    (b"myers", 0),
    (b"minimal", XDF_NEED_MINIMAL),
    (b"patience", XDF_PATIENCE_DIFF),
    (b"histogram", XDF_HISTOGRAM_DIFF),
];

/// `inline:` values.  Each *replaces* the others, so the whole `ALL_INLINE`
/// group is cleared first.
const INLINE_MODES: &[(&[u8], c_int)] = &[
    (b"none", DIFF_INLINE_NONE),
    (b"simple", DIFF_INLINE_SIMPLE),
    (b"char", DIFF_INLINE_CHAR),
    (b"word", DIFF_INLINE_WORD),
];

const _: () = assert!(
    ALL_INLINE == DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD
);

/// The value in `text` after `prefix`, if the item is spelled that way.
///
/// Upstream requires a digit immediately after the colon as part of the *item
/// match*, so `context:x` does not match `context:` at all -- it falls out of
/// the chain with nothing consumed and fails the separator test below, which
/// is how the whole option value comes to be rejected.
fn number_item<'a>(text: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    let rest = text.strip_prefix(prefix)?;
    rest.first()
        .is_some_and(|b| b.is_ascii_digit())
        .then_some(rest)
}

/// Read `'diffanchors'` for `buf`, optionally into `anchors`.
///
/// Each item is an ordinary `:` address, which is why this has to make the
/// buffer and its window current around `get_address` -- the grammar reaches
/// marks, patterns and the cursor.  `check_only` is the option validator,
/// which runs before the buffer is necessarily in a window.
pub(crate) unsafe fn parse_diffanchors(
    check_only: bool,
    buf: Buf,
    anchors: *mut linenr_T,
    num_anchors: *mut c_int,
) -> c_int {
    let mut dia = if unsafe { *buf.b_p_dia } == 0 {
        p_dia.get()
    } else {
        buf.b_p_dia
    };
    let orig_curbuf = curbuf.get();
    let orig_curwin = curwin.get();

    let bufwin = if check_only {
        curwin.get()
    } else {
        let shown = windows().find(|w| w.w_buffer == buf.raw() && w.w_onebuf_opt.wo_diff != 0);
        if shown.is_none() && unsafe { *dia } != 0 {
            unsafe {
                emsg(gettext(
                    &raw const e_diff_anchors_with_hidden_windows as *const c_char,
                ))
            };
            return FAIL;
        }
        shown.map_or(::core::ptr::null_mut(), Win::raw)
    };

    let mut i = 0;
    while i < MAX_DIFF_ANCHORS && unsafe { *dia } != 0 {
        // An empty item -- a leading or doubled comma -- is not an
        // address, and `get_address` would answer the cursor line.
        if unsafe { *dia } == b',' as c_char {
            return FAIL;
        }
        curbuf.set(buf.raw());
        curwin.set(bufwin);
        let mut errormsg = None;
        let lnum = unsafe {
            get_address(
                ::core::ptr::null_mut(),
                &raw mut dia,
                CmdAddr::Lines,
                check_only,
                true,
                0,
                1,
                &mut errormsg,
            )
        };
        curbuf.set(orig_curbuf);
        curwin.set(orig_curwin);
        if let Some(msg) = &errormsg {
            unsafe { emsg(msg.as_ptr()) };
        }
        if dia.is_null() {
            return FAIL;
        }
        if unsafe { *dia } != b',' as c_char && unsafe { *dia } != 0 {
            return FAIL;
        }
        // The validator accepts an address it cannot resolve yet; only
        // the real parse insists the line exists.
        if !check_only
            && (lnum == MAXLNUM as linenr_T || lnum <= 0 || lnum > buf.b_ml.ml_line_count + 1)
        {
            unsafe { emsg(gettext(&raw const e_invrange as *const c_char)) };
            return FAIL;
        }
        if !anchors.is_null() {
            unsafe { *anchors.offset(i as isize) = lnum };
        }
        if unsafe { *dia } == b',' as c_char {
            dia = unsafe { dia.offset(1) };
        }
        i += 1;
    }
    if i == MAX_DIFF_ANCHORS && unsafe { *dia } != 0 {
        unsafe {
            semsg_c!(
                gettext(&raw const e_cannot_have_more_than_nr_diff_anchors as *const c_char),
                MAX_DIFF_ANCHORS,
            )
        };
        return FAIL;
    }
    if !num_anchors.is_null() {
        unsafe { *num_anchors = i };
    }
    OK
}

/// `'diffanchors'` was set: validate it, and invalidate the diffs it reaches.
///
/// `buflocal` says the buffer-local value changed rather than the global one,
/// so only tabpages showing the current buffer need recomputing.
pub unsafe fn diffanchors_changed(buflocal: bool) -> c_int {
    let result = unsafe {
        parse_diffanchors(
            true,
            Buf::current(),
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        )
    };
    if result != OK || diff_flags.get() & DIFF_ANCHOR == 0 {
        return result;
    }
    for mut tp in tabs() {
        if !buflocal || tp.tp_diffbuf.contains(&curbuf.get()) {
            tp.tp_diff_invalid = 1;
        }
    }
    result
}

/// `'diffopt'` was set: parse it whole, or reject it whole.
pub unsafe fn diffopt_changed() -> c_int {
    let mut context_new = 6;
    let mut foldcolumn_new = 2;
    let mut linematch_new = 0;
    let mut flags_new = 0;
    let mut algorithm_new: u64 = 0;
    let mut indent_heuristic: u64 = 0;

    let base = p_dip.get();
    let text = unsafe { CStr::from_ptr(base) }.to_bytes();
    // `getdigits_int` walks a `char *`, so the parse tracks an offset and
    // hands it the matching pointer where it needs one.
    let number_at = |at: usize, default| {
        let mut p = unsafe { base.add(at) };
        let n = unsafe { getdigits_int(&raw mut p, false, default) };
        (n, unsafe { p.offset_from(base) } as usize)
    };

    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if let Some((name, flag)) = FLAG_ITEMS.iter().find(|(name, _)| rest.starts_with(name)) {
            at += name.len();
            flags_new |= flag;
        } else if let Some(digits) = number_item(rest, b"context:") {
            (context_new, at) = number_at(text.len() - digits.len(), context_new);
        } else if let Some(digits) = number_item(rest, b"foldcolumn:") {
            (foldcolumn_new, at) = number_at(text.len() - digits.len(), foldcolumn_new);
        } else if let Some(digits) = number_item(rest, b"linematch:") {
            (linematch_new, at) = number_at(text.len() - digits.len(), linematch_new);
            // Line matching needs filler lines to show its alignment.
            flags_new |= DIFF_LINEMATCH | DIFF_FILLER;
        } else if rest.starts_with(b"indent-heuristic") {
            at += b"indent-heuristic".len();
            indent_heuristic = XDF_INDENT_HEURISTIC;
        } else if let Some(rest) = rest.strip_prefix(b"algorithm:") {
            let Some((name, bits)) = ALGORITHMS.iter().find(|(n, _)| rest.starts_with(n)) else {
                return FAIL;
            };
            at += b"algorithm:".len() + name.len();
            algorithm_new = *bits;
        } else if let Some(rest) = rest.strip_prefix(b"inline:") {
            let Some((name, bit)) = INLINE_MODES.iter().find(|(n, _)| rest.starts_with(n)) else {
                return FAIL;
            };
            at += b"inline:".len() + name.len();
            flags_new = flags_new & !ALL_INLINE | bit;
        }
        // An item nothing matched consumed nothing, so this is where an
        // unknown spelling is rejected -- and where `context:x` lands too.
        if at < text.len() {
            if text[at] != b',' {
                return FAIL;
            }
            at += 1;
        }
    }

    algorithm_new |= indent_heuristic;
    // The two layouts are mutually exclusive; neither wins.
    if flags_new & DIFF_HORIZONTAL != 0 && flags_new & DIFF_VERTICAL != 0 {
        return FAIL;
    }

    if diff_flags.get() != flags_new || diff_algorithm.get() != algorithm_new {
        for mut tp in tabs() {
            tp.tp_diff_invalid = 1;
        }
    }
    diff_flags.set(flags_new);
    // `context:0` would fold every unchanged line, including the ones
    // either side of a change; one is the floor.
    diff_context.set(if context_new == 0 { 1 } else { context_new });
    linematch_lines.set(linematch_new);
    diff_foldcolumn.set(foldcolumn_new);
    diff_algorithm.set(algorithm_new);
    unsafe { diff_redraw(true) };
    unsafe { check_scrollbind(0, 0) };
    OK
}

pub fn diffopt_horizontal() -> bool {
    diff_flags.get() & DIFF_HORIZONTAL != 0
}

pub fn diffopt_hiddenoff() -> bool {
    diff_flags.get() & DIFF_HIDDEN_OFF != 0
}

pub fn diffopt_closeoff() -> bool {
    diff_flags.get() & DIFF_CLOSE_OFF != 0
}

pub fn diffopt_filler() -> bool {
    diff_flags.get() & DIFF_FILLER != 0
}
