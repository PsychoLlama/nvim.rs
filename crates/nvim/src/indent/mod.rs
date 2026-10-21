//! Indent width, and the two questions every indent engine ends in.
//!
//! "How wide is this line's indent" ([`get_indent`] and the `indent_size_*`
//! pair) and "make it this wide" ([`set_indent`]). The tabstop arithmetic
//! both are written against lives in [`tabstop`]; the engines that decide
//! *what* the width should be are `indent_c.rs` (C), [`expr`] ('indentexpr'
//! and Lisp) and [`breakindent`] ('breakindent'), and [`edit`] holds the
//! commands that apply one.
//!
//! | file | what |
//! | --- | --- |
//! | `tabstop` | the 'vartabstop' list and the uniform-width fallback |
//! | `edit` | `=`, `:retab`, `<C-t>`/`<C-d>`, smart indent, `copy_indent` |
//! | `expr` | 'indentexpr', `lispindent()`, `indent()` |
//! | `breakindent` | 'breakindentopt' and the wrapped-line indent |

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::semsg;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint};

use crate::ascii::ascii_iswhite;
use crate::change::{changed_bytes, get_leader_len};
use crate::charset::{byte2cells, char2cells, getwhitecols_curline, skip};
use crate::cstr::byte_at;
use crate::edit::get_nolist_virtcol;
use crate::extmark::extmark_splice_cols;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::memline::Lines;
use crate::memline::ml_replace_owned;
use crate::memory::XString;
use crate::message::e_positive;
use crate::message::emsg;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::options::kOptDyFlagUhex;
use crate::os::cshim::gettext;
use crate::plines::getvcol;
use crate::state::mode::{State, saved_cursor};
use crate::textformat::has_format_option;
use crate::types::*;

// `regexp.rs` keeps its own copy of `RegProg`, so these stay declarations
// rather than imports. `breakindent.rs` reaches them through `use super::*`.
use crate::undo::u_savesub;

pub mod breakindent;
pub mod edit;
pub mod expr;
pub mod tabstop;

// Split out for size. The names below are what the rest of the tree calls,
// and it calls them as `indent::*`.
use crate::optionstr::OptString;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec};
use crate::state::MODE_INSERT;
use crate::winlayer::Win;
pub(crate) use breakindent::check_window_briopt;
pub use breakindent::{briopt_check, get_breakindent_win};
pub use edit::{
    change_indent, copy_indent, ex_retab, inindent, ins_try_si, may_do_si, op_reindent,
    preprocs_left,
};
pub use expr::{
    f_indent, f_lispindent, fix_indent, fixthisline, get_expr_indent, get_lisp_indent,
    use_indentexpr_for_lisp,
};

const INDENT_SET: c_uint = 1;
const INDENT_DEC: c_uint = 3;
const kBufOptIndentexpr: c_uint = 47;
const kExtmarkUndo: ExtmarkOp = 1;
const kOptIndentexpr: OptIndex = 148;
const kOptVartabstop: OptIndex = 338;
const TAB: c_int = 9;
const SIN_CHANGED: c_uint = 1;
const SIN_INSERT: c_uint = 2;
const SIN_UNDO: c_uint = 4;
const SIN_NOMARK: c_uint = 8;

impl Indenter {
    /// The indent this engine answers for the cursor line; negative when it
    /// has no answer and the line should be left alone.
    pub fn amount(self) -> c_int {
        match self {
            Indenter::Expr => get_expr_indent(),
            Indenter::Lisp => get_lisp_indent(),
            Indenter::C => crate::indent_c::get_c_indent(),
        }
    }
}

/// How many stops a tab-stop array names, or zero when it names none.
fn stop_count(vts: Option<&[ColNr]>) -> c_int {
    tabstops(vts).map_or(0, tabstop::TabStops::count)
}

/// A tab-stop array's first width, or the default of eight.
fn first_stop(vts: Option<&[ColNr]>) -> c_int {
    tabstops(vts).map_or(8, tabstop::TabStops::first)
}

