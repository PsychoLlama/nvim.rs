//! The accessors the rest of the editor reads options through.
//!
//! Most of these exist because a value is not simply "the variable": a
//! global-local option falls back to the global when the local copy is
//! unset, 'shortmess' has an abbreviation that stands for four other flags,
//! 'virtualedit' and 'cursorlineopt' are read as parsed flag words, and
//! 'scrolloff' is forced to zero in a terminal buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
use core::ptr;

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::buffer::bt_prompt;
use crate::src::nvim::drawscreen::redraw_buf_status_later;
use crate::src::nvim::eval::typval::{
    callback_free, kCallbackNone, tv_dict_add_tv, tv_dict_alloc, tv_free,
};
use crate::src::nvim::eval::vars::optval_as_tv;
use crate::src::nvim::eval::{callback_from_typval, eval_expr};
use crate::src::nvim::main::{
    OPTION_MAGIC_OFF, OPTION_MAGIC_ON, State, bkc_flags, curbuf, empty_string_option,
    magic_overruled, need_maketitle, p_bs, p_ep, p_ffs, p_ffu, p_flp, p_magic, p_sbr, p_sh, p_shm,
    p_siso, p_so, redraw_tabline, ve_flags,
};
use crate::src::nvim::memory::{xcalloc, xfree, xstrdup};
use crate::src::nvim::options::*;
use crate::src::nvim::os::env::{os_setenv, vim_getenv};
use crate::src::nvim::os::libc::{strcmp, strlen, strncmp, strstr};
use crate::src::nvim::path::{FullName_save, path_tail};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    Callback, Callback_data, OptIndex, OptVal, OptValData, buf_T, dict_T, exarg_T, int64_t, scid_T,
    size_t, typval_T, uint8_t, vimoption_T, win_T,
};

use super::{
    BS_NOSTOP, BS_START, EOL_DOS, EOL_MAC, EOL_UNIX, FAIL, FORCE_BIN, NUL, OK, SHM_LINES, SHM_MOD,
    SHM_RO, SHM_WRI, VAR_STRING, get_varp, kOptFlagWasSet, kOptScopeBuf, kOptScopeWin,
    kOptValTypeString, option_has_scope, optval_from_varp, set_option_direct,
};
use crate::src::nvim::state::MODE_TERMINAL;

/// 'equalprg', local where set.
pub fn get_equalprg() -> *mut c_char {
    // SAFETY: `curbuf` is live, and its string options are never null.
    unsafe {
        if *(*curbuf.get()).b_p_ep == 0 {
            p_ep.get()
        } else {
            (*curbuf.get()).b_p_ep
        }
    }
}

/// 'findfunc', local where set.
pub fn get_findfunc() -> *mut c_char {
    // SAFETY: `curbuf` is live, and its string options are never null.
    unsafe {
        if *(*curbuf.get()).b_p_ffu == 0 {
            p_ffu.get()
        } else {
            (*curbuf.get()).b_p_ffu
        }
    }
}

/// Whether 'shortmess' asks for message `x` to be shortened. The `a` flag is
/// an abbreviation standing for these four and nothing else.
pub fn shortmess(x: c_int) -> bool {
    const ABBREVIATED: [c_uint; 4] = [SHM_RO, SHM_MOD, SHM_LINES, SHM_WRI];
    // SAFETY: 'shortmess' is a string option; the null test is upstream's.
    unsafe {
        !p_shm.ptr().is_null()
            && (!vim_strchr(p_shm.get(), x).is_null()
                || (!vim_strchr(p_shm.get(), 'a' as c_int).is_null()
                    && ABBREVIATED.contains(&(x as c_uint))))
    }
}

/// Record where a vimrc was found in `$MYVIMRC`/`$MYVIMDIR`, unless the
/// environment variable is already set.
///
/// # Safety
///
/// `fname` and `envname`, when non-null, must be NUL-terminated.
pub unsafe fn vimrc_found(fname: *mut c_char, envname: *mut c_char) {
    if fname.is_null() || envname.is_null() {
        return;
    }
    // SAFETY: the caller's strings are NUL-terminated.
    unsafe {
        let existing = vim_getenv(envname);
        if !existing.is_null() {
            xfree(existing.cast::<c_void>());
            return;
        }
        let full = FullName_save(fname, false);
        if !full.is_null() {
            os_setenv(envname, full, 1);
            xfree(full.cast::<c_void>());
        }
    }
}

/// Whether anything has ever set the option.
pub fn option_was_set(opt_idx: OptIndex) -> bool {
    assert!(opt_idx != kOptInvalid);
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*options.ptr())[opt_idx as usize].flags & kOptFlagWasSet != 0 }
}

