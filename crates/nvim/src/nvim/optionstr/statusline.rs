//! The callbacks for the options holding a format string, and for the
//! session/history/shell specs alongside them.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{getdigits_int, transchar_byte};
use crate::src::nvim::drawscreen::comp_col;
use crate::src::nvim::main::{
    e_invalid_format_string_single_percent_s, p_ruf, p_shada, ru_wid, ssop_flags, stl_syntax,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{verbose_open, verbose_stop};
use crate::src::nvim::option::{did_set_title, get_option_default, p_vfile};
use crate::src::nvim::options::{
    kOptSsopFlagCurdir, kOptSsopFlagSesdir, kOptStatusline, opt_ssop_values,
};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::shada::get_shada_parameter;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{linenr_T, optset_T};
use crate::src::nvim::winfloat::win_config_float;

use super::frame::{errbuf, invalid, old_value, varp, win};
use super::{
    FAIL, NUL, OPT_GLOBAL, OPT_LOCAL, SHM_ALL, STL_IN_ICON, STL_IN_TITLE, check_stl_option,
    did_set_option_listflag, did_set_str_generic, illegal_char, opt_strings_flags,
};

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_iconstring(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_titleiconstring(args, STL_IN_ICON) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_titlestring(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_titleiconstring(args, STL_IN_TITLE) }
}

/// 'title' and 'icon' strings are only run through the statusline formatter
/// when they contain a `%` *and* that format is valid; otherwise they are
/// shown literally, so a bad format is not an error here.
///
/// # Safety
/// `args` points at the option table's call frame.
pub(crate) unsafe fn did_set_titleiconstring(args: *mut optset_T, flagval: c_int) -> *const c_char {
    // SAFETY: the frame's value is a C string.
    unsafe {
        let value = *varp(args);
        let formatted =
            !vim_strchr(value, c_int::from(b'%')).is_null() && check_stl_option(value).is_null();
        if formatted {
            *stl_syntax.ptr() |= flagval;
        } else {
            *stl_syntax.ptr() &= !flagval;
        }
        did_set_title();
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_rulerformat(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_statustabline_rulerformat(args, true, false) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_statuscolumn(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_statustabline_rulerformat(args, false, true) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_statusline(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_statustabline_rulerformat(args, false, false) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_tabline(args: *mut optset_T) -> *const c_char {
    unsafe { did_set_statustabline_rulerformat(args, false, false) }
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
) -> *const c_char {
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
    let global = flags & OPT_GLOBAL as c_int != 0 || flags & OPT_LOCAL as c_int == 0;
    if is_stl && global && unsafe { c_int::from(*s) } == NUL {
        // SAFETY: the option's own variable, and the table's default for
        // it, which is a string.
        unsafe {
            xfree((*varp).cast::<c_void>());
            *varp = xstrdup(get_option_default(idx, flags).data.string.data);
            s = *varp;
        }
    }
    // A floating window's status line is part of its frame.
    if is_stl && !wp.is_null() && unsafe { (*wp).w_floating } {
        // SAFETY: the frame's window and its own configuration.
        unsafe { win_config_float(wp, (*wp).w_config) };
    }

    let mut errmsg: *const c_char = ptr::null();
    // SAFETY: `s` is a C string throughout.
    unsafe {
        if rulerformat && c_int::from(*s) == c_int::from(b'%') {
            s = s.add(1);
            if c_int::from(*s) == c_int::from(b'-') {
                s = s.add(1);
            }
            let wid = getdigits_int(&raw mut s, true, 0);
            if wid != 0 && c_int::from(*s) == c_int::from(b'(') && {
                errmsg = check_stl_option(p_ruf.get());
                errmsg.is_null()
            } {
                ru_wid.set(wid);
            } else if c_int::from(*(*varp).add(1)) != c_int::from(b'!') {
                // Not a width group and not an expression: check the whole
                // format after all.
                errmsg = check_stl_option(p_ruf.get());
            }
        } else if rulerformat
            || c_int::from(*s) != c_int::from(b'%')
            || c_int::from(*s.add(1)) != c_int::from(b'!')
        {
            errmsg = check_stl_option(s);
        }
    }
    if rulerformat && errmsg.is_null() {
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
pub unsafe extern "C" fn did_set_sessionoptions(args: *mut optset_T) -> *const c_char {
    let errmsg = unsafe { did_set_str_generic(args) };
    if !errmsg.is_null() {
        return errmsg;
    }
    let both = kOptSsopFlagCurdir as c_uint | kOptSsopFlagSesdir as c_uint;
    if ssop_flags.get() & both == both {
        // SAFETY: the frame's old value is a C string, and the table's own
        // word list and mask.
        unsafe {
            opt_strings_flags(
                old_value(args),
                opt_ssop_values.ptr().cast::<*const c_char>(),
                ssop_flags.ptr(),
                true,
            );
        }
        return invalid();
    }
    ptr::null()
}

/// 'shada' is a comma-separated list of one-letter items, most of which
/// take a number. The value is walked here rather than by the generic
/// flag-letter check because each letter decides what may follow it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_shada(args: *mut optset_T) -> *const c_char {
    let (buf, buflen) = unsafe { errbuf(args) };
    // SAFETY: the option's own C string value, walked to its terminator.
    unsafe {
        let mut s = p_shada.get();
        while *s != 0 {
            if vim_strchr(c"!\"%'/:<@cfhnrs".as_ptr(), c_int::from(*s as u8)).is_null() {
                return illegal_char(buf, buflen, c_int::from(*s as u8));
            }
            if *s == b'n' as c_char {
                break; // The file name is always last, and takes the rest.
            } else if *s == b'r' as c_char {
                // A removable-media path runs to the next comma.
                while {
                    s = s.add(1);
                    *s != 0 && *s != b',' as c_char
                } {}
            } else if *s == b'%' as c_char {
                // The buffer-list count is optional.
                while {
                    s = s.add(1);
                    ascii_isdigit(c_int::from(*s))
                } {}
            } else if *s == b'!' as c_char || *s == b'h' as c_char || *s == b'c' as c_char {
                s = s.add(1); // Takes nothing.
            } else {
                // Everything else must have a number.
                while {
                    s = s.add(1);
                    ascii_isdigit(c_int::from(*s))
                } {}
                if !ascii_isdigit(c_int::from(*s.sub(1))) {
                    if buf.is_null() {
                        return c"".as_ptr();
                    }
                    vim_snprintf(
                        buf,
                        buflen,
                        gettext(c"E526: Missing number after <%s>".as_ptr()),
                        transchar_byte(c_int::from(*s.sub(1) as u8)),
                    );
                    return buf;
                }
            }
            if *s == b',' as c_char {
                s = s.add(1);
            } else if *s != 0 {
                return if buf.is_null() {
                    c"".as_ptr()
                } else {
                    c"E527: Missing comma".as_ptr()
                };
            }
        }
        // The ' item, how many files to remember marks for, is required.
        if *p_shada.get() != 0 && get_shada_parameter(c_int::from(b'\'')) < 0 {
            return c"E528: Must specify a ' value".as_ptr();
        }
    }
    ptr::null()
}

/// 'shellpipe' and 'shellredir' are printf-style: at most one `%s`, and a
/// `%` has to be followed by something.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_shellpipe_redir(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's new value is a C string.
    let value = unsafe { CStr::from_ptr((*args).os_newval.string.data) }.to_bytes();
    let bad = e_invalid_format_string_single_percent_s
        .ptr()
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
pub unsafe extern "C" fn did_set_shortmess(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame, its value and its error buffer.
    unsafe {
        let (buf, len) = errbuf(args);
        did_set_option_listflag(*varp(args), SHM_ALL.as_ptr(), buf, len)
    }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe extern "C" fn did_set_verbosefile(_args: *mut optset_T) -> *const c_char {
    // SAFETY: closes and reopens this process's own log file.
    unsafe {
        verbose_stop();
        if c_int::from(*p_vfile.get()) != NUL && verbose_open() == FAIL {
            return invalid();
        }
    }
    ptr::null()
}
