//! `'incsearch'`: previewing a `/`, `?` or `:s` pattern while it is typed.
//!
//! [`may_do_incsearch_highlighting`] runs on every command-line change and,
//! for the commands that take a pattern, searches from the saved view state
//! and highlights what it found.  [`parse_pattern_and_range`] is what decides
//! whether the line being typed *is* such a command, and the `viewstate_T`
//! pair saves and restores the window the preview scrolled.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn trigger_cmd_autocmd(
    mut typechar: ::core::ffi::c_int,
    mut evt: event_T,
) {
    unsafe {
        let mut typestr: [::core::ffi::c_char; 2] =
            [typechar as ::core::ffi::c_char, NUL as ::core::ffi::c_char];
        apply_autocmds(
            evt,
            &raw mut typestr as *mut ::core::ffi::c_char,
            &raw mut typestr as *mut ::core::ffi::c_char,
            false_0 != 0,
            curbuf.get(),
        );
    }
}

pub(crate) unsafe extern "C" fn save_viewstate(mut wp: *mut win_T, mut vs: *mut viewstate_T) {
    unsafe {
        (*vs).vs_curswant = (*wp).w_curswant;
        (*vs).vs_leftcol = (*wp).w_leftcol;
        (*vs).vs_skipcol = (*wp).w_skipcol;
        (*vs).vs_topline = (*wp).w_topline;
        (*vs).vs_topfill = (*wp).w_topfill;
        (*vs).vs_botline = (*wp).w_botline;
        (*vs).vs_empty_rows = (*wp).w_empty_rows;
    }
}

pub(crate) unsafe extern "C" fn restore_viewstate(mut wp: *mut win_T, mut vs: *mut viewstate_T) {
    unsafe {
        (*wp).w_curswant = (*vs).vs_curswant;
        (*wp).w_leftcol = (*vs).vs_leftcol;
        (*wp).w_skipcol = (*vs).vs_skipcol;
        (*wp).w_topline = (*vs).vs_topline;
        (*wp).w_topfill = (*vs).vs_topfill;
        (*wp).w_botline = (*vs).vs_botline;
        (*wp).w_empty_rows = (*vs).vs_empty_rows;
    }
}

pub(crate) unsafe extern "C" fn init_incsearch_state(mut s: *mut incsearch_state_T) {
    unsafe {
        (*s).winid = (*curwin.get()).handle;
        (*s).match_start = (*curwin.get()).w_cursor;
        (*s).did_incsearch = false_0 != 0;
        (*s).incsearch_postponed = false_0 != 0;
        (*s).magic_overruled_save = magic_overruled.get();
        clearpos(&mut (*s).match_end);
        (*s).save_cursor = (*curwin.get()).w_cursor;
        (*s).search_start = (*curwin.get()).w_cursor;
        save_viewstate(curwin.get(), &raw mut (*s).init_viewstate);
        save_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
    }
}

