//! 'breakindent': the indent a wrapped line's continuation carries, and
//! the 'breakindentopt' value that shapes it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::CStr;

use super::*;
use crate::buffer::buf_get_changedtick;
use crate::charset::{getdigits_at, getdigits_int_at};
use crate::global_cell::GlobalCell;
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::r#move::win_col_off2;
use crate::narrow::number_as_int;
use crate::option::vars::P_FLP;
use crate::option::vars::dy_flags;
use crate::optionstr::local_or_global;
use crate::optionstr::{OptString, OptStringRef};
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
fn parse_briopt(bytes: &[u8]) -> Option<Briopt> {
    let mut opt = Briopt::default();
    let number = |at: &mut usize, strict: bool| -> c_int {
        let n;
        (n, *at) = if strict {
            getdigits_int_at(bytes, *at, true, 0)
        } else {
            let (n, past) = getdigits_at(bytes, *at, false, 0);
            (number_as_int(n), past)
        };
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

/// [`briopt_check`] over the window's own 'breakindentopt'.
pub(crate) fn check_window_briopt(window: Win) -> bool {
    briopt_check(None, Some(window))
}

/// Checks `briopt` as 'breakindentopt' and, when `window` is not null, writes
/// what it says onto that window. Called when the option is set and when a
/// window is initialised.
///
/// No `briopt` reads the window's own value; no `window` only checks.
pub fn briopt_check(briopt: Option<&CStr>, window: Option<Win>) -> bool {
    // The caller's option string, or the window's own copy of it, which the
    // parse reads before anything below writes the window.
    let parsed = match (briopt, window) {
        (Some(briopt), _) => parse_briopt(briopt.to_bytes()),
        (None, Some(w)) => parse_briopt(w.w_onebuf_opt.wo_briopt.bytes()),
        // Upstream reads `empty_string_option` here, which is only ever
        // the empty string and is never written through.
        (None, None) => parse_briopt(b""),
    };
    let Some(opt) = parsed else {
        return false;
    };
    let Some(mut window) = window else {
        return true; // Only the check was asked for.
    };
    window.w_briopt_shift = opt.shift;
    window.w_briopt_min = opt.min;
    window.w_briopt_sbr = opt.sbr;
    window.w_briopt_list = opt.list;
    window.w_briopt_vcol = opt.vcol;
    true
}

/// Everything the cached indent depends on that is a plain value. The three
/// that are not — the line, 'vartabstop' and 'formatlistpat' — are compared
/// by content.
#[derive(Clone, Copy, PartialEq, Eq)]
struct BreakindentKey {
    fnum: c_int,
    ts: OptInt,
    tick: VarNumber,
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
    /// Owned copies of the line, 'vartabstop' and 'formatlistpat'; the line
    /// and the pattern are absent before the first miss.
    line: Option<XString>,
    vts: TabStopArray,
    flp: Option<XString>,
}

static CACHE: GlobalCell<BreakindentCache> = GlobalCell::new(BreakindentCache {
    key: BreakindentKey {
        fnum: 0,
        ts: 0,
        tick: 0,
        listopt: 0,
        no_ts: false,
        dy_uhex: 0,
    },
    indent: 0,
    list: 0,
    line: None,
    vts: None,
    flp: None,
});

impl BreakindentCache {
    /// Whether the cached indent still answers for this line under these
    /// options.
    fn answers(
        &self,
        key: &BreakindentKey,
        line: &[u8],
        vts: Option<&[ColNr]>,
        flp: &[u8],
    ) -> bool {
        self.key == *key
            && self.flp.as_deref() == Some(flp)
            && self.line.as_deref() == Some(line)
            && self.vts.as_deref() == vts
    }

    /// Measures `line`'s indent and stores it against `key`.
    ///
    /// `window` is the window whose buffer `key` and `vts` were read from.
    fn refill(
        &mut self,
        window: Win,
        key: &BreakindentKey,
        line: &[u8],
        vts: TabStopArray,
        flp: XString,
    ) {
        let owned = XString::from_bytes(line);
        self.key = *key;
        self.list = 0;
        // A fixed column needs no measurement.
        if window.w_briopt_vcol == 0 {
            self.indent = if key.no_ts {
                indent_size_no_ts(line)
            } else {
                indent_size_ts(line, key.ts, vts.as_deref())
            };
            if window.w_briopt_list != 0 {
                self.add_list_indent(window, owned.as_cstr(), flp.as_cstr());
            }
        }
        self.line = Some(owned);
        self.vts = vts;
        self.flp = Some(flp);
    }

    /// The extra indent a numbered list asks for: either the flat
    /// 'breakindentopt' "list" value, or — when that is negative — the width
    /// of what 'formatlistpat' matched, which then *replaces* the indent.
    fn add_list_indent(&mut self, window: Win, line: &CStr, flp: &CStr) {
        let mut regmatch: RegMatch = RegMatch::new(
            vim_regcomp(flp, RE_MAGIC + RE_STRING + RE_AUTO + RE_STRICT),
            false,
        );
        if regmatch.regprog.is_none() {
            return;
        }
        if vim_regexec(&mut regmatch, line, 0) {
            if window.w_briopt_list > 0 {
                self.list += window.w_briopt_list;
            } else {
                // Measure the match with `win_chartabsize`, so that a TAB
                // is the right width and wrapping is ignored.
                let span = regmatch.group(0).unwrap_or(0..0);
                let bytes = line.to_bytes();
                let mut at = span.start;
                let mut indent = 0;
                while at < span.end {
                    indent += win_chartabsize(window, &bytes[at..], indent as ColNr);
                    at += c_cluster_len(&bytes[at..]);
                }
                self.indent = indent;
            }
        }
    }
}

/// The indent a wrapped line's continuation carries, in screen cells.
///
/// The window has to be named because it is not necessarily the current one.
///
/// `line` is the text up to its first NUL, as the C string it stands for.
pub fn get_breakindent_win(window: Win, line: &[u8]) -> c_int {
    let line = cstr::until_nul(line);
    // SAFETY: the caller's window and its buffer.
    // SAFETY: the caller's window; a live window has a live buffer.
    let win = window;
    let buf = win.buffer();
    let key = BreakindentKey {
        fnum: buf.handle,
        ts: buf.b_p_ts,
        tick: buf_get_changedtick(buf),
        listopt: win.w_briopt_list,
        no_ts: win.w_onebuf_opt.wo_list != 0 && win.w_p_lcs_chars.tab1 == NUL as ScreenChar,
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
    let col_off2 = win_col_off2(win);
    let flp = local_or_global(&buf.b_p_flp, P_FLP);
    // The window width minus its margins: what is left for text.
    let eff_wwidth = win.w_view_width - (window).col_off() + col_off2;
    // One exclusive borrow for the whole computation: nothing below calls
    // back into this function (the regex engine and chartabsize helpers run
    // no user code), and debug builds will catch it if that ever changes.
    let vts = buf.b_p_vts_array.as_deref();
    let hit = flp.with_bytes(|flp| CACHE.with(|prev| prev.answers(&key, line, vts, flp)));
    // Copies of what the cache keys on, taken only on a miss.
    let fill = (!hit).then(|| (vts.map(Box::from), flp.get()));
    let mut bri = CACHE.with_mut(|prev| {
        if let Some((vts, flp)) = fill {
            prev.refill(window, &key, line, vts, flp);
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
        bri -= win.showbreak_cells();
    }
    // Never indent past the left window margin, and always leave `min`
    // columns for the text when the window is wide enough for them.
    bri.clamp(0, (eff_wwidth - opt.min).max(0))
}