/// Forget that anything set the option — what `:set all&` does.
pub fn reset_option_was_set(opt_idx: OptIndex) {
    assert!(opt_idx != kOptInvalid);
    // SAFETY: the option table is a plain array; nothing holds a borrow.
    unsafe { (*options.ptr())[opt_idx as usize].flags &= !kOptFlagWasSet }
}

/// Parse 'cursorlineopt' into `wp->w_p_culopt_flags`, from `val` or from the
/// window's own value. `FAIL` for a value that does not parse; the flags are
/// only stored on success.
///
/// # Safety
///
/// `val`, when non-null, must be NUL-terminated; `wp` must be live.
pub unsafe fn fill_culopt_flags(val: *mut c_char, wp: *mut win_T) -> c_int {
    // SAFETY: the caller's `wp` is live and `val` is NUL-terminated.
    unsafe {
        let mut p = if val.is_null() {
            (*wp).w_onebuf_opt.wo_culopt
        } else {
            val
        };
        let mut flags: uint8_t = 0;
        while *p != 0 {
            for (word, bits) in [
                (c"line".to_bytes(), kOptCuloptFlagLine),
                (
                    c"both".to_bytes(),
                    kOptCuloptFlagLine | kOptCuloptFlagNumber,
                ),
                (c"number".to_bytes(), kOptCuloptFlagNumber),
                (c"screenline".to_bytes(), kOptCuloptFlagScreenline),
            ] {
                if strncmp(p, word.as_ptr().cast::<c_char>(), word.len() as size_t) == 0 {
                    p = p.add(word.len());
                    flags |= bits as uint8_t;
                    break;
                }
            }
            // Anything the words above did not consume is a syntax error.
            if *p != b',' as c_char && *p != 0 {
                return FAIL;
            }
            if *p == b',' as c_char {
                p = p.add(1);
            }
        }
        // "line" and "screenline" are mutually exclusive; "both" implies
        // "line", so it collides with "screenline" too.
        if flags as c_int & kOptCuloptFlagLine as c_int != 0
            && flags as c_int & kOptCuloptFlagScreenline as c_int != 0
        {
            return FAIL;
        }
        (*wp).w_p_culopt_flags = flags;
        OK
    }
}

/// Whether patterns are magic right now — `\v`/`\V` in the pattern override
/// 'magic' for the pattern they appear in.
pub fn magic_isset() -> bool {
    match magic_overruled.get() {
        OPTION_MAGIC_ON => true,
        OPTION_MAGIC_OFF => false,
        _ => p_magic.get() != 0,
    }
}

