//! The identifier under the cursor, and the commands that look it up:
//! tags, `:help`, 'keywordprg', a declaration, a file name.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn find_is_eval_item(
    ptr: *const c_char,
    colp: *mut c_int,
    bnp: *mut c_int,
    dir: c_int,
) -> bool {
    if *ptr as c_int == ']' as c_int && dir == BACKWARD as c_int
        || *ptr as c_int == '[' as c_int && dir == FORWARD as c_int
    {
        *bnp += 1 as c_int;
    }
    if *bnp > 0 as c_int {
        if *ptr as c_int == '[' as c_int && dir == BACKWARD as c_int
            || *ptr as c_int == ']' as c_int && dir == FORWARD as c_int
        {
            *bnp -= 1 as c_int;
        }
        return true_0 != 0;
    }
    if *ptr as c_int == '.' as c_int {
        return true_0 != 0;
    }
    if *ptr.offset(
        (if dir == BACKWARD as c_int {
            0 as c_int
        } else {
            1 as c_int
        }) as isize,
    ) as c_int
        == '>' as c_int
        && *ptr.offset(
            (if dir == BACKWARD as c_int {
                -1 as c_int
            } else {
                0 as c_int
            }) as isize,
        ) as c_int
            == '-' as c_int
    {
        *colp += dir;
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub unsafe extern "C" fn find_ident_under_cursor(
    mut text: *mut *mut c_char,
    mut find_type: c_int,
    mut offset: *mut c_int,
) -> size_t {
    let mut textcol: c_int = 0 as c_int;
    let mut len: size_t = find_ident_at_pos(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum,
        (*curwin.get()).w_cursor.col,
        text,
        if !offset.is_null() {
            &raw mut textcol
        } else {
            ::core::ptr::null_mut::<c_int>()
        },
        find_type,
    );
    if !offset.is_null() {
        *offset = (*curwin.get()).w_cursor.col as c_int - textcol;
    }
    return len;
}

pub unsafe extern "C" fn find_ident_at_pos(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut startcol: colnr_T,
    mut text: *mut *mut c_char,
    mut textcol: *mut c_int,
    mut find_type: c_int,
) -> size_t {
    let mut col: c_int = 0 as c_int;
    let mut i: c_int = 0;
    let mut this_class: c_int = 0 as c_int;
    let mut prev_class: c_int = 0;
    let mut prevcol: c_int = 0;
    let mut bn: c_int = 0 as c_int;
    let mut ptr: *mut c_char = ml_get_buf((*wp).w_buffer, lnum);
    i = if find_type & FIND_IDENT as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    while i < 2 as c_int {
        col = startcol as c_int;
        while *ptr.offset(col as isize) as c_int != NUL {
            if find_type & FIND_EVAL as c_int != 0
                && *ptr.offset(col as isize) as c_int == ']' as c_int
            {
                break;
            }
            this_class = mb_get_class(ptr.offset(col as isize));
            if this_class != 0 as c_int && (i == 1 as c_int || this_class != 1 as c_int) {
                break;
            }
            col += utfc_ptr2len(ptr.offset(col as isize));
        }
        bn = (*ptr.offset(col as isize) as c_int == ']' as c_int) as c_int;
        if find_type & FIND_EVAL as c_int != 0 && *ptr.offset(col as isize) as c_int == ']' as c_int
        {
            this_class = mb_get_class(b"a\0".as_ptr() as *const c_char);
        } else {
            this_class = mb_get_class(ptr.offset(col as isize));
        }
        while col > 0 as c_int && this_class != 0 as c_int {
            prevcol = col
                - 1 as c_int
                - utf_head_off(ptr, ptr.offset(col as isize).offset(-(1 as c_int as isize)));
            prev_class = mb_get_class(ptr.offset(prevcol as isize));
            if this_class != prev_class
                && (i == 0 as c_int
                    || prev_class == 0 as c_int
                    || find_type & FIND_IDENT as c_int != 0)
                && (find_type & FIND_EVAL as c_int == 0
                    || prevcol == 0 as c_int
                    || !find_is_eval_item(
                        ptr.offset(prevcol as isize),
                        &raw mut prevcol,
                        &raw mut bn,
                        BACKWARD as c_int,
                    ))
            {
                break;
            }
            col = prevcol;
        }
        this_class = if this_class < 2 as c_int {
            this_class
        } else {
            2 as c_int
        };
        if find_type & FIND_STRING as c_int == 0 || this_class == 2 as c_int {
            break;
        }
        i += 1;
    }
    if *ptr.offset(col as isize) as c_int == NUL || i == 0 as c_int && this_class != 2 as c_int {
        if find_type & FIND_STRING as c_int != 0 {
            emsg(gettext(
                b"E348: No string under cursor\0".as_ptr() as *const c_char
            ));
        } else {
            emsg(gettext(&raw const e_noident as *const c_char));
        }
        return 0 as size_t;
    }
    ptr = ptr.offset(col as isize);
    *text = ptr;
    if !textcol.is_null() {
        *textcol = col;
    }
    bn = 0 as c_int;
    startcol -= col;
    col = 0 as c_int;
    this_class = mb_get_class(ptr);
    while *ptr.offset(col as isize) as c_int != NUL
        && ((if i == 0 as c_int {
            (mb_get_class(ptr.offset(col as isize)) == this_class) as c_int
        } else {
            (mb_get_class(ptr.offset(col as isize)) != 0 as c_int) as c_int
        }) != 0
            || find_type & FIND_EVAL as c_int != 0
                && col <= startcol
                && find_is_eval_item(
                    ptr.offset(col as isize),
                    &raw mut col,
                    &raw mut bn,
                    FORWARD as c_int,
                ) as c_int
                    != 0)
    {
        col += utfc_ptr2len(ptr.offset(col as isize));
    }
    '_c2rust_label: {
        if col >= 0 as c_int {
        } else {
            __assert_fail(
                b"col >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                1748 as c_uint,
                b"size_t find_ident_at_pos(win_T *, linenr_T, colnr_T, char **, int *, int)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    return col as size_t;
}

pub(crate) unsafe extern "C" fn nv_gd(
    mut oap: *mut oparg_T,
    mut nchar: c_int,
    mut thisblock: c_int,
) {
    let mut len: size_t = 0;
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    len = find_ident_under_cursor(
        &raw mut ptr,
        FIND_IDENT as c_int,
        ::core::ptr::null_mut::<c_int>(),
    );
    if len == 0 as size_t
        || !find_decl(
            ptr,
            len,
            nchar == 'd' as c_int,
            thisblock != 0,
            SEARCH_START as c_int,
        )
    {
        clearopbeep(oap);
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
    if messaging() as c_int != 0 && msg_silent.get() == 0 && !shortmess(SHM_SEARCHCOUNT as c_int) {
        clear_cmdline.set(true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn is_ident(mut line: *const c_char, mut offset: c_int) -> bool {
    let mut incomment: bool = false_0 != 0;
    let mut instring: c_int = 0 as c_int;
    let mut prev: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    while i < offset && *line.offset(i as isize) as c_int != NUL {
        if instring != 0 as c_int {
            if prev != '\\' as c_int && *line.offset(i as isize) as uint8_t as c_int == instring {
                instring = 0 as c_int;
            }
        } else if (*line.offset(i as isize) as c_int == '"' as c_int
            || *line.offset(i as isize) as c_int == '\'' as c_int)
            && !incomment
        {
            instring = *line.offset(i as isize) as uint8_t as c_int;
        } else if incomment {
            if prev == '*' as c_int && *line.offset(i as isize) as c_int == '/' as c_int {
                incomment = false_0 != 0;
            }
        } else if prev == '/' as c_int && *line.offset(i as isize) as c_int == '*' as c_int {
            incomment = true_0 != 0;
        } else if prev == '/' as c_int && *line.offset(i as isize) as c_int == '/' as c_int {
            return false_0 != 0;
        }
        prev = *line.offset(i as isize) as uint8_t as c_int;
        i += 1;
    }
    return incomment as c_int == false_0 && instring == 0 as c_int;
}

pub unsafe extern "C" fn find_decl(
    mut ptr: *mut c_char,
    mut len: size_t,
    mut locally: bool,
    mut thisblock: bool,
    mut flags_arg: c_int,
) -> bool {
    let mut par_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut t: bool = false;
    let mut retval: bool = true_0 != 0;
    let mut incll: bool = false;
    let mut searchflags: c_int = flags_arg;
    let mut patsize: size_t = len.wrapping_add(7 as size_t);
    let mut pat: *mut c_char = xmalloc(patsize) as *mut c_char;
    '_c2rust_label: {
        if patsize <= 2147483647 as c_int as size_t {
        } else {
            __assert_fail(
                b"patsize <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                2387 as c_uint,
                b"_Bool find_decl(char *, size_t, _Bool, _Bool, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut patlen: size_t = snprintf(
        pat,
        patsize,
        if vim_iswordp(ptr) as c_int != 0 {
            b"\\V\\<%.*s\\>\0".as_ptr() as *const c_char
        } else {
            b"\\V%.*s\0".as_ptr() as *const c_char
        },
        len as c_int,
        ptr,
    ) as size_t;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut save_p_scs: bool = p_scs.get() != 0;
    p_ws.set(false_0);
    p_scs.set(false_0);
    if !locally
        || !findpar(
            &raw mut incll,
            BACKWARD as c_int,
            1 as c_int,
            '{' as c_int,
            false_0 != 0,
        )
    {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        par_pos = (*curwin.get()).w_cursor;
    } else {
        par_pos = (*curwin.get()).w_cursor;
        while (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            && *skipwhite(get_cursor_line_ptr()) as c_int != NUL
        {
            (*curwin.get()).w_cursor.lnum -= 1;
        }
    }
    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    clearpos(&mut found_pos);
    loop {
        t = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut (*curwin.get()).w_cursor,
            ::core::ptr::null_mut::<pos_T>(),
            FORWARD,
            pat,
            patlen,
            1 as c_int,
            searchflags,
            RE_LAST as c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) != 0;
        if (*curwin.get()).w_cursor.lnum >= old_pos.lnum {
            t = false_0 != 0;
        }
        if thisblock as c_int != 0 && t as c_int != false_0 {
            let maxtravel: int64_t =
                (old_pos.lnum - (*curwin.get()).w_cursor.lnum + 1 as linenr_T) as int64_t;
            let mut pos: *const pos_T = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                '}' as c_int,
                FM_FORWARD as c_int,
                maxtravel,
            );
            if !pos.is_null() && (*pos).lnum < old_pos.lnum {
                (*curwin.get()).w_cursor = *pos;
                continue;
            }
        }
        if t as c_int == false_0 {
            if found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                t = true_0 != 0;
            }
            break;
        } else if get_leader_len(
            get_cursor_line_ptr(),
            ::core::ptr::null_mut::<*mut c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) > 0 as c_int
        {
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
        } else {
            let mut valid: bool =
                is_ident(get_cursor_line_ptr(), (*curwin.get()).w_cursor.col as c_int);
            if !valid && found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                break;
            } else {
                if valid as c_int != 0 && !locally {
                    break;
                }
                if valid as c_int != 0 && (*curwin.get()).w_cursor.lnum >= par_pos.lnum {
                    if found_pos.lnum != 0 as linenr_T {
                        (*curwin.get()).w_cursor = found_pos;
                    }
                    break;
                } else {
                    if !valid {
                        clearpos(&mut found_pos);
                    } else {
                        found_pos = (*curwin.get()).w_cursor;
                    }
                    searchflags &= !(SEARCH_START as c_int);
                }
            }
        }
    }
    if t as c_int == false_0 {
        retval = false_0 != 0;
        (*curwin.get()).w_cursor = old_pos;
    } else {
        (*curwin.get()).w_set_curswant = true_0;
        reset_search_dir();
    }
    xfree(pat as *mut c_void);
    p_ws.set(save_p_ws as c_int);
    p_scs.set(save_p_scs as c_int);
    return retval;
}

pub unsafe extern "C" fn do_nv_ident(mut c1: c_int, mut c2: c_int) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut ca: cmdarg_T = cmdarg_T {
        oap: ::core::ptr::null_mut::<oparg_T>(),
        prechar: 0,
        cmdchar: 0,
        nchar: 0,
        nchar_composing: [0; 32],
        nchar_len: 0,
        extra_char: 0,
        opcount: 0,
        count0: 0,
        count1: 0,
        arg: 0,
        retval: 0,
        searchbuf: ::core::ptr::null_mut::<c_char>(),
    };
    clear_oparg(&raw mut oa);
    memset(
        &raw mut ca as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    ca.oap = &raw mut oa;
    ca.cmdchar = c1;
    ca.nchar = c2;
    nv_ident(&raw mut ca);
}

pub(crate) unsafe extern "C" fn nv_K_getcmd(
    mut cap: *mut cmdarg_T,
    mut kp: *mut c_char,
    mut kp_help: bool,
    mut kp_ex: bool,
    mut ptr_arg: *mut *mut c_char,
    mut n: size_t,
    mut buf: *mut c_char,
    mut bufsize: size_t,
    mut buflen: *mut size_t,
) -> size_t {
    if kp_help {
        strcpy(buf, b"help! \0".as_ptr() as *const c_char as *mut c_char);
        *buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
        return n;
    }
    if kp_ex {
        *buflen = 0 as size_t;
        *buflen = snprintf(buf, bufsize, b"%s \0".as_ptr() as *const c_char, kp) as size_t;
        if (*cap).count0 != 0 as c_int {
            *buflen = (*buflen).wrapping_add(snprintf(
                buf.offset(*buflen as isize),
                bufsize.wrapping_sub(*buflen),
                b"%ld \0".as_ptr() as *const c_char,
                (*cap).count0 as int64_t,
            ) as size_t);
        }
        return n;
    }
    let mut ptr: *mut c_char = *ptr_arg;
    while *ptr as c_int == '-' as c_int && n > 0 as size_t {
        ptr = ptr.offset(1);
        n = n.wrapping_sub(1);
    }
    if n == 0 as size_t {
        emsg(gettext(&raw const e_noident as *const c_char));
        xfree(buf as *mut c_void);
        *ptr_arg = ptr;
        return 0 as size_t;
    }
    let mut isman: bool = strcmp(kp, b"man\0".as_ptr() as *const c_char) == 0 as c_int;
    let mut isman_s: bool = strcmp(kp, b"man -s\0".as_ptr() as *const c_char) == 0 as c_int;
    if (*cap).count0 != 0 as c_int && !(isman as c_int != 0 || isman_s as c_int != 0) {
        *buflen = snprintf(
            buf,
            bufsize,
            b".,.+%ld\0".as_ptr() as *const c_char,
            ((*cap).count0 - 1 as c_int) as int64_t,
        ) as size_t;
    }
    do_cmdline_cmd(b"tabnew\0".as_ptr() as *const c_char);
    *buflen = (*buflen).wrapping_add(snprintf(
        buf.offset(*buflen as isize),
        bufsize.wrapping_sub(*buflen),
        b"terminal \0".as_ptr() as *const c_char,
    ) as size_t);
    if (*cap).count0 == 0 as c_int && isman_s as c_int != 0 {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"man \0".as_ptr() as *const c_char,
        ) as size_t);
    } else {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%s \0".as_ptr() as *const c_char,
            kp,
        ) as size_t);
    }
    if (*cap).count0 != 0 as c_int && (isman as c_int != 0 || isman_s as c_int != 0) {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%ld \0".as_ptr() as *const c_char,
            (*cap).count0 as int64_t,
        ) as size_t);
    }
    *ptr_arg = ptr;
    return n;
}

