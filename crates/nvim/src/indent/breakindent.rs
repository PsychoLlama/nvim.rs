//! 'breakindent': the indent a wrapped line's continuation carries, and
//! the 'breakindentopt' value that shapes it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::CStr;

use super::*;
use crate::buffer::buf_get_changedtick;
use crate::charset::{getdigits, getdigits_int, vim_strsize};
use crate::global_cell::GlobalCell;
use crate::main::dy_flags;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xstrdup};
use crate::r#move::{win_col_off, win_col_off2};
use crate::option::{get_flp_value, get_showbreak_value};
use crate::plines::win_chartabsize;
use crate::regexp::{RE_AUTO, RE_MAGIC, RE_STRICT, RE_STRING};
use crate::winlayer::Win;

/// 'breakindentopt', parsed: the five values [`briopt_check`] writes onto a
/// window as `w_briopt_*`.
struct Briopt {
    /// "shift:" — added to the measured indent.
    shift: c_int,
    /// "min:" — columns that must be left on the left of the text.
    min: c_int,
    /// "sbr" — take 'showbreak' out of the indent.
    sbr: bool,
    /// "list:" — extra indent for a numbered list; negative means "measure
    /// the 'formatlistpat' match instead".
    list: c_int,
    /// "column:" — a fixed column, which overrides the measured indent.
    vcol: c_int,
}

impl Default for Briopt {
    fn default() -> Self {
        // `min` is the only one whose default is not zero.
        Self {
            shift: 0,
            min: 20,
            sbr: false,
            list: 0,
            vcol: 0,
        }
    }
}

/// Parses one 'breakindentopt' value; `None` is a malformed one, which is
/// what upstream reports as `FAIL`.
///
/// Keep this in sync with `opt_briopt_values`, which is what completes it.
///
/// # Safety
/// `value` must be a NUL-terminated string.
unsafe fn parse_briopt(value: *const c_char) -> Option<Briopt> {
    let mut opt = Briopt::default();
    // SAFETY: the caller's NUL-terminated option value.
    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    // `getdigits` walks a `char *` of its own, so the cursor is kept here as
    // an offset and handed over as a pointer only for the digits themselves.
    let number = |at: &mut usize, strict: bool| -> c_int {
        // SAFETY: `at` is an offset into `value`, so the cursor starts inside
        // the string, and `getdigits` only advances it over digits.
        let mut p = unsafe { value.cast_mut().add(*at) };
        let n = if strict {
            unsafe { getdigits_int(&raw mut p, true, 0) }
        } else {
            unsafe { getdigits(&raw mut p, false, 0) as c_int }
        };
        *at = unsafe { p.offset_from(value) } as usize;
        n
    };
    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        let digit_at = |n: usize| rest.get(n).is_some_and(u8::is_ascii_digit);
        if rest.starts_with(b"shift:") && (rest.get(6) == Some(&b'-') && digit_at(7) || digit_at(6))
        {
            i += 6;
            opt.shift = number(&mut i, true);
        } else if rest.starts_with(b"min:") && digit_at(4) {
            i += 4;
            opt.min = number(&mut i, true);
        } else if rest.starts_with(b"sbr") {
            i += 3;
            opt.sbr = true;
        } else if rest.starts_with(b"list:") {
            i += 5;
            opt.list = number(&mut i, false);
        } else if rest.starts_with(b"column:") {
            i += 7;
            opt.vcol = number(&mut i, false);
        }
        // Anything an arm did not consume has to be an entry separator, or
        // the value is malformed. An unrecognised name lands here unmoved.
        match bytes.get(i) {
            None => {}
            Some(b',') => i += 1,
            Some(_) => return None,
        }
    }
    Some(opt)
}

