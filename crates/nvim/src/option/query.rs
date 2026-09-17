//! The accessors the rest of the editor reads options through.
//!
//! Most of these exist because a value is not simply "the variable": a
//! global-local option falls back to the global when the local copy is
//! unset, 'shortmess' has an abbreviation that stands for four other flags,
//! 'virtualedit' and 'cursorlineopt' are read as parsed flag words, and
//! 'scrolloff' is forced to zero in a terminal buffer.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::strings::has_bytes;
use crate::strings::has_char;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_uchar, c_uint, c_void};
use core::mem::ManuallyDrop;

use crate::buffer::{buf_is_prompt, current_buf};
use crate::cstr;
use crate::drawscreen::redraw_buf_status_later;
use crate::drawscreen::state::{need_maketitle, redraw_tabline};
use crate::eval::typval::{callback_free, tv_dict_alloc, tv_free};
use crate::eval::vars::optval_as_tv;
use crate::eval::{callback_from_typval, eval_expr};
use crate::memory::{XString, xcalloc, xfree, xstrdup};
use crate::option::vars::{
    P_SISO, P_SO, bkc_flags, p_bs, p_cpo, p_ffs, p_magic, p_sh, p_shm, p_siso, p_so, ve_flags,
};
use crate::options::*;
use crate::optionstr::empty_option;
use crate::os::env::{os_setenv, vim_getenv};
use crate::path::{full_name_save, path_tail};
use crate::regexp::state::{OPTION_MAGIC_OFF, OPTION_MAGIC_ON};
use crate::search::state::magic_overruled;
use crate::state::mode::State;
use crate::types::{
    BsFlag, Callback, CpoFlag, Dict, ExArg, Failed, NUL, OptInt, OptVal, OptionSetFlags, ScriptId,
    ShmFlag, TypVal, int64_t, size_t, uint8_t,
};

use super::{
    EOL_DOS, EOL_MAC, EOL_UNIX, FORCE_BIN, get_option, get_varp, kOptScopeBuf, kOptScopeWin,
    option_has_scope, optval_from_varp, set_option_direct,
};
use crate::state::MODE_TERMINAL;
use crate::winlayer::Win;

/// A global-local string option's value in force: the buffer's or window's
/// own copy where it set one, else the global value.
///
/// The answer is a **copy**, because the two halves cannot be one borrow:
/// the local copy is a raw field the option layer owns and the global value
/// is a string the option record owns, and every caller here walks the
/// answer or hands it to a C callee past the end of a projection.
///
/// # Safety
///
/// `local` must be the live, NUL-terminated local copy of `global`'s option.
pub(crate) unsafe fn local_or_global(local: *const c_char, global: StrOpt) -> XString {
    // SAFETY: the caller's value.
    let local = unsafe { cstr::at(local) };
    if local.is_empty() {
        global.get()
    } else {
        XString::from_cstr(local)
    }
}

/// [`local_or_global`] as the raw pointer the draw paths still read.
///
/// The answer is the option's *own* buffer -- the global record's string or
/// the buffer's or window's raw field -- and is live until that option is
/// written. Only the readers that are asked several times per screen line
/// use this; everything else takes the copy. When the local copies own their
/// storage too, both halves become one projection and this goes.
///
/// # Safety
///
/// `local` must be the live, NUL-terminated local copy of `global`'s option,
/// and the answer must not outlive a write to either.
unsafe fn local_or_global_raw(local: *const c_char, global: StrOpt) -> *mut c_char {
    // SAFETY: the caller's value.
    if unsafe { *local } == 0 {
        global.value_ptr()
    } else {
        local.cast_mut()
    }
}

/// 'equalprg', local where set.
pub(crate) fn get_equalprg() -> XString {
    // SAFETY: `curbuf` is live, and its string options are never null.
    unsafe { local_or_global(Buf::current().b_p_ep, P_EP) }
}

/// 'findfunc', local where set.
pub(crate) fn get_findfunc() -> XString {
    // SAFETY: `curbuf` is live, and its string options are never null.
    unsafe { local_or_global(Buf::current().b_p_ffu, P_FFU) }
}

/// Whether 'shortmess' asks for message `x` to be shortened. The `a` flag is
/// an abbreviation standing for these four and nothing else.
pub(crate) fn shortmess(x: ShmFlag) -> bool {
    const ABBREVIATED: [ShmFlag; 4] = [ShmFlag::RO, ShmFlag::MOD, ShmFlag::LINES, ShmFlag::WRI];
    p_shm(|shm| x.is_in(shm) || (ShmFlag::ABBREVIATIONS.is_in(shm) && ABBREVIATED.contains(&x)))
}

