//! Going to a tag.
//!
//! [`jumpto_tag`] takes one matching line, opens the file it names and runs
//! the search command (or line number) that follows, then puts the cursor
//! on the tag. [`parse_match`] and [`parse_tag_line`] are the readers for
//! the two tags-file line formats, and [`find_extra`] finds the optional
//! extra fields at the end of a line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn parse_tag_line(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*tagp).tagname = lbuf;
        let mut p: *mut ::core::ffi::c_char = vim_strchr(lbuf, TAB);
        if p.is_null() {
            return FAIL;
        }
        (*tagp).tagname_end = p;
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
        (*tagp).fname = p;
        p = vim_strchr(p, TAB);
        if p.is_null() {
            return FAIL;
        }
        (*tagp).fname_end = p;
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        (*tagp).command = p;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn test_for_static(mut tagp: *mut tagptrs_T) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = (*tagp).command;
        loop {
            p = vim_strchr(p, '\t' as ::core::ffi::c_int);
            if p.is_null() {
                break;
            }
            p = p.offset(1);
            if strncmp(
                p,
                b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn matching_line_len(lbuf: *const ::core::ffi::c_char) -> size_t {
    unsafe {
        let mut p: *const ::core::ffi::c_char = lbuf.offset(1 as ::core::ffi::c_int as isize);
        p = p.offset(strlen(p).wrapping_add(1 as size_t) as isize);
        return (p.offset_from(lbuf) as size_t).wrapping_add(strlen(p));
    }
}

pub(crate) unsafe extern "C" fn parse_match(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*tagp).tag_fname = lbuf.offset(1 as ::core::ffi::c_int as isize);
        lbuf = lbuf.offset(strlen((*tagp).tag_fname).wrapping_add(2 as size_t) as isize);
        let mut retval: ::core::ffi::c_int = parse_tag_line(lbuf, tagp);
        (*tagp).tagkind = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*tagp).user_data = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*tagp).tagline = 0 as ::core::ffi::c_int as linenr_T;
        (*tagp).command_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if retval != OK {
            return retval;
        }
        let mut p: *mut ::core::ffi::c_char = (*tagp).command;
        if find_extra(&raw mut p) == OK {
            (*tagp).command_end = p;
            if p > (*tagp).command
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '|' as ::core::ffi::c_int
            {
                (*tagp).command_end = p.offset(-(1 as ::core::ffi::c_int as isize));
            }
            p = p.offset(2 as ::core::ffi::c_int as isize);
            let c2rust_fresh3 = p;
            p = p.offset(1);
            if *c2rust_fresh3 as ::core::ffi::c_int == TAB {
                while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    || utfc_ptr2len(p) > 1 as ::core::ffi::c_int
                {
                    if strncmp(
                        p,
                        b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*tagp).tagkind = p.offset(5 as ::core::ffi::c_int as isize);
                    } else if strncmp(
                        p,
                        b"user_data:\0".as_ptr() as *const ::core::ffi::c_char,
                        10 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*tagp).user_data = p.offset(10 as ::core::ffi::c_int as isize);
                    } else if strncmp(
                        p,
                        b"line:\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*tagp).tagline =
                            atoi(p.offset(5 as ::core::ffi::c_int as isize)) as linenr_T;
                    }
                    if !(*tagp).tagkind.is_null() && !(*tagp).user_data.is_null() {
                        break;
                    }
                    let mut pc: *mut ::core::ffi::c_char = vim_strchr(p, ':' as ::core::ffi::c_int);
                    let mut pt: *mut ::core::ffi::c_char =
                        vim_strchr(p, '\t' as ::core::ffi::c_int);
                    if pc.is_null() || !pt.is_null() && pc > pt {
                        (*tagp).tagkind = p;
                    }
                    if pt.is_null() {
                        break;
                    }
                    p = pt;
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            }
        }
        if !(*tagp).tagkind.is_null() {
            p = (*tagp).tagkind;
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            (*tagp).tagkind_end = p;
        }
        if !(*tagp).user_data.is_null() {
            p = (*tagp).user_data;
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            (*tagp).user_data_end = p;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn tag_full_fname(
    mut tagp: *mut tagptrs_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut c: ::core::ffi::c_char = *(*tagp).fname_end;
        *(*tagp).fname_end = NUL as ::core::ffi::c_char;
        let mut fullname: *mut ::core::ffi::c_char =
            expand_tag_fname((*tagp).fname, (*tagp).tag_fname, false_0 != 0);
        *(*tagp).fname_end = c;
        return fullname;
    }
}

