//! Rendering the one line a closed fold is drawn as.
//!
//! This is display, not tree: 'foldtext' is an expression the user controls,
//! so most of the work here is evaluating it safely (a failed evaluation
//! latches, so a broken 'foldtext' does not raise an error per fold) and
//! sanitising whatever it answers.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
use crate::api::extmark::parse_virt_text;
use crate::cstr;

use crate::api::private::helpers::api_free_object;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{ptr2cells, skipwhite, transstr, vim_isprintc};
use crate::eval::eval_foldtext;
use crate::eval::vars::{set_vim_var_nr, set_vim_var_string};
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memory::xfree;
use crate::message::state::did_emsg;
use crate::os::cshim::{ngettext, strstr};
use crate::runtime::state::current_sctx;
use crate::strings::vim_snprintf;
use crate::types::Vv;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::ptr;

use super::marker::*;
use super::*;
use crate::options::kWinOptFoldtext;

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
/// `buf` must be writable for [`FOLD_TEXT_LEN`] bytes, and `vt` writable.
pub unsafe fn get_foldtext(
    window: Win,
    lnum: LineNr,
    lnume: LineNr,
    foldinfo: FoldInfo,
    buf: *mut c_char,
    vt: *mut VirtText,
) -> *mut c_char {
    let mut text: *mut c_char = ptr::null_mut();
    // A 'foldtext' that errored is not evaluated again until the window or
    // the direction of travel changes, so one broken expression does not
    // raise one error per drawn fold.
    static got_fdt_error: GlobalCell<bool> = GlobalCell::new(false);
    static last_wp: GlobalCell<*mut Window> = GlobalCell::new(ptr::null_mut());
    static last_lnum: GlobalCell<LineNr> = GlobalCell::new(0);
    let save_did_emsg = did_emsg.get();
    if last_wp.get().is_null()
        || last_wp.get() != window.raw()
        || last_lnum.get() > lnum
        || last_lnum.get() == 0
    {
        got_fdt_error.set(false);
    }
    if !got_fdt_error.get() {
        did_emsg.set(0);
    }
    let win = window;
    // SAFETY: 'foldtext' is a NUL-terminated option string.
    if unsafe { *win.w_onebuf_opt.wo_fdt } as c_int != NUL {
        let mut dashes: [c_char; 22] = [0; 22];
        let level = foldinfo.fi_level.min(dashes.len() as c_int - 1);
        dashes[..level as usize].fill('-' as c_char);
        dashes[level as usize] = NUL as c_char;
        // SAFETY: `dashes` is this frame's, and `level` bytes of it are set.
        let ds = dashes.as_mut_ptr();
        set_vim_var_nr(Vv::Foldstart, lnum as VarNumber);
        set_vim_var_nr(Vv::Foldend, lnume as VarNumber);
        unsafe { set_vim_var_string(Vv::Folddashes, ds, level as ptrdiff_t) };
        set_vim_var_nr(Vv::Foldlevel, level as VarNumber);
        if !got_fdt_error.get() {
            let saved = switch_to(window);
            let saved_sctx = current_sctx.get();
            current_sctx.set(win.w_onebuf_opt.wo_script_ctx[kWinOptFoldtext as usize]);
            let no_emsg = Suppress::emsg();
            let mut obj: Object = unsafe { eval_foldtext(window) };
            if let Object::Array(chunks) = obj {
                // A list of `[text, hl]` chunks: the caller draws them,
                // and the returned text is empty.
                if let Ok(parsed) = unsafe { parse_virt_text(chunks, ptr::null_mut()) } {
                    unsafe { (*vt, *buf) = (parsed, NUL as c_char) };
                    text = buf;
                }
            } else if let Object::String(s) = obj {
                // `text` keeps the bytes; clear the object so the free below
                // leaves them alone.
                text = s.data();
                obj = Object::Nil;
            }
            unsafe { api_free_object(obj) };
            drop(no_emsg);
            if text.is_null() || did_emsg.get() != 0 {
                got_fdt_error.set(true);
            }
            saved.restore();
            current_sctx.set(saved_sctx);
        }
        last_lnum.set(lnum);
        last_wp.set(window.raw());
        unsafe { set_vim_var_string(Vv::Folddashes, ptr::null(), -1 as ptrdiff_t) };
        if did_emsg.get() == 0 && save_did_emsg != 0 {
            did_emsg.set(save_did_emsg);
        }
        if !text.is_null() {
            // Tabs become spaces and anything unprintable or wide sends
            // the whole string through `transstr`.
            // `text` is one NUL-terminated string; the walk reads a byte
            // per step and stops on the terminator.
            // SAFETY: `p` is inside it, at or before that terminator.
            let at = |p: *const c_char| unsafe { *p } as c_int;
            let mut p = text;
            while at(p) != NUL {
                // SAFETY: as `at`.
                let len = unsafe { utfc_ptr2len(p) };
                if len > 1 {
                    // SAFETY: as `at`.
                    if !unsafe { vim_isprintc(utf_ptr2char(p)) } {
                        break;
                    }
                    p = p.wrapping_offset((len - 1) as isize);
                } else if at(p) == TAB {
                    // SAFETY: as `at`; the string is the caller's to change.
                    unsafe { *p = ' ' as c_char };
                } else if unsafe { ptr2cells(p) } > 1 {
                    break;
                }
                p = p.wrapping_offset(1);
            }
            if at(p) != NUL {
                // SAFETY: `text` is a live NUL-terminated allocation of ours.
                p = unsafe { transstr(text, true) };
                unsafe { xfree(text as *mut c_void) };
                text = p;
            }
        }
    }
    if text.is_null() {
        let count = lnume - lnum + 1;
        // SAFETY: the caller's promise -- `buf` holds `FOLD_TEXT_LEN` bytes,
        // whose size is passed with it.
        unsafe {
            let one = c"+--%3d line folded";
            let many = c"+--%3d lines folded ";
            let fmt = ngettext(one, many, count as c_ulong);
            vim_snprintf(buf, FOLD_TEXT_LEN as size_t, fmt.as_ptr(), count)
        };
        text = buf;
    }
    text
}