pub(crate) unsafe extern "C" fn set_search_match(mut t: *mut pos_T) {
    unsafe {
        (*t).lnum += search_match_lines.get();
        (*t).col = search_match_endcol.get();
        if (*t).lnum > (*curbuf.get()).b_ml.ml_line_count {
            (*t).lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
    }
}

pub unsafe extern "C" fn parse_pattern_and_range(
    mut incsearch_start: *mut pos_T,
    mut search_delim: *mut ::core::ffi::c_int,
    mut skiplen: *mut ::core::ffi::c_int,
    mut patlen: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut delim_optional: bool = false_0 != 0;
        let mut dummy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut magic: magic_T = 0 as magic_T;
        *skiplen = 0 as ::core::ffi::c_int;
        *patlen = (*ccline.ptr()).cmdlen;
        search_first_line.set(0 as ::core::ffi::c_int as linenr_T);
        search_last_line.set(MAXLNUM as ::core::ffi::c_int as linenr_T);
        let mut ea: exarg_T = exarg {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: (*ccline.ptr()).cmdbuff,
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 1 as linenr_T,
            line2: 1 as linenr_T,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut dummy_cmdmod: cmdmod_T = cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        };
        parse_command_modifiers(
            &raw mut ea,
            &raw mut dummy,
            &raw mut dummy_cmdmod,
            true_0 != 0,
        );
        let mut cmd: *mut ::core::ffi::c_char =
            skip_range(ea.cmd, ::core::ptr::null_mut::<::core::ffi::c_int>());
        if vim_strchr(
            b"sgvlu\0".as_ptr() as *const ::core::ffi::c_char,
            *cmd as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return false_0 != 0;
        }
        p = cmd;
        while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        {
            p = p.offset(1);
        }
        if *skipwhite(p) as ::core::ffi::c_int == NUL {
            return false_0 != 0;
        }
        if strncmp(
            cmd,
            b"substitute\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(cmd) as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"smagic\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(cmd) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"snomagic\0".as_ptr() as *const ::core::ffi::c_char,
                (if p.offset_from(cmd) > 3 as isize {
                    p.offset_from(cmd)
                } else {
                    3 as isize
                }) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"vglobal\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(cmd) as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'm' as ::core::ffi::c_int
            {
                magic_overruled.set(OPTION_MAGIC_ON);
            } else if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'n' as ::core::ffi::c_int
            {
                magic_overruled.set(OPTION_MAGIC_OFF);
            }
        } else if strncmp(
            cmd,
            b"sort\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 3 as isize {
                p.offset_from(cmd)
            } else {
                3 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"uniq\0".as_ptr() as *const ::core::ffi::c_char,
                (if p.offset_from(cmd) > 3 as isize {
                    p.offset_from(cmd)
                } else {
                    3 as isize
                }) as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            }
            loop {
                p = skipwhite(p);
                if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint && {
                    p = skipwhite(p);
                    *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                } || {
                    p = skipwhite(p);
                    *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint && {
                        p = skipwhite(p);
                        *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                    }
                }) {
                    break;
                }
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                return false_0 != 0;
            }
        } else if strncmp(
            cmd,
            b"vimgrep\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 3 as isize {
                p.offset_from(cmd)
            } else {
                3 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"vimgrepadd\0".as_ptr() as *const ::core::ffi::c_char,
                (if p.offset_from(cmd) > 8 as isize {
                    p.offset_from(cmd)
                } else {
                    8 as isize
                }) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"lvimgrep\0".as_ptr() as *const ::core::ffi::c_char,
                (if p.offset_from(cmd) > 2 as isize {
                    p.offset_from(cmd)
                } else {
                    2 as isize
                }) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"lvimgrepadd\0".as_ptr() as *const ::core::ffi::c_char,
                (if p.offset_from(cmd) > 9 as isize {
                    p.offset_from(cmd)
                } else {
                    9 as isize
                }) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncmp(
                cmd,
                b"global\0".as_ptr() as *const ::core::ffi::c_char,
                p.offset_from(cmd) as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                p = p.offset(1);
                if *skipwhite(p) as ::core::ffi::c_int == NUL {
                    return false_0 != 0;
                }
            }
            if *cmd as ::core::ffi::c_int != 'g' as ::core::ffi::c_int {
                delim_optional = true_0 != 0;
            }
        } else {
            return false_0 != 0;
        }
        p = skipwhite(p);
        let mut delim: ::core::ffi::c_int = if delim_optional as ::core::ffi::c_int != 0
            && vim_isIDc(*p as uint8_t as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            ' ' as ::core::ffi::c_int
        } else {
            let c2rust_fresh0 = p;
            p = p.offset(1);
            *c2rust_fresh0 as ::core::ffi::c_int
        };
        *search_delim = delim;
        let mut end: *mut ::core::ffi::c_char = skip_regexp_ex(
            p,
            delim,
            magic_isset() as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut magic,
        );
        let mut use_last_pat: bool = end == p && *end as ::core::ffi::c_int == delim;
        if end == p && !use_last_pat {
            return false_0 != 0;
        }
        if !use_last_pat {
            let mut c: ::core::ffi::c_char = *end;
            *end = NUL as ::core::ffi::c_char;
            let mut empty: bool = empty_pattern_magic(p, end.offset_from(p) as size_t, magic);
            *end = c;
            if empty {
                return false_0 != 0;
            }
        }
        *skiplen = p.offset_from((*ccline.ptr()).cmdbuff) as ::core::ffi::c_int;
        *patlen = end.offset_from(p) as ::core::ffi::c_int;
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = *incsearch_start;
        parse_cmd_address(&raw mut ea, &raw mut dummy, true_0 != 0);
        if ea.addr_count > 0 as ::core::ffi::c_int {
            search_first_line.set(if ea.line2 < ea.line1 {
                ea.line2
            } else {
                ea.line1
            });
            search_last_line.set(if ea.line2 > ea.line1 {
                ea.line2
            } else {
                ea.line1
            });
        } else if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 's' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 'o' as ::core::ffi::c_int
        {
            search_last_line.set((*curwin.get()).w_cursor.lnum);
            search_first_line.set(search_last_line.get());
        }
        (*curwin.get()).w_cursor = save_cursor;
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn do_incsearch_highlighting(
    mut firstc: ::core::ffi::c_int,
    mut search_delim: *mut ::core::ffi::c_int,
    mut is_state: *mut incsearch_state_T,
    mut skiplen: *mut ::core::ffi::c_int,
    mut patlen: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut retval: bool = false_0 != 0;
        *skiplen = 0 as ::core::ffi::c_int;
        *patlen = (*ccline.ptr()).cmdlen;
        if p_is.get() == 0 || cmd_silent.get() as ::core::ffi::c_int != 0 {
            return false_0 != 0;
        }
        search_first_line.set(0 as ::core::ffi::c_int as linenr_T);
        search_last_line.set(MAXLNUM as ::core::ffi::c_int as linenr_T);
        if firstc == '/' as ::core::ffi::c_int || firstc == '?' as ::core::ffi::c_int {
            *search_delim = firstc;
            return true_0 != 0;
        }
        if firstc != ':' as ::core::ffi::c_int {
            return false_0 != 0;
        }
        (*emsg_off.ptr()) += 1;
        retval = parse_pattern_and_range(
            &raw mut (*is_state).search_start,
            search_delim,
            skiplen,
            patlen,
        );
        (*emsg_off.ptr()) -= 1;
        return retval;
    }
}

