//! Turning an expression into a buffer position.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn buf_byteidx_to_charidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut byteidx: c_int,
) -> c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut c_char = ml_get_buf(buf, lnum);
    if *str as c_int == NUL {
        return 0 as c_int;
    }
    let mut t: *mut c_char = str;
    let mut count: c_int = 0;
    count = 0 as c_int;
    while *t as c_int != NUL && t <= str.offset(byteidx as isize) {
        t = t.offset(utfc_ptr2len(t) as isize);
        count += 1;
    }
    if *t as c_int == NUL && byteidx != 0 as c_int && t == str.offset(byteidx as isize) {
        count += 1;
    }
    return count - 1 as c_int;
}

pub unsafe extern "C" fn buf_charidx_to_byteidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut charidx: c_int,
) -> c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut c_char = ml_get_buf(buf, lnum);
    let mut t: *mut c_char = str;
    while *t as c_int != NUL && {
        charidx -= 1;
        charidx > 0 as c_int
    } {
        t = t.offset(utfc_ptr2len(t) as isize);
    }
    return t.offset_from(str) as c_int;
}

pub unsafe extern "C" fn var2fpos(
    tv: *const typval_T,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
    mut wp: *mut win_T,
) -> *mut pos_T {
    static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut bp: *mut buf_T = (*wp).w_buffer;
    if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        let mut error: bool = false_0 != 0;
        let mut l: *mut list_T = (*tv).vval.v_list;
        if l.is_null() {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).lnum = tv_list_find_nr(l, 0 as c_int, &raw mut error) as linenr_T;
        if error as c_int != 0
            || (*pos.ptr()).lnum <= 0 as linenr_T
            || (*pos.ptr()).lnum > (*bp).b_ml.ml_line_count
        {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).col = tv_list_find_nr(l, 1 as c_int, &raw mut error) as colnr_T;
        if error {
            return ::core::ptr::null_mut::<pos_T>();
        }
        let mut len: c_int = 0;
        if charcol {
            len = mb_charlen(ml_get_buf(bp, (*pos.ptr()).lnum));
        } else {
            len = ml_get_buf_len(bp, (*pos.ptr()).lnum) as c_int;
        }
        let mut li: *mut listitem_T = tv_list_find(l, 1 as c_int);
        if !li.is_null()
            && (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint
            && !(*li).li_tv.vval.v_string.is_null()
            && strcmp((*li).li_tv.vval.v_string, b"$\0".as_ptr() as *const c_char) == 0 as c_int
        {
            (*pos.ptr()).col = (len + 1 as c_int) as colnr_T;
        }
        if (*pos.ptr()).col == 0 as c_int || (*pos.ptr()).col > len + 1 as c_int {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).col -= 1;
        (*pos.ptr()).coladd = tv_list_find_nr(l, 2 as c_int, &raw mut error) as colnr_T;
        if error {
            (*pos.ptr()).coladd = 0 as c_int as colnr_T;
        }
        return pos.ptr();
    }
    let name: *const c_char = tv_get_string_chk(tv);
    if name.is_null() {
        return ::core::ptr::null_mut::<pos_T>();
    }
    (*pos.ptr()).lnum = 0 as c_int as linenr_T;
    if *name.offset(0 as c_int as isize) as c_int == '.' as c_int {
        pos.set((*wp).w_cursor);
    } else if *name.offset(0 as c_int as isize) as c_int == 'v' as c_int
        && *name.offset(1 as c_int as isize) as c_int == NUL
    {
        if VIsual_active.get() as c_int != 0 && wp == curwin.get() {
            pos.set(VIsual.get());
        } else {
            pos.set((*wp).w_cursor);
        }
    } else if *name.offset(0 as c_int as isize) as c_int == '\'' as c_int {
        let mut mname: c_int = *name.offset(1 as c_int as isize) as uint8_t as c_int;
        let fm: *const fmark_T =
            mark_get(bp, wp, ::core::ptr::null_mut::<fmark_T>(), kMarkAll, mname);
        if fm.is_null() || (*fm).mark.lnum <= 0 as linenr_T {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos.set((*fm).mark);
        *ret_fnum = if mname as c_uint >= 'A' as c_uint && mname as c_uint <= 'Z' as c_uint
            || ascii_isdigit(mname) as c_int != 0
        {
            (*fm).fnum
        } else {
            *ret_fnum
        };
    }
    if (*pos.ptr()).lnum != 0 as linenr_T {
        if charcol {
            (*pos.ptr()).col =
                buf_byteidx_to_charidx(bp, (*pos.ptr()).lnum, (*pos.ptr()).col as c_int) as colnr_T;
        }
        return pos.ptr();
    }
    (*pos.ptr()).coladd = 0 as c_int as colnr_T;
    if *name.offset(0 as c_int as isize) as c_int == 'w' as c_int && dollar_lnum as c_int != 0 {
        check_cursor_moved(wp);
        (*pos.ptr()).col = 0 as c_int as colnr_T;
        if *name.offset(1 as c_int as isize) as c_int == '0' as c_int {
            update_topline(wp);
            (*pos.ptr()).lnum = if (*wp).w_topline > 0 as linenr_T {
                (*wp).w_topline
            } else {
                1 as linenr_T
            };
            return pos.ptr();
        } else if *name.offset(1 as c_int as isize) as c_int == '$' as c_int {
            validate_botline_win(wp);
            (*pos.ptr()).lnum = if (*wp).w_botline > 0 as linenr_T {
                (*wp).w_botline - 1 as linenr_T
            } else {
                0 as linenr_T
            };
            return pos.ptr();
        }
    } else if *name.offset(0 as c_int as isize) as c_int == '$' as c_int {
        if dollar_lnum {
            (*pos.ptr()).lnum = (*bp).b_ml.ml_line_count;
            (*pos.ptr()).col = 0 as c_int as colnr_T;
        } else {
            (*pos.ptr()).lnum = (*wp).w_cursor.lnum;
            if charcol {
                (*pos.ptr()).col = mb_charlen(ml_get_buf(bp, (*wp).w_cursor.lnum));
            } else {
                (*pos.ptr()).col = ml_get_buf_len(bp, (*wp).w_cursor.lnum);
            }
        }
        return pos.ptr();
    }
    return ::core::ptr::null_mut::<pos_T>();
}

pub unsafe extern "C" fn list2fpos(
    mut arg: *mut typval_T,
    mut posp: *mut pos_T,
    mut fnump: *mut c_int,
    mut curswantp: *mut colnr_T,
    mut charcol: bool,
) -> c_int {
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if (*arg).v_type as c_uint != VAR_LIST as c_int as c_uint
        || {
            l = (*arg).vval.v_list;
            l.is_null()
        }
        || tv_list_len(l)
            < (if fnump.is_null() {
                2 as c_int
            } else {
                3 as c_int
            })
        || tv_list_len(l)
            > (if fnump.is_null() {
                4 as c_int
            } else {
                5 as c_int
            })
    {
        return FAIL;
    }
    let mut i: c_int = 0 as c_int;
    let mut n: c_int = 0;
    if !fnump.is_null() {
        let c2rust_fresh18 = i;
        i = i + 1;
        n = tv_list_find_nr(l, c2rust_fresh18, ::core::ptr::null_mut::<bool>()) as c_int;
        if n < 0 as c_int {
            return FAIL;
        }
        if n == 0 as c_int {
            n = (*curbuf.get()).handle as c_int;
        }
        *fnump = n;
    }
    let c2rust_fresh19 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh19, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        return FAIL;
    }
    (*posp).lnum = n as linenr_T;
    let c2rust_fresh20 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh20, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        return FAIL;
    }
    if charcol {
        let mut buf: *mut buf_T = buflist_findnr(if fnump.is_null() {
            (*curbuf.get()).handle as c_int
        } else {
            *fnump
        });
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return FAIL;
        }
        n = buf_charidx_to_byteidx(
            buf,
            if (*posp).lnum == 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*posp).lnum
            },
            n,
        ) + 1 as c_int;
    }
    (*posp).col = n as colnr_T;
    n = tv_list_find_nr(l, i, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        (*posp).coladd = 0 as c_int as colnr_T;
    } else {
        (*posp).coladd = n as colnr_T;
    }
    if !curswantp.is_null() {
        *curswantp = tv_list_find_nr(l, i + 1 as c_int, ::core::ptr::null_mut::<bool>()) as colnr_T;
    }
    return OK;
}
