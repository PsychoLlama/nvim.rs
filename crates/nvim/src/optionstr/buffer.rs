//! The callbacks for options that decide how a buffer's text is read,
//! written and understood.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::strings::has_bytes;
use crate::strings::has_char;
use core::ffi::{CStr, c_char, c_int, c_uchar, c_uint, c_void};
use core::ptr;

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
use crate::indent::tabstop_set;
use crate::indent_c::parse_cino;
use crate::mark::free_fmark;
use crate::mbyte::{enc_canonize, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_setflags;
use crate::memory::xfree;
use crate::message::{e_modifiable, e_unsupportedoption};
use crate::option::option_var;
use crate::option::vars::{bkc_flags, p_bex, p_bkc, p_bs, p_enc, p_pm};
use crate::option::{
    get_fileformat, redraw_titles, set_iminsert_global, set_imsearch_global, set_option_direct,
    skip_to_option_part,
};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes, kOptComments, kOptEncoding, kOptFileencoding,
    kOptIskeyword, opt_bh_values, opt_bkc_values, opt_bt_values,
};
use crate::os::time::os_time;
use crate::spell::spell_reload;
use crate::strings::vim_strchr;
use crate::types::{
    AdditionalData, ColNr, FileMark, FileMarkView, LineNr, NUL, OptInt, OptSet, OptVal,
    OptionSetFlags, Pos, String_0,
};
use crate::window::global_stl_height;

use super::frame::{errbuf, invalid, old_value, varp, win};
use super::{
    B_IMODE_LMAP, B_IMODE_NONE, B_IMODE_USE_INSERT, COM_ALL, CPO_VI, EOL_MAC, FO_ALL, SID_NONE,
    did_set_opt_flags, did_set_optexpr, did_set_option_listflag, did_set_str_generic,
    e_backupext_and_patchmode_are_equal, e_comma_required, illegal_char, opt_strings_mask,
    opt_strings_ok, valid_filetype,
};
use crate::pos::MAXLNUM;

/// 'backspace' is a word list, except that the number 2 is also accepted
/// and means everything but "nostop".
pub fn did_set_backspace(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the option's own C string value.
    if unsafe { ascii_isdigit(c_int::from(*p_bs.get())) } {
        if unsafe { *p_bs.get() } != b'2' as c_char {
            return invalid();
        }
        return None;
    }
    did_set_str_generic(args)
}

