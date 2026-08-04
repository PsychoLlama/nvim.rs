//! Reindenting text: the `=` operator, `:retab`, the `<C-t>`/`<C-d>`
//! shifts, Insert-mode smart indent, and copying an existing line's indent.

use super::*;
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::src::nvim::change::{changed_lines, ins_bytes, ins_str};
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::cursor::{coladvance, get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::drawscreen::{UPD_INVERTED, UPD_NOT_VALID, redraw_curbuf_later};
use crate::src::nvim::edit::{backspace_until_column, beginline, replace_join, replace_push_nul};
use crate::src::nvim::extmark::extmark_splice_cols;
use crate::src::nvim::indent_c::in_cinkeys;
use crate::src::nvim::main::{
    IObuff, Insstart, State, ai_col, can_si, can_si_back, cmdmod, curbuf, curbuf_splice_pending,
    curwin, did_si, e_interr, e_modifiable, e_resulting_text_too_long, got_int, old_indent,
    p_paste, p_report, trylevel,
};
use crate::src::nvim::mbyte::{utf_ptr2StrCharInfo, utfc_next, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_len, ml_replace};
use crate::src::nvim::memory::{xfree, xmalloc, xmallocz, xmemdupz};
use crate::src::nvim::message::{emsg, msg_progress};
use crate::src::nvim::r#move::changed_cline_bef_curs;
use crate::src::nvim::ops::shift_line;
use crate::src::nvim::option::set_option_direct;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{gettext, memmove, memset, ngettext, snprintf, strncmp};
use crate::src::nvim::plines::{getvcol_nolist, init_charsize_arg, win_charsize, win_chartabsize};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::findmatch;
use crate::src::nvim::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG};
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::undo::{u_clearline, u_save, u_savecommon};

