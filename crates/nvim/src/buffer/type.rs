//! What kind of buffer is this -- the `'buftype'` predicates.
//!
//! The `bt_*` family answers `'buftype'` questions the rest of the editor
//! asks constantly -- is this a help buffer, a quickfix list, a terminal, a
//! prompt; does it have a file name; may it be written -- and
//! [`buf_spname`] gives the special buffers the name that is displayed
//! instead of a file.  [`buf_hide`] is the `'hidden'`/`'bufhidden'` decision,
//! [`set_buflisted`] the `'buflisted'` half, and the `changedtick` pair the
//! `b:changedtick` counter every change bumps.
//!
//! Every predicate here is a raw entry point taking `buf_T *`, and most
//! accept null for "no buffer", so each begins by turning its pointer into
//! an `Option<Buf>` -- one unchecked line apiece, after which the body is
//! ordinary field access through [`Buf`]'s `Deref`.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{EVENT_BUFADD, EVENT_BUFDELETE, apply_autocmds};
use crate::eval::typval::{tv_dict_find, tv_dict_is_watched, tv_dict_watcher_notify};
use crate::main::{cmdmod, cmdwin_buf, curbuf, msg_loclist, msg_qflist, p_hid};
use crate::memline::ml_get_buf;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::quickfix::qf_stack_get_bufnr;
use crate::types::{
    CMOD_HIDE, VAR_FIXED, VAR_NUMBER, buf_T, dictitem_T, linenr_T, ptrdiff_t, typval_T, varnumber_T,
};
use crate::winlayer::Buf;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `_()` over a pointer: the message catalogue's translation of a string
/// that is a literal here or was set once at startup.
fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: every caller passes a NUL-terminated literal or one of the two
    // quickfix titles, which `qf_init` sets from the catalogue at startup.
    unsafe { gettext(msg) }
}

/// `_()`.
fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// The buffer behind a pointer the caller has promised is null or live.
///
/// # Safety
/// `buf` is null or points at a live buffer.
unsafe fn opt_buf(buf: *const buf_T) -> Option<Buf> {
    // SAFETY: the caller's promise. Nothing below writes through the result,
    // so dropping `const` costs nothing.
    (!buf.is_null()).then(|| unsafe { Buf::new(buf.cast_mut()) })
}

/// The first byte of `'buftype'`, or NUL when there is no buffer. Option
/// variables are never null, so upstream indexes `b_p_bt` unconditionally.
fn buftype(buf: Option<Buf>) -> c_char {
    // SAFETY: an option variable holds a NUL-terminated string, so its
    // first byte is there to be read.
    buf.map_or(0, |b| unsafe { *b.b_p_bt })
}

/// `b_p_bt[2]`, which upstream reads only once `b_p_bt[0] == 'n'` has said
/// there are at least three bytes ("nofile" or "nowrite") to read.
fn buftype_2(buf: Buf) -> c_char {
    // SAFETY: a `'buftype'` beginning with 'n' is one of those two words.
    unsafe { *buf.b_p_bt.add(2) }
}

fn has_terminal(buf: Buf) -> bool {
    !buf.terminal.is_null()
}

/// One byte of `'buftype'`, as a `char`.
const fn ch(byte: u8) -> c_char {
    byte as c_char
}

// ---------------------------------------------------------------------------
// The 'buftype' predicates

pub unsafe extern "C" fn bt_prompt(buf: *mut buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    buftype(unsafe { opt_buf(buf) }) == ch(b'p')
}

/// [`bt_prompt`] over a buffer already wrapped.
fn is_prompt(buf: Buf) -> bool {
    buftype(Some(buf)) == ch(b'p')
}

pub unsafe extern "C" fn bt_help(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    unsafe { opt_buf(buf) }.is_some_and(|b| b.b_help)
}

/// A normal buffer: `'buftype'` is empty.
pub unsafe extern "C" fn bt_normal(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    let buf = unsafe { opt_buf(buf) };
    buf.is_some() && buftype(buf) == 0
}

pub unsafe extern "C" fn bt_quickfix(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    buftype(unsafe { opt_buf(buf) }) == ch(b'q')
}