pub(crate) unsafe extern "C" fn may_do_incsearch_highlighting(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
) {
    unsafe {
        let mut skiplen: ::core::ffi::c_int = 0;
        let mut patlen: ::core::ffi::c_int = 0;
        let mut search_delim: ::core::ffi::c_int = 0;
        save_last_search_pattern();
        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            finish_incsearch_highlighting(false_0 != 0, s, true_0 != 0);
            return;
        }
        if char_avail() {
            restore_last_search_pattern();
            (*s).incsearch_postponed = true_0 != 0;
            return;
        }
        (*s).incsearch_postponed = false_0 != 0;
        let mut next_char: ::core::ffi::c_char =
            *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize);
        let mut use_last_pat: bool = patlen == 0 as ::core::ffi::c_int
            && skiplen > 0 as ::core::ffi::c_int
            && *(*ccline.ptr())
                .cmdbuff
                .offset((skiplen - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == next_char as ::core::ffi::c_int;
        if patlen != 0 as ::core::ffi::c_int || use_last_pat as ::core::ffi::c_int != 0 {
            ui_busy_start();
            ui_flush();
        }
        if search_first_line.get() == 0 as linenr_T {
            (*curwin.get()).w_cursor = (*s).search_start;
        } else if search_first_line.get() > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            (*curwin.get()).w_cursor.col = MAXCOL as ::core::ffi::c_int as colnr_T;
        } else {
            (*curwin.get()).w_cursor.lnum = search_first_line.get();
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if patlen != 0 as ::core::ffi::c_int || use_last_pat as ::core::ffi::c_int != 0 {
            let mut search_flags: ::core::ffi::c_int = SEARCH_OPT as ::core::ffi::c_int
                + SEARCH_NOOF as ::core::ffi::c_int
                + SEARCH_PEEK as ::core::ffi::c_int;
            if p_hls.get() == 0 {
                search_flags += SEARCH_KEEP as ::core::ffi::c_int;
            }
            if search_first_line.get() != 0 as linenr_T {
                search_flags += SEARCH_START as ::core::ffi::c_int;
            }
            let mut tm: proftime_T = profile_setlimit(500 as int64_t);
            let mut sia: searchit_arg_T = searchit_arg_T {
                sa_stop_lnum: 0,
                sa_tm: &raw mut tm,
                sa_timed_out: 0,
                sa_wrapped: 0,
            };
            *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize) =
                NUL as ::core::ffi::c_char;
            (*emsg_off.ptr()) += 1;
            found = do_search(
                ::core::ptr::null_mut::<oparg_T>(),
                if firstc == ':' as ::core::ffi::c_int {
                    '/' as ::core::ffi::c_int
                } else {
                    firstc
                },
                search_delim,
                (*ccline.ptr()).cmdbuff.offset(skiplen as isize),
                patlen as size_t,
                count,
                search_flags,
                &raw mut sia,
            );
            (*emsg_off.ptr()) -= 1;
            *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize) = next_char;
            if (*curwin.get()).w_cursor.lnum < search_first_line.get()
                || (*curwin.get()).w_cursor.lnum > search_last_line.get()
            {
                found = 0 as ::core::ffi::c_int;
                (*curwin.get()).w_cursor = (*s).search_start;
            }
            if got_int.get() {
                vpeekc();
                got_int.set(false_0 != 0);
                found = 0 as ::core::ffi::c_int;
            } else if char_avail() {
                (*s).incsearch_postponed = true_0 != 0;
            }
            ui_busy_stop();
        } else {
            set_no_hlsearch(true_0 != 0);
            redraw_all_later(UPD_SOME_VALID);
        }
        highlight_match.set(found != 0 as ::core::ffi::c_int);
        restore_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
        changed_cline_bef_curs(curwin.get());
        update_topline(curwin.get());
        let mut end_pos: pos_T = (*curwin.get()).w_cursor;
        if found != 0 as ::core::ffi::c_int {
            (*s).match_start = (*curwin.get()).w_cursor;
            set_search_match(&raw mut (*curwin.get()).w_cursor);
            validate_cursor(curwin.get());
            (*s).match_end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = end_pos;
            end_pos = (*s).match_end;
        }
        if !use_last_pat {
            next_char = *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize);
            *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize) =
                NUL as ::core::ffi::c_char;
            if empty_pattern(
                (*ccline.ptr()).cmdbuff.offset(skiplen as isize),
                patlen as size_t,
                search_delim,
            ) as ::core::ffi::c_int
                != 0
                && !no_hlsearch.get()
            {
                redraw_all_later(UPD_SOME_VALID);
                set_no_hlsearch(true_0 != 0);
            }
            *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize) = next_char;
        }
        validate_cursor(curwin.get());
        if p_ru.get() != 0
            && ((*curwin.get()).w_status_height > 0 as ::core::ffi::c_int
                || global_stl_height() > 0 as ::core::ffi::c_int)
        {
            (*curwin.get()).w_redr_status = true_0 != 0;
        }
        redraw_later(curwin.get(), UPD_SOME_VALID);
        update_screen();
        highlight_match.set(false_0 != 0);
        restore_last_search_pattern();
        if *(*ccline.ptr()).cmdbuff.offset((skiplen + patlen) as isize) as ::core::ffi::c_int != NUL
        {
            (*curwin.get()).w_cursor = (*s).search_start;
        } else if found != 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor = end_pos;
            (*curwin.get()).w_valid_cursor = end_pos;
        }
        msg_starthere();
        redrawcmdline();
        (*s).did_incsearch = true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn may_add_char_to_search(
    mut firstc: ::core::ffi::c_int,
    mut c: *mut ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut skiplen: ::core::ffi::c_int = 0;
        let mut patlen: ::core::ffi::c_int = 0;
        let mut search_delim: ::core::ffi::c_int = 0;
        save_last_search_pattern();
        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            return FAIL;
        }
        restore_last_search_pattern();
        if (*s).did_incsearch {
            (*curwin.get()).w_cursor = (*s).match_end;
            *c = gchar_cursor();
            if *c != NUL {
                if p_ic.get() != 0
                    && p_scs.get() != 0
                    && !pat_has_uppercase((*ccline.ptr()).cmdbuff.offset(skiplen as isize))
                {
                    *c = mb_tolower(*c);
                }
                if *c == search_delim
                    || !vim_strchr(
                        if magic_isset() as ::core::ffi::c_int != 0 {
                            b"\\~^$.*[\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"\\^$\0".as_ptr() as *const ::core::ffi::c_char
                        },
                        *c,
                    )
                    .is_null()
                {
                    stuffcharReadbuff(*c);
                    *c = '\\' as ::core::ffi::c_int;
                }
                if utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                    let save_c: ::core::ffi::c_int = *c;
                    while utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                        (*curwin.get()).w_cursor.col += utf_char2len(*c);
                        *c = gchar_cursor();
                        stuffcharReadbuff(*c);
                    }
                    *c = save_c;
                }
                return FAIL;
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn finish_incsearch_highlighting(
    mut gotesc: bool,
    mut s: *mut incsearch_state_T,
    mut call_update_screen: bool,
) {
    unsafe {
        if !(*s).did_incsearch {
            return;
        }
        (*s).did_incsearch = false_0 != 0;
        if gotesc {
            (*curwin.get()).w_cursor = (*s).save_cursor;
        } else {
            if !equalpos((*s).save_cursor, (*s).search_start) {
                (*curwin.get()).w_cursor = (*s).save_cursor;
                setpcmark();
            }
            (*curwin.get()).w_cursor = (*s).search_start;
        }
        restore_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
        highlight_match.set(false_0 != 0);
        search_first_line.set(0 as ::core::ffi::c_int as linenr_T);
        search_last_line.set(MAXLNUM as ::core::ffi::c_int as linenr_T);
        magic_overruled.set((*s).magic_overruled_save);
        validate_cursor(curwin.get());
        status_redraw_all();
        redraw_all_later(UPD_SOME_VALID);
        if call_update_screen {
            update_screen();
        }
    }
}