/// Parse a `'*func'` option's value into `optcb`. An empty value clears the
/// callback; anything that does not resolve to one leaves `optcb` alone.
///
/// # Safety
///
/// `optval`, when non-null, must be NUL-terminated; `optcb` must point at a
/// live `Callback` this call may replace.
pub unsafe fn option_set_callback_func(optval: *mut c_char, optcb: *mut Callback) -> c_int {
    // SAFETY: the caller's pointers are valid for the call.
    unsafe {
        if optval.is_null() || *optval == 0 {
            callback_free(optcb);
            return OK;
        }
        // A lambda, `function(...)` or `funcref(...)` is an expression; a
        // bare name is the function's name.
        let tv = if *optval == b'{' as c_char
            || strncmp(optval, c"function(".as_ptr(), 9) == 0
            || strncmp(optval, c"funcref(".as_ptr(), 8) == 0
        {
            let tv = eval_expr(optval, ptr::null_mut::<exarg_T>());
            if tv.is_null() {
                return FAIL;
            }
            tv
        } else {
            let tv = xcalloc(1, size_of::<typval_T>()).cast::<typval_T>();
            (*tv).v_type = VAR_STRING;
            (*tv).vval.v_string = xstrdup(optval);
            tv
        };
        let mut cb = Callback {
            data: Callback_data {
                funcref: ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        };
        if !callback_from_typval(&raw mut cb, tv) || cb.type_0 == kCallbackNone {
            tv_free(tv);
            return FAIL;
        }
        callback_free(optcb);
        *optcb = cb;
        tv_free(tv);
        OK
    }
}

/// Whether 'backspace' allows backspacing over `what`. A prompt buffer never
/// lets the prompt itself be backspaced over.
pub fn can_bs(what: c_int) -> bool {
    // SAFETY: `curbuf` is live; 'backspace' is a string option.
    unsafe {
        if what == BS_START && bt_prompt(curbuf.get()) {
            return false;
        }
        // The historic numeric spelling: 2 is everything but "nostop".
        if *p_bs.get() == b'2' as c_char {
            return what != BS_NOSTOP;
        }
        !vim_strchr(p_bs.get(), what).is_null()
    }
}

/// 'backupcopy' as flags, local where set.
///
/// # Safety
///
/// `buf` must be live.
pub unsafe fn get_bkc_flags(buf: *mut buf_T) -> c_uint {
    // SAFETY: the caller's buffer is live.
    match unsafe { (*buf).b_bkc_flags } {
        0 => bkc_flags.get(),
        local => local,
    }
}

/// 'formatlistpat', local where set.
///
/// # Safety
///
/// `buf` must be live.
pub unsafe fn get_flp_value(buf: *mut buf_T) -> *mut c_char {
    // SAFETY: the caller's buffer is live.
    unsafe {
        if (*buf).b_p_flp.is_null() || *(*buf).b_p_flp == 0 {
            p_flp.get()
        } else {
            (*buf).b_p_flp
        }
    }
}

/// 'virtualedit' as flags, local where set. The two "none" bits only exist
/// so a window can spell out that it overrides the global value with
/// nothing, so they never reach a caller.
///
/// # Safety
///
/// `wp` must be live.
pub unsafe fn get_ve_flags(wp: *mut win_T) -> c_uint {
    // SAFETY: the caller's window is live.
    let flags = match unsafe { (*wp).w_onebuf_opt.wo_ve_flags } {
        0 => ve_flags.get(),
        local => local,
    };
    flags & !(kOptVeFlagNone | kOptVeFlagNoneU)
}

/// 'showbreak', local where set. `"NONE"` is how a window says "no leader"
/// against a global value that has one.
///
/// # Safety
///
/// `win` must be live.
pub unsafe fn get_showbreak_value(win: *mut win_T) -> *mut c_char {
    // SAFETY: the caller's window is live.
    unsafe {
        let local = (*win).w_onebuf_opt.wo_sbr;
        if local.is_null() || *local == 0 {
            return p_sbr.get();
        }
        if strcmp(local, c"NONE".as_ptr()) == 0 {
            return empty_string_option.ptr().cast::<c_char>();
        }
        local
    }
}

/// The buffer's line ending. 'binary' forces Unix whatever 'fileformat' says.
///
/// # Safety
///
/// `buf` must be live.
pub unsafe fn get_fileformat(buf: *const buf_T) -> c_int {
    // SAFETY: the caller's buffer is live; 'fileformat' is never null.
    unsafe {
        let c = *(*buf).b_p_ff as c_uchar;
        if (*buf).b_p_bin != 0 || c == b'u' {
            EOL_UNIX
        } else if c == b'm' {
            EOL_MAC
        } else {
            EOL_DOS
        }
    }
}

/// [`get_fileformat`] with a command's `++ff`/`++bin` overriding the buffer.
///
/// # Safety
///
/// `buf` must be live; `eap`, when non-null, must be a live command.
pub unsafe fn get_fileformat_force(buf: *const buf_T, eap: *const exarg_T) -> c_int {
    // SAFETY: the caller's pointers are live.
    let c = unsafe {
        if !eap.is_null() && (*eap).force_ff != 0 {
            (*eap).force_ff
        } else {
            let binary = if !eap.is_null() && (*eap).force_bin != 0 {
                ((*eap).force_bin == FORCE_BIN) as c_int
            } else {
                (*buf).b_p_bin
            };
            if binary != 0 {
                return EOL_UNIX;
            }
            *(*buf).b_p_ff as c_uchar as c_int
        }
    };
    match c as u8 {
        b'u' => EOL_UNIX,
        b'm' => EOL_MAC,
        _ => EOL_DOS,
    }
}

/// The line ending a new file gets: the first entry of 'fileformats'.
pub fn default_fileformat() -> c_int {
    // SAFETY: 'fileformats' is a string option; it is never null.
    match unsafe { *p_ffs.get() } as u8 {
        b'm' => EOL_MAC,
        b'd' => EOL_DOS,
        _ => EOL_UNIX,
    }
}

/// Set 'fileformat' to `eol_style` and redraw what shows it.
pub fn set_fileformat(eol_style: c_int, opt_flags: c_int) {
    let name = match eol_style {
        EOL_UNIX => Some(c"unix"),
        EOL_MAC => Some(c"mac"),
        EOL_DOS => Some(c"dos"),
        _ => None,
    };
    // SAFETY: the names are static; `curbuf` is live.
    unsafe {
        if let Some(name) = name {
            set_option_direct(
                kOptFileformat,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(name.as_ptr().cast_mut()),
                    },
                },
                opt_flags,
                0 as scid_T,
            );
        }
        redraw_buf_status_later(curbuf.get());
    }
    redraw_tabline.set(true);
    need_maketitle.set(true);
}