/// 'backupcopy' has to name exactly one of "yes", "no" and "auto"; the
/// other words only qualify that choice.
pub fn did_set_backupcopy(args: &mut OptSet) -> Option<&CStr> {
    let (mut buf, opt_flags) = (args.os_buf, args.os_flags);
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    let value = if local {
        buf.b_p_bkc
    } else {
        if !opt_flags.has(OptionSetFlags::GLOBAL) {
            // A plain `:set` drops the buffer's own answer.
            buf.b_bkc_flags = 0 as c_uint;
        }
        p_bkc.get()
    };
    let mut store = |mask: c_uint| {
        if local {
            buf.b_bkc_flags = mask;
        } else {
            bkc_flags.set(mask);
        }
    };

    // An empty buffer-local value means "no override", not "no words".
    // SAFETY: an option's value is a C string.
    if local && unsafe { c_int::from(*value) } == NUL {
        store(0 as c_uint);
        return None;
    }
    // SAFETY: a C string, against the table's own word list.
    let Some(mask) = (unsafe { opt_strings_mask(value, &opt_bkc_values, true) }) else {
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
    None
}

/// 'backupext' and 'patchmode' both rename a file out of the way, so they
/// cannot be the same — a leading dot is not part of the comparison.
pub fn did_set_backupext_or_patchmode(_args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: both are the process's own C string option values.
    let undotted = |value: *mut c_char| {
        if unsafe { *value } == b'.' as c_char {
            unsafe { value.add(1) }
        } else {
            value
        }
    };
    if unsafe { cstr::eq(undotted(p_bex.get()), undotted(p_pm.get())) } {
        return Some(e_backupext_and_patchmode_are_equal);
    }
    None
}

pub fn did_set_bufhidden(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the table's own word list.
    unsafe { did_set_opt_flags(args.os_buf.b_p_bh, &opt_bh_values, false) }
}

/// 'buftype' cannot be changed into or out of "terminal": that is decided
/// by whether the buffer actually has a terminal attached.
pub fn did_set_buftype(args: &mut OptSet) -> Option<&CStr> {
    let (mut buf, mut wp) = (args.os_buf, win(args));
    // SAFETY: the buffer's own C string value; only the first letter is
    // ever distinguishing.
    let first = unsafe { *buf.b_p_bt };
    let has_terminal = !buf.terminal.is_null();
    if has_terminal != (first == b't' as c_char)
        || !unsafe { opt_strings_ok(buf.b_p_bt, &opt_bt_values, false) }
    {
        return invalid();
    }

    if first == b'p' as c_char {
        // A prompt buffer has no comment leaders, and its prompt starts at
        // the end of what is there now.
        // SAFETY: sets this buffer's own option, and replaces its prompt
        // mark (freeing what the old one held).
        set_option_direct(
            kOptComments,
            OptVal::string(String_0::from_cstr(c"")),
            OptionSetFlags::LOCAL,
            SID_NONE,
        );
        let prompt: *mut FileMark = &raw mut buf.b_prompt_start;
        unsafe { free_fmark((*prompt).clone()) };
        unsafe {
            (*prompt).mark = Pos {
                lnum: buf.b_ml.ml_line_count,
                col: buf.b_prompt_start.mark.col,
                coladd: 0 as ColNr,
            }
        };
        unsafe { (*prompt).fnum = 0 };
        unsafe { (*prompt).timestamp = os_time() };
        unsafe {
            (*prompt).view = FileMarkView {
                topline_offset: MAXLNUM as LineNr,
                skipcol: 0 as ColNr,
            }
        };
        unsafe { (*prompt).additional_data = ptr::null_mut::<AdditionalData>() };
    }

    if wp.w_status_height != 0 || global_stl_height() != 0 {
        wp.w_redr_status = true;
        redraw_later(wp, UPD_VALID);
    }
    buf.b_help = first == b'h' as c_char;
    redraw_titles();
    None
}

pub fn did_set_cinoptions(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: `parse_cino` re-derives the buffer's cache.
    unsafe { parse_cino(args.os_buf) };
    None
}

/// 'comments' is a comma-separated list of `{flags}:{leader}` parts.
///
/// The order the two messages come out in is upstream's and is load
/// bearing: an illegal flag letter still falls through to the colon and
/// length checks, so a bad one-letter value is reported as E525 rather than
/// as the illegal character.
pub fn did_set_comments(args: &mut OptSet) -> Option<&CStr> {
    let (buf, buflen) = errbuf(args);
    let mut errmsg: Option<&CStr> = None;
    // SAFETY: the frame's C string value, walked to its terminator.
    let mut s = unsafe { *varp(args) };
    while unsafe { *s } != 0 {
        // The flag letters, up to the colon.
        while unsafe { *s } != 0 && unsafe { *s } != b':' as c_char {
            if !has_char(COM_ALL, c_int::from(unsafe { *s } as u8))
                && !ascii_isdigit(c_int::from(unsafe { *s }))
                && unsafe { *s } != b'-' as c_char
            {
                errmsg = Some(unsafe { illegal_char(buf, buflen, c_int::from(*s as u8)) });
                break;
            }
            s = unsafe { s.add(1) };
        }
        let at_colon = unsafe { *s };
        s = unsafe { s.add(1) };
        if c_int::from(at_colon) == NUL {
            errmsg = Some(c"E524: Missing colon");
        } else if unsafe { *s } == b',' as c_char || c_int::from(unsafe { *s }) == NUL {
            errmsg = Some(c"E525: Zero length string");
        }
        if errmsg.is_some() {
            break;
        }
        // The leader, in which a backslash escapes the next byte.
        while unsafe { *s } != 0 && unsafe { *s } != b',' as c_char {
            if unsafe { *s } == b'\\' as c_char && c_int::from(unsafe { *s.add(1) }) != NUL {
                s = unsafe { s.add(1) };
            }
            s = unsafe { s.add(1) };
        }
        s = unsafe { skip_to_option_part(s) };
    }
    errmsg
}

pub fn did_set_commentstring(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value.
    let value = unsafe { *varp(args) };
    if c_int::from(unsafe { *value }) != NUL && !has_bytes(unsafe { cstr::at(value) }, b"%s") {
        return Some(c"E537: 'commentstring' must be empty or contain %s");
    }
    None
}

pub fn did_set_cpoptions(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), CPO_VI.as_ptr(), buf, len) }
}

