//! The accessors the rest of the editor reads options through.
//!
//! Most of these exist because a value is not simply "the variable": a
//! global-local option falls back to the global when the local copy is
//! unset, 'shortmess' has an abbreviation that stands for four other flags,
//! 'virtualedit' and 'cursorlineopt' are read as parsed flag words, and
//! 'scrolloff' is forced to zero in a terminal buffer.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_uchar, c_uint, c_void};
use core::ptr;

use crate::api::private::helpers::cstr_as_string;
use crate::buffer::{buf_is_prompt, current_buf};
use crate::cstr;
use crate::drawscreen::redraw_buf_status_later;
use crate::eval::typval::{callback_free, kCallbackNone, tv_dict_add_tv, tv_dict_alloc, tv_free};
use crate::eval::vars::optval_as_tv;
use crate::eval::{callback_from_typval, eval_expr};
use crate::main::{
    OPTION_MAGIC_OFF, OPTION_MAGIC_ON, State, bkc_flags, curbuf, magic_overruled, need_maketitle,
    p_bs, p_cpo, p_ep, p_ffs, p_ffu, p_flp, p_magic, p_sbr, p_sh, p_shm, p_siso, p_so,
    redraw_tabline, ve_flags,
};
use crate::memory::{xcalloc, xfree, xstrdup};
use crate::options::*;
use crate::optionstr::empty_option;
use crate::os::cshim::strstr;
use crate::os::env::{os_setenv, vim_getenv};
use crate::path::{full_name_save, path_tail};
use crate::strings::vim_strchr;
use crate::types::{
    BsFlag, Callback, Callback_data, CpoFlag, Failed, NUL, OptInt, OptVal, OptValData,
    OptionSetFlags, ShmFlag, VAR_STRING, dict_T, exarg_T, int64_t, scid_T, size_t, typval_T,
    uint8_t,
};

use super::{
    EOL_DOS, EOL_MAC, EOL_UNIX, FORCE_BIN, get_option, get_varp, kOptScopeBuf, kOptScopeWin,
    kOptValTypeString, option_has_scope, optval_from_varp, set_option_direct,
};
use crate::state::MODE_TERMINAL;
use crate::winlayer::Win;

/// 'equalprg', local where set.
pub(crate) fn get_equalprg() -> *mut c_char {
    // SAFETY: `curbuf` is live, and its string options are never null.
    if unsafe { *cur_buf().b_p_ep } == 0 {
        p_ep.get()
    } else {
        cur_buf().b_p_ep
    }
}

/// 'findfunc', local where set.
pub(crate) fn get_findfunc() -> *mut c_char {
    // SAFETY: `curbuf` is live, and its string options are never null.
    if unsafe { *cur_buf().b_p_ffu } == 0 {
        p_ffu.get()
    } else {
        cur_buf().b_p_ffu
    }
}

/// Whether 'shortmess' asks for message `x` to be shortened. The `a` flag is
/// an abbreviation standing for these four and nothing else.
pub(crate) fn shortmess(x: ShmFlag) -> bool {
    const ABBREVIATED: [ShmFlag; 4] = [ShmFlag::RO, ShmFlag::MOD, ShmFlag::LINES, ShmFlag::WRI];
    // SAFETY: 'shortmess' is a string option; the null test is upstream's.
    let Some(shm) = (unsafe { cstr::at_opt(p_shm.get()) }) else {
        return false;
    };
    x.is_in(shm) || (ShmFlag::ABBREVIATIONS.is_in(shm) && ABBREVIATED.contains(&x))
}

/// Whether 'cpoptions' contains `flag`.
pub(crate) fn cpo_has(flag: CpoFlag) -> bool {
    // SAFETY: 'cpoptions' is a string option and is never null.
    flag.is_in(unsafe { CStr::from_ptr(p_cpo.get()) })
}

