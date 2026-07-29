//! Pattern searches driven from normal mode, and the marks and jumps
//! that share their "remember where we were" bookkeeping.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_search(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == '?' as c_int && (*(*cap).oap).op_type == OP_ROT13 as c_int {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = '?' as c_int;
        nv_operator(cap);
        return;
    }
    (*cap).searchbuf = getcmdline((*cap).cmdchar, (*cap).count1, 0 as c_int, true_0 != 0);
    if (*cap).searchbuf.is_null() {
        clearop(oap);
        return;
    }
    normal_search(
        cap,
        (*cap).cmdchar,
        (*cap).searchbuf,
        strlen((*cap).searchbuf),
        if (*cap).arg != 0 || !equalpos(save_cursor, (*curwin.get()).w_cursor) {
            0 as c_int
        } else {
            SEARCH_MARK as c_int
        },
        ::core::ptr::null_mut::<c_int>(),
    );
}

pub(crate) unsafe extern "C" fn nv_next(mut cap: *mut cmdarg_T) {
    let mut old: pos_T = (*curwin.get()).w_cursor;
    let mut wrapped: c_int = false_0;
    let mut i: c_int = normal_search(
        cap,
        0 as c_int,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
        SEARCH_MARK as c_int | (*cap).arg,
        &raw mut wrapped,
    );
    if i == 1 as c_int && wrapped == 0 && equalpos(old, (*curwin.get()).w_cursor) as c_int != 0 {
        (*cap).count1 += 1 as c_int;
        normal_search(
            cap,
            0 as c_int,
            ::core::ptr::null_mut::<c_char>(),
            0 as size_t,
            SEARCH_MARK as c_int | (*cap).arg,
            ::core::ptr::null_mut::<c_int>(),
        );
        (*cap).count1 -= 1 as c_int;
    }
    if i > 0 as c_int
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as c_int) != win_hl_attr(curwin.get(), HLF_L as c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as c_int);
    }
}

pub(crate) unsafe extern "C" fn normal_search(
    mut cap: *mut cmdarg_T,
    mut dir: c_int,
    mut pat: *mut c_char,
    mut patlen: size_t,
    mut opt: c_int,
    mut wrapped: *mut c_int,
) -> c_int {
    let mut sia: searchit_arg_T = searchit_arg_T {
        sa_stop_lnum: 0,
        sa_tm: ::core::ptr::null_mut::<proftime_T>(),
        sa_timed_out: 0,
        sa_wrapped: 0,
    };
    let prev_cursor: pos_T = (*curwin.get()).w_cursor;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    memset(
        &raw mut sia as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<searchit_arg_T>(),
    );
    let mut i: c_int = do_search(
        (*cap).oap,
        dir,
        dir,
        pat,
        patlen,
        (*cap).count1,
        opt | SEARCH_OPT as c_int | SEARCH_ECHO as c_int | SEARCH_MSG as c_int,
        &raw mut sia,
    );
    if !wrapped.is_null() {
        *wrapped = sia.sa_wrapped;
    }
    if i == 0 as c_int {
        clearop((*cap).oap);
    } else {
        if i == 2 as c_int {
            (*(*cap).oap).motion_type = kMTLineWise;
        }
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
        if (*(*cap).oap).op_type == OP_NOP as c_int
            && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
        {
            foldOpenCursor();
        }
    }
    if !equalpos((*curwin.get()).w_cursor, prev_cursor)
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as c_int) != win_hl_attr(curwin.get(), HLF_L as c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as c_int);
    }
    check_cursor(curwin.get());
    return i;
}

pub(crate) unsafe extern "C" fn nv_mark(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if setmark((*cap).nchar) == false_0 {
        clearopbeep((*cap).oap);
    }
}

