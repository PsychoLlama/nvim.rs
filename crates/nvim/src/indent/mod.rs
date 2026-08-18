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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

use crate::ascii::ascii_iswhite;
use crate::change::{changed_bytes, get_leader_len};
use crate::charset::{byte2cells, char2cells, getwhitecols_curline, skipwhite};
use crate::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::edit::get_nolist_virtcol;
use crate::extmark::extmark_splice_cols;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::main::{State, curbuf, curwin, e_invarg2, e_positive, saved_cursor};
use crate::memline::{ml_get, ml_get_buf, ml_get_pos, ml_replace};
use crate::memory::{xfree, xmalloc};
use crate::message::emsg;
use crate::options::kOptDyFlagUhex;
use crate::os::cshim::gettext;
use crate::plines::getvcol;
use crate::textformat::has_format_option;
use crate::types::*;
use ::libc::abort;

// `regexp.rs` keeps its own copy of `regprog_T`, so these stay declarations
// rather than imports. `breakindent.rs` reaches them through `use super::*`.
use crate::undo::u_savesub;

pub mod breakindent;
pub mod edit;
pub mod expr;
pub mod tabstop;

// Split out for size. The names below are what the rest of the tree calls,
// and it calls them as `indent::*`.
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::state::MODE_INSERT;
pub use breakindent::{briopt_check, get_breakindent_win};
pub use edit::{
    change_indent, copy_indent, ex_retab, inindent, ins_try_si, may_do_si, op_reindent,
    preprocs_left,
};
pub use expr::{
    f_indent, f_lispindent, fix_indent, fixthisline, get_expr_indent, get_lisp_indent,
    use_indentexpr_for_lisp,
};

const BL_WHITE: c_int = 1;
const BL_SOL: c_int = 2;
const BL_FIX: c_int = 4;
const INDENT_SET: c_uint = 1;
const INDENT_DEC: c_uint = 3;
const kBufOptIndentexpr: c_uint = 47;
const kExtmarkUndo: ExtmarkOp = 1;
const kOptIndentexpr: OptIndex = 148;
const kOptValTypeString: OptValType = 2;
const kOptVartabstop: OptIndex = 338;
const NUL: c_int = 0;
const TAB: c_int = 9;
const OPT_LOCAL: c_int = 2;
const SIN_CHANGED: c_uint = 1;
const SIN_INSERT: c_uint = 2;
const SIN_UNDO: c_uint = 4;
const SIN_NOMARK: c_uint = 8;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const FO_Q_COMS: c_int = 'q' as c_int;
pub const IOSIZE: c_int = 1024 + 1;

/// The screen column byte `col` of line `lnum` sits at.
///
/// `getvcol` needs a `pos_T` and three out-parameters to answer one
/// position, which is all any indent amount wants of it. Promoted out of
/// `indent_c.rs` at B15-17, where four functions ask it and two here do.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn line_vcol(lnum: linenr_T, col: colnr_T) -> c_int {
    unsafe {
        let mut fp = pos_T {
            lnum,
            col,
            coladd: 0,
        };
        let mut vcol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut fp,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        vcol
    }
}

/// The byte at `i` of a NUL-terminated line held as a slice.
///
/// Past the end is the terminator: `CStr::to_bytes()` drops it, but the
/// memory is still there, and upstream reaches these strings through the NUL
/// rather than through a length.
pub(crate) fn byte_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// Borrow a 'vartabstop' array as a slice. `None` when the option is unset
/// or names no stops, which is every caller's "use the uniform width" case.
///
/// # Safety
/// `vts`, if not null, must point at a count-prefixed array of that length.
unsafe fn tabstops<'a>(vts: *const colnr_T) -> Option<tabstop::TabStops<'a>> {
    if vts.is_null() {
        return None;
    }
    unsafe {
        let count = *vts;
        tabstop::TabStops::new(::core::slice::from_raw_parts(vts, count as usize + 1))
    }
}