pub unsafe extern "C" fn inindent(mut extra: ::core::ffi::c_int) -> bool {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut col: colnr_T = 0;
    col = 0 as ::core::ffi::c_int as colnr_T;
    ptr = get_cursor_line_ptr();
    while ascii_iswhite(*ptr as ::core::ffi::c_int) {
        ptr = ptr.offset(1);
        col += 1;
    }
    if col >= (*curwin.get()).w_cursor.col as ::core::ffi::c_int + extra {
        return true;
    }
    return false;
}
pub unsafe extern "C" fn op_reindent(mut oap: *mut oparg_T, mut how: Indenter) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut first_changed: linenr_T = 0 as linenr_T;
    let mut last_changed: linenr_T = 0 as linenr_T;
    let mut start_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        return;
    }
    if u_savecommon(
        curbuf.get(),
        start_lnum - 1 as linenr_T,
        start_lnum + (*oap).line_count,
        start_lnum + (*oap).line_count,
        false,
    ) == OK
    {
        let mut amount: ::core::ffi::c_int = 0;
        i = ((*oap).line_count - 1 as linenr_T) as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int && !got_int.get() {
            if i > 1 as ::core::ffi::c_int
                && (i % 50 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    || i as linenr_T == (*oap).line_count - 1 as linenr_T)
                && (*oap).line_count as OptInt > p_report.get()
            {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(c"%ld lines to indent... ".as_ptr()),
                    i as int64_t,
                );
                let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                (*curwin.get()).w_cursor.lnum = start_lnum;
                msg_progress(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    c"indent".as_ptr() as *mut ::core::ffi::c_char,
                    c"running".as_ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    true,
                    false,
                );
                (*curwin.get()).w_cursor.lnum = save_lnum;
            }
            if i as linenr_T != (*oap).line_count - 1 as linenr_T
                || (*oap).line_count == 1 as linenr_T
                || !how.is_some_and(|f| {
                    ::core::ptr::fn_addr_eq(
                        f,
                        get_lisp_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                    )
                })
            {
                let mut l: *mut ::core::ffi::c_char = skipwhite(get_cursor_line_ptr());
                amount = if *l as ::core::ffi::c_int == NUL {
                    0 as ::core::ffi::c_int
                } else {
                    how.expect("non-null function pointer")()
                };
                if amount >= 0 as ::core::ffi::c_int && set_indent(amount, 0 as ::core::ffi::c_int)
                {
                    if first_changed == 0 as linenr_T {
                        first_changed = (*curwin.get()).w_cursor.lnum;
                    }
                    last_changed = (*curwin.get()).w_cursor.lnum;
                }
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            i -= 1;
        }
    }
    (*curwin.get()).w_cursor.lnum = start_lnum;
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    if last_changed != 0 as linenr_T {
        changed_lines(
            curbuf.get(),
            first_changed,
            0 as colnr_T,
            if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
                start_lnum + (*oap).line_count
            } else {
                last_changed + 1 as linenr_T
            },
            0 as linenr_T,
            true,
        );
    } else if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED);
    }
    if (*oap).line_count as OptInt > p_report.get() {
        i = ((*oap).line_count - (i as linenr_T + 1 as linenr_T)) as ::core::ffi::c_int;
        snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            ngettext(
                c"%ld line indented ".as_ptr(),
                c"%ld lines indented ".as_ptr(),
                i as ::core::ffi::c_ulong,
            ),
            i as int64_t,
        );
        msg_progress(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            c"indent".as_ptr() as *mut ::core::ffi::c_char,
            c"success".as_ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            true,
            false,
        );
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
    }
}
pub unsafe extern "C" fn preprocs_left() -> bool {
    return (*curbuf.get()).b_p_si != 0 && (*curbuf.get()).b_p_cin == 0
        || (*curbuf.get()).b_p_cin != 0
            && in_cinkeys('#' as ::core::ffi::c_int, ' ' as ::core::ffi::c_int, true)
            && (*curbuf.get()).b_ind_hash_comment == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn may_do_si() -> bool {
    return (*curbuf.get()).b_p_si != 0
        && (*curbuf.get()).b_p_cin == 0
        && *(*curbuf.get()).b_p_inde as ::core::ffi::c_int == NUL
        && p_paste.get() == 0;
}
pub unsafe extern "C" fn ins_try_si(mut c: ::core::ffi::c_int) {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if (did_si.get() || can_si_back.get()) && c == '{' as ::core::ffi::c_int
        || can_si.get() && c == '}' as ::core::ffi::c_int && inindent(0 as ::core::ffi::c_int)
    {
        let mut old_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: ::core::ffi::c_int = 0;
        let mut temp: bool = false;
        if c == '}' as ::core::ffi::c_int && {
            pos = findmatch(
                ::core::ptr::null_mut::<oparg_T>(),
                '{' as ::core::ffi::c_int,
            );
            !pos.is_null()
        } {
            old_pos = (*curwin.get()).w_cursor;
            ptr = ml_get((*pos).lnum);
            i = (*pos).col as ::core::ffi::c_int;
            if i > 0 as ::core::ffi::c_int {
                loop {
                    i -= 1;
                    if !(i > 0 as ::core::ffi::c_int
                        && ascii_iswhite(*ptr.offset(i as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0)
                    {
                        break;
                    }
                }
            }
            (*curwin.get()).w_cursor.lnum = (*pos).lnum;
            (*curwin.get()).w_cursor.col = i as colnr_T;
            if *ptr.offset(i as isize) as ::core::ffi::c_int == ')' as ::core::ffi::c_int && {
                pos = findmatch(
                    ::core::ptr::null_mut::<oparg_T>(),
                    '(' as ::core::ffi::c_int,
                );
                !pos.is_null()
            } {
                (*curwin.get()).w_cursor = *pos;
            }
            i = get_indent();
            (*curwin.get()).w_cursor = old_pos;
            if State.get() & VREPLACE_FLAG != 0 {
                change_indent(INDENT_SET as ::core::ffi::c_int, i, 0, true);
            } else {
                set_indent(i, SIN_CHANGED as ::core::ffi::c_int);
            }
        } else if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int {
            temp = true;
            if c == '{' as ::core::ffi::c_int
                && can_si_back.get()
                && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            {
                old_pos = (*curwin.get()).w_cursor;
                i = get_indent();
                while (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                    (*curwin.get()).w_cursor.lnum -= 1;
                    ptr = skipwhite(ml_get((*curwin.get()).w_cursor.lnum));
                    if *ptr as ::core::ffi::c_int != '#' as ::core::ffi::c_int
                        && *ptr as ::core::ffi::c_int != NUL
                    {
                        break;
                    }
                }
                if get_indent() >= i {
                    temp = false;
                }
                (*curwin.get()).w_cursor = old_pos;
            }
            if temp {
                shift_line(true, false, 1 as ::core::ffi::c_int, 1);
            }
        }
    }
    if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && can_si.get()
        && c == '#' as ::core::ffi::c_int
        && inindent(0 as ::core::ffi::c_int)
    {
        old_indent.set(get_indent());
        set_indent(0 as ::core::ffi::c_int, SIN_CHANGED as ::core::ffi::c_int);
    }
    ai_col.set(if ai_col.get() < (*curwin.get()).w_cursor.col {
        ai_col.get()
    } else {
        (*curwin.get()).w_cursor.col
    });
}
pub unsafe extern "C" fn change_indent(
    mut type_0: ::core::ffi::c_int,
    mut amount: ::core::ffi::c_int,
    mut round: ::core::ffi::c_int,
    mut call_changed_bytes: bool,
) {
    let mut insstart_less: ::core::ffi::c_int = 0;
    let mut orig_col: colnr_T = 0 as colnr_T;
    let mut orig_line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if State.get() & VREPLACE_FLAG != 0 {
        orig_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
        orig_col = (*curwin.get()).w_cursor.col;
    }
    let mut save_p_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
    (*curwin.get()).w_onebuf_opt.wo_list = 0;
    let mut vc: colnr_T = getvcol_nolist(&raw mut (*curwin.get()).w_cursor);
    let mut vcol: ::core::ffi::c_int = vc as ::core::ffi::c_int;
    let mut start_col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    let mut new_cursor_col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    beginline(BL_WHITE as ::core::ffi::c_int);
    new_cursor_col -= (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    insstart_less = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    if new_cursor_col < 0 as ::core::ffi::c_int {
        vcol = get_indent() - vcol;
    }
    if new_cursor_col > 0 as ::core::ffi::c_int {
        start_col = -1 as ::core::ffi::c_int;
    }
    if type_0 == INDENT_SET as ::core::ffi::c_int {
        set_indent(
            amount,
            if call_changed_bytes as ::core::ffi::c_int != 0 {
                SIN_CHANGED as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
    } else {
        let mut save_State: ::core::ffi::c_int = State.get();
        if State.get() & VREPLACE_FLAG != 0 {
            State.set(MODE_INSERT);
        }
        shift_line(
            type_0 == INDENT_DEC as ::core::ffi::c_int,
            round != 0,
            1 as ::core::ffi::c_int,
            call_changed_bytes as ::core::ffi::c_int,
        );
        State.set(save_State);
    }
    insstart_less -= (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    if new_cursor_col >= 0 as ::core::ffi::c_int {
        if new_cursor_col == 0 as ::core::ffi::c_int {
            insstart_less = MAXCOL as ::core::ffi::c_int;
        }
        new_cursor_col += (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    } else if State.get() & MODE_INSERT == 0 {
        new_cursor_col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    } else {
        vcol = get_indent() - vcol;
        let end_vcol: ::core::ffi::c_int = if vcol < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            vcol
        };
        (*curwin.get()).w_virtcol = end_vcol as colnr_T;
        new_cursor_col = 0 as ::core::ffi::c_int;
        let line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        vcol = 0 as ::core::ffi::c_int;
        if *line as ::core::ffi::c_int != NUL {
            let mut csarg: CharsizeArg = CharsizeArg::default();
            let mut cstype: CharsizeKind =
                init_charsize_arg(&mut csarg, curwin.get(), 0 as linenr_T, line);
            let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
            loop {
                let mut next_vcol: ::core::ffi::c_int =
                    vcol + win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                if next_vcol > end_vcol {
                    break;
                }
                vcol = next_vcol;
                ci = utfc_next(ci);
                if *ci.ptr as ::core::ffi::c_int == NUL {
                    break;
                }
            }
            new_cursor_col = ci.ptr.offset_from(line) as ::core::ffi::c_int;
        }
        if vcol != (*curwin.get()).w_virtcol {
            (*curwin.get()).w_cursor.col = new_cursor_col;
            let ptrlen: size_t = ((*curwin.get()).w_virtcol as ::core::ffi::c_int - vcol) as size_t;
            let mut ptr: *mut ::core::ffi::c_char = xmallocz(ptrlen) as *mut ::core::ffi::c_char;
            memset(
                ptr as *mut ::core::ffi::c_void,
                ' ' as ::core::ffi::c_int,
                ptrlen,
            );
            new_cursor_col += ptrlen as ::core::ffi::c_int;
            ins_str(ptr, ptrlen);
            xfree(ptr as *mut ::core::ffi::c_void);
        }
        insstart_less = MAXCOL as ::core::ffi::c_int;
    }
    (*curwin.get()).w_onebuf_opt.wo_list = save_p_list;
    (*curwin.get()).w_cursor.col = (if 0 as ::core::ffi::c_int > new_cursor_col {
        0 as ::core::ffi::c_int
    } else {
        new_cursor_col
    }) as colnr_T;
    (*curwin.get()).w_set_curswant = 1;
    changed_cline_bef_curs(curwin.get());
    if State.get() & MODE_INSERT != 0 {
        if (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
            && (*Insstart.ptr()).col != 0 as ::core::ffi::c_int
        {
            if (*Insstart.ptr()).col <= insstart_less {
                (*Insstart.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                (*Insstart.ptr()).col -= insstart_less;
            }
        }
        if ai_col.get() <= insstart_less {
            ai_col.set(0 as ::core::ffi::c_int as colnr_T);
        } else {
            (*ai_col.ptr()) -= insstart_less;
        }
    }
    if State.get() & REPLACE_FLAG != 0
        && State.get() & VREPLACE_FLAG == 0
        && start_col >= 0 as ::core::ffi::c_int
    {
        while start_col > (*curwin.get()).w_cursor.col {
            replace_join(0 as ::core::ffi::c_int);
            start_col -= 1;
        }
        while start_col < (*curwin.get()).w_cursor.col {
            replace_push_nul();
            start_col += 1;
        }
    }
    if State.get() & VREPLACE_FLAG != 0 {
        let mut new_line: *mut ::core::ffi::c_char =
            xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
        *new_line.offset((*curwin.get()).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
        let mut new_col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        ml_replace((*curwin.get()).w_cursor.lnum, orig_line, false);
        (*curwin.get()).w_cursor.col = orig_col;
        (*curbuf_splice_pending.ptr()) += 1;
        backspace_until_column(0 as ::core::ffi::c_int);
        ins_bytes(new_line);
        xfree(new_line as *mut ::core::ffi::c_void);
        (*curbuf_splice_pending.ptr()) -= 1;
        let mut delta: ::core::ffi::c_int = orig_col as ::core::ffi::c_int - new_col;
        extmark_splice_cols(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            new_col as colnr_T,
            if delta < 0 as ::core::ffi::c_int {
                -(delta as colnr_T)
            } else {
                0 as colnr_T
            },
            if delta > 0 as ::core::ffi::c_int {
                delta as colnr_T
            } else {
                0 as colnr_T
            },
            kExtmarkUndo,
        );
    }
}
pub unsafe extern "C" fn copy_indent(
    mut size: ::core::ffi::c_int,
    mut src: *mut ::core::ffi::c_char,
) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ind_len: ::core::ffi::c_int = 0;
    let mut line_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tab_pad: ::core::ffi::c_int = 0;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        let mut todo: ::core::ffi::c_int = size;
        ind_len = 0 as ::core::ffi::c_int;
        let mut ind_done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ind_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = src;
        while todo > 0 as ::core::ffi::c_int && ascii_iswhite(*s as ::core::ffi::c_int) {
            if *s as ::core::ffi::c_int == TAB {
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
                ind_col += tab_pad;
            } else {
                todo -= 1;
                ind_done += 1;
                ind_col += 1;
            }
            ind_len += 1;
            if !p.is_null() {
                let c2rust_fresh12 = p;
                p = p.offset(1);
                *c2rust_fresh12 = *s;
            }
            s = s.offset(1);
        }
        tab_pad = tabstop_padding(
            ind_done as colnr_T,
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
        );
        if todo >= tab_pad && (*curbuf.get()).b_p_et == 0 {
            todo -= tab_pad;
            ind_len += 1;
            ind_col += tab_pad;
            if !p.is_null() {
                let c2rust_fresh13 = p;
                p = p.offset(1);
                *c2rust_fresh13 = TAB as ::core::ffi::c_char;
            }
        }
        if (*curbuf.get()).b_p_et == 0 {
            loop {
                tab_pad = tabstop_padding(
                    ind_col as colnr_T,
                    (*curbuf.get()).b_p_ts,
                    (*curbuf.get()).b_p_vts_array,
                );
                if todo < tab_pad {
                    break;
                }
                todo -= tab_pad;
                ind_len += 1;
                ind_col += tab_pad;
                if !p.is_null() {
                    let c2rust_fresh14 = p;
                    p = p.offset(1);
                    *c2rust_fresh14 = TAB as ::core::ffi::c_char;
                }
            }
        }
        while todo > 0 as ::core::ffi::c_int {
            todo -= 1;
            ind_len += 1;
            if !p.is_null() {
                let c2rust_fresh15 = p;
                p = p.offset(1);
                *c2rust_fresh15 = ' ' as ::core::ffi::c_char;
            }
        }
        if p.is_null() {
            line_len = get_cursor_line_len() + 1 as ::core::ffi::c_int;
            // Both operands are non-negative `int`s, so the only way the
            // narrowing to `size_t` the C guarded could fail is a negative sum.
            assert!(ind_len + line_len >= 0, "STRICT_ADD overflow");
            line = xmalloc((ind_len + line_len) as size_t) as *mut ::core::ffi::c_char;
            p = line;
        }
        round += 1;
    }
    memmove(
        p as *mut ::core::ffi::c_void,
        get_cursor_line_ptr() as *const ::core::ffi::c_void,
        line_len as size_t,
    );
    ml_replace((*curwin.get()).w_cursor.lnum, line, false);
    (*curwin.get()).w_cursor.col = ind_len as colnr_T;
    return true;
}
unsafe extern "C" fn emsg_text_too_long() {
    emsg(gettext(
        &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
    ));
    if trylevel.get() == 0 as ::core::ffi::c_int {
        got_int.set(true);
    }
}
pub unsafe fn ex_retab(mut eap: *mut exarg_T) {
    let mut got_tab: bool = false;
    let mut num_spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start_vcol: int64_t = 0 as int64_t;
    let mut new_line: *mut ::core::ffi::c_char = ::core::ptr::with_exposed_provenance_mut::<
        ::core::ffi::c_char,
    >(1 as ::core::ffi::c_int as usize);
    let mut new_vts_array: *mut colnr_T = ::core::ptr::null_mut::<colnr_T>();
    let mut new_ts_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut first_line: linenr_T = 0 as linenr_T;
    let mut last_line: linenr_T = 0 as linenr_T;
    let mut is_indent_only: bool = false;
    let mut save_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
    (*curwin.get()).w_onebuf_opt.wo_list = 0 as ::core::ffi::c_int;
    let mut ptr: *mut ::core::ffi::c_char = (*eap).arg;
    if strncmp(ptr, c"-indentonly".as_ptr(), 11 as size_t) == 0 as ::core::ffi::c_int
        && ascii_iswhite_or_nul(*ptr.offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        is_indent_only = true;
        ptr = skipwhite(ptr.offset(11 as ::core::ffi::c_int as isize));
    }
    new_ts_str = ptr;
    if !tabstop_set(ptr, &raw mut new_vts_array) {
        return;
    }
    while ascii_isdigit(*ptr as ::core::ffi::c_int)
        || *ptr as ::core::ffi::c_int == ',' as ::core::ffi::c_int
    {
        ptr = ptr.offset(1);
    }
    if new_vts_array.is_null() {
        new_vts_array = (*curbuf.get()).b_p_vts_array;
        new_ts_str = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        new_ts_str = xmemdupz(
            new_ts_str as *const ::core::ffi::c_void,
            ptr.offset_from(new_ts_str) as size_t,
        ) as *mut ::core::ffi::c_char;
    }
    let mut lnum: linenr_T = (*eap).line1;
    while !got_int.get() && lnum <= (*eap).line2 {
        ptr = ml_get(lnum);
        let mut old_len: ::core::ffi::c_int = ml_get_len(lnum);
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut vcol: int64_t = 0 as int64_t;
        let mut did_undo: bool = false;
        loop {
            if ascii_iswhite(*ptr.offset(col as isize) as ::core::ffi::c_int) {
                if !got_tab && num_spaces == 0 as ::core::ffi::c_int {
                    start_vcol = vcol;
                    start_col = col;
                }
                if *ptr.offset(col as isize) as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                    num_spaces += 1;
                } else {
                    got_tab = true;
                }
            } else {
                if got_tab as ::core::ffi::c_int != 0
                    || (*eap).forceit != 0 && num_spaces > 1 as ::core::ffi::c_int
                {
                    num_spaces = (vcol - start_vcol) as ::core::ffi::c_int;
                    let mut len: ::core::ffi::c_int = num_spaces;
                    let mut num_tabs: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*curbuf.get()).b_p_et == 0 {
                        let mut t: ::core::ffi::c_int = 0;
                        let mut s: ::core::ffi::c_int = 0;
                        tabstop_fromto(
                            start_vcol as colnr_T,
                            vcol as colnr_T,
                            (*curbuf.get()).b_p_ts as ::core::ffi::c_int,
                            new_vts_array,
                            &raw mut t,
                            &raw mut s,
                        );
                        num_tabs = t;
                        num_spaces = s;
                    }
                    if (*curbuf.get()).b_p_et != 0
                        || got_tab as ::core::ffi::c_int != 0
                        || num_spaces + num_tabs < len
                    {
                        if did_undo as ::core::ffi::c_int == 0 {
                            did_undo = true;
                            if u_save(lnum - 1 as linenr_T, lnum + 1 as linenr_T) == FAIL {
                                new_line = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                break;
                            }
                        }
                        len = num_spaces + num_tabs;
                        let new_len: ::core::ffi::c_int =
                            old_len - col + start_col + len + 1 as ::core::ffi::c_int;
                        if new_len <= 0 as ::core::ffi::c_int
                            || new_len >= MAXCOL as ::core::ffi::c_int
                        {
                            emsg_text_too_long();
                            break;
                        } else {
                            new_line = xmalloc(new_len as size_t) as *mut ::core::ffi::c_char;
                            if start_col > 0 as ::core::ffi::c_int {
                                memmove(
                                    new_line as *mut ::core::ffi::c_void,
                                    ptr as *const ::core::ffi::c_void,
                                    start_col as size_t,
                                );
                            }
                            memmove(
                                new_line.offset(start_col as isize).offset(len as isize)
                                    as *mut ::core::ffi::c_void,
                                ptr.offset(col as isize) as *const ::core::ffi::c_void,
                                (old_len as size_t)
                                    .wrapping_sub(col as size_t)
                                    .wrapping_add(1 as size_t),
                            );
                            ptr = new_line.offset(start_col as isize);
                            col = 0 as ::core::ffi::c_int;
                            while col < len {
                                *ptr.offset(col as isize) = (if col < num_tabs {
                                    '\t' as ::core::ffi::c_int
                                } else {
                                    ' ' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                                col += 1;
                            }
                            if ml_replace(lnum, new_line, false) == OK {
                                new_line = (*curbuf.get()).b_ml.ml_line_ptr;
                                extmark_splice_cols(
                                    curbuf.get(),
                                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    old_len,
                                    new_len - 1 as colnr_T,
                                    kExtmarkUndo,
                                );
                            }
                            if first_line == 0 as linenr_T {
                                first_line = lnum;
                            }
                            last_line = lnum;
                            ptr = new_line;
                            old_len = new_len - 1 as ::core::ffi::c_int;
                            col = start_col + len;
                        }
                    }
                }
                got_tab = false;
                num_spaces = 0 as ::core::ffi::c_int;
                if is_indent_only {
                    break;
                }
            }
            if *ptr.offset(col as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            vcol +=
                win_chartabsize(curwin.get(), ptr.offset(col as isize), vcol as colnr_T) as int64_t;
            if vcol >= MAXCOL as ::core::ffi::c_int as int64_t {
                emsg_text_too_long();
                break;
            } else {
                col += utfc_ptr2len(ptr.offset(col as isize));
            }
        }
        if new_line.is_null() {
            break;
        }
        line_breakcheck();
        lnum += 1;
    }
    if got_int.get() {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
    }
    if !(tabstop_count((*curbuf.get()).b_p_vts_array) == 0 as ::core::ffi::c_int
        && tabstop_count(new_vts_array) == 1 as ::core::ffi::c_int
        && (*curbuf.get()).b_p_ts == tabstop_first(new_vts_array) as OptInt)
    {
        if !(tabstop_count((*curbuf.get()).b_p_vts_array) > 0 as ::core::ffi::c_int
            && tabstop_eq((*curbuf.get()).b_p_vts_array, new_vts_array))
        {
            redraw_curbuf_later(UPD_NOT_VALID);
        }
    }
    if first_line != 0 as linenr_T {
        changed_lines(
            curbuf.get(),
            first_line,
            0 as colnr_T,
            last_line + 1 as linenr_T,
            0 as linenr_T,
            true,
        );
    }
    (*curwin.get()).w_onebuf_opt.wo_list = save_list;
    if !new_ts_str.is_null() {
        let mut old_vts_ary: *mut colnr_T = (*curbuf.get()).b_p_vts_array;
        if tabstop_count(old_vts_ary) > 0 as ::core::ffi::c_int
            || tabstop_count(new_vts_array) > 1 as ::core::ffi::c_int
        {
            set_option_direct(
                kOptVartabstop,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(new_ts_str),
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
                0 as scid_T,
            );
            (*curbuf.get()).b_p_vts_array = new_vts_array;
            xfree(old_vts_ary as *mut ::core::ffi::c_void);
        } else {
            (*curbuf.get()).b_p_ts = tabstop_first(new_vts_array) as OptInt;
            xfree(new_vts_array as *mut ::core::ffi::c_void);
        }
        xfree(new_ts_str as *mut ::core::ffi::c_void);
    }
    coladvance(curwin.get(), (*curwin.get()).w_curswant);
    u_clearline(curbuf.get());
}