pub(crate) unsafe extern "C" fn nv_ident(mut cap: *mut cmdarg_T) {
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut n: size_t = 0 as size_t;
    let mut cmdchar: c_int = 0;
    let mut g_cmd: bool = false;
    let mut tag_cmd: bool = false_0 != 0;
    if (*cap).cmdchar == 'g' as c_int {
        cmdchar = (*cap).nchar;
        g_cmd = true_0 != 0;
    } else {
        cmdchar = (*cap).cmdchar;
        g_cmd = false_0 != 0;
    }
    if cmdchar == POUND {
        cmdchar = '#' as c_int;
    }
    let mut visual_sel: bool = false_0 != 0;
    if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
        if VIsual_active.get() as c_int != 0
            && get_visual_text(cap, &raw mut ptr, &raw mut n) as c_int == false_0
        {
            return;
        }
        visual_sel = !ptr.is_null();
        if checkclearopq((*cap).oap) {
            return;
        }
    }
    let mut ident_offset: c_int = 0 as c_int;
    if ptr.is_null() && {
        n = find_ident_under_cursor(
            &raw mut ptr,
            if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
                FIND_IDENT as c_int | FIND_STRING as c_int
            } else {
                FIND_IDENT as c_int
            },
            &raw mut ident_offset,
        );
        n == 0 as size_t
    } {
        clearop((*cap).oap);
        return;
    }
    let mut kp: *mut c_char = if *(*curbuf.get()).b_p_kp as c_int == NUL {
        p_kp.get()
    } else {
        (*curbuf.get()).b_p_kp
    };
    let mut kp_helpbang: bool = strequal(kp, b":help!\0".as_ptr() as *const c_char);
    let mut kp_help: bool = kp_helpbang as c_int != 0
        || *kp as c_int == NUL
        || strequal(kp, b":he\0".as_ptr() as *const c_char) as c_int != 0
        || strequal(kp, b":help\0".as_ptr() as *const c_char) as c_int != 0;
    if kp_help as c_int != 0 && !kp_helpbang && *skipwhite(ptr) as c_int == NUL {
        emsg(gettext(&raw const e_noident as *const c_char));
        return;
    }
    let mut kp_ex: bool = *kp as c_int == ':' as c_int;
    let mut bufsize: size_t = n
        .wrapping_mul(2 as size_t)
        .wrapping_add(30 as size_t)
        .wrapping_add(strlen(kp));
    let mut buf: *mut c_char = xmalloc(bufsize) as *mut c_char;
    *buf.offset(0 as c_int as isize) = NUL as c_char;
    let mut buflen: size_t = 0 as size_t;
    match cmdchar {
        42 | 35 => {
            setpcmark();
            (*curwin.get()).w_cursor.col = ptr.offset_from(get_cursor_line_ptr()) as colnr_T;
            if !g_cmd && vim_iswordp(ptr) as c_int != 0 {
                strcpy(buf, b"\\<\0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as usize) as size_t;
            }
            no_smartcase.set(true_0 != 0);
        }
        75 => {
            n = nv_K_getcmd(
                cap,
                kp,
                kp_help,
                kp_ex,
                &raw mut ptr,
                n,
                buf,
                bufsize,
                &raw mut buflen,
            );
            if n == 0 as size_t {
                return;
            }
        }
        93 => {
            tag_cmd = true_0 != 0;
            strcpy(buf, b"tselect \0".as_ptr() as *const c_char as *mut c_char);
            buflen = ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as size_t;
        }
        _ => {
            tag_cmd = true_0 != 0;
            if (*curbuf.get()).b_help {
                strcpy(buf, b"help! \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
            } else if g_cmd {
                strcpy(buf, b"tjump \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
            } else if (*cap).count0 == 0 as c_int {
                strcpy(buf, b"tag \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as size_t;
            } else {
                buflen = snprintf(
                    buf,
                    bufsize,
                    b":%ldtag \0".as_ptr() as *const c_char,
                    (*cap).count0 as int64_t,
                ) as size_t;
            }
        }
    }
    if cmdchar == 'K' as c_int && kp_helpbang as c_int != 0 && !visual_sel {
        strcpy(buf, b"help!\0".as_ptr() as *const c_char as *mut c_char);
        buflen = ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as usize) as size_t;
    } else if cmdchar == 'K' as c_int && !kp_help {
        ptr = xstrnsave(ptr, n);
        if kp_ex {
            p = vim_strsave_fnameescape(ptr, VSE_NONE as c_int);
        } else {
            p = vim_strsave_shellescape(ptr, true_0 != 0, true_0 != 0);
        }
        xfree(ptr as *mut c_void);
        let mut plen: size_t = strlen(p);
        let mut newbuf: *mut c_char = xrealloc(
            buf as *mut c_void,
            buflen.wrapping_add(plen).wrapping_add(1 as size_t),
        ) as *mut c_char;
        buf = newbuf;
        strcpy(buf.offset(buflen as isize), p);
        buflen = buflen.wrapping_add(plen);
        xfree(p as *mut c_void);
    } else {
        let mut aux_ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if cmdchar == '*' as c_int {
            aux_ptr = (if magic_isset() as c_int != 0 {
                b"/.*~[^$\\\0".as_ptr() as *const c_char
            } else {
                b"/^$\\\0".as_ptr() as *const c_char
            }) as *mut c_char;
        } else if cmdchar == '#' as c_int {
            aux_ptr = (if magic_isset() as c_int != 0 {
                b"/?.*~[^$\\\0".as_ptr() as *const c_char
            } else {
                b"/?^$\\\0".as_ptr() as *const c_char
            }) as *mut c_char;
        } else if tag_cmd {
            if strcmp((*curbuf.get()).b_p_ft, b"help\0".as_ptr() as *const c_char) == 0 as c_int {
                aux_ptr = b"\0".as_ptr() as *const c_char as *mut c_char;
            } else {
                aux_ptr = b"\\|\"\n[\0".as_ptr() as *const c_char as *mut c_char;
            }
        } else {
            aux_ptr = b"\\|\"\n*?[\0".as_ptr() as *const c_char as *mut c_char;
        }
        p = buf.offset(buflen as isize);
        loop {
            let c2rust_fresh0 = n;
            n = n.wrapping_sub(1);
            if c2rust_fresh0 <= 0 as size_t {
                break;
            }
            if !vim_strchr(aux_ptr, *ptr as uint8_t as c_int).is_null() {
                let c2rust_fresh1 = p;
                p = p.offset(1);
                *c2rust_fresh1 = '\\' as c_char;
            }
            let len: size_t = (utfc_ptr2len(ptr) - 1 as c_int) as size_t;
            let mut i: size_t = 0 as size_t;
            while i < len && n > 0 as size_t {
                let c2rust_fresh2 = ptr;
                ptr = ptr.offset(1);
                let c2rust_fresh3 = p;
                p = p.offset(1);
                *c2rust_fresh3 = *c2rust_fresh2;
                i = i.wrapping_add(1);
                n = n.wrapping_sub(1);
            }
            let c2rust_fresh4 = ptr;
            ptr = ptr.offset(1);
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = *c2rust_fresh4;
        }
        *p = NUL as c_char;
        buflen = p.offset_from(buf) as size_t;
    }
    if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
        if !g_cmd && vim_iswordp(mb_prevptr(get_cursor_line_ptr(), ptr)) as c_int != 0 {
            strcpy(
                buf.offset(buflen as isize),
                b"\\>\0".as_ptr() as *const c_char as *mut c_char,
            );
            buflen = (buflen as c_ulong).wrapping_add(
                ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as usize) as c_ulong,
            ) as size_t;
        }
        init_history();
        add_to_history(
            HIST_SEARCH as c_int,
            ::core::slice::from_raw_parts(buf as *const u8, buflen as usize),
            true_0 != 0,
            NUL as u8,
        );
        normal_search(
            cap,
            if cmdchar == '*' as c_int {
                '/' as c_int
            } else {
                '?' as c_int
            },
            buf,
            buflen,
            0 as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
    } else {
        g_tag_at_cursor.set(true_0 != 0);
        do_cmdline_cmd(buf);
        g_tag_at_cursor.set(false_0 != 0);
        if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
            restart_edit.set('i' as c_int);
            add_map(
                b"<esc>\0".as_ptr() as *const c_char as *mut c_char,
                b"<Cmd>bdelete!<CR>\0".as_ptr() as *const c_char as *mut c_char,
                MODE_TERMINAL as c_int,
                true_0 != 0,
            );
        }
    }
    xfree(buf as *mut c_void);
}