/// Record where a vimrc was found in `$MYVIMRC`/`$MYVIMDIR`, unless the
/// environment variable is already set.
///
/// # Safety
///
/// `fname` and `envname`, when non-null, must be NUL-terminated.
pub(crate) unsafe fn vimrc_found(fname: *mut c_char, envname: *mut c_char) {
    if fname.is_null() || envname.is_null() {
        return;
    }
    // SAFETY: the caller's strings are NUL-terminated.
    let existing = unsafe { vim_getenv(envname) };
    if !existing.is_null() {
        unsafe { xfree(existing.cast::<c_void>()) };
        return;
    }
    let full = unsafe { full_name_save(fname, false) };
    if !full.is_null() {
        unsafe { os_setenv(envname, full, 1) };
        unsafe { xfree(full.cast::<c_void>()) };
    }
}

/// Parse 'cursorlineopt' into `wp->w_p_culopt_flags`, from `val` or from the
/// window's own value. `Err` for a value that does not parse; the flags are
/// only stored on success.
///
/// # Safety
///
/// `val`, where given, must outlive the call.
pub(crate) unsafe fn fill_culopt_flags(val: Option<&CStr>, mut wp: Win) -> Result<(), Failed> {
    let mut p = match val {
        Some(val) => val.as_ptr().cast_mut(),
        None => wp.w_onebuf_opt.wo_culopt,
    };
    let mut flags: uint8_t = 0;
    while unsafe { *p } != 0 {
        for (word, bits) in [
            (c"line".to_bytes(), kOptCuloptFlagLine),
            (
                c"both".to_bytes(),
                kOptCuloptFlagLine | kOptCuloptFlagNumber,
            ),
            (c"number".to_bytes(), kOptCuloptFlagNumber),
            (c"screenline".to_bytes(), kOptCuloptFlagScreenline),
        ] {
            if unsafe { cstr::prefix_eq(p, word.as_ptr().cast::<c_char>(), word.len() as size_t) } {
                p = unsafe { p.add(word.len()) };
                flags |= bits as uint8_t;
                break;
            }
        }
        // Anything the words above did not consume is a syntax error.
        if unsafe { *p } != b',' as c_char && unsafe { *p } != 0 {
            return Err(Failed);
        }
        if unsafe { *p } == b',' as c_char {
            p = unsafe { p.add(1) };
        }
    }
    // "line" and "screenline" are mutually exclusive; "both" implies
    // "line", so it collides with "screenline" too.
    if flags as c_int & kOptCuloptFlagLine as c_int != 0
        && flags as c_int & kOptCuloptFlagScreenline as c_int != 0
    {
        return Err(Failed);
    }
    wp.w_p_culopt_flags = flags;
    Ok(())
}

