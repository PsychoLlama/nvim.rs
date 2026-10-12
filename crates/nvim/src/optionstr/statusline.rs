//! The callbacks for the options holding a format string, and for the
//! session/history/shell specs alongside them.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::formatted;
use core::ffi::{c_int, c_uint};

use crate::ascii::ascii_isdigit;
use crate::charset::{getdigits_int_at, transchar_byte};
use crate::drawscreen::comp_col;
use crate::drawscreen::state::ru_wid;
use crate::memory::XString;
use crate::message::e_invalid_format_string_single_percent_s;
use crate::message::{verbose_open, verbose_stop};
use crate::option::vars::{P_SHADA, P_VFILE, p_ruf, ssop_flags};
use crate::option::{did_set_title, get_option_default};
use crate::options::{kOptSsopFlagCurdir, kOptSsopFlagSesdir, kOptStatusline, opt_ssop_values};
use crate::os::cshim::gettext;
use crate::shada::get_shada_parameter;
use crate::statusline::state::stl_syntax;
use crate::types::{LineNr, OptError, OptSet, OptionSetFlags, StlSyntax};
use crate::winfloat::win_config_float;

use super::frame::{invalid, new_bytes, old_bytes, varp, win};
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
    let value = new_bytes(args);
    let formatted = value.contains(&b'%') && check_stl_option(value).is_ok();
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

/// Check `'rulerformat'` as a whole.
fn check_ruf() -> Result<(), OptError> {
    p_ruf(|value| check_stl_option(value.to_bytes()))
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

    let (idx, flags) = (args.os_idx, args.os_flags);
    let is_stl = idx as c_int == kOptStatusline as c_int;
    let global = flags.has(OptionSetFlags::GLOBAL) || !flags.has(OptionSetFlags::LOCAL);
    // What is checked is what the variable holds: the frame's copy of the
    // new value, or the default an empty global 'statusline' is given.
    let default;
    let text = if is_stl && global && new_bytes(args).is_empty() {
        default = get_option_default(idx, flags)
            .as_string()
            .expect("every option reaching here is a string option")
            .clone()
            .into_xstring();
        varp.set(default.clone());
        &default[..]
    } else {
        new_bytes(args)
    };
    // A floating window's status line is part of its frame.
    if is_stl && wp.w_floating {
        win_config_float(wp, wp.w_config.clone());
    }

    let mut errmsg = Ok(());
    if rulerformat && text.first() == Some(&b'%') {
        // Step past the `%` and an optional `-`; the width itself is read
        // as `getdigits_int` reads it, whose overflow behaviour is what
        // decides that an absurd width is no width at all.
        let at = 1 + usize::from(text.get(1) == Some(&b'-'));
        let (wid, past) = getdigits_int_at(text, at, true, 0);
        if wid != 0 && text.get(past) == Some(&b'(') && {
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
        errmsg = check_stl_option(text);
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
        if let Some(mask) = opt_strings_mask(old_bytes(args), &opt_ssop_values, true) {
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
                let shown = transchar_byte(byte);
                let message = formatted!(OptError::ROOM, fmt.to_bytes(), cstr::in_chars(&shown));
                return Err(XString::from_vec(message).into());
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
    let value = new_bytes(args);
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
    did_set_option_listflag(new_bytes(args), SHM_ALL)
}

pub fn did_set_verbosefile(_args: &mut OptSet) -> Result<(), OptError> {
    verbose_stop();
    if P_VFILE.first_byte() != 0 && verbose_open().is_err() {
        return invalid();
    }
    Ok(())
}
