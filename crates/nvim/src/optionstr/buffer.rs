//! The callbacks for options that decide how a buffer's text is read,
//! written and understood.
//!
//! They are `pub` only so the generated option table can name them; see
//! [`super::frame`] for what they are handed.

#![deny(unsafe_op_in_unsafe_fn)]

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
    bkc_flags, e_modifiable, e_unsupportedoption, p_bex, p_bkc, p_bs, p_enc, p_fenc, p_isk, p_pm,
    secure,
};
use crate::mark::free_fmark;
use crate::mbyte::{enc_canonize, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_setflags;
use crate::memory::xfree;
use crate::option::{
    get_fileformat, get_option_varp_scope_from, redraw_titles, set_iminsert_global,
    set_imsearch_global, set_option_direct, skip_to_option_part,
};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagNo, kOptBkcFlagYes, kOptComments, opt_bh_values, opt_bkc_values,
    opt_bt_values,
};
use crate::os::cshim::strstr;
use crate::os::time::os_time;
use crate::spell::spell_reload;
use crate::strings::vim_strchr;
use crate::types::{
    AdditionalData, FAIL, NUL, OK, OptInt, OptVal, OptValData, OptionSetFlags, String_0, buf_T,
    colnr_T, fmark_T, fmarkv_T, linenr_T, optset_T, pos_T, win_T,
};
use crate::window::global_stl_height;
use ::libc::strcmp;

use super::frame::{errbuf, invalid, old_value, varp, win};
use super::{
    B_IMODE_LMAP, B_IMODE_NONE, B_IMODE_USE_INSERT, COM_ALL, CPO_VI, EOL_MAC, FO_ALL, SID_NONE,
    did_set_opt_flags, did_set_optexpr, did_set_option_listflag, did_set_str_generic,
    e_backupext_and_patchmode_are_equal, e_comma_required, illegal_char, kOptValTypeString,
    opt_strings_flags, valid_filetype,
};
use crate::pos::MAXLNUM;

/// 'backspace' is a word list, except that the number 2 is also accepted
/// and means everything but "nostop".
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backspace(args: *mut optset_T) -> *const c_char {
    // SAFETY: the option's own C string value.
    if unsafe { ascii_isdigit(c_int::from(*p_bs.get())) } {
        if unsafe { *p_bs.get() } != b'2' as c_char {
            return invalid();
        }
        return ptr::null();
    }
    unsafe { did_set_str_generic(args) }
}

/// 'backupcopy' has to name exactly one of "yes", "no" and "auto"; the
/// other words only qualify that choice.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backupcopy(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, opt_flags) = unsafe { ((*args).os_buf.cast::<buf_T>(), (*args).os_flags) };
    let local = opt_flags.has(OptionSetFlags::LOCAL);
    // SAFETY: the frame's buffer.
    let (value, flags) = unsafe {
        if local {
            ((*buf).b_p_bkc, &raw mut (*buf).b_bkc_flags)
        } else {
            if !opt_flags.has(OptionSetFlags::GLOBAL) {
                // A plain `:set` drops the buffer's own answer.
                (*buf).b_bkc_flags = 0 as c_uint;
            }
            (p_bkc.get(), bkc_flags.ptr())
        }
    };

    // SAFETY: a C string, the table's own word list, and the mask beside it.
    unsafe {
        // An empty buffer-local value means "no override", not "no words".
        if local && c_int::from(*value) == NUL {
            *flags = 0 as c_uint;
            return ptr::null();
        }
        if opt_strings_flags(value, &opt_bkc_values, flags, true) != OK {
            return invalid();
        }
        let named = [kOptBkcFlagAuto, kOptBkcFlagYes, kOptBkcFlagNo]
            .into_iter()
            .filter(|word| *flags & *word as c_uint != 0)
            .count();
        if named != 1 {
            // The mask was already rebuilt from the new value; put the old
            // one back, since the caller only restores the string.
            opt_strings_flags(old_value(args), &opt_bkc_values, flags, true);
            return invalid();
        }
    }
    ptr::null()
}