/// Whether patterns are magic right now — `\v`/`\V` in the pattern override
/// 'magic' for the pattern they appear in.
pub(crate) fn magic_isset() -> bool {
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
pub(crate) unsafe fn option_set_callback_func(
    optval: *mut c_char,
    optcb: *mut Callback,
) -> Result<(), Failed> {
    // SAFETY: the caller's pointers are valid for the call.
    if optval.is_null() || unsafe { *optval } == 0 {
        unsafe { callback_free(optcb) };
        return Ok(());
    }
    // A lambda, `function(...)` or `funcref(...)` is an expression; a
    // bare name is the function's name.
    let tv = if unsafe { *optval } == b'{' as c_char
        || unsafe { cstr::starts_with(optval, b"function(") }
        || unsafe { cstr::starts_with(optval, b"funcref(") }
    {
        let tv = unsafe { eval_expr(optval, ptr::null_mut::<exarg_T>()) };
        if tv.is_null() {
            return Err(Failed);
        }
        tv
    } else {
        let tv = unsafe { xcalloc(1, size_of::<typval_T>()) }.cast::<typval_T>();
        unsafe { (*tv).v_type = VAR_STRING };
        unsafe { (*tv).vval.v_string = xstrdup(optval) };
        tv
    };
    let mut cb = Callback {
        data: Callback_data {
            funcref: ptr::null_mut::<c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !unsafe { callback_from_typval(&raw mut cb, tv) } || cb.type_0 == kCallbackNone {
        unsafe { tv_free(tv) };
        return Err(Failed);
    }
    unsafe { callback_free(optcb) };
    unsafe { *optcb = cb };
    unsafe { tv_free(tv) };
    Ok(())
}

/// Whether 'backspace' allows backspacing over `what`. A prompt buffer never
/// lets the prompt itself be backspaced over.
pub(crate) fn can_bs(what: BsFlag) -> bool {
    if what == BsFlag::START && buf_is_prompt(current_buf()) {
        return false;
    }
    // SAFETY: 'backspace' is a string option, so it is a live, NUL-terminated
    // string.
    // The historic numeric spelling: 2 is everything but "nostop".
    if unsafe { *p_bs.get() } == b'2' as c_char {
        return what != BsFlag::NOSTOP;
    }
    what.is_in(unsafe { CStr::from_ptr(p_bs.get()) })
}

/// 'backupcopy' as flags, local where set.
pub(crate) fn get_bkc_flags(buf: Buf) -> c_uint {
    match buf.b_bkc_flags {
        0 => bkc_flags.get(),
        local => local,
    }
}

/// 'formatlistpat', local where set.
///
pub(crate) fn get_flp_value(buf: Buf) -> *mut c_char {
    // SAFETY: a string option is either null or NUL-terminated.
    if buf.b_p_flp.is_null() || unsafe { *buf.b_p_flp } == 0 {
        p_flp.get()
    } else {
        buf.b_p_flp
    }
}

/// 'virtualedit' as flags, local where set. The two "none" bits only exist
/// so a window can spell out that it overrides the global value with
/// nothing, so they never reach a caller.
///
pub(crate) fn get_ve_flags(wp: Win) -> c_uint {
    let flags = match wp.w_onebuf_opt.wo_ve_flags {
        0 => ve_flags.get(),
        local => local,
    };
    flags & !(kOptVeFlagNone | kOptVeFlagNoneU)
}

/// 'showbreak', local where set. `"NONE"` is how a window says "no leader"
/// against a global value that has one.
///
pub(crate) fn get_showbreak_value(win: Win) -> *mut c_char {
    let local = win.w_onebuf_opt.wo_sbr;
    // SAFETY: a string option is either null or NUL-terminated.
    if local.is_null() || unsafe { *local } == 0 {
        return p_sbr.get();
    }
    if unsafe { cstr::bytes_at(local) == b"NONE" } {
        return empty_option();
    }
    local
}

/// The buffer's line ending. 'binary' forces Unix whatever 'fileformat' says.
///
pub(crate) fn get_fileformat(buf: Buf) -> c_int {
    // SAFETY: 'fileformat' is a string option, so it is never null.
    let c = unsafe { *buf.b_p_ff } as c_uchar;
    if buf.b_p_bin != 0 || c == b'u' {
        EOL_UNIX
    } else if c == b'm' {
        EOL_MAC
    } else {
        EOL_DOS
    }
}

/// [`get_fileformat`] with a command's `++ff`/`++bin` overriding the buffer.
///
/// # Safety
///
/// `eap`, when non-null, must be a live command.
pub(crate) unsafe fn get_fileformat_force(buf: Buf, eap: *const exarg_T) -> c_int {
    // SAFETY: the caller's command, where they gave one. Reading both
    // fields together is the same answer: they are plain fields of a live
    // `exarg_T`, and only their values decide anything below.
    let (force_ff, force_bin) = if eap.is_null() {
        (0, 0)
    } else {
        unsafe { ((*eap).force_ff, (*eap).force_bin) }
    };
    let c = if force_ff != 0 {
        force_ff
    } else {
        let binary = if force_bin != 0 {
            (force_bin == FORCE_BIN) as c_int
        } else {
            buf.b_p_bin
        };
        if binary != 0 {
            return EOL_UNIX;
        }
        // SAFETY: 'fileformat' is a string option, so it is never null.
        (unsafe { *buf.b_p_ff }) as c_uchar as c_int
    };
    match c as u8 {
        b'u' => EOL_UNIX,
        b'm' => EOL_MAC,
        _ => EOL_DOS,
    }
}

/// The line ending a new file gets: the first entry of 'fileformats'.
pub(crate) fn default_fileformat() -> c_int {
    // SAFETY: 'fileformats' is a string option; it is never null.
    match unsafe { *p_ffs.get() } as u8 {
        b'm' => EOL_MAC,
        b'd' => EOL_DOS,
        _ => EOL_UNIX,
    }
}

/// Set 'fileformat' to `eol_style` and redraw what shows it.
pub(crate) fn set_fileformat(eol_style: c_int, opt_flags: OptionSetFlags) {
    let name = match eol_style {
        EOL_UNIX => Some(c"unix"),
        EOL_MAC => Some(c"mac"),
        EOL_DOS => Some(c"dos"),
        _ => None,
    };
    // SAFETY: the names are static; `curbuf` is live.
    if let Some(name) = name {
        set_option_direct(
            kOptFileformat,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: unsafe { cstr_as_string(name.as_ptr().cast_mut()) },
                },
            },
            opt_flags,
            0 as scid_T,
        );
    }
    unsafe { redraw_buf_status_later(curbuf.get()) };
    redraw_tabline.set(true);
    need_maketitle.set(true);
}

/// Step over the separator and any padding between two parts of a
/// comma-separated option.
///
/// # Safety
///
/// `p` must be NUL-terminated.
pub(crate) unsafe fn skip_to_option_part(mut p: *const c_char) -> *mut c_char {
    // SAFETY: the caller's string is NUL-terminated.
    if unsafe { *p } == b',' as c_char {
        p = unsafe { p.add(1) };
    }
    while unsafe { *p } == b' ' as c_char {
        p = unsafe { p.add(1) };
    }
    p.cast_mut()
}

/// Copy one part of a separated option into `buf` and advance `option` past
/// it, answering how many bytes it wrote: the part's length, or
/// `maxlen - 1` when the part is longer than that. A caller may use the
/// answer as the length of what is in the buffer.
///
/// The one exception is a leading `'.'`, which is written and counted
/// without a bound test — a `maxlen` below 2 is not usable here. No caller
/// passes one.
///
/// # Safety
///
/// `option` must point at a NUL-terminated string; `buf` must have room for
/// `maxlen` bytes; `sep_chars` must be NUL-terminated.
pub(crate) unsafe fn copy_option_part(
    option: *mut *mut c_char,
    buf: *mut c_char,
    maxlen: size_t,
    sep_chars: *mut c_char,
) -> size_t {
    // SAFETY: the caller's pointers are valid for the lengths documented.
    let mut len: size_t = 0;
    let mut p = unsafe { *option };
    // A leading '.' is copied without being tested against the
    // separators, so `.` can start a path entry.
    if unsafe { *p } == b'.' as c_char {
        unsafe { *buf = *p };
        p = unsafe { p.add(1) };
        len = 1;
    }
    while unsafe { *p } != 0 && unsafe { vim_strchr(sep_chars, *p as uint8_t as c_int) }.is_null() {
        // A backslash escapes a separator, and is dropped.
        if unsafe { *p } == b'\\' as c_char
            && !unsafe { vim_strchr(sep_chars, *p.add(1) as uint8_t as c_int) }.is_null()
        {
            p = unsafe { p.add(1) };
        }
        if len < maxlen.wrapping_sub(1) {
            unsafe { *buf.add(len) = *p };
            len += 1;
        }
        p = unsafe { p.add(1) };
    }
    unsafe { *buf.add(len) = NUL as c_char };
    // Step over the separator we stopped on — unless it is a comma,
    // which `skip_to_option_part` handles along with the padding.
    if unsafe { *p } != 0 && unsafe { *p } != b',' as c_char {
        p = unsafe { p.add(1) };
    }
    unsafe { *option = skip_to_option_part(p) };
    len
}

/// Whether 'shell' is a csh derivative, which needs its own quoting.
pub(crate) fn csh_like_shell() -> bool {
    // SAFETY: 'shell' is a string option; it is never null.
    unsafe { !strstr(path_tail(p_sh.get()), c"csh".as_ptr()).is_null() }
}

/// Whether 'shell' is fish, which needs its own quoting.
pub(crate) fn fish_like_shell() -> bool {
    // SAFETY: 'shell' is a string option; it is never null.
    unsafe { !strstr(path_tail(p_sh.get()), c"fish".as_ptr()).is_null() }
}

/// Every buffer-local (or window-local) option of the current buffer and
/// window, as a dictionary — what `b:` and `w:` expose.
pub(crate) fn get_winbuf_options(bufopt: c_int) -> *mut dict_T {
    let scope = if bufopt != 0 {
        kOptScopeBuf
    } else {
        kOptScopeWin
    };
    // SAFETY: the option table is a plain array, and `get_varp` hands back
    // the variable for the current buffer and window.
    let d = unsafe { tv_dict_alloc() };
    for opt_idx in kOptAleph..kOptCount {
        if !option_has_scope(opt_idx, scope) {
            continue;
        }
        let varp = get_varp(opt_idx);
        if varp.is_none() {
            continue;
        }
        let mut tv = unsafe { optval_as_tv(optval_from_varp(opt_idx, varp), true) };
        let name = get_option(opt_idx).fullname;
        let _ = unsafe { tv_dict_add_tv(d, name, cstr::bytes_at(name).len(), &raw mut tv) };
    }
    d
}

/// 'scrolloff' for a window, local where set. A terminal buffer never
/// scrolls off, whatever the option says.
///
pub(crate) fn get_scrolloff_value(wp: Win) -> int64_t {
    // SAFETY: a window that is being scrolled has a buffer.
    if State.get() & MODE_TERMINAL != 0 && !unsafe { (*wp.w_buffer).terminal }.is_null() {
        return 0;
    }
    match wp.w_onebuf_opt.wo_so {
        local if local < 0 => p_so.get(),
        local => local,
    }
}

/// 'sidescrolloff' for a window, local where set.
///
pub(crate) fn get_sidescrolloff_value(wp: Win) -> int64_t {
    match wp.w_onebuf_opt.wo_siso {
        local if local < 0 => p_siso.get(),
        local => local,
    }
}

/// Which scroll margin: 'scrolloff', in lines, or 'sidescrolloff', in
/// columns.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollMargin {
    Lines,
    Columns,
}

