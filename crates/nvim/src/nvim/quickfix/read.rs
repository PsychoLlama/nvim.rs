//! Reading the lines to be parsed, from wherever they come from.
//!
//! [`qf_init_ext`] is the driver every list-building command reaches: it
//! compiles `'errorformat'`, then pulls lines one at a time and files what
//! the parser makes of them. There are four sources — a file, a buffer, a
//! Vimscript list and a single string — behind
//! [`qf_get_nextline`]/[`qf_setup_state`], and
//! [`qf_get_next_file_line`] is the one that has to deal with long lines
//! and character conversion.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::CStr;

pub(crate) unsafe fn qf_init_process_nextline(
    mut qfl: *mut qf_list_T,
    efm: &mut Efm,
    mut state: *mut qfstate_T,
    fields: &mut Fields,
) -> Status {
    unsafe {
        let mut status: ::core::ffi::c_int = qf_get_nextline(state);
        if status != QF_OK as ::core::ffi::c_int {
            return if status == QF_END_OF_INPUT as ::core::ffi::c_int {
                Status::EndOfInput
            } else {
                Status::Fail
            };
        }
        let parsed = parse_line(qfl, (*state).linebuf, (*state).linelen, efm, fields);
        if parsed != Status::Ok {
            return parsed;
        }
        let name = entry_file_name(fields, qfl);
        status = qf_add_entry(
            qfl,
            (*qfl).qf_directory,
            name,
            fields.module(),
            fields.bnr,
            fields.errmsg(),
            fields.lnum,
            fields.end_lnum,
            fields.col,
            fields.end_col,
            fields.use_viscol as ::core::ffi::c_char,
            fields.pattern(),
            fields.enr,
            fields.kind,
            fields.user_data,
            fields.valid as ::core::ffi::c_char,
        );
        if status == QF_OK as ::core::ffi::c_int {
            Status::Ok
        } else {
            Status::Fail
        }
    }
}