/// [`bt_quickfix`] over a buffer already wrapped, for the callers in this
/// file.
fn is_quickfix(buf: Buf) -> bool {
    buftype(Some(buf)) == ch(b'q')
}

pub unsafe extern "C" fn bt_terminal(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    buftype(unsafe { opt_buf(buf) }) == ch(b't')
}

/// A "nofile", "acwrite", terminal or "prompt" buffer: its name may not be a
/// file name, at least not one to write to.
pub unsafe extern "C" fn bt_nofilename(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    unsafe { opt_buf(buf) }.is_some_and(is_nofilename)
}

/// [`bt_nofilename`] over a buffer already wrapped.
fn is_nofilename(buf: Buf) -> bool {
    let bt = buftype(Some(buf));
    bt == ch(b'n') && buftype_2(buf) == ch(b'f')
        || bt == ch(b'a')
        || has_terminal(buf)
        || bt == ch(b'p')
}

/// A "nofile", "quickfix", terminal or "prompt" buffer: not to be read from
/// a file.
pub(crate) unsafe extern "C" fn bt_nofileread(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    let Some(buf) = (unsafe { opt_buf(buf) }) else {
        return false;
    };
    let bt = buftype(Some(buf));
    bt == ch(b'n') && buftype_2(buf) == ch(b'f')
        || bt == ch(b't')
        || bt == ch(b'q')
        || bt == ch(b'p')
}

pub unsafe extern "C" fn bt_nofile(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    let Some(buf) = (unsafe { opt_buf(buf) }) else {
        return false;
    };
    buftype(Some(buf)) == ch(b'n') && buftype_2(buf) == ch(b'f')
}

/// A "nowrite", "nofile", terminal or "prompt" buffer.
pub unsafe extern "C" fn bt_dontwrite(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    unsafe { opt_buf(buf) }.is_some_and(is_dontwrite)
}

/// [`bt_dontwrite`] over a buffer already wrapped.
fn is_dontwrite(buf: Buf) -> bool {
    let bt = buftype(Some(buf));
    bt == ch(b'n') || has_terminal(buf) || bt == ch(b'p')
}

pub unsafe extern "C" fn bt_dontwrite_msg(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    if unsafe { opt_buf(buf) }.is_some_and(is_dontwrite) {
        // SAFETY: a translated message literal.
        unsafe { emsg(tr(c"E382: Cannot write, 'buftype' option is set")) };
        return true;
    }
    false
}

/// Whether the buffer should be hidden rather than unloaded, according to
/// `'bufhidden'`, `'hidden'` and `:hide`.
pub unsafe extern "C" fn buf_hide(buf: *const buf_T) -> bool {
    // SAFETY: the caller's promise -- a live buffer. Upstream dereferences
    // this one without a null test.
    let bufhidden = unsafe { *(*buf).b_p_bh };
    match bufhidden as u8 {
        b'u' | b'w' | b'd' => return false, // "unload", "wipe", "delete"
        b'h' => return true,                // "hide"
        _ => {}
    }
    p_hid.get() != 0 || cmdmod.with(|m| m.cmod_flags) & CMOD_HIDE as c_int != 0
}

// ---------------------------------------------------------------------------
// The name a buffer without a file is shown under

/// The name to display for a special buffer, or null for an ordinary one.
pub unsafe extern "C" fn buf_spname(buf: *mut buf_T) -> *mut c_char {
    // SAFETY: the caller's promise -- a live buffer.
    let b = unsafe { Buf::new(buf) };
    if is_quickfix(b) {
        if b.handle == qf_stack_get_bufnr() {
            return tr_raw(msg_qflist.get());
        }
        return tr_raw(msg_loclist.get());
    }
    if is_nofilename(b) {
        if !b.b_fname.is_null() {
            return b.b_fname;
        }
        if buf == cmdwin_buf.get() {
            return tr(c"[Command Line]");
        }
        if is_prompt(b) {
            return tr(c"[Prompt]");
        }
        return tr(c"[Scratch]");
    }
    if b.b_fname.is_null() {
        return tr(c"[No Name]");
    }
    ptr::null_mut()
}

