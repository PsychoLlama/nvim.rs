//! The callbacks for the options holding a format string, and for the
//! session/history/shell specs alongside them.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;
use std::ffi::CString;

use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int, transchar_byte};
use crate::drawscreen::comp_col;
use crate::main::{
    e_invalid_format_string_single_percent_s, p_ruf, p_shada, ru_wid, ssop_flags, stl_syntax,
};
use crate::memory::{xfree, xstrdup};
use crate::message::{verbose_open, verbose_stop};
use crate::option::{answer_err, did_set_title, get_option_default, p_vfile};
use crate::options::{kOptSsopFlagCurdir, kOptSsopFlagSesdir, kOptStatusline, opt_ssop_values};
use crate::os::cshim::gettext;
use crate::shada::get_shada_parameter;
use crate::strings::{vim_snprintf, vim_strchr};
use crate::types::{NUL, OptionSetFlags, StlSyntax, linenr_T, optset_T};
use crate::winfloat::win_config_float;

use super::frame::{errbuf, invalid, old_value, varp, win};
use super::{
    SHM_ALL, check_stl_option, did_set_option_listflag, did_set_str_generic, illegal_char,
    opt_strings_mask,
};

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_iconstring(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_titleiconstring(args, StlSyntax::ICON) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_titlestring(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_titleiconstring(args, StlSyntax::TITLE) }
}

/// 'title' and 'icon' strings are only run through the statusline formatter
/// when they contain a `%` *and* that format is valid; otherwise they are
/// shown literally, so a bad format is not an error here.
///
/// # Safety
/// `args` points at the option table's call frame.
pub(crate) unsafe fn did_set_titleiconstring(
    args: *mut optset_T,
    flagval: StlSyntax,
) -> *const c_char {
    // SAFETY: the frame's value is a C string.
    let value = unsafe { *varp(args) };
    // SAFETY: as above; the checker walks it to its terminator.
    let formatted = unsafe {
        !vim_strchr(value, c_int::from(b'%')).is_null() && check_stl_option(value).is_none()
    };
    let mut syntax = stl_syntax.get();
    if formatted {
        syntax |= flagval;
    } else {
        syntax.clear(flagval);
    }
    stl_syntax.set(syntax);
    did_set_title();
    ptr::null()
}

/// An option's value as bytes.
///
/// Safe because every option of string type holds a NUL-terminated string
/// from the moment the option table is initialised.
fn opt_bytes<'a>(s: *const c_char) -> &'a [u8] {
    // SAFETY: the invariant above.
    unsafe { CStr::from_ptr(s) }.to_bytes()
}

