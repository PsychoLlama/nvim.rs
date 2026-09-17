//! The callbacks for the options holding a format string, and for the
//! session/history/shell specs alongside them.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::strings::has_char;
use core::ffi::{CStr, c_char, c_int, c_uint};

use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int, transchar_byte};
use crate::drawscreen::comp_col;
use crate::drawscreen::state::ru_wid;
use crate::memory::xstrdup;
use crate::message::e_invalid_format_string_single_percent_s;
use crate::message::{verbose_open, verbose_stop};
use crate::option::vars::p_vfile;
use crate::option::vars::{P_SHADA, p_ruf, ssop_flags};
use crate::option::{did_set_title, get_option_default};
use crate::options::{kOptSsopFlagCurdir, kOptSsopFlagSesdir, kOptStatusline, opt_ssop_values};
use crate::os::cshim::gettext;
use crate::shada::get_shada_parameter;
use crate::statusline::state::stl_syntax;
use crate::strings::vim_snprintf;
use crate::types::{LineNr, NUL, OptError, OptSet, OptionSetFlags, StlSyntax, size_t};
use crate::winfloat::win_config_float;

use super::frame::{formatted, invalid, old_value, varp, win};
use super::free_string_option;
use super::{
    SHM_ALL, check_stl_option, did_set_option_listflag, did_set_str_generic, illegal_char,
    opt_strings_mask,
};

pub fn did_set_iconstring(args: &mut OptSet) -> Result<(), OptError> {
    did_set_titleiconstring(args, StlSyntax::ICON)
}

pub fn did_set_titlestring(args: &mut OptSet) -> Result<(), OptError> {
    did_set_titleiconstring(args, StlSyntax::TITLE)
}

