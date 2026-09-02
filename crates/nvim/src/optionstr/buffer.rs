//! The callbacks for options that decide how a buffer's text is read,
//! written and understood.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
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
use crate::indent::tabstop_set;
use crate::indent_c::parse_cino;
use crate::main::{
    bkc_flags, e_modifiable, e_unsupportedoption, p_bex, p_bkc, p_bs, p_enc, p_pm, secure,
};
use crate::mark::free_fmark;
use crate::mbyte::{enc_canonize, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_setflags;
use crate::memory::xfree;
use crate::option::option_var;
use crate::option::{
    get_fileformat, redraw_titles, set_iminsert_global, set_imsearch_global, set_option_direct,
    skip_to_option_part,
};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes, kOptComments, kOptEncoding, kOptFileencoding,
    kOptIskeyword, opt_bh_values, opt_bkc_values, opt_bt_values,
};
use crate::os::cshim::strstr;
use crate::os::time::os_time;
use crate::spell::spell_reload;
use crate::strings::vim_strchr;
use crate::types::{
    AdditionalData, NUL, OptInt, OptVal, OptionSetFlags, String_0, buf_T, colnr_T, fmark_T,
    fmarkv_T, linenr_T, optset_T, pos_T,
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
use crate::winlayer::{Buf, Win};

/// 'backspace' is a word list, except that the number 2 is also accepted
/// and means everything but "nostop".
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backspace(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the option's own C string value.
    if unsafe { ascii_isdigit(c_int::from(*p_bs.get())) } {
        if unsafe { *p_bs.get() } != b'2' as c_char {
            return invalid();
        }
        return None;
    }
    unsafe { did_set_str_generic(args) }
}

/// 'backupcopy' has to name exactly one of "yes", "no" and "auto"; the
/// other words only qualify that choice.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backupcopy(args: &mut optset_T) -> Option<&CStr> {
    let (buf, opt_flags) = (args.os_buf.cast::<buf_T>(), args.os_flags);
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    // SAFETY: the frame's buffer.
    let value = unsafe {
        if local {
            (*buf).b_p_bkc
        } else {
            if !opt_flags.has(OptionSetFlags::GLOBAL) {
                // A plain `:set` drops the buffer's own answer.
                (*buf).b_bkc_flags = 0 as c_uint;
            }
            p_bkc.get()
        }
    };
    let store = |mask: c_uint| {
        if local {
            // SAFETY: the frame's buffer.
            unsafe { (*buf).b_bkc_flags = mask };
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
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backupext_or_patchmode(_args: &mut optset_T) -> Option<&CStr> {
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

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_bufhidden(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's buffer, and the table's own word list.
    let buf = args.os_buf.cast::<buf_T>();
    unsafe { did_set_opt_flags((*buf).b_p_bh, &opt_bh_values, false) }
}

/// 'buftype' cannot be changed into or out of "terminal": that is decided
/// by whether the buffer actually has a terminal attached.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_buftype(args: &mut optset_T) -> Option<&CStr> {
    let (buf, wp) = (args.os_buf.cast::<buf_T>(), win(args));
    // SAFETY: the buffer's own C string value; only the first letter is
    // ever distinguishing.
    let first = unsafe { *(*buf).b_p_bt };
    let has_terminal = unsafe { !(*buf).terminal.is_null() };
    if has_terminal != (first == b't' as c_char)
        || !unsafe { opt_strings_ok((*buf).b_p_bt, &opt_bt_values, false) }
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
            OptVal::String(String_0::from_raw_parts(c"".as_ptr().cast_mut(), 0)),
            OptionSetFlags::LOCAL,
            SID_NONE,
        );
        let prompt: *mut fmark_T = unsafe { &raw mut (*buf).b_prompt_start };
        unsafe { free_fmark((*prompt).clone()) };
        unsafe {
            (*prompt).mark = pos_T {
                lnum: (*buf).b_ml.ml_line_count,
                col: (*buf).b_prompt_start.mark.col,
                coladd: 0 as colnr_T,
            }
        };
        unsafe { (*prompt).fnum = 0 };
        unsafe { (*prompt).timestamp = os_time() };
        unsafe {
            (*prompt).view = fmarkv_T {
                topline_offset: MAXLNUM as c_int as linenr_T,
                skipcol: 0 as colnr_T,
            }
        };
        unsafe { (*prompt).additional_data = ptr::null_mut::<AdditionalData>() };
    }

    // SAFETY: the frame's window and buffer.
    if unsafe { (*wp).w_status_height } != 0 || global_stl_height() != 0 {
        unsafe { (*wp).w_redr_status = true };
        unsafe { redraw_later(wp, UPD_VALID) };
    }
    unsafe { (*buf).b_help = first == b'h' as c_char };
    redraw_titles();
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_cinoptions(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's buffer; `parse_cino` re-derives its cache.
    unsafe { parse_cino(Buf::new(args.os_buf.cast::<buf_T>())) };
    None
}

/// 'comments' is a comma-separated list of `{flags}:{leader}` parts.
///
/// The order the two messages come out in is upstream's and is load
/// bearing: an illegal flag letter still falls through to the colon and
/// length checks, so a bad one-letter value is reported as E525 rather than
/// as the illegal character.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_comments(args: &mut optset_T) -> Option<&CStr> {
    let (buf, buflen) = errbuf(args);
    let mut errmsg: Option<&CStr> = None;
    // SAFETY: the frame's C string value, walked to its terminator.
    let mut s = unsafe { *varp(args) };
    while unsafe { *s } != 0 {
        // The flag letters, up to the colon.
        while unsafe { *s } != 0 && unsafe { *s } != b':' as c_char {
            if unsafe { vim_strchr(COM_ALL.as_ptr(), c_int::from(*s as u8)) }.is_null()
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

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_commentstring(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's C string value.
    let value = unsafe { *varp(args) };
    if c_int::from(unsafe { *value }) != NUL && unsafe { strstr(value, c"%s".as_ptr()) }.is_null() {
        return Some(c"E537: 'commentstring' must be empty or contain %s");
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_cpoptions(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), CPO_VI.as_ptr(), buf, len) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_diffanchors(args: &mut optset_T) -> Option<&CStr> {
    let local = args.os_flags.has(OptionSetFlags::LOCAL);
    // SAFETY: re-reads the option's own value.
    if unsafe { diffanchors_changed(local) }.is_err() {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_diffopt(_args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: re-reads the option's own value.
    if unsafe { diffopt_changed() }.is_err() {
        return invalid();
    }
    None
}

/// 'encoding', 'fileencoding' and 'termencoding' share a callback. Only
/// UTF-8 is supported for the internal 'encoding'; the others are
/// canonicalised in place, which is why the variable is rewritten here.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_encoding(args: &mut optset_T) -> Option<&CStr> {
    let (buf, varp, opt_flags, idx) = (
        args.os_buf.cast::<buf_T>(),
        varp(args),
        args.os_flags,
        args.os_idx,
    );
    // 'fileencoding' is the buffer-local one of the three; the other two
    // ('encoding' and 'makeencoding') are global and skip this block.
    if idx == kOptFileencoding {
        // SAFETY: the frame's buffer and C string value.
        if unsafe { (*buf).b_p_ma } == 0 && opt_flags != OptionSetFlags::GLOBAL {
            return Some(e_modifiable);
        }
        // 'fileencoding' is one encoding, not a list.
        if !unsafe { vim_strchr(*varp, c_int::from(b',')) }.is_null() {
            return invalid();
        }
        redraw_titles();
        unsafe { ml_setflags(buf) };
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
        unsafe { spell_reload() };
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_eventignore(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's C string value.
    if unsafe { check_ei(*varp(args)) }.is_err() {
        return invalid();
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_fileformat(args: &mut optset_T) -> Option<&CStr> {
    let (buf, opt_flags) = (args.os_buf.cast::<buf_T>(), args.os_flags);
    // SAFETY: `optset_T` names a live buffer for exactly this call.
    let b = unsafe { Buf::new(buf) };
    // Changing a buffer's line endings changes its text.
    if unsafe { (*buf).b_p_ma } == 0 && !opt_flags.has(OptionSetFlags::GLOBAL) {
        return Some(e_modifiable);
    }
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // SAFETY: the frame's buffer and old value.
    redraw_titles();
    unsafe { ml_setflags(buf) };
    // Only "mac" is drawn differently, so a redraw is needed when
    // entering or leaving it.
    if get_fileformat(b) == EOL_MAC || unsafe { *old_value(args) } == b'm' as c_char {
        unsafe { redraw_buf_later(buf, UPD_NOT_VALID) };
    }
    None
}

/// 'filetype' and 'syntax' fire an autocommand, and only when the value
/// really changed — which is what `os_value_changed` tells the caller.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_filetype_or_syntax(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's C string value and its old one.
    let value = unsafe { *varp(args) };
    if !valid_filetype(unsafe { CStr::from_ptr(value) }) {
        return invalid();
    }
    unsafe { args.os_value_changed = !cstr::eq(old_value(args), value) };
    args.os_value_checked = true;
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldexpr(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the caller's frame and window.
    unsafe { did_set_optexpr(args) };
    let wp = win(args);
    if foldmethod_is_expr(unsafe { Win::new(wp) }) {
        fold_update_all(unsafe { Win::new(wp) });
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldignore(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's window.
    let wp = win(args);
    if foldmethod_is_indent(unsafe { Win::new(wp) }) {
        fold_update_all(unsafe { Win::new(wp) });
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldmarker(args: &mut optset_T) -> Option<&CStr> {
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
    if foldmethod_is_marker(unsafe { Win::new(wp) }) {
        fold_update_all(unsafe { Win::new(wp) });
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldmethod(args: &mut optset_T) -> Option<&CStr> {
    let errmsg = unsafe { did_set_str_generic(args) };
    if errmsg.is_some() {
        return errmsg;
    }
    // SAFETY: the frame's window.
    let wp = win(args);
    fold_update_all(unsafe { Win::new(wp) });
    // Diff folds are closed to whatever 'foldlevel' says as soon as
    // they exist.
    if foldmethod_is_diff(unsafe { Win::new(wp) }) {
        unsafe { new_fold_level() };
    }
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_formatoptions(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame, its value and its error buffer.
    let (buf, len) = errbuf(args);
    unsafe { did_set_option_listflag(*varp(args), FO_ALL.as_ptr(), buf, len) }
}

/// 'iskeyword' is one of the character-class options, except that the
/// global one only has to parse — no buffer's character table depends on
/// it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_iskeyword(args: &mut optset_T) -> Option<&CStr> {
    let varp = varp(args);
    if varp != option_var(kOptIskeyword).string_var() {
        return unsafe { did_set_isopt(args) };
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
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_isopt(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the frame's buffer.
    if !unsafe { buf_init_chartab(args.os_buf.cast::<buf_T>(), true) } {
        args.os_restore_chartab = true;
        return invalid();
    }
    None
}

/// Load a 'keymap' and switch the buffer's language modes to it.
///
/// The keymap file is sourced, which is why 'secure' is lifted for the
/// duration: the file is part of the runtime, not user input.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_keymap(args: &mut optset_T) -> Option<&CStr> {
    let (buf, varp, opt_flags) = (args.os_buf.cast::<buf_T>(), varp(args), args.os_flags);
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

    // SAFETY: the frame's buffer.
    if c_int::from(unsafe { *(*buf).b_p_keymap }) != NUL {
        unsafe { (*buf).b_p_iminsert = B_IMODE_LMAP as OptInt };
        // 'imsearch' at -1 means "follow 'iminsert'", and stays that
        // way.
        if unsafe { (*buf).b_p_imsearch } != B_IMODE_USE_INSERT as OptInt {
            unsafe { (*buf).b_p_imsearch = B_IMODE_LMAP as OptInt };
        }
    } else {
        if unsafe { (*buf).b_p_iminsert } == B_IMODE_LMAP as OptInt {
            unsafe { (*buf).b_p_iminsert = B_IMODE_NONE as OptInt };
        }
        if unsafe { (*buf).b_p_imsearch } == B_IMODE_LMAP as OptInt {
            unsafe { (*buf).b_p_imsearch = B_IMODE_USE_INSERT as OptInt };
        }
    }
    if !opt_flags.has(OptionSetFlags::LOCAL) {
        unsafe { set_iminsert_global(buf) };
        unsafe { set_imsearch_global(buf) };
    }
    unsafe { status_redraw_buf(buf) };
    None
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_lispoptions(args: &mut optset_T) -> Option<&CStr> {
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
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_matchpairs(args: &mut optset_T) -> Option<&CStr> {
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

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_varsofttabstop(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the caller's frame and buffer.
    let buf = args.os_buf.cast::<buf_T>();
    unsafe { did_set_vartabs(args, &raw mut (*buf).b_p_vsts_array) }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_vartabstop(args: &mut optset_T) -> Option<&CStr> {
    // SAFETY: the caller's frame, buffer and window.
    let buf = args.os_buf.cast::<buf_T>();
    let errmsg = unsafe { did_set_vartabs(args, &raw mut (*buf).b_p_vts_array) };
    if errmsg.is_none() {
        // Indent folds are computed from the tab stops.
        let wp = win(args);
        if foldmethod_is_indent(unsafe { Win::new(wp) }) {
            fold_update_all(unsafe { Win::new(wp) });
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
unsafe fn did_set_vartabs(args: &optset_T, into: *mut *mut colnr_T) -> Option<&'static CStr> {
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