pub(crate) unsafe extern "C" fn nv_tagpop(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        do_tag(
            b"\0".as_ptr() as *const c_char as *mut c_char,
            DT_POP as c_int,
            (*cap).count1,
            false_0,
            true_0 != 0,
        );
    }
}

pub(crate) unsafe extern "C" fn nv_gotofile(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = -1 as linenr_T;
    if check_text_or_curbuf_locked((*cap).oap) {
        return;
    }
    if !check_can_set_curbuf_disabled() {
        return;
    }
    let mut ptr: *mut c_char = grab_file_name((*cap).count1, &raw mut lnum);
    if !ptr.is_null() {
        if curbufIsChanged() as c_int != 0
            && (*curbuf.get()).b_nwindows <= 1 as c_int
            && !buf_hide(curbuf.get())
        {
            autowrite(curbuf.get(), false_0 != 0);
        }
        setpcmark();
        if do_ecmd(
            0 as c_int,
            ptr,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_LAST as c_int as linenr_T,
            if buf_hide(curbuf.get()) as c_int != 0 {
                ECMD_HIDE as c_int
            } else {
                0 as c_int
            },
            curwin.get(),
        ) == OK
            && (*cap).nchar == 'F' as c_int
            && lnum >= 0 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = lnum;
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        xfree(ptr as *mut c_void);
    } else {
        clearop((*cap).oap);
    };
}