pub(crate) unsafe extern "C" fn jumpto_tag(
    mut lbuf_arg: *const ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut keep_help: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if postponed_split.get() == 0 as ::core::ffi::c_int
            && !check_can_set_curbuf_forceit(forceit)
        {
            return FAIL;
        }
        let mut pbuf_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tofree_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tagp: tagptrs_T = tagptrs_T {
            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagline: 0,
        };
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut getfile_result: ::core::ffi::c_int = GETFILE_UNUSED as ::core::ffi::c_int;
        let mut search_options: ::core::ffi::c_int = 0;
        let mut curwin_save: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut full_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let old_KeyTyped: bool = KeyTyped.get();
        let l_g_do_tagpreview: ::core::ffi::c_int = g_do_tagpreview.get();
        let len: size_t = matching_line_len(lbuf_arg).wrapping_add(1 as size_t);
        let mut lbuf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        memmove(
            lbuf as *mut ::core::ffi::c_void,
            lbuf_arg as *const ::core::ffi::c_void,
            len,
        );
        let mut pbuf: *mut ::core::ffi::c_char =
            xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
        '_erret: {
            if parse_match(lbuf, &raw mut tagp) == FAIL {
                tagp.fname_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                *tagp.fname_end = NUL as ::core::ffi::c_char;
                fname = tagp.fname;
                str = tagp.command;
                pbuf_end = pbuf;
                while *str as ::core::ffi::c_int != 0
                    && *str as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    && *str as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                {
                    let c2rust_fresh1 = str;
                    str = str.offset(1);
                    let c2rust_fresh2 = pbuf_end;
                    pbuf_end = pbuf_end.offset(1);
                    *c2rust_fresh2 = *c2rust_fresh1;
                    if pbuf_end.offset_from(pbuf) + 1 as isize
                        >= LSIZE as ::core::ffi::c_int as isize
                    {
                        break;
                    }
                }
                *pbuf_end = NUL as ::core::ffi::c_char;
                str = pbuf;
                if find_extra(&raw mut str) == OK {
                    pbuf_end = str;
                    *pbuf_end = NUL as ::core::ffi::c_char;
                }
                fname = expand_tag_fname(fname, tagp.tag_fname, true_0 != 0);
                tofree_fname = fname;
                if !os_path_exists(fname)
                    && !has_autocmd(EVENT_BUFREADCMD, fname, ::core::ptr::null_mut::<buf_T>())
                {
                    retval = NOTAGFILE;
                    xfree(nofile_fname.get() as *mut ::core::ffi::c_void);
                    nofile_fname.set(xstrdup(fname));
                } else {
                    (*RedrawingDisabled.ptr()) += 1;
                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                        postponed_split.set(0 as ::core::ffi::c_int);
                        curwin_save = curwin.get();
                        if (*curwin.get()).w_onebuf_opt.wo_pvw == 0 {
                            full_fname = FullName_save(fname, false_0 != 0);
                            fname = full_fname;
                            prepare_tagpreview(true_0 != 0);
                        }
                    }
                    if postponed_split.get() != 0
                        && swb_flags.get()
                            & (kOptSwbFlagUseopen as ::core::ffi::c_int
                                | kOptSwbFlagUsetab as ::core::ffi::c_int)
                                as ::core::ffi::c_uint
                            != 0
                    {
                        let existing_buf: *mut buf_T = buflist_findname_exp(fname);
                        if !existing_buf.is_null() {
                            if !swbuf_goto_win_with_buf(existing_buf).is_null() {
                                getfile_result = GETFILE_SAME_FILE as ::core::ffi::c_int;
                            }
                        }
                    }
                    if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int
                        && (postponed_split.get() != 0
                            || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int)
                    {
                        if swb_flags.get()
                            & kOptSwbFlagVsplit as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                        {
                            (*cmdmod.ptr()).cmod_split |= WSP_VERT as ::core::ffi::c_int;
                        }
                        if swb_flags.get()
                            & kOptSwbFlagNewtab as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                            && (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int
                        {
                            (*cmdmod.ptr()).cmod_tab =
                                tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                        }
                        if win_split(
                            if postponed_split.get() > 0 as ::core::ffi::c_int {
                                postponed_split.get()
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            postponed_split_flags.get(),
                        ) == FAIL
                        {
                            (*RedrawingDisabled.ptr()) -= 1;
                            break '_erret;
                        } else {
                            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
                        }
                    }
                    if keep_help {
                        if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                            keep_help_flag.set(bt_help((*curwin_save).w_buffer));
                        } else {
                            keep_help_flag.set((*curbuf.get()).b_help);
                        }
                    }
                    if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int {
                        getfile_result = getfile(
                            0 as ::core::ffi::c_int,
                            fname,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            true_0 != 0,
                            0 as linenr_T,
                            forceit != 0,
                        );
                    }
                    keep_help_flag.set(false_0 != 0);
                    if getfile_result <= 0 as ::core::ffi::c_int {
                        (*curwin.get()).w_set_curswant = true_0;
                        postponed_split.set(0 as ::core::ffi::c_int);
                        let save_magic_overruled: optmagic_T = magic_overruled.get();
                        magic_overruled.set(OPTION_MAGIC_OFF);
                        let save_no_hlsearch: bool = no_hlsearch.get();
                        if !vim_strchr(p_cpo.get(), CPO_TAGPAT).is_null() {
                            search_options = 0 as ::core::ffi::c_int;
                        } else {
                            search_options = SEARCH_KEEP as ::core::ffi::c_int;
                        }
                        str = pbuf;
                        if *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                            || *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '?' as ::core::ffi::c_int
                        {
                            str = skip_regexp(
                                pbuf.offset(1 as ::core::ffi::c_int as isize),
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                false_0,
                            )
                            .offset(1 as ::core::ffi::c_int as isize);
                        }
                        if str > pbuf_end.offset(-(1 as ::core::ffi::c_int as isize)) {
                            let mut pbuflen: size_t = pbuf_end.offset_from(pbuf) as size_t;
                            let mut save_p_ws: bool = p_ws.get() != 0;
                            let mut save_p_ic: ::core::ffi::c_int = p_ic.get();
                            let mut save_p_scs: ::core::ffi::c_int = p_scs.get();
                            p_ws.set(true_0);
                            p_ic.set(false_0);
                            p_scs.set(false_0);
                            let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                            (*curwin.get()).w_cursor.lnum = if tagp.tagline > 0 as linenr_T {
                                tagp.tagline - 1 as linenr_T
                            } else {
                                0 as linenr_T
                            };
                            if do_search(
                                ::core::ptr::null_mut::<oparg_T>(),
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                pbuf.offset(1 as ::core::ffi::c_int as isize),
                                pbuflen.wrapping_sub(1 as size_t),
                                1 as ::core::ffi::c_int,
                                search_options,
                                ::core::ptr::null_mut::<searchit_arg_T>(),
                            ) != 0
                            {
                                retval = OK;
                            } else {
                                let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                                p_ic.set(true_0);
                                if do_search(
                                    ::core::ptr::null_mut::<oparg_T>(),
                                    *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                    *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                    pbuf.offset(1 as ::core::ffi::c_int as isize),
                                    pbuflen.wrapping_sub(1 as size_t),
                                    1 as ::core::ffi::c_int,
                                    search_options,
                                    ::core::ptr::null_mut::<searchit_arg_T>(),
                                ) == 0
                                {
                                    found = 2 as ::core::ffi::c_int;
                                    test_for_static(&raw mut tagp);
                                    let mut cc: ::core::ffi::c_char = *tagp.tagname_end;
                                    *tagp.tagname_end = NUL as ::core::ffi::c_char;
                                    pbuflen = snprintf(
                                        pbuf,
                                        LSIZE as ::core::ffi::c_int as size_t,
                                        b"^%s\\s\\*(\0".as_ptr() as *const ::core::ffi::c_char,
                                        tagp.tagname,
                                    ) as size_t;
                                    if do_search(
                                        ::core::ptr::null_mut::<oparg_T>(),
                                        '/' as ::core::ffi::c_int,
                                        '/' as ::core::ffi::c_int,
                                        pbuf,
                                        pbuflen,
                                        1 as ::core::ffi::c_int,
                                        search_options,
                                        ::core::ptr::null_mut::<searchit_arg_T>(),
                                    ) == 0
                                    {
                                        pbuflen = snprintf(
                                            pbuf,
                                            LSIZE as ::core::ffi::c_int as size_t,
                                            b"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            tagp.tagname,
                                        )
                                            as size_t;
                                        if do_search(
                                            ::core::ptr::null_mut::<oparg_T>(),
                                            '/' as ::core::ffi::c_int,
                                            '/' as ::core::ffi::c_int,
                                            pbuf,
                                            pbuflen,
                                            1 as ::core::ffi::c_int,
                                            search_options,
                                            ::core::ptr::null_mut::<searchit_arg_T>(),
                                        ) == 0
                                        {
                                            found = 0 as ::core::ffi::c_int;
                                        }
                                    }
                                    *tagp.tagname_end = cc;
                                }
                                if found == 0 as ::core::ffi::c_int {
                                    emsg(gettext(b"E434: Can't find tag pattern\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                    (*curwin.get()).w_cursor.lnum = save_lnum;
                                } else {
                                    if found == 2 as ::core::ffi::c_int || save_p_ic == 0 {
                                        msg(
                                            gettext(
                                                b"E435: Couldn't find tag, just guessing!\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            0 as ::core::ffi::c_int,
                                        );
                                        if msg_scrolled.get() == 0
                                            && msg_silent.get() == 0 as ::core::ffi::c_int
                                        {
                                            msg_delay(1010 as uint64_t, true_0 != 0);
                                        }
                                    }
                                    retval = OK;
                                }
                            }
                            p_ws.set(save_p_ws as ::core::ffi::c_int);
                            p_ic.set(save_p_ic);
                            p_scs.set(save_p_scs);
                            check_cursor(curwin.get());
                        } else {
                            let save_secure: ::core::ffi::c_int = secure.get();
                            secure.set(1 as ::core::ffi::c_int);
                            (*sandbox.ptr()) += 1;
                            (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                            do_cmdline_cmd(pbuf);
                            retval = OK;
                            if secure.get() == 2 as ::core::ffi::c_int {
                                wait_return(true_0);
                            }
                            secure.set(save_secure);
                            (*sandbox.ptr()) -= 1;
                        }
                        magic_overruled.set(save_magic_overruled);
                        if search_options != 0 {
                            set_no_hlsearch(save_no_hlsearch);
                        }
                        if getfile_result == GETFILE_OPEN_OTHER as ::core::ffi::c_int {
                            retval = OK;
                        }
                        if retval == OK {
                            if (*curbuf.get()).b_help {
                                set_topline(curwin.get(), (*curwin.get()).w_cursor.lnum);
                            }
                            if fdo_flags.get()
                                & kOptFdoFlagTag as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                                && old_KeyTyped as ::core::ffi::c_int != 0
                            {
                                foldOpenCursor();
                            }
                        }
                        if l_g_do_tagpreview != 0 as ::core::ffi::c_int
                            && curwin.get() != curwin_save
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            validate_cursor(curwin.get());
                            redraw_later(curwin.get(), UPD_VALID);
                            win_enter(curwin_save, true_0 != 0);
                        }
                        (*RedrawingDisabled.ptr()) -= 1;
                    } else {
                        (*RedrawingDisabled.ptr()) -= 1;
                        if postponed_split.get() != 0 {
                            win_close(curwin.get(), false_0 != 0, false_0 != 0);
                            postponed_split.set(0 as ::core::ffi::c_int);
                        }
                    }
                }
            }
        }
        g_do_tagpreview.set(0 as ::core::ffi::c_int);
        xfree(lbuf as *mut ::core::ffi::c_void);
        xfree(pbuf as *mut ::core::ffi::c_void);
        xfree(tofree_fname as *mut ::core::ffi::c_void);
        xfree(full_fname as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn test_for_current(
    mut fname: *mut ::core::ffi::c_char,
    mut fname_end: *mut ::core::ffi::c_char,
    mut tag_fname: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = false_0;
        if !buf_ffname.is_null() {
            let mut c: ::core::ffi::c_char = 0;
            c = *fname_end;
            *fname_end = NUL as ::core::ffi::c_char;
            let mut fullname: *mut ::core::ffi::c_char =
                expand_tag_fname(fname, tag_fname, true_0 != 0);
            retval = (path_full_compare(fullname, buf_ffname, true_0 != 0, true_0 != 0)
                as ::core::ffi::c_uint
                & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
            xfree(fullname as *mut ::core::ffi::c_void);
            *fname_end = c;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn find_extra(
    mut pp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut str: *mut ::core::ffi::c_char = *pp;
        let mut first_char: ::core::ffi::c_char = **pp;
        loop {
            if ascii_isdigit(*str as ::core::ffi::c_int) {
                str = skipdigits(str.offset(1 as ::core::ffi::c_int as isize));
            } else if *str as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                || *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int
            {
                str = skip_regexp(
                    str.offset(1 as ::core::ffi::c_int as isize),
                    *str as ::core::ffi::c_int,
                    false_0,
                );
                if *str as ::core::ffi::c_int != first_char as ::core::ffi::c_int {
                    str = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    str = str.offset(1);
                }
            } else {
                str = strstr(str, b"|;\"\0".as_ptr() as *const ::core::ffi::c_char);
                if !str.is_null() {
                    str = str.offset(1);
                    break;
                }
            }
            if str.is_null()
                || *str as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                || !(ascii_isdigit(
                    *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                    || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '?' as ::core::ffi::c_int)
            {
                break;
            }
            str = str.offset(1);
            first_char = *str;
        }
        if !str.is_null()
            && strncmp(
                str,
                b";\"\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            *pp = str;
            return OK;
        }
        return FAIL;
    }
}