/// Checks `briopt` as 'breakindentopt' and, when `wp` is not null, writes
/// what it says onto that window. Called when the option is set and when a
/// window is initialised.
///
/// A null `briopt` reads the window's own value; a null `wp` only checks.
///
/// # Safety
/// `briopt` must be null or a NUL-terminated string, and `wp` null or a
/// window.
pub unsafe fn briopt_check(briopt: *mut c_char, wp: *mut win_T) -> bool {
    // SAFETY: the caller's option string, or the window's own copy of it.
    let value = unsafe {
        if !briopt.is_null() {
            briopt.cast_const()
        } else if !wp.is_null() {
            (*wp).w_onebuf_opt.wo_briopt.cast_const()
        } else {
            // Upstream reads `empty_string_option` here, which is only ever
            // the empty string and is never written through.
            c"".as_ptr()
        }
    };
    // SAFETY: `value` is one of three NUL-terminated strings.
    let Some(opt) = (unsafe { parse_briopt(value) }) else {
        return false;
    };
    if wp.is_null() {
        return true; // Only the check was asked for.
    }
    // SAFETY: the caller's window.
    unsafe { (*wp).w_briopt_shift = opt.shift };
    unsafe { (*wp).w_briopt_min = opt.min };
    unsafe { (*wp).w_briopt_sbr = opt.sbr };
    unsafe { (*wp).w_briopt_list = opt.list };
    unsafe { (*wp).w_briopt_vcol = opt.vcol };
    true
}

/// Everything the cached indent depends on that is a plain value. The two
/// that are not — the line and 'formatlistpat' — are compared as strings.
#[derive(Clone, Copy, PartialEq, Eq)]
struct BreakindentKey {
    fnum: c_int,
    ts: OptInt,
    vts: *mut colnr_T,
    tick: varnumber_T,
    /// 'breakindentopt' "list".
    listopt: c_int,
    /// In 'list' mode with no "tab" in 'listchars', a TAB shows as `^I`.
    no_ts: bool,
    /// 'display' "uhex".
    dy_uhex: c_uint,
}

/// The last indent [`get_breakindent_win`] measured, and what it measured it
/// from.
///
/// Upstream's eleven function-local `static`s. The answer is asked for once
/// per screen line of a wrapped line, so a miss here is a redraw's worth of
/// `indent_size_*` and one regex match.
struct BreakindentCache {
    key: BreakindentKey,
    /// The measured indent, in screen cells.
    indent: c_int,
    /// The extra indent 'breakindentopt' "list" asked for.
    list: c_int,
    /// Owned copies of the line and of 'formatlistpat'; null before the
    /// first miss.
    line: *mut c_char,
    flp: *mut c_char,
}

static CACHE: GlobalCell<BreakindentCache> = GlobalCell::new(BreakindentCache {
    key: BreakindentKey {
        fnum: 0,
        ts: 0,
        vts: ::core::ptr::null_mut(),
        tick: 0,
        listopt: 0,
        no_ts: false,
        dy_uhex: 0,
    },
    indent: 0,
    list: 0,
    line: ::core::ptr::null_mut(),
    flp: ::core::ptr::null_mut(),
});

impl BreakindentCache {
    /// Whether the cached indent still answers for this line under these
    /// options.
    ///
    /// # Safety
    /// `line` and `flp` must be NUL-terminated strings.
    unsafe fn answers(
        &self,
        key: &BreakindentKey,
        line: *const c_char,
        flp: *const c_char,
    ) -> bool {
        // SAFETY: the cache's own copies are NUL-terminated when non-null,
        // and the caller's two strings are as well.
        self.key == *key
            && !self.flp.is_null()
            && unsafe { cstr::eq(self.flp, flp) }
            && !self.line.is_null()
            && unsafe { cstr::eq(self.line, line) }
    }

    /// Measures `line`'s indent and stores it against `key`.
    ///
    /// # Safety
    /// `wp` must be a window whose buffer `key` was read from, and `line`
    /// and `flp` NUL-terminated strings.
    unsafe fn refill(
        &mut self,
        wp: *mut win_T,
        key: &BreakindentKey,
        line: *mut c_char,
        flp: *const c_char,
    ) {
        // SAFETY: the caller's window and strings; the two copies this
        // replaces are the cache's own allocations.
        unsafe { xfree(self.line.cast()) };
        self.line = unsafe { xstrdup(line) };
        unsafe { xfree(self.flp.cast()) };
        self.flp = unsafe { xstrdup(flp) };
        self.key = *key;
        self.list = 0;
        if unsafe { (*wp).w_briopt_vcol } != 0 {
            // A fixed column needs no measurement.
            return;
        }
        self.indent = if key.no_ts {
            unsafe { indent_size_no_ts(line) }
        } else {
            unsafe { indent_size_ts(line, key.ts, key.vts) }
        };
        if unsafe { (*wp).w_briopt_list } != 0 {
            unsafe { self.add_list_indent(wp, line) };
        }
    }