pub fn did_set_diffanchors(args: &mut OptSet) -> Option<&CStr> {
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    if diffanchors_changed(local).is_err() {
        return invalid();
    }
    None
}

pub fn did_set_diffopt(_args: &mut OptSet) -> Option<&CStr> {
    if diffopt_changed().is_err() {
        return invalid();
    }
    None
}

/// 'encoding', 'fileencoding' and 'termencoding' share a callback. Only
/// UTF-8 is supported for the internal 'encoding'; the others are
/// canonicalised in place, which is why the variable is rewritten here.
pub fn did_set_encoding(args: &mut OptSet) -> Option<&CStr> {
    let (buf, varp, opt_flags, idx) = (args.os_buf, varp(args), args.os_flags, args.os_idx);
    // 'fileencoding' is the buffer-local one of the three; the other two
    // ('encoding' and 'makeencoding') are global and skip this block.
    if idx == kOptFileencoding {
        if buf.b_p_ma == 0 && opt_flags != OptionSetFlags::GLOBAL {
            return Some(e_modifiable);
        }
        // 'fileencoding' is one encoding, not a list.
        if has_char(unsafe { cstr::at(*varp) }, c_int::from(b',')) {
            return invalid();
        }
        redraw_titles();
        ml_setflags(buf);
    }

    // SAFETY: the option's own variable; `enc_canonize` allocates the
    // replacement and the old value is freed here.
    let canonical = unsafe { enc_canonize(*varp) };
    unsafe { xfree((*varp).cast::<c_void>()) };
    unsafe { *varp = canonical };
    if idx == kOptEncoding {
        if unsafe { !cstr::eq_bytes(p_enc.get(), b"utf-8") } {
            return Some(e_unsupportedoption);
        }
        spell_reload();
    }
    None
}

pub fn did_set_eventignore(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value.
    if unsafe { check_ei(*varp(args)) }.is_err() {
        return invalid();
    }
    None
}

pub fn did_set_fileformat(args: &mut OptSet) -> Option<&CStr> {
    let (buf, opt_flags) = (args.os_buf, args.os_flags);
    // Changing a buffer's line endings changes its text.
    if buf.b_p_ma == 0 && !opt_flags.has(OptionSetFlags::GLOBAL) {
        return Some(e_modifiable);
    }
    let errmsg = did_set_str_generic(args);
    if errmsg.is_some() {
        return errmsg;
    }
    redraw_titles();
    ml_setflags(buf);
    // Only "mac" is drawn differently, so a redraw is needed when
    // entering or leaving it.
    if get_fileformat(buf) == EOL_MAC || unsafe { *old_value(args) } == b'm' as c_char {
        redraw_buf_later(buf, UPD_NOT_VALID);
    }
    None
}

