use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::change::{changed_bytes, get_leader_len};
use crate::src::nvim::charset::{byte2cells, char2cells, getwhitecols_curline, skipwhite};
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::edit::get_nolist_virtcol;
use crate::src::nvim::extmark::extmark_splice_cols;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{State, curbuf, curwin, e_invarg2, e_positive, saved_cursor};
use crate::src::nvim::memline::{ml_get, ml_get_buf, ml_get_pos, ml_replace};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{abort, gettext, memmove};
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::textformat::has_format_option;
use crate::src::nvim::types::*;

// `regexp.rs` keeps its own copy of `regprog_T`, so these stay declarations
// rather than imports. `breakindent.rs` reaches them through `use super::*`.
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
use crate::src::nvim::undo::u_savesub;

pub mod breakindent;
pub mod edit;
pub mod expr;
pub mod tabstop;

// Split out for size. The names below are what the rest of the tree calls,
// and it calls them as `indent::*`.
pub use breakindent::{briopt_check, get_breakindent_win};
pub use edit::{
    change_indent, copy_indent, ex_retab, inindent, ins_try_si, may_do_si, op_reindent,
    preprocs_left,
};
pub use expr::{
    f_indent, f_lispindent, fix_indent, fixthisline, get_expr_indent, get_lisp_indent,
    use_indentexpr_for_lisp,
};

// The handful of enum constants this module reads. c2rust gave every
// translation unit a copy of all 1,545 names it could see; these are the ones
// the code below names.
type C2Rust_Unnamed_15 = MarkTreeIter_s;
const BL_WHITE: ::core::ffi::c_int = 1;
const BL_SOL: ::core::ffi::c_int = 2;
const BL_FIX: ::core::ffi::c_int = 4;
const CMOD_LOCKMARKS: ::core::ffi::c_uint = 2048;
const INDENT_SET: ::core::ffi::c_uint = 1;
const INDENT_INC: ::core::ffi::c_uint = 2;
const INDENT_DEC: ::core::ffi::c_uint = 3;
const kBufOptIndentexpr: ::core::ffi::c_uint = 47;
const kExtmarkUndo: ExtmarkOp = 1;
const kOptDyFlagUhex: ::core::ffi::c_uint = 4;
const kOptIndentexpr: OptIndex = 148;
const kOptValTypeString: OptValType = 2;
const kOptVartabstop: OptIndex = 338;
const MAXCOL: ::core::ffi::c_int = 2147483647;
const MODE_INSERT: ::core::ffi::c_int = 16;
const REPLACE_FLAG: ::core::ffi::c_int = 256;
const VREPLACE_FLAG: ::core::ffi::c_int = 512;
const NUL: ::core::ffi::c_int = 0;
const TAB: ::core::ffi::c_int = 9;
const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
const OPT_LOCAL: ::core::ffi::c_int = 2;
const SIN_CHANGED: ::core::ffi::c_uint = 1;
const SIN_INSERT: ::core::ffi::c_uint = 2;
const SIN_UNDO: ::core::ffi::c_uint = 4;
const SIN_NOMARK: ::core::ffi::c_uint = 8;
const UPD_INVERTED: ::core::ffi::c_int = 20;
const UPD_NOT_VALID: ::core::ffi::c_int = 40;
const VV_LNUM: VimVarIndex = 9;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const FO_Q_COMS: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;

/// Borrow a 'vartabstop' array as a slice. `None` when the option is unset
/// or names no stops, which is every caller's "use the uniform width" case.
///
/// # Safety
/// `vts`, if not null, must point at a count-prefixed array of that length.
unsafe fn tabstops<'a>(vts: *const colnr_T) -> Option<tabstop::TabStops<'a>> {
    if vts.is_null() {
        return None;
    }
    let count = *vts;
    tabstop::TabStops::new(::core::slice::from_raw_parts(vts, count as usize + 1))
}

/// Parse a 'vartabstop'-style option value into `array`, reporting the
/// message the option code expects when it is malformed.
///
/// # Safety
/// `var` must be NUL-terminated and `array` must own its current value.
pub unsafe fn tabstop_set(var: *mut ::core::ffi::c_char, array: *mut *mut colnr_T) -> bool {
    let text = ::core::ffi::CStr::from_ptr(var).to_bytes();
    let parsed = match tabstop::parse(text) {
        Ok(parsed) => parsed,
        Err(tabstop::ParseError::NotPositive(_)) => {
            emsg(gettext(&raw const e_positive as *const ::core::ffi::c_char));
            return false;
        }
        Err(tabstop::ParseError::Malformed(at) | tabstop::ParseError::OutOfRange(at)) => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                var.add(at),
            );
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

/// How many columns from `col` to the next tabstop.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_padding(col: colnr_T, ts: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int {
    match tabstops(vts) {
        Some(stops) => stops.padding(col),
        None => tabstop::uniform_padding(col, ts),
    }
}