/// 'title' and 'icon' strings are only run through the statusline formatter
/// when they contain a `%` *and* that format is valid; otherwise they are
/// shown literally, so a bad format is not an error here.
pub(crate) fn did_set_titleiconstring(args: &OptSet, flagval: StlSyntax) -> Result<(), OptError> {
    // SAFETY: the frame's value is a C string.
    let value = unsafe { varp(args).get() };
    // SAFETY: as above; the checker walks it to its terminator.
    let formatted =
        unsafe { has_char(cstr::at(value), c_int::from(b'%')) && check_stl_option(value).is_ok() };
    let mut syntax = stl_syntax.get();
    if formatted {
        syntax |= flagval;
    } else {
        syntax.clear(flagval);
    }
    stl_syntax.set(syntax);
    did_set_title();
    Ok(())
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
fn check_ruf() -> Result<(), OptError> {
    // SAFETY: the option's own value.
    p_ruf(|value| unsafe { check_stl_option(value.as_ptr().cast_mut()) })
}

pub fn did_set_rulerformat(args: &mut OptSet) -> Result<(), OptError> {
    did_set_statustabline_rulerformat(args, true, false)
}

pub fn did_set_statuscolumn(args: &mut OptSet) -> Result<(), OptError> {
    did_set_statustabline_rulerformat(args, false, true)
}

pub fn did_set_statusline(args: &mut OptSet) -> Result<(), OptError> {
    did_set_statustabline_rulerformat(args, false, false)
}

pub fn did_set_tabline(args: &mut OptSet) -> Result<(), OptError> {
    did_set_statustabline_rulerformat(args, false, false)
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
pub(crate) fn did_set_statustabline_rulerformat(
    args: &OptSet,
    rulerformat: bool,
    statuscolumn: bool,
) -> Result<(), OptError> {
    let (mut wp, varp) = (win(args), varp(args));
    if rulerformat {
        ru_wid.set(0);
    } else if statuscolumn {
        wp.w_nrwidth_line_count = 0 as LineNr;
    }

    // SAFETY: the frame and its C string value.
    let mut s = unsafe { varp.get() };
    let (idx, flags) = (args.os_idx, args.os_flags);
    let is_stl = idx as c_int == kOptStatusline as c_int;
    let global = flags.has(OptionSetFlags::GLOBAL) || !flags.has(OptionSetFlags::LOCAL);
    if is_stl && global && unsafe { c_int::from(*s) } == NUL {
        let mut expansion = None;
        let default = get_option_default(idx, flags, &mut expansion)
            .as_string()
            .expect("every option reaching here is a string option");
        // SAFETY: the option's own variable. Replace and *then* free; see
        // `crate::optionstr::did_set_optexpr`.
        let old = unsafe { varp.replace(xstrdup(default.data())) };
        unsafe { free_string_option(old) };
        s = unsafe { varp.get() };
    }
    // A floating window's status line is part of its frame.
    if is_stl && wp.w_floating {
        win_config_float(wp, wp.w_config.clone());
    }

    let mut errmsg = Ok(());
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
            errmsg.is_ok()
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
    if rulerformat && errmsg.is_ok() {
        // The ruler's width decides where the last line's columns start.
        comp_col();
    }
    errmsg
}

/// 'sessionoptions' cannot ask for both "curdir" and "sesdir".
///
/// The check runs after the mask has already been rebuilt, so rejecting the
/// value means rebuilding the mask from the old one — the caller restores
/// the string but not anything derived from it.
pub fn did_set_sessionoptions(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
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
    Ok(())
}

/// 'shada' is a comma-separated list of one-letter items, most of which
/// take a number. The value is walked here rather than by the generic
/// flag-letter check because each letter decides what may follow it.
///
/// The one-letter items 'shada' may name.
const SHADA_ITEMS: &[u8] = b"!\"%'/:<@cfhnrs";

pub fn did_set_shada(_args: &mut OptSet) -> Result<(), OptError> {
    // A copy: the walk below outlives the projection's borrow.
    let shada = P_SHADA.get();
    let value = &*shada;
    // Reading past the end answers the terminator, as walking the C string
    // does.
    let at = |i: usize| value.get(i).copied().unwrap_or(0);
    let mut i = 0;
    while at(i) != 0 {
        let item = at(i);
        if !SHADA_ITEMS.contains(&item) {
            // SAFETY: the frame's error buffer, with its own length.
            return Err(illegal_char(c_int::from(item)));
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
                let byte = c_int::from(at(i - 1));
                let fmt = gettext(c"E526: Missing number after <%s>");
                // SAFETY: `formatted` hands the closure a buffer of the
                // size it passes on, and the format takes one string.
                return formatted(|buf| unsafe {
                    vim_snprintf(
                        buf,
                        OptError::ROOM as size_t,
                        fmt.as_ptr(),
                        transchar_byte(byte).as_ptr(),
                    );
                });
            }
        }
        if at(i) == b',' {
            i += 1;
        } else if at(i) != 0 {
            return Err(c"E527: Missing comma".into());
        }
    }
    // The ' item, how many files to remember marks for, is required.
    if !value.is_empty() && get_shada_parameter(c_int::from(b'\'')) < 0 {
        return Err((c"E528: Must specify a ' value").into());
    }
    Ok(())
}

/// 'shellpipe' and 'shellredir' are printf-style: at most one `%s`, and a
/// `%` has to be followed by something.
pub fn did_set_shellpipe_redir(args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the caller's frame, and its new value is a C string.
    let new = args
        .os_newval
        .as_string()
        .expect("the table installs this callback on a string option only");
    let value = unsafe { CStr::from_ptr(new.data()) }.to_bytes();
    let bad = || Err(e_invalid_format_string_single_percent_s.into());
    let mut seen = false;
    let mut at = 0;
    while at < value.len() {
        if value[at] == b'%' {
            match value.get(at + 1) {
                None => return bad(),
                Some(&b'%') => at += 1,
                Some(&b's') if !seen => {
                    seen = true;
                    at += 1;
                }
                _ => return bad(),
            }
        }
        at += 1;
    }
    Ok(())
}

pub fn did_set_shortmess(args: &mut OptSet) -> Result<(), OptError> {
    // SAFETY: the frame's own C string value.
    unsafe { did_set_option_listflag(varp(args).get(), SHM_ALL.as_ptr()) }
}

pub fn did_set_verbosefile(_args: &mut OptSet) -> Result<(), OptError> {
    verbose_stop();
    if p_vfile(|value| !value.is_empty()) && verbose_open().is_err() {
        return invalid();
    }
    Ok(())
}