pub(crate) unsafe extern "C" fn may_do_command_line_next_incsearch(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
    mut next_match: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut skiplen: ::core::ffi::c_int = 0;
        let mut patlen: ::core::ffi::c_int = 0;
        let mut search_delim: ::core::ffi::c_int = 0;
        save_last_search_pattern();
        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            return OK;
        }
        if patlen == 0 as ::core::ffi::c_int
            && *(*ccline.ptr()).cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int == NUL
        {
            restore_last_search_pattern();
            return FAIL;
        }
        ui_busy_start();
        ui_flush();
        let mut t: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_flags: ::core::ffi::c_int = SEARCH_NOOF as ::core::ffi::c_int;
        if search_delim == *(*ccline.ptr()).cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int {
            pat = last_search_pattern();
            if pat.is_null() {
                restore_last_search_pattern();
                return FAIL;
            }
            skiplen = 0 as ::core::ffi::c_int;
            patlen = last_search_pattern_len() as ::core::ffi::c_int;
        } else {
            pat = (*ccline.ptr()).cmdbuff.offset(skiplen as isize);
        }
        let mut bslsh: bool = false_0 != 0;
        if patlen > 2 as ::core::ffi::c_int
            && firstc
                == *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        {
            patlen -= 1;
            if *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
            {
                *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) =
                    firstc as uint8_t as ::core::ffi::c_char;
                bslsh = true_0 != 0;
            }
        }
        if next_match {
            t = (*s).match_end;
            if lt((*s).match_start, (*s).match_end) {
                decl(&raw mut t);
            }
            search_flags += SEARCH_COL as ::core::ffi::c_int;
        } else {
            t = (*s).match_start;
        }
        if p_hls.get() == 0 {
            search_flags += SEARCH_KEEP as ::core::ffi::c_int;
        }
        (*emsg_off.ptr()) += 1;
        let mut save: ::core::ffi::c_char = *pat.offset(patlen as isize);
        *pat.offset(patlen as isize) = NUL as ::core::ffi::c_char;
        let mut found: ::core::ffi::c_int = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut t,
            ::core::ptr::null_mut::<pos_T>(),
            (if next_match as ::core::ffi::c_int != 0 {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
            pat,
            patlen as size_t,
            count,
            search_flags,
            RE_SEARCH as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        );
        (*emsg_off.ptr()) -= 1;
        *pat.offset(patlen as isize) = save;
        if bslsh {
            *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) = '\\' as ::core::ffi::c_char;
        }
        ui_busy_stop();
        if found != 0 {
            (*s).search_start = (*s).match_start;
            (*s).match_end = t;
            (*s).match_start = t;
            if !next_match && firstc != '?' as ::core::ffi::c_int {
                (*s).search_start = t;
                decl(&raw mut (*s).search_start);
            } else if next_match as ::core::ffi::c_int != 0 && firstc == '?' as ::core::ffi::c_int {
                (*s).search_start = t;
                incl(&raw mut (*s).search_start);
            }
            if lt(t, (*s).search_start) as ::core::ffi::c_int != 0
                && next_match as ::core::ffi::c_int != 0
            {
                (*s).search_start = t;
                if firstc == '?' as ::core::ffi::c_int {
                    incl(&raw mut (*s).search_start);
                } else {
                    decl(&raw mut (*s).search_start);
                }
            }
            set_search_match(&raw mut (*s).match_end);
            (*curwin.get()).w_cursor = (*s).match_start;
            changed_cline_bef_curs(curwin.get());
            update_topline(curwin.get());
            validate_cursor(curwin.get());
            highlight_match.set(true_0 != 0);
            save_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
            redraw_later(curwin.get(), UPD_NOT_VALID);
            update_screen();
            highlight_match.set(false_0 != 0);
            redrawcmdline();
            (*curwin.get()).w_cursor = (*s).match_end;
        } else {
            vim_beep(kOptBoFlagError as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        restore_last_search_pattern();
        return FAIL;
    }
}

pub(crate) unsafe extern "C" fn empty_pattern(
    mut p: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut delim: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut magic_val: magic_T = MAGIC_ON;
        if len > 0 as size_t {
            skip_regexp_ex(
                p,
                delim,
                magic_isset() as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut magic_val,
            );
        } else {
            return true_0 != 0;
        }
        return empty_pattern_magic(p, len, magic_val);
    }
}

pub(crate) unsafe extern "C" fn empty_pattern_magic(
    mut p: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut magic_val: magic_T,
) -> bool {
    unsafe {
        while len >= 2 as size_t
            && *p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
            && !vim_strchr(
                b"mMvVcCZ\0".as_ptr() as *const ::core::ffi::c_char,
                *p.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            len = len.wrapping_sub(2 as size_t);
        }
        return len == 0 as size_t
            || len > 1 as size_t
                && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '|' as ::core::ffi::c_int
                && (*p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && magic_val as ::core::ffi::c_uint
                        == MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
                    || *p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                        && magic_val as ::core::ffi::c_uint
                            == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
}
