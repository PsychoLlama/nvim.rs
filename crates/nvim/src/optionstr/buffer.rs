//! The callbacks for options that decide how a buffer's text is read,
//! written and understood.
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

use crate::global_cell::{Field, field};
use crate::strings::has_bytes;
use core::ffi::{CStr, c_int, c_uint};

use crate::ascii::ascii_isdigit;
use crate::autocmd::check_ei;
use crate::charset::{buf_init_chartab, check_isopt};
use crate::diff::{diffanchors_changed, diffopt_changed};
use crate::digraph::keymap_init;
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_later, redraw_later, status_redraw_buf,
};
use crate::fold::{
    fold_update_all, foldmethod_is_diff, foldmethod_is_expr, foldmethod_is_indent,
    foldmethod_is_marker, new_fold_level,
};
use crate::guard::secure;
use crate::indent::{TabStopArray, replace_tabstop_array, tabstop_set};
use crate::indent_c::parse_cino;
use crate::mark::fresh_mark;
use crate::mbyte::canonize;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memline::ml_setflags;
use crate::message::{e_modifiable, e_unsupportedoption};
use crate::option::option_var;
use crate::option::vars::{P_BS, bkc_flags, p_bex, p_enc, p_pm};
use crate::option::{
    get_fileformat, redraw_titles, set_iminsert_global, set_imsearch_global, set_option_direct,
    skip_option_separator,
};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes, kOptComments, kOptEncoding, kOptEventignorewin,
    kOptFileencoding, kOptIskeyword, opt_bh_values, opt_bkc_values, opt_bt_values,
};
use crate::spell::spell_reload;
use crate::types::{
    Buffer, ColNr, FileMarkView, OptError, OptInt, OptSet, OptVal, OptionSetFlags, Pos,
};
use crate::window::global_stl_height;

use super::OptStringRef;
use super::frame::{invalid, new_bytes, new_value, old_bytes, varp, win};
use super::{
    B_IMODE_LMAP, B_IMODE_NONE, B_IMODE_USE_INSERT, COM_ALL, CPO_VI, EOL_MAC, FO_ALL, SID_NONE,
    did_set_opt_flags, did_set_optexpr, did_set_option_listflag, did_set_str_generic,
    e_backupext_and_patchmode_are_equal, e_comma_required, illegal_char, opt_strings_mask,
    opt_strings_ok, valid_filetype,
};

/// 'backspace' is a word list, except that the number 2 is also accepted
/// and means everything but "nostop".
pub fn did_set_backspace(args: &mut OptSet) -> Result<(), OptError> {
    if ascii_isdigit(c_int::from(P_BS.first_byte())) {
        if P_BS.first_byte() != b'2' {
            return invalid();
        }
        return Ok(());
    }
    did_set_str_generic(args)
}

/// 'backupcopy' has to name exactly one of "yes", "no" and "auto"; the
/// other words only qualify that choice.
pub fn did_set_backupcopy(args: &mut OptSet) -> Result<(), OptError> {
    let (mut buf, opt_flags) = (args.os_buf, args.os_flags);
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    // The variable being set is the buffer's own copy for `:setlocal` and
    // the global one otherwise -- which is the value the frame holds.
    let value = new_bytes(args);
    if !local && !opt_flags.has(OptionSetFlags::GLOBAL) {
        // A plain `:set` drops the buffer's own answer.
        buf.b_bkc_flags = 0 as c_uint;
    }
    let mut store = |mask: c_uint| {
        if local {
            buf.b_bkc_flags = mask;
        } else {
            bkc_flags.set(mask);
        }
    };

    // An empty buffer-local value means "no override", not "no words".
    if local && value.is_empty() {
        store(0 as c_uint);
        return Ok(());
    }
    let Some(mask) = opt_strings_mask(value, &opt_bkc_values, true) else {
        return invalid();
    };
    let named = [kOptBkcFlagAuto, kOptBkcFlagYes, kOptBkcFlagNo]
        .into_iter()
        .filter(|word| mask & *word as c_uint != 0)
        .count();
    if named != 1 {
        // Nothing was stored, so the mask still describes the old value --
        // which is what upstream re-parses it to get back.
        return invalid();
    }
    store(mask);
    Ok(())
}

/// 'backupext' and 'patchmode' both rename a file out of the way, so they
/// cannot be the same — a leading dot is not part of the comparison.
pub fn did_set_backupext_or_patchmode(_args: &mut OptSet) -> Result<(), OptError> {
    let undotted = |value: &CStr| {
        let bytes = value.to_bytes();
        bytes.strip_prefix(b".").unwrap_or(bytes).to_vec()
    };
    if p_bex(undotted) == p_pm(undotted) {
        return Err((e_backupext_and_patchmode_are_equal).into());
    }
    Ok(())
}