pub unsafe extern "C" fn buf_get_fname(buf: *const buf_T) -> *mut c_char {
    // SAFETY: the caller's promise -- a live buffer.
    let name = unsafe { (*buf).b_fname };
    if name.is_null() {
        return tr(c"[No Name]");
    }
    name
}

// ---------------------------------------------------------------------------
// 'buflisted', emptiness and b:changedtick

/// Set `'buflisted'` for the current buffer, firing `BufAdd`/`BufDelete` if
/// it changed.
pub unsafe extern "C" fn set_buflisted(on: c_int) {
    // SAFETY: `curbuf` is set from startup to exit.
    let mut buf = unsafe { Buf::current() };
    if on == buf.b_p_bl {
        return;
    }
    buf.b_p_bl = on;
    let event = if on != 0 {
        EVENT_BUFADD
    } else {
        EVENT_BUFDELETE
    };
    let raw = curbuf.get();
    // SAFETY: a live buffer; both name arguments are optional.
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, raw) };
}

pub unsafe extern "C" fn buf_is_empty(buf: *mut buf_T) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    let b = unsafe { Buf::new(buf) };
    // SAFETY: line 1 exists in every buffer, and `ml_get_buf` answers a
    // NUL-terminated line.
    b.b_ml.ml_line_count == 1 as linenr_T && unsafe { *ml_get_buf(buf, 1 as linenr_T) } == 0
}

pub unsafe extern "C" fn buf_inc_changedtick(buf: *mut buf_T) {
    // SAFETY: the caller's promise -- a live buffer.
    unsafe { buf_set_changedtick(buf, buf_get_changedtick(buf) + 1 as varnumber_T) };
}

/// Set `b:changedtick`, telling any `b:` watcher about the change.
pub unsafe extern "C" fn buf_set_changedtick(buf: *mut buf_T, changedtick: varnumber_T) {
    // SAFETY: the caller's promise -- a live buffer.
    let mut b = unsafe { Buf::new(buf) };
    let mut old_val: typval_T = b.changedtick_di.di_tv;
    check_changedtick_item(b);
    b.changedtick_di.di_tv.vval.v_number = changedtick;
    // SAFETY: `b_vars` is the buffer's own dictionary, allocated with it.
    if unsafe { tv_dict_is_watched(b.b_vars) } {
        b.b_locked += 1;
        let vars = b.b_vars;
        let key = (&raw mut b.changedtick_di.di_key).cast::<c_char>();
        let new = &raw mut b.changedtick_di.di_tv;
        // SAFETY: the buffer's own dictionary and its `changedtick` entry,
        // plus a local holding the value it had.
        unsafe { tv_dict_watcher_notify(vars, key, new, &raw mut old_val) };
        b.b_locked -= 1;
    }
}

/// The consistency checks upstream wraps in `#ifndef NDEBUG`: `b:` must
/// still hold the fixed, read-only number `buf_init_changedtick` put there.
fn check_changedtick_item(buf: Buf) {
    if !cfg!(debug_assertions) {
        return;
    }
    let vars = buf.b_vars;
    let key = c"changedtick";
    let keylen = key.count_bytes() as ptrdiff_t;
    // SAFETY: the buffer's own dictionary; the key is a literal with its
    // length, as `S_LEN` spells it.
    let di = unsafe { tv_dict_find(vars, key.as_ptr(), keylen) };
    assert!(!di.is_null(), "changedtick_di != NULL");
    // SAFETY: non-null, and `tv_dict_find` answers a live dictionary item.
    let item = unsafe { *di };
    assert!(
        item.di_tv.v_type == VAR_NUMBER as _,
        "changedtick_di->di_tv.v_type == VAR_NUMBER"
    );
    assert!(
        item.di_tv.v_lock == VAR_FIXED as _,
        "changedtick_di->di_tv.v_lock == VAR_FIXED"
    );
    assert!(
        item.di_flags as c_int == DI_FLAGS_RO as c_int | DI_FLAGS_FIX as c_int,
        "changedtick_di->di_flags == (DI_FLAGS_RO|DI_FLAGS_FIX)"
    );
    assert!(
        di == (&raw const buf.changedtick_di)
            .cast::<dictitem_T>()
            .cast_mut(),
        "changedtick_di == (dictitem_T *)&buf->changedtick_di"
    );
}