/// Check `'rulerformat'` as a whole.
fn check_ruf() -> Option<CString> {
    // SAFETY: the option's own value.
    unsafe { check_stl_option(p_ruf.get()) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_rulerformat(args: *mut optset_T) -> *const c_char {
    unsafe { answer_err(args, did_set_statustabline_rulerformat(args, true, false)) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_statuscolumn(args: *mut optset_T) -> *const c_char {
    unsafe { answer_err(args, did_set_statustabline_rulerformat(args, false, true)) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_statusline(args: *mut optset_T) -> *const c_char {
    unsafe { answer_err(args, did_set_statustabline_rulerformat(args, false, false)) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_tabline(args: *mut optset_T) -> *const c_char {
    unsafe { answer_err(args, did_set_statustabline_rulerformat(args, false, false)) }
}

/// The shared check for every option holding a 'statusline' format:
/// 'statusline', 'tabline', 'winbar', 'statuscolumn' and 'rulerformat'.
///
/// Three of them need something extra. 'rulerformat' may open with
/// `%<width>(`, which reserves that many columns on the last line.
/// 'statuscolumn' caches a number width per window, which a new format
/// invalidates. And an empty *global* 'statusline' means "use the built-in
/// one", so it is replaced by the default rather than left blank.
///
/// A format that opens with `%!` is an expression producing the real
/// format, so there is nothing to check until it is evaluated.
///
/// # Safety
/// `args` points at the option table's call frame.
pub(crate) unsafe fn did_set_statustabline_rulerformat(
    args: *mut optset_T,
    rulerformat: bool,
    statuscolumn: bool,
) -> Option<CString> {
    let (wp, varp) = unsafe { (win(args), varp(args)) };
    if rulerformat {
        ru_wid.set(0);
    } else if statuscolumn {
        // SAFETY: the frame's window.
        unsafe { (*wp).w_nrwidth_line_count = 0 as linenr_T };
    }

    // SAFETY: the frame and its C string value.
    let mut s = unsafe { *varp };
    let (idx, flags) = unsafe { ((*args).os_idx, (*args).os_flags) };
    let is_stl = idx as c_int == kOptStatusline as c_int;
    let global = flags.has(OptionSetFlags::GLOBAL) || !flags.has(OptionSetFlags::LOCAL);
    if is_stl && global && unsafe { c_int::from(*s) } == NUL {
        let mut expansion = None;
        let default = get_option_default(idx, flags, &mut expansion)
            .as_string()
            .expect("every option reaching here is a string option");
        // SAFETY: the option's own variable.
        unsafe { xfree((*varp).cast::<c_void>()) };
        unsafe { *varp = xstrdup(default.data()) };
        s = unsafe { *varp };
    }
    // A floating window's status line is part of its frame.
    if is_stl && !wp.is_null() && unsafe { (*wp).w_floating } {
        // SAFETY: the frame's window and its own configuration.
        unsafe { win_config_float(crate::winlayer::Win::new(wp), (*wp).w_config.clone()) };
    }

    let mut errmsg = None;
    let text = opt_bytes(s);
    if rulerformat && text.first() == Some(&b'%') {
        // Step past the `%` and an optional `-`; the width itself is read
        // with `getdigits_int`, whose overflow behaviour is what decides
        // that an absurd width is no width at all.
        let at = 1 + usize::from(text.get(1) == Some(&b'-'));
        // SAFETY: `at` is at most the terminator's index.
        let mut p = unsafe { s.add(at) };
        // SAFETY: `p` is a C string, and the walk stops at its terminator.
        let wid = unsafe { getdigits_int(&raw mut p, true, 0) };
        if wid != 0 && opt_bytes(p).first() == Some(&b'(') && {
            errmsg = check_ruf();
            errmsg.is_none()
        } {
            ru_wid.set(wid);
        } else if text.get(1) != Some(&b'!') {
            // Not a width group and not an expression: check the whole
            // format after all.
            errmsg = check_ruf();
        }
    } else if rulerformat || text.first() != Some(&b'%') || text.get(1) != Some(&b'!') {
        // SAFETY: the frame's own C string value.
        errmsg = unsafe { check_stl_option(s) };
    }
    if rulerformat && errmsg.is_none() {
        // The ruler's width decides where the last line's columns start.
        // SAFETY: recomputes a global from the editor's own state.
        unsafe { comp_col() };
    }
    errmsg
}

/// 'sessionoptions' cannot ask for both "curdir" and "sesdir".
///
/// The check runs after the mask has already been rebuilt, so rejecting the
/// value means rebuilding the mask from the old one — the caller restores
/// the string but not anything derived from it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_sessionoptions(args: *mut optset_T) -> *const c_char {
    let errmsg = unsafe { did_set_str_generic(args) };
    if !errmsg.is_null() {
        return errmsg;
    }
    let both = kOptSsopFlagCurdir as c_uint | kOptSsopFlagSesdir as c_uint;
    if ssop_flags.get() & both == both {
        // The caller only restores the string, so put the old value's mask
        // back here. A value that does not parse leaves the mask alone.
        // SAFETY: the frame's old value is a C string.
        if let Some(mask) = unsafe { opt_strings_mask(old_value(args), &opt_ssop_values, true) } {
            ssop_flags.set(mask);
        }
        return invalid();
    }
    ptr::null()
}

/// 'shada' is a comma-separated list of one-letter items, most of which
/// take a number. The value is walked here rather than by the generic
/// flag-letter check because each letter decides what may follow it.
///
/// The one-letter items 'shada' may name.
const SHADA_ITEMS: &[u8] = b"!\"%'/:<@cfhnrs";

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_shada(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's error buffer and the option's own C string value.
    let (buf, buflen) = unsafe { errbuf(args) };
    // SAFETY: the option's own value, which is NUL-terminated.
    let value = unsafe { CStr::from_ptr(p_shada.get()) }.to_bytes();
    // Reading past the end answers the terminator, as walking the C string
    // does.
    let at = |i: usize| value.get(i).copied().unwrap_or(0);
    let mut i = 0;
    while at(i) != 0 {
        let item = at(i);
        if !SHADA_ITEMS.contains(&item) {
            // SAFETY: the frame's error buffer, with its own length.
            return unsafe { illegal_char(buf, buflen, c_int::from(item)) };
        }
        if item == b'n' {
            break; // The file name is always last, and takes the rest.
        } else if item == b'r' {
            // A removable-media path runs to the next comma.
            i += 1;
            while at(i) != 0 && at(i) != b',' {
                i += 1;
            }
        } else if item == b'%' {
            // The buffer-list count is optional.
            i += 1;
            while ascii_isdigit(c_int::from(at(i))) {
                i += 1;
            }
        } else if matches!(item, b'!' | b'h' | b'c') {
            i += 1; // Takes nothing.
        } else {
            // Everything else must have a number.
            i += 1;
            while ascii_isdigit(c_int::from(at(i))) {
                i += 1;
            }
            if !ascii_isdigit(c_int::from(at(i - 1))) {
                if buf.is_null() {
                    return c"".as_ptr();
                }
                // SAFETY: the frame's error buffer, with its own length,
                // and a one-string format.
                let byte = c_int::from(at(i - 1));
                let fmt = gettext(c"E526: Missing number after <%s>");
                unsafe { vim_snprintf(buf, buflen, fmt.as_ptr(), transchar_byte(byte).as_ptr()) };
                return buf;
            }
        }
        if at(i) == b',' {
            i += 1;
        } else if at(i) != 0 {
            return if buf.is_null() {
                c"".as_ptr()
            } else {
                c"E527: Missing comma".as_ptr()
            };
        }
    }
    // The ' item, how many files to remember marks for, is required.
    // SAFETY: reads the option's own parsed value.
    if !value.is_empty() && unsafe { get_shada_parameter(c_int::from(b'\'')) } < 0 {
        return c"E528: Must specify a ' value".as_ptr();
    }
    ptr::null()
}

/// 'shellpipe' and 'shellredir' are printf-style: at most one `%s`, and a
/// `%` has to be followed by something.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_shellpipe_redir(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame, and its new value is a C string.
    let new = unsafe { (*args).os_newval }
        .as_string()
        .expect("the table installs this callback on a string option only");
    let value = unsafe { CStr::from_ptr(new.data()) }.to_bytes();
    let bad = e_invalid_format_string_single_percent_s
        .as_ptr()
        .cast::<c_char>();
    let mut seen = false;
    let mut at = 0;
    while at < value.len() {
        if value[at] == b'%' {
            match value.get(at + 1) {
                None => return bad,
                Some(&b'%') => at += 1,
                Some(&b's') if !seen => {
                    seen = true;
                    at += 1;
                }
                _ => return bad,
            }
        }
        at += 1;
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_shortmess(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = unsafe { errbuf(args) };
    unsafe { did_set_option_listflag(*varp(args), SHM_ALL.as_ptr(), buf, len) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_verbosefile(_args: *mut optset_T) -> *const c_char {
    // SAFETY: closes and reopens this process's own log file.
    unsafe { verbose_stop() };
    if c_int::from(unsafe { *p_vfile.get() }) != NUL && unsafe { verbose_open() }.is_err() {
        return invalid();
    }
    ptr::null()
}