/// Parse a 'vartabstop'-style option value into `array`, reporting the
/// message the option code expects when it is malformed.
///
/// # Safety
/// `var` must be NUL-terminated and `array` must own its current value.
pub unsafe fn tabstop_set(var: *mut c_char, array: *mut *mut colnr_T) -> bool {
    unsafe {
        let text = CStr::from_ptr(var).to_bytes();
        let parsed = match tabstop::parse(text) {
            Ok(parsed) => parsed,
            Err(tabstop::ParseError::NotPositive(_)) => {
                emsg(gettext(&raw const e_positive as *const c_char));
                return false;
            }
            Err(tabstop::ParseError::Malformed(at) | tabstop::ParseError::OutOfRange(at)) => {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), var.add(at));
                return false;
            }
        };
        // The option owns a malloc'd array, so hand one over rather than a Vec.
        *array = match parsed {
            None => ::core::ptr::null_mut(),
            Some(stops) => {
                let out = xmalloc(size_of::<colnr_T>() * stops.len()) as *mut colnr_T;
                ::core::ptr::copy_nonoverlapping(stops.as_ptr(), out, stops.len());
                out
            }
        };
        true
    }
}

/// How many columns from `col` to the next tabstop.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_padding(col: colnr_T, ts: OptInt, vts: *const colnr_T) -> c_int {
    match unsafe { tabstops(vts) } {
        Some(stops) => stops.padding(col),
        None => tabstop::uniform_padding(col, ts),
    }
}

/// The width of the tabstop at `col`; with `left`, of the one a cursor
/// moving back would cross.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_at(col: colnr_T, ts: OptInt, vts: *const colnr_T, left: bool) -> c_int {
    match unsafe { tabstops(vts) } {
        Some(stops) => stops.at(col, left),
        None => ts as c_int,
    }
}

/// The column the tabstop containing `col` starts at.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_start(col: colnr_T, ts: c_int, vts: *mut colnr_T) -> colnr_T {
    match unsafe { tabstops(vts) } {
        Some(stops) => stops.start(col),
        None => col - col % ts,
    }
}

/// The tabs and trailing spaces that fill the columns `start_col` to
/// `end_col`. A zero `ts_arg` means the current buffer's 'tabstop'.
///
/// # Safety
/// `vts` must be a valid tabstop array or null; the out-pointers must be
/// writable.
pub unsafe fn tabstop_fromto(
    start_col: colnr_T,
    end_col: colnr_T,
    ts_arg: c_int,
    vts: *const colnr_T,
    ntabs: *mut c_int,
    nspcs: *mut c_int,
) {
    unsafe {
        let ts = if ts_arg == 0 {
            (*curbuf.get()).b_p_ts as c_int
        } else {
            ts_arg
        };
        debug_assert!(ts != 0);
        let (tabs, spaces) = match tabstops(vts) {
            Some(stops) => stops.from_to(start_col, end_col),
            None => tabstop::uniform_from_to(start_col, end_col, ts),
        };
        *ntabs = tabs;
        *nspcs = spaces;
    }
}

/// Whether two 'vartabstop' arrays name the same stops.
///
/// # Safety
/// Both must be valid tabstop arrays or null.
unsafe fn tabstop_eq(ts1: *const colnr_T, ts2: *const colnr_T) -> bool {
    unsafe {
        let borrow = |ts: *const colnr_T| {
            (!ts.is_null()).then(|| ::core::slice::from_raw_parts(ts, *ts as usize + 1))
        };
        tabstop::eq(borrow(ts1), borrow(ts2))
    }
}

/// How many stops `ts` names, or zero when it names none.
///
/// # Safety
/// `ts` must be a valid tabstop array or null.
pub unsafe fn tabstop_count(ts: *mut colnr_T) -> c_int {
    if ts.is_null() { 0 } else { unsafe { *ts } }
}

/// The first stop's width, or the default of eight.
///
/// # Safety
/// `ts` must be a valid tabstop array or null.
pub unsafe fn tabstop_first(ts: *mut colnr_T) -> c_int {
    if ts.is_null() {
        8
    } else {
        unsafe { *ts.offset(1) }
    }
}

