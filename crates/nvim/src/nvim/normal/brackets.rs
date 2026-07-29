//! The `[` and `]` commands: the block, comment, define and method
//! searches, and the paste variants.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_bracket_block(
    mut cap: *mut cmdarg_T,
    mut old_pos: *const pos_T,
) {
    let mut new_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut prev_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut n: c_int = 0;
    let mut findc: c_int = 0;
    if (*cap).nchar == '*' as c_int {
        (*cap).nchar = '/' as c_int;
    }
    prev_pos.lnum = 0 as c_int as linenr_T;
    if (*cap).nchar == 'm' as c_int || (*cap).nchar == 'M' as c_int {
        if (*cap).cmdchar == '[' as c_int {
            findc = '{' as c_int;
        } else {
            findc = '}' as c_int;
        }
        n = 9999 as c_int;
    } else {
        findc = (*cap).nchar;
        n = (*cap).count1;
    }
    while n > 0 as c_int {
        pos = findmatchlimit(
            (*cap).oap,
            findc,
            if (*cap).cmdchar == '[' as c_int {
                FM_BACKWARD as c_int
            } else {
                FM_FORWARD as c_int
            },
            0 as int64_t,
        );
        if pos.is_null() {
            if new_pos.lnum == 0 as linenr_T {
                if (*cap).nchar != 'm' as c_int && (*cap).nchar != 'M' as c_int {
                    clearopbeep((*cap).oap);
                }
            } else {
                pos = &raw mut new_pos;
            }
            break;
        } else {
            prev_pos = new_pos;
            (*curwin.get()).w_cursor = *pos;
            new_pos = *pos;
            n -= 1;
        }
    }
    (*curwin.get()).w_cursor = *old_pos;
    if (*cap).nchar == 'm' as c_int || (*cap).nchar == 'M' as c_int {
        let mut c: c_int = 0;
        let mut norm: bool =
            (findc == '{' as c_int) as c_int == ((*cap).nchar == 'm' as c_int) as c_int;
        n = (*cap).count1;
        if prev_pos.lnum != 0 as linenr_T {
            pos = &raw mut prev_pos;
            (*curwin.get()).w_cursor = prev_pos;
            if norm {
                n -= 1;
            }
        } else {
            pos = ::core::ptr::null_mut::<pos_T>();
        }
        while n > 0 as c_int {
            loop {
                if (if findc == '{' as c_int {
                    dec_cursor()
                } else {
                    inc_cursor()
                }) < 0 as c_int
                {
                    if pos.is_null() {
                        clearopbeep((*cap).oap);
                    }
                    n = 0 as c_int;
                    break;
                } else {
                    c = gchar_cursor();
                    if !(c == '{' as c_int || c == '}' as c_int) {
                        continue;
                    }
                    if c == findc && norm as c_int != 0 || n == 1 as c_int && !norm {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                        n = 0 as c_int;
                    } else if new_pos.lnum == 0 as linenr_T {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                    } else {
                        pos = findmatchlimit(
                            (*cap).oap,
                            findc,
                            if (*cap).cmdchar == '[' as c_int {
                                FM_BACKWARD as c_int
                            } else {
                                FM_FORWARD as c_int
                            },
                            0 as int64_t,
                        );
                        if pos.is_null() {
                            n = 0 as c_int;
                        } else {
                            (*curwin.get()).w_cursor = *pos;
                        }
                    }
                    break;
                }
            }
            n -= 1;
        }
        (*curwin.get()).w_cursor = *old_pos;
        if pos.is_null() && new_pos.lnum != 0 as linenr_T {
            clearopbeep((*cap).oap);
        }
    }
    if !pos.is_null() {
        setpcmark();
        (*curwin.get()).w_cursor = *pos;
        (*curwin.get()).w_set_curswant = true_0;
        if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as c_int
        {
            foldOpenCursor();
        }
    }
}