/// The scroll margin in effect for a window: its own value where it has set
/// one, the global option otherwise.
///
/// C reaches this through an `OptInt *`, because the three callers that want
/// it *write* through it -- `showmatch`, `do_ecmd`'s `recenter` and
/// `update_topline`'s mouse-drag arm each set the margin aside for one
/// redraw and put it back. Resolving the fallback once and answering a
/// `get`/`set` pair is the same thing without the address, which is also
/// what keeps the write from landing in the wrong scope.
#[derive(Clone, Copy)]
pub(crate) enum ScrollOff {
    /// The window's own value, which it has set.
    Window(Win, ScrollMargin),
    /// The global option, because the window has not set its own.
    Global(ScrollMargin),
}

impl ScrollOff {
    /// The margin `win` reads right now.
    pub(crate) fn of(win: Win, margin: ScrollMargin) -> Self {
        let local = match margin {
            ScrollMargin::Lines => win.w_onebuf_opt.wo_so,
            ScrollMargin::Columns => win.w_onebuf_opt.wo_siso,
        };
        if local >= 0 {
            Self::Window(win, margin)
        } else {
            Self::Global(margin)
        }
    }

    /// The value in effect.
    pub(crate) fn get(self) -> OptInt {
        match self {
            Self::Window(win, ScrollMargin::Lines) => win.w_onebuf_opt.wo_so,
            Self::Window(win, ScrollMargin::Columns) => win.w_onebuf_opt.wo_siso,
            Self::Global(ScrollMargin::Lines) => p_so.get(),
            Self::Global(ScrollMargin::Columns) => p_siso.get(),
        }
    }

    /// Write it back where it came from.
    pub(crate) fn set(self, value: OptInt) {
        match self {
            Self::Window(mut win, ScrollMargin::Lines) => win.w_onebuf_opt.wo_so = value,
            Self::Window(mut win, ScrollMargin::Columns) => win.w_onebuf_opt.wo_siso = value,
            Self::Global(ScrollMargin::Lines) => p_so.set(value),
            Self::Global(ScrollMargin::Columns) => p_siso.set(value),
        }
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