/// 'backupext' and 'patchmode' both rename a file out of the way, so they
/// cannot be the same — a leading dot is not part of the comparison.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_backupext_or_patchmode(_args: *mut optset_T) -> *const c_char {
    // SAFETY: both are the process's own C string option values.
    unsafe {
        let undotted = |value: *mut c_char| {
            if *value == b'.' as c_char {
                value.add(1)
            } else {
                value
            }
        };
        if strcmp(undotted(p_bex.get()), undotted(p_pm.get())) == 0 {
            return e_backupext_and_patchmode_are_equal.as_ptr();
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_bufhidden(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's buffer, and the table's own word list.
    unsafe {
        let buf = (*args).os_buf.cast::<buf_T>();
        did_set_opt_flags((*buf).b_p_bh, &opt_bh_values, ptr::null_mut(), false)
    }
}

/// 'buftype' cannot be changed into or out of "terminal": that is decided
/// by whether the buffer actually has a terminal attached.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_buftype(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame, buffer and window.
    let (buf, wp) = unsafe { ((*args).os_buf.cast::<buf_T>(), win(args)) };
    // SAFETY: the buffer's own C string value; only the first letter is
    // ever distinguishing.
    let first = unsafe { *(*buf).b_p_bt };
    let has_terminal = unsafe { !(*buf).terminal.is_null() };
    if has_terminal != (first == b't' as c_char)
        || unsafe { opt_strings_flags((*buf).b_p_bt, &opt_bt_values, ptr::null_mut(), false) } != OK
    {
        return invalid();
    }

    if first == b'p' as c_char {
        // A prompt buffer has no comment leaders, and its prompt starts at
        // the end of what is there now.
        // SAFETY: sets this buffer's own option, and replaces its prompt
        // mark (freeing what the old one held).
        unsafe {
            set_option_direct(
                kOptComments,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0::from_raw_parts(c"".as_ptr().cast_mut(), 0),
                    },
                },
                OptionSetFlags::LOCAL,
                SID_NONE,
            );
            let prompt: *mut fmark_T = &raw mut (*buf).b_prompt_start;
            free_fmark(*prompt);
            (*prompt).mark = pos_T {
                lnum: (*buf).b_ml.ml_line_count,
                col: (*buf).b_prompt_start.mark.col,
                coladd: 0 as colnr_T,
            };
            (*prompt).fnum = 0;
            (*prompt).timestamp = os_time();
            (*prompt).view = fmarkv_T {
                topline_offset: MAXLNUM as c_int as linenr_T,
                skipcol: 0 as colnr_T,
            };
            (*prompt).additional_data = ptr::null_mut::<AdditionalData>();
        }
    }

    // SAFETY: the frame's window and buffer.
    unsafe {
        if (*wp).w_status_height != 0 || global_stl_height() != 0 {
            (*wp).w_redr_status = true;
            redraw_later(wp, UPD_VALID);
        }
        (*buf).b_help = first == b'h' as c_char;
        redraw_titles();
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_cinoptions(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's buffer; `parse_cino` re-derives its cache.
    unsafe { parse_cino((*args).os_buf.cast::<buf_T>()) };
    ptr::null()
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
pub unsafe fn did_set_comments(args: *mut optset_T) -> *const c_char {
    let (buf, buflen) = unsafe { errbuf(args) };
    let mut errmsg: *const c_char = ptr::null();
    // SAFETY: the frame's C string value, walked to its terminator.
    unsafe {
        let mut s = *varp(args);
        while *s != 0 {
            // The flag letters, up to the colon.
            while *s != 0 && *s != b':' as c_char {
                if vim_strchr(COM_ALL.as_ptr(), c_int::from(*s as u8)).is_null()
                    && !ascii_isdigit(c_int::from(*s))
                    && *s != b'-' as c_char
                {
                    errmsg = illegal_char(buf, buflen, c_int::from(*s as u8));
                    break;
                }
                s = s.add(1);
            }
            let at_colon = *s;
            s = s.add(1);
            if c_int::from(at_colon) == NUL {
                errmsg = c"E524: Missing colon".as_ptr();
            } else if *s == b',' as c_char || c_int::from(*s) == NUL {
                errmsg = c"E525: Zero length string".as_ptr();
            }
            if !errmsg.is_null() {
                break;
            }
            // The leader, in which a backslash escapes the next byte.
            while *s != 0 && *s != b',' as c_char {
                if *s == b'\\' as c_char && c_int::from(*s.add(1)) != NUL {
                    s = s.add(1);
                }
                s = s.add(1);
            }
            s = skip_to_option_part(s);
        }
    }
    errmsg
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_commentstring(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    unsafe {
        let value = *varp(args);
        if c_int::from(*value) != NUL && strstr(value, c"%s".as_ptr()).is_null() {
            return c"E537: 'commentstring' must be empty or contain %s".as_ptr();
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_cpoptions(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame, its value and its error buffer.
    unsafe {
        let (buf, len) = errbuf(args);
        did_set_option_listflag(*varp(args), CPO_VI.as_ptr(), buf, len)
    }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_diffanchors(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame.
    let local = unsafe { (*args).os_flags }.has(OptionSetFlags::LOCAL);
    // SAFETY: re-reads the option's own value.
    if unsafe { diffanchors_changed(local) } == FAIL {
        return invalid();
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_diffopt(_args: *mut optset_T) -> *const c_char {
    // SAFETY: re-reads the option's own value.
    if unsafe { diffopt_changed() } == FAIL {
        return invalid();
    }
    ptr::null()
}

/// 'encoding', 'fileencoding' and 'termencoding' share a callback. Only
/// UTF-8 is supported for the internal 'encoding'; the others are
/// canonicalised in place, which is why the variable is rewritten here.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_encoding(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, varp, opt_flags, idx) = unsafe {
        (
            (*args).os_buf.cast::<buf_T>(),
            varp(args),
            (*args).os_flags,
            (*args).os_idx,
        )
    };
    // SAFETY: the option table's scope plumbing, with this buffer.
    let gvarp = unsafe {
        get_option_varp_scope_from(idx, OptionSetFlags::GLOBAL, buf, ptr::null_mut::<win_T>())
    }
    .cast::<*mut c_char>();

    if gvarp == p_fenc.ptr() {
        // SAFETY: the frame's buffer and C string value.
        unsafe {
            if (*buf).b_p_ma == 0 && opt_flags != OptionSetFlags::GLOBAL {
                return e_modifiable.as_ptr();
            }
            // 'fileencoding' is one encoding, not a list.
            if !vim_strchr(*varp, c_int::from(b',')).is_null() {
                return invalid();
            }
            redraw_titles();
            ml_setflags(buf);
        }
    }

    // SAFETY: the option's own variable; `enc_canonize` allocates the
    // replacement and the old value is freed here.
    unsafe {
        let canonical = enc_canonize(*varp);
        xfree((*varp).cast::<c_void>());
        *varp = canonical;
        if varp == p_enc.ptr() {
            if strcmp(p_enc.get(), c"utf-8".as_ptr()) != 0 {
                return e_unsupportedoption.as_ptr();
            }
            spell_reload();
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_eventignore(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    if unsafe { check_ei(*varp(args)) } == FAIL {
        return invalid();
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_fileformat(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, opt_flags) = unsafe { ((*args).os_buf.cast::<buf_T>(), (*args).os_flags) };
    // Changing a buffer's line endings changes its text.
    if unsafe { (*buf).b_p_ma } == 0 && !opt_flags.has(OptionSetFlags::GLOBAL) {
        return e_modifiable.as_ptr();
    }
    let errmsg = unsafe { did_set_str_generic(args) };
    if !errmsg.is_null() {
        return errmsg;
    }
    // SAFETY: the frame's buffer and old value.
    unsafe {
        redraw_titles();
        ml_setflags(buf);
        // Only "mac" is drawn differently, so a redraw is needed when
        // entering or leaving it.
        if get_fileformat(buf) == EOL_MAC || *old_value(args) == b'm' as c_char {
            redraw_buf_later(buf, UPD_NOT_VALID);
        }
    }
    ptr::null()
}

/// 'filetype' and 'syntax' fire an autocommand, and only when the value
/// really changed — which is what `os_value_changed` tells the caller.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_filetype_or_syntax(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value and its old one.
    unsafe {
        let value = *varp(args);
        if !valid_filetype(CStr::from_ptr(value)) {
            return invalid();
        }
        (*args).os_value_changed = strcmp(old_value(args), value) != 0;
        (*args).os_value_checked = true;
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldexpr(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and window.
    unsafe {
        did_set_optexpr(args);
        let wp = win(args);
        if foldmethod_is_expr(wp) {
            fold_update_all(wp);
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldignore(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's window.
    unsafe {
        let wp = win(args);
        if foldmethod_is_indent(wp) {
            fold_update_all(wp);
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldmarker(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value and window.
    unsafe {
        let value = *varp(args);
        // Two markers separated by a comma, neither of them empty.
        let comma = vim_strchr(value, c_int::from(b','));
        if comma.is_null() {
            return e_comma_required.as_ptr();
        }
        if comma == value || c_int::from(*comma.add(1)) == NUL {
            return invalid();
        }
        let wp = win(args);
        if foldmethod_is_marker(wp) {
            fold_update_all(wp);
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_foldmethod(args: *mut optset_T) -> *const c_char {
    let errmsg = unsafe { did_set_str_generic(args) };
    if !errmsg.is_null() {
        return errmsg;
    }
    // SAFETY: the frame's window.
    unsafe {
        let wp = win(args);
        fold_update_all(wp);
        // Diff folds are closed to whatever 'foldlevel' says as soon as
        // they exist.
        if foldmethod_is_diff(wp) {
            new_fold_level();
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_formatoptions(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame, its value and its error buffer.
    unsafe {
        let (buf, len) = errbuf(args);
        did_set_option_listflag(*varp(args), FO_ALL.as_ptr(), buf, len)
    }
}

/// 'iskeyword' is one of the character-class options, except that the
/// global one only has to parse — no buffer's character table depends on
/// it.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_iskeyword(args: *mut optset_T) -> *const c_char {
    let varp = unsafe { varp(args) };
    if varp != p_isk.ptr() {
        return unsafe { did_set_isopt(args) };
    }
    // SAFETY: the frame's C string value.
    if unsafe { check_isopt(*varp) } == FAIL {
        return invalid();
    }
    ptr::null()
}

/// The shared callback for 'isident', 'isfname', 'isprint' and the
/// buffer-local 'iskeyword': rebuild the buffer's character table, and ask
/// the caller to put the old one back if it does not parse.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_isopt(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's buffer.
    unsafe {
        if !buf_init_chartab((*args).os_buf.cast::<buf_T>(), true) {
            (*args).os_restore_chartab = true;
            return invalid();
        }
    }
    ptr::null()
}

/// Load a 'keymap' and switch the buffer's language modes to it.
///
/// The keymap file is sourced, which is why 'secure' is lifted for the
/// duration: the file is part of the runtime, not user input.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_keymap(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    let (buf, varp, opt_flags) =
        unsafe { ((*args).os_buf.cast::<buf_T>(), varp(args), (*args).os_flags) };
    // SAFETY: the frame's C string value.
    if !unsafe { valid_filetype(CStr::from_ptr(*varp)) } {
        return invalid();
    }

    let secure_save = secure.get();
    secure.set(0);
    // Sources the keymap file named by the option.
    let errmsg = keymap_init();
    secure.set(secure_save);
    // SAFETY: the caller's frame.
    unsafe { (*args).os_value_checked = true };
    if !errmsg.is_null() {
        return errmsg;
    }

    // SAFETY: the frame's buffer.
    unsafe {
        if c_int::from(*(*buf).b_p_keymap) != NUL {
            (*buf).b_p_iminsert = B_IMODE_LMAP as OptInt;
            // 'imsearch' at -1 means "follow 'iminsert'", and stays that
            // way.
            if (*buf).b_p_imsearch != B_IMODE_USE_INSERT as OptInt {
                (*buf).b_p_imsearch = B_IMODE_LMAP as OptInt;
            }
        } else {
            if (*buf).b_p_iminsert == B_IMODE_LMAP as OptInt {
                (*buf).b_p_iminsert = B_IMODE_NONE as OptInt;
            }
            if (*buf).b_p_imsearch == B_IMODE_LMAP as OptInt {
                (*buf).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
            }
        }
        if !opt_flags.has(OptionSetFlags::LOCAL) {
            set_iminsert_global(buf);
            set_imsearch_global(buf);
        }
        status_redraw_buf(buf);
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_lispoptions(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    unsafe {
        let value = *varp(args);
        if c_int::from(*value) != NUL
            && strcmp(value, c"expr:0".as_ptr()) != 0
            && strcmp(value, c"expr:1".as_ptr()) != 0
        {
            return invalid();
        }
    }
    ptr::null()
}

/// 'matchpairs' is a comma-separated list of `{open}:{close}` pairs. The
/// separator has to be a single-byte colon, but either character of a pair
/// may be multibyte.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_matchpairs(args: *mut optset_T) -> *const c_char {
    // SAFETY: the frame's C string value, walked by character length.
    unsafe {
        let mut p = *varp(args);
        while c_int::from(*p) != NUL {
            let mut separator = -1;
            let mut close = -1;
            p = p.add(utfc_ptr2len(p) as usize);
            if c_int::from(*p) != NUL {
                separator = c_int::from(*p as c_uchar);
                p = p.add(1);
            }
            if c_int::from(*p) != NUL {
                close = utf_ptr2char(p);
                p = p.add(utfc_ptr2len(p) as usize);
            }
            if separator != c_int::from(b':')
                || close == -1
                || (c_int::from(*p) != NUL && *p != b',' as c_char)
            {
                return invalid();
            }
            if c_int::from(*p) == NUL {
                break;
            }
            p = p.add(1);
        }
    }
    ptr::null()
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_varsofttabstop(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame and buffer.
    unsafe {
        let buf = (*args).os_buf.cast::<buf_T>();
        did_set_vartabs(args, &raw mut (*buf).b_p_vsts_array)
    }
}

/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_vartabstop(args: *mut optset_T) -> *const c_char {
    // SAFETY: the caller's frame, buffer and window.
    unsafe {
        let buf = (*args).os_buf.cast::<buf_T>();
        let errmsg = did_set_vartabs(args, &raw mut (*buf).b_p_vts_array);
        if errmsg.is_null() {
            // Indent folds are computed from the tab stops.
            let wp = win(args);
            if foldmethod_is_indent(wp) {
                fold_update_all(wp);
            }
        }
        errmsg
    }
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
unsafe fn did_set_vartabs(args: *mut optset_T, into: *mut *mut colnr_T) -> *const c_char {
    // SAFETY: the frame's C string value.
    let value = unsafe { CStr::from_ptr(*varp(args)) }.to_bytes();
    if value.is_empty() || value == b"0" {
        // SAFETY: the buffer's own array.
        unsafe {
            xfree((*into).cast::<c_void>());
            *into = ptr::null_mut();
        }
        return ptr::null();
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
    unsafe {
        let old = *into;
        if !tabstop_set(*varp(args), into) {
            return invalid();
        }
        xfree(old.cast::<c_void>());
    }
    ptr::null()
}