/// 'bufhidden' is checked as the buffer holds it, whichever copy was set.
pub fn did_set_bufhidden(args: &mut OptSet) -> Result<(), OptError> {
    did_set_opt_flags(args.os_buf.b_p_bh.bytes(), &opt_bh_values, false)
}

/// 'buftype' cannot be changed into or out of "terminal": that is decided
/// by whether the buffer actually has a terminal attached.
pub fn did_set_buftype(args: &mut OptSet) -> Result<(), OptError> {
    let (mut buf, mut wp) = (args.os_buf, win(args));
    // Checked as the buffer holds it, whichever copy was set. Only the first
    // letter is ever distinguishing.
    let (first, listed) = {
        let value = buf.b_p_bt.bytes();
        let first = value.first().copied().unwrap_or(0);
        (first, opt_strings_ok(value, &opt_bt_values, false))
    };
    let has_terminal = !buf.terminal.is_null();
    if has_terminal != (first == b't') || !listed {
        return invalid();
    }

    if first == b'p' {
        // A prompt buffer has no comment leaders, and its prompt starts at
        // the end of what is there now.
        set_option_direct(
            kOptComments,
            OptVal::static_string(c""),
            OptionSetFlags::LOCAL,
            SID_NONE,
        );
        let next_prompt = Pos {
            lnum: buf.b_ml.ml_line_count,
            col: buf.b_prompt_start.mark.col,
            coladd: 0 as ColNr,
        };
        buf.b_prompt_start = fresh_mark(next_prompt, 0, FileMarkView::NONE);
    }

    if wp.w_status_height != 0 || global_stl_height() != 0 {
        wp.w_redr_status = true;
        redraw_later(wp, UPD_VALID);
    }
    buf.b_help = first == b'h';
    redraw_titles();
    Ok(())
}

pub fn did_set_cinoptions(args: &mut OptSet) -> Result<(), OptError> {
    parse_cino(args.os_buf);
    Ok(())
}

/// 'comments' is a comma-separated list of `{flags}:{leader}` parts.
///
/// The order the two messages come out in is upstream's and is load
/// bearing: an illegal flag letter still falls through to the colon and
/// length checks, so a bad one-letter value is reported as E525 rather than
/// as the illegal character.
pub fn did_set_comments(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_bytes(args);
    // Reading past the end answers the terminator, as walking the C string
    // does.
    let at = |i: usize| value.get(i).copied().unwrap_or(0);
    let mut errmsg: Result<(), OptError> = Ok(());
    let mut s = 0;
    while at(s) != 0 {
        // The flag letters, up to the colon.
        while at(s) != 0 && at(s) != b':' {
            let flag = at(s);
            if !COM_ALL.to_bytes().contains(&flag) && !flag.is_ascii_digit() && flag != b'-' {
                errmsg = Err(illegal_char(c_int::from(flag)));
                break;
            }
            s += 1;
        }
        let at_colon = at(s);
        s += 1;
        if at_colon == 0 {
            errmsg = Err((c"E524: Missing colon").into());
        } else if at(s) == b',' || at(s) == 0 {
            errmsg = Err((c"E525: Zero length string").into());
        }
        if errmsg.is_err() {
            break;
        }
        // The leader, in which a backslash escapes the next byte.
        while at(s) != 0 && at(s) != b',' {
            if at(s) == b'\\' && at(s + 1) != 0 {
                s += 1;
            }
            s += 1;
        }
        s = value.len() - skip_option_separator(value.get(s..).unwrap_or_default()).len();
    }
    errmsg
}

pub fn did_set_commentstring(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_value(args);
    if !value.is_empty() && !has_bytes(value.as_cstr(), b"%s") {
        return Err((c"E537: 'commentstring' must be empty or contain %s").into());
    }
    Ok(())
}

pub fn did_set_cpoptions(args: &mut OptSet) -> Result<(), OptError> {
    did_set_option_listflag(new_bytes(args), CPO_VI)
}

