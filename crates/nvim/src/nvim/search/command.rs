//! The user-facing search commands.
//!
//! [`do_search`] is `/` and `?` in full: it parses the offset off the end
//! of the pattern, adds the pattern to the history, reports where it wound
//! up, and drives [`searchit`](super::searchit) once per count.
//! [`searchc`] is the `f`/`t`/`;`/`,` character search, [`current_search`]
//! is `gn`/`gN`, and [`showmatch`] is the `'showmatch'` blink.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn last_csearch() -> *const ::core::ffi::c_char {
    return lastc_bytes.ptr() as *mut ::core::ffi::c_char;
}

pub unsafe extern "C" fn last_csearch_forward() -> ::core::ffi::c_int {
    return (lastcdir.get() as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}

pub unsafe extern "C" fn last_csearch_until() -> ::core::ffi::c_int {
    return last_t_cmd.get() as ::core::ffi::c_int;
}

pub unsafe extern "C" fn set_last_csearch(
    mut c: ::core::ffi::c_int,
    mut s: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    unsafe {
        *(lastc.ptr() as *mut uint8_t) = c as uint8_t;
        lastc_bytelen.set(len);
        if len != 0 {
            memcpy(
                lastc_bytes.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                len as size_t,
            );
        } else {
            memset(
                lastc_bytes.ptr() as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[::core::ffi::c_char; 33]>(),
            );
        };
    }
}

pub unsafe extern "C" fn set_csearch_direction(mut cdir: Direction) {
    lastcdir.set(cdir);
}

pub unsafe extern "C" fn set_csearch_until(mut t_cmd: ::core::ffi::c_int) {
    last_t_cmd.set(t_cmd != 0);
}

pub unsafe extern "C" fn do_search(
    mut oap: *mut oparg_T,
    mut dirc: ::core::ffi::c_int,
    mut search_delim: ::core::ffi::c_int,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut count: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut sia: *mut searchit_arg_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut searchstr: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut searchstrlen: size_t = 0;
        let mut retval: ::core::ffi::c_int = 0;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut c: int64_t = 0;
        let mut dircp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut strcopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut ps: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut msgbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut msgbuflen: size_t = 0 as size_t;
        let mut has_offset: bool = false_0 != 0;
        searchcmdlen.set(0 as ::core::ffi::c_int);
        if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line as ::core::ffi::c_int != 0
            && !vim_strchr(p_cpo.get(), CPO_LINEOFF).is_null()
        {
            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = false_0 != 0;
            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 0 as int64_t;
        }
        let mut old_off: SearchOffset = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off;
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        if dirc == 0 as ::core::ffi::c_int {
            dirc = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir as uint8_t
                as ::core::ffi::c_int;
        } else {
            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = dirc as ::core::ffi::c_char;
            set_vv_searchforward();
        }
        if options & SEARCH_REV as ::core::ffi::c_int != 0 {
            dirc = if dirc == '/' as ::core::ffi::c_int {
                '?' as ::core::ffi::c_int
            } else {
                '/' as ::core::ffi::c_int
            };
        }
        if dirc == '/' as ::core::ffi::c_int {
            if hasFolding(
                curwin.get(),
                pos.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut pos.lnum,
            ) {
                pos.col = (MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as colnr_T;
            }
        } else if hasFolding(
            curwin.get(),
            pos.lnum,
            &raw mut pos.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            pos.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        if no_hlsearch.get() as ::core::ffi::c_int != 0
            && options & SEARCH_KEEP as ::core::ffi::c_int == 0
        {
            redraw_all_later(UPD_SOME_VALID);
            set_no_hlsearch(false_0 != 0);
        }
        '_end_do_search: {
            loop {
                let mut show_top_bot_msg: bool = false_0 != 0;
                searchstr = pat;
                searchstrlen = patlen;
                dircp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                if pat.is_null()
                    || *pat as ::core::ffi::c_int == NUL
                    || *pat as ::core::ffi::c_int == search_delim
                {
                    if (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]
                        .pat
                        .is_null()
                    {
                        if (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize]
                            .pat
                            .is_null()
                        {
                            emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                            retval = 0 as ::core::ffi::c_int;
                            break '_end_do_search;
                        } else {
                            searchstr = (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize].pat;
                            searchstrlen =
                                (*spats.ptr())[RE_SUBST as ::core::ffi::c_int as usize].patlen;
                        }
                    } else {
                        searchstr = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        searchstrlen = 0 as size_t;
                    }
                }
                if !pat.is_null() && *pat as ::core::ffi::c_int != NUL {
                    ps = strcopy;
                    p = skip_regexp_ex(
                        pat,
                        search_delim,
                        magic_isset() as ::core::ffi::c_int,
                        &raw mut strcopy,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ::core::ptr::null_mut::<magic_T>(),
                    );
                    if strcopy != ps {
                        let mut len: size_t = strlen(strcopy);
                        (*searchcmdlen.ptr()) += patlen.wrapping_sub(len) as ::core::ffi::c_int;
                        pat = strcopy;
                        patlen = len;
                        searchstr = strcopy;
                        searchstrlen = len;
                    }
                    if *p as ::core::ffi::c_int == search_delim {
                        searchstrlen = p.offset_from(pat) as size_t;
                        dircp = p;
                        let c2rust_fresh1 = p;
                        p = p.offset(1);
                        *c2rust_fresh1 = NUL as ::core::ffi::c_char;
                    }
                    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = false_0 != 0;
                    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end = false_0 != 0;
                    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 0 as int64_t;
                    if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line = true_0 != 0;
                    } else if options & SEARCH_OPT as ::core::ffi::c_int != 0
                        && (*p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == 'b' as ::core::ffi::c_int)
                    {
                        if *p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end = true_0 != 0;
                        }
                        p = p.offset(1);
                    }
                    if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    {
                        if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            || ascii_isdigit(
                                *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                        {
                            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off =
                                atol(p) as int64_t;
                        } else if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off =
                                -1 as int64_t;
                        } else {
                            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off = 1 as int64_t;
                        }
                        p = p.offset(1);
                        while ascii_isdigit(*p as ::core::ffi::c_int) {
                            p = p.offset(1);
                        }
                    }
                    (*searchcmdlen.ptr()) += p.offset_from(pat) as ::core::ffi::c_int;
                    patlen = patlen.wrapping_sub(p.offset_from(pat) as size_t);
                    pat = p;
                }
                let mut show_search_stats: bool = false_0 != 0;
                if options & SEARCH_ECHO as ::core::ffi::c_int != 0
                    && messaging() as ::core::ffi::c_int != 0
                    && msg_silent.get() == 0
                    && (!cmd_silent.get() || !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int))
                {
                    let mut off_buf: [::core::ffi::c_char; 40] = [0; 40];
                    let mut off_len: size_t = 0 as size_t;
                    msg_start();
                    msg_ext_set_kind(b"search_cmd\0".as_ptr() as *const ::core::ffi::c_char);
                    if !cmd_silent.get()
                        && ((*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                            as ::core::ffi::c_int
                            != 0
                            || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end
                                as ::core::ffi::c_int
                                != 0
                            || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0)
                    {
                        let c2rust_fresh2 = off_len;
                        off_len = off_len.wrapping_add(1);
                        off_buf[c2rust_fresh2 as usize] = dirc as ::core::ffi::c_char;
                        if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end {
                            let c2rust_fresh3 = off_len;
                            off_len = off_len.wrapping_add(1);
                            off_buf[c2rust_fresh3 as usize] = 'e' as ::core::ffi::c_char;
                        } else if !(*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line {
                            let c2rust_fresh4 = off_len;
                            off_len = off_len.wrapping_add(1);
                            off_buf[c2rust_fresh4 as usize] = 's' as ::core::ffi::c_char;
                        }
                        off_buf[off_len as usize] = NUL as ::core::ffi::c_char;
                        if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0 as int64_t
                            || (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                                as ::core::ffi::c_int
                                != 0
                        {
                            off_len = off_len.wrapping_add(snprintf(
                                (&raw mut off_buf as *mut ::core::ffi::c_char)
                                    .offset(off_len as isize),
                                ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                    .wrapping_sub(off_len),
                                b"%+ld\0".as_ptr() as *const ::core::ffi::c_char,
                                (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off,
                            ) as size_t);
                        }
                    }
                    let mut plen: size_t = 0;
                    if *searchstr as ::core::ffi::c_int == NUL {
                        p = (*spats.ptr())[0 as ::core::ffi::c_int as usize].pat;
                        plen = (*spats.ptr())[0 as ::core::ffi::c_int as usize].patlen;
                    } else {
                        p = searchstr;
                        plen = searchstrlen;
                    }
                    let mut msgbufsize: size_t = 0;
                    if !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int)
                        || cmd_silent.get() as ::core::ffi::c_int != 0
                    {
                        if ui_has(kUIMessages) {
                            msgbufsize = 0 as size_t;
                        } else if msg_scrolled.get() != 0 as ::core::ffi::c_int && !cmd_silent.get()
                        {
                            msgbufsize = ((Rows.get() - msg_row.get()) * Columns.get()
                                - 1 as ::core::ffi::c_int)
                                as size_t;
                        } else {
                            msgbufsize = ((Rows.get() - msg_row.get() - 1 as ::core::ffi::c_int)
                                * Columns.get()
                                + sc_col.get()
                                - 1 as ::core::ffi::c_int)
                                as size_t;
                        }
                        if msgbufsize
                            < plen
                                .wrapping_add(off_len)
                                .wrapping_add(SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t)
                                .wrapping_add(3 as size_t)
                        {
                            msgbufsize = plen
                                .wrapping_add(off_len)
                                .wrapping_add(SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t)
                                .wrapping_add(3 as size_t);
                        }
                    } else {
                        msgbufsize = plen.wrapping_add(off_len).wrapping_add(3 as size_t);
                    }
                    xfree(msgbuf as *mut ::core::ffi::c_void);
                    msgbuf = xmalloc(msgbufsize) as *mut ::core::ffi::c_char;
                    memset(
                        msgbuf as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        msgbufsize,
                    );
                    msgbuflen = msgbufsize.wrapping_sub(1 as size_t);
                    *msgbuf.offset(msgbuflen as isize) = NUL as ::core::ffi::c_char;
                    if !cmd_silent.get() {
                        ui_busy_start();
                        *msgbuf.offset(0 as ::core::ffi::c_int as isize) =
                            dirc as ::core::ffi::c_char;
                        if utf_iscomposing_first(utf_ptr2char(p)) {
                            *msgbuf.offset(1 as ::core::ffi::c_int as isize) =
                                ' ' as ::core::ffi::c_char;
                            memmove(
                                msgbuf.offset(2 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                p as *const ::core::ffi::c_void,
                                plen,
                            );
                        } else {
                            memmove(
                                msgbuf.offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                p as *const ::core::ffi::c_void,
                                plen,
                            );
                        }
                        if off_len > 0 as size_t {
                            memmove(
                                msgbuf
                                    .offset(plen as isize)
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                &raw mut off_buf as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                off_len,
                            );
                        }
                        let mut trunc: *mut ::core::ffi::c_char = msg_strtrunc(msgbuf, true_0);
                        if !trunc.is_null() {
                            xfree(msgbuf as *mut ::core::ffi::c_void);
                            msgbuf = trunc;
                            msgbuflen = strlen(msgbuf);
                        }
                        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
                            && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int
                                == 's' as ::core::ffi::c_int
                        {
                            let mut r: *mut ::core::ffi::c_char = reverse_text(msgbuf);
                            xfree(msgbuf as *mut ::core::ffi::c_void);
                            msgbuf = r;
                            msgbuflen = strlen(msgbuf);
                            while *r as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                                r = r.offset(1);
                            }
                            let mut pat_len: size_t =
                                msgbuf.offset(msgbuflen as isize).offset_from(r) as size_t;
                            memmove(
                                msgbuf as *mut ::core::ffi::c_void,
                                r as *const ::core::ffi::c_void,
                                pat_len,
                            );
                            if r.offset_from(msgbuf) as size_t >= pat_len {
                                memset(
                                    r as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    pat_len,
                                );
                            } else {
                                memset(
                                    msgbuf.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    r.offset_from(msgbuf) as size_t,
                                );
                            }
                        }
                        msg_outtrans(msgbuf, 0 as ::core::ffi::c_int, false_0 != 0);
                        msg_clr_eos();
                        msg_check();
                        gotocmdline(false_0 != 0);
                        ui_flush();
                        ui_busy_stop();
                        msg_nowait.set(true_0 != 0);
                    }
                    if !shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) {
                        show_search_stats = true_0 != 0;
                    }
                }
                if !(*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line
                    && (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off != 0
                    && pos.col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                {
                    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off > 0 as int64_t {
                        c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                        while c != 0 {
                            if decl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                break;
                            }
                            c -= 1;
                        }
                        if c != 0 {
                            pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                            pos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                        }
                    } else {
                        c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                        while c != 0 {
                            if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                break;
                            }
                            c += 1;
                        }
                        if c != 0 {
                            pos.lnum = (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T;
                            pos.col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                    }
                }
                c = searchit(
                    curwin.get(),
                    curbuf.get(),
                    &raw mut pos,
                    ::core::ptr::null_mut::<pos_T>(),
                    (if dirc == '/' as ::core::ffi::c_int {
                        FORWARD as ::core::ffi::c_int
                    } else {
                        BACKWARD as ::core::ffi::c_int
                    }) as Direction,
                    searchstr,
                    searchstrlen,
                    count,
                    (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end as ::core::ffi::c_int
                        * SEARCH_END as ::core::ffi::c_int
                        + (options
                            & SEARCH_KEEP as ::core::ffi::c_int
                                + SEARCH_PEEK as ::core::ffi::c_int
                                + SEARCH_HIS as ::core::ffi::c_int
                                + SEARCH_MSG as ::core::ffi::c_int
                                + SEARCH_START as ::core::ffi::c_int
                                + (if !pat.is_null()
                                    && *pat as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                                {
                                    0 as ::core::ffi::c_int
                                } else {
                                    SEARCH_NOOF as ::core::ffi::c_int
                                })),
                    RE_LAST as ::core::ffi::c_int,
                    sia,
                ) as int64_t;
                if !dircp.is_null() {
                    *dircp = search_delim as ::core::ffi::c_char;
                }
                if !shortmess(SHM_SEARCH as ::core::ffi::c_int)
                    && !sia.is_null()
                    && (*sia).sa_wrapped != 0
                {
                    show_top_bot_msg = true_0 != 0;
                }
                if c == FAIL as int64_t {
                    retval = 0 as ::core::ffi::c_int;
                    break '_end_do_search;
                } else {
                    if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.end
                        as ::core::ffi::c_int
                        != 0
                        && !oap.is_null()
                    {
                        (*oap).inclusive = true_0 != 0;
                    }
                    retval = 1 as ::core::ffi::c_int;
                    if !sia.is_null() && (*sia).sa_wrapped != 0 {
                        apply_autocmds(
                            EVENT_SEARCHWRAPPED,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            ::core::ptr::null_mut::<buf_T>(),
                        );
                    }
                    if options & SEARCH_NOOF as ::core::ffi::c_int == 0
                        || !pat.is_null() && *pat as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    {
                        let mut org_pos: pos_T = pos;
                        if (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.line {
                            c = pos.lnum as int64_t
                                + (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                            if c < 1 as int64_t {
                                pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                            } else if c > (*curbuf.get()).b_ml.ml_line_count as int64_t {
                                pos.lnum = (*curbuf.get()).b_ml.ml_line_count;
                            } else {
                                pos.lnum = c as linenr_T;
                            }
                            pos.col = 0 as ::core::ffi::c_int as colnr_T;
                            retval = 2 as ::core::ffi::c_int;
                        } else if pos.col < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int {
                            c = (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.off;
                            if c > 0 as int64_t {
                                loop {
                                    let c2rust_fresh5 = c;
                                    c = c - 1;
                                    if c2rust_fresh5 <= 0 as int64_t {
                                        break;
                                    }
                                    if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            } else {
                                loop {
                                    let c2rust_fresh6 = c;
                                    c = c + 1;
                                    if c2rust_fresh6 >= 0 as int64_t {
                                        break;
                                    }
                                    if decl(&raw mut pos) == -1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                        }
                        if !equalpos(pos, org_pos) {
                            has_offset = true_0 != 0;
                        }
                    }
                    if show_search_stats {
                        cmdline_search_stat(
                            dirc,
                            &raw mut pos,
                            &raw mut (*curwin.get()).w_cursor,
                            show_top_bot_msg,
                            msgbuf,
                            msgbuflen,
                            count != 1 as ::core::ffi::c_int
                                || has_offset as ::core::ffi::c_int != 0
                                || fdo_flags.get()
                                    & kOptFdoFlagSearch as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                    == 0
                                    && hasFolding(
                                        curwin.get(),
                                        (*curwin.get()).w_cursor.lnum,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                        ::core::ptr::null_mut::<linenr_T>(),
                                    ) as ::core::ffi::c_int
                                        != 0,
                            p_msc.get() as ::core::ffi::c_int,
                            SEARCH_STAT_DEF_TIMEOUT as ::core::ffi::c_int,
                        );
                    }
                    if options & SEARCH_OPT as ::core::ffi::c_int == 0
                        || pat.is_null()
                        || *pat as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                    {
                        break;
                    }
                    pat = pat.offset(1);
                    dirc = *pat as uint8_t as ::core::ffi::c_int;
                    search_delim = dirc;
                    if dirc != '?' as ::core::ffi::c_int && dirc != '/' as ::core::ffi::c_int {
                        retval = 0 as ::core::ffi::c_int;
                        emsg(gettext(b"E386: Expected '?' or '/'  after ';'\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        break '_end_do_search;
                    } else {
                        pat = pat.offset(1);
                        patlen = patlen.wrapping_sub(1);
                    }
                }
            }
            if options & SEARCH_MARK as ::core::ffi::c_int != 0 {
                setpcmark();
            }
            (*curwin.get()).w_cursor = pos;
            (*curwin.get()).w_set_curswant = true_0;
        }
        if options & SEARCH_KEEP as ::core::ffi::c_int != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0
        {
            (*spats.ptr())[0 as ::core::ffi::c_int as usize].off = old_off;
        }
        xfree(strcopy as *mut ::core::ffi::c_void);
        xfree(msgbuf as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn searchc(mut cap: *mut cmdarg_T, mut t_cmd: bool) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = (*cap).nchar;
        let mut dir: ::core::ffi::c_int = (*cap).arg;
        let mut count: ::core::ffi::c_int = (*cap).count1;
        let mut stop: bool = true_0 != 0;
        if c != NUL {
            if KeyStuffed.get() == 0 {
                *(lastc.ptr() as *mut uint8_t) = c as uint8_t;
                set_csearch_direction(dir as Direction);
                set_csearch_until(t_cmd as ::core::ffi::c_int);
                if (*cap).nchar_len != 0 {
                    lastc_bytelen.set((*cap).nchar_len);
                    memcpy(
                        lastc_bytes.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        &raw mut (*cap).nchar_composing as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        (*cap).nchar_len as size_t,
                    );
                } else {
                    lastc_bytelen.set(utf_char2bytes(
                        c,
                        lastc_bytes.ptr() as *mut ::core::ffi::c_char,
                    ));
                }
            }
        } else {
            if *(lastc.ptr() as *mut uint8_t) as ::core::ffi::c_int == NUL
                && lastc_bytelen.get() <= 1 as ::core::ffi::c_int
            {
                return FAIL;
            }
            dir = if dir != 0 {
                -(lastcdir.get() as ::core::ffi::c_int)
            } else {
                lastcdir.get() as ::core::ffi::c_int
            };
            t_cmd = last_t_cmd.get();
            c = *(lastc.ptr() as *mut uint8_t) as ::core::ffi::c_int;
            if vim_strchr(p_cpo.get(), CPO_SCOLON).is_null()
                && count == 1 as ::core::ffi::c_int
                && t_cmd as ::core::ffi::c_int != 0
            {
                stop = false_0 != 0;
            }
        }
        (*(*cap).oap).inclusive = dir != BACKWARD as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        let mut len: ::core::ffi::c_int = get_cursor_line_len();
        loop {
            let c2rust_fresh7 = count;
            count = count - 1;
            if c2rust_fresh7 == 0 {
                break;
            }
            loop {
                if dir > 0 as ::core::ffi::c_int {
                    col += utfc_ptr2len(p.offset(col as isize));
                    if col >= len {
                        return FAIL;
                    }
                } else {
                    if col == 0 as ::core::ffi::c_int {
                        return FAIL;
                    }
                    col -= utf_head_off(
                        p,
                        p.offset(col as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) + 1 as ::core::ffi::c_int;
                }
                if lastc_bytelen.get() <= 1 as ::core::ffi::c_int {
                    if *p.offset(col as isize) as ::core::ffi::c_int == c
                        && stop as ::core::ffi::c_int != 0
                    {
                        break;
                    }
                } else if strncmp(
                    p.offset(col as isize),
                    lastc_bytes.ptr() as *mut ::core::ffi::c_char,
                    lastc_bytelen.get() as size_t,
                ) == 0 as ::core::ffi::c_int
                    && stop as ::core::ffi::c_int != 0
                {
                    break;
                }
                stop = true_0 != 0;
            }
        }
        if t_cmd {
            col -= dir;
            if dir < 0 as ::core::ffi::c_int {
                col += lastc_bytelen.get() - 1 as ::core::ffi::c_int;
            } else {
                col -= utf_head_off(p, p.offset(col as isize));
            }
        }
        (*curwin.get()).w_cursor.col = col as colnr_T;
        return OK;
    }
}

pub unsafe extern "C" fn showmatch(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut lpos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut vcol: colnr_T = 0;
        let mut so: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_so >= 0 as OptInt {
            &raw mut (*curwin.get()).w_onebuf_opt.wo_so
        } else {
            p_so.ptr()
        };
        let mut siso: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_siso >= 0 as OptInt {
            &raw mut (*curwin.get()).w_onebuf_opt.wo_siso
        } else {
            p_siso.ptr()
        };
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = (*curbuf.get()).b_p_mps;
        while *p as ::core::ffi::c_int != NUL {
            if utf_ptr2char(p) == c && (*curwin.get()).w_onebuf_opt.wo_rl ^ p_ri.get() != 0 {
                break;
            }
            p = p.offset((utfc_ptr2len(p) + 1 as ::core::ffi::c_int) as isize);
            if utf_ptr2char(p) == c && (*curwin.get()).w_onebuf_opt.wo_rl ^ p_ri.get() == 0 {
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
            if *p as ::core::ffi::c_int == NUL {
                return;
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            return;
        }
        lpos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
        if lpos.is_null() {
            vim_beep(kOptBoFlagShowmatch as ::core::ffi::c_int as ::core::ffi::c_uint);
            return;
        }
        if (*lpos).lnum < (*curwin.get()).w_topline || (*lpos).lnum >= (*curwin.get()).w_botline {
            return;
        }
        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
            getvcol(
                curwin.get(),
                lpos,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol,
                ::core::ptr::null_mut::<colnr_T>(),
            );
        }
        let mut col_visible: bool = (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            || vcol >= (*curwin.get()).w_leftcol
                && vcol
                    < (*curwin.get()).w_leftcol as ::core::ffi::c_int
                        + (*curwin.get()).w_view_width;
        if !col_visible {
            return;
        }
        let mut mpos: pos_T = *lpos;
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        let mut save_so: OptInt = *so;
        let mut save_siso: OptInt = *siso;
        if dollar_vcol.get() >= 0 as ::core::ffi::c_int
            && dollar_vcol.get() == (*curwin.get()).w_virtcol
        {
            dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
        }
        (*curwin.get()).w_virtcol += 1;
        let mut save_dollar_vcol: colnr_T = dollar_vcol.get();
        let mut save_state: ::core::ffi::c_int = State.get();
        State.set(MODE_SHOWMATCH);
        ui_cursor_shape();
        (*curwin.get()).w_cursor = mpos;
        *so = 0 as OptInt;
        *siso = 0 as OptInt;
        show_cursor_info_later(false_0 != 0);
        update_screen();
        setcursor();
        ui_flush();
        dollar_vcol.set(save_dollar_vcol);
        if !vim_strchr(p_cpo.get(), CPO_SHOWMATCH).is_null() {
            os_delay(
                (p_mat.get() as uint64_t)
                    .wrapping_mul(100 as uint64_t)
                    .wrapping_add(8 as uint64_t),
                true_0 != 0,
            );
        } else if !char_avail() {
            os_delay(
                (p_mat.get() as uint64_t)
                    .wrapping_mul(100 as uint64_t)
                    .wrapping_add(9 as uint64_t),
                false_0 != 0,
            );
        }
        (*curwin.get()).w_cursor = save_cursor;
        *so = save_so;
        *siso = save_siso;
        State.set(save_state);
        ui_cursor_shape();
    }
}

pub unsafe extern "C" fn current_search(
    mut count: ::core::ffi::c_int,
    mut forward: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut old_p_ws: bool = p_ws.get() != 0;
        let mut save_VIsual: pos_T = VIsual.get();
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
        {
            dec_cursor();
        }
        let skip_first_backward: bool = forward as ::core::ffi::c_int != 0
            && VIsual_active.get() as ::core::ffi::c_int != 0
            && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0;
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        let mut orig_pos: pos_T = (*curwin.get()).w_cursor;
        if VIsual_active.get() {
            if forward {
                incl(&raw mut pos);
            } else {
                decl(&raw mut pos);
            }
        }
        let mut zero_width: ::core::ffi::c_int = is_zero_width(
            (*spats.ptr())[last_idx.get() as usize].pat,
            (*spats.ptr())[last_idx.get() as usize].patlen,
            true_0 != 0,
            &raw mut (*curwin.get()).w_cursor,
            FORWARD,
        );
        if zero_width == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        let mut end_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut result: ::core::ffi::c_int = 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 2 as ::core::ffi::c_int {
            let mut dir: ::core::ffi::c_int = 0;
            's_71: {
                if forward {
                    if i == 0 as ::core::ffi::c_int
                        && skip_first_backward as ::core::ffi::c_int != 0
                    {
                        break 's_71;
                    } else {
                        dir = i;
                    }
                } else {
                    dir = (i == 0) as ::core::ffi::c_int;
                }
                let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if dir == 0 && zero_width == 0 {
                    flags = SEARCH_END as ::core::ffi::c_int;
                }
                end_pos = pos;
                if i == 0 as ::core::ffi::c_int {
                    p_ws.set(false_0);
                }
                result = searchit(
                    curwin.get(),
                    curbuf.get(),
                    &raw mut pos,
                    &raw mut end_pos,
                    (if dir != 0 {
                        FORWARD as ::core::ffi::c_int
                    } else {
                        BACKWARD as ::core::ffi::c_int
                    }) as Direction,
                    (*spats.ptr())[last_idx.get() as usize].pat,
                    (*spats.ptr())[last_idx.get() as usize].patlen,
                    if i != 0 {
                        count
                    } else {
                        1 as ::core::ffi::c_int
                    },
                    SEARCH_KEEP as ::core::ffi::c_int | flags,
                    RE_SEARCH as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<searchit_arg_T>(),
                );
                p_ws.set(old_p_ws as ::core::ffi::c_int);
                if i == 1 as ::core::ffi::c_int && result == 0 {
                    (*curwin.get()).w_cursor = orig_pos;
                    if VIsual_active.get() {
                        VIsual.set(save_VIsual);
                    }
                    return FAIL;
                } else if i == 0 as ::core::ffi::c_int && result == 0 {
                    if forward {
                        clearpos(&mut pos);
                    } else {
                        pos.lnum = (*(*curwin.get()).w_buffer).b_ml.ml_line_count;
                        pos.col = ml_get_len((*(*curwin.get()).w_buffer).b_ml.ml_line_count);
                    }
                }
            }
            i += 1;
        }
        let mut start_pos: pos_T = pos;
        if !VIsual_active.get() {
            VIsual.set(start_pos);
        }
        (*curwin.get()).w_cursor = end_pos;
        if lt(VIsual.get(), end_pos) as ::core::ffi::c_int != 0
            && forward as ::core::ffi::c_int != 0
        {
            if skip_first_backward {
                (*curwin.get()).w_cursor = pos;
            } else {
                dec_cursor();
            }
        } else if VIsual_active.get() as ::core::ffi::c_int != 0
            && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
            && forward as ::core::ffi::c_int != 0
        {
            (*curwin.get()).w_cursor = pos;
        }
        VIsual_active.set(true_0 != 0);
        VIsual_mode.set('v' as ::core::ffi::c_int);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if forward as ::core::ffi::c_int != 0
                && ltoreq(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
            {
                inc_cursor();
            } else if !forward
                && ltoreq((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
            {
                inc(VIsual.ptr());
            }
        }
        if fdo_flags.get() & kOptFdoFlagSearch as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && KeyTyped.get() as ::core::ffi::c_int != 0
        {
            foldOpenCursor();
        }
        may_start_select('c' as ::core::ffi::c_int);
        setmouse();
        redraw_curbuf_later(UPD_INVERTED);
        showmode();
        return OK;
    }
}