/// Whether 'cpoptions' contains `flag`.
pub(crate) fn cpo_has(flag: CpoFlag) -> bool {
    p_cpo(|cpo| flag.is_in(cpo))
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

/// Parse 'cursorlineopt' into `window.w_p_culopt_flags`, from `val` or from the
/// window's own value. `Err` for a value that does not parse; the flags are
/// only stored on success.
pub(crate) fn fill_culopt_flags(val: Option<&CStr>, mut window: Win) -> Result<(), Failed> {
    let mut p = match val {
        Some(val) => val.as_ptr().cast_mut(),
        None => window.w_onebuf_opt.wo_culopt,
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
    window.w_p_culopt_flags = flags;
    Ok(())
}

/// Whether patterns are magic right now — `\v`/`\V` in the pattern override
/// 'magic' for the pattern they appear in.
pub(crate) fn magic_isset() -> bool {
    match magic_overruled.get() {
        OPTION_MAGIC_ON => true,
        OPTION_MAGIC_OFF => false,
        _ => p_magic(),
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
        let tv = unsafe { eval_expr(optval, None) };
        if tv.is_null() {
            return Err(Failed);
        }
        tv
    } else {
        let tv = unsafe { xcalloc(1, size_of::<TypVal>()) }.cast::<TypVal>();
        unsafe { (*tv).write_string(xstrdup(optval)) };
        tv
    };
    let mut cb = Callback::None;
    if !unsafe { callback_from_typval(&raw mut cb, &*tv) } || !cb.is_set() {
        unsafe { tv_free(tv.as_mut()) };
        return Err(Failed);
    }
    unsafe { callback_free(optcb) };
    unsafe { *optcb = cb };
    unsafe { tv_free(tv.as_mut()) };
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
    if p_bs(|value| cstr::first(value) == b'2') {
        return what != BsFlag::NOSTOP;
    }
    what.is_in(p_bs(|value| unsafe {
        CStr::from_ptr(value.as_ptr().cast_mut())
    }))
}

/// 'backupcopy' as flags, local where set.
pub(crate) fn get_bkc_flags(buffer: Buf) -> c_uint {
    match buffer.b_bkc_flags {
        0 => bkc_flags.get(),
        local => local,
    }
}

/// 'formatlistpat', local where set.
///
pub(crate) fn get_flp_value(buffer: Buf) -> *mut c_char {
    // SAFETY: a string option is either null or NUL-terminated.
    if buffer.b_p_flp.is_null() {
        return P_FLP.value_ptr();
    }
    unsafe { local_or_global_raw(buffer.b_p_flp, P_FLP) }
}

/// 'virtualedit' as flags, local where set. The two "none" bits only exist
/// so a window can spell out that it overrides the global value with
/// nothing, so they never reach a caller.
///
pub(crate) fn get_ve_flags(window: Win) -> c_uint {
    let flags = match window.w_onebuf_opt.wo_ve_flags {
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
    if local.is_null() {
        return P_SBR.value_ptr();
    }
    // SAFETY: a string option is NUL-terminated.
    if unsafe { cstr::eq_bytes(local, b"NONE") } {
        return empty_option();
    }
    unsafe { local_or_global_raw(local, P_SBR) }
}

/// The buffer's line ending. 'binary' forces Unix whatever 'fileformat' says.
///
pub(crate) fn get_fileformat(buffer: Buf) -> c_int {
    // SAFETY: 'fileformat' is a string option, so it is never null.
    let c = unsafe { *buffer.b_p_ff } as c_uchar;
    if buffer.b_p_bin != 0 || c == b'u' {
        EOL_UNIX
    } else if c == b'm' {
        EOL_MAC
    } else {
        EOL_DOS
    }
}

/// [`get_fileformat`] with a command's `++ff`/`++bin` overriding the buffer.
///
pub(crate) fn get_fileformat_force(buffer: Buf, excmd: Option<&ExArg>) -> c_int {
    let (force_ff, force_bin) =
        excmd.map_or((0, 0), |command| (command.force_ff, command.force_bin));
    let c = if force_ff != 0 {
        force_ff
    } else {
        let binary = if force_bin != 0 {
            (force_bin == FORCE_BIN) as c_int
        } else {
            buffer.b_p_bin
        };
        if binary != 0 {
            return EOL_UNIX;
        }
        // SAFETY: 'fileformat' is a string option, so it is never null.
        (unsafe { *buffer.b_p_ff }) as c_uchar as c_int
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
    match p_ffs(cstr::first) {
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
            OptVal::static_string(name),
            opt_flags,
            0 as ScriptId,
        );
    }
    redraw_buf_status_later(Buf::current());
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
    let seps = unsafe { cstr::at(sep_chars) };
    // A leading '.' is copied without being tested against the
    // separators, so `.` can start a path entry.
    if unsafe { *p } == b'.' as c_char {
        unsafe { *buf = *p };
        p = unsafe { p.add(1) };
        len = 1;
    }
    loop {
        let c = unsafe { *p };
        if c == 0 || has_char(seps, c_int::from(c as uint8_t)) {
            break;
        }
        // A backslash escapes a separator, and is dropped. Reading the byte
        // after `c` is in bounds because `c` is not the terminator.
        if c == b'\\' as c_char && has_char(seps, c_int::from(unsafe { *p.add(1) } as uint8_t)) {
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
    // SAFETY: an option value is a C string, and `path_tail` answers a
    // position inside it.
    p_sh(|sh| {
        has_bytes(
            unsafe { cstr::at(path_tail(sh.as_ptr().cast_mut())) },
            b"csh",
        )
    })
}

/// Whether 'shell' is fish, which needs its own quoting.
pub(crate) fn fish_like_shell() -> bool {
    // SAFETY: as [`csh_like_shell`].
    p_sh(|sh| {
        has_bytes(
            unsafe { cstr::at(path_tail(sh.as_ptr().cast_mut())) },
            b"fish",
        )
    })
}

/// Every buffer-local (or window-local) option of the current buffer and
/// window, as a dictionary — what `b:` and `w:` expose.
pub(crate) fn get_winbuf_options(bufopt: c_int) -> *mut Dict {
    let scope = if bufopt != 0 {
        kOptScopeBuf
    } else {
        kOptScopeWin
    };
    // SAFETY: the option table is a plain array, and `get_varp` hands back
    // the variable for the current buffer and window.
    let d_held = tv_dict_alloc();
    let d = d_held.as_ptr();
    for opt_idx in kOptAleph..kOptCount {
        if !option_has_scope(opt_idx, scope) {
            continue;
        }
        let varp = get_varp(opt_idx);
        if varp.is_none() {
            continue;
        }
        // The value names the option's own storage; the dictionary takes a
        // copy, so this releases nothing.
        let tv = ManuallyDrop::new(unsafe { optval_as_tv(optval_from_varp(opt_idx, varp), true) });
        let name = get_option(opt_idx).fullname;
        let _ = unsafe { (*d).add_tv(cstr::bytes_at(name), &tv) };
    }
    // The caller takes the reference over.
    d_held.into_raw()
}

/// 'scrolloff' for a window, local where set. A terminal buffer never
/// scrolls off, whatever the option says.
///
pub(crate) fn get_scrolloff_value(window: Win) -> int64_t {
    // SAFETY: a window that is being scrolled has a buffer.
    if State.get() & MODE_TERMINAL != 0 && !unsafe { (*window.w_buffer).terminal }.is_null() {
        return 0;
    }
    match window.w_onebuf_opt.wo_so {
        local if local < 0 => p_so(),
        local => local,
    }
}

/// 'sidescrolloff' for a window, local where set.
///
pub(crate) fn get_sidescrolloff_value(window: Win) -> int64_t {
    match window.w_onebuf_opt.wo_siso {
        local if local < 0 => p_siso(),
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
            Self::Global(ScrollMargin::Lines) => p_so(),
            Self::Global(ScrollMargin::Columns) => p_siso(),
        }
    }

    /// Write it back where it came from.
    pub(crate) fn set(self, value: OptInt) {
        match self {
            Self::Window(mut win, ScrollMargin::Lines) => win.w_onebuf_opt.wo_so = value,
            Self::Window(mut win, ScrollMargin::Columns) => win.w_onebuf_opt.wo_siso = value,
            Self::Global(ScrollMargin::Lines) => P_SO.set(value),
            Self::Global(ScrollMargin::Columns) => P_SISO.set(value),
        }
    }
}