/// Remove 'foldmarker' and 'commentstring' from "str" (in-place).
///
/// # Safety
/// `str` must be a writable NUL-terminated string, and the current window
/// must be live.
pub(super) unsafe fn foldtext_cleanup(str: *mut c_char) {
    // Everything below walks two NUL-terminated strings -- `str`, the
    // caller's, and 'commentstring' -- and no step passes either terminator.
    // SAFETY: `p` is inside one of them, at or before its terminator.
    let at = |p: *const c_char| unsafe { *p } as c_int;
    // SAFETY: as `at`; `n` never reaches past the shorter of the two.
    let ncmp = |a: *const c_char, b: *const c_char, n: size_t| unsafe { cstr::prefix_eq(a, b, n) };
    // SAFETY: as `at`.
    let skip_ws = |p: *mut c_char| unsafe { skipwhite(p) };
    // How far `b` is past `a`, in bytes, without dereferencing either.
    let gap = |b: *const c_char, a: *const c_char| b.addr().wrapping_sub(a.addr()) as size_t;

    // 'commentstring' split around its `%s`, with the padding trimmed.
    let cms_start = skip_ws(Buf::current().b_p_cms);
    // SAFETY: 'commentstring' is a NUL-terminated option string.
    let mut cms_slen = unsafe { cstr::bytes_at(cms_start) }.len();
    while cms_slen > 0 && ascii_iswhite(at(cms_start.wrapping_add(cms_slen - 1))) {
        cms_slen -= 1;
    }
    // SAFETY: as `at`; both arguments are NUL-terminated.
    let mut cms_end = unsafe { strstr(cms_start, c"%s".as_ptr()) };
    let mut cms_elen: size_t = 0;
    if !cms_end.is_null() {
        cms_elen = cms_slen.wrapping_sub(gap(cms_end, cms_start));
        cms_slen = gap(cms_end, cms_start);
        while cms_slen > 0 && ascii_iswhite(at(cms_start.wrapping_add(cms_slen - 1))) {
            cms_slen -= 1;
        }
        let s = skip_ws(cms_end.wrapping_offset(2));
        cms_elen = cms_elen.wrapping_sub(gap(s, cms_end));
        cms_end = s;
    }
    parse_marker(Win::current());

    // Each half of 'commentstring' is only removed once.
    let mut did1 = false;
    let mut did2 = false;
    let mut s = str;
    while at(s) != NUL {
        let mut len: size_t = 0;
        if ncmp(
            s,
            Win::current().w_onebuf_opt.wo_fmr,
            foldstartmarkerlen.get(),
        ) {
            len = foldstartmarkerlen.get();
        } else if ncmp(s, foldendmarker.get(), foldendmarkerlen.get()) {
            len = foldendmarkerlen.get();
        }
        if len > 0 {
            // A numbered marker, and any comment opener it sits behind.
            if ascii_isdigit(at(s.wrapping_add(len))) {
                len += 1;
            }
            let mut p = s;
            while p > str && ascii_iswhite(at(p.wrapping_offset(-1))) {
                p = p.wrapping_offset(-1);
            }
            let opener = p.wrapping_offset(-(cms_slen as isize));
            if p >= str.wrapping_add(cms_slen) && ncmp(opener, cms_start, cms_slen) {
                len = len.wrapping_add(gap(s, p).wrapping_add(cms_slen));
                s = opener;
            }
        } else if !cms_end.is_null() {
            if !did1 && cms_slen > 0 && ncmp(s, cms_start, cms_slen) {
                len = cms_slen;
                did1 = true;
            } else if !did2 && cms_elen > 0 && ncmp(s, cms_end, cms_elen) {
                len = cms_elen;
                did2 = true;
            }
        }
        if len != 0 {
            while ascii_iswhite(at(s.wrapping_add(len))) {
                len += 1;
            }
            // SAFETY: `s + len` is inside `str`, so the tail from there --
            // its terminator included -- fits where `s` is.
            unsafe {
                let tail = s.add(len);
                s.cast::<u8>()
                    .copy_from(tail.cast(), cstr::bytes_at(tail).len() + 1)
            };
        } else {
            // SAFETY: `s` is on a character of `str`.
            s = s.wrapping_offset(unsafe { utfc_ptr2len(s) } as isize);
        }
    }
}