/// 'filetype' and 'syntax' fire an autocommand, and only when the value
/// really changed — which is what `os_value_changed` tells the caller.
pub fn did_set_filetype_or_syntax(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value and its old one.
    let value = unsafe { *varp(args) };
    if !valid_filetype(unsafe { CStr::from_ptr(value) }) {
        return invalid();
    }
    unsafe { args.os_value_changed = !cstr::eq(old_value(args), value) };
    args.os_value_checked = true;
    None
}

pub fn did_set_foldexpr(args: &mut OptSet) -> Option<&CStr> {
    did_set_optexpr(args);
    let wp = win(args);
    if foldmethod_is_expr(wp) {
        fold_update_all(wp);
    }
    None
}

pub fn did_set_foldignore(args: &mut OptSet) -> Option<&CStr> {
    let wp = win(args);
    if foldmethod_is_indent(wp) {
        fold_update_all(wp);
    }
    None
}

pub fn did_set_foldmarker(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value and window.
    let value = unsafe { *varp(args) };
    // Two markers separated by a comma, neither of them empty.
    let comma = unsafe { vim_strchr(value, c_int::from(b',')) };
    if comma.is_null() {
        return Some(e_comma_required);
    }
    if comma == value || c_int::from(unsafe { *comma.add(1) }) == NUL {
        return invalid();
    }
    let wp = win(args);
    if foldmethod_is_marker(wp) {
        fold_update_all(wp);
    }
    None
}

pub fn did_set_foldmethod(args: &mut OptSet) -> Option<&CStr> {
    let errmsg = did_set_str_generic(args);
    if errmsg.is_some() {
        return errmsg;
    }
    // SAFETY: the frame's window.
    let wp = win(args);
    fold_update_all(wp);
    // Diff folds are closed to whatever 'foldlevel' says as soon as
    // they exist.
    if foldmethod_is_diff(wp) {
        new_fold_level();
    }
    None
}

pub fn did_set_formatoptions(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), FO_ALL.as_ptr(), buf, len) }
}

/// 'iskeyword' is one of the character-class options, except that the
/// global one only has to parse — no buffer's character table depends on
/// it.
pub fn did_set_iskeyword(args: &mut OptSet) -> Option<&CStr> {
    let varp = varp(args);
    if varp != option_var(kOptIskeyword).string_var() {
        return did_set_isopt(args);
    }
    // SAFETY: the frame's C string value.
    if unsafe { check_isopt(*varp) }.is_err() {
        return invalid();
    }
    None
}

/// The shared callback for 'isident', 'isfname', 'isprint' and the
/// buffer-local 'iskeyword': rebuild the buffer's character table, and ask
/// the caller to put the old one back if it does not parse.
pub fn did_set_isopt(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's buffer's chartab is its own.
    if !unsafe { buf_init_chartab(args.os_buf, true) } {
        args.os_restore_chartab = true;
        return invalid();
    }
    None
}

/// Load a 'keymap' and switch the buffer's language modes to it.
///
/// The keymap file is sourced, which is why 'secure' is lifted for the
/// duration: the file is part of the runtime, not user input.
pub fn did_set_keymap(args: &mut OptSet) -> Option<&CStr> {
    let (mut buf, varp, opt_flags) = (args.os_buf, varp(args), args.os_flags);
    // SAFETY: the frame's C string value.
    if !unsafe { valid_filetype(CStr::from_ptr(*varp)) } {
        return invalid();
    }

    let secure_save = secure.get();
    secure.set(0);
    // Sources the keymap file named by the option.
    let errmsg = keymap_init();
    secure.set(secure_save);
    args.os_value_checked = true;
    if errmsg.is_some() {
        return errmsg;
    }

    // SAFETY: the option's own C string value.
    if c_int::from(unsafe { *buf.b_p_keymap }) != NUL {
        buf.b_p_iminsert = B_IMODE_LMAP as OptInt;
        // 'imsearch' at -1 means "follow 'iminsert'", and stays that
        // way.
        if buf.b_p_imsearch != B_IMODE_USE_INSERT as OptInt {
            buf.b_p_imsearch = B_IMODE_LMAP as OptInt;
        }
    } else {
        if buf.b_p_iminsert == B_IMODE_LMAP as OptInt {
            buf.b_p_iminsert = B_IMODE_NONE as OptInt;
        }
        if buf.b_p_imsearch == B_IMODE_LMAP as OptInt {
            buf.b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
        }
    }
    if !opt_flags.has(OptionSetFlags::LOCAL) {
        set_iminsert_global(buf);
        set_imsearch_global(buf);
    }
    status_redraw_buf(buf);
    None
}