pub(crate) unsafe extern "C" fn nv_mark_move_to(
    mut cap: *mut cmdarg_T,
    mut flags: MarkMove,
    mut fm: *mut fmark_T,
) -> MarkMoveRes {
    let mut res: MarkMoveRes = mark_move_to(fm, flags);
    if res as c_uint & kMarkMoveFailed as c_int as c_uint != 0 {
        clearop((*cap).oap);
    }
    (*(*cap).oap).motion_type = (if flags as c_uint & kMarkBeginLine as c_int as c_uint != 0 {
        kMTLineWise as c_int
    } else {
        kMTCharWise as c_int
    }) as MotionType;
    if (*cap).cmdchar == '`' as c_int {
        (*(*cap).oap).use_reg_one = true_0 != 0;
    }
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    return res;
}

pub(crate) unsafe extern "C" fn nv_gomark(mut cap: *mut cmdarg_T) {
    let mut name: c_int = 0;
    let mut flags: MarkMove = (if jop_flags.get() & kOptJopFlagView as c_int as c_uint != 0 {
        kMarkSetView as c_int
    } else {
        0 as c_int
    }) as MarkMove;
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        flags = 0 as MarkMove;
    }
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if (*cap).cmdchar == 'g' as c_int {
        name = (*cap).extra_char;
        flags = (flags as c_uint | KMarkNoContext as c_int as c_uint) as MarkMove;
    } else {
        name = (*cap).nchar;
        flags = (flags as c_uint | kMarkContext as c_int as c_uint) as MarkMove;
    }
    flags = (flags as c_uint
        | (if (*cap).arg != 0 {
            kMarkBeginLine as c_int
        } else {
            0 as c_int
        }) as c_uint) as MarkMove;
    flags = (flags as c_uint
        | (if (*cap).count0 != 0 {
            kMarkSetView as c_int
        } else {
            0 as c_int
        }) as c_uint) as MarkMove;
    let mut fm: *mut fmark_T = mark_get(
        curbuf.get(),
        curwin.get(),
        ::core::ptr::null_mut::<fmark_T>(),
        kMarkAll,
        name,
    );
    move_res = nv_mark_move_to(cap, flags, fm);
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && move_res as c_uint & kMarkMoveSuccess as c_int as c_uint != 0
        && (move_res as c_uint & kMarkSwitchedBuf as c_int as c_uint != 0
            || move_res as c_uint & kMarkChangedCursor as c_int as c_uint != 0)
        && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
        && old_KeyTyped as c_int != 0
    {
        foldOpenCursor();
    }
}

pub(crate) unsafe extern "C" fn nv_pcmark(mut cap: *mut cmdarg_T) {
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut flags: MarkMove = (if jop_flags.get() & kOptJopFlagView as c_int as c_uint != 0 {
        kMarkSetView as c_int
    } else {
        0 as c_int
    }) as MarkMove;
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == TAB && mod_mask.get() == MOD_MASK_CTRL {
        if !goto_tabpage_lastused() {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if (*cap).cmdchar == 'g' as c_int {
        fm = get_changelist(curbuf.get(), curwin.get(), (*cap).count1);
    } else {
        fm = get_jumplist(curwin.get(), (*cap).count1);
        flags = (flags as c_uint | (KMarkNoContext as c_int | kMarkJumpList as c_int) as c_uint)
            as MarkMove;
    }
    if !fm.is_null() {
        move_res = nv_mark_move_to(cap, flags, fm);
    } else if (*cap).cmdchar == 'g' as c_int {
        if (*curbuf.get()).b_changelistlen == 0 as c_int {
            emsg(gettext(e_changelist_is_empty.as_ptr()));
        } else if (*cap).count1 < 0 as c_int {
            emsg(gettext(
                b"E662: At start of changelist\0".as_ptr() as *const c_char
            ));
        } else {
            emsg(gettext(
                b"E663: At end of changelist\0".as_ptr() as *const c_char
            ));
        }
    } else {
        clearopbeep((*cap).oap);
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && (move_res as c_uint & kMarkSwitchedBuf as c_int as c_uint != 0
            || move_res as c_uint & kMarkChangedLine as c_int as c_uint != 0)
        && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
        && old_KeyTyped as c_int != 0
    {
        foldOpenCursor();
    }
}
