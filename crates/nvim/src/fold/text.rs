//! Rendering the one line a closed fold is drawn as.
//!
//! This is display, not tree: 'foldtext' is an expression the user controls,
//! so most of the work here is evaluating it safely (a failed evaluation
//! latches, so a broken 'foldtext' does not raise an error per fold) and
//! sanitising whatever it answers.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::extmark::parse_virt_text;
use crate::api::private::helpers::{api_clear_error, api_free_object};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{ptr2cells, skipwhite, transstr, vim_isprintc};
use crate::eval::eval_foldtext;
use crate::eval::vars::{set_vim_var_nr, set_vim_var_string};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::main::{curbuf, current_sctx, curwin, did_emsg};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memory::xfree;
use crate::os::cshim::{memmove, ngettext, strncmp, strstr};
use crate::strings::vim_snprintf;
use crate::types::{Vv, kErrorTypeNone, kObjectTypeArray, kObjectTypeNil, kObjectTypeString};
use ::libc::{memset, strlen};
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
///
/// # Safety
/// `wp` must be a live window, `buf` writable for [`FOLD_TEXT_LEN`] bytes,
/// and `vt` writable.
pub unsafe fn get_foldtext(
    wp: *mut win_T,
    lnum: linenr_T,
    lnume: linenr_T,
    foldinfo: foldinfo_T,
    buf: *mut c_char,
    vt: *mut VirtText,
) -> *mut c_char {
    let mut text: *mut c_char = ptr::null_mut();
    // A 'foldtext' that errored is not evaluated again until the window or
    // the direction of travel changes, so one broken expression does not
    // raise one error per drawn fold.
    static got_fdt_error: GlobalCell<bool> = GlobalCell::new(false);
    static last_wp: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());
    static last_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
    let save_did_emsg = did_emsg.get();
    if last_wp.get().is_null()
        || last_wp.get() != wp
        || last_lnum.get() > lnum
        || last_lnum.get() == 0
    {
        got_fdt_error.set(false);
    }
    if !got_fdt_error.get() {
        did_emsg.set(0);
    }
    // SAFETY: the caller's promise; every pointer below is either `wp`'s,
    // ours, or one the API handed back.
    unsafe {
        if *(*wp).w_onebuf_opt.wo_fdt as c_int != NUL {
            let mut dashes: [c_char; 22] = [0; 22];
            set_vim_var_nr(Vv::Foldstart, lnum as varnumber_T);
            set_vim_var_nr(Vv::Foldend, lnume as varnumber_T);
            let level = foldinfo.fi_level.min(dashes.len() as c_int - 1);
            memset(
                &raw mut dashes as *mut c_char as *mut c_void,
                '-' as c_int,
                level as size_t,
            );
            dashes[level as usize] = NUL as c_char;
            set_vim_var_string(
                Vv::Folddashes,
                &raw mut dashes as *mut c_char,
                level as ptrdiff_t,
            );
            set_vim_var_nr(Vv::Foldlevel, level as varnumber_T);
            if !got_fdt_error.get() {
                let save_curwin = curwin.get();
                let saved_sctx = current_sctx.get();
                curwin.set(wp);
                curbuf.set((*wp).w_buffer);
                current_sctx.set((*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldtext as usize]);
                let no_emsg = Suppress::emsg();
                let mut obj: Object = eval_foldtext(wp);
                if obj.type_0 as c_uint == kObjectTypeArray as c_uint {
                    // A list of `[text, hl]` chunks: the caller draws them,
                    // and the returned text is empty.
                    let mut err = Error {
                        type_0: kErrorTypeNone,
                        msg: ptr::null_mut(),
                    };
                    *vt = parse_virt_text(obj.data.array, &raw mut err, ptr::null_mut());
                    if err.type_0 as c_int == kErrorTypeNone as c_int {
                        *buf = NUL as c_char;
                        text = buf;
                    }
                    api_clear_error(&raw mut err);
                } else if obj.type_0 as c_uint == kObjectTypeString as c_uint {
                    text = obj.data.string.data();
                    obj = object {
                        type_0: kObjectTypeNil,
                        data: object_data { boolean: false },
                    };
                }
                api_free_object(obj);
                drop(no_emsg);
                if text.is_null() || did_emsg.get() != 0 {
                    got_fdt_error.set(true);
                }
                curwin.set(save_curwin);
                curbuf.set((*curwin.get()).w_buffer);
                current_sctx.set(saved_sctx);
            }
            last_lnum.set(lnum);
            last_wp.set(wp);
            set_vim_var_string(Vv::Folddashes, ptr::null(), -1 as ptrdiff_t);
            if did_emsg.get() == 0 && save_did_emsg != 0 {
                did_emsg.set(save_did_emsg);
            }
            if !text.is_null() {
                // Tabs become spaces and anything unprintable or wide sends
                // the whole string through `transstr`.
                let mut p = text;
                while *p as c_int != NUL {
                    let len = utfc_ptr2len(p);
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
            let count = lnume - lnum + 1;
            vim_snprintf(
                buf,
                FOLD_TEXT_LEN as size_t,
                ngettext(
                    c"+--%3d line folded".as_ptr(),
                    c"+--%3d lines folded ".as_ptr(),
                    count as c_ulong,
                ),
                count,
            );
            text = buf;
        }
    }
    text
}

/// Remove 'foldmarker' and 'commentstring' from "str" (in-place).
///
/// # Safety
/// `str` must be a writable NUL-terminated string, and the current window
/// must be live.
pub(super) unsafe fn foldtext_cleanup(str: *mut c_char) {
    // SAFETY: the caller's promise; both 'commentstring' and `str` are
    // NUL-terminated, so every scan below stops inside its own string.
    unsafe {
        // 'commentstring' split around its `%s`, with the padding trimmed.
        let cms_start = skipwhite((*curbuf.get()).b_p_cms);
        let mut cms_slen = strlen(cms_start);
        while cms_slen > 0 && ascii_iswhite(*cms_start.add(cms_slen.wrapping_sub(1)) as c_int) {
            cms_slen = cms_slen.wrapping_sub(1);
        }
        let mut cms_end = strstr(cms_start, c"%s".as_ptr());
        let mut cms_elen: size_t = 0;
        if !cms_end.is_null() {
            cms_elen = cms_slen.wrapping_sub(cms_end.offset_from(cms_start) as size_t);
            cms_slen = cms_end.offset_from(cms_start) as size_t;
            while cms_slen > 0 && ascii_iswhite(*cms_start.add(cms_slen.wrapping_sub(1)) as c_int) {
                cms_slen = cms_slen.wrapping_sub(1);
            }
            let s = skipwhite(cms_end.offset(2));
            cms_elen = cms_elen.wrapping_sub(s.offset_from(cms_end) as size_t);
            cms_end = s;
        }
        parse_marker(curwin.get());
        // Each half of 'commentstring' is only removed once.
        let mut did1 = false;
        let mut did2 = false;
        let mut s = str;
        while *s as c_int != NUL {
            let mut len: size_t = 0;
            if strncmp(
                s,
                (*curwin.get()).w_onebuf_opt.wo_fmr,
                foldstartmarkerlen.get(),
            ) == 0
            {
                len = foldstartmarkerlen.get();
            } else if strncmp(s, foldendmarker.get(), foldendmarkerlen.get()) == 0 {
                len = foldendmarkerlen.get();
            }
            if len > 0 {
                // A numbered marker, and any comment opener it sits behind.
                if ascii_isdigit(*s.add(len) as c_int) {
                    len = len.wrapping_add(1);
                }
                let mut p = s;
                while p > str && ascii_iswhite(*p.offset(-1) as c_int) {
                    p = p.offset(-1);
                }
                if p >= str.add(cms_slen)
                    && strncmp(p.offset(-(cms_slen as isize)), cms_start, cms_slen) == 0
                {
                    len = len.wrapping_add((s.offset_from(p) as size_t).wrapping_add(cms_slen));
                    s = p.offset(-(cms_slen as isize));
                }
            } else if !cms_end.is_null() {
                if !did1 && cms_slen > 0 && strncmp(s, cms_start, cms_slen) == 0 {
                    len = cms_slen;
                    did1 = true;
                } else if !did2 && cms_elen > 0 && strncmp(s, cms_end, cms_elen) == 0 {
                    len = cms_elen;
                    did2 = true;
                }
            }
            if len != 0 {
                while ascii_iswhite(*s.add(len) as c_int) {
                    len = len.wrapping_add(1);
                }
                memmove(
                    s as *mut c_void,
                    s.add(len) as *const c_void,
                    strlen(s.add(len)).wrapping_add(1),
                );
            } else {
                s = s.offset(utfc_ptr2len(s) as isize);
            }
        }
    }
}
