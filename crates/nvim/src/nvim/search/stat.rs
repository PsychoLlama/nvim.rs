//! Counting the matches, for `[N/M]` and for `searchcount()`.
//!
//! [`update_search_stat`] does the counting — forwards from the top of the
//! buffer, giving up after a timeout or after `maxcount` matches — and
//! caches the answer against the buffer's changedtick so that repeating
//! the search is cheap. [`cmdline_search_stat`] renders it into the
//! message line; [`f_searchcount`] is the Vimscript view of the same
//! numbers.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cmdline_search_stat(
    mut dirc: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut cursor_pos: *mut pos_T,
    mut show_top_bot_msg: bool,
    mut msgbuf: *mut ::core::ffi::c_char,
    mut msgbuflen: size_t,
    mut recompute: bool,
    mut maxcount: ::core::ffi::c_int,
    mut timeout: ::core::ffi::c_int,
) {
    unsafe {
        let mut stat: searchstat_T = searchstat_T {
            cur: 0,
            cnt: 0,
            exact_match: false,
            incomplete: 0,
            last_maxcount: 0,
        };
        update_search_stat(
            dirc,
            pos,
            cursor_pos,
            &raw mut stat,
            recompute,
            maxcount,
            timeout,
        );
        if stat.cur <= 0 as ::core::ffi::c_int {
            return;
        }
        let mut t: [::core::ffi::c_char; 16] = [0; 16];
        let mut len: size_t = 0;
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
        {
            if stat.incomplete == 1 as ::core::ffi::c_int {
                len = vim_snprintf(
                    &raw mut t as *mut ::core::ffi::c_char,
                    SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"[?/??]\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t;
            } else if stat.cnt > maxcount && stat.cur > maxcount {
                len = vim_snprintf(
                    &raw mut t as *mut ::core::ffi::c_char,
                    SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"[>%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
                    maxcount,
                    maxcount,
                ) as size_t;
            } else if stat.cnt > maxcount {
                len = vim_snprintf(
                    &raw mut t as *mut ::core::ffi::c_char,
                    SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"[>%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
                    maxcount,
                    stat.cur,
                ) as size_t;
            } else {
                len = vim_snprintf(
                    &raw mut t as *mut ::core::ffi::c_char,
                    SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                    b"[%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
                    stat.cnt,
                    stat.cur,
                ) as size_t;
            }
        } else if stat.incomplete == 1 as ::core::ffi::c_int {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[?/??]\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        } else if stat.cnt > maxcount && stat.cur > maxcount {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[>%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
                maxcount,
                maxcount,
            ) as size_t;
        } else if stat.cnt > maxcount {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[%d/>%d]\0".as_ptr() as *const ::core::ffi::c_char,
                stat.cur,
                maxcount,
            ) as size_t;
        } else {
            len = vim_snprintf(
                &raw mut t as *mut ::core::ffi::c_char,
                SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t,
                b"[%d/%d]\0".as_ptr() as *const ::core::ffi::c_char,
                stat.cur,
                stat.cnt,
            ) as size_t;
        }
        if show_top_bot_msg as ::core::ffi::c_int != 0
            && len.wrapping_add(2 as size_t) < SEARCH_STAT_BUF_LEN as ::core::ffi::c_int as size_t
        {
            memmove(
                (&raw mut t as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                &raw mut t as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                len,
            );
            t[0 as ::core::ffi::c_int as usize] = 'W' as ::core::ffi::c_char;
            t[1 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
            len = len.wrapping_add(2 as size_t);
        }
        if len > msgbuflen {
            len = msgbuflen;
        }
        memmove(
            msgbuf.offset(msgbuflen as isize).offset(-(len as isize)) as *mut ::core::ffi::c_void,
            &raw mut t as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            len,
        );
        if dirc == '?' as ::core::ffi::c_int && stat.cur == maxcount + 1 as ::core::ffi::c_int {
            stat.cur = -1 as ::core::ffi::c_int;
        }
        msg_ext_overwrite.set(true_0 != 0);
        msg_ext_set_kind(b"search_count\0".as_ptr() as *const ::core::ffi::c_char);
        give_warning(msgbuf, false_0 != 0, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn update_search_stat(
    mut dirc: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut cursor_pos: *mut pos_T,
    mut stat: *mut searchstat_T,
    mut recompute: bool,
    mut maxcount: ::core::ffi::c_int,
    mut timeout: ::core::ffi::c_int,
) {
    unsafe {
        let mut save_ws: ::core::ffi::c_int = p_ws.get();
        let mut wraparound: bool = false_0 != 0;
        let mut p: pos_T = *pos;
        static lastpos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        });
        static cur: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static exact_match: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static incomplete: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static last_maxcount: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static chgtick: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static lastpat: GlobalCell<*mut ::core::ffi::c_char> =
            GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
        static lastpatlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
        static lbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
        memset(
            stat as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<searchstat_T>(),
        );
        if dirc == 0 as ::core::ffi::c_int
            && !recompute
            && !((*lastpos.ptr()).lnum == 0 as linenr_T
                && (*lastpos.ptr()).col == 0 as ::core::ffi::c_int
                && (*lastpos.ptr()).coladd == 0 as ::core::ffi::c_int)
        {
            (*stat).cur = cur.get();
            (*stat).cnt = cnt.get();
            (*stat).exact_match = exact_match.get();
            (*stat).incomplete = incomplete.get();
            (*stat).last_maxcount = p_msc.get() as ::core::ffi::c_int;
            return;
        }
        last_maxcount.set(maxcount);
        wraparound = dirc == '?' as ::core::ffi::c_int
            && lt(lastpos.get(), p) as ::core::ffi::c_int != 0
            || dirc == '/' as ::core::ffi::c_int && lt(p, lastpos.get()) as ::core::ffi::c_int != 0;
        if !(chgtick.get() as varnumber_T == buf_get_changedtick(curbuf.get())
            && (!(*lastpat.ptr()).is_null()
                && strncmp(
                    lastpat.get(),
                    (*spats.ptr())[last_idx.get() as usize].pat,
                    lastpatlen.get(),
                ) == 0 as ::core::ffi::c_int
                && lastpatlen.get() == (*spats.ptr())[last_idx.get() as usize].patlen)
            && equalpos(lastpos.get(), *cursor_pos) as ::core::ffi::c_int != 0
            && lbuf.get() == curbuf.get())
            || wraparound as ::core::ffi::c_int != 0
            || cur.get() < 0 as ::core::ffi::c_int
            || maxcount > 0 as ::core::ffi::c_int && cur.get() > maxcount
            || recompute as ::core::ffi::c_int != 0
        {
            cur.set(0 as ::core::ffi::c_int);
            cnt.set(0 as ::core::ffi::c_int);
            exact_match.set(false_0 != 0);
            incomplete.set(0 as ::core::ffi::c_int);
            clearpos(&mut *lastpos.ptr());
            lbuf.set(curbuf.get());
        }
        if equalpos(lastpos.get(), *cursor_pos) as ::core::ffi::c_int != 0
            && !wraparound
            && (if dirc == 0 as ::core::ffi::c_int || dirc == '/' as ::core::ffi::c_int {
                (cur.get() < cnt.get()) as ::core::ffi::c_int
            } else {
                (cur.get() > 1 as ::core::ffi::c_int) as ::core::ffi::c_int
            }) != 0
        {
            (*cur.ptr()) += if dirc == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else if dirc == '/' as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else {
            let mut start: proftime_T = 0;
            let mut done_search: bool = false_0 != 0;
            let mut endpos: pos_T = pos_T {
                lnum: 0 as linenr_T,
                col: 0 as colnr_T,
                coladd: 0 as colnr_T,
            };
            p_ws.set(false_0);
            if timeout > 0 as ::core::ffi::c_int {
                start = profile_setlimit(timeout as int64_t);
            }
            while !got_int.get()
                && searchit(
                    curwin.get(),
                    curbuf.get(),
                    lastpos.ptr(),
                    &raw mut endpos,
                    FORWARD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as size_t,
                    1 as ::core::ffi::c_int,
                    SEARCH_KEEP as ::core::ffi::c_int,
                    RE_LAST as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<searchit_arg_T>(),
                ) != FAIL
            {
                done_search = true_0 != 0;
                if timeout > 0 as ::core::ffi::c_int
                    && profile_passed_limit(start) as ::core::ffi::c_int != 0
                {
                    incomplete.set(1 as ::core::ffi::c_int);
                    break;
                } else {
                    (*cnt.ptr()) += 1;
                    if ltoreq(lastpos.get(), p) {
                        cur.set(cnt.get());
                        if lt(p, endpos) {
                            exact_match.set(true_0 != 0);
                        }
                    }
                    fast_breakcheck();
                    if !(maxcount > 0 as ::core::ffi::c_int && cnt.get() > maxcount) {
                        continue;
                    }
                    incomplete.set(2 as ::core::ffi::c_int);
                    break;
                }
            }
            if got_int.get() {
                cur.set(-1 as ::core::ffi::c_int);
            }
            if done_search {
                xfree(lastpat.get() as *mut ::core::ffi::c_void);
                lastpat.set(xstrnsave(
                    (*spats.ptr())[last_idx.get() as usize].pat,
                    (*spats.ptr())[last_idx.get() as usize].patlen,
                ));
                lastpatlen.set((*spats.ptr())[last_idx.get() as usize].patlen);
                chgtick.set(buf_get_changedtick(curbuf.get()) as ::core::ffi::c_int);
                lbuf.set(curbuf.get());
                lastpos.set(p);
            }
        }
        (*stat).cur = cur.get();
        (*stat).cnt = cnt.get();
        (*stat).exact_match = exact_match.get();
        (*stat).incomplete = incomplete.get();
        (*stat).last_maxcount = last_maxcount.get();
        p_ws.set(save_ws);
    }
}

pub unsafe extern "C" fn f_searchcount(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut maxcount: ::core::ffi::c_int = p_msc.get() as ::core::ffi::c_int;
        let mut timeout: ::core::ffi::c_int = SEARCH_STAT_DEF_TIMEOUT as ::core::ffi::c_int;
        let mut recompute: bool = true_0 != 0;
        let mut stat: searchstat_T = searchstat_T {
            cur: 0,
            cnt: 0,
            exact_match: false,
            incomplete: 0,
            last_maxcount: 0,
        };
        tv_dict_alloc_ret(rettv);
        if shortmess(SHM_SEARCHCOUNT as ::core::ffi::c_int) {
            recompute = true_0 != 0;
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
            let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            let mut error: bool = false_0 != 0;
            if tv_check_for_nonnull_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
                return;
            }
            dict = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            di = tv_dict_find(
                dict,
                b"timeout\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                timeout =
                    tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) as ::core::ffi::c_int;
                if error {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"maxcount\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                maxcount =
                    tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) as ::core::ffi::c_int;
                if error {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"recompute\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                recompute = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) != 0;
                if error {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                pattern = tv_get_string_chk(&raw mut (*di).di_tv) as *mut ::core::ffi::c_char;
                if pattern.is_null() {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"pos\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                if (*di).di_tv.v_type as ::core::ffi::c_uint
                    != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        b"pos\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return;
                }
                if tv_list_len((*di).di_tv.vval.v_list) != 3 as ::core::ffi::c_int {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        b"List format should be [lnum, col, off]\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    return;
                }
                let mut li: *mut listitem_T =
                    tv_list_find((*di).di_tv.vval.v_list, 0 as ::core::ffi::c_int);
                if !li.is_null() {
                    pos.lnum = tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error) as linenr_T;
                    if error {
                        return;
                    }
                }
                li = tv_list_find((*di).di_tv.vval.v_list, 1 as ::core::ffi::c_int);
                if !li.is_null() {
                    pos.col = (tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error)
                        as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int) as colnr_T;
                    if error {
                        return;
                    }
                }
                li = tv_list_find((*di).di_tv.vval.v_list, 2 as ::core::ffi::c_int);
                if !li.is_null() {
                    pos.coladd = tv_get_number_chk(&raw mut (*li).li_tv, &raw mut error) as colnr_T;
                    if error {
                        return;
                    }
                }
            }
        }
        save_last_search_pattern();
        save_incsearch_state();
        '_the_end: {
            if !pattern.is_null() {
                if *pattern as ::core::ffi::c_int == NUL {
                    break '_the_end;
                } else {
                    xfree((*spats.ptr())[last_idx.get() as usize].pat as *mut ::core::ffi::c_void);
                    (*spats.ptr())[last_idx.get() as usize].patlen = strlen(pattern);
                    (*spats.ptr())[last_idx.get() as usize].pat =
                        xstrnsave(pattern, (*spats.ptr())[last_idx.get() as usize].patlen);
                }
            }
            if !((*spats.ptr())[last_idx.get() as usize].pat.is_null()
                || *(*spats.ptr())[last_idx.get() as usize].pat as ::core::ffi::c_int == NUL)
            {
                update_search_stat(
                    0 as ::core::ffi::c_int,
                    &raw mut pos,
                    &raw mut pos,
                    &raw mut stat,
                    recompute,
                    maxcount,
                    timeout,
                );
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    b"current\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    stat.cur as varnumber_T,
                );
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    b"total\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    stat.cnt as varnumber_T,
                );
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    b"exact_match\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                    stat.exact_match as varnumber_T,
                );
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    b"incomplete\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                    stat.incomplete as varnumber_T,
                );
                tv_dict_add_nr(
                    (*rettv).vval.v_dict,
                    b"maxcount\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    stat.last_maxcount as varnumber_T,
                );
            }
        }
        restore_last_search_pattern();
        restore_incsearch_state();
    }
}