/// `buf`'s 'shiftwidth': the option, or the width of the tabstop at column
/// zero when the option is 0.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn get_sw_value(buf: *mut buf_T) -> c_int {
    unsafe { get_sw_value_col(buf, 0, false) }
}

/// `buf`'s 'shiftwidth' as seen from `pos`, which only differs from
/// [`get_sw_value`] under 'vartabstop'.
///
/// # Safety
/// `buf` must be a live buffer and `pos` a position in the current one: the
/// cursor is moved there and restored.
unsafe fn get_sw_value_pos(buf: *mut buf_T, pos: *mut pos_T, left: bool) -> c_int {
    unsafe {
        let save_cursor = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = *pos;
        let sw_value = get_sw_value_col(buf, get_nolist_virtcol(), left);
        (*curwin.get()).w_cursor = save_cursor;
        sw_value
    }
}

/// `buf`'s 'shiftwidth' as seen from the end of the current line's indent.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn get_sw_value_indent(buf: *mut buf_T, left: bool) -> c_int {
    unsafe {
        let mut pos = (*curwin.get()).w_cursor;
        pos.col = getwhitecols_curline() as colnr_T;
        get_sw_value_pos(buf, &raw mut pos, left)
    }
}

/// `buf`'s 'shiftwidth' at screen column `col`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn get_sw_value_col(buf: *mut buf_T, col: colnr_T, left: bool) -> c_int {
    unsafe {
        if (*buf).b_p_sw != 0 {
            (*buf).b_p_sw as c_int
        } else {
            tabstop_at(col, (*buf).b_p_ts, (*buf).b_p_vts_array, left)
        }
    }
}

/// The current buffer's 'softtabstop', with a negative value meaning
/// 'shiftwidth'.
///
/// # Safety
/// There must be a current buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_sts_value() -> c_int {
    unsafe {
        if (*curbuf.get()).b_p_sts < 0 {
            get_sw_value(curbuf.get())
        } else {
            (*curbuf.get()).b_p_sts as c_int
        }
    }
}

/// The screen width of the current line's indent.
///
/// # Safety
/// There must be a current line.
pub unsafe fn get_indent() -> c_int {
    unsafe {
        indent_size_ts(
            get_cursor_line_ptr(),
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
        )
    }
}

/// The screen width of line `lnum`'s indent, in the current buffer.
///
/// # Safety
/// `lnum` must be a valid line.
pub unsafe fn get_indent_lnum(lnum: linenr_T) -> c_int {
    unsafe {
        indent_size_ts(
            ml_get(lnum),
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
        )
    }
}

/// The screen width of line `lnum`'s indent, in `buf`.
///
/// # Safety
/// `lnum` must be a valid line of `buf`.
pub unsafe fn get_indent_buf(buf: *mut buf_T, lnum: linenr_T) -> c_int {
    unsafe { indent_size_ts(ml_get_buf(buf, lnum), (*buf).b_p_ts, (*buf).b_p_vts_array) }
}

