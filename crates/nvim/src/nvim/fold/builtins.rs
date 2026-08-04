use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::decoration::{clear_virttext, next_virt_text_chunk};
use crate::src::nvim::eval::typval::tv_get_lnum;
use crate::src::nvim::eval::vars::{get_vim_var_nr, get_vim_var_str};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curbuf, curwin};
use crate::src::nvim::memline::ml_get;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::os::libc::{ngettext, snprintf, strcat, strlen};
use crate::src::nvim::search::linewhite;
use crate::src::nvim::strings::concat_str;
use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::ptr;

use super::text::*;
use super::*;
use crate::src::nvim::types::{VV_FOLDDASHES, VV_FOLDEND, VV_FOLDSTART};

/// "foldclosed()" and "foldclosedend()" functions
pub(super) unsafe extern "C" fn foldclosed_both(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut end: bool,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 && lnum <= (*curbuf.get()).b_ml.ml_line_count {
        let mut first: linenr_T = 0;
        let mut last: linenr_T = 0;
        if hasFoldingWin(
            curwin.get(),
            lnum,
            &raw mut first,
            &raw mut last,
            false,
            ptr::null_mut(),
        ) {
            (*rettv).vval.v_number = (if end { last } else { first }) as varnumber_T;
            return;
        }
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
}

/// "foldclosed()" function
pub unsafe extern "C" fn f_foldclosed(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    foldclosed_both(argvars, rettv, false);
}

/// "foldclosedend()" function
pub unsafe extern "C" fn f_foldclosedend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    foldclosed_both(argvars, rettv, true);
}

/// "foldlevel()" function
pub unsafe extern "C" fn f_foldlevel(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 && lnum <= (*curbuf.get()).b_ml.ml_line_count {
        (*rettv).vval.v_number = foldLevel(lnum) as varnumber_T;
    }
}

/// "foldtext()" function
pub unsafe extern "C" fn f_foldtext(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    let mut foldstart: linenr_T = get_vim_var_nr(VV_FOLDSTART) as linenr_T;
    let mut foldend: linenr_T = get_vim_var_nr(VV_FOLDEND) as linenr_T;
    let mut dashes: *mut c_char = get_vim_var_str(VV_FOLDDASHES);
    if foldstart > 0 && foldend <= (*curbuf.get()).b_ml.ml_line_count {
        let mut lnum: linenr_T = 0;
        lnum = foldstart;
        while lnum < foldend {
            if !linewhite(lnum) {
                break;
            }
            lnum += 1;
        }
        let mut s: *mut c_char = skipwhite(ml_get(lnum));
        if *s.offset(0) as c_int == '/' as c_int
            && (*s.offset(1) as c_int == '*' as c_int || *s.offset(1) as c_int == '/' as c_int)
        {
            s = skipwhite(s.offset(2));
            if *skipwhite(s) as c_int == NUL && (lnum + 1) < foldend {
                s = skipwhite(ml_get(lnum + 1));
                if *s as c_int == '*' as c_int {
                    s = skipwhite(s.offset(1));
                }
            }
        }
        let mut count: c_int = foldend as c_int - foldstart as c_int + 1;
        let mut txt: *mut c_char = ngettext(
            c"+-%s%3d line: ".as_ptr(),
            c"+-%s%3d lines: ".as_ptr(),
            count as c_ulong,
        );
        let mut len: size_t = strlen(txt)
            .wrapping_add(strlen(dashes))
            .wrapping_add(20)
            .wrapping_add(strlen(s));
        let mut r: *mut c_char = xmalloc(len) as *mut c_char;
        snprintf(r, len, txt, dashes, count);
        len = strlen(r);
        strcat(r, s);
        foldtext_cleanup(r.add(len));
        (*rettv).vval.v_string = r;
    }
}

/// "foldtextresult(lnum)" function
pub unsafe extern "C" fn f_foldtextresult(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [c_char; 51] = [0; 51];
    static entered: GlobalCell<bool> = GlobalCell::new(false);
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    if entered.get() {
        return;
    }
    entered.set(true);
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    lnum = if lnum > 0 { lnum } else { 0 };
    let mut info: foldinfo_T = fold_info(curwin.get(), lnum);
    if info.fi_lines > 0 {
        let mut vt: VirtText = VIRTTEXT_EMPTY;
        let mut text: *mut c_char = get_foldtext(
            curwin.get(),
            lnum,
            lnum + info.fi_lines - 1,
            info,
            &raw mut buf as *mut c_char,
            &raw mut vt,
        );
        if text == &raw mut buf as *mut c_char {
            text = xstrdup(text);
        }
        if vt.size > 0 {
            assert!(*text as c_int == '\0' as c_int, "*text == NUL");
            let mut i: size_t = 0;
            while i < vt.size {
                let mut attr: c_int = 0;
                let mut new_text: *mut c_char = next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
                if new_text.is_null() {
                    break;
                }
                new_text = concat_str(text, new_text);
                xfree(text as *mut c_void);
                text = new_text;
            }
        }
        clear_virttext(&raw mut vt);
        (*rettv).vval.v_string = text;
    }
    entered.set(false);
}