pub(crate) unsafe extern "C" fn nv_brackets(mut cap: *mut cmdarg_T) {
    let mut flag: c_int = 0;
    let mut n: c_int = 0;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if (*cap).nchar == 'f' as c_int {
        nv_gotofile(cap);
    } else if !vim_strchr(b"iI\tdD\x04\0".as_ptr() as *const c_char, (*cap).nchar).is_null() {
        let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut len: size_t = 0;
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
        if len == 0 as size_t {
            clearop((*cap).oap);
        } else {
            ptr = xmemdupz(ptr as *const c_void, len) as *mut c_char;
            find_pattern_in_path(
                ptr,
                kDirectionNotSet,
                len,
                true_0 != 0,
                if (*cap).count0 == 0 as c_int {
                    (*(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                        & _ISupper as c_int as c_ushort as c_int
                        == 0) as c_int
                } else {
                    false_0
                } != 0,
                if (*cap).nchar & 0xf as c_int == 'd' as c_int & 0xf as c_int {
                    FIND_DEFINE as c_int
                } else {
                    FIND_ANY as c_int
                },
                (*cap).count1,
                if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                    & _ISupper as c_int as c_ushort as c_int
                    != 0
                {
                    ACTION_SHOW_ALL as c_int
                } else if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                    & _ISlower as c_int as c_ushort as c_int
                    != 0
                {
                    ACTION_SHOW as c_int
                } else {
                    ACTION_GOTO as c_int
                },
                if (*cap).cmdchar == ']' as c_int {
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T
                } else {
                    1 as linenr_T
                },
                MAXLNUM as c_int as linenr_T,
                false_0 != 0,
                false_0 != 0,
            );
            xfree(ptr as *mut c_void);
            (*curwin.get()).w_set_curswant = true_0;
        }
    } else if (*cap).cmdchar == '[' as c_int
        && !vim_strchr(b"{(*/#mM\0".as_ptr() as *const c_char, (*cap).nchar).is_null()
        || (*cap).cmdchar == ']' as c_int
            && !vim_strchr(b"})*/#mM\0".as_ptr() as *const c_char, (*cap).nchar).is_null()
    {
        nv_bracket_block(cap, &raw mut old_pos);
    } else if (*cap).nchar == '[' as c_int || (*cap).nchar == ']' as c_int {
        if (*cap).nchar == (*cap).cmdchar {
            flag = '{' as c_int;
        } else {
            flag = '}' as c_int;
        }
        (*curwin.get()).w_set_curswant = true_0;
        if !findpar(
            &raw mut (*(*cap).oap).inclusive,
            (*cap).arg,
            (*cap).count1,
            flag,
            (*(*cap).oap).op_type != OP_NOP as c_int
                && (*cap).arg == FORWARD as c_int
                && flag == '{' as c_int,
        ) {
            clearopbeep((*cap).oap);
        } else {
            if (*(*cap).oap).op_type == OP_NOP as c_int {
                beginline(BL_WHITE as c_int | BL_FIX as c_int);
            }
            if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
                && KeyTyped.get() as c_int != 0
                && (*(*cap).oap).op_type == OP_NOP as c_int
            {
                foldOpenCursor();
            }
        }
    } else if (*cap).nchar == 'p' as c_int || (*cap).nchar == 'P' as c_int {
        nv_put_opt(cap, true_0 != 0);
    } else if (*cap).nchar == '\'' as c_int || (*cap).nchar == '`' as c_int {
        let mut fm: *mut fmark_T = pos_to_mark(
            curbuf.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            (*curwin.get()).w_cursor,
        );
        '_c2rust_label: {
            if !fm.is_null() {
            } else {
                __assert_fail(
                    b"fm != NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    4311 as c_uint,
                    b"void nv_brackets(cmdarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        let mut prev_fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
        n = (*cap).count1;
        while n > 0 as c_int {
            prev_fm = fm;
            fm = getnextmark(
                &raw mut (*fm).mark,
                if (*cap).cmdchar == '[' as c_int {
                    BACKWARD as c_int
                } else {
                    FORWARD as c_int
                },
                ((*cap).nchar == '\'' as c_int) as c_int,
            );
            if fm.is_null() {
                break;
            }
            n -= 1;
        }
        if fm.is_null() {
            fm = prev_fm;
        }
        let mut flags: MarkMove = kMarkContext;
        flags = (flags as c_uint
            | (if (*cap).nchar == '\'' as c_int {
                kMarkBeginLine as c_int
            } else {
                0 as c_int
            }) as c_uint) as MarkMove;
        nv_mark_move_to(cap, flags, fm);
    } else if (*cap).nchar >= -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int))
        && (*cap).nchar <= -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int))
    {
        do_mouse(
            (*cap).oap,
            (*cap).nchar,
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
            PUT_FIXINDENT as c_int != 0,
        );
    } else if (*cap).nchar == 'z' as c_int {
        if foldMoveTo(
            false_0 != 0,
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'c' as c_int {
        if diff_move_to(
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'r' as c_int
        || (*cap).nchar == 's' as c_int
        || (*cap).nchar == 'S' as c_int
    {
        setpcmark();
        n = 0 as c_int;
        while n < (*cap).count1 {
            if spell_move_to(
                curwin.get(),
                if (*cap).cmdchar == ']' as c_int {
                    FORWARD as c_int
                } else {
                    BACKWARD as c_int
                },
                (if (*cap).nchar == 's' as c_int {
                    SMT_ALL as c_int
                } else {
                    if (*cap).nchar == 'r' as c_int {
                        SMT_RARE as c_int
                    } else {
                        SMT_BAD as c_int
                    }
                }) as smt_T,
                false_0 != 0,
                ::core::ptr::null_mut::<hlf_T>(),
            ) == 0 as size_t
            {
                clearopbeep((*cap).oap);
                break;
            } else {
                (*curwin.get()).w_set_curswant = true_0;
                n += 1;
            }
        }
        if (*(*cap).oap).op_type == OP_NOP as c_int
            && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
        {
            foldOpenCursor();
        }
    } else {
        clearopbeep((*cap).oap);
    };
}