/// The screen width of the indent at `ptr`, with every tab a fixed
/// `byte2cells(TAB)` wide.
///
/// That is the shape 'breakindent' wants: it asks about a line it is not
/// going to change, so where the tabstops actually sit does not matter.
///
/// # Safety
/// `ptr` must point at a NUL-terminated string.
pub unsafe fn indent_size_no_ts(ptr: *const c_char) -> c_int {
    unsafe {
        let tab_size = byte2cells(TAB);
        let mut vcol = 0;
        let mut ptr = ptr;
        loop {
            let c = *ptr as u8;
            ptr = ptr.add(1);
            if c == b' ' {
                vcol += 1;
            } else if c_int::from(c) == TAB {
                vcol += tab_size;
            } else {
                return vcol;
            }
        }
    }
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
/// `next` answers the indent's bytes and then NUL forever, which is how a
/// pointer into a NUL-terminated line behaves and what makes this the whole
/// arithmetic with none of the pointer.
fn indent_width(mut next: impl FnMut() -> u8, stops: Option<&[colnr_T]>, ts: OptInt) -> c_int {
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

/// The screen width of the indent at `ptr` under 'tabstop' `ts` and
/// 'vartabstop' `vts`.
///
/// # Safety
/// `ptr` must point at a NUL-terminated string; `vts` must be a valid
/// tabstop array or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn indent_size_ts(
    ptr: *const c_char,
    ts: OptInt,
    vts: *mut colnr_T,
) -> c_int {
    unsafe {
        debug_assert!(char2cells(' ' as c_int) == 1);
        // `vts[0]` is the count and `vts[1..=count]` the widths.
        let stops = (!vts.is_null() && *vts >= 1)
            .then(|| ::core::slice::from_raw_parts(vts.add(1), *vts as usize));
        let mut ptr = ptr;
        indent_width(
            || {
                let c = *ptr as u8;
                ptr = ptr.add(1);
                c
            },
            stops,
            ts,
        )
    }
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
    /// The first byte of the line the plan did not account for.
    rest: *mut c_char,
}

/// Measure the indent `set_indent(size, flags)` needs, without writing one.
///
/// Two answers in one walk: `ind_len` is how many bytes the new indent will
/// take, and `doit` whether any of them differs from what is already there --
/// an indent that is already right is left alone, which is what keeps
/// `set_indent` from marking every line changed.
///
/// # Safety
/// `oldline` must be the current line, NUL-terminated.
unsafe fn plan_indent(size: c_int, flags: c_int, oldline: *mut c_char) -> IndentPlan {
    unsafe {
        let buf = curbuf.get();
        let preserve = flags & SIN_INSERT as c_int == 0 && (*buf).b_p_pi != 0;
        let pad = |col: c_int| tabstop_padding(col as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array);
        let mut plan = IndentPlan {
            doit: false,
            ind_len: 0,
            ind_done: 0,
            orig_char_len: -1,
            rest: oldline,
        };
        let mut todo = size;

        if (*buf).b_p_et == 0 || preserve {
            let mut ind_col = 0;
            if preserve {
                // Reuse as much of the existing indent's structure as fits.
                while todo > 0 && ascii_iswhite(*plan.rest as c_int) {
                    if *plan.rest as c_int == TAB {
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
                    plan.rest = plan.rest.add(1);
                }
                // The two counts diverge from here: `ind_col` goes on to
                // drive the tab fill, `ind_done` records what was kept.
                ind_col = plan.ind_done;
                // The initial run of characters to copy when the indent is
                // preserved under 'expandtab'.
                if (*buf).b_p_et != 0 {
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
                if *plan.rest as c_int != TAB {
                    plan.doit = true;
                } else {
                    plan.rest = plan.rest.add(1);
                }
                todo -= tab_pad;
                plan.ind_len += 1;
                ind_col += tab_pad;
            }
        }
        // Count the spaces the indent needs.
        while todo > 0 {
            if *plan.rest as c_int != ' ' as c_int {
                plan.doit = true;
            } else {
                plan.rest = plan.rest.add(1);
            }
            todo -= 1;
            plan.ind_len += 1;
        }
        plan
    }
}

/// Set the current line's indent to `size` screen columns.
///
/// `flags` is a `SIN_*` set: `CHANGED` reports the change, `INSERT` says the
/// line is being typed (so the text after the indent is kept rather than
/// skipped), `UNDO` saves the line first, `NOMARK` leaves extmarks alone.
///
/// Answers whether the line was changed.
///
/// # Safety
/// There must be a current line, and it must be modifiable.
pub unsafe fn set_indent(size: c_int, flags: c_int) -> bool {
    unsafe {
        let buf = curbuf.get();
        let oldline = get_cursor_line_ptr();
        // The size of the line, including the NUL.
        let mut line_len = get_cursor_line_len() + 1;
        let pad = |col: c_int| tabstop_padding(col as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array);
        // `STRICT_ADD`/`STRICT_SUB` (`macros.h`): the arithmetic sizing the
        // replacement line must not wrap, and upstream logs and aborts rather
        // than trusting a wrapped answer. `line` is the site in
        // `v0.12.4:indent.c`. A closure, because one written inside an
        // `unsafe` block inherits it and so costs the ratchet nothing.
        let strict = |v: Option<c_int>, line: c_int, what: &CStr| -> c_int {
            v.unwrap_or_else(|| {
                logmsg_c!(
                    LOGLVL_ERR,
                    ::core::ptr::null::<c_char>(),
                    c"set_indent".as_ptr(),
                    line,
                    true,
                    what.as_ptr(),
                );
                abort()
            })
        };

        let IndentPlan {
            doit,
            mut ind_len,
            mut ind_done,
            mut orig_char_len,
            rest,
        } = plan_indent(size, flags, oldline);
        let mut p = rest;

        // Return if the indent is OK already.
        if !doit && !ascii_iswhite(*p as c_int) && flags & SIN_INSERT as c_int == 0 {
            return false;
        }

        if flags & SIN_INSERT as c_int != 0 {
            p = oldline;
        } else {
            p = skipwhite(p);
            line_len -= p.offset_from(oldline) as c_int;
        }

        // Columns (in bytes) of the old indent that were preserved, and so
        // that an extmark inside them must not be moved by.
        let mut skipcols: colnr_T = 0;
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
        let newline = xmalloc(newline_size as size_t) as *mut c_char;
        // The rest of the build is ordinary indexing over the allocation.
        let out = ::core::slice::from_raw_parts_mut(newline as *mut u8, newline_size);
        let mut n = 0usize;

        if orig_char_len != -1 {
            p = oldline;
            while orig_char_len > 0 {
                out[n] = *p as u8;
                n += 1;
                p = p.add(1);
                orig_char_len -= 1;
            }
            // Skip any further white space, which is there when the new
            // indent is smaller than the old one.
            while ascii_iswhite(*p as c_int) {
                p = p.add(1);
            }
        }

        // Put the characters in the new line; without 'expandtab', use tabs.
        if (*buf).b_p_et == 0 {
            if flags & SIN_INSERT as c_int == 0 && (*buf).b_p_pi != 0 {
                // Reuse as much of the existing indent's structure as fits.
                p = oldline;
                ind_done = 0;
                while todo > 0 && ascii_iswhite(*p as c_int) {
                    if *p as c_int == TAB {
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
                    out[n] = *p as u8;
                    n += 1;
                    p = p.add(1);
                    skipcols += 1;
                }
                // Fill to the next tabstop with a tab, if possible.
                let tab_pad = pad(ind_done);
                if todo >= tab_pad {
                    out[n] = b'\t';
                    n += 1;
                    todo -= tab_pad;
                    ind_done += tab_pad;
                }
                p = skipwhite(p);
            }
            loop {
                let tab_pad = pad(ind_done);
                if todo < tab_pad {
                    break;
                }
                out[n] = b'\t';
                n += 1;
                todo -= tab_pad;
                ind_done += tab_pad;
            }
        }
        while todo > 0 {
            out[n] = b' ';
            n += 1;
            todo -= 1;
        }
        out[n..n + line_len as usize].copy_from_slice(::core::slice::from_raw_parts(
            p as *const u8,
            line_len as usize,
        ));

        let old_offset = p.offset_from(oldline) as colnr_T;
        let new_offset = n as colnr_T;
        let mut retval = false;
        // Replace the line, unless undo fails.
        if flags & SIN_UNDO as c_int == 0 || u_savesub((*curwin.get()).w_cursor.lnum) == OK {
            // This may free `newline`.
            ml_replace((*curwin.get()).w_cursor.lnum, newline, false);
            if flags & SIN_NOMARK as c_int == 0 {
                extmark_splice_cols(
                    buf,
                    (*curwin.get()).w_cursor.lnum as c_int - 1,
                    skipcols,
                    old_offset - skipcols,
                    new_offset - skipcols,
                    kExtmarkUndo,
                );
            }
            if flags & SIN_CHANGED as c_int != 0 {
                changed_bytes((*curwin.get()).w_cursor.lnum, 0);
            }
            // Correct the saved cursor position if it is on this line.
            let saved = saved_cursor.ptr();
            if (*saved).lnum == (*curwin.get()).w_cursor.lnum {
                if (*saved).col >= old_offset {
                    // It was after the indent: shift it by the byte delta.
                    (*saved).col += ind_len - old_offset;
                } else if (*saved).col >= new_offset {
                    // It was inside the indent and is now past it (spaces
                    // replaced by a tab): put it back at the indent's end.
                    (*saved).col = new_offset;
                }
            }
            retval = true;
        } else {
            xfree(newline as *mut c_void);
        }
        (*curwin.get()).w_cursor.col = ind_len as colnr_T;
        retval
    }
}

/// The indent of line `lnum` *after* a 'formatlistpat' match, or -1 when the
/// line does not match.
///
/// This is 'formatoptions' `n`'s numbered-list indent. The pattern is
/// arbitrary, so it can name more than a number, and it is matched past any
/// comment leader -- which is the only reason `get_leader_len` is here.
///
/// # Safety
/// There must be a current buffer and window.
pub unsafe fn get_number_indent(lnum: linenr_T) -> c_int {
    unsafe {
        if lnum > (*curbuf.get()).b_ml.ml_line_count {
            return -1;
        }
        let mut pos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        // In `format_lines` -- that is, outside Insert mode -- 'formatoptions'
        // `q` is needed as well before a leader is stepped over.
        let mut lead_len = 0;
        if State.get() & MODE_INSERT != 0 || has_format_option(FO_Q_COMS) {
            lead_len = get_leader_len(
                ml_get(lnum),
                ::core::ptr::null_mut::<*mut c_char>(),
                false,
                true,
            );
        }
        let mut regmatch = regmatch_T {
            regprog: vim_regcomp((*curbuf.get()).b_p_flp, RE_MAGIC),
            startp: [::core::ptr::null_mut::<c_char>(); 10],
            endp: [::core::ptr::null_mut::<c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = false;
            // `vim_regexec` wants a pointer to a line, which is what lets the
            // match start past the comment leader.
            if vim_regexec(&raw mut regmatch, ml_get(lnum).offset(lead_len as isize), 0) {
                pos.lnum = lnum;
                pos.col = regmatch.endp[0].offset_from(ml_get(lnum)) as colnr_T;
                pos.coladd = 0;
            }
            vim_regfree(regmatch.regprog);
        }
        if pos.lnum == 0 || *ml_get_pos(&raw mut pos) as c_int == NUL {
            return -1;
        }
        line_vcol(pos.lnum, pos.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ask [`indent_width`] about a line, the way a NUL-terminated one is
    /// read: the bytes, then the terminator forever.
    fn width(line: &[u8], stops: Option<&[colnr_T]>, ts: OptInt) -> c_int {
        let mut i = 0;
        indent_width(
            || {
                let c = byte_at(line, i);
                i += 1;
                c
            },
            stops,
            ts,
        )
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
        let stops: &[colnr_T] = &[3, 5, 7];
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
        let stops: &[colnr_T] = &[3, 5, 7];
        assert_eq!(width(b"    x", Some(stops), 8), 4);
        assert_eq!(width(b"x", Some(stops), 8), 0);
    }

    #[test]
    fn one_vartabstop_is_a_uniform_width() {
        // The single stop is the "last" one, so nothing is walked and it
        // becomes the repeating width.
        let stops: &[colnr_T] = &[4];
        assert_eq!(width(b"\tx", Some(stops), 8), 4);
        assert_eq!(width(b"\t\tx", Some(stops), 8), 8);
        assert_eq!(width(b"  \tx", Some(stops), 8), 4);
    }
}
