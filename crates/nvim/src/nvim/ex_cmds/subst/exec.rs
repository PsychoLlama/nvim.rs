//! `do_sub` -- the substitute engine.
//!
//! One transpiled function of 1,220 lines, and the batch's largest single item:
//! it parses what `parse.rs` left it, loops over the range calling the regex
//! engine, builds each replacement (including `\=` expressions, which re-enter
//! the evaluator, and `\n` which joins), runs the `c` flag's confirmation
//! dialog, applies the change through the undo and extmark layers, and collects
//! what `report.rs` prints.  It is over the file-size cap and stays that way
//! until it is decomposed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn do_sub(
    mut eap: *mut exarg_T,
    timeout: proftime_T,
    cmdpreview_ns: ::core::ffi::c_int,
    cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        static subflags: GlobalCell<subflags_T> = GlobalCell::new(subflags_T {
            do_all: false_0 != 0,
            do_ask: false_0 != 0,
            do_count: false_0 != 0,
            do_error: true_0 != 0,
            do_print: false_0 != 0,
            do_list: false_0 != 0,
            do_number: false_0 != 0,
            do_ic: kSubHonorOptions,
        });
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut sub: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut patlen: size_t = 0 as size_t;
        let mut delimiter: ::core::ffi::c_int = 0;
        let mut has_second_delim: bool = false_0 != 0;
        let mut sublen: ::core::ffi::c_int = 0;
        let mut got_quit: bool = false_0 != 0;
        let mut got_match: bool = false_0 != 0;
        let mut which_pat: ::core::ffi::c_int = 0;
        let mut cmd: *mut ::core::ffi::c_char = (*eap).arg;
        let mut first_line: linenr_T = 0 as linenr_T;
        let mut last_line: linenr_T = 0 as linenr_T;
        let mut old_line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
        let mut sub_firstline: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut endcolumn: bool = false_0 != 0;
        let keeppatterns: bool =
            (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0;
        let mut preview_lines: PreviewLines = PreviewLines {
            subresults: C2Rust_Unnamed_33 {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<SubResult>(),
            },
            lines_needed: 0 as linenr_T,
        };
        static pre_hl_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        let mut start_nsubs: ::core::ffi::c_int = 0;
        let mut did_save: bool = false_0 != 0;
        if global_busy.get() == 0 {
            sub_nsubs.set(0 as ::core::ffi::c_int);
            sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
        }
        start_nsubs = sub_nsubs.get();
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_tilde as ::core::ffi::c_int {
            which_pat = RE_LAST as ::core::ffi::c_int;
        } else {
            which_pat = RE_SUBST as ::core::ffi::c_int;
        }
        if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 's' as ::core::ffi::c_int
            && *cmd as ::core::ffi::c_int != NUL
            && !ascii_iswhite(*cmd as ::core::ffi::c_int)
            && vim_strchr(
                c"0123456789cegriIp|\"".as_ptr(),
                *cmd as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
                return 0 as ::core::ffi::c_int;
            }
            if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                cmd = cmd.offset(1);
                if vim_strchr(c"/?&".as_ptr(), *cmd as uint8_t as ::core::ffi::c_int).is_null() {
                    emsg(gettext(
                        &raw const e_backslash as *const ::core::ffi::c_char,
                    ));
                    return 0 as ::core::ffi::c_int;
                }
                if *cmd as ::core::ffi::c_int != '&' as ::core::ffi::c_int {
                    which_pat = RE_SEARCH as ::core::ffi::c_int;
                }
                pat = c"".as_ptr() as *mut ::core::ffi::c_char;
                patlen = 0 as size_t;
                let c2rust_fresh6 = cmd;
                cmd = cmd.offset(1);
                delimiter = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
                has_second_delim = true_0 != 0;
            } else {
                which_pat = RE_LAST as ::core::ffi::c_int;
                let c2rust_fresh7 = cmd;
                cmd = cmd.offset(1);
                delimiter = *c2rust_fresh7 as uint8_t as ::core::ffi::c_int;
                pat = cmd;
                cmd = skip_regexp_ex(
                    cmd,
                    delimiter,
                    magic_isset() as ::core::ffi::c_int,
                    &raw mut (*eap).arg,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<magic_T>(),
                );
                if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delimiter
                {
                    let c2rust_fresh8 = cmd;
                    cmd = cmd.offset(1);
                    *c2rust_fresh8 = NUL as ::core::ffi::c_char;
                    has_second_delim = true_0 != 0;
                }
                patlen = strlen(pat);
            }
            let mut p: *mut ::core::ffi::c_char = cmd;
            cmd = skip_substitute(cmd, delimiter);
            sub = xstrdup(p);
            if (*eap).skip == 0 && !keeppatterns && cmdpreview_ns <= 0 as ::core::ffi::c_int {
                sub_set_replacement(SubReplacementString {
                    sub: xstrdup(sub),
                    timestamp: os_time(),
                    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
                });
            }
        } else if (*eap).skip == 0 {
            if (*old_sub.ptr()).sub.is_null() {
                emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
                return 0 as ::core::ffi::c_int;
            }
            pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
            patlen = 0 as size_t;
            sub = xstrdup((*old_sub.ptr()).sub);
            endcolumn = (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int;
        }
        if !sub.is_null()
            && sub_joining_lines(
                eap,
                pat,
                patlen,
                sub,
                cmd,
                cmdpreview_ns <= 0 as ::core::ffi::c_int,
                keeppatterns,
            ) as ::core::ffi::c_int
                != 0
        {
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        cmd = sub_parse_flags(cmd, subflags.ptr(), &raw mut which_pat);
        let mut save_do_all: bool = (*subflags.ptr()).do_all;
        let mut save_do_ask: bool = (*subflags.ptr()).do_ask;
        cmd = skipwhite(cmd);
        if ascii_isdigit(*cmd as ::core::ffi::c_int) {
            let count_arg: *const ::core::ffi::c_char = cmd;
            i = getdigits_int(&raw mut cmd, false_0 != 0, INT_MAX);
            if i <= 0 as ::core::ffi::c_int
                && (*eap).skip == 0
                && (*subflags.ptr()).do_error as ::core::ffi::c_int != 0
            {
                emsg(gettext(
                    &raw const e_zerocount as *const ::core::ffi::c_char,
                ));
                xfree(sub as *mut ::core::ffi::c_void);
                return 0 as ::core::ffi::c_int;
            } else if i == INT_MAX {
                semsg_c!(
                    gettext(&raw const e_val_too_large_len as *const ::core::ffi::c_char),
                    cmd.offset_from(count_arg) as ::core::ffi::c_int,
                    count_arg,
                );
                xfree(sub as *mut ::core::ffi::c_void);
                return 0 as ::core::ffi::c_int;
            }
            (*eap).line1 = (*eap).line2;
            (*eap).line2 = ((*eap).line2 as ::core::ffi::c_int
                + (i as linenr_T - 1 as linenr_T) as ::core::ffi::c_int)
                as linenr_T;
            (*eap).line2 = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
                (*eap).line2
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
        }
        cmd = skipwhite(cmd);
        if *cmd as ::core::ffi::c_int != 0
            && *cmd as ::core::ffi::c_int != '"' as ::core::ffi::c_int
        {
            (*eap).nextcmd = check_nextcmd(cmd);
            if (*eap).nextcmd.is_null() {
                semsg_c!(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    cmd,
                );
                xfree(sub as *mut ::core::ffi::c_void);
                return 0 as ::core::ffi::c_int;
            }
        }
        if (*eap).skip != 0 {
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        if !(*subflags.ptr()).do_count && (*curbuf.get()).b_p_ma == 0 {
            emsg(gettext(
                &raw const e_modifiable as *const ::core::ffi::c_char,
            ));
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        if search_regcomp(
            pat,
            patlen,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            RE_SUBST as ::core::ffi::c_int,
            which_pat,
            if cmdpreview_ns > 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                SEARCH_HIS as ::core::ffi::c_int
            },
            &raw mut regmatch,
        ) == FAIL
        {
            if (*subflags.ptr()).do_error {
                emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
            }
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        if (*subflags.ptr()).do_ic as ::core::ffi::c_uint
            == kSubIgnoreCase as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            regmatch.rmm_ic = true_0;
        } else if (*subflags.ptr()).do_ic as ::core::ffi::c_uint
            == kSubMatchCase as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            regmatch.rmm_ic = false_0;
        }
        sub_firstline = ::core::ptr::null_mut::<::core::ffi::c_char>();
        debug_assert!(!sub.is_null(), "sub != NULL");
        if *sub.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *sub.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            let mut p_0: *mut ::core::ffi::c_char = xstrdup(sub);
            xfree(sub as *mut ::core::ffi::c_void);
            sub = p_0;
        } else {
            let mut p_1: *mut ::core::ffi::c_char = regtilde(
                sub,
                magic_isset() as ::core::ffi::c_int,
                cmdpreview_ns > 0 as ::core::ffi::c_int,
            );
            if p_1 != sub {
                xfree(sub as *mut ::core::ffi::c_void);
                sub = p_1;
            }
        }
        let mut line2: linenr_T = (*eap).line2;
        let mut lnum: linenr_T = (*eap).line1;
        while lnum <= line2
            && !got_quit
            && !aborting()
            && (cmdpreview_ns <= 0 as ::core::ffi::c_int
                || preview_lines.lines_needed <= p_cwh.get() as linenr_T
                || lnum <= (*curwin.get()).w_botline)
        {
            let mut nmatch: ::core::ffi::c_int = vim_regexec_multi(
                &raw mut regmatch,
                curwin.get(),
                curbuf.get(),
                lnum,
                0 as colnr_T,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if nmatch != 0 {
                let mut copycol: colnr_T = 0;
                let mut matchcol: colnr_T = 0;
                let mut prev_matchcol: colnr_T = MAXCOL as ::core::ffi::c_int;
                let mut new_end: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut new_start: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut new_start_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut p1: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut did_sub: bool = false_0 != 0;
                let mut lastone: ::core::ffi::c_int = 0;
                let mut nmatch_tl: linenr_T = 0 as linenr_T;
                let mut do_again: ::core::ffi::c_int = 0;
                let mut skip_match: bool = false_0 != 0;
                let mut sub_firstlnum: linenr_T = 0;
                let mut lnum_start: linenr_T = 0 as linenr_T;
                let mut line_matches: C2Rust_Unnamed_34 = C2Rust_Unnamed_34 {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<LineData>(),
                };
                sub_firstlnum = lnum;
                copycol = 0 as ::core::ffi::c_int as colnr_T;
                matchcol = 0 as ::core::ffi::c_int as colnr_T;
                if !got_match {
                    setpcmark();
                    got_match = true_0 != 0;
                }
                loop {
                    let mut current_match: SubResult = SubResult {
                        start: lpos_T {
                            lnum: 0 as linenr_T,
                            col: 0 as colnr_T,
                        },
                        end: lpos_T {
                            lnum: 0 as linenr_T,
                            col: 0 as colnr_T,
                        },
                        pre_match: 0 as linenr_T,
                    };
                    if regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T {
                        current_match.pre_match = lnum;
                        lnum += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                        sub_firstlnum += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                        nmatch -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                            as ::core::ffi::c_int;
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut sub_firstline as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        let _ = *ptr_;
                    }
                    current_match.start.lnum = sub_firstlnum;
                    if lnum > (*curbuf.get()).b_ml.ml_line_count {
                        break;
                    }
                    if sub_firstline.is_null() {
                        sub_firstline =
                            xstrnsave(ml_get(sub_firstlnum), ml_get_len(sub_firstlnum) as size_t);
                    }
                    (*curwin.get()).w_cursor.lnum = lnum;
                    do_again = false_0;
                    '_skip: {
                        if matchcol == prev_matchcol
                            && regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                                == 0 as linenr_T
                            && matchcol == regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                        {
                            if *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int == NUL
                            {
                                skip_match = true_0 != 0;
                            } else {
                                matchcol += utfc_ptr2len(sub_firstline.offset(matchcol as isize));
                            }
                            current_match.start.col = matchcol;
                            current_match.end.lnum = sub_firstlnum;
                            current_match.end.col = matchcol;
                        } else {
                            matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                            prev_matchcol = matchcol;
                            if (*subflags.ptr()).do_count {
                                if nmatch > 1 as ::core::ffi::c_int {
                                    matchcol = strlen(sub_firstline) as colnr_T;
                                    nmatch = 1 as ::core::ffi::c_int;
                                    skip_match = true_0 != 0;
                                }
                                (*sub_nsubs.ptr()) += 1;
                                did_sub = true_0 != 0;
                                if !(*sub.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                                    && *sub.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '=' as ::core::ffi::c_int)
                                {
                                    break '_skip;
                                }
                            }
                            if (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0
                                && cmdpreview_ns <= 0 as ::core::ffi::c_int
                            {
                                let mut typed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                let mut save_State: ::core::ffi::c_int = State.get();
                                (*curwin.get()).w_cursor.col =
                                    regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                                if (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
                                    do_check_cursorbind();
                                }
                                if !vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
                                    (*no_u_sync.ptr()) += 1;
                                }
                                while (*subflags.ptr()).do_ask {
                                    if exmode_active.get() {
                                        print_line_no_prefix(
                                            lnum,
                                            (*subflags.ptr()).do_number,
                                            (*subflags.ptr()).do_list,
                                        );
                                        let mut sc: colnr_T = 0;
                                        let mut ec: colnr_T = 0;
                                        getvcol(
                                            curwin.get(),
                                            &raw mut (*curwin.get()).w_cursor,
                                            &raw mut sc,
                                            ::core::ptr::null_mut::<colnr_T>(),
                                            ::core::ptr::null_mut::<colnr_T>(),
                                        );
                                        (*curwin.get()).w_cursor.col = (if regmatch.endpos
                                            [0 as ::core::ffi::c_int as usize]
                                            .col
                                            as ::core::ffi::c_int
                                            - 1 as ::core::ffi::c_int
                                            > 0 as ::core::ffi::c_int
                                        {
                                            regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                                as ::core::ffi::c_int
                                                - 1 as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        })
                                            as colnr_T;
                                        getvcol(
                                            curwin.get(),
                                            &raw mut (*curwin.get()).w_cursor,
                                            ::core::ptr::null_mut::<colnr_T>(),
                                            ::core::ptr::null_mut::<colnr_T>(),
                                            &raw mut ec,
                                        );
                                        (*curwin.get()).w_cursor.col =
                                            regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                                        if (*subflags.ptr()).do_number as ::core::ffi::c_int != 0
                                            || (*curwin.get()).w_onebuf_opt.wo_nu != 0
                                        {
                                            let mut numw: ::core::ffi::c_int =
                                                number_width(curwin.get())
                                                    + 1 as ::core::ffi::c_int;
                                            sc += numw;
                                            ec += numw;
                                        }
                                        let mut prompt: *mut ::core::ffi::c_char =
                                            xmallocz((ec as size_t).wrapping_add(1 as size_t))
                                                as *mut ::core::ffi::c_char;
                                        memset(
                                            prompt as *mut ::core::ffi::c_void,
                                            ' ' as ::core::ffi::c_int,
                                            sc as size_t,
                                        );
                                        memset(
                                            prompt.offset(sc as isize) as *mut ::core::ffi::c_void,
                                            '^' as ::core::ffi::c_int,
                                            ((ec - sc) as size_t).wrapping_add(1 as size_t),
                                        );
                                        let mut resp: *mut ::core::ffi::c_char = getcmdline_prompt(
                                            -1 as ::core::ffi::c_int,
                                            prompt,
                                            0 as ::core::ffi::c_int,
                                            EXPAND_NOTHING as ::core::ffi::c_int,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                            Callback {
                                                data: C2Rust_Unnamed_5 {
                                                    funcref: ::core::ptr::null_mut::<
                                                        ::core::ffi::c_char,
                                                    >(
                                                    ),
                                                },
                                                type_0: kCallbackNone,
                                            },
                                            false_0 != 0,
                                            ::core::ptr::null_mut::<bool>(),
                                        );
                                        if !ui_has(kUIMessages) {
                                            msg_putchar('\n' as ::core::ffi::c_int);
                                        }
                                        xfree(prompt as *mut ::core::ffi::c_void);
                                        if !resp.is_null() {
                                            typed = *resp as uint8_t as ::core::ffi::c_int;
                                            xfree(resp as *mut ::core::ffi::c_void);
                                        } else {
                                            typed = NUL;
                                        }
                                        if ex_normal_busy.get() != 0 && typed == NUL {
                                            typed = 'q' as ::core::ffi::c_int;
                                        }
                                    } else {
                                        let mut orig_line: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        let mut len_change: ::core::ffi::c_int =
                                            0 as ::core::ffi::c_int;
                                        let save_p_lz: bool = p_lz.get() != 0;
                                        let mut save_p_fen: ::core::ffi::c_int =
                                            (*curwin.get()).w_onebuf_opt.wo_fen;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
                                        let mut temp: ::core::ffi::c_int = RedrawingDisabled.get();
                                        RedrawingDisabled.set(0 as ::core::ffi::c_int);
                                        p_lz.set(false_0);
                                        if !new_start.is_null() {
                                            orig_line =
                                                xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
                                            let mut new_line: *mut ::core::ffi::c_char = concat_str(
                                                new_start,
                                                sub_firstline.offset(copycol as isize),
                                            );
                                            len_change = strlen(new_line) as ::core::ffi::c_int
                                                - strlen(orig_line) as ::core::ffi::c_int;
                                            (*curwin.get()).w_cursor.col += len_change;
                                            ml_replace(lnum, new_line, false_0 != 0);
                                        }
                                        search_match_lines.set(
                                            regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                                                - regmatch.startpos
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .lnum,
                                        );
                                        search_match_endcol.set(
                                            (regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                                as ::core::ffi::c_int
                                                + len_change)
                                                as colnr_T,
                                        );
                                        if search_match_lines.get() == 0 as linenr_T
                                            && search_match_endcol.get() == 0 as ::core::ffi::c_int
                                        {
                                            search_match_endcol
                                                .set(1 as ::core::ffi::c_int as colnr_T);
                                        }
                                        highlight_match.set(true_0 != 0);
                                        update_topline(curwin.get());
                                        validate_cursor(curwin.get());
                                        redraw_later(curwin.get(), UPD_SOME_VALID);
                                        show_cursor_info_later(true_0 != 0);
                                        update_screen();
                                        redraw_later(curwin.get(), UPD_SOME_VALID);
                                        (*curwin.get()).w_onebuf_opt.wo_fen = save_p_fen;
                                        let mut p_2: *mut ::core::ffi::c_char = gettext(
                                        c"replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)"
                                            .as_ptr(),
                                    );
                                        snprintf(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            IOSIZE as size_t,
                                            p_2,
                                            sub,
                                        );
                                        p_2 = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
                                        typed = prompt_for_input(
                                            p_2,
                                            HLF_R,
                                            true_0 != 0,
                                            ::core::ptr::null_mut::<bool>(),
                                        );
                                        highlight_match.set(false_0 != 0);
                                        xfree(p_2 as *mut ::core::ffi::c_void);
                                        msg_didout.set(false_0 != 0);
                                        gotocmdline(true_0 != 0);
                                        p_lz.set(save_p_lz as ::core::ffi::c_int);
                                        RedrawingDisabled.set(temp);
                                        if !orig_line.is_null() {
                                            ml_replace(lnum, orig_line, false_0 != 0);
                                        }
                                    }
                                    need_wait_return.set(false_0 != 0);
                                    if typed == 'q' as ::core::ffi::c_int
                                        || typed == ESC
                                        || typed == Ctrl_C
                                    {
                                        got_quit = true_0 != 0;
                                        break;
                                    } else {
                                        if typed == 'n' as ::core::ffi::c_int {
                                            break;
                                        }
                                        if typed == 'y' as ::core::ffi::c_int {
                                            break;
                                        }
                                        if typed == 'l' as ::core::ffi::c_int {
                                            (*subflags.ptr()).do_all = false_0 != 0;
                                            line2 = lnum;
                                            break;
                                        } else if typed == 'a' as ::core::ffi::c_int {
                                            (*subflags.ptr()).do_ask = false_0 != 0;
                                            break;
                                        } else if typed == Ctrl_E {
                                            scrollup_clamp();
                                        } else if typed == Ctrl_Y {
                                            scrolldown_clamp();
                                        }
                                    }
                                }
                                State.set(save_State);
                                setmouse();
                                if !vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
                                    (*no_u_sync.ptr()) -= 1;
                                }
                                if typed == 'n' as ::core::ffi::c_int {
                                    if nmatch > 1 as ::core::ffi::c_int {
                                        matchcol = strlen(sub_firstline) as colnr_T;
                                        skip_match = true_0 != 0;
                                    }
                                    break '_skip;
                                } else if got_quit {
                                    break '_skip;
                                }
                            }
                            (*curwin.get()).w_cursor.col =
                                regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                            if nmatch as linenr_T
                                > (*curbuf.get()).b_ml.ml_line_count - sub_firstlnum + 1 as linenr_T
                            {
                                nmatch = ((*curbuf.get()).b_ml.ml_line_count - sub_firstlnum
                                    + 1 as linenr_T)
                                    as ::core::ffi::c_int;
                                current_match.end.lnum = sub_firstlnum + nmatch as linenr_T;
                                skip_match = true_0 != 0;
                                if nmatch < 0 as ::core::ffi::c_int {
                                    break '_skip;
                                }
                            }
                            if cmdpreview_ns > 0 as ::core::ffi::c_int && !has_second_delim {
                                current_match.start.col =
                                    regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                                if current_match.end.lnum == 0 as linenr_T {
                                    current_match.end.lnum =
                                        sub_firstlnum + nmatch as linenr_T - 1 as linenr_T;
                                }
                                current_match.end.col =
                                    regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                if nmatch > 1 as ::core::ffi::c_int {
                                    sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                        + (nmatch as linenr_T - 1 as linenr_T)
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline = xstrnsave(
                                        ml_get(sub_firstlnum),
                                        ml_get_len(sub_firstlnum) as size_t,
                                    );
                                    if sub_firstlnum <= line2 {
                                        do_again = true_0;
                                    } else {
                                        (*subflags.ptr()).do_all = false_0 != 0;
                                    }
                                }
                                if skip_match {
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline = xstrdup(c"".as_ptr());
                                    copycol = 0 as ::core::ffi::c_int as colnr_T;
                                }
                                lnum = (lnum as ::core::ffi::c_int
                                    + (nmatch as linenr_T - 1 as linenr_T) as ::core::ffi::c_int)
                                    as linenr_T;
                            } else if cmdpreview_ns <= 0 as ::core::ffi::c_int
                                || has_second_delim as ::core::ffi::c_int != 0
                            {
                                lnum_start = lnum;
                                let mut save_ma: ::core::ffi::c_int = (*curbuf.get()).b_p_ma;
                                let mut save_sandbox: ::core::ffi::c_int = sandbox.get();
                                if (*subflags.ptr()).do_count {
                                    (*curbuf.get()).b_p_ma = false_0;
                                    (*sandbox.ptr()) += 1;
                                }
                                let mut subflags_save: subflags_T = subflags.get();
                                (*textlock.ptr()) += 1;
                                sublen = vim_regsub_multi(
                                    &raw mut regmatch,
                                    sub_firstlnum
                                        - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum,
                                    sub,
                                    sub_firstline,
                                    0 as ::core::ffi::c_int,
                                    REGSUB_BACKSLASH as ::core::ffi::c_int
                                        | (if magic_isset() as ::core::ffi::c_int != 0 {
                                            REGSUB_MAGIC as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }),
                                );
                                (*textlock.ptr()) -= 1;
                                subflags.set(subflags_save);
                                if sublen == 0 as ::core::ffi::c_int
                                    || aborting() as ::core::ffi::c_int != 0
                                    || (*subflags.ptr()).do_count as ::core::ffi::c_int != 0
                                {
                                    (*curbuf.get()).b_p_ma = save_ma;
                                    sandbox.set(save_sandbox);
                                } else {
                                    if nmatch == 1 as ::core::ffi::c_int {
                                        p1 = sub_firstline;
                                    } else {
                                        let mut lastlnum: linenr_T =
                                            sub_firstlnum + nmatch as linenr_T - 1 as linenr_T;
                                        p1 = ml_get(lastlnum);
                                        nmatch_tl = (nmatch_tl as ::core::ffi::c_int
                                            + (nmatch - 1 as ::core::ffi::c_int))
                                            as linenr_T;
                                    }
                                    let mut copy_len: ::core::ffi::c_int =
                                        regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int
                                            - copycol as ::core::ffi::c_int;
                                    new_end = sub_grow_buf(
                                        &raw mut new_start,
                                        &raw mut new_start_len,
                                        strlen(p1) as ::core::ffi::c_int
                                            - regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                                as ::core::ffi::c_int
                                            + copy_len
                                            + sublen
                                            + 1 as ::core::ffi::c_int,
                                    );
                                    memmove(
                                        new_end as *mut ::core::ffi::c_void,
                                        sub_firstline.offset(copycol as isize)
                                            as *const ::core::ffi::c_void,
                                        copy_len as size_t,
                                    );
                                    new_end = new_end.offset(copy_len as isize);
                                    if new_start_len - copy_len < sublen {
                                        sublen = new_start_len - copy_len - 1 as ::core::ffi::c_int;
                                    }
                                    let mut start_col: ::core::ffi::c_int =
                                        new_end.offset_from(new_start) as ::core::ffi::c_int;
                                    current_match.start.col = start_col as colnr_T;
                                    (*textlock.ptr()) += 1;
                                    vim_regsub_multi(
                                        &raw mut regmatch,
                                        sub_firstlnum
                                            - regmatch.startpos[0 as ::core::ffi::c_int as usize]
                                                .lnum,
                                        sub,
                                        new_end,
                                        sublen,
                                        REGSUB_COPY as ::core::ffi::c_int
                                            | REGSUB_BACKSLASH as ::core::ffi::c_int
                                            | (if magic_isset() as ::core::ffi::c_int != 0 {
                                                REGSUB_MAGIC as ::core::ffi::c_int
                                            } else {
                                                0 as ::core::ffi::c_int
                                            }),
                                    );
                                    (*textlock.ptr()) -= 1;
                                    (*sub_nsubs.ptr()) += 1;
                                    did_sub = true_0 != 0;
                                    (*curwin.get()).w_cursor.col =
                                        0 as ::core::ffi::c_int as colnr_T;
                                    copycol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                    if nmatch > 1 as ::core::ffi::c_int {
                                        sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                            + (nmatch as linenr_T - 1 as linenr_T)
                                                as ::core::ffi::c_int)
                                            as linenr_T;
                                        xfree(sub_firstline as *mut ::core::ffi::c_void);
                                        sub_firstline = xstrnsave(
                                            ml_get(sub_firstlnum),
                                            ml_get_len(sub_firstlnum) as size_t,
                                        );
                                        if sub_firstlnum <= line2 {
                                            do_again = true_0;
                                        } else {
                                            (*subflags.ptr()).do_all = false_0 != 0;
                                        }
                                    }
                                    if skip_match {
                                        xfree(sub_firstline as *mut ::core::ffi::c_void);
                                        sub_firstline = xstrdup(c"".as_ptr());
                                        copycol = 0 as ::core::ffi::c_int as colnr_T;
                                    }
                                    let mut replaced_bytes: bcount_t = 0 as bcount_t;
                                    let mut start: lpos_T =
                                        regmatch.startpos[0 as ::core::ffi::c_int as usize];
                                    let mut end: lpos_T =
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize];
                                    i = 0 as ::core::ffi::c_int;
                                    while i < nmatch - 1 as ::core::ffi::c_int {
                                        replaced_bytes += strlen(ml_get(lnum_start + i as linenr_T))
                                            as bcount_t
                                            + 1 as bcount_t;
                                        i += 1;
                                    }
                                    replaced_bytes += (end.col - start.col) as bcount_t;
                                    let mut lnum_before_newlines: linenr_T = lnum;
                                    p1 = new_end;
                                    while *p1 != 0 {
                                        if *p1.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '\\' as ::core::ffi::c_int
                                            && *p1.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                != NUL
                                        {
                                            sublen -= 1;
                                            memmove(
                                                p1 as *mut ::core::ffi::c_void,
                                                p1.offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(p1.offset(1 as ::core::ffi::c_int as isize))
                                                    .wrapping_add(1 as size_t),
                                            );
                                        } else if *p1 as ::core::ffi::c_int == CAR {
                                            if u_inssub(lnum) == OK {
                                                *p1 = NUL as ::core::ffi::c_char;
                                                ml_append(
                                                    lnum - 1 as linenr_T,
                                                    new_start,
                                                    (p1.offset_from(new_start) + 1_isize)
                                                        as colnr_T,
                                                    false_0 != 0,
                                                );
                                                mark_adjust(
                                                    lnum + 1 as linenr_T,
                                                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                                                    1 as linenr_T,
                                                    0 as linenr_T,
                                                    kExtmarkNOOP,
                                                );
                                                if (*subflags.ptr()).do_ask {
                                                    appended_lines(
                                                        lnum - 1 as linenr_T,
                                                        1 as linenr_T,
                                                    );
                                                } else {
                                                    if first_line == 0 as linenr_T {
                                                        first_line = lnum;
                                                    }
                                                    last_line = lnum + 1 as linenr_T;
                                                }
                                                sub_firstlnum += 1;
                                                lnum += 1;
                                                line2 += 1;
                                                (*curwin.get()).w_cursor.lnum += 1;
                                                memmove(
                                                    new_start as *mut ::core::ffi::c_void,
                                                    p1.offset(1 as ::core::ffi::c_int as isize)
                                                        as *const ::core::ffi::c_void,
                                                    strlen(
                                                        p1.offset(1 as ::core::ffi::c_int as isize),
                                                    )
                                                    .wrapping_add(1 as size_t),
                                                );
                                                p1 = new_start
                                                    .offset(-(1 as ::core::ffi::c_int as isize));
                                            }
                                        } else {
                                            p1 = p1.offset(
                                                (utfc_ptr2len(p1) - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            );
                                        }
                                        p1 = p1.offset(1);
                                    }
                                    let mut new_endcol: colnr_T = strlen(new_start) as colnr_T;
                                    current_match.end.col = new_endcol;
                                    current_match.end.lnum = lnum;
                                    let mut matchcols: ::core::ffi::c_int = end.col
                                        as ::core::ffi::c_int
                                        - (if end.lnum == start.lnum {
                                            start.col as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        });
                                    let mut subcols: ::core::ffi::c_int = new_endcol
                                        as ::core::ffi::c_int
                                        - (if lnum == lnum_start {
                                            start_col
                                        } else {
                                            0 as ::core::ffi::c_int
                                        });
                                    if !did_save {
                                        u_save_cursor();
                                        did_save = true_0 != 0;
                                    }
                                    if line_matches.size == line_matches.capacity {
                                        line_matches.capacity = if line_matches.capacity != 0 {
                                            line_matches.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                        line_matches.items = xrealloc(
                                            line_matches.items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<LineData>()
                                                .wrapping_mul(line_matches.capacity),
                                        )
                                            as *mut LineData;
                                    } else {
                                    };
                                    let c2rust_fresh9 = line_matches.size;
                                    line_matches.size = line_matches.size.wrapping_add(1);
                                    let mut data: *mut LineData =
                                        line_matches.items.add(c2rust_fresh9);
                                    (*data).start_col = start_col;
                                    (*data).start = start;
                                    (*data).end = end;
                                    (*data).matchcols = matchcols;
                                    (*data).matchbytes = replaced_bytes;
                                    (*data).subcols = subcols;
                                    (*data).subbytes =
                                        (sublen - 1 as ::core::ffi::c_int) as bcount_t;
                                    (*data).lnum_before = lnum_before_newlines;
                                    (*data).lnum_after = lnum;
                                }
                            }
                        }
                    }
                    lastone = (skip_match as ::core::ffi::c_int != 0
                        || got_int.get() as ::core::ffi::c_int != 0
                        || got_quit as ::core::ffi::c_int != 0
                        || lnum > line2
                        || !((*subflags.ptr()).do_all as ::core::ffi::c_int != 0 || do_again != 0)
                        || *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int == NUL
                            && nmatch <= 1 as ::core::ffi::c_int
                            && re_multiline(regmatch.regprog) == 0)
                        as ::core::ffi::c_int;
                    nmatch = -1 as ::core::ffi::c_int;
                    if lastone != 0
                        || nmatch_tl > 0 as linenr_T
                        || {
                            nmatch = vim_regexec_multi(
                                &raw mut regmatch,
                                curwin.get(),
                                curbuf.get(),
                                sub_firstlnum,
                                matchcol,
                                ::core::ptr::null_mut::<proftime_T>(),
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            );
                            nmatch == 0 as ::core::ffi::c_int
                        }
                        || regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                    {
                        if !new_start.is_null() {
                            strcat(new_start, sub_firstline.offset(copycol as isize));
                            matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                            prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                            if u_savesub(lnum) != OK {
                                break;
                            }
                            ml_replace(lnum, new_start, true_0 != 0);
                            let mut match_idx: size_t = 0 as size_t;
                            while match_idx < line_matches.size {
                                let mut match_0: *mut LineData = line_matches.items.add(match_idx);
                                extmark_splice(
                                    curbuf.get(),
                                    (*match_0).lnum_before as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int,
                                    (*match_0).start_col as colnr_T,
                                    (*match_0).end.lnum as ::core::ffi::c_int
                                        - (*match_0).start.lnum as ::core::ffi::c_int,
                                    (*match_0).matchcols as colnr_T,
                                    (*match_0).matchbytes,
                                    (*match_0).lnum_after as ::core::ffi::c_int
                                        - (*match_0).lnum_before as ::core::ffi::c_int,
                                    (*match_0).subcols as colnr_T,
                                    (*match_0).subbytes,
                                    kExtmarkUndo,
                                );
                                match_idx = match_idx.wrapping_add(1);
                            }
                            line_matches.size = 0 as size_t;
                            if nmatch_tl > 0 as linenr_T {
                                lnum += 1;
                                if u_savedel(lnum, nmatch_tl) != OK {
                                    break;
                                }
                                i = 0 as ::core::ffi::c_int;
                                while (i as linenr_T) < nmatch_tl {
                                    ml_delete(lnum);
                                    i += 1;
                                }
                                mark_adjust(
                                    lnum,
                                    lnum + nmatch_tl - 1 as linenr_T,
                                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                                    -nmatch_tl,
                                    kExtmarkNOOP,
                                );
                                if (*subflags.ptr()).do_ask {
                                    deleted_lines(lnum, nmatch_tl);
                                }
                                lnum -= 1;
                                line2 -= nmatch_tl;
                                nmatch_tl = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            if (*subflags.ptr()).do_ask {
                                changed_bytes(lnum, 0 as colnr_T);
                            } else {
                                if first_line == 0 as linenr_T {
                                    first_line = lnum;
                                }
                                last_line = lnum + 1 as linenr_T;
                            }
                            sub_firstlnum = lnum;
                            xfree(sub_firstline as *mut ::core::ffi::c_void);
                            sub_firstline = new_start;
                            new_start = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                            prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                            copycol = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if nmatch == -1 as ::core::ffi::c_int && lastone == 0 {
                            nmatch = vim_regexec_multi(
                                &raw mut regmatch,
                                curwin.get(),
                                curbuf.get(),
                                sub_firstlnum,
                                matchcol,
                                ::core::ptr::null_mut::<proftime_T>(),
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            );
                        }
                        if nmatch <= 0 as ::core::ffi::c_int {
                            if nmatch == -1 as ::core::ffi::c_int {
                                lnum -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                            }
                            if cmdpreview_ns > 0 as ::core::ffi::c_int {
                                let mut match_lines: linenr_T = current_match.end.lnum
                                    - current_match.start.lnum
                                    + 1 as linenr_T;
                                if preview_lines.subresults.size > 0 as size_t {
                                    let mut last: linenr_T = (*preview_lines.subresults.items.add(
                                        preview_lines
                                            .subresults
                                            .size
                                            .wrapping_sub(0 as size_t)
                                            .wrapping_sub(1 as size_t),
                                    ))
                                    .end
                                    .lnum;
                                    if last == current_match.start.lnum {
                                        preview_lines.lines_needed = (preview_lines.lines_needed
                                            as ::core::ffi::c_int
                                            + (match_lines - 1 as linenr_T) as ::core::ffi::c_int)
                                            as linenr_T;
                                    } else {
                                        preview_lines.lines_needed += match_lines;
                                    }
                                } else {
                                    preview_lines.lines_needed += match_lines;
                                }
                                if preview_lines.subresults.size
                                    == preview_lines.subresults.capacity
                                {
                                    preview_lines.subresults.capacity =
                                        if preview_lines.subresults.capacity != 0 {
                                            preview_lines.subresults.capacity
                                                << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                    preview_lines.subresults.items = xrealloc(
                                        preview_lines.subresults.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<SubResult>()
                                            .wrapping_mul(preview_lines.subresults.capacity),
                                    )
                                        as *mut SubResult;
                                } else {
                                };
                                let c2rust_fresh10 = preview_lines.subresults.size;
                                preview_lines.subresults.size =
                                    preview_lines.subresults.size.wrapping_add(1);
                                *preview_lines.subresults.items.add(c2rust_fresh10) = current_match;
                            }
                            break;
                        }
                    }
                    if cmdpreview_ns > 0 as ::core::ffi::c_int {
                        let mut match_lines_0: linenr_T =
                            current_match.end.lnum - current_match.start.lnum + 1 as linenr_T;
                        if preview_lines.subresults.size > 0 as size_t {
                            let mut last_0: linenr_T = (*preview_lines.subresults.items.add(
                                preview_lines
                                    .subresults
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t),
                            ))
                            .end
                            .lnum;
                            if last_0 == current_match.start.lnum {
                                preview_lines.lines_needed = (preview_lines.lines_needed
                                    as ::core::ffi::c_int
                                    + (match_lines_0 - 1 as linenr_T) as ::core::ffi::c_int)
                                    as linenr_T;
                            } else {
                                preview_lines.lines_needed += match_lines_0;
                            }
                        } else {
                            preview_lines.lines_needed += match_lines_0;
                        }
                        if preview_lines.subresults.size == preview_lines.subresults.capacity {
                            preview_lines.subresults.capacity =
                                if preview_lines.subresults.capacity != 0 {
                                    preview_lines.subresults.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    8 as size_t
                                };
                            preview_lines.subresults.items = xrealloc(
                                preview_lines.subresults.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<SubResult>()
                                    .wrapping_mul(preview_lines.subresults.capacity),
                            )
                                as *mut SubResult;
                        } else {
                        };
                        let c2rust_fresh11 = preview_lines.subresults.size;
                        preview_lines.subresults.size =
                            preview_lines.subresults.size.wrapping_add(1);
                        *preview_lines.subresults.items.add(c2rust_fresh11) = current_match;
                    }
                    line_breakcheck();
                }
                if did_sub {
                    (*sub_nlines.ptr()) += 1;
                }
                xfree(new_start as *mut ::core::ffi::c_void);
                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                    &raw mut sub_firstline as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__0);
                *ptr__0 = NULL_0;
                let _ = *ptr__0;
                xfree(line_matches.items as *mut ::core::ffi::c_void);
                line_matches.capacity = 0 as size_t;
                line_matches.size = line_matches.capacity;
                line_matches.items = ::core::ptr::null_mut::<LineData>();
            }
            line_breakcheck();
            if profile_passed_limit(timeout) {
                got_quit = true_0 != 0;
            }
            lnum += 1;
        }
        (*curbuf.get()).deleted_bytes2 = 0 as size_t;
        if first_line != 0 as linenr_T {
            i = ((*curbuf.get()).b_ml.ml_line_count - old_line_count) as ::core::ffi::c_int;
            changed_lines(
                curbuf.get(),
                first_line,
                0 as colnr_T,
                last_line - i as linenr_T,
                i as linenr_T,
                false_0 != 0,
            );
            let mut num_added: int64_t = (last_line - first_line) as int64_t;
            let mut num_removed: int64_t = num_added - i as int64_t;
            buf_updates_send_changes(curbuf.get(), first_line, num_added, num_removed);
        }
        xfree(sub_firstline as *mut ::core::ffi::c_void);
        if (*subflags.ptr()).do_count {
            (*curwin.get()).w_cursor = old_cursor;
        }
        if sub_nsubs.get() > start_nsubs {
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*curbuf.get()).b_op_start.lnum = (*eap).line1;
                (*curbuf.get()).b_op_end.lnum = line2;
                (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
            }
            if global_busy.get() == 0 {
                if !(*subflags.ptr()).do_ask {
                    if endcolumn {
                        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
                    } else {
                        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                    }
                }
                if cmdpreview_ns <= 0 as ::core::ffi::c_int
                    && !do_sub_msg((*subflags.ptr()).do_count)
                    && (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0
                    && p_ch.get() > 0 as OptInt
                {
                    msg(c"".as_ptr(), 0 as ::core::ffi::c_int);
                }
            } else {
                global_need_beginline.set(true_0);
            }
            if (*subflags.ptr()).do_print {
                print_line(
                    (*curwin.get()).w_cursor.lnum,
                    (*subflags.ptr()).do_number,
                    (*subflags.ptr()).do_list,
                    true_0 != 0,
                );
            }
        } else if global_busy.get() == 0 {
            if got_int.get() {
                emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
            } else if got_match {
                if p_ch.get() > 0 as OptInt && !ui_has(kUIMessages) {
                    msg(c"".as_ptr(), 0 as ::core::ffi::c_int);
                }
            } else if (*subflags.ptr()).do_error {
                semsg_c!(
                    gettext(&raw const e_patnotf2 as *const ::core::ffi::c_char),
                    get_search_pat(),
                );
            }
        }
        if (*subflags.ptr()).do_ask as ::core::ffi::c_int != 0 && hasAnyFolding(curwin.get()) != 0 {
            changed_window_setting(curwin.get());
        }
        vim_regfree(regmatch.regprog);
        xfree(sub as *mut ::core::ffi::c_void);
        (*subflags.ptr()).do_all = save_do_all;
        (*subflags.ptr()).do_ask = save_do_ask;
        let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if cmdpreview_ns > 0 as ::core::ffi::c_int && !aborting() {
            if got_quit as ::core::ffi::c_int != 0
                || profile_passed_limit(timeout) as ::core::ffi::c_int != 0
            {
                set_option_direct(
                    kOptInccommand,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: String_0 {
                                data: c"".as_ptr() as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                                    .wrapping_sub(1 as size_t),
                            },
                        },
                    },
                    0 as ::core::ffi::c_int,
                    SID_NONE,
                );
            } else if *p_icm.get() as ::core::ffi::c_int != NUL && !pat.is_null() {
                if pre_hl_id.get() == 0 as ::core::ffi::c_int {
                    pre_hl_id.set(syn_check_group(
                        c"Substitute".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>()
                            .wrapping_sub(1 as size_t),
                    ));
                }
                retv = show_sub(
                    eap,
                    old_cursor,
                    &raw mut preview_lines,
                    pre_hl_id.get(),
                    cmdpreview_ns,
                    cmdpreview_bufnr,
                );
            }
        }
        xfree(preview_lines.subresults.items as *mut ::core::ffi::c_void);
        preview_lines.subresults.capacity = 0 as size_t;
        preview_lines.subresults.size = preview_lines.subresults.capacity;
        preview_lines.subresults.items = ::core::ptr::null_mut::<SubResult>();
        return retv;
    }
}