    /// The extra indent a numbered list asks for: either the flat
    /// 'breakindentopt' "list" value, or — when that is negative — the width
    /// of what 'formatlistpat' matched, which then *replaces* the indent.
    ///
    /// # Safety
    /// `wp` must be a window and `line` a NUL-terminated string; `self.flp`
    /// must hold the current 'formatlistpat'.
    unsafe fn add_list_indent(&mut self, wp: *mut win_T, line: *mut c_char) {
        // SAFETY: the caller's window and line, and the cache's own pattern.
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: unsafe { vim_regcomp(self.flp, RE_MAGIC + RE_STRING + RE_AUTO + RE_STRICT) },
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        if regmatch.regprog.is_null() {
            return;
        }
        if unsafe { vim_regexec(&raw mut regmatch, line, 0 as colnr_T) } {
            if unsafe { (*wp).w_briopt_list } > 0 {
                self.list += unsafe { (*wp).w_briopt_list };
            } else {
                // Measure the match with `win_chartabsize`, so that a TAB
                // is the right width and wrapping is ignored.
                let end = regmatch.endp[0];
                let mut ptr = regmatch.startp[0];
                let mut indent = 0;
                while ptr < end {
                    indent += unsafe { win_chartabsize(Win::new(wp), ptr, indent as colnr_T) };
                    ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
                }
                self.indent = indent;
            }
        }
        unsafe { vim_regfree(regmatch.regprog) };
    }
}

/// The indent a wrapped line's continuation carries, in screen cells.
///
/// The window has to be named because it is not necessarily the current one.
///
/// # Safety
/// `wp` must be a window and `line` a NUL-terminated string.
pub unsafe fn get_breakindent_win(wp: *mut win_T, line: *mut c_char) -> c_int {
    // SAFETY: the caller's window and its buffer.
    // SAFETY: the caller's window; a live window has a live buffer.
    let win = unsafe { Win::new(wp) };
    let buf = win.buffer();
    let key = BreakindentKey {
        fnum: buf.handle,
        ts: buf.b_p_ts,
        vts: buf.b_p_vts_array,
        tick: buf_get_changedtick(buf),
        listopt: win.w_briopt_list,
        no_ts: win.w_onebuf_opt.wo_list != 0 && win.w_p_lcs_chars.tab1 == NUL as schar_T,
        dy_uhex: dy_flags.get() & kOptDyFlagUhex as c_uint,
    };
    let opt = Briopt {
        shift: win.w_briopt_shift,
        min: win.w_briopt_min,
        sbr: win.w_briopt_sbr,
        list: win.w_briopt_list,
        vcol: win.w_briopt_vcol,
    };
    // SAFETY: a live window.
    let (col_off2, flp) = (win_col_off2(win), get_flp_value(buf));
    // The window width minus its margins: what is left for text.
    let eff_wwidth = win.w_view_width - unsafe { win_col_off(wp) } + col_off2;
    // One exclusive borrow for the whole computation: nothing below calls
    // back into this function (the regex engine and chartabsize helpers run
    // no user code), and debug builds will catch it if that ever changes.
    let mut bri = CACHE.with_mut(|prev| {
        // SAFETY: `line` and `flp` are NUL-terminated, and `wp` is the
        // caller's window, which `key` was just read from.
        if !unsafe { prev.answers(&key, line, flp) } {
            unsafe { prev.refill(wp, &key, line, flp) };
        }
        let mut bri = if opt.vcol != 0 {
            // A column value has priority over the measured indent.
            prev.list = 0;
            opt.vcol
        } else {
            prev.indent + opt.shift
        };
        // Offset for the number column, if 'n' is in 'cpoptions'.
        bri += col_off2;
        if opt.list > 0 {
            bri += prev.list;
        }
        bri
    });
    if opt.sbr {
        // SAFETY: 'showbreak' is a NUL-terminated option value.
        bri -= unsafe { vim_strsize(get_showbreak_value(win)) };
    }
    // Never indent past the left window margin, and always leave `min`
    // columns for the text when the window is wide enough for them.
    bri.clamp(0, (eff_wwidth - opt.min).max(0))
}