/// Step over the separator and any padding between two parts of a
/// comma-separated option.
///
/// # Safety
///
/// `p` must be NUL-terminated.
pub unsafe fn skip_to_option_part(mut p: *const c_char) -> *mut c_char {
    // SAFETY: the caller's string is NUL-terminated.
    unsafe {
        if *p == b',' as c_char {
            p = p.add(1);
        }
        while *p == b' ' as c_char {
            p = p.add(1);
        }
    }
    p.cast_mut()
}

/// Copy one part of a separated option into `buf` and advance `option` past
/// it, returning the part's length. A part longer than `maxlen - 1` is
/// truncated in `buf` but still counted in full.
///
/// # Safety
///
/// `option` must point at a NUL-terminated string; `buf` must have room for
/// `maxlen` bytes; `sep_chars` must be NUL-terminated.
pub unsafe fn copy_option_part(
    option: *mut *mut c_char,
    buf: *mut c_char,
    maxlen: size_t,
    sep_chars: *mut c_char,
) -> size_t {
    // SAFETY: the caller's pointers are valid for the lengths documented.
    unsafe {
        let mut len: size_t = 0;
        let mut p = *option;
        // A leading '.' is copied without being tested against the
        // separators, so `.` can start a path entry.
        if *p == b'.' as c_char {
            *buf = *p;
            p = p.add(1);
            len = 1;
        }
        while *p != 0 && vim_strchr(sep_chars, *p as uint8_t as c_int).is_null() {
            // A backslash escapes a separator, and is dropped.
            if *p == b'\\' as c_char
                && !vim_strchr(sep_chars, *p.add(1) as uint8_t as c_int).is_null()
            {
                p = p.add(1);
            }
            if len < maxlen.wrapping_sub(1) {
                *buf.add(len) = *p;
                len += 1;
            }
            p = p.add(1);
        }
        *buf.add(len) = NUL as c_char;
        // Step over the separator we stopped on — unless it is a comma,
        // which `skip_to_option_part` handles along with the padding.
        if *p != 0 && *p != b',' as c_char {
            p = p.add(1);
        }
        *option = skip_to_option_part(p);
        len
    }
}

/// Whether 'shell' is a csh derivative, which needs its own quoting.
pub fn csh_like_shell() -> bool {
    // SAFETY: 'shell' is a string option; it is never null.
    unsafe { !strstr(path_tail(p_sh.get()), c"csh".as_ptr()).is_null() }
}

/// Whether 'shell' is fish, which needs its own quoting.
pub fn fish_like_shell() -> bool {
    // SAFETY: 'shell' is a string option; it is never null.
    unsafe { !strstr(path_tail(p_sh.get()), c"fish".as_ptr()).is_null() }
}

/// Every buffer-local (or window-local) option of the current buffer and
/// window, as a dictionary — what `b:` and `w:` expose.
pub fn get_winbuf_options(bufopt: c_int) -> *mut dict_T {
    let scope = if bufopt != 0 {
        kOptScopeBuf
    } else {
        kOptScopeWin
    };
    // SAFETY: the option table is a plain array, and `get_varp` hands back
    // the variable for the current buffer and window.
    unsafe {
        let d = tv_dict_alloc();
        for opt_idx in kOptAleph..kOptCount {
            if !option_has_scope(opt_idx, scope) {
                continue;
            }
            let opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            let varp = get_varp(opt);
            if varp.is_null() {
                continue;
            }
            let mut tv = optval_as_tv(optval_from_varp(opt_idx, varp), true);
            tv_dict_add_tv(d, (*opt).fullname, strlen((*opt).fullname), &raw mut tv);
        }
        d
    }
}

/// 'scrolloff' for a window, local where set. A terminal buffer never
/// scrolls off, whatever the option says.
///
/// # Safety
///
/// `wp` must be live, with a live buffer.
pub unsafe fn get_scrolloff_value(wp: *mut win_T) -> int64_t {
    // SAFETY: the caller's window and its buffer are live.
    unsafe {
        if State.get() & MODE_TERMINAL != 0 && !(*(*wp).w_buffer).terminal.is_null() {
            return 0;
        }
        match (*wp).w_onebuf_opt.wo_so {
            local if local < 0 => p_so.get(),
            local => local,
        }
    }
}

/// 'sidescrolloff' for a window, local where set.
///
/// # Safety
///
/// `wp` must be live.
pub unsafe fn get_sidescrolloff_value(wp: *mut win_T) -> int64_t {
    // SAFETY: the caller's window is live.
    match unsafe { (*wp).w_onebuf_opt.wo_siso } {
        local if local < 0 => p_siso.get(),
        local => local,
    }
}