pub unsafe fn qf_init(
    mut wp: *mut win_T,
    mut efile: *const ::core::ffi::c_char,
    mut errorformat: *mut ::core::ffi::c_char,
    mut newlist: ::core::ffi::c_int,
    mut qf_title: *const ::core::ffi::c_char,
    mut enc: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qi: *mut qf_info_T = if wp.is_null() {
            ql_info.get()
        } else {
            ll_get_or_alloc_list(wp)
        };
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                390 as ::core::ffi::c_uint,
                b"int qf_init(win_T *, const char *restrict, char *restrict, int, const char *restrict, char *restrict)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        return qf_init_ext(
            qi,
            (*qi).qf_curlist,
            efile,
            curbuf.get(),
            ::core::ptr::null_mut::<typval_T>(),
            errorformat,
            newlist != 0,
            0 as linenr_T,
            0 as linenr_T,
            qf_title,
            enc,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_grow_linebuf(
    mut state: *mut qfstate_T,
    mut newsz: size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        (*state).linelen = if newsz > LINE_MAXLEN.get() {
            (*LINE_MAXLEN.ptr()).wrapping_sub(1 as size_t)
        } else {
            newsz
        };
        if (*state).growbuf.is_null() {
            (*state).growbuf =
                xmalloc((*state).linelen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            (*state).growbufsiz = (*state).linelen;
        } else if (*state).linelen > (*state).growbufsiz {
            (*state).growbuf = xrealloc(
                (*state).growbuf as *mut ::core::ffi::c_void,
                (*state).linelen.wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            (*state).growbufsiz = (*state).linelen;
        }
        return (*state).growbuf;
    }
}

pub(crate) unsafe extern "C" fn qf_get_next_str_line(
    mut state: *mut qfstate_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p_str: *mut ::core::ffi::c_char = (*state).p_str;
        if *p_str as ::core::ffi::c_int == NUL {
            return QF_END_OF_INPUT as ::core::ffi::c_int;
        }
        let mut p: *mut ::core::ffi::c_char = vim_strchr(p_str, '\n' as ::core::ffi::c_int);
        let mut len: size_t = if !p.is_null() {
            (p.offset_from(p_str) as size_t).wrapping_add(1 as size_t)
        } else {
            strlen(p_str)
        };
        if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
            (*state).linebuf = qf_grow_linebuf(state, len);
        } else {
            (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
            (*state).linelen = len;
        }
        memcpy(
            (*state).linebuf as *mut ::core::ffi::c_void,
            p_str as *const ::core::ffi::c_void,
            (*state).linelen,
        );
        *(*state).linebuf.offset((*state).linelen as isize) = NUL as ::core::ffi::c_char;
        p_str = p_str.offset(len as isize);
        (*state).p_str = p_str;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_get_next_list_line(
    mut state: *mut qfstate_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p_li: *mut listitem_T = (*state).p_li;
        while !p_li.is_null()
            && ((*p_li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*p_li).li_tv.vval.v_string.is_null())
        {
            p_li = (*p_li).li_next;
        }
        if p_li.is_null() {
            (*state).p_li = ::core::ptr::null_mut::<listitem_T>();
            return QF_END_OF_INPUT as ::core::ffi::c_int;
        }
        let mut len: size_t = strlen((*p_li).li_tv.vval.v_string);
        if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
            (*state).linebuf = qf_grow_linebuf(state, len);
        } else {
            (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
            (*state).linelen = len;
        }
        xstrlcpy(
            (*state).linebuf,
            (*p_li).li_tv.vval.v_string,
            (*state).linelen.wrapping_add(1 as size_t),
        );
        (*state).p_li = (*p_li).li_next;
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_get_next_buf_line(
    mut state: *mut qfstate_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*state).buflnum > (*state).lnumlast {
            return QF_END_OF_INPUT as ::core::ffi::c_int;
        }
        let mut p_buf: *mut ::core::ffi::c_char = ml_get_buf((*state).buf, (*state).buflnum);
        let mut len: size_t = ml_get_buf_len((*state).buf, (*state).buflnum) as size_t;
        (*state).buflnum =
            ((*state).buflnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
        if len > (IOSIZE - 2 as ::core::ffi::c_int) as size_t {
            (*state).linebuf = qf_grow_linebuf(state, len);
        } else {
            (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
            (*state).linelen = len;
        }
        xstrlcpy(
            (*state).linebuf,
            p_buf,
            (*state).linelen.wrapping_add(1 as size_t),
        );
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_get_next_file_line(
    mut state: *mut qfstate_T,
) -> ::core::ffi::c_int {
    unsafe {
        loop {
            *__errno_location() = 0 as ::core::ffi::c_int;
            if fgets(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE,
                (*state).fd,
            )
            .is_null()
            {
                if *__errno_location() == EINTR {
                    continue;
                }
                return QF_END_OF_INPUT as ::core::ffi::c_int;
            } else {
                let mut discard: bool = false_0 != 0;
                (*state).linelen = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
                if (*state).linelen == (IOSIZE - 1 as ::core::ffi::c_int) as size_t
                    && !((*IObuff.ptr())[(*state).linelen.wrapping_sub(1 as size_t) as usize]
                        as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int)
                {
                    if (*state).growbuf.is_null() {
                        (*state).growbufsiz = (2 as ::core::ffi::c_int
                            * (IOSIZE - 1 as ::core::ffi::c_int))
                            as size_t;
                        (*state).growbuf = xmalloc((*state).growbufsiz) as *mut ::core::ffi::c_char;
                    }
                    memcpy(
                        (*state).growbuf as *mut ::core::ffi::c_void,
                        IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        (IOSIZE - 1 as ::core::ffi::c_int) as size_t,
                    );
                    let mut growbuflen: size_t = (*state).linelen;
                    *(*state).growbuf.offset(growbuflen as isize) = NUL as ::core::ffi::c_char;
                    loop {
                        *__errno_location() = 0 as ::core::ffi::c_int;
                        if fgets(
                            (*state).growbuf.offset(growbuflen as isize),
                            (*state).growbufsiz.wrapping_sub(growbuflen) as ::core::ffi::c_int,
                            (*state).fd,
                        )
                        .is_null()
                        {
                            if *__errno_location() != EINTR {
                                break;
                            }
                        } else {
                            (*state).linelen = strlen((*state).growbuf.offset(growbuflen as isize));
                            growbuflen = growbuflen.wrapping_add((*state).linelen);
                            if *(*state)
                                .growbuf
                                .offset(growbuflen.wrapping_sub(1 as size_t) as isize)
                                as ::core::ffi::c_int
                                == '\n' as ::core::ffi::c_int
                            {
                                break;
                            }
                            if (*state).growbufsiz == LINE_MAXLEN.get() {
                                discard = true_0 != 0;
                                break;
                            } else {
                                (*state).growbufsiz = if (2 as size_t)
                                    .wrapping_mul((*state).growbufsiz)
                                    < LINE_MAXLEN.get()
                                {
                                    (2 as size_t).wrapping_mul((*state).growbufsiz)
                                } else {
                                    LINE_MAXLEN.get()
                                };
                                (*state).growbuf = xrealloc(
                                    (*state).growbuf as *mut ::core::ffi::c_void,
                                    (*state).growbufsiz,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                        }
                    }
                    // Upstream's `if (discard) while (true)` merged by the transpile; exits via break.
                    #[allow(clippy::while_immutable_condition)]
                    while discard {
                        *__errno_location() = 0 as ::core::ffi::c_int;
                        if fgets(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE,
                            (*state).fd,
                        )
                        .is_null()
                        {
                            if *__errno_location() != EINTR {
                                break;
                            }
                        } else if strlen(IObuff.ptr() as *mut ::core::ffi::c_char)
                            < (IOSIZE - 1 as ::core::ffi::c_int) as size_t
                            || (*IObuff.ptr())[(IOSIZE - 2 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_int
                                == '\n' as ::core::ffi::c_int
                        {
                            break;
                        }
                    }
                    (*state).linebuf = (*state).growbuf;
                    (*state).linelen = growbuflen;
                } else {
                    (*state).linebuf = IObuff.ptr() as *mut ::core::ffi::c_char;
                }
                if (*state).vc.vc_type != CONV_NONE as ::core::ffi::c_int
                    && has_non_ascii((*state).linebuf) as ::core::ffi::c_int != 0
                {
                    let mut line: *mut ::core::ffi::c_char = string_convert(
                        &raw mut (*state).vc,
                        (*state).linebuf,
                        &raw mut (*state).linelen,
                    );
                    if !line.is_null() {
                        if (*state).linelen < IOSIZE as size_t {
                            xstrlcpy(
                                (*state).linebuf,
                                line,
                                (*state).linelen.wrapping_add(1 as size_t),
                            );
                            xfree(line as *mut ::core::ffi::c_void);
                        } else {
                            xfree((*state).growbuf as *mut ::core::ffi::c_void);
                            (*state).linebuf = line;
                            (*state).growbuf = line;
                            (*state).growbufsiz = if (*state).linelen < LINE_MAXLEN.get() {
                                (*state).linelen
                            } else {
                                LINE_MAXLEN.get()
                            };
                        }
                    }
                }
                return QF_OK as ::core::ffi::c_int;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_get_nextline(mut state: *mut qfstate_T) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = QF_FAIL as ::core::ffi::c_int;
        if (*state).fd.is_null() {
            if !(*state).tv.is_null() {
                if (*(*state).tv).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    status = qf_get_next_str_line(state);
                } else if (*(*state).tv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    status = qf_get_next_list_line(state);
                }
            } else {
                status = qf_get_next_buf_line(state);
            }
        } else {
            status = qf_get_next_file_line(state);
        }
        if status != QF_OK as ::core::ffi::c_int {
            return status;
        }
        if (*state).linelen > 0 as size_t
            && *(*state)
                .linebuf
                .offset((*state).linelen.wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int
                == '\n' as ::core::ffi::c_int
        {
            *(*state)
                .linebuf
                .offset((*state).linelen.wrapping_sub(1 as size_t) as isize) =
                NUL as ::core::ffi::c_char;
        }
        remove_bom((*state).linebuf);
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_setup_state(
    mut pstate: *mut qfstate_T,
    mut enc: *mut ::core::ffi::c_char,
    mut efile: *const ::core::ffi::c_char,
    mut tv: *mut typval_T,
    mut buf: *mut buf_T,
    mut lnumfirst: linenr_T,
    mut lnumlast: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*pstate).vc.vc_type = CONV_NONE as ::core::ffi::c_int;
        if !enc.is_null() && *enc as ::core::ffi::c_int != NUL {
            convert_setup(&raw mut (*pstate).vc, enc, p_enc.get());
        }
        if !efile.is_null() && {
            (*pstate).fd = if strequal(efile, b"-\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            {
                fdopen(
                    os_open_stdin_fd(),
                    b"r\0".as_ptr() as *const ::core::ffi::c_char,
                )
            } else {
                os_fopen(efile, b"r\0".as_ptr() as *const ::core::ffi::c_char)
            };
            (*pstate).fd.is_null()
        } {
            semsg(
                gettext(&raw const e_openerrf as *const ::core::ffi::c_char),
                efile,
            );
            return FAIL;
        }
        if !tv.is_null() {
            if (*tv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*pstate).p_str = (*tv).vval.v_string;
            } else if (*tv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*pstate).p_li = tv_list_first((*tv).vval.v_list);
            }
            (*pstate).tv = tv;
        }
        (*pstate).buf = buf;
        (*pstate).buflnum = lnumfirst;
        (*pstate).lnumlast = lnumlast;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_cleanup_state(mut pstate: *mut qfstate_T) {
    unsafe {
        if !(*pstate).fd.is_null() {
            fclose((*pstate).fd);
        }
        xfree((*pstate).growbuf as *mut ::core::ffi::c_void);
        if (*pstate).vc.vc_type != CONV_NONE as ::core::ffi::c_int {
            convert_setup(
                &raw mut (*pstate).vc,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
        }
    }
}

pub(crate) unsafe extern "C" fn qf_init_ext(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut efile: *const ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut tv: *mut typval_T,
    mut errorformat: *mut ::core::ffi::c_char,
    mut newlist: bool,
    mut lnumfirst: linenr_T,
    mut lnumlast: linenr_T,
    mut qf_title: *const ::core::ffi::c_char,
    mut enc: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qfl: *mut qf_list_T = ::core::ptr::null_mut::<qf_list_T>();
        let mut adding: bool = false;
        let mut efm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut state: qfstate_T = qfstate_T {
            linebuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            linelen: 0,
            growbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            growbufsiz: 0,
            fd: ::core::ptr::null_mut::<FILE>(),
            tv: ::core::ptr::null_mut::<typval_T>(),
            p_str: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            p_list: ::core::ptr::null_mut::<list_T>(),
            p_li: ::core::ptr::null_mut::<listitem_T>(),
            buf: ::core::ptr::null_mut::<buf_T>(),
            buflnum: 0,
            lnumlast: 0,
            vc: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false,
            },
        };
        let mut fields = Fields::new();
        let mut old_last: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            qf_last_bufname.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        // The compiled 'errorformat' is taken out of the cache for the
        // length of this read and put back at the end. Adding an entry can
        // fire `BufNew`, an autocommand can run another `:cexpr`, and
        // upstream — which keeps the compiled option in a bare static and
        // frees it whenever the option text changes — would then free what
        // this loop is still walking. Owning it here costs the re-entrant
        // call a recompile and nothing otherwise.
        let mut compiled = (*EFM_CACHE.ptr()).take();
        '_qf_init_end: {
            if qf_setup_state(&raw mut state, enc, efile, tv, buf, lnumfirst, lnumlast) != FAIL {
                qfl = ::core::ptr::null_mut::<qf_list_T>();
                adding = false_0 != 0;
                if newlist as ::core::ffi::c_int != 0 || qf_idx == (*qi).qf_listcount {
                    qf_new_list(qi, qf_title);
                    qf_idx = (*qi).qf_curlist;
                    qfl = qf_get_list(qi, qf_idx);
                } else {
                    adding = true_0 != 0;
                    qfl = qf_get_list(qi, qf_idx);
                    if !qf_list_empty(qfl) {
                        old_last = (*qfl).qf_last;
                    }
                }
                efm = if errorformat == p_efm.get()
                    && tv.is_null()
                    && !buf.is_null()
                    && *(*buf).b_p_efm as ::core::ffi::c_int != NUL
                {
                    (*buf).b_p_efm
                } else {
                    errorformat
                };
                // Reuse the previously compiled option when the text has
                // not changed; otherwise throw it away and compile again.
                let text = CStr::from_ptr(efm).to_bytes();
                if !compiled.as_ref().is_some_and(|(had, _)| had == text) {
                    compiled = Efm::compile(efm).map(|parsed| (text.to_vec(), parsed));
                }
                '_error2: {
                    if let Some((_, parsed)) = compiled.as_mut() {
                        got_int.set(false_0 != 0);
                        while !got_int.get() {
                            let status =
                                qf_init_process_nextline(qfl, parsed, &raw mut state, &mut fields);
                            if status == Status::EndOfInput {
                                break;
                            }
                            if status == Status::Fail {
                                break '_error2;
                            }
                            line_breakcheck();
                        }
                        if state.fd.is_null() || ferror(state.fd) == 0 {
                            if (*qfl).qf_index == 0 as ::core::ffi::c_int {
                                (*qfl).qf_ptr = (*qfl).qf_start;
                                (*qfl).qf_index = 1 as ::core::ffi::c_int;
                                (*qfl).qf_nonevalid = true_0 != 0;
                            } else {
                                (*qfl).qf_nonevalid = false_0 != 0;
                                if (*qfl).qf_ptr.is_null() {
                                    (*qfl).qf_ptr = (*qfl).qf_start;
                                }
                            }
                            retval = (*qfl).qf_count;
                            break '_qf_init_end;
                        } else {
                            emsg(gettext(&raw const e_readerrf as *const ::core::ffi::c_char));
                        }
                    }
                }
                if !adding {
                    qf_free(qfl);
                    (*qi).qf_listcount -= 1;
                    if (*qi).qf_curlist > 0 as ::core::ffi::c_int {
                        (*qi).qf_curlist -= 1;
                    }
                }
            }
        }
        if qf_idx == (*qi).qf_curlist {
            qf_update_buffer(qi, old_last);
        }
        qf_cleanup_state(&raw mut state);
        *EFM_CACHE.ptr() = compiled;
        return retval;
    }
}

/// The compiled `'errorformat'`, kept between calls together with the option
/// text it was compiled from, so a repeated command does not recompile it.
static EFM_CACHE: GlobalCell<Option<(Vec<u8>, Efm)>> = GlobalCell::new(None);

pub(crate) unsafe extern "C" fn qf_store_title(
    mut qfl: *mut qf_list_T,
    mut title: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*qfl).qf_title as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        if title.is_null() {
            return;
        }
        let mut len: size_t = strlen(title).wrapping_add(1 as size_t);
        let mut p: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
        (*qfl).qf_title = p;
        xstrlcpy(p, title, len.wrapping_add(1 as size_t));
    }
}

pub(crate) unsafe extern "C" fn qf_cmdtitle(
    mut cmd: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static qftitle_str: GlobalCell<[::core::ffi::c_char; 1025]> = GlobalCell::new([0; 1025]);
        snprintf(
            qftitle_str.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b":%s\0".as_ptr() as *const ::core::ffi::c_char,
            cmd,
        );
        return qftitle_str.ptr() as *mut ::core::ffi::c_char;
    }
}