/// The screen column byte `col` of line `lnum` sits at.
///
/// `getvcol` needs a `Pos` and three out-parameters to answer one
/// position, which is all any indent amount wants of it. Promoted out of
/// `indent_c.rs` at B15-17, where four functions ask it and two here do.
pub(crate) fn line_vcol(lnum: LineNr, col: ColNr) -> c_int {
    let mut fp = Pos {
        lnum,
        col,
        coladd: 0,
    };
    getvcol(Win::current(), &mut fp).0
}

/// A 'vartabstop' array as a list of stops. `None` when the option is unset
/// or names no stops, which is every caller's "use the uniform width" case.
///
/// The array is count-prefixed: `vts[0]` stops follow it.
fn tabstops(vts: Option<&[ColNr]>) -> Option<tabstop::TabStops<'_>> {
    vts.and_then(tabstop::TabStops::new)
}

/// A buffer's 'vartabstop' or 'varsofttabstop' array: its count first, the
/// widths after it; `None` for a value that names no stops.
pub type TabStopArray = Option<Box<[ColNr]>>;

/// Parse a 'vartabstop'-style option value into a fresh tab-stop array,
/// reporting the message the option code expects when it is malformed.
///
/// `None` for a value that does not parse; `Some(None)` for one that names
/// no stops.
pub fn tabstop_set(text: &[u8]) -> Option<TabStopArray> {
    match tabstop::parse(text) {
        Ok(parsed) => Some(parsed.map(Vec::into_boxed_slice)),
        Err(tabstop::ParseError::NotPositive(_)) => {
            emsg(gettext(e_positive));
            None
        }
        Err(tabstop::ParseError::Malformed(at) | tabstop::ParseError::OutOfRange(at)) => {
            let arg0 = msg_bytes(&text[at..]);
            semsg!("E475: Invalid argument: {arg0}");
            None
        }
    }
}

/// The tab-stop array a 'vartabstop'-style value describes, or `None` for a
/// value that names none -- or does not parse, which [`tabstop_set`] has
/// then reported.
pub(crate) fn tabstop_array_of(value: &CStr) -> TabStopArray {
    tabstop_set(value.to_bytes()).flatten()
}

/// Give `slot` the tab-stop array `value` describes. A value that does not
/// parse leaves it empty.
pub(crate) fn replace_tabstop_array(slot: &mut TabStopArray, value: Option<&CStr>) {
    *slot = value.and_then(tabstop_array_of);
}

/// Which of a buffer's two tab-stop lists.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopList {
    /// 'vartabstop', which 'tabstop' stands in for.
    Tab,
    /// 'varsofttabstop', which 'softtabstop' stands in for.
    SoftTab,
}

