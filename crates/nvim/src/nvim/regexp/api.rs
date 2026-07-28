//! The public entry points: compiling a pattern with the engine `re`
//! and the pattern itself select, and running it over a string or buffer.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regcomp(
    mut expr_arg: *const ::core::ffi::c_char,
    mut re_flags: ::core::ffi::c_int,
) -> *mut regprog_T {
    let mut prog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut expr: *const ::core::ffi::c_char = expr_arg;
    regexp_engine.set(p_re.get() as ::core::ffi::c_int);
    if strncmp(
        expr,
        b"\\%#=\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        let mut newengine: ::core::ffi::c_int = *expr.offset(4 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            - '0' as ::core::ffi::c_int;
        if newengine == AUTOMATIC_ENGINE as ::core::ffi::c_int
            || newengine == BACKTRACKING_ENGINE as ::core::ffi::c_int
            || newengine == NFA_ENGINE as ::core::ffi::c_int
        {
            regexp_engine.set(
                *expr.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    - '0' as ::core::ffi::c_int,
            );
            expr = expr.offset(5 as ::core::ffi::c_int as isize);
        } else {
            emsg(
                gettext(
                    b"E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used \0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
            );
            regexp_engine.set(AUTOMATIC_ENGINE as ::core::ffi::c_int);
        }
    }
    (*rex.ptr()).reg_buf = curbuf.get();
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    if regexp_engine.get() != BACKTRACKING_ENGINE as ::core::ffi::c_int {
        prog = (*nfa_regengine.ptr())
            .regcomp
            .expect("non-null function pointer")(
            expr as *mut uint8_t,
            re_flags
                + (if regexp_engine.get() == AUTOMATIC_ENGINE as ::core::ffi::c_int {
                    RE_AUTO
                } else {
                    0 as ::core::ffi::c_int
                }),
        );
    } else {
        prog = (*bt_regengine.ptr())
            .regcomp
            .expect("non-null function pointer")(expr as *mut uint8_t, re_flags);
    }
    if prog.is_null() {
        if regexp_engine.get() == AUTOMATIC_ENGINE as ::core::ffi::c_int
            && called_emsg.get() == called_emsg_before
        {
            regexp_engine.set(BACKTRACKING_ENGINE as ::core::ffi::c_int);
            report_re_switch(expr);
            prog = (*bt_regengine.ptr())
                .regcomp
                .expect("non-null function pointer")(
                expr as *mut uint8_t, re_flags
            );
        }
    }
    if !prog.is_null() {
        (*prog).re_engine = regexp_engine.get() as ::core::ffi::c_uint;
        (*prog).re_flags = re_flags as ::core::ffi::c_uint;
    }
    return prog;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regfree(mut prog: *mut regprog_T) {
    if !prog.is_null() {
        (*(*prog).engine)
            .regfree
            .expect("non-null function pointer")(prog);
    }
}
pub(crate) unsafe extern "C" fn report_re_switch(mut pat: *const ::core::ffi::c_char) {
    if p_verbose.get() > 0 as OptInt {
        verbose_enter();
        msg_puts(gettext(
            b"Switching to backtracking RE engine for pattern: \0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        msg_puts(pat);
        verbose_leave();
    }
}
pub(crate) unsafe extern "C" fn vim_regexec_string(
    mut rmp: *mut regmatch_T,
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
    mut nl: bool,
) -> bool {
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
    if (*(*rmp).regprog).re_in_use {
        emsg(gettext(
            (e_recursive.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    (*(*rmp).regprog).re_in_use = true_0 != 0;
    if rex_in_use.get() {
        rex_save = rex.get();
    }
    rex_in_use.set(true_0 != 0);
    (*rex.ptr()).reg_startp = ::core::ptr::null_mut::<*mut uint8_t>();
    (*rex.ptr()).reg_endp = ::core::ptr::null_mut::<*mut uint8_t>();
    (*rex.ptr()).reg_startpos = ::core::ptr::null_mut::<lpos_T>();
    (*rex.ptr()).reg_endpos = ::core::ptr::null_mut::<lpos_T>();
    let mut result: ::core::ffi::c_int =
        (*(*(*rmp).regprog).engine)
            .regexec_nl
            .expect("non-null function pointer")(rmp, line as *mut uint8_t, col, nl);
    (*(*rmp).regprog).re_in_use = false_0 != 0;
    if (*(*rmp).regprog).re_engine == AUTOMATIC_ENGINE as ::core::ffi::c_int as ::core::ffi::c_uint
        && result == NFA_TOO_EXPENSIVE as ::core::ffi::c_int
    {
        let mut save_p_re: ::core::ffi::c_int = p_re.get() as ::core::ffi::c_int;
        let mut re_flags: ::core::ffi::c_int = (*(*rmp).regprog).re_flags as ::core::ffi::c_int;
        let mut pat: *mut ::core::ffi::c_char =
            xstrdup((*((*rmp).regprog as *mut nfa_regprog_T)).pattern);
        p_re.set(BACKTRACKING_ENGINE as ::core::ffi::c_int as OptInt);
        vim_regfree((*rmp).regprog);
        report_re_switch(pat);
        (*rmp).regprog = vim_regcomp(pat, re_flags);
        if !(*rmp).regprog.is_null() {
            (*(*rmp).regprog).re_in_use = true_0 != 0;
            result = (*(*(*rmp).regprog).engine)
                .regexec_nl
                .expect("non-null function pointer")(
                rmp, line as *mut uint8_t, col, nl
            );
            (*(*rmp).regprog).re_in_use = false_0 != 0;
        }
        xfree(pat as *mut ::core::ffi::c_void);
        p_re.set(save_p_re as OptInt);
    }
    rex_in_use.set(rex_in_use_save);
    if rex_in_use.get() {
        rex.set(rex_save);
    }
    return result > 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regexec_prog(
    mut prog: *mut *mut regprog_T,
    mut ignore_case: bool,
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> bool {
    let mut regmatch_0: regmatch_T = regmatch_T {
        regprog: *prog,
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: ignore_case,
    };
    let mut r: bool = vim_regexec_string(&raw mut regmatch_0, line, col, false_0 != 0);
    *prog = regmatch_0.regprog;
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regexec(
    mut rmp: *mut regmatch_T,
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> bool {
    return vim_regexec_string(rmp, line, col, false_0 != 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regexec_nl(
    mut rmp: *mut regmatch_T,
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> bool {
    return vim_regexec_string(rmp, line, col, true_0 != 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_regexec_multi(
    mut rmp: *mut regmmatch_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut tm: *mut proftime_T,
    mut timed_out: *mut ::core::ffi::c_int,
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
    if (*(*rmp).regprog).re_in_use {
        emsg(gettext(
            (e_recursive.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return false_0;
    }
    (*(*rmp).regprog).re_in_use = true_0 != 0;
    if rex_in_use.get() {
        rex_save = rex.get();
    }
    rex_in_use.set(true_0 != 0);
    let mut result: ::core::ffi::c_int =
        (*(*(*rmp).regprog).engine)
            .regexec_multi
            .expect("non-null function pointer")(rmp, win, buf, lnum, col, tm, timed_out);
    (*(*rmp).regprog).re_in_use = false_0 != 0;
    if (*(*rmp).regprog).re_engine == AUTOMATIC_ENGINE as ::core::ffi::c_int as ::core::ffi::c_uint
        && result == NFA_TOO_EXPENSIVE as ::core::ffi::c_int
    {
        let mut save_p_re: ::core::ffi::c_int = p_re.get() as ::core::ffi::c_int;
        let mut re_flags: ::core::ffi::c_int = (*(*rmp).regprog).re_flags as ::core::ffi::c_int;
        let mut pat: *mut ::core::ffi::c_char =
            xstrdup((*((*rmp).regprog as *mut nfa_regprog_T)).pattern);
        p_re.set(BACKTRACKING_ENGINE as ::core::ffi::c_int as OptInt);
        let mut prev_prog: *mut regprog_T = (*rmp).regprog;
        report_re_switch(pat);
        reg_do_extmatch.set(REX_ALL);
        (*rmp).regprog = vim_regcomp(pat, re_flags);
        reg_do_extmatch.set(0 as ::core::ffi::c_int);
        if (*rmp).regprog.is_null() {
            (*rmp).regprog = prev_prog;
        } else {
            vim_regfree(prev_prog);
            (*(*rmp).regprog).re_in_use = true_0 != 0;
            result = (*(*(*rmp).regprog).engine)
                .regexec_multi
                .expect("non-null function pointer")(
                rmp, win, buf, lnum, col, tm, timed_out
            );
            (*(*rmp).regprog).re_in_use = false_0 != 0;
        }
        xfree(pat as *mut ::core::ffi::c_void);
        p_re.set(save_p_re as OptInt);
    }
    rex_in_use.set(rex_in_use_save);
    if rex_in_use.get() {
        rex.set(rex_save);
    }
    return if result <= 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        result
    };
}
