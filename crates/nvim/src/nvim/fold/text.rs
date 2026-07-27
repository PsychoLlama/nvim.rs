use crate::src::nvim::api::extmark::parse_virt_text;
use crate::src::nvim::api::private::helpers::{api_clear_error, api_free_object};
use crate::src::nvim::charset::{ptr2cells, skipwhite, transstr, vim_isprintc};
use crate::src::nvim::eval::vars::{set_vim_var_nr, set_vim_var_string};
use crate::src::nvim::eval_1::eval_foldtext;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curbuf, current_sctx, curwin, did_emsg, emsg_off};
use crate::src::nvim::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::os::libc::{memmove, memset, ngettext, strlen, strncmp, strstr};
use crate::src::nvim::strings::vim_snprintf;
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

use super::marker::*;
use super::*;

/// Generates text to display
///
/// `buf` — allocated memory of length FOLD_TEXT_LEN. Used when 'foldtext'
///            isn't set puts the result in "buf[FOLD_TEXT_LEN]".
/// `at` — line "lnum", with last line "lnume".
/// Returns the text for a closed fold
///
/// Otherwise the result is in allocated memory.
pub unsafe extern "C" fn get_foldtext(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut lnume: linenr_T,
    mut foldinfo: foldinfo_T,
    mut buf: *mut c_char,
    mut vt: *mut VirtText,
) -> *mut c_char {
    let mut text: *mut c_char = ptr::null_mut();
    static got_fdt_error: GlobalCell<bool> = GlobalCell::new(false);
    let mut save_did_emsg: c_int = did_emsg.get();
    static last_wp: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());
    static last_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
    if (*last_wp.ptr()).is_null()
        || last_wp.get() != wp
        || last_lnum.get() > lnum
        || last_lnum.get() == 0
    {
        got_fdt_error.set(false);
    }
    if !got_fdt_error.get() {
        did_emsg.set(false_0);
    }
    if *(*wp).w_onebuf_opt.wo_fdt as c_int != NUL {
        let mut dashes: [c_char; 22] = [0; 22];
        set_vim_var_nr(VV_FOLDSTART, lnum as varnumber_T);
        set_vim_var_nr(VV_FOLDEND, lnume as varnumber_T);
        let mut level: c_int = if foldinfo.fi_level < size_of::<[c_char; 22]>() as c_int - 1 {
            foldinfo.fi_level
        } else {
            size_of::<[c_char; 22]>() as c_int - 1
        };
        memset(
            &raw mut dashes as *mut c_char as *mut c_void,
            '-' as c_int,
            level as size_t,
        );
        dashes[level as usize] = NUL as c_char;
        set_vim_var_string(
            VV_FOLDDASHES,
            &raw mut dashes as *mut c_char,
            level as ptrdiff_t,
        );
        set_vim_var_nr(VV_FOLDLEVEL, level as varnumber_T);
        if !got_fdt_error.get() {
            let save_curwin: *mut win_T = curwin.get();
            let saved_sctx: sctx_T = current_sctx.get();
            curwin.set(wp);
            curbuf.set((*wp).w_buffer);
            current_sctx.set((*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldtext as c_int as usize]);
            (*emsg_off.ptr()) += 1;
            let mut obj: Object = eval_foldtext(wp);
            if obj.type_0 as c_uint == kObjectTypeArray as c_int as c_uint {
                let mut err: Error = Error {
                    type_0: kErrorTypeNone,
                    msg: ptr::null_mut(),
                };
                *vt = parse_virt_text(obj.data.array, &raw mut err, ptr::null_mut());
                if !(err.type_0 as c_int != kErrorTypeNone as c_int) {
                    *buf = NUL as c_char;
                    text = buf;
                }
                api_clear_error(&raw mut err);
            } else if obj.type_0 as c_uint == kObjectTypeString as c_int as c_uint {
                text = obj.data.string.data;
                obj = object {
                    type_0: kObjectTypeNil,
                    data: object_data { boolean: false },
                };
            }
            api_free_object(obj);
            (*emsg_off.ptr()) -= 1;
            if text.is_null() || did_emsg.get() != 0 {
                got_fdt_error.set(true);
            }
            curwin.set(save_curwin);
            curbuf.set((*curwin.get()).w_buffer);
            current_sctx.set(saved_sctx);
        }
        last_lnum.set(lnum);
        last_wp.set(wp);
        set_vim_var_string(VV_FOLDDASHES, ptr::null(), -1 as ptrdiff_t);
        if did_emsg.get() == 0 && save_did_emsg != 0 {
            did_emsg.set(save_did_emsg);
        }
        if !text.is_null() {
            let mut p: *mut c_char = ptr::null_mut();
            p = text;
            while *p as c_int != NUL {
                let mut len: c_int = utfc_ptr2len(p);
                if len > 1 {
                    if !vim_isprintc(utf_ptr2char(p)) {
                        break;
                    }
                    p = p.offset((len - 1) as isize);
                } else if *p as c_int == TAB {
                    *p = ' ' as c_char;
                } else if ptr2cells(p) > 1 {
                    break;
                }
                p = p.offset(1);
            }
            if *p as c_int != NUL {
                p = transstr(text, true);
                xfree(text as *mut c_void);
                text = p;
            }
        }
    }
    if text.is_null() {
        let mut count: c_int = lnume as c_int - lnum as c_int + 1;
        vim_snprintf(
            buf,
            FOLD_TEXT_LEN as c_int as size_t,
            ngettext(
                c"+--%3d line folded".as_ptr(),
                c"+--%3d lines folded ".as_ptr(),
                count as c_ulong,
            ),
            count,
        );
        text = buf;
    }
    return text;
}

