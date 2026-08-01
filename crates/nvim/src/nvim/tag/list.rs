//! Showing the matches to the user.
//!
//! [`print_tag_list`] is the numbered listing `:tselect` prompts with, and
//! [`add_llist_tags`] is the same information as a location list for
//! `:ltag`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn print_tag_list(
    mut new_tag: bool,
    mut use_tagstack: bool,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut tagstack: *mut taggy_T = &raw mut (*curwin.get()).w_tagstack as *mut taggy_T;
        let mut tagstackidx: ::core::ffi::c_int = (*curwin.get()).w_tagstackidx;
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
        parse_match(
            *matches.offset(0 as ::core::ffi::c_int as isize),
            &raw mut tagp,
        );
        let mut taglen: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
            + 2 as isize) as ::core::ffi::c_int
            > 18 as ::core::ffi::c_int
        {
            (tagp.tagname_end.offset_from(tagp.tagname) + 2 as isize) as ::core::ffi::c_int
        } else {
            18 as ::core::ffi::c_int
        };
        if taglen > Columns.get() - 25 as ::core::ffi::c_int {
            taglen = MAXCOL as ::core::ffi::c_int;
        }
        if msg_col.get() == 0 as ::core::ffi::c_int {
            msg_didout.set(false_0 != 0);
        }
        msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
        msg_start();
        msg_puts_hl(
            gettext(b"  # pri kind tag\0".as_ptr() as *const ::core::ffi::c_char),
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_clr_eos();
        taglen_advance(taglen);
        msg_puts_hl(
            gettext(b"file\n\0".as_ptr() as *const ::core::ffi::c_char),
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_matches && !got_int.get() {
            parse_match(*matches.offset(i as isize), &raw mut tagp);
            if !new_tag
                && (g_do_tagpreview.get() != 0 as ::core::ffi::c_int
                    && i == (*ptag_entry.ptr()).cur_match
                    || use_tagstack as ::core::ffi::c_int != 0
                        && i == (*tagstack.offset(tagstackidx as isize)).cur_match)
            {
                *(IObuff.ptr() as *mut ::core::ffi::c_char) = '>' as ::core::ffi::c_char;
            } else {
                *(IObuff.ptr() as *mut ::core::ffi::c_char) = ' ' as ::core::ffi::c_char;
            }
            vim_snprintf(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
                (IOSIZE - 1 as ::core::ffi::c_int) as size_t,
                b"%2d %s \0".as_ptr() as *const ::core::ffi::c_char,
                i + 1 as ::core::ffi::c_int,
                (*mt_names.ptr())[(*(*matches.offset(i as isize))
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & MT_MASK as ::core::ffi::c_int) as usize],
            );
            msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
            if !tagp.tagkind.is_null() {
                msg_outtrans_len(
                    tagp.tagkind,
                    tagp.tagkind_end.offset_from(tagp.tagkind) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
            msg_advance(13 as ::core::ffi::c_int);
            msg_outtrans_len(
                tagp.tagname,
                tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int,
                HLF_T as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_putchar(' ' as ::core::ffi::c_int);
            taglen_advance(taglen);
            let mut p: *const ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
            if !p.is_null() {
                msg_outtrans(p, HLF_D as ::core::ffi::c_int, false_0 != 0);
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut p as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            }
            if msg_col.get() > 0 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            if got_int.get() {
                break;
            }
            msg_advance(15 as ::core::ffi::c_int);
            let mut command_end: *const ::core::ffi::c_char = tagp.command_end;
            if !command_end.is_null() {
                p = command_end.offset(3 as ::core::ffi::c_int as isize);
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                {
                    while *p as ::core::ffi::c_int == TAB {
                        p = p.offset(1);
                    }
                    if strncmp(
                        p,
                        b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                        && ascii_isspace(
                            *p.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        p = p.offset(5 as ::core::ffi::c_int as isize);
                    } else if p == tagp.tagkind as *const ::core::ffi::c_char
                        || p.offset(5 as ::core::ffi::c_int as isize)
                            == tagp.tagkind as *const ::core::ffi::c_char
                            && strncmp(
                                p,
                                b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                    {
                        p = tagp.tagkind_end;
                    } else {
                        let mut hl_id: ::core::ffi::c_int = HLF_CM as ::core::ffi::c_int;
                        while *p as ::core::ffi::c_int != 0
                            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        {
                            if msg_col.get() + ptr2cells(p) >= Columns.get() {
                                msg_putchar('\n' as ::core::ffi::c_int);
                                if got_int.get() {
                                    break;
                                }
                                msg_advance(15 as ::core::ffi::c_int);
                            }
                            p = msg_outtrans_one(p, hl_id, false_0 != 0);
                            if *p as ::core::ffi::c_int == TAB {
                                msg_puts_hl(
                                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                break;
                            } else if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                                hl_id = 0 as ::core::ffi::c_int;
                            }
                        }
                    }
                }
                if msg_col.get() > 15 as ::core::ffi::c_int {
                    msg_putchar('\n' as ::core::ffi::c_int);
                    if got_int.get() {
                        break;
                    }
                    msg_advance(15 as ::core::ffi::c_int);
                }
            } else {
                p = tagp.command;
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
                command_end = p;
            }
            p = tagp.command;
            if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int
            {
                p = p.offset(1);
                if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                    p = p.offset(1);
                }
            }
            while p != command_end
                && ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                p = p.offset(1);
            }
            while p != command_end {
                if msg_col.get()
                    + (if *p as ::core::ffi::c_int == TAB {
                        1 as ::core::ffi::c_int
                    } else {
                        ptr2cells(p)
                    })
                    > Columns.get()
                {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                if got_int.get() {
                    break;
                }
                msg_advance(15 as ::core::ffi::c_int);
                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == *tagp.command as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int)
                {
                    p = p.offset(1);
                }
                if *p as ::core::ffi::c_int == TAB {
                    msg_putchar(' ' as ::core::ffi::c_int);
                    p = p.offset(1);
                } else {
                    p = msg_outtrans_one(p, 0 as ::core::ffi::c_int, false_0 != 0);
                }
                if p == command_end.offset(-(2 as ::core::ffi::c_int as isize))
                    && *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == *tagp.command as ::core::ffi::c_int
                {
                    break;
                }
                if p == command_end.offset(-(1 as ::core::ffi::c_int as isize))
                    && *p as ::core::ffi::c_int == *tagp.command as ::core::ffi::c_int
                    && (*p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int)
                {
                    break;
                }
            }
            if msg_col.get() != 0
                && (!ui_has(kUIMessages) || i < num_matches - 1 as ::core::ffi::c_int)
            {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            os_breakcheck();
            i += 1;
        }
        if got_int.get() {
            got_int.set(false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn add_llist_tags(
    mut tag: *mut ::core::ffi::c_char,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tag_name: [::core::ffi::c_char; 129] = [0; 129];
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
        let mut fname: *mut ::core::ffi::c_char =
            xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        let mut cmd: *mut ::core::ffi::c_char =
            xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        let mut list: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_matches {
            let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
            parse_match(*matches.offset(i as isize), &raw mut tagp);
            let mut len: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
                as ::core::ffi::c_int)
                < 128 as ::core::ffi::c_int
            {
                tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int
            } else {
                128 as ::core::ffi::c_int
            };
            xmemcpyz(
                &raw mut tag_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                tagp.tagname as *const ::core::ffi::c_void,
                len as size_t,
            );
            tag_name[len as usize] = NUL as ::core::ffi::c_char;
            let mut p: *mut ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
            if !p.is_null() {
                xstrlcpy(fname, p, MAXPATHL as size_t);
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut p as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
                let mut lnum: linenr_T = 0 as linenr_T;
                if *(*__ctype_b_loc())
                    .offset(*tagp.command as uint8_t as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                {
                    lnum = atoi(tagp.command) as linenr_T;
                } else {
                    let mut cmd_start: *mut ::core::ffi::c_char = tagp.command;
                    let mut cmd_end: *mut ::core::ffi::c_char = tagp.command_end;
                    if cmd_end.is_null() {
                        p = tagp.command;
                        while *p as ::core::ffi::c_int != 0
                            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                        }
                        cmd_end = p;
                    }
                    cmd_end = cmd_end.offset(-1);
                    if *cmd_start as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                        || *cmd_start as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                    {
                        cmd_start = cmd_start.offset(1);
                    }
                    if *cmd_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                        || *cmd_end as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                    {
                        cmd_end = cmd_end.offset(-1);
                    }
                    len = 0 as ::core::ffi::c_int;
                    *cmd.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                    if *cmd_start as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                        strcpy(
                            cmd,
                            b"^\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        cmd_start = cmd_start.offset(1);
                        len += 1;
                    }
                    strcat(cmd, b"\\V\0".as_ptr() as *const ::core::ffi::c_char);
                    len += 2 as ::core::ffi::c_int;
                    let mut cmd_len: ::core::ffi::c_int =
                        if ((cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int)
                            < 1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                        {
                            (cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int
                        } else {
                            1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                        };
                    snprintf(
                        cmd.offset(len as isize),
                        (CMDBUFFSIZE + 1 as ::core::ffi::c_int - len) as size_t,
                        b"%.*s\0".as_ptr() as *const ::core::ffi::c_char,
                        cmd_len,
                        cmd_start,
                    );
                    len += cmd_len;
                    if *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        == '$' as ::core::ffi::c_int
                    {
                        *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) =
                            '\\' as ::core::ffi::c_char;
                        *cmd.offset(len as isize) = '$' as ::core::ffi::c_char;
                        len += 1;
                    }
                    *cmd.offset(len as isize) = NUL as ::core::ffi::c_char;
                }
                dict = tv_dict_alloc();
                tv_list_append_dict(list, dict);
                tv_dict_add_str(
                    dict,
                    b"text\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                    &raw mut tag_name as *mut ::core::ffi::c_char,
                );
                tv_dict_add_str(
                    dict,
                    b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    fname,
                );
                tv_dict_add_nr(
                    dict,
                    b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                    lnum as varnumber_T,
                );
                if lnum == 0 as linenr_T {
                    tv_dict_add_str(
                        dict,
                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        cmd,
                    );
                }
            }
            i += 1;
        }
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"ltag %s\0".as_ptr() as *const ::core::ffi::c_char,
            tag,
        );
        set_errorlist(
            curwin.get(),
            list,
            ' ' as ::core::ffi::c_int,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<dict_T>(),
        );
        tv_list_free(list);
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut fname as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut cmd as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        let _ = *ptr__1;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn taglen_advance(mut l: ::core::ffi::c_int) {
    unsafe {
        if l == MAXCOL as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
            msg_advance(24 as ::core::ffi::c_int);
        } else {
            msg_advance(13 as ::core::ffi::c_int + l);
        };
    }
}
