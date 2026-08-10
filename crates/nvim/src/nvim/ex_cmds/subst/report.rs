//! What the user sees after a `:s` -- the summary line and the live preview.
//!
//! `do_sub_msg` is the "N substitutions on N lines" report, with the
//! 'report' option and `:s#` numbering deciding whether it prints at all;
//! `show_sub` is `'inccommand'`, which runs the substitution into a preview
//! buffer and highlights the matches as the command line is typed.
//! `ex_substitute` and `ex_substitute_preview` are the two Ex entry points.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_sub_msg(mut count_only: bool) -> bool {
    unsafe {
        if (sub_nsubs.get() as OptInt > p_report.get()
            && (KeyTyped.get() as ::core::ffi::c_int != 0
                || sub_nlines.get() > 1 as linenr_T
                || p_report.get() < 1 as OptInt)
            || count_only as ::core::ffi::c_int != 0)
            && messaging() as ::core::ffi::c_int != 0
        {
            if got_int.get() {
                strcpy(
                    msg_buf.ptr() as *mut ::core::ffi::c_char,
                    gettext(c"(Interrupted) ".as_ptr()),
                );
            } else {
                *(msg_buf.ptr() as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
            }
            let mut msg_single: *mut ::core::ffi::c_char = if count_only as ::core::ffi::c_int != 0
            {
                ngettext(
                    c"%ld match on %ld line".as_ptr(),
                    c"%ld matches on %ld line".as_ptr(),
                    sub_nsubs.get() as ::core::ffi::c_ulong,
                )
            } else {
                ngettext(
                    c"%ld substitution on %ld line".as_ptr(),
                    c"%ld substitutions on %ld line".as_ptr(),
                    sub_nsubs.get() as ::core::ffi::c_ulong,
                )
            };
            let mut msg_plural: *mut ::core::ffi::c_char = if count_only as ::core::ffi::c_int != 0
            {
                ngettext(
                    c"%ld match on %ld lines".as_ptr(),
                    c"%ld matches on %ld lines".as_ptr(),
                    sub_nsubs.get() as ::core::ffi::c_ulong,
                )
            } else {
                ngettext(
                    c"%ld substitution on %ld lines".as_ptr(),
                    c"%ld substitutions on %ld lines".as_ptr(),
                    sub_nsubs.get() as ::core::ffi::c_ulong,
                )
            };
            vim_snprintf_add(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
                ngettext(
                    msg_single,
                    msg_plural,
                    sub_nlines.get() as ::core::ffi::c_ulong,
                ),
                sub_nsubs.get() as int64_t,
                sub_nlines.get() as int64_t,
            );
            if msg(
                msg_buf.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            ) {
                set_keep_msg(
                    msg_buf.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
            return true_0 != 0;
        }
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn show_sub(
    mut eap: *mut exarg_T,
    mut old_cusr: pos_T,
    mut preview_lines: *mut PreviewLines,
    mut hl_id: ::core::ffi::c_int,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut save_shm_p: *mut ::core::ffi::c_char = xstrdup(p_shm.get());
        let mut lines: PreviewLines = *preview_lines;
        let mut orig_buf: *mut buf_T = curbuf.get();
        let mut cmdpreview_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        set_option_direct(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: c"F".as_ptr() as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        let mut i: size_t = 0 as size_t;
        while i < lines.subresults.size {
            let mut curres: SubResult = *lines.subresults.items.add(i);
            if curres.start.lnum >= old_cusr.lnum {
                (*curwin.get()).w_cursor.lnum = curres.start.lnum;
                (*curwin.get()).w_cursor.col = curres.start.col;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        update_topline(curwin.get());
        let mut col_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut preview: bool = *p_icm.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && ((*eap).line1 != old_cusr.lnum || (*eap).line2 != old_cusr.lnum);
        if preview {
            cmdpreview_buf = buflist_findnr(cmdpreview_bufnr as ::core::ffi::c_int);
            debug_assert!(!cmdpreview_buf.is_null(), "cmdpreview_buf != NULL");
            if lines.subresults.size > 0 as size_t {
                let mut last_match: SubResult = *lines.subresults.items.add(
                    lines
                        .subresults
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t),
                );
                let mut highest_lnum: linenr_T = if last_match.start.lnum > last_match.end.lnum {
                    last_match.start.lnum
                } else {
                    last_match.end.lnum
                };
                debug_assert!(highest_lnum > 0 as linenr_T, "highest_lnum > 0");
                col_width = log10(highest_lnum as ::core::ffi::c_double) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 3 as ::core::ffi::c_int;
            }
        }
        let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut old_line_size: colnr_T = 0 as colnr_T;
        let mut line_size: colnr_T = 0 as colnr_T;
        let mut linenr_preview: linenr_T = 0 as linenr_T;
        let mut linenr_origbuf: linenr_T = 0 as linenr_T;
        let mut next_linenr: linenr_T = 0 as linenr_T;
        let mut matchidx: size_t = 0 as size_t;
        while matchidx < lines.subresults.size {
            let mut match_0: SubResult = *lines.subresults.items.add(matchidx);
            if !cmdpreview_buf.is_null() {
                let mut p_start: lpos_T = lpos_T {
                    lnum: 0 as linenr_T,
                    col: match_0.start.col,
                };
                let mut p_end: lpos_T = lpos_T {
                    lnum: 0 as linenr_T,
                    col: match_0.end.col,
                };
                buf_ensure_loaded(cmdpreview_buf);
                if match_0.pre_match == 0 as linenr_T {
                    next_linenr = match_0.start.lnum;
                } else {
                    next_linenr = match_0.pre_match;
                }
                if next_linenr == linenr_origbuf {
                    next_linenr += 1;
                    p_start.lnum = linenr_preview;
                    p_end.lnum = linenr_preview;
                }
                while next_linenr <= match_0.end.lnum {
                    if next_linenr == match_0.start.lnum {
                        p_start.lnum = linenr_preview + 1 as linenr_T;
                    }
                    if next_linenr == match_0.end.lnum {
                        p_end.lnum = linenr_preview + 1 as linenr_T;
                    }
                    let mut line: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if next_linenr == (*orig_buf).b_ml.ml_line_count + 1 as linenr_T {
                        line = c"".as_ptr() as *mut ::core::ffi::c_char;
                    } else {
                        line = ml_get_buf(orig_buf, next_linenr);
                        line_size = (ml_get_buf_len(orig_buf, next_linenr)
                            + col_width
                            + 1 as ::core::ffi::c_int)
                            as colnr_T;
                        if line_size > old_line_size {
                            str = xrealloc(
                                str as *mut ::core::ffi::c_void,
                                (line_size as size_t)
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            ) as *mut ::core::ffi::c_char;
                            old_line_size = line_size;
                        }
                    }
                    snprintf(
                        str,
                        line_size as size_t,
                        c"|%*d| %s".as_ptr(),
                        col_width - 3 as ::core::ffi::c_int,
                        next_linenr,
                        line,
                    );
                    if linenr_preview == 0 as linenr_T {
                        ml_replace_buf(
                            cmdpreview_buf,
                            1 as linenr_T,
                            str,
                            true_0 != 0,
                            false_0 != 0,
                        );
                    } else {
                        ml_append_buf(cmdpreview_buf, linenr_preview, str, line_size, false_0 != 0);
                    }
                    linenr_preview = (linenr_preview as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as linenr_T;
                    next_linenr += 1;
                }
                linenr_origbuf = match_0.end.lnum;
                bufhl_add_hl_pos_offset(
                    cmdpreview_buf,
                    cmdpreview_ns,
                    hl_id,
                    p_start,
                    p_end,
                    col_width as colnr_T,
                );
            }
            bufhl_add_hl_pos_offset(
                orig_buf,
                cmdpreview_ns,
                hl_id,
                match_0.start,
                match_0.end,
                0 as colnr_T,
            );
            matchidx = matchidx.wrapping_add(1);
        }
        xfree(str as *mut ::core::ffi::c_void);
        set_option_direct(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(save_shm_p),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        xfree(save_shm_p as *mut ::core::ffi::c_void);
        return if preview as ::core::ffi::c_int != 0 {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

pub unsafe fn ex_substitute(mut eap: *mut exarg_T) {
    unsafe {
        do_sub(eap, profile_zero(), 0 as ::core::ffi::c_int, 0 as handle_T);
    }
}

pub unsafe fn ex_substitute_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    unsafe {
        if *(*eap).arg as ::core::ffi::c_int != 0
            && !(*(*eap).arg as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *(*eap).arg as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *(*eap).arg as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *(*eap).arg as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        {
            let mut save_eap: *mut ::core::ffi::c_char = (*eap).arg;
            let mut retv: ::core::ffi::c_int = do_sub(
                eap,
                profile_setlimit(p_rdt.get() as int64_t),
                cmdpreview_ns,
                cmdpreview_bufnr,
            );
            (*eap).arg = save_eap;
            return retv;
        }
        return 0 as ::core::ffi::c_int;
    }
}
