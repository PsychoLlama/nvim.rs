//! `open_line` -- the new line `o`, `O`, `<CR>` and a wrap all make.
//!
//! Still one 1,001-line function here, and still over the file cap: a carve
//! cannot split a single over-cap item, so this file is a holding pen until the
//! rewrite decomposes it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn open_line(
    mut dir: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut second_line_indent: ::core::ffi::c_int,
    mut did_do_comment: *mut bool,
) -> bool {
    unsafe {
        let mut next_line: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p_extra: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut less_cols: colnr_T = 0 as colnr_T;
        let mut less_cols_off: colnr_T = 0 as colnr_T;
        let mut old_cursor: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut newcol: colnr_T = 0 as colnr_T;
        let mut newindent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut trunc_line: bool = false_0 != 0;
        let mut retval: bool = false_0 != 0;
        let mut extra_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lead_len: ::core::ffi::c_int = 0;
        let mut comment_start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lead_flags: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut leader: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut allocated: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut saved_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut do_si: bool = may_do_si();
        let mut no_si: bool = false_0 != 0;
        let mut first_char: ::core::ffi::c_int = NUL;
        let mut vreplace_mode: ::core::ffi::c_int = 0;
        let mut did_append: bool = false;
        let mut saved_pi: ::core::ffi::c_int = (*curbuf.get()).b_p_pi;
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut mincol: colnr_T = (*curwin.get()).w_cursor.col + 1 as colnr_T;
        let mut saved_line: *mut ::core::ffi::c_char =
            xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
        if State.get() & VREPLACE_FLAG != 0 {
            if (*curwin.get()).w_cursor.lnum < orig_line_count.get() {
                next_line = xstrnsave(
                    ml_get((*curwin.get()).w_cursor.lnum + 1 as linenr_T),
                    ml_get_len((*curwin.get()).w_cursor.lnum + 1 as linenr_T) as size_t,
                );
            } else {
                next_line = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
            }
            replace_push_nul();
            replace_push_nul();
            p = saved_line.offset((*curwin.get()).w_cursor.col as isize);
            replace_push(p, strlen(p));
            *saved_line.offset((*curwin.get()).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
        }
        if State.get() & MODE_INSERT != 0 && State.get() & VREPLACE_FLAG == 0 as ::core::ffi::c_int
        {
            p_extra = saved_line.offset((*curwin.get()).w_cursor.col as isize);
            if do_si {
                p = skipwhite(p_extra);
                first_char = *p as ::core::ffi::c_uchar as ::core::ffi::c_int;
            }
            extra_len = strlen(p_extra) as ::core::ffi::c_int;
            saved_char = *p_extra;
            *p_extra = NUL as ::core::ffi::c_char;
        }
        u_clearline(curbuf.get());
        did_si.set(false_0 != 0);
        ai_col.set(0 as ::core::ffi::c_int as colnr_T);
        if dir == FORWARD as ::core::ffi::c_int && did_ai.get() as ::core::ffi::c_int != 0 {
            trunc_line = true_0 != 0;
        }
        if flags & OPENLINE_FORCE_INDENT as ::core::ffi::c_int != 0 {
            newindent = second_line_indent;
        } else if (*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0 {
            newindent = indent_size_ts(
                saved_line,
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
            if newindent == 0 as ::core::ffi::c_int
                && flags & OPENLINE_COM_LIST as ::core::ffi::c_int == 0
            {
                newindent = second_line_indent;
            }
            if !trunc_line
                && do_si as ::core::ffi::c_int != 0
                && *saved_line as ::core::ffi::c_int != NUL
                && (p_extra.is_null() || first_char != '{' as ::core::ffi::c_int)
            {
                old_cursor = (*curwin.get()).w_cursor;
                let mut ptr: *mut ::core::ffi::c_char = saved_line;
                if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
                    lead_len = get_leader_len(
                        ptr,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        true_0 != 0,
                    );
                } else {
                    lead_len = 0 as ::core::ffi::c_int;
                }
                if dir == FORWARD as ::core::ffi::c_int {
                    if lead_len == 0 as ::core::ffi::c_int
                        && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '#' as ::core::ffi::c_int
                    {
                        while *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '#' as ::core::ffi::c_int
                            && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                        {
                            (*curwin.get()).w_cursor.lnum -= 1;
                            ptr = ml_get((*curwin.get()).w_cursor.lnum);
                        }
                        newindent = get_indent();
                    }
                    if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
                        lead_len = get_leader_len(
                            ptr,
                            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            false_0 != 0,
                            true_0 != 0,
                        );
                    } else {
                        lead_len = 0 as ::core::ffi::c_int;
                    }
                    if lead_len > 0 as ::core::ffi::c_int {
                        p = skipwhite(ptr);
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '*' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                        }
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                            while *p != 0 {
                                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    == '/' as ::core::ffi::c_int
                                    && *p.offset(-1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '*' as ::core::ffi::c_int
                                {
                                    (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
                                    pos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
                                    if !pos.is_null() {
                                        (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                                        newindent = get_indent();
                                        break;
                                    } else {
                                        ptr = ml_get((*curwin.get()).w_cursor.lnum);
                                        p = ptr.offset((*curwin.get()).w_cursor.col as isize);
                                    }
                                }
                                p = p.offset(1);
                            }
                        }
                    } else {
                        p = ptr
                            .offset(strlen(ptr) as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize));
                        while p > ptr
                            && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            p = p.offset(-1);
                        }
                        let mut last_char: ::core::ffi::c_char = *p;
                        if last_char as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                            || last_char as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                        {
                            if p > ptr {
                                p = p.offset(-1);
                            }
                            while p > ptr
                                && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(-1);
                            }
                        }
                        if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
                            pos = findmatch(
                                ::core::ptr::null_mut::<oparg_T>(),
                                '(' as ::core::ffi::c_int,
                            );
                            if !pos.is_null() {
                                (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                                newindent = get_indent();
                                ptr = get_cursor_line_ptr();
                            }
                        }
                        if last_char as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                            did_si.set(true_0 != 0);
                            no_si = true_0 != 0;
                        } else if last_char as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                            && last_char as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                            && cin_is_cinword(ptr) as ::core::ffi::c_int != 0
                        {
                            did_si.set(true_0 != 0);
                        }
                    }
                } else {
                    if lead_len == 0 as ::core::ffi::c_int
                        && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '#' as ::core::ffi::c_int
                    {
                        let mut was_backslashed: bool = false_0 != 0;
                        while (*ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '#' as ::core::ffi::c_int
                            || was_backslashed as ::core::ffi::c_int != 0)
                            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
                        {
                            if *ptr as ::core::ffi::c_int != 0
                                && *ptr.offset(strlen(ptr).wrapping_sub(1 as size_t) as isize)
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                            {
                                was_backslashed = true_0 != 0;
                            } else {
                                was_backslashed = false_0 != 0;
                            }
                            (*curwin.get()).w_cursor.lnum += 1;
                            ptr = ml_get((*curwin.get()).w_cursor.lnum);
                        }
                        if was_backslashed {
                            newindent = 0 as ::core::ffi::c_int;
                        } else {
                            newindent = get_indent();
                        }
                    }
                    p = skipwhite(ptr);
                    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        did_si.set(true_0 != 0);
                    } else {
                        can_si_back.set(true_0 != 0);
                    }
                }
                (*curwin.get()).w_cursor = old_cursor;
            }
            if do_si {
                can_si.set(true_0 != 0);
            }
            did_ai.set(true_0 != 0);
        }
        let mut do_cindent: bool = p_paste.get() == 0
            && ((*curbuf.get()).b_p_cin != 0
                || *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL)
            && in_cinkeys(
                if dir == FORWARD as ::core::ffi::c_int {
                    KEY_OPEN_FORW as ::core::ffi::c_int
                } else {
                    KEY_OPEN_BACK as ::core::ffi::c_int
                },
                ' ' as ::core::ffi::c_int,
                linewhite((*curwin.get()).w_cursor.lnum),
            ) as ::core::ffi::c_int
                != 0
            && flags & OPENLINE_FORCE_INDENT as ::core::ffi::c_int == 0;
        end_comment_pending.set(NUL);
        if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
            lead_len = get_leader_len(
                saved_line,
                &raw mut lead_flags,
                dir == BACKWARD as ::core::ffi::c_int,
                true_0 != 0,
            );
            if lead_len == 0 as ::core::ffi::c_int
                && (*curbuf.get()).b_p_cin != 0
                && do_cindent as ::core::ffi::c_int != 0
                && dir == FORWARD as ::core::ffi::c_int
                && (!has_format_option(FO_NO_OPEN_COMS)
                    || flags & OPENLINE_FORMAT as ::core::ffi::c_int != 0)
            {
                comment_start = check_linecomment(saved_line);
                if comment_start != MAXCOL as ::core::ffi::c_int {
                    lead_len = get_leader_len(
                        saved_line.offset(comment_start as isize),
                        &raw mut lead_flags,
                        false_0 != 0,
                        true_0 != 0,
                    );
                    if lead_len != 0 as ::core::ffi::c_int {
                        lead_len += comment_start;
                        if !did_do_comment.is_null() {
                            *did_do_comment = true_0 != 0;
                        }
                    }
                }
            }
        } else {
            lead_len = 0 as ::core::ffi::c_int;
        }
        if lead_len > 0 as ::core::ffi::c_int {
            let mut lead_repl: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut lead_repl_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut lead_middle: [::core::ffi::c_char; 50] = [0; 50];
            let mut lead_middle_len: ::core::ffi::c_int = 0;
            let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
            let mut comment_end: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut extra_space: ::core::ffi::c_int = false_0;
            let mut require_blank: bool = false_0 != 0;
            let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = lead_flags;
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            {
                if *p as ::core::ffi::c_int == COM_BLANK {
                    require_blank = true_0 != 0;
                } else if *p as ::core::ffi::c_int == COM_START
                    || *p as ::core::ffi::c_int == COM_MIDDLE
                {
                    let mut current_flag: ::core::ffi::c_int =
                        *p as ::core::ffi::c_uchar as ::core::ffi::c_int;
                    if *p as ::core::ffi::c_int == COM_START {
                        if dir == BACKWARD as ::core::ffi::c_int {
                            lead_len = 0 as ::core::ffi::c_int;
                            break;
                        } else {
                            copy_option_part(
                                &raw mut p,
                                &raw mut lead_middle as *mut ::core::ffi::c_char,
                                COM_MAX_LEN as size_t,
                                b",\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                            require_blank = false_0 != 0;
                        }
                    }
                    while *p as ::core::ffi::c_int != 0
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != ':' as ::core::ffi::c_int
                    {
                        if *p as ::core::ffi::c_int == COM_BLANK {
                            require_blank = true_0 != 0;
                        }
                        p = p.offset(1);
                    }
                    lead_middle_len = copy_option_part(
                        &raw mut p,
                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                        COM_MAX_LEN as size_t,
                        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    while *p as ::core::ffi::c_int != 0
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != ':' as ::core::ffi::c_int
                    {
                        if *p as ::core::ffi::c_int == COM_AUTO_END {
                            end_comment_pending.set(-1 as ::core::ffi::c_int);
                        }
                        p = p.offset(1);
                    }
                    let mut n: size_t = copy_option_part(
                        &raw mut p,
                        &raw mut lead_end as *mut ::core::ffi::c_char,
                        COM_MAX_LEN as size_t,
                        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    if end_comment_pending.get() == -1 as ::core::ffi::c_int {
                        end_comment_pending.set(
                            lead_end[n.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_uchar
                                as ::core::ffi::c_int,
                        );
                    }
                    if dir == FORWARD as ::core::ffi::c_int {
                        p = saved_line.offset(lead_len as isize);
                        while *p != 0 {
                            if strncmp(p, &raw mut lead_end as *mut ::core::ffi::c_char, n)
                                == 0 as ::core::ffi::c_int
                            {
                                comment_end = p;
                                lead_len = 0 as ::core::ffi::c_int;
                                break;
                            } else {
                                p = p.offset(1);
                            }
                        }
                    }
                    if lead_len > 0 as ::core::ffi::c_int {
                        if current_flag == COM_START {
                            lead_repl = &raw mut lead_middle as *mut ::core::ffi::c_char;
                            lead_repl_len = lead_middle_len;
                        }
                        if !ascii_iswhite(
                            *saved_line.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                        ) && (!p_extra.is_null() && (*curwin.get()).w_cursor.col == lead_len
                            || p_extra.is_null()
                                && *saved_line.offset(lead_len as isize) as ::core::ffi::c_int
                                    == NUL
                            || require_blank as ::core::ffi::c_int != 0)
                        {
                            extra_space = true_0;
                        }
                    }
                    break;
                } else if *p as ::core::ffi::c_int == COM_END {
                    if dir == FORWARD as ::core::ffi::c_int {
                        comment_end = skipwhite(saved_line);
                        lead_len = 0 as ::core::ffi::c_int;
                        break;
                    } else {
                        while p > (*curbuf.get()).b_p_com
                            && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        {
                            p = p.offset(-1);
                        }
                        lead_repl = p;
                        while lead_repl > (*curbuf.get()).b_p_com
                            && *lead_repl.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                != ':' as ::core::ffi::c_int
                        {
                            lead_repl = lead_repl.offset(-1);
                        }
                        lead_repl_len = p.offset_from(lead_repl) as ::core::ffi::c_int;
                        extra_space = true_0;
                        p2 = p;
                        while *p2 as ::core::ffi::c_int != 0
                            && *p2 as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                        {
                            if *p2 as ::core::ffi::c_int == COM_AUTO_END {
                                end_comment_pending.set(-1 as ::core::ffi::c_int);
                            }
                            p2 = p2.offset(1);
                        }
                        if end_comment_pending.get() == -1 as ::core::ffi::c_int {
                            while *p2 as ::core::ffi::c_int != 0
                                && *p2 as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                            {
                                p2 = p2.offset(1);
                            }
                            end_comment_pending.set(*p2.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_uchar
                                as ::core::ffi::c_int);
                        }
                        break;
                    }
                } else if *p as ::core::ffi::c_int == COM_FIRST {
                    if dir == BACKWARD as ::core::ffi::c_int {
                        lead_len = 0 as ::core::ffi::c_int;
                    } else {
                        lead_repl = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        lead_repl_len = 0 as ::core::ffi::c_int;
                    }
                    break;
                }
                p = p.offset(1);
            }
            if lead_len > 0 as ::core::ffi::c_int {
                let mut bytes: ::core::ffi::c_int = lead_len
                    + lead_repl_len
                    + extra_space
                    + extra_len
                    + (if second_line_indent > 0 as ::core::ffi::c_int {
                        second_line_indent
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + 1 as ::core::ffi::c_int;
                '_c2rust_label: {
                    if bytes >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"bytes >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/change.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1386 as ::core::ffi::c_uint,
                            b"_Bool open_line(int, int, int, _Bool *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                leader = xmalloc(bytes as size_t) as *mut ::core::ffi::c_char;
                allocated = leader;
                xmemcpyz(
                    leader as *mut ::core::ffi::c_void,
                    saved_line as *const ::core::ffi::c_void,
                    lead_len as size_t,
                );
                let mut li: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while li < comment_start {
                    if !ascii_iswhite(*leader.offset(li as isize) as ::core::ffi::c_int) {
                        *leader.offset(li as isize) = ' ' as ::core::ffi::c_char;
                    }
                    li += 1;
                }
                if !lead_repl.is_null() {
                    let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    p = lead_flags;
                    while *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                    {
                        if *p as ::core::ffi::c_int == COM_RIGHT
                            || *p as ::core::ffi::c_int == COM_LEFT
                        {
                            let c2rust_fresh0 = p;
                            p = p.offset(1);
                            c = *c2rust_fresh0 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                        } else if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                        {
                            off = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
                        } else {
                            p = p.offset(1);
                        }
                    }
                    if c == COM_RIGHT {
                        p = leader
                            .offset(lead_len as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize));
                        while p > leader
                            && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            p = p.offset(-1);
                        }
                        p = p.offset(1);
                        let mut repl_size: ::core::ffi::c_int =
                            vim_strnsize(lead_repl, lead_repl_len);
                        let mut old_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut endp: *mut ::core::ffi::c_char = p;
                        while old_size < repl_size && p > leader {
                            p = p.offset(
                                -((utf_head_off(
                                    leader,
                                    p.offset(-(1 as ::core::ffi::c_int as isize)),
                                ) + 1 as ::core::ffi::c_int)
                                    as isize),
                            );
                            old_size += ptr2cells(p);
                        }
                        let mut l: ::core::ffi::c_int =
                            lead_repl_len - endp.offset_from(p) as ::core::ffi::c_int;
                        if l != 0 as ::core::ffi::c_int {
                            memmove(
                                endp.offset(l as isize) as *mut ::core::ffi::c_void,
                                endp as *const ::core::ffi::c_void,
                                leader.offset(lead_len as isize).offset_from(endp) as size_t,
                            );
                        }
                        lead_len += l;
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            lead_repl as *const ::core::ffi::c_void,
                            lead_repl_len as size_t,
                        );
                        if p.offset(lead_repl_len as isize) > leader.offset(lead_len as isize) {
                            *p.offset(lead_repl_len as isize) = NUL as ::core::ffi::c_char;
                        }
                        loop {
                            p = p.offset(-1);
                            if p < leader {
                                break;
                            }
                            let mut l_0: ::core::ffi::c_int = utf_head_off(leader, p);
                            if l_0 > 1 as ::core::ffi::c_int {
                                p = p.offset(-(l_0 as isize));
                                if ptr2cells(p) > 1 as ::core::ffi::c_int {
                                    *p.offset(1 as ::core::ffi::c_int as isize) =
                                        ' ' as ::core::ffi::c_char;
                                    l_0 -= 1;
                                }
                                memmove(
                                    p.offset(1 as ::core::ffi::c_int as isize)
                                        as *mut ::core::ffi::c_void,
                                    p.offset(l_0 as isize)
                                        .offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    leader.offset(lead_len as isize).offset_from(
                                        p.offset(l_0 as isize)
                                            .offset(1 as ::core::ffi::c_int as isize),
                                    ) as size_t,
                                );
                                lead_len -= l_0;
                                *p = ' ' as ::core::ffi::c_char;
                            } else if !ascii_iswhite(*p as ::core::ffi::c_int) {
                                *p = ' ' as ::core::ffi::c_char;
                            }
                        }
                    } else {
                        p = skipwhite(leader);
                        let mut repl_size_0: ::core::ffi::c_int =
                            vim_strnsize(lead_repl, lead_repl_len);
                        let mut i: ::core::ffi::c_int = 0;
                        let mut l_1: ::core::ffi::c_int = 0;
                        i = 0 as ::core::ffi::c_int;
                        while i < lead_len && *p.offset(i as isize) as ::core::ffi::c_int != NUL {
                            l_1 = utfc_ptr2len(p.offset(i as isize));
                            if vim_strnsize(p, i + l_1) > repl_size_0 {
                                break;
                            }
                            i += l_1;
                        }
                        if i != lead_repl_len {
                            memmove(
                                p.offset(lead_repl_len as isize) as *mut ::core::ffi::c_void,
                                p.offset(i as isize) as *const ::core::ffi::c_void,
                                ((lead_len - i) as isize - p.offset_from(leader)) as size_t,
                            );
                            lead_len += lead_repl_len - i;
                        }
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            lead_repl as *const ::core::ffi::c_void,
                            lead_repl_len as size_t,
                        );
                        p = p.offset(lead_repl_len as isize);
                        while p < leader.offset(lead_len as isize) {
                            if !ascii_iswhite(*p as ::core::ffi::c_int) {
                                if p.offset(1 as ::core::ffi::c_int as isize)
                                    < leader.offset(lead_len as isize)
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == TAB
                                {
                                    lead_len -= 1;
                                    memmove(
                                        p as *mut ::core::ffi::c_void,
                                        p.offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        leader.offset(lead_len as isize).offset_from(p) as size_t,
                                    );
                                } else {
                                    let mut l_2: ::core::ffi::c_int = utfc_ptr2len(p);
                                    if l_2 > 1 as ::core::ffi::c_int {
                                        if ptr2cells(p) > 1 as ::core::ffi::c_int {
                                            l_2 -= 1;
                                            let c2rust_fresh1 = p;
                                            p = p.offset(1);
                                            *c2rust_fresh1 = ' ' as ::core::ffi::c_char;
                                        }
                                        memmove(
                                            p.offset(1 as ::core::ffi::c_int as isize)
                                                as *mut ::core::ffi::c_void,
                                            p.offset(l_2 as isize) as *const ::core::ffi::c_void,
                                            leader.offset(lead_len as isize).offset_from(p)
                                                as size_t,
                                        );
                                        lead_len -= l_2 - 1 as ::core::ffi::c_int;
                                    }
                                    *p = ' ' as ::core::ffi::c_char;
                                }
                            }
                            p = p.offset(1);
                        }
                        *p = NUL as ::core::ffi::c_char;
                    }
                    if (*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0 {
                        newindent = indent_size_ts(
                            leader,
                            (*curbuf.get()).b_p_ts,
                            (*curbuf.get()).b_p_vts_array,
                        );
                    }
                    if newindent + off < 0 as ::core::ffi::c_int {
                        off = -newindent;
                        newindent = 0 as ::core::ffi::c_int;
                    } else {
                        newindent += off;
                    }
                    while off > 0 as ::core::ffi::c_int
                        && lead_len > 0 as ::core::ffi::c_int
                        && *leader.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        if !vim_strchr(skipwhite(leader), '\t' as ::core::ffi::c_int).is_null() {
                            break;
                        }
                        lead_len -= 1;
                        off -= 1;
                    }
                    if lead_len > 0 as ::core::ffi::c_int
                        && ascii_iswhite(
                            *leader.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        extra_space = false_0;
                    }
                    *leader.offset(lead_len as isize) = NUL as ::core::ffi::c_char;
                }
                if extra_space != 0 {
                    let c2rust_fresh2 = lead_len;
                    lead_len = lead_len + 1;
                    *leader.offset(c2rust_fresh2 as isize) = ' ' as ::core::ffi::c_char;
                    *leader.offset(lead_len as isize) = NUL as ::core::ffi::c_char;
                }
                newcol = lead_len as colnr_T;
                if newindent != 0 || did_si.get() as ::core::ffi::c_int != 0 {
                    while lead_len != 0
                        && ascii_iswhite(*leader as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        lead_len -= 1;
                        newcol -= 1;
                        leader = leader.offset(1);
                    }
                }
                can_si.set(false_0 != 0);
                did_si.set(can_si.get());
            } else if !comment_end.is_null() {
                if *comment_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                    && *comment_end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                    && ((*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0)
                {
                    old_cursor = (*curwin.get()).w_cursor;
                    (*curwin.get()).w_cursor.col = comment_end.offset_from(saved_line) as colnr_T;
                    pos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
                    if !pos.is_null() {
                        (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                        newindent = get_indent();
                    }
                    (*curwin.get()).w_cursor = old_cursor;
                }
            }
        }
        if !p_extra.is_null() {
            *p_extra = saved_char;
            if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                replace_push_nul();
            }
            if (*curbuf.get()).b_p_ai != 0 || flags & OPENLINE_DELSPACES as ::core::ffi::c_int != 0
            {
                while (*p_extra as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                    || *p_extra as ::core::ffi::c_int == '\t' as ::core::ffi::c_int)
                    && !utf_iscomposing_first(utf_ptr2char(
                        p_extra.offset(1 as ::core::ffi::c_int as isize),
                    ))
                {
                    if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                        replace_push(p_extra, 1 as size_t);
                    }
                    p_extra = p_extra.offset(1);
                    less_cols_off += 1;
                }
            }
            less_cols = p_extra.offset_from(saved_line) as ::core::ffi::c_int as colnr_T;
        }
        if p_extra.is_null() {
            p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if lead_len > 0 as ::core::ffi::c_int {
            if flags & OPENLINE_COM_LIST as ::core::ffi::c_int != 0
                && second_line_indent > 0 as ::core::ffi::c_int
            {
                let mut padding: ::core::ffi::c_int =
                    second_line_indent - (newindent + strlen(leader) as ::core::ffi::c_int);
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < padding {
                    strcat(leader, b" \0".as_ptr() as *const ::core::ffi::c_char);
                    less_cols -= 1;
                    newcol += 1;
                    i_0 += 1;
                }
            }
            strcat(leader, p_extra);
            p_extra = leader;
            did_ai.set(true_0 != 0);
            less_cols -= lead_len;
        } else {
            end_comment_pending.set(NUL);
        }
        (*curbuf_splice_pending.ptr()) += 1;
        old_cursor = (*curwin.get()).w_cursor;
        let mut old_cmod_flags: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_flags;
        let mut prompt_moved: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        if dir == BACKWARD as ::core::ffi::c_int {
            if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
                && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum
            {
                let mut prompt_line: *mut ::core::ffi::c_char =
                    ml_get((*curwin.get()).w_cursor.lnum);
                let mut prompt: *mut ::core::ffi::c_char = prompt_text();
                let mut prompt_len: size_t = strlen(prompt);
                if strncmp(prompt_line, prompt, prompt_len) == 0 as ::core::ffi::c_int {
                    memmove(
                        prompt_line as *mut ::core::ffi::c_void,
                        prompt_line.offset(prompt_len as isize) as *const ::core::ffi::c_void,
                        strlen(prompt_line.offset(prompt_len as isize)).wrapping_add(1 as size_t),
                    );
                    (*cmdmod.ptr()).cmod_flags =
                        (*cmdmod.ptr()).cmod_flags | CMOD_LOCKMARKS as ::core::ffi::c_int;
                    ml_replace((*curwin.get()).w_cursor.lnum, prompt_line, true_0 != 0);
                    prompt_moved = concat_str(prompt, p_extra);
                    p_extra = prompt_moved;
                }
            }
            (*curwin.get()).w_cursor.lnum -= 1;
        }
        '_theend: {
            if State.get() & VREPLACE_FLAG == 0 as ::core::ffi::c_int
                || old_cursor.lnum >= orig_line_count.get()
            {
                if ml_append(
                    (*curwin.get()).w_cursor.lnum,
                    p_extra,
                    0 as colnr_T,
                    false_0 != 0,
                ) == FAIL
                {
                    break '_theend;
                } else {
                    mark_adjust(
                        (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        1 as linenr_T,
                        0 as linenr_T,
                        kExtmarkNOOP,
                    );
                    did_append = true_0 != 0;
                }
            } else {
                (*curwin.get()).w_cursor.lnum += 1;
                if (*curwin.get()).w_cursor.lnum
                    >= (*Insstart.ptr()).lnum + vr_lines_changed.get() as linenr_T
                {
                    u_save_cursor();
                    (*vr_lines_changed.ptr()) += 1;
                }
                ml_replace((*curwin.get()).w_cursor.lnum, p_extra, true_0 != 0);
                changed_bytes((*curwin.get()).w_cursor.lnum, 0 as colnr_T);
                (*curwin.get()).w_cursor.lnum -= 1;
                did_append = false_0 != 0;
            }
            (*inhibit_delete_count.ptr()) += 1;
            if newindent != 0 || did_si.get() as ::core::ffi::c_int != 0 {
                (*curwin.get()).w_cursor.lnum += 1;
                if did_si.get() {
                    let mut sw: ::core::ffi::c_int = get_sw_value(curbuf.get());
                    if p_sr.get() != 0 {
                        newindent -= newindent % sw;
                    }
                    newindent += sw;
                }
                if (*curbuf.get()).b_p_ci != 0 {
                    copy_indent(newindent, saved_line);
                    (*curbuf.get()).b_p_pi = true_0;
                } else {
                    set_indent(
                        newindent,
                        SIN_INSERT as ::core::ffi::c_int | SIN_NOMARK as ::core::ffi::c_int,
                    );
                }
                less_cols -= (*curwin.get()).w_cursor.col;
                ai_col.set((*curwin.get()).w_cursor.col);
                if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                    let mut n_0: colnr_T = 0 as colnr_T;
                    while n_0 < (*curwin.get()).w_cursor.col {
                        replace_push_nul();
                        n_0 += 1;
                    }
                }
                newcol += (*curwin.get()).w_cursor.col;
                if no_si {
                    did_si.set(false_0 != 0);
                }
            }
            (*inhibit_delete_count.ptr()) -= 1;
            if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                loop {
                    let c2rust_fresh3 = lead_len;
                    lead_len = lead_len - 1;
                    if c2rust_fresh3 <= 0 as ::core::ffi::c_int {
                        break;
                    }
                    replace_push_nul();
                }
            }
            (*curwin.get()).w_cursor = old_cursor;
            if dir == FORWARD as ::core::ffi::c_int {
                if trunc_line as ::core::ffi::c_int != 0 || State.get() & MODE_INSERT != 0 {
                    *saved_line.offset((*curwin.get()).w_cursor.col as isize) =
                        NUL as ::core::ffi::c_char;
                    if trunc_line as ::core::ffi::c_int != 0
                        && flags & OPENLINE_KEEPTRAIL as ::core::ffi::c_int == 0
                    {
                        truncate_spaces(saved_line, (*curwin.get()).w_cursor.col as size_t);
                    }
                    ml_replace((*curwin.get()).w_cursor.lnum, saved_line, false_0 != 0);
                    let mut new_len: ::core::ffi::c_int = strlen(saved_line) as ::core::ffi::c_int;
                    let mut cols_spliced: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if new_len < (*curwin.get()).w_cursor.col {
                        extmark_splice_cols(
                            curbuf.get(),
                            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                            new_len as colnr_T,
                            (*curwin.get()).w_cursor.col - new_len as colnr_T,
                            0 as colnr_T,
                            kExtmarkUndo,
                        );
                        cols_spliced = (*curwin.get()).w_cursor.col as ::core::ffi::c_int - new_len;
                    }
                    saved_line = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if did_append {
                        let mut cols_added: ::core::ffi::c_int = mincol as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int
                            + less_cols_off as ::core::ffi::c_int
                            - less_cols as ::core::ffi::c_int;
                        extmark_splice(
                            curbuf.get(),
                            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            mincol - 1 as colnr_T - cols_spliced as colnr_T,
                            0 as ::core::ffi::c_int,
                            less_cols_off,
                            less_cols_off as bcount_t,
                            1 as ::core::ffi::c_int,
                            cols_added as colnr_T,
                            (1 as ::core::ffi::c_int + cols_added) as bcount_t,
                            kExtmarkUndo,
                        );
                        changed_lines(
                            curbuf.get(),
                            (*curwin.get()).w_cursor.lnum,
                            (*curwin.get()).w_cursor.col,
                            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                            1 as linenr_T,
                            true_0 != 0,
                        );
                        did_append = false_0 != 0;
                        if flags & OPENLINE_MARKFIX as ::core::ffi::c_int != 0 {
                            mark_col_adjust(
                                (*curwin.get()).w_cursor.lnum,
                                (*curwin.get()).w_cursor.col + less_cols_off,
                                1 as linenr_T,
                                -less_cols,
                                0 as ::core::ffi::c_int,
                            );
                        }
                    } else {
                        changed_bytes((*curwin.get()).w_cursor.lnum, (*curwin.get()).w_cursor.col);
                    }
                }
                (*curwin.get()).w_cursor.lnum = old_cursor.lnum + 1 as linenr_T;
            }
            if did_append {
                let mut extra: bcount_t = ml_get_len((*curwin.get()).w_cursor.lnum) as bcount_t;
                extmark_splice(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    0 as colnr_T,
                    0 as ::core::ffi::c_int,
                    0 as colnr_T,
                    0 as bcount_t,
                    1 as ::core::ffi::c_int,
                    0 as colnr_T,
                    1 as bcount_t + extra,
                    kExtmarkUndo,
                );
                changed_lines(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum,
                    0 as colnr_T,
                    (*curwin.get()).w_cursor.lnum,
                    1 as linenr_T,
                    true_0 != 0,
                );
            }
            (*curbuf_splice_pending.ptr()) -= 1;
            (*curwin.get()).w_cursor.col = newcol;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            if State.get() & VREPLACE_FLAG != 0 {
                vreplace_mode = State.get();
                State.set(MODE_INSERT);
            } else {
                vreplace_mode = 0 as ::core::ffi::c_int;
            }
            if p_paste.get() == 0 {
                if leader.is_null()
                    && !use_indentexpr_for_lisp()
                    && (*curbuf.get()).b_p_lisp != 0
                    && (*curbuf.get()).b_p_ai != 0
                {
                    fixthisline(Some(
                        get_lisp_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                    ));
                    ai_col.set(getwhitecols_curline() as colnr_T);
                } else if do_cindent as ::core::ffi::c_int != 0
                    || (*curbuf.get()).b_p_ai != 0
                        && use_indentexpr_for_lisp() as ::core::ffi::c_int != 0
                {
                    do_c_expr_indent();
                    ai_col.set(getwhitecols_curline() as colnr_T);
                }
            }
            if vreplace_mode != 0 as ::core::ffi::c_int {
                State.set(vreplace_mode);
            }
            if State.get() & VREPLACE_FLAG != 0 {
                p_extra = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
                ml_replace((*curwin.get()).w_cursor.lnum, next_line, false_0 != 0);
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                ins_bytes(p_extra);
                xfree(p_extra as *mut ::core::ffi::c_void);
                next_line = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            retval = true_0 != 0;
        }
        (*curbuf.get()).b_p_pi = saved_pi;
        xfree(saved_line as *mut ::core::ffi::c_void);
        xfree(next_line as *mut ::core::ffi::c_void);
        xfree(allocated as *mut ::core::ffi::c_void);
        xfree(prompt_moved as *mut ::core::ffi::c_void);
        (*cmdmod.ptr()).cmod_flags = old_cmod_flags;
        return retval;
    }
}