pub fn did_set_lispoptions(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value.
    let value = unsafe { *varp(args) };
    if c_int::from(unsafe { *value }) != NUL
        && unsafe { !cstr::eq_bytes(value, b"expr:0") }
        && unsafe { !cstr::eq_bytes(value, b"expr:1") }
    {
        return invalid();
    }
    None
}

/// 'matchpairs' is a comma-separated list of `{open}:{close}` pairs. The
/// separator has to be a single-byte colon, but either character of a pair
/// may be multibyte.
pub fn did_set_matchpairs(args: &mut OptSet) -> Option<&CStr> {
    // SAFETY: the frame's C string value, walked by character length.
    let mut p = unsafe { *varp(args) };
    while c_int::from(unsafe { *p }) != NUL {
        let mut separator = -1;
        let mut close = -1;
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        if c_int::from(unsafe { *p }) != NUL {
            separator = c_int::from(unsafe { *p } as c_uchar);
            p = unsafe { p.add(1) };
        }
        if c_int::from(unsafe { *p }) != NUL {
            close = unsafe { utf_ptr2char(p) };
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }
        if separator != c_int::from(b':')
            || close == -1
            || (c_int::from(unsafe { *p }) != NUL && unsafe { *p } != b',' as c_char)
        {
            return invalid();
        }
        if c_int::from(unsafe { *p }) == NUL {
            break;
        }
        p = unsafe { p.add(1) };
    }
    None
}

pub fn did_set_varsofttabstop(args: &mut OptSet) -> Option<&CStr> {
    let mut buf = args.os_buf;
    // SAFETY: the caller's frame, and the buffer's own array.
    unsafe { did_set_vartabs(args, &raw mut buf.b_p_vsts_array) }
}

pub fn did_set_vartabstop(args: &mut OptSet) -> Option<&CStr> {
    let mut buf = args.os_buf;
    // SAFETY: the caller's frame, and the buffer's own array.
    let errmsg = unsafe { did_set_vartabs(args, &raw mut buf.b_p_vts_array) };
    if errmsg.is_none() {
        // Indent folds are computed from the tab stops.
        let wp = win(args);
        if foldmethod_is_indent(wp) {
            fold_update_all(wp);
        }
    }
    errmsg
}

/// The shared check for 'varsofttabstop' and 'vartabstop': a comma-
/// separated list of numbers, parsed into the buffer's stop array.
///
/// An empty value and a lone `0` both mean "no list", and free the array
/// rather than replacing it.
///
/// # Safety
/// `args` points at the option table's call frame, and `into` at the
/// buffer's array for this option.
unsafe fn did_set_vartabs(args: &OptSet, into: *mut *mut ColNr) -> Option<&'static CStr> {
    // SAFETY: the frame's C string value.
    let value = unsafe { CStr::from_ptr(*varp(args)) }.to_bytes();
    if value.is_empty() || value == b"0" {
        // SAFETY: the buffer's own array.
        unsafe { xfree((*into).cast::<c_void>()) };
        unsafe { *into = ptr::null_mut() };
        return None;
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
    // SAFETY: the frame's value and the buffer's own array; `tabstop_set`
    // replaces it only on success, so the old one is freed only then.
    let old = unsafe { *into };
    if !unsafe { tabstop_set(*varp(args), into) } {
        return invalid();
    }
    unsafe { xfree(old.cast::<c_void>()) };
    None
}