/// Remove 'foldmarker' and 'commentstring' from "str" (in-place).
pub(super) unsafe extern "C" fn foldtext_cleanup(mut str: *mut c_char) {
    let mut cms_start: *mut c_char = skipwhite((*curbuf.get()).b_p_cms);
    let mut cms_slen: size_t = strlen(cms_start);
    while cms_slen > 0 && ascii_iswhite(*cms_start.add(cms_slen.wrapping_sub(1)) as c_int) {
        cms_slen = cms_slen.wrapping_sub(1);
    }
    let mut cms_end: *mut c_char = strstr(cms_start, c"%s".as_ptr());
    let mut cms_elen: size_t = 0;
    if !cms_end.is_null() {
        cms_elen = cms_slen.wrapping_sub(cms_end.offset_from(cms_start) as size_t);
        cms_slen = cms_end.offset_from(cms_start) as size_t;
        while cms_slen > 0 && ascii_iswhite(*cms_start.add(cms_slen.wrapping_sub(1)) as c_int) {
            cms_slen = cms_slen.wrapping_sub(1);
        }
        let mut s: *mut c_char = skipwhite(cms_end.offset(2));
        cms_elen = cms_elen.wrapping_sub(s.offset_from(cms_end) as size_t);
        cms_end = s;
    }
    parseMarker(curwin.get());
    let mut did1: bool = false;
    let mut did2: bool = false;
    let mut s_0: *mut c_char = str;
    while *s_0 as c_int != NUL {
        let mut len: size_t = 0;
        if strncmp(
            s_0,
            (*curwin.get()).w_onebuf_opt.wo_fmr,
            foldstartmarkerlen.get(),
        ) == 0
        {
            len = foldstartmarkerlen.get();
        } else if strncmp(s_0, foldendmarker.get(), foldendmarkerlen.get()) == 0 {
            len = foldendmarkerlen.get();
        }
        if len > 0 {
            if ascii_isdigit(*s_0.add(len) as c_int) {
                len = len.wrapping_add(1);
            }
            let mut p: *mut c_char = ptr::null_mut();
            p = s_0;
            while p > str && ascii_iswhite(*p.offset(-1) as c_int) {
                p = p.offset(-1);
            }
            if p >= str.add(cms_slen)
                && strncmp(p.offset(-(cms_slen as isize)), cms_start, cms_slen) == 0
            {
                len = len.wrapping_add((s_0.offset_from(p) as size_t).wrapping_add(cms_slen));
                s_0 = p.offset(-(cms_slen as isize));
            }
        } else if !cms_end.is_null() {
            if !did1 && cms_slen > 0 && strncmp(s_0, cms_start, cms_slen) == 0 {
                len = cms_slen;
                did1 = true;
            } else if !did2 && cms_elen > 0 && strncmp(s_0, cms_end, cms_elen) == 0 {
                len = cms_elen;
                did2 = true;
            }
        }
        if len != 0 {
            while ascii_iswhite(*s_0.add(len) as c_int) {
                len = len.wrapping_add(1);
            }
            memmove(
                s_0 as *mut c_void,
                s_0.add(len) as *const c_void,
                strlen(s_0.add(len)).wrapping_add(1),
            );
        } else {
            s_0 = s_0.offset(utfc_ptr2len(s_0) as isize);
        }
    }
}
