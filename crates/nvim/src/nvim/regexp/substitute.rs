//! Building the replacement text: `~` expansion and the `vim_regsub`
//! family that expands `\\1`, `\\u` and a `\\=` expression.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;
use crate::src::nvim::mbyte::{mb_tolower, mb_toupper};
use core::ffi::c_int;

pub unsafe extern "C" fn regtilde(
    mut source: *mut ::core::ffi::c_char,
    mut magic: ::core::ffi::c_int,
    mut preview: bool,
) -> *mut ::core::ffi::c_char {
    let mut newsub: *mut ::core::ffi::c_char = source;
    let mut newsublen: size_t = 0 as size_t;
    let mut tilde: [::core::ffi::c_char; 3] = [
        '~' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ];
    let mut tildelen: size_t = 1 as size_t;
    let mut error: bool = false_0 != 0;
    if magic == 0 {
        tilde[0 as ::core::ffi::c_int as usize] = '\\' as ::core::ffi::c_char;
        tilde[1 as ::core::ffi::c_int as usize] = '~' as ::core::ffi::c_char;
        tilde[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        tildelen = 2 as size_t;
    }
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = newsub;
    while *p != 0 {
        if strncmp(p, &raw mut tilde as *mut ::core::ffi::c_char, tildelen)
            == 0 as ::core::ffi::c_int
        {
            let mut prefixlen: size_t = p.offset_from(newsub) as size_t;
            let mut postfix: *mut ::core::ffi::c_char = p.offset(tildelen as isize);
            let mut postfixlen: size_t = 0;
            let mut tmpsublen: size_t = 0;
            if newsublen == 0 as size_t {
                newsublen = strlen(newsub);
            }
            newsublen = newsublen.wrapping_sub(tildelen);
            postfixlen = newsublen.wrapping_sub(prefixlen);
            tmpsublen = prefixlen
                .wrapping_add(reg_prev_sublen.get())
                .wrapping_add(postfixlen);
            if tmpsublen > 0 as size_t && !(*reg_prev_sub.ptr()).is_null() {
                if tmpsublen > MAXCOL as ::core::ffi::c_int as size_t {
                    emsg(gettext(
                        &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
                    ));
                    error = true_0 != 0;
                    break;
                } else {
                    let mut tmpsub: *mut ::core::ffi::c_char =
                        xmalloc(tmpsublen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
                    memmove(
                        tmpsub as *mut ::core::ffi::c_void,
                        newsub as *const ::core::ffi::c_void,
                        prefixlen,
                    );
                    memmove(
                        tmpsub.offset(prefixlen as isize) as *mut ::core::ffi::c_void,
                        reg_prev_sub.get() as *const ::core::ffi::c_void,
                        reg_prev_sublen.get(),
                    );
                    strcpy(
                        tmpsub
                            .offset(prefixlen as isize)
                            .offset(reg_prev_sublen.get() as isize),
                        postfix,
                    );
                    if newsub != source {
                        xfree(newsub as *mut ::core::ffi::c_void);
                    }
                    newsub = tmpsub;
                    newsublen = tmpsublen;
                    p = newsub
                        .offset(prefixlen as isize)
                        .offset(reg_prev_sublen.get() as isize);
                }
            } else {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    postfix as *const ::core::ffi::c_void,
                    postfixlen.wrapping_add(1 as size_t),
                );
            }
            p = p.offset(-1);
        } else {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            {
                p = p.offset(1);
            }
            p = p.offset((utfc_ptr2len(p) - 1 as ::core::ffi::c_int) as isize);
        }
        p = p.offset(1);
    }
    if error {
        if newsub != source {
            xfree(newsub as *mut ::core::ffi::c_void);
        }
        return source;
    }
    if !preview {
        newsublen = p.offset_from(newsub) as size_t;
        if newsublen == 0 as size_t {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                reg_prev_sub.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            xfree(reg_prev_sub.get() as *mut ::core::ffi::c_void);
            reg_prev_sub.set(xstrnsave(newsub, newsublen));
        }
        reg_prev_sublen.set(newsublen);
    }
    return newsub;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regsub(
    mut rmp: *mut regmatch_T,
    mut source: *mut ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut dest: *mut ::core::ffi::c_char,
    mut destlen: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rex_save: regexec_T = regexec_T {
        reg_match: ::core::ptr::null_mut::<regmatch_T>(),
        reg_mmatch: ::core::ptr::null_mut::<regmmatch_T>(),
        reg_startp: ::core::ptr::null_mut::<*mut uint8_t>(),
        reg_endp: ::core::ptr::null_mut::<*mut uint8_t>(),
        reg_startpos: ::core::ptr::null_mut::<lpos_T>(),
        reg_endpos: ::core::ptr::null_mut::<lpos_T>(),
        reg_win: ::core::ptr::null_mut::<win_T>(),
        reg_buf: ::core::ptr::null_mut::<buf_T>(),
        reg_firstlnum: 0,
        reg_maxline: 0,
        reg_line_lbr: false,
        lnum: 0,
        line: ::core::ptr::null_mut::<uint8_t>(),
        input: ::core::ptr::null_mut::<uint8_t>(),
        need_clear_subexpr: 0,
        need_clear_zsubexpr: 0,
        reg_ic: false,
        reg_icombine: false,
        reg_nobreak: false,
        reg_maxcol: 0,
        nfa_has_zend: 0,
        nfa_has_backref: 0,
        nfa_nsubexpr: 0,
        nfa_listid: 0,
        nfa_alt_listid: 0,
        nfa_has_zsubexpr: 0,
    };
    let mut rex_in_use_save: bool = rex_in_use.get();
    if rex_in_use.get() {
        rex_save = rex.get();
    }
    rex_in_use.set(true_0 != 0);
    (*rex.ptr()).reg_match = rmp;
    (*rex.ptr()).reg_mmatch = ::core::ptr::null_mut::<regmmatch_T>();
    (*rex.ptr()).reg_maxline = 0 as ::core::ffi::c_int as linenr_T;
    (*rex.ptr()).reg_buf = curbuf.get();
    (*rex.ptr()).reg_line_lbr = true_0 != 0;
    let mut result: ::core::ffi::c_int = vim_regsub_both(source, expr, dest, destlen, flags);
    rex_in_use.set(rex_in_use_save);
    if rex_in_use.get() {
        rex.set(rex_save);
    }
    return result;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regsub_multi(
    mut rmp: *mut regmmatch_T,
    mut lnum: linenr_T,
    mut source: *mut ::core::ffi::c_char,
    mut dest: *mut ::core::ffi::c_char,
    mut destlen: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rex_save: regexec_T = regexec_T {
        reg_match: ::core::ptr::null_mut::<regmatch_T>(),
        reg_mmatch: ::core::ptr::null_mut::<regmmatch_T>(),
        reg_startp: ::core::ptr::null_mut::<*mut uint8_t>(),
        reg_endp: ::core::ptr::null_mut::<*mut uint8_t>(),
        reg_startpos: ::core::ptr::null_mut::<lpos_T>(),
        reg_endpos: ::core::ptr::null_mut::<lpos_T>(),
        reg_win: ::core::ptr::null_mut::<win_T>(),
        reg_buf: ::core::ptr::null_mut::<buf_T>(),
        reg_firstlnum: 0,
        reg_maxline: 0,
        reg_line_lbr: false,
        lnum: 0,
        line: ::core::ptr::null_mut::<uint8_t>(),
        input: ::core::ptr::null_mut::<uint8_t>(),
        need_clear_subexpr: 0,
        need_clear_zsubexpr: 0,
        reg_ic: false,
        reg_icombine: false,
        reg_nobreak: false,
        reg_maxcol: 0,
        nfa_has_zend: 0,
        nfa_has_backref: 0,
        nfa_nsubexpr: 0,
        nfa_listid: 0,
        nfa_alt_listid: 0,
        nfa_has_zsubexpr: 0,
    };
    let mut rex_in_use_save: bool = rex_in_use.get();
    if rex_in_use.get() {
        rex_save = rex.get();
    }
    rex_in_use.set(true_0 != 0);
    (*rex.ptr()).reg_match = ::core::ptr::null_mut::<regmatch_T>();
    (*rex.ptr()).reg_mmatch = rmp;
    (*rex.ptr()).reg_buf = curbuf.get();
    (*rex.ptr()).reg_firstlnum = lnum;
    (*rex.ptr()).reg_maxline = (*curbuf.get()).b_ml.ml_line_count - lnum;
    (*rex.ptr()).reg_line_lbr = false_0 != 0;
    let mut result: ::core::ffi::c_int = vim_regsub_both(
        source,
        ::core::ptr::null_mut::<typval_T>(),
        dest,
        destlen,
        flags,
    );
    rex_in_use.set(rex_in_use_save);
    if rex_in_use.get() {
        rex.set(rex_save);
    }
    return result;
}
pub(crate) unsafe extern "C" fn vim_regsub_both(
    mut source: *mut ::core::ffi::c_char,
    mut expr: *mut typval_T,
    mut dest: *mut ::core::ffi::c_char,
    mut destlen: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut src: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dst: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c: ::core::ffi::c_int = 0;
    let mut cc: ::core::ffi::c_int = 0;
    let mut no: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut func_all: Option<CaseFolder> = None;
    let mut func_one: Option<CaseFolder> = None;
    let mut clnum: linenr_T = 0 as linenr_T;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    static nesting: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut copy: bool = flags & REGSUB_COPY as ::core::ffi::c_int != 0;
    if source.is_null() && expr.is_null() || dest.is_null() {
        emsg(gettext(&raw const e_null as *const ::core::ffi::c_char));
        return 0 as ::core::ffi::c_int;
    }
    if prog_magic_wrong() != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if nesting.get() == MAX_REGSUB_NESTING {
        emsg(gettext(E_SUBSTITUTE_NESTING_TOO_DEEP.as_ptr()));
        return 0 as ::core::ffi::c_int;
    }
    let mut nested: ::core::ffi::c_int = nesting.get();
    src = source;
    dst = dest;
    '_exit: {
        's_1155: {
            if !expr.is_null()
                || *source.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *source.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int
            {
                if copy {
                    if !(*eval_result.ptr())[nested as usize].is_null() {
                        let mut eval_len: size_t = strlen((*eval_result.ptr())[nested as usize]);
                        if eval_len < destlen as size_t {
                            strcpy(dest, (*eval_result.ptr())[nested as usize]);
                            dst = dst.offset(eval_len as isize);
                            let mut ptr_: *mut *mut ::core::ffi::c_void = (eval_result.ptr()
                                as *mut *mut ::core::ffi::c_char)
                                .offset(nested as isize)
                                as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL_0;
                            let _ = *ptr_;
                        }
                    }
                } else {
                    let prev_can_f_submatch: bool = can_f_submatch.get();
                    let mut rsm_save: regsubmatch_T = regsubmatch_T {
                        sm_match: ::core::ptr::null_mut::<regmatch_T>(),
                        sm_mmatch: ::core::ptr::null_mut::<regmmatch_T>(),
                        sm_firstlnum: 0,
                        sm_maxline: 0,
                        sm_line_lbr: 0,
                    };
                    let mut ptr__0: *mut *mut ::core::ffi::c_void = (eval_result.ptr()
                        as *mut *mut ::core::ffi::c_char)
                        .offset(nested as isize)
                        as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL_0;
                    let _ = *ptr__0;
                    if can_f_submatch.get() {
                        rsm_save = rsm.get();
                    }
                    can_f_submatch.set(true_0 != 0);
                    (*rsm.ptr()).sm_match = (*rex.ptr()).reg_match;
                    (*rsm.ptr()).sm_mmatch = (*rex.ptr()).reg_mmatch;
                    (*rsm.ptr()).sm_firstlnum = (*rex.ptr()).reg_firstlnum;
                    (*rsm.ptr()).sm_maxline = (*rex.ptr()).reg_maxline;
                    (*rsm.ptr()).sm_line_lbr = (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int;
                    (*nesting.ptr()) += 1;
                    if !expr.is_null() {
                        let mut argv: [typval_T; 2] = [typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        }; 2];
                        let mut rettv: typval_T = typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        };
                        let mut matchList: staticList10_T = staticList10_T {
                            sl_list: listvar_S {
                                lv_first: ::core::ptr::null_mut::<listitem_T>(),
                                lv_last: ::core::ptr::null_mut::<listitem_T>(),
                                lv_watch: ::core::ptr::null_mut::<listwatch_T>(),
                                lv_idx_item: ::core::ptr::null_mut::<listitem_T>(),
                                lv_copylist: ::core::ptr::null_mut::<list_T>(),
                                lv_used_next: ::core::ptr::null_mut::<list_T>(),
                                lv_used_prev: ::core::ptr::null_mut::<list_T>(),
                                lv_refcount: 0 as ::core::ffi::c_int,
                                lv_len: 0 as ::core::ffi::c_int,
                                lv_idx: 0,
                                lv_copyID: 0,
                                lv_lock: VAR_FIXED,
                                lua_table_ref: 0,
                            },
                            sl_items: [listitem_T {
                                li_next: ::core::ptr::null_mut::<listitem_T>(),
                                li_prev: ::core::ptr::null_mut::<listitem_T>(),
                                li_tv: typval_T {
                                    v_type: VAR_UNKNOWN,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_number: 0 },
                                },
                            }; 10],
                        };
                        rettv.v_type = VAR_STRING;
                        rettv.vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_LIST;
                        argv[0 as ::core::ffi::c_int as usize].vval.v_list =
                            &raw mut matchList.sl_list;
                        let mut funcexe: funcexe_T = FUNCEXE_INIT;
                        funcexe.fe_argv_func = Some(
                            fill_submatch_list
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    *mut typval_T,
                                    ::core::ffi::c_int,
                                    *mut ufunc_T,
                                )
                                    -> ::core::ffi::c_int,
                        ) as ArgvFunc;
                        funcexe.fe_evaluate = true_0 != 0;
                        if (*expr).v_type as ::core::ffi::c_uint
                            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            s = (*expr).vval.v_string;
                            call_func(
                                s,
                                -1 as ::core::ffi::c_int,
                                &raw mut rettv,
                                1 as ::core::ffi::c_int,
                                &raw mut argv as *mut typval_T,
                                &raw mut funcexe,
                            );
                        } else if (*expr).v_type as ::core::ffi::c_uint
                            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let mut partial: *mut partial_T = (*expr).vval.v_partial;
                            s = partial_name(partial);
                            funcexe.fe_partial = partial;
                            call_func(
                                s,
                                -1 as ::core::ffi::c_int,
                                &raw mut rettv,
                                1 as ::core::ffi::c_int,
                                &raw mut argv as *mut typval_T,
                                &raw mut funcexe,
                            );
                        }
                        if tv_list_len(&raw mut matchList.sl_list) > 0 as ::core::ffi::c_int {
                            clear_submatch_list(&raw mut matchList);
                        }
                        if rettv.v_type as ::core::ffi::c_uint
                            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            (*eval_result.ptr())[nested as usize] =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                        } else {
                            let mut buf: [::core::ffi::c_char; 65] = [0; 65];
                            (*eval_result.ptr())[nested as usize] = tv_get_string_buf_chk(
                                &raw mut rettv,
                                &raw mut buf as *mut ::core::ffi::c_char,
                            )
                                as *mut ::core::ffi::c_char;
                            if !(*eval_result.ptr())[nested as usize].is_null() {
                                (*eval_result.ptr())[nested as usize] =
                                    xstrdup((*eval_result.ptr())[nested as usize]);
                            }
                        }
                        tv_clear(&raw mut rettv);
                    } else {
                        (*eval_result.ptr())[nested as usize] = eval_to_string(
                            source.offset(2 as ::core::ffi::c_int as isize),
                            true_0 != 0,
                            false_0 != 0,
                        );
                    }
                    (*nesting.ptr()) -= 1;
                    if !(*eval_result.ptr())[nested as usize].is_null() {
                        let mut had_backslash: ::core::ffi::c_int = false_0;
                        s = (*eval_result.ptr())[nested as usize];
                        while *s as ::core::ffi::c_int != NUL {
                            if *s as ::core::ffi::c_int == NL && (*rsm.ptr()).sm_line_lbr == 0 {
                                *s = CAR as ::core::ffi::c_char;
                            } else if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    != NUL
                            {
                                s = s.offset(1);
                                if *s as ::core::ffi::c_int == NL && (*rsm.ptr()).sm_line_lbr == 0 {
                                    *s = CAR as ::core::ffi::c_char;
                                }
                                had_backslash = true_0;
                            }
                            s = s.offset(utfc_ptr2len(s) as isize);
                        }
                        if had_backslash != 0 && flags & REGSUB_BACKSLASH as ::core::ffi::c_int != 0
                        {
                            s = vim_strsave_escaped(
                                (*eval_result.ptr())[nested as usize],
                                b"\\\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            xfree(
                                (*eval_result.ptr())[nested as usize] as *mut ::core::ffi::c_void,
                            );
                            (*eval_result.ptr())[nested as usize] = s;
                        }
                        dst = dst.offset(strlen((*eval_result.ptr())[nested as usize]) as isize);
                    }
                    can_f_submatch.set(prev_can_f_submatch);
                    if can_f_submatch.get() {
                        rsm.set(rsm_save);
                    }
                }
            } else {
                loop {
                    let c2rust_fresh0 = src;
                    src = src.offset(1);
                    c = *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
                    if c == NUL {
                        break 's_1155;
                    }
                    if c == '&' as ::core::ffi::c_int
                        && flags & REGSUB_MAGIC as ::core::ffi::c_int != 0
                    {
                        no = 0 as ::core::ffi::c_int;
                    } else if c == '\\' as ::core::ffi::c_int && *src as ::core::ffi::c_int != NUL {
                        if *src as ::core::ffi::c_int == '&' as ::core::ffi::c_int
                            && flags & REGSUB_MAGIC as ::core::ffi::c_int == 0
                        {
                            src = src.offset(1);
                            no = 0 as ::core::ffi::c_int;
                        } else if '0' as ::core::ffi::c_int <= *src as ::core::ffi::c_int
                            && *src as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
                        {
                            let c2rust_fresh1 = src;
                            src = src.offset(1);
                            no = *c2rust_fresh1 as ::core::ffi::c_int - '0' as ::core::ffi::c_int;
                        } else if !vim_strchr(
                            b"uUlLeE\0".as_ptr() as *const ::core::ffi::c_char,
                            *src as uint8_t as ::core::ffi::c_int,
                        )
                        .is_null()
                        {
                            let c2rust_fresh2 = src;
                            src = src.offset(1);
                            match *c2rust_fresh2 as ::core::ffi::c_int {
                                117 => {
                                    func_one = Some(to_upper);
                                    continue;
                                }
                                85 => {
                                    func_all = Some(to_upper);
                                    continue;
                                }
                                108 => {
                                    func_one = Some(to_lower);
                                    continue;
                                }
                                76 => {
                                    func_all = Some(to_lower);
                                    continue;
                                }
                                101 | 69 => {
                                    func_all = None;
                                    func_one = None;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    if no < 0 as ::core::ffi::c_int {
                        if c == K_SPECIAL
                            && *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != NUL
                            && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != NUL
                        {
                            if copy {
                                if dst.offset(3 as ::core::ffi::c_int as isize)
                                    > dest.offset(destlen as isize)
                                {
                                    iemsg(b"vim_regsub_both(): not enough space\0".as_ptr()
                                        as *const ::core::ffi::c_char);
                                    return 0 as ::core::ffi::c_int;
                                }
                                let c2rust_fresh3 = dst;
                                dst = dst.offset(1);
                                *c2rust_fresh3 = c as ::core::ffi::c_char;
                                let c2rust_fresh4 = src;
                                src = src.offset(1);
                                let c2rust_fresh5 = dst;
                                dst = dst.offset(1);
                                *c2rust_fresh5 = *c2rust_fresh4;
                                let c2rust_fresh6 = src;
                                src = src.offset(1);
                                let c2rust_fresh7 = dst;
                                dst = dst.offset(1);
                                *c2rust_fresh7 = *c2rust_fresh6;
                            } else {
                                dst = dst.offset(3 as ::core::ffi::c_int as isize);
                                src = src.offset(2 as ::core::ffi::c_int as isize);
                            }
                        } else {
                            if c == '\\' as ::core::ffi::c_int && *src as ::core::ffi::c_int != NUL
                            {
                                match *src as ::core::ffi::c_int {
                                    114 => {
                                        c = CAR;
                                        src = src.offset(1);
                                    }
                                    110 => {
                                        c = NL;
                                        src = src.offset(1);
                                    }
                                    116 => {
                                        c = TAB;
                                        src = src.offset(1);
                                    }
                                    98 => {
                                        c = Ctrl_H;
                                        src = src.offset(1);
                                    }
                                    _ => {
                                        if flags & REGSUB_BACKSLASH as ::core::ffi::c_int != 0 {
                                            if copy {
                                                if dst.offset(1 as ::core::ffi::c_int as isize)
                                                    > dest.offset(destlen as isize)
                                                {
                                                    iemsg(
                                                        b"vim_regsub_both(): not enough space\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                    return 0 as ::core::ffi::c_int;
                                                }
                                                *dst = '\\' as ::core::ffi::c_char;
                                            }
                                            dst = dst.offset(1);
                                        }
                                        let c2rust_fresh8 = src;
                                        src = src.offset(1);
                                        c = *c2rust_fresh8 as uint8_t as ::core::ffi::c_int;
                                    }
                                }
                            } else {
                                c = utf_ptr2char(src.offset(-(1 as ::core::ffi::c_int as isize)));
                            }
                            cc = match func_one.take().or(func_all) {
                                Some(fold) => fold(c),
                                None => c,
                            };
                            let mut totlen: ::core::ffi::c_int =
                                utfc_ptr2len(src.offset(-(1 as ::core::ffi::c_int as isize)));
                            let mut charlen: ::core::ffi::c_int = utf_char2len(cc);
                            if copy {
                                if dst.offset(charlen as isize) > dest.offset(destlen as isize) {
                                    iemsg(b"vim_regsub_both(): not enough space\0".as_ptr()
                                        as *const ::core::ffi::c_char);
                                    return 0 as ::core::ffi::c_int;
                                }
                                utf_char2bytes(cc, dst);
                            }
                            dst = dst.offset((charlen - 1 as ::core::ffi::c_int) as isize);
                            let mut clen: ::core::ffi::c_int =
                                utf_ptr2len(src.offset(-(1 as ::core::ffi::c_int as isize)));
                            if clen < totlen {
                                if copy {
                                    if dst.offset(totlen as isize).offset(-(clen as isize))
                                        > dest.offset(destlen as isize)
                                    {
                                        iemsg(b"vim_regsub_both(): not enough space\0".as_ptr()
                                            as *const ::core::ffi::c_char);
                                        return 0 as ::core::ffi::c_int;
                                    }
                                    memmove(
                                        dst.offset(1 as ::core::ffi::c_int as isize)
                                            as *mut ::core::ffi::c_void,
                                        src.offset(-(1 as ::core::ffi::c_int as isize))
                                            .offset(clen as isize)
                                            as *const ::core::ffi::c_void,
                                        (totlen - clen) as size_t,
                                    );
                                }
                                dst = dst.offset((totlen - clen) as isize);
                            }
                            src = src.offset((totlen - 1 as ::core::ffi::c_int) as isize);
                            dst = dst.offset(1);
                        }
                    } else {
                        if (*rex.ptr()).reg_match.is_null() {
                            clnum = (*(*rex.ptr()).reg_mmatch).startpos[no as usize].lnum;
                            if clnum < 0 as linenr_T
                                || (*(*rex.ptr()).reg_mmatch).endpos[no as usize].lnum
                                    < 0 as linenr_T
                            {
                                s = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            } else {
                                s = reg_getline(clnum).offset(
                                    (*(*rex.ptr()).reg_mmatch).startpos[no as usize].col as isize,
                                );
                                if (*(*rex.ptr()).reg_mmatch).endpos[no as usize].lnum == clnum {
                                    len = ((*(*rex.ptr()).reg_mmatch).endpos[no as usize].col
                                        - (*(*rex.ptr()).reg_mmatch).startpos[no as usize].col)
                                        as ::core::ffi::c_int;
                                } else {
                                    len = (reg_getline_len(clnum)
                                        - (*(*rex.ptr()).reg_mmatch).startpos[no as usize].col)
                                        as ::core::ffi::c_int;
                                }
                            }
                        } else {
                            s = (*(*rex.ptr()).reg_match).startp[no as usize];
                            if (*(*rex.ptr()).reg_match).endp[no as usize].is_null() {
                                s = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            } else {
                                len = (*(*rex.ptr()).reg_match).endp[no as usize].offset_from(s)
                                    as ::core::ffi::c_int;
                            }
                        }
                        's_1140: {
                            if !s.is_null() {
                                loop {
                                    if len == 0 as ::core::ffi::c_int {
                                        if !(*rex.ptr()).reg_match.is_null() {
                                            break 's_1140;
                                        }
                                        if (*(*rex.ptr()).reg_mmatch).endpos[no as usize].lnum
                                            == clnum
                                        {
                                            break 's_1140;
                                        }
                                        if copy {
                                            if dst.offset(1 as ::core::ffi::c_int as isize)
                                                > dest.offset(destlen as isize)
                                            {
                                                iemsg(
                                                    b"vim_regsub_both(): not enough space\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            *dst = CAR as ::core::ffi::c_char;
                                        }
                                        dst = dst.offset(1);
                                        clnum += 1;
                                        s = reg_getline(clnum);
                                        if (*(*rex.ptr()).reg_mmatch).endpos[no as usize].lnum
                                            == clnum
                                        {
                                            len = (*(*rex.ptr()).reg_mmatch).endpos[no as usize].col
                                                as ::core::ffi::c_int;
                                        } else {
                                            len = reg_getline_len(clnum) as ::core::ffi::c_int;
                                        }
                                    } else if *s as ::core::ffi::c_int == NUL {
                                        if copy {
                                            iemsg(gettext(
                                                &raw const e_re_damg as *const ::core::ffi::c_char,
                                            ));
                                        }
                                        break '_exit;
                                    } else {
                                        if flags & REGSUB_BACKSLASH as ::core::ffi::c_int != 0
                                            && (*s as ::core::ffi::c_int == CAR
                                                || *s as ::core::ffi::c_int
                                                    == '\\' as ::core::ffi::c_int)
                                        {
                                            if copy {
                                                if dst.offset(2 as ::core::ffi::c_int as isize)
                                                    > dest.offset(destlen as isize)
                                                {
                                                    iemsg(
                                                        b"vim_regsub_both(): not enough space\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                    return 0 as ::core::ffi::c_int;
                                                }
                                                *dst.offset(0 as ::core::ffi::c_int as isize) =
                                                    '\\' as ::core::ffi::c_char;
                                                *dst.offset(1 as ::core::ffi::c_int as isize) = *s;
                                            }
                                            dst = dst.offset(2 as ::core::ffi::c_int as isize);
                                        } else {
                                            c = utf_ptr2char(s);
                                            cc = match func_one.take().or(func_all) {
                                                Some(fold) => fold(c),
                                                None => c,
                                            };
                                            let mut l: ::core::ffi::c_int = 0;
                                            let mut charlen_0: ::core::ffi::c_int = 0;
                                            l = utf_ptr2len(s) - 1 as ::core::ffi::c_int;
                                            s = s.offset(l as isize);
                                            len -= l;
                                            charlen_0 = utf_char2len(cc);
                                            if copy {
                                                if dst.offset(charlen_0 as isize)
                                                    > dest.offset(destlen as isize)
                                                {
                                                    iemsg(
                                                        b"vim_regsub_both(): not enough space\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                    return 0 as ::core::ffi::c_int;
                                                }
                                                utf_char2bytes(cc, dst);
                                            }
                                            dst = dst.offset(
                                                (charlen_0 - 1 as ::core::ffi::c_int) as isize,
                                            );
                                            dst = dst.offset(1);
                                        }
                                        s = s.offset(1);
                                        len -= 1;
                                    }
                                }
                            }
                        }
                        no = -1 as ::core::ffi::c_int;
                    }
                }
            }
        }
        if copy {
            *dst = NUL as ::core::ffi::c_char;
        }
    }
    return (dst.offset_from(dest) + 1 as isize) as ::core::ffi::c_int;
}

/// The `\u`/`\U` and `\l`/`\L` case hooks a `:substitute` replacement can
/// install. See [`CaseFolder`].
pub(crate) fn to_upper(c: c_int) -> c_int {
    // SAFETY: `mb_toupper` is a pure table lookup over a code point.
    unsafe { mb_toupper(c) }
}

pub(crate) fn to_lower(c: c_int) -> c_int {
    // SAFETY: `mb_tolower` is a pure table lookup over a code point.
    unsafe { mb_tolower(c) }
}