impl Buf {
    /// `f` of the buffer's `which` list, `None` when it names no stops.
    ///
    /// Private, and every `f` below is arithmetic on the list: nothing can
    /// set the option -- and free the array -- while it is borrowed.
    #[inline(always)]
    fn with_stops<R>(
        self,
        which: StopList,
        f: impl FnOnce(Option<tabstop::TabStops<'_>>) -> R,
    ) -> R {
        let vts = match which {
            StopList::Tab => self.b_p_vts_array.as_deref(),
            StopList::SoftTab => self.b_p_vsts_array.as_deref(),
        };
        f(tabstops(vts))
    }

    /// How many columns from `col` to this buffer's next tabstop, by its
    /// `'tabstop'` and `'vartabstop'`.
    #[inline]
    pub fn tab_padding(self, col: ColNr) -> ColNr {
        self.stop_padding(StopList::Tab, col, self.b_p_ts)
    }

    /// How many columns from `col` to the next stop of `which`, the uniform
    /// width `ts` standing in when the list names none.
    #[inline]
    pub(crate) fn stop_padding(self, which: StopList, col: ColNr, ts: OptInt) -> ColNr {
        self.with_stops(which, |stops| match stops {
            Some(stops) => stops.padding(col),
            None => tabstop::uniform_padding(col, ts),
        })
    }

    /// How many stops `which` names, or zero when it names none.
    pub(crate) fn stop_count(self, which: StopList) -> c_int {
        self.with_stops(which, |stops| stops.map_or(0, tabstop::TabStops::count))
    }

    /// The first 'vartabstop' width, or the default of eight.
    pub(crate) fn first_tab_stop(self) -> c_int {
        self.with_stops(StopList::Tab, |stops| {
            stops.map_or(8, tabstop::TabStops::first)
        })
    }

    /// The width of the 'vartabstop' stop at `col` ('tabstop' when there is
    /// no list); with `left`, of the one a cursor moving back would cross.
    pub(crate) fn tab_width_at(self, col: ColNr, left: bool) -> c_int {
        let ts = self.b_p_ts as c_int;
        self.with_stops(StopList::Tab, |stops| {
            stops.map_or(ts, |stops| stops.at(col, left))
        })
    }

    /// The column the stop of `which` containing `col` starts at, `ts`
    /// standing in when the list names none.
    pub(crate) fn stop_start(self, which: StopList, col: ColNr, ts: c_int) -> ColNr {
        self.with_stops(which, |stops| match stops {
            Some(stops) => stops.start(col),
            None => col - col % ts,
        })
    }

    /// The tabs and trailing spaces that fill the columns `start_col` to
    /// `end_col`, by this buffer's 'tabstop' and 'vartabstop'.
    pub(crate) fn tab_fill(self, start_col: ColNr, end_col: ColNr) -> (c_int, c_int) {
        let ts = self.b_p_ts as c_int;
        self.with_stops(StopList::Tab, |stops| match stops {
            Some(stops) => stops.from_to(start_col, end_col),
            None => tabstop::uniform_from_to(start_col, end_col, ts),
        })
    }

    /// A copy of this buffer's 'vartabstop' list, its count first; `None`
    /// when it names no stops.
    pub(crate) fn tab_stop_list(self) -> Option<Vec<ColNr>> {
        self.with_stops(StopList::Tab, |stops| {
            stops.map(|stops| {
                let mut list = Vec::with_capacity(stops.widths().len() + 1);
                list.push(stops.count());
                list.extend_from_slice(stops.widths());
                list
            })
        })
    }

    /// The screen width of the indent at `text`, by this buffer's
    /// 'tabstop' and 'vartabstop'.
    pub(crate) fn indent_width(self, text: &[u8]) -> c_int {
        let ts = self.b_p_ts;
        self.with_stops(StopList::Tab, |stops| {
            indent_width(text, stops.map(tabstop::TabStops::widths), ts)
        })
    }
}

/// `buffer`'s 'shiftwidth': the option, or the width of the tabstop at column
/// zero when the option is 0.
pub fn get_sw_value(buffer: Buf) -> c_int {
    get_sw_value_col(buffer, 0, false)
}

/// `buffer`'s 'shiftwidth' as seen from `pos`, which only differs from
/// [`get_sw_value`] under 'vartabstop'.
///
/// The cursor is moved to `pos` (a position in the current buffer) and
/// restored.
fn get_sw_value_pos(buffer: Buf, pos: Pos, left: bool) -> c_int {
    let save_cursor = Win::current().w_cursor;
    Win::current().w_cursor = pos;
    let sw_value = get_sw_value_col(buffer, get_nolist_virtcol(), left);
    Win::current().w_cursor = save_cursor;
    sw_value
}

/// `buffer`'s 'shiftwidth' as seen from the end of the current line's indent.
pub fn get_sw_value_indent(buffer: Buf, left: bool) -> c_int {
    let mut pos = Win::current().w_cursor;
    pos.col = getwhitecols_curline() as ColNr;
    get_sw_value_pos(buffer, pos, left)
}

/// `buffer`'s 'shiftwidth' at screen column `col`.
pub fn get_sw_value_col(buffer: Buf, col: ColNr, left: bool) -> c_int {
    if buffer.b_p_sw != 0 {
        buffer.b_p_sw as c_int
    } else {
        buffer.tab_width_at(col, left)
    }
}

/// The current buffer's 'softtabstop', with a negative value meaning
/// 'shiftwidth'.
pub fn get_sts_value() -> c_int {
    if Buf::current().b_p_sts < 0 {
        get_sw_value(Buf::current())
    } else {
        Buf::current().b_p_sts as c_int
    }
}

/// The screen width of the current line's indent.
///
/// Safe: the only promise is that the editor exists, which `Buf::current()`
/// and the cursor accessors carry.
pub fn get_indent() -> c_int {
    get_indent_lnum(Win::current().w_cursor.lnum)
}

/// The screen width of line `lnum`'s indent, in the current buffer.
///
/// Safe: an out-of-range `lnum` reads the placeholder, whose indent is zero.
pub fn get_indent_lnum(lnum: LineNr) -> c_int {
    get_indent_buf(Buf::current(), lnum)
}

/// The screen width of line `lnum`'s indent, in `buffer`.
///
/// Safe: as [`get_indent_lnum`]. The line is borrowed for the measurement
/// and dropped with it, which is the whole of what the cache asks.
pub fn get_indent_buf(buffer: Buf, lnum: LineNr) -> c_int {
    let mut lines = buffer.lines();
    buffer.indent_width(lines.line(lnum))
}

/// The screen width of the indent at `text`, with every tab a fixed
/// `byte2cells(TAB)` wide.
///
/// That is the shape 'breakindent' wants: it asks about a line it is not
/// going to change, so where the tabstops actually sit does not matter.
///
pub fn indent_size_no_ts(text: &[u8]) -> c_int {
    let tab_size = byte2cells(TAB);
    let mut vcol = 0;
    for &c in text {
        if c == b' ' {
            vcol += 1;
        } else if c_int::from(c) == TAB {
            vcol += tab_size;
        } else {
            break;
        }
    }
    vcol
}

/// The screen width of an indent read one byte at a time, under `stops`
/// ('vartabstop') or a uniform `ts` ('tabstop').
///
/// The 'vartabstop' arm walks the *stops* rather than the bytes: each one is
/// either filled by spaces -- in which case the walk continues into the next
/// stop -- or jumped by a tab. A byte that is neither ends the indent inside
/// a stop, and the answer is the column reached so far, which is why the
/// inner loop answers `cur_vcol` and not `vcol`. The last stop is left out of
/// that walk because it repeats forever, and so becomes the uniform width the
/// final loop runs on.
///
/// The bytes are read through [`byte_at`], so an index past the end answers
/// NUL and ends the indent — which is how a pointer into a NUL-terminated
/// line behaves, and what makes this the whole arithmetic with none of the
/// pointer.
fn indent_width(text: &[u8], stops: Option<&[ColNr]>, ts: OptInt) -> c_int {
    let mut at = 0;
    let mut next = || {
        let c = byte_at(text, at);
        at += 1;
        c
    };
    let mut vcol: c_int = 0;
    let tabstop_width: c_int;
    let mut next_tab_vcol: c_int;
    match stops {
        None => {
            tabstop_width = (if ts == 0 { 8 } else { ts }) as c_int;
            next_tab_vcol = tabstop_width;
        }
        Some(stops) => {
            let (&last, rest) = stops.split_last().expect("at least one stop");
            for &width in rest {
                let mut cur_vcol = vcol;
                vcol += width;
                debug_assert!(cur_vcol < vcol);
                loop {
                    let c = next();
                    if c == b' ' {
                        cur_vcol += 1;
                        if cur_vcol == vcol {
                            break;
                        }
                    } else if c_int::from(c) == TAB {
                        break;
                    } else {
                        return cur_vcol;
                    }
                }
            }
            tabstop_width = last;
            next_tab_vcol = vcol + tabstop_width;
        }
    }
    debug_assert!(tabstop_width != 0);
    loop {
        let c = next();
        if c == b' ' {
            vcol += 1;
            if vcol == next_tab_vcol {
                next_tab_vcol += tabstop_width;
            }
        } else if c_int::from(c) == TAB {
            vcol = next_tab_vcol;
            next_tab_vcol += tabstop_width;
        } else {
            return vcol;
        }
    }
}

/// The screen width of the indent at `text` under 'tabstop' `ts` and
/// 'vartabstop' `vts`, a count-prefixed array.
pub fn indent_size_ts(text: &[u8], ts: OptInt, vts: Option<&[ColNr]>) -> c_int {
    debug_assert!(char2cells(' ' as c_int) == 1);
    indent_width(text, tabstops(vts).map(tabstop::TabStops::widths), ts)
}

/// What [`set_indent`] is going to write, measured before anything is
/// allocated.
struct IndentPlan {
    /// The line's existing indent is not already the one asked for, so the
    /// line has to be rebuilt.
    doit: bool,
    /// Bytes the new indent will occupy.
    ind_len: c_int,
    /// Columns of the existing indent 'preserveindent' is keeping.
    ind_done: c_int,
    /// Bytes of the existing indent to copy verbatim, when 'preserveindent'
    /// *and* 'expandtab' are both set; -1 when they are not.
    orig_char_len: c_int,
    /// The offset of the first byte of the line the plan did not account for.
    rest: usize,
}

/// Measure the indent `set_indent(size, flags)` needs, without writing one.
///
/// Two answers in one walk: `ind_len` is how many bytes the new indent will
/// take, and `doit` whether any of them differs from what is already there --
/// an indent that is already right is left alone, which is what keeps
/// `set_indent` from marking every line changed.
///
/// `oldline` is the current line; reading past its end reads its NUL.
fn plan_indent(size: c_int, flags: c_int, oldline: &[u8]) -> IndentPlan {
    let buf = Buf::current();
    let preserve = flags & SIN_INSERT as c_int == 0 && buf.b_p_pi != 0;
    let pad = |col: c_int| buf.tab_padding(col as ColNr);
    let at = |i: usize| c_int::from(byte_at(oldline, i));
    let mut plan = IndentPlan {
        doit: false,
        ind_len: 0,
        ind_done: 0,
        orig_char_len: -1,
        rest: 0,
    };
    let mut todo = size;

    if buf.b_p_et == 0 || preserve {
        let mut ind_col = 0;
        if preserve {
            // Reuse as much of the existing indent's structure as fits.
            while todo > 0 && ascii_iswhite(at(plan.rest)) {
                if at(plan.rest) == TAB {
                    let tab_pad = pad(plan.ind_done);
                    // Stop if this tab would overshoot the target.
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    plan.ind_len += 1;
                    plan.ind_done += tab_pad;
                } else {
                    todo -= 1;
                    plan.ind_len += 1;
                    plan.ind_done += 1;
                }
                plan.rest += 1;
            }
            // The two counts diverge from here: `ind_col` goes on to
            // drive the tab fill, `ind_done` records what was kept.
            ind_col = plan.ind_done;
            // The initial run of characters to copy when the indent is
            // preserved under 'expandtab'.
            if buf.b_p_et != 0 {
                plan.orig_char_len = plan.ind_len;
            }
            // Fill to the next tabstop with a tab, if possible.
            let tab_pad = pad(plan.ind_done);
            if todo >= tab_pad && plan.orig_char_len == -1 {
                plan.doit = true;
                todo -= tab_pad;
                plan.ind_len += 1;
                ind_col += tab_pad;
            }
        }
        // Count the tabs the indent needs.
        loop {
            let tab_pad = pad(ind_col);
            if todo < tab_pad {
                break;
            }
            if at(plan.rest) != TAB {
                plan.doit = true;
            } else {
                plan.rest += 1;
            }
            todo -= tab_pad;
            plan.ind_len += 1;
            ind_col += tab_pad;
        }
    }
    // Count the spaces the indent needs.
    while todo > 0 {
        if at(plan.rest) != ' ' as c_int {
            plan.doit = true;
        } else {
            plan.rest += 1;
        }
        todo -= 1;
        plan.ind_len += 1;
    }
    plan
}

/// The line [`set_indent`] writes, and where its text starts in both lines.
struct Reindented {
    line: Vec<u8>,
    /// The new indent's length in bytes, which is where the cursor goes.
    ind_len: c_int,
    /// Bytes of the old indent kept in place, which extmarks must not move by.
    skipcols: ColNr,
    /// Where the kept text starts in the old line and in the new one.
    old_offset: ColNr,
    new_offset: ColNr,
}

/// Build the line `set_indent(size, flags)` writes over `oldline`, or `None`
/// when its indent is already right.
fn reindent_line(size: c_int, flags: c_int, oldline: &[u8]) -> Option<Reindented> {
    let buf = Buf::current();
    // The size of the line, including the NUL.
    let mut line_len = oldline.len() as c_int + 1;
    let pad = |col: c_int| buf.tab_padding(col as ColNr);
    let at = |i: usize| c_int::from(byte_at(oldline, i));
    // `STRICT_ADD`/`STRICT_SUB` (`macros.h`): the arithmetic sizing the
    // replacement line must not wrap, and upstream logs and aborts rather
    // than trusting a wrapped answer. `line` is the site in
    // `v0.12.4:indent.c`.
    let strict = |v: Option<c_int>, line: c_int, what: &CStr| -> c_int {
        v.unwrap_or_else(|| {
            let what = msg_cstr(what);
            logmsg!(LOGLVL_ERR, c"set_indent", line, "{what}");
            std::process::abort()
        })
    };

    let IndentPlan {
        doit,
        mut ind_len,
        mut ind_done,
        orig_char_len,
        rest,
    } = plan_indent(size, flags, oldline);
    let mut p = rest;

    // Return if the indent is OK already.
    if !doit && !ascii_iswhite(at(p)) && flags & SIN_INSERT as c_int == 0 {
        return None;
    }

    if flags & SIN_INSERT as c_int != 0 {
        p = 0;
    } else {
        p += skip::white(oldline.get(p..).unwrap_or_default());
        line_len -= p as c_int;
    }

    // Columns (in bytes) of the old indent that were preserved, and so
    // that an extmark inside them must not be moved by.
    let mut skipcols: ColNr = 0;
    // What is left to emit, in screen columns.
    let mut todo;
    let newline_size: usize;
    if orig_char_len != -1 {
        // 'preserveindent' and 'expandtab' both set: keep the original
        // characters and size for them plus the spaces that follow.
        let mut n = strict(orig_char_len.checked_add(size), 598, c"STRICT_ADD overflow");
        n = strict(n.checked_sub(ind_done), 599, c"STRICT_SUB overflow");
        n = strict(n.checked_add(line_len), 600, c"STRICT_ADD overflow");
        debug_assert!(n >= 0);
        newline_size = n as usize;
        todo = size - ind_done;
        // The indent's total length in characters, which was undercounted
        // until now.
        ind_len = orig_char_len + todo;
        skipcols = orig_char_len;
    } else {
        todo = size;
        debug_assert!(ind_len + line_len >= 0);
        let n = strict(ind_len.checked_add(line_len), 626, c"STRICT_ADD overflow");
        newline_size = n as usize;
    }
    // The terminator is `XString::from_vec`'s, inside the size counted.
    let mut out: Vec<u8> = Vec::with_capacity(newline_size);

    if orig_char_len != -1 {
        out.extend_from_slice(&oldline[..orig_char_len as usize]);
        p = orig_char_len as usize;
        // Skip any further white space, which is there when the new
        // indent is smaller than the old one.
        while ascii_iswhite(at(p)) {
            p += 1;
        }
    }

    // Put the characters in the new line; without 'expandtab', use tabs.
    if buf.b_p_et == 0 {
        if flags & SIN_INSERT as c_int == 0 && buf.b_p_pi != 0 {
            // Reuse as much of the existing indent's structure as fits.
            p = 0;
            ind_done = 0;
            while todo > 0 && ascii_iswhite(at(p)) {
                if at(p) == TAB {
                    let tab_pad = pad(ind_done);
                    // Stop if this tab would overshoot the target.
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    ind_done += tab_pad;
                } else {
                    todo -= 1;
                    ind_done += 1;
                }
                out.push(oldline[p]);
                p += 1;
                skipcols += 1;
            }
            // Fill to the next tabstop with a tab, if possible.
            let tab_pad = pad(ind_done);
            if todo >= tab_pad {
                out.push(b'\t');
                todo -= tab_pad;
                ind_done += tab_pad;
            }
            p += skip::white(oldline.get(p..).unwrap_or_default());
        }
        loop {
            let tab_pad = pad(ind_done);
            if todo < tab_pad {
                break;
            }
            out.push(b'\t');
            todo -= tab_pad;
            ind_done += tab_pad;
        }
    }
    let new_offset = (out.len() + todo.max(0) as usize) as ColNr;
    out.resize(new_offset as usize, b' ');
    out.extend_from_slice(oldline.get(p..).unwrap_or_default());
    Some(Reindented {
        line: out,
        ind_len,
        skipcols,
        old_offset: p as ColNr,
        new_offset,
    })
}

/// Set the current line's indent to `size` screen columns.
///
/// `flags` is a `SIN_*` set: `CHANGED` reports the change, `INSERT` says the
/// line is being typed (so the text after the indent is kept rather than
/// skipped), `UNDO` saves the line first, `NOMARK` leaves extmarks alone.
///
/// Answers whether the line was changed.
pub fn set_indent(size: c_int, flags: c_int) -> bool {
    let buf = Buf::current();
    let lnum = Win::current().w_cursor.lnum;
    // One read of the line: the new one is built from it before anything
    // writes the memline.
    let Some(Reindented {
        line,
        ind_len,
        skipcols,
        old_offset,
        new_offset,
    }) = reindent_line(size, flags, Lines::current().line(lnum))
    else {
        return false;
    };

    let mut retval = false;
    // Replace the line, unless undo fails.
    if flags & SIN_UNDO as c_int == 0 || u_savesub(lnum).is_ok() {
        let _ = ml_replace_owned(lnum, XString::from_vec(line));
        if flags & SIN_NOMARK as c_int == 0 {
            extmark_splice_cols(
                buf,
                lnum as c_int - 1,
                skipcols,
                old_offset - skipcols,
                new_offset - skipcols,
                kExtmarkUndo,
            );
        }
        if flags & SIN_CHANGED as c_int != 0 {
            changed_bytes(lnum, 0);
        }
        // Correct the saved cursor position if it is on this line.
        let saved = saved_cursor.get();
        if saved.lnum == lnum {
            if saved.col >= old_offset {
                // It was after the indent: shift it by the byte delta.
                saved_cursor.set(saved.with_col(saved.col + ind_len - old_offset));
            } else if saved.col >= new_offset {
                // It was inside the indent and is now past it (spaces
                // replaced by a tab): put it back at the indent's end.
                saved_cursor.set(saved.with_col(new_offset));
            }
        }
        retval = true;
    }
    Win::current().w_cursor.col = ind_len as ColNr;
    retval
}

/// The indent of line `lnum` *after* a 'formatlistpat' match, or -1 when the
/// line does not match.
///
/// This is 'formatoptions' `n`'s numbered-list indent. The pattern is
/// arbitrary, so it can name more than a number, and it is matched past any
/// comment leader -- which is the only reason `get_leader_len` is here.
pub fn get_number_indent(lnum: LineNr) -> c_int {
    if lnum > Buf::current().b_ml.ml_line_count {
        return -1;
    }
    let mut pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    // In `format_lines` -- that is, outside Insert mode -- 'formatoptions'
    // `q` is needed as well before a leader is stepped over.
    // The match re-enters the editor (a `\=` in 'formatlistpat' evaluates),
    // so the line is copied rather than borrowed from the memline.
    let line = Lines::current().line_copy(lnum);
    let mut lead_len = 0;
    if State.get() & MODE_INSERT != 0 || has_format_option(FoFlag::Q_COMS) {
        lead_len = get_leader_len(&line, false, true).0;
    }
    let flp = Buf::current().b_p_flp.get();
    let mut regmatch = RegMatch::new(vim_regcomp(flp.as_cstr(), RE_MAGIC), false);
    if regmatch.regprog.is_some() {
        regmatch.rm_ic = false;
        // The match starts past the comment leader, so its offsets are
        // relative to there and the leader goes back on at the end.
        let lead_len = lead_len as usize;
        if vim_regexec(&mut regmatch, cstr::suffix(line.as_cstr(), lead_len), 0) {
            pos.lnum = lnum;
            pos.col = (lead_len + regmatch.ends[0].unwrap_or(0)) as ColNr;
            pos.coladd = 0;
        }
    }
    // The match may have ended at the line's NUL, which is not an indent.
    if pos.lnum == 0 || byte_at(Lines::current().line(pos.lnum), pos.col as usize) == 0 {
        return -1;
    }
    line_vcol(pos.lnum, pos.col)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ask [`indent_width`] about a line, the way a NUL-terminated one is
    /// read: the bytes, then the terminator forever.
    fn width(line: &[u8], stops: Option<&[ColNr]>, ts: OptInt) -> c_int {
        indent_width(line, stops, ts)
    }

    #[test]
    fn uniform_tabstops() {
        assert_eq!(width(b"   x", None, 8), 3);
        assert_eq!(width(b"\tx", None, 8), 8);
        assert_eq!(width(b"  \tx", None, 8), 8);
        assert_eq!(width(b"\t\tx", None, 4), 8);
        // A tab lands on the next stop even when spaces already crossed one.
        assert_eq!(width(b"        \tx", None, 8), 16);
        // 'tabstop' 0 is the built-in default of eight.
        assert_eq!(width(b"\tx", None, 0), 8);
    }

    #[test]
    fn indent_is_the_whole_line() {
        // No terminator is needed: past the end reads NUL, which ends it.
        assert_eq!(width(b"    ", None, 8), 4);
        assert_eq!(width(b"", None, 8), 0);
    }

    #[test]
    fn vartabstop_walks_the_stops() {
        let stops: &[ColNr] = &[3, 5, 7];
        // A tab jumps to the end of the stop it is inside.
        assert_eq!(width(b"\tx", Some(stops), 8), 3);
        assert_eq!(width(b"\t\tx", Some(stops), 8), 8);
        assert_eq!(width(b"\t\t\tx", Some(stops), 8), 15);
        // The last stop repeats forever.
        assert_eq!(width(b"\t\t\t\tx", Some(stops), 8), 22);
        // Spaces fill a stop and the walk continues into the next one.
        assert_eq!(width(b"   \tx", Some(stops), 8), 8);
    }

    #[test]
    fn a_stop_left_part_filled_answers_the_column_reached() {
        // The inner loop's `cur_vcol`: three stops of 3, 5, 7, and the indent
        // ends four columns in -- inside the second stop, not at its end.
        let stops: &[ColNr] = &[3, 5, 7];
        assert_eq!(width(b"    x", Some(stops), 8), 4);
        assert_eq!(width(b"x", Some(stops), 8), 0);
    }

    #[test]
    fn one_vartabstop_is_a_uniform_width() {
        // The single stop is the "last" one, so nothing is walked and it
        // becomes the repeating width.
        let stops: &[ColNr] = &[4];
        assert_eq!(width(b"\tx", Some(stops), 8), 4);
        assert_eq!(width(b"\t\tx", Some(stops), 8), 8);
        assert_eq!(width(b"  \tx", Some(stops), 8), 4);
    }
}