/// The width of the tabstop at `col`; with `left`, of the one a cursor
/// moving back would cross.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_at(
    col: colnr_T,
    ts: OptInt,
    vts: *const colnr_T,
    left: bool,
) -> ::core::ffi::c_int {
    match tabstops(vts) {
        Some(stops) => stops.at(col, left),
        None => ts as ::core::ffi::c_int,
    }
}

/// The column the tabstop containing `col` starts at.
///
/// # Safety
/// `vts` must be a valid tabstop array or null.
pub unsafe fn tabstop_start(col: colnr_T, ts: ::core::ffi::c_int, vts: *mut colnr_T) -> colnr_T {
    match tabstops(vts) {
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
    ts_arg: ::core::ffi::c_int,
    vts: *const colnr_T,
    ntabs: *mut ::core::ffi::c_int,
    nspcs: *mut ::core::ffi::c_int,
) {
    let ts = if ts_arg == 0 {
        (*curbuf.get()).b_p_ts as ::core::ffi::c_int
    } else {
        ts_arg
    };
    assert!(ts != 0);
    let (tabs, spaces) = match tabstops(vts) {
        Some(stops) => stops.from_to(start_col, end_col),
        None => tabstop::uniform_from_to(start_col, end_col, ts),
    };
    *ntabs = tabs;
    *nspcs = spaces;
}

/// Whether two 'vartabstop' arrays name the same stops.
///
/// # Safety
/// Both must be valid tabstop arrays or null.
unsafe fn tabstop_eq(ts1: *const colnr_T, ts2: *const colnr_T) -> bool {
    let borrow = |ts: *const colnr_T| {
        (!ts.is_null()).then(|| ::core::slice::from_raw_parts(ts, *ts as usize + 1))
    };
    tabstop::eq(borrow(ts1), borrow(ts2))
}

/// How many stops `ts` names, or zero when it names none.
///
/// # Safety
/// `ts` must be a valid tabstop array or null.
pub unsafe fn tabstop_count(ts: *mut colnr_T) -> ::core::ffi::c_int {
    if ts.is_null() { 0 } else { *ts }
}

/// The first stop's width, or the default of eight.
///
/// # Safety
/// `ts` must be a valid tabstop array or null.
pub unsafe fn tabstop_first(ts: *mut colnr_T) -> ::core::ffi::c_int {
    if ts.is_null() { 8 } else { *ts.offset(1) }
}

pub unsafe extern "C" fn get_sw_value(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = get_sw_value_col(buf, 0 as colnr_T, false);
    return result;
}
unsafe extern "C" fn get_sw_value_pos(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut left: bool,
) -> ::core::ffi::c_int {
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor = *pos;
    let mut sw_value: ::core::ffi::c_int = get_sw_value_col(buf, get_nolist_virtcol(), left);
    (*curwin.get()).w_cursor = save_cursor;
    return sw_value;
}
pub unsafe extern "C" fn get_sw_value_indent(
    mut buf: *mut buf_T,
    mut left: bool,
) -> ::core::ffi::c_int {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    pos.col = getwhitecols_curline() as colnr_T;
    return get_sw_value_pos(buf, &raw mut pos, left);
}
pub unsafe extern "C" fn get_sw_value_col(
    mut buf: *mut buf_T,
    mut col: colnr_T,
    mut left: bool,
) -> ::core::ffi::c_int {
    return if (*buf).b_p_sw != 0 {
        (*buf).b_p_sw as ::core::ffi::c_int
    } else {
        tabstop_at(col, (*buf).b_p_ts, (*buf).b_p_vts_array, left)
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_sts_value() -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = if (*curbuf.get()).b_p_sts < 0 as OptInt {
        get_sw_value(curbuf.get())
    } else {
        (*curbuf.get()).b_p_sts as ::core::ffi::c_int
    };
    return result;
}
pub unsafe extern "C" fn get_indent() -> ::core::ffi::c_int {
    return indent_size_ts(
        get_cursor_line_ptr(),
        (*curbuf.get()).b_p_ts,
        (*curbuf.get()).b_p_vts_array,
    );
}
pub unsafe extern "C" fn get_indent_lnum(mut lnum: linenr_T) -> ::core::ffi::c_int {
    return indent_size_ts(
        ml_get(lnum),
        (*curbuf.get()).b_p_ts,
        (*curbuf.get()).b_p_vts_array,
    );
}
pub unsafe extern "C" fn get_indent_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    return indent_size_ts(ml_get_buf(buf, lnum), (*buf).b_p_ts, (*buf).b_p_vts_array);
}
pub unsafe extern "C" fn indent_size_no_ts(
    mut ptr: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tab_size: ::core::ffi::c_int = byte2cells(TAB);
    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let c2rust_fresh4 = ptr;
        ptr = ptr.offset(1);
        let c: ::core::ffi::c_char = *c2rust_fresh4;
        if c as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            vcol += 1;
        } else if c as ::core::ffi::c_int == TAB {
            vcol += tab_size;
        } else {
            return vcol;
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn indent_size_ts(
    mut ptr: *const ::core::ffi::c_char,
    mut ts: OptInt,
    mut vts: *mut colnr_T,
) -> ::core::ffi::c_int {
    assert!(char2cells(' ' as ::core::ffi::c_int) == 1 as ::core::ffi::c_int);
    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tabstop_width: ::core::ffi::c_int = 0;
    let mut next_tab_vcol: ::core::ffi::c_int = 0;
    if vts.is_null() || *vts.offset(0 as ::core::ffi::c_int as isize) < 1 as ::core::ffi::c_int {
        tabstop_width = (if ts == 0 as OptInt { 8 as OptInt } else { ts }) as ::core::ffi::c_int;
        next_tab_vcol = tabstop_width;
    } else {
        let mut cur_tabstop: *mut colnr_T = vts.offset(1 as ::core::ffi::c_int as isize);
        let last_tabstop: *mut colnr_T =
            vts.offset(*vts.offset(0 as ::core::ffi::c_int as isize) as isize);
        while cur_tabstop != last_tabstop {
            let mut cur_vcol: ::core::ffi::c_int = vcol;
            let c2rust_fresh1 = cur_tabstop;
            cur_tabstop = cur_tabstop.offset(1);
            vcol += *c2rust_fresh1 as ::core::ffi::c_int;
            assert!(cur_vcol < vcol);
            loop {
                let c2rust_fresh2 = ptr;
                ptr = ptr.offset(1);
                let c: ::core::ffi::c_char = *c2rust_fresh2;
                if c as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                    cur_vcol += 1;
                    if cur_vcol == vcol {
                        break;
                    }
                } else {
                    if c as ::core::ffi::c_int == TAB {
                        break;
                    }
                    return cur_vcol;
                }
            }
        }
        tabstop_width = *last_tabstop as ::core::ffi::c_int;
        next_tab_vcol = vcol + tabstop_width;
    }
    assert!(tabstop_width != 0 as ::core::ffi::c_int);
    loop {
        let c2rust_fresh3 = ptr;
        ptr = ptr.offset(1);
        let c_0: ::core::ffi::c_char = *c2rust_fresh3;
        if c_0 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            vcol += 1;
            next_tab_vcol += if vcol == next_tab_vcol {
                tabstop_width
            } else {
                0 as ::core::ffi::c_int
            };
        } else if c_0 as ::core::ffi::c_int == TAB {
            vcol = next_tab_vcol;
            next_tab_vcol += tabstop_width;
        } else {
            return vcol;
        }
    }
}
pub unsafe extern "C" fn set_indent(
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> bool {
    let mut newline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oldline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut doit = false;
    let mut ind_done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tab_pad: ::core::ffi::c_int = 0;
    let mut retval: bool = false;
    let mut orig_char_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut todo: ::core::ffi::c_int = size;
    let mut ind_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    oldline = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = oldline;
    let mut line_len: ::core::ffi::c_int = get_cursor_line_len() + 1 as ::core::ffi::c_int;
    if (*curbuf.get()).b_p_et == 0
        || flags & SIN_INSERT as ::core::ffi::c_int == 0 && (*curbuf.get()).b_p_pi != 0
    {
        let mut ind_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if flags & SIN_INSERT as ::core::ffi::c_int == 0 && (*curbuf.get()).b_p_pi != 0 {
            ind_done = 0 as ::core::ffi::c_int;
            while todo > 0 as ::core::ffi::c_int && ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == TAB {
                    tab_pad = tabstop_padding(
                        ind_done as colnr_T,
                        (*curbuf.get()).b_p_ts,
                        (*curbuf.get()).b_p_vts_array,
                    );
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    ind_len += 1;
                    ind_done += tab_pad;
                } else {
                    todo -= 1;
                    ind_len += 1;
                    ind_done += 1;
                }
                p = p.offset(1);
            }
            ind_col = ind_done;
            if (*curbuf.get()).b_p_et != 0 {
                orig_char_len = ind_len;
            }
            tab_pad = tabstop_padding(
                ind_done as colnr_T,
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
            if todo >= tab_pad && orig_char_len == -1 as ::core::ffi::c_int {
                doit = true;
                todo -= tab_pad;
                ind_len += 1;
                ind_col += tab_pad;
            }
        }
        loop {
            tab_pad = tabstop_padding(
                ind_col as colnr_T,
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
            if todo < tab_pad {
                break;
            }
            if *p as ::core::ffi::c_int != TAB {
                doit = true;
            } else {
                p = p.offset(1);
            }
            todo -= tab_pad;
            ind_len += 1;
            ind_col += tab_pad;
        }
    }
    while todo > 0 as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
            doit = true;
        } else {
            p = p.offset(1);
        }
        todo -= 1;
        ind_len += 1;
    }
    if !doit
        && !ascii_iswhite(*p as ::core::ffi::c_int)
        && flags & SIN_INSERT as ::core::ffi::c_int == 0
    {
        return false;
    }
    if flags & SIN_INSERT as ::core::ffi::c_int != 0 {
        p = oldline;
    } else {
        p = skipwhite(p);
        line_len -= p.offset_from(oldline) as ::core::ffi::c_int;
    }
    let mut skipcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if orig_char_len != -1 as ::core::ffi::c_int {
        let mut newline_size: ::core::ffi::c_int = 0;
        let (c2rust_result, c2rust_overflowed) = orig_char_len.overflowing_add(size);
        *&raw mut newline_size = c2rust_result;
        if c2rust_overflowed {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"set_indent\0".as_ptr() as *const ::core::ffi::c_char,
                598 as ::core::ffi::c_int,
                true,
                b"STRICT_ADD overflow\0".as_ptr() as *const ::core::ffi::c_char,
            );
            abort();
        }
        let (c2rust_result_0, c2rust_overflowed_0) = newline_size.overflowing_sub(ind_done);
        *&raw mut newline_size = c2rust_result_0;
        if c2rust_overflowed_0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"set_indent\0".as_ptr() as *const ::core::ffi::c_char,
                599 as ::core::ffi::c_int,
                true,
                b"STRICT_SUB overflow\0".as_ptr() as *const ::core::ffi::c_char,
            );
            abort();
        }
        let (c2rust_result_1, c2rust_overflowed_1) = newline_size.overflowing_add(line_len);
        *&raw mut newline_size = c2rust_result_1;
        if c2rust_overflowed_1 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"set_indent\0".as_ptr() as *const ::core::ffi::c_char,
                600 as ::core::ffi::c_int,
                true,
                b"STRICT_ADD overflow\0".as_ptr() as *const ::core::ffi::c_char,
            );
            abort();
        }
        assert!(newline_size >= 0 as ::core::ffi::c_int);
        newline = xmalloc(newline_size as size_t) as *mut ::core::ffi::c_char;
        todo = size - ind_done;
        ind_len = orig_char_len + todo;
        p = oldline;
        s = newline;
        skipcols = orig_char_len;
        while orig_char_len > 0 as ::core::ffi::c_int {
            let c2rust_fresh5 = p;
            p = p.offset(1);
            let c2rust_fresh6 = s;
            s = s.offset(1);
            *c2rust_fresh6 = *c2rust_fresh5;
            orig_char_len -= 1;
        }
        while ascii_iswhite(*p as ::core::ffi::c_int) {
            p = p.offset(1);
        }
    } else {
        todo = size;
        assert!(ind_len + line_len >= 0 as ::core::ffi::c_int);
        let mut newline_size_0: size_t = 0;
        let (c2rust_result_2, c2rust_overflowed_2) =
            (ind_len as i128).overflowing_add(line_len as i128);
        let c2rust_result_narrow = c2rust_result_2 as size_t;
        *&raw mut newline_size_0 = c2rust_result_narrow;
        if c2rust_overflowed_2 || c2rust_result_narrow as i128 != c2rust_result_2 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"set_indent\0".as_ptr() as *const ::core::ffi::c_char,
                626 as ::core::ffi::c_int,
                true,
                b"STRICT_ADD overflow\0".as_ptr() as *const ::core::ffi::c_char,
            );
            abort();
        }
        newline = xmalloc(newline_size_0) as *mut ::core::ffi::c_char;
        s = newline;
    }
    if (*curbuf.get()).b_p_et == 0 {
        if flags & SIN_INSERT as ::core::ffi::c_int == 0 && (*curbuf.get()).b_p_pi != 0 {
            p = oldline;
            ind_done = 0 as ::core::ffi::c_int;
            while todo > 0 as ::core::ffi::c_int && ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == TAB {
                    tab_pad = tabstop_padding(
                        ind_done as colnr_T,
                        (*curbuf.get()).b_p_ts,
                        (*curbuf.get()).b_p_vts_array,
                    );
                    if todo < tab_pad {
                        break;
                    }
                    todo -= tab_pad;
                    ind_done += tab_pad;
                } else {
                    todo -= 1;
                    ind_done += 1;
                }
                let c2rust_fresh7 = p;
                p = p.offset(1);
                let c2rust_fresh8 = s;
                s = s.offset(1);
                *c2rust_fresh8 = *c2rust_fresh7;
                skipcols += 1;
            }
            tab_pad = tabstop_padding(
                ind_done as colnr_T,
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
            if todo >= tab_pad {
                let c2rust_fresh9 = s;
                s = s.offset(1);
                *c2rust_fresh9 = TAB as ::core::ffi::c_char;
                todo -= tab_pad;
                ind_done += tab_pad;
            }
            p = skipwhite(p);
        }
        loop {
            tab_pad = tabstop_padding(
                ind_done as colnr_T,
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
            if todo < tab_pad {
                break;
            }
            let c2rust_fresh10 = s;
            s = s.offset(1);
            *c2rust_fresh10 = TAB as ::core::ffi::c_char;
            todo -= tab_pad;
            ind_done += tab_pad;
        }
    }
    while todo > 0 as ::core::ffi::c_int {
        let c2rust_fresh11 = s;
        s = s.offset(1);
        *c2rust_fresh11 = ' ' as ::core::ffi::c_char;
        todo -= 1;
    }
    memmove(
        s as *mut ::core::ffi::c_void,
        p as *const ::core::ffi::c_void,
        line_len as size_t,
    );
    if flags & SIN_UNDO as ::core::ffi::c_int == 0 || u_savesub((*curwin.get()).w_cursor.lnum) == OK
    {
        let old_offset: colnr_T = p.offset_from(oldline) as colnr_T;
        let new_offset: colnr_T = s.offset_from(newline) as colnr_T;
        ml_replace((*curwin.get()).w_cursor.lnum, newline, false);
        if flags & SIN_NOMARK as ::core::ffi::c_int == 0 {
            extmark_splice_cols(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                skipcols as colnr_T,
                old_offset - skipcols as colnr_T,
                new_offset - skipcols as colnr_T,
                kExtmarkUndo,
            );
        }
        if flags & SIN_CHANGED as ::core::ffi::c_int != 0 {
            changed_bytes((*curwin.get()).w_cursor.lnum, 0 as colnr_T);
        }
        if (*saved_cursor.ptr()).lnum == (*curwin.get()).w_cursor.lnum {
            if (*saved_cursor.ptr()).col >= old_offset {
                (*saved_cursor.ptr()).col +=
                    (ind_len as colnr_T - old_offset) as ::core::ffi::c_int;
            } else if (*saved_cursor.ptr()).col >= new_offset {
                (*saved_cursor.ptr()).col = new_offset;
            }
        }
        retval = true;
    } else {
        xfree(newline as *mut ::core::ffi::c_void);
    }
    (*curwin.get()).w_cursor.col = ind_len as colnr_T;
    return retval;
}
pub unsafe extern "C" fn get_number_indent(mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut col: colnr_T = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut lead_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if lnum > (*curbuf.get()).b_ml.ml_line_count {
        return -1 as ::core::ffi::c_int;
    }
    pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 || has_format_option(FO_Q_COMS) {
        lead_len = get_leader_len(
            ml_get(lnum),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false,
            true,
        );
    }
    regmatch.regprog = vim_regcomp((*curbuf.get()).b_p_flp, RE_MAGIC);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = false;
        if vim_regexec(
            &raw mut regmatch,
            ml_get(lnum).offset(lead_len as isize),
            0 as colnr_T,
        ) {
            pos.lnum = lnum;
            pos.col = (*(&raw mut regmatch.endp as *mut *mut ::core::ffi::c_char))
                .offset_from(ml_get(lnum)) as colnr_T;
            pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
        vim_regfree(regmatch.regprog);
    }
    if pos.lnum == 0 as linenr_T || *ml_get_pos(&raw mut pos) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    getvcol(
        curwin.get(),
        &raw mut pos,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_STRICT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const RE_AUTO: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