pub fn did_set_diffanchors(args: &mut OptSet) -> Result<(), OptError> {
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    if diffanchors_changed(local).is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_diffopt(_args: &mut OptSet) -> Result<(), OptError> {
    if diffopt_changed().is_err() {
        return invalid();
    }
    Ok(())
}

/// 'encoding', 'fileencoding' and 'termencoding' share a callback. Only
/// UTF-8 is supported for the internal 'encoding'; the others are
/// canonicalised in place, which is why the variable is rewritten here.
pub fn did_set_encoding(args: &mut OptSet) -> Result<(), OptError> {
    let (buf, varp, opt_flags, idx) = (args.os_buf, varp(args), args.os_flags, args.os_idx);
    // 'fileencoding' is the buffer-local one of the three; the other two
    // ('encoding' and 'makeencoding') are global and skip this block.
    if idx == kOptFileencoding {
        if buf.b_p_ma == 0 && opt_flags != OptionSetFlags::GLOBAL {
            return Err((e_modifiable).into());
        }
        // 'fileencoding' is one encoding, not a list.
        if new_bytes(args).contains(&b',') {
            return invalid();
        }
        redraw_titles();
        ml_setflags(buf);
    }

    varp.set(canonize(new_bytes(args)));
    if idx == kOptEncoding {
        if p_enc(|value| value.to_bytes() != b"utf-8") {
            return Err((e_unsupportedoption).into());
        }
        spell_reload();
    }
    Ok(())
}

pub fn did_set_eventignore(args: &mut OptSet) -> Result<(), OptError> {
    let window_local = args.os_idx == kOptEventignorewin;
    if check_ei(new_value(args).as_cstr(), window_local).is_err() {
        return invalid();
    }
    Ok(())
}

pub fn did_set_fileformat(args: &mut OptSet) -> Result<(), OptError> {
    let (buf, opt_flags) = (args.os_buf, args.os_flags);
    // Changing a buffer's line endings changes its text.
    if buf.b_p_ma == 0 && !opt_flags.has(OptionSetFlags::GLOBAL) {
        return Err((e_modifiable).into());
    }
    did_set_str_generic(args)?;
    redraw_titles();
    ml_setflags(buf);
    // Only "mac" is drawn differently, so a redraw is needed when
    // entering or leaving it.
    if get_fileformat(buf) == EOL_MAC || old_bytes(args).first() == Some(&b'm') {
        redraw_buf_later(buf, UPD_NOT_VALID);
    }
    Ok(())
}

/// 'filetype' and 'syntax' fire an autocommand, and only when the value
/// really changed — which is what `os_value_changed` tells the caller.
pub fn did_set_filetype_or_syntax(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_value(args);
    if !valid_filetype(value.as_cstr()) {
        return invalid();
    }
    // The frame's own copies: the autocommand this reports to the caller
    // fires after the callback, and may set the option again.
    let changed = old_bytes(args) != value.as_bytes();
    args.os_value_changed = changed;
    args.os_value_checked = true;
    Ok(())
}

pub fn did_set_foldexpr(args: &mut OptSet) -> Result<(), OptError> {
    let _ = did_set_optexpr(args);
    let wp = win(args);
    if foldmethod_is_expr(wp) {
        fold_update_all(wp);
    }
    Ok(())
}

pub fn did_set_foldignore(args: &mut OptSet) -> Result<(), OptError> {
    let wp = win(args);
    if foldmethod_is_indent(wp) {
        fold_update_all(wp);
    }
    Ok(())
}

pub fn did_set_foldmarker(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_bytes(args);
    // Two markers separated by a comma, neither of them empty.
    let Some(comma) = value.iter().position(|&b| b == b',') else {
        return Err((e_comma_required).into());
    };
    if comma == 0 || comma + 1 == value.len() {
        return invalid();
    }
    let wp = win(args);
    if foldmethod_is_marker(wp) {
        fold_update_all(wp);
    }
    Ok(())
}

pub fn did_set_foldmethod(args: &mut OptSet) -> Result<(), OptError> {
    did_set_str_generic(args)?;
    let wp = win(args);
    fold_update_all(wp);
    // Diff folds are closed to whatever 'foldlevel' says as soon as
    // they exist.
    if foldmethod_is_diff(wp) {
        new_fold_level();
    }
    Ok(())
}

pub fn did_set_formatoptions(args: &mut OptSet) -> Result<(), OptError> {
    did_set_option_listflag(new_bytes(args), FO_ALL)
}

/// 'iskeyword' is one of the character-class options, except that the
/// global one only has to parse — no buffer's character table depends on
/// it.
pub fn did_set_iskeyword(args: &mut OptSet) -> Result<(), OptError> {
    let varp = varp(args);
    if varp != option_var(kOptIskeyword).string_var() {
        return did_set_isopt(args);
    }
    if check_isopt(new_bytes(args)).is_err() {
        return invalid();
    }
    Ok(())
}

/// The shared callback for 'isident', 'isfname', 'isprint' and the
/// buffer-local 'iskeyword': rebuild the buffer's character table, and ask
/// the caller to put the old one back if it does not parse.
pub fn did_set_isopt(args: &mut OptSet) -> Result<(), OptError> {
    if !buf_init_chartab(args.os_buf, true) {
        args.os_restore_chartab = true;
        return invalid();
    }
    Ok(())
}

/// Load a 'keymap' and switch the buffer's language modes to it.
///
/// The keymap file is sourced, which is why 'secure' is lifted for the
/// duration: the file is part of the runtime, not user input.
pub fn did_set_keymap(args: &mut OptSet) -> Result<(), OptError> {
    let (mut buf, opt_flags) = (args.os_buf, args.os_flags);
    if !valid_filetype(new_value(args).as_cstr()) {
        return invalid();
    }

    let secure_save = secure.get();
    secure.set(0);
    // Sources the keymap file named by the option.
    let errmsg = keymap_init();
    secure.set(secure_save);
    args.os_value_checked = true;
    errmsg?;

    if !buf.b_p_keymap.bytes().is_empty() {
        buf.b_p_iminsert = OptInt::from(B_IMODE_LMAP);
        // 'imsearch' at -1 means "follow 'iminsert'", and stays that
        // way.
        if buf.b_p_imsearch != OptInt::from(B_IMODE_USE_INSERT) {
            buf.b_p_imsearch = OptInt::from(B_IMODE_LMAP);
        }
    } else {
        if buf.b_p_iminsert == OptInt::from(B_IMODE_LMAP) {
            buf.b_p_iminsert = OptInt::from(B_IMODE_NONE);
        }
        if buf.b_p_imsearch == OptInt::from(B_IMODE_LMAP) {
            buf.b_p_imsearch = OptInt::from(B_IMODE_USE_INSERT);
        }
    }
    if !opt_flags.has(OptionSetFlags::LOCAL) {
        set_iminsert_global(buf);
        set_imsearch_global(buf);
    }
    status_redraw_buf(buf);
    Ok(())
}

pub fn did_set_lispoptions(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_bytes(args);
    if !value.is_empty() && value != b"expr:0" && value != b"expr:1" {
        return invalid();
    }
    Ok(())
}

/// 'matchpairs' is a comma-separated list of `{open}:{close}` pairs. The
/// separator has to be a single-byte colon, but either character of a pair
/// may be multibyte.
pub fn did_set_matchpairs(args: &mut OptSet) -> Result<(), OptError> {
    let value = new_bytes(args);
    // Reading past the end answers the terminator, as walking the C string
    // does; every slice below starts before it.
    let at = |i: usize| value.get(i).copied().unwrap_or(0);
    let mut p = 0;
    while at(p) != 0 {
        let mut separator = -1;
        let mut close = -1;
        p += c_cluster_len(&value[p..]);
        if at(p) != 0 {
            separator = c_int::from(at(p));
            p += 1;
        }
        if at(p) != 0 {
            close = char_at(&value[p..]);
            p += c_cluster_len(&value[p..]);
        }
        if separator != c_int::from(b':') || close == -1 || (at(p) != 0 && at(p) != b',') {
            return invalid();
        }
        if at(p) == 0 {
            break;
        }
        p += 1;
    }
    Ok(())
}

pub fn did_set_varsofttabstop(args: &mut OptSet) -> Result<(), OptError> {
    did_set_vartabs(args, field!(Buffer, b_p_vsts_array))
}

pub fn did_set_vartabstop(args: &mut OptSet) -> Result<(), OptError> {
    let errmsg = did_set_vartabs(args, field!(Buffer, b_p_vts_array));
    if errmsg.is_ok() {
        // Indent folds are computed from the tab stops.
        let wp = win(args);
        if foldmethod_is_indent(wp) {
            fold_update_all(wp);
        }
    }
    errmsg
}

/// The shared check for 'varsofttabstop' and 'vartabstop': a comma-
/// separated list of numbers, parsed into the buffer's stop array, which
/// `array` names.
///
/// An empty value and a lone `0` both mean "no list", and free the array
/// rather than replacing it. The array is reached after the parse, which
/// may report, so nothing holds the buffer while it does.
fn did_set_vartabs(args: &OptSet, array: Field<Buffer, TabStopArray>) -> Result<(), OptError> {
    let mut buf = args.os_buf;
    let value = new_bytes(args);
    if value.is_empty() || value == b"0" {
        replace_tabstop_array(array.of(&mut buf), None);
        return Ok(());
    }
    // Digits and separating commas only; no empty item, and no leading
    // comma.
    let mut previous = b',';
    for &byte in value {
        if !byte.is_ascii_digit() && !(byte == b',' && previous != b',') {
            return invalid();
        }
        previous = byte;
    }
    // A value that does not parse leaves the old array in place.
    let Some(stops) = tabstop_set(value) else {
        return invalid();
    };
    *array.of(&mut buf) = stops;
    Ok(())
}
