//! Indent computed by running something: 'indentexpr', and the built-in
//! Lisp indenter behind 'lisp'.

use super::*;
use crate::src::nvim::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::src::nvim::cursor::{check_cursor, get_cursor_line_ptr};
use crate::src::nvim::eval::eval_to_number;
use crate::src::nvim::eval::typval::tv_get_lnum;
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::ex_docmd::handle_did_throw;
use crate::src::nvim::indent_c::{cindent_on, do_c_expr_indent};
use crate::src::nvim::main::{
    State, curbuf, current_sctx, curwin, did_ai, did_throw, p_debug, p_lispwords, p_paste, sandbox,
    textlock, trylevel,
};
use crate::src::nvim::mbyte::{utf_ptr2CharInfo, utf_ptr2StrCharInfo, utfc_next};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::option::{copy_option_part, was_set_insecurely};
use crate::src::nvim::os::libc::{strcmp, strncmp};
use crate::src::nvim::plines::{init_charsize_arg, win_charsize};
use crate::src::nvim::pos::lt;
use crate::src::nvim::search::{findmatch, linewhite};
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::vim_strchr;

pub unsafe extern "C" fn get_expr_indent() -> ::core::ffi::c_int {
    let mut use_sandbox: bool = was_set_insecurely(
        curwin.get(),
        kOptIndentexpr,
        OPT_LOCAL as ::core::ffi::c_int,
    );
    let save_sctx: sctx_T = current_sctx.get();
    let mut save_pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_curswant: colnr_T = (*curwin.get()).w_curswant;
    let mut save_set_curswant: bool = (*curwin.get()).w_set_curswant != 0;
    set_vim_var_nr(VV_LNUM, (*curwin.get()).w_cursor.lnum as varnumber_T);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    current_sctx
        .set((*curbuf.get()).b_p_script_ctx[kBufOptIndentexpr as ::core::ffi::c_int as usize]);
    let mut inde_copy: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_inde);
    let mut indent: ::core::ffi::c_int = eval_to_number(inde_copy, true) as ::core::ffi::c_int;
    xfree(inde_copy as *mut ::core::ffi::c_void);
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    current_sctx.set(save_sctx);
    let mut save_State: ::core::ffi::c_int = State.get();
    State.set(MODE_INSERT);
    (*curwin.get()).w_cursor = save_pos;
    (*curwin.get()).w_curswant = save_curswant;
    (*curwin.get()).w_set_curswant = save_set_curswant as ::core::ffi::c_int;
    check_cursor(curwin.get());
    State.set(save_State);
    if did_throw.get()
        && (vim_strchr(p_debug.get(), 't' as ::core::ffi::c_int).is_null()
            || trylevel.get() == 0 as ::core::ffi::c_int)
    {
        handle_did_throw();
        did_throw.set(false);
    }
    if indent < 0 as ::core::ffi::c_int {
        indent = get_indent();
    }
    return indent;
}
pub unsafe extern "C" fn get_lisp_indent() -> ::core::ffi::c_int {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut paren: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut amount: ::core::ffi::c_int = 0;
    let mut realpos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    pos = findmatch(
        ::core::ptr::null_mut::<oparg_T>(),
        '(' as ::core::ffi::c_int,
    );
    if pos.is_null() {
        pos = findmatch(
            ::core::ptr::null_mut::<oparg_T>(),
            '[' as ::core::ffi::c_int,
        );
    } else {
        paren = *pos;
        pos = findmatch(
            ::core::ptr::null_mut::<oparg_T>(),
            '[' as ::core::ffi::c_int,
        );
        if pos.is_null() || lt(*pos, paren) {
            pos = &raw mut paren;
        }
    }
    if !pos.is_null() {
        amount = -1 as ::core::ffi::c_int;
        let mut parencount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            (*curwin.get()).w_cursor.lnum -= 1;
            if (*curwin.get()).w_cursor.lnum < (*pos).lnum {
                break;
            }
            if linewhite((*curwin.get()).w_cursor.lnum) {
                continue;
            }
            let mut that: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            while *that as ::core::ffi::c_int != NUL {
                if *that as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                    while *that.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != NUL
                    {
                        that = that.offset(1);
                    }
                } else if *that as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    if *that.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                        that = that.offset(1);
                    }
                } else {
                    if *that as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                        && *that.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        loop {
                            that = that.offset(1);
                            if !(*that as ::core::ffi::c_int != 0
                                && *that as ::core::ffi::c_int != '"' as ::core::ffi::c_int)
                            {
                                break;
                            }
                            if *that as ::core::ffi::c_int != '\\' as ::core::ffi::c_int {
                                continue;
                            }
                            that = that.offset(1);
                            if *that as ::core::ffi::c_int == NUL {
                                break;
                            }
                            if *that.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != NUL
                            {
                                continue;
                            }
                            that = that.offset(1);
                            break;
                        }
                        if *that as ::core::ffi::c_int == NUL {
                            break;
                        }
                    }
                    if *that as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                        || *that as ::core::ffi::c_int == '[' as ::core::ffi::c_int
                    {
                        parencount += 1;
                    } else if *that as ::core::ffi::c_int == ')' as ::core::ffi::c_int
                        || *that as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                    {
                        parencount -= 1;
                    }
                }
                that = that.offset(1);
            }
            if parencount != 0 as ::core::ffi::c_int {
                continue;
            }
            amount = get_indent();
            break;
        }
        if amount == -1 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.lnum = (*pos).lnum;
            (*curwin.get()).w_cursor.col = (*pos).col;
            let mut col: colnr_T = (*pos).col;
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let mut csarg: CharsizeArg = CharsizeArg::default();
            let mut cstype: CharsizeKind =
                init_charsize_arg(&mut csarg, curwin.get(), (*pos).lnum, line);
            let mut sci: StrCharInfo = utf_ptr2StrCharInfo(line);
            amount = 0 as ::core::ffi::c_int;
            while *sci.ptr as ::core::ffi::c_int != NUL && col > 0 as ::core::ffi::c_int {
                amount += win_charsize(cstype, amount, sci.ptr, sci.chr.value, &mut csarg).width;
                sci = utfc_next(sci);
                col -= 1;
            }
            let mut that_0: *mut ::core::ffi::c_char = sci.ptr;
            if (*that_0 as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                || *that_0 as ::core::ffi::c_int == '[' as ::core::ffi::c_int)
                && lisp_match(that_0.offset(1 as ::core::ffi::c_int as isize))
            {
                amount += 2 as ::core::ffi::c_int;
            } else {
                if *that_0 as ::core::ffi::c_int != NUL {
                    that_0 = that_0.offset(1);
                    amount += 1;
                }
                let mut firsttry: colnr_T = amount as colnr_T;
                while ascii_iswhite(*that_0 as ::core::ffi::c_int) {
                    amount += win_charsize(
                        cstype,
                        amount,
                        that_0,
                        *that_0 as uint8_t as int32_t,
                        &mut csarg,
                    )
                    .width;
                    that_0 = that_0.offset(1);
                }
                if *that_0 as ::core::ffi::c_int != 0
                    && *that_0 as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                {
                    if *that_0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int
                        && *that_0 as ::core::ffi::c_int != '[' as ::core::ffi::c_int
                    {
                        firsttry += 1;
                    }
                    parencount = 0 as ::core::ffi::c_int;
                    let mut ci: CharInfo = utf_ptr2CharInfo(that_0);
                    if ci.value != '"' as int32_t
                        && ci.value != '\'' as int32_t
                        && ci.value != '#' as int32_t
                        && (ci.value < '0' as int32_t || ci.value > '9' as int32_t)
                    {
                        let mut quotecount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while *that_0 as ::core::ffi::c_int != 0
                            && (!ascii_iswhite(ci.value as ::core::ffi::c_int)
                                || quotecount != 0
                                || parencount != 0)
                        {
                            if ci.value == '"' as int32_t {
                                quotecount = (quotecount == 0) as ::core::ffi::c_int;
                            }
                            if (ci.value == '(' as int32_t || ci.value == '[' as int32_t)
                                && quotecount == 0
                            {
                                parencount += 1;
                            }
                            if (ci.value == ')' as int32_t || ci.value == ']' as int32_t)
                                && quotecount == 0
                            {
                                parencount -= 1;
                            }
                            if ci.value == '\\' as int32_t
                                && *that_0.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != NUL
                            {
                                amount +=
                                    win_charsize(cstype, amount, that_0, ci.value, &mut csarg)
                                        .width;
                                let mut next_sci: StrCharInfo = utfc_next(StrCharInfo {
                                    ptr: that_0,
                                    chr: ci,
                                });
                                that_0 = next_sci.ptr;
                                ci = next_sci.chr;
                            }
                            amount +=
                                win_charsize(cstype, amount, that_0, ci.value, &mut csarg).width;
                            let mut next_sci_0: StrCharInfo = utfc_next(StrCharInfo {
                                ptr: that_0,
                                chr: ci,
                            });
                            that_0 = next_sci_0.ptr;
                            ci = next_sci_0.chr;
                        }
                    }
                    while ascii_iswhite(*that_0 as ::core::ffi::c_int) {
                        amount += win_charsize(
                            cstype,
                            amount,
                            that_0,
                            *that_0 as uint8_t as int32_t,
                            &mut csarg,
                        )
                        .width;
                        that_0 = that_0.offset(1);
                    }
                    if *that_0 == 0 || *that_0 as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                        amount = firsttry as ::core::ffi::c_int;
                    }
                }
            }
        }
    } else {
        amount = 0 as ::core::ffi::c_int;
    }
    (*curwin.get()).w_cursor = realpos;
    return amount;
}
unsafe extern "C" fn lisp_match(mut p: *mut ::core::ffi::c_char) -> bool {
    let mut buf: [::core::ffi::c_char; 512] = [0; 512];
    let mut word: *mut ::core::ffi::c_char = if *(*curbuf.get()).b_p_lw as ::core::ffi::c_int != NUL
    {
        (*curbuf.get()).b_p_lw
    } else {
        p_lispwords.get()
    };
    while *word as ::core::ffi::c_int != NUL {
        let mut len: size_t = copy_option_part(
            &raw mut word,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 512]>(),
            c",".as_ptr() as *mut ::core::ffi::c_char,
        );
        if strncmp(&raw mut buf as *mut ::core::ffi::c_char, p, len) == 0 as ::core::ffi::c_int
            && ascii_iswhite_or_nul(*p.add(len) as ::core::ffi::c_int)
        {
            return true;
        }
    }
    false
}
pub unsafe extern "C" fn fixthisline(mut get_the_indent: IndentGetter) {
    let mut amount: ::core::ffi::c_int = get_the_indent.expect("non-null function pointer")();
    if amount < 0 as ::core::ffi::c_int {
        return;
    }
    change_indent(INDENT_SET as ::core::ffi::c_int, amount, 0, true);
    if linewhite((*curwin.get()).w_cursor.lnum) {
        did_ai.set(true);
    }
}
pub unsafe extern "C" fn use_indentexpr_for_lisp() -> bool {
    return (*curbuf.get()).b_p_lisp != 0
        && *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL
        && strcmp((*curbuf.get()).b_p_lop, c"expr:1".as_ptr()) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn fix_indent() {
    if p_paste.get() != 0 {
        return;
    }
    if (*curbuf.get()).b_p_lisp != 0 && (*curbuf.get()).b_p_ai != 0 {
        if use_indentexpr_for_lisp() {
            do_c_expr_indent();
        } else {
            fixthisline(Some(get_lisp_indent));
        }
    } else if cindent_on() {
        do_c_expr_indent();
    }
}
pub unsafe extern "C" fn f_indent(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf.get()).b_ml.ml_line_count {
        (*rettv).vval.v_number = get_indent_lnum(lnum) as varnumber_T;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
pub unsafe extern "C" fn f_lispindent(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let pos: pos_T = (*curwin.get()).w_cursor;
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_cursor.lnum = lnum;
        (*rettv).vval.v_number = get_lisp_indent() as varnumber_T;
        (*curwin.get()).w_cursor = pos;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
