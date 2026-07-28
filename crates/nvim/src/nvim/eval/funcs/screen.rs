//! What is on the screen: the `screen*()` cell queries, the `syn*()`
//! syntax queries and the highlight-group lookups.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_hlID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = syn_name2id(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
pub unsafe extern "C" fn f_hlexists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = highlight_exists(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
unsafe extern "C" fn screenchar_adjust(
    mut grid: *mut *mut ScreenGrid,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    msg_scroll_flush();
    *grid = ui_comp_get_grid_at_coord(*row, *col);
    *row -= (**grid).comp_row;
    *col -= (**grid).comp_col;
}
pub unsafe extern "C" fn f_screenattr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    let mut c: ::core::ffi::c_int = 0;
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        c = -1 as ::core::ffi::c_int;
    } else {
        c = *(*grid).attrs.offset(
            (*(*grid).line_offset.offset(row as isize)).wrapping_add(col as size_t) as isize,
        ) as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = c as varnumber_T;
}
pub unsafe extern "C" fn f_screenchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    (*rettv).vval.v_number = (if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        -1 as ::core::ffi::c_int
    } else {
        schar_get_first_codepoint(grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ))
    }) as varnumber_T;
}
pub unsafe extern "C" fn f_screenchars(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        return;
    }
    let mut buf: [::core::ffi::c_char; 33] = [0; 33];
    schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ),
    );
    let mut i: size_t = 0 as size_t;
    loop {
        let mut c: ::core::ffi::c_int =
            utf_ptr2char((&raw mut buf as *mut ::core::ffi::c_char).offset(i as isize));
        tv_list_append_number((*rettv).vval.v_list, c as varnumber_T);
        i = i.wrapping_add(utf_ptr2len(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(i as isize),
        ) as size_t);
        if buf[i as usize] as ::core::ffi::c_int == NUL {
            break;
        }
    }
}
pub unsafe extern "C" fn f_screencol(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (ui_current_col() + 1 as ::core::ffi::c_int) as varnumber_T;
}
pub unsafe extern "C" fn f_screenrow(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (ui_current_row() + 1 as ::core::ffi::c_int) as varnumber_T;
}
pub unsafe extern "C" fn f_screenstring(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut row: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    screenchar_adjust(&raw mut grid, &raw mut row, &raw mut col);
    if row < 0 as ::core::ffi::c_int
        || row >= (*grid).rows
        || col < 0 as ::core::ffi::c_int
        || col >= (*grid).cols
    {
        return;
    }
    let mut buf: [::core::ffi::c_char; 33] = [0; 33];
    schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        grid_getchar(
            grid,
            row,
            col,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ),
    );
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_synID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    let mut transerr: bool = false_0 != 0;
    let trans: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut transerr,
    ) as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !transerr
        && lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col < ml_get_len(lnum)
    {
        id = syn_get_id(
            curwin.get(),
            lnum,
            col,
            trans,
            ::core::ptr::null_mut::<bool>(),
            false_0,
        );
    }
    (*rettv).vval.v_number = id as varnumber_T;
}
pub unsafe extern "C" fn f_synIDattr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let what: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut modec: ::core::ffi::c_int = 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut modebuf: [::core::ffi::c_char; 65] = [0; 65];
        let mode: *const ::core::ffi::c_char = tv_get_string_buf(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut modebuf as *mut ::core::ffi::c_char,
        );
        modec = if (*mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'A' as ::core::ffi::c_int
            || *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'Z' as ::core::ffi::c_int
        {
            *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *mode.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        if modec != 'c' as ::core::ffi::c_int && modec != 'g' as ::core::ffi::c_int {
            modec = 0 as ::core::ffi::c_int;
        }
    } else if ui_rgb_attached() {
        modec = 'g' as ::core::ffi::c_int;
    } else {
        modec = 'c' as ::core::ffi::c_int;
    }
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match if (*what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        < 'A' as ::core::ffi::c_int
        || *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            > 'Z' as ::core::ffi::c_int
    {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        *what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
    } {
        98 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'g' as ::core::ffi::c_int
            {
                p = highlight_color(id, what, modec);
            } else if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'l' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_BLINK as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_BOLD as ::core::ffi::c_int, modec);
            }
        }
        99 => {
            p = highlight_has_attr(id, HL_CONCEALED as ::core::ffi::c_int, modec);
        }
        100 => {
            p = highlight_has_attr(id, HL_DIM as ::core::ffi::c_int, modec);
        }
        111 => {
            p = highlight_has_attr(id, HL_OVERLINE as ::core::ffi::c_int, modec);
        }
        102 => {
            p = highlight_color(id, what, modec);
        }
        105 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'n' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_INVERSE as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_ITALIC as ::core::ffi::c_int, modec);
            }
        }
        110 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'o' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_NOCOMBINE as ::core::ffi::c_int, modec);
            } else {
                p = get_highlight_name_ext(
                    ::core::ptr::null_mut::<expand_T>(),
                    id - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
        }
        114 => {
            p = highlight_has_attr(id, HL_INVERSE as ::core::ffi::c_int, modec);
        }
        115 => {
            if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'p' as ::core::ffi::c_int
            {
                p = highlight_color(id, what, modec);
            } else if (if (*what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'A' as ::core::ffi::c_int
                || *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'Z' as ::core::ffi::c_int
            {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *what.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 't' as ::core::ffi::c_int
                && (if (*what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'r' as ::core::ffi::c_int
            {
                p = highlight_has_attr(id, HL_STRIKETHROUGH as ::core::ffi::c_int, modec);
            } else {
                p = highlight_has_attr(id, HL_STANDOUT as ::core::ffi::c_int, modec);
            }
        }
        117 => {
            if strlen(what) >= 9 as size_t {
                if (if (*what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'l' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERLINE as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) != 'd' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERCURL as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) != 'o' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERDASHED as ::core::ffi::c_int, modec);
                } else if (if (*what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < 'A' as ::core::ffi::c_int
                    || *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        > 'Z' as ::core::ffi::c_int
                {
                    *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *what.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) == 'u' as ::core::ffi::c_int
                {
                    p = highlight_has_attr(id, HL_UNDERDOUBLE as ::core::ffi::c_int, modec);
                } else {
                    p = highlight_has_attr(id, HL_UNDERDOTTED as ::core::ffi::c_int, modec);
                }
            } else {
                p = highlight_color(id, what, modec);
            }
        }
        _ => {}
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = if p.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(p)
    };
}
pub unsafe extern "C" fn f_synIDtrans(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if id > 0 as ::core::ffi::c_int {
        id = syn_get_final_id(id);
    } else {
        id = 0 as ::core::ffi::c_int;
    }
    (*rettv).vval.v_number = id as varnumber_T;
}
pub unsafe extern "C" fn f_synconcealed(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut syntax_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matchid: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut str: [::core::ffi::c_char; 65] = [0; 65];
    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    memset(
        &raw mut str as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
    );
    if lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col <= ml_get_len(lnum)
        && (*curwin.get()).w_onebuf_opt.wo_cole > 0 as OptInt
    {
        syn_get_id(
            curwin.get(),
            lnum,
            col,
            false_0,
            ::core::ptr::null_mut::<bool>(),
            false_0,
        );
        syntax_flags = get_syntax_info(&raw mut matchid);
        if syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0
            && (*curwin.get()).w_onebuf_opt.wo_cole < 3 as OptInt
        {
            let mut cchar: schar_T = schar_from_char(syn_get_sub_char());
            if cchar == NUL as schar_T && (*curwin.get()).w_onebuf_opt.wo_cole == 1 as OptInt {
                cchar = if (*curwin.get()).w_p_lcs_chars.conceal == NUL as schar_T {
                    ' ' as ::core::ffi::c_int as schar_T
                } else {
                    (*curwin.get()).w_p_lcs_chars.conceal
                };
            }
            if cchar != NUL as schar_T {
                schar_get(&raw mut str as *mut ::core::ffi::c_char, cchar);
            }
        }
    }
    tv_list_alloc_ret(rettv, 3 as ptrdiff_t);
    tv_list_append_number(
        (*rettv).vval.v_list,
        (syntax_flags & HL_CONCEAL as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_list_append_string(
        (*rettv).vval.v_list,
        &raw mut str as *mut ::core::ffi::c_char,
        -1 as ssize_t,
    );
    tv_list_append_number((*rettv).vval.v_list, matchid as varnumber_T);
}
pub unsafe extern "C" fn f_synstack(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    let lnum: linenr_T = tv_get_lnum(argvars);
    let col: colnr_T =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as colnr_T - 1 as colnr_T;
    if lnum >= 1 as linenr_T
        && lnum <= (*curbuf.get()).b_ml.ml_line_count
        && col >= 0 as ::core::ffi::c_int
        && col <= ml_get_len(lnum)
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        syn_get_id(
            curwin.get(),
            lnum,
            col,
            false_0,
            ::core::ptr::null_mut::<bool>(),
            true_0,
        );
        let mut id: ::core::ffi::c_int = 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let c2rust_fresh6 = i;
            i = i + 1;
            id = syn_get_stack_item(c2rust_fresh6);
            if id < 0 as ::core::ffi::c_int {
                break;
            }
            tv_list_append_number((*rettv).vval.v_list, id as varnumber_T);
        }
    }
}
