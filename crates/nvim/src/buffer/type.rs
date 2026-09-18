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
//! Upstream's predicates are the `bt_*` family and every one of them opens
//! with `buf != NULL &&`.  Here they are `buf_is_*` and take an
//! `Option<Buf>`, which puts that null test in the type and makes the whole
//! family safe: the body is then ordinary field access through [`Buf`]'s
//! `Deref`.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::types::AutoEvent;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::apply_autocmds;
use crate::eval::typval::{dict_find, dict_is_watched, dict_watcher_notify};
use crate::ex_docmd::cmdmod_has;
use crate::message::emsg_ptr;
use crate::option::vars::p_hid;
use crate::optionstr::LocalOptStr;
use crate::os::cshim::gettext_ptr;
use crate::quickfix::qf_stack_get_bufnr;
use crate::quickfix::{msg_loclist, msg_qflist};
use crate::types::{CmdModFlags, DictItem, LineNr, TypVal, VAR_NUMBER, VarLock, VarNumber};
use crate::winlayer::Buf;
use crate::winlayer::graph::cmdwin_buf;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `_()` over a pointer: the message catalogue's translation of a string
/// that is a literal here or was set once at startup.
fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: every caller passes a NUL-terminated literal or one of the two
    // quickfix titles, which `qf_init` sets from the catalogue at startup.
    unsafe { gettext_ptr(msg).as_ptr().cast_mut() }
}

/// `_()`.
fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// The first byte of `'buftype'`, or NUL when there is no buffer. Option
/// variables are never null, so upstream indexes `b_p_bt` unconditionally.
fn buftype(buffer: Option<Buf>) -> u8 {
    buffer.map_or(0, |b| b.b_p_bt.first_byte())
}

/// `b_p_bt[2]`, which upstream reads only once `b_p_bt[0] == 'n'` has said
/// there are at least three bytes ("nofile" or "nowrite") to read.
fn buftype_2(buffer: Buf) -> u8 {
    // A `'buftype'` beginning with 'n' is one of those two words.
    buffer.b_p_bt.bytes()[2]
}

fn has_terminal(buffer: Buf) -> bool {
    !buffer.terminal.is_null()
}

// ---------------------------------------------------------------------------
// The 'buftype' predicates
//
// Upstream spells these `bt_prompt`, `bt_help`, `bt_normal`, `bt_quickfix`,
// `bt_terminal`, `bt_nofilename`, `bt_nofileread`, `bt_nofile`,
// `bt_dontwrite` and `bt_dontwrite_msg`, and every one of them opens with
// `buf != NULL &&`.  That test is the whole reason each takes an
// `Option<Buf>`: the absence is in the type, so the predicates are safe and
// answer `false` for "no buffer" exactly as the C does.

/// `bt_prompt()`: a "prompt" buffer.
pub(crate) fn buf_is_prompt(buffer: Option<Buf>) -> bool {
    buftype(buffer) == b'p'
}

/// `bt_help()`: a help buffer.
pub(crate) fn buf_is_help(buffer: Option<Buf>) -> bool {
    buffer.is_some_and(|b| b.b_help)
}

/// `bt_normal()`: a normal buffer, `'buftype'` empty.
pub(crate) fn buf_is_normal(buffer: Option<Buf>) -> bool {
    buffer.is_some() && buftype(buffer) == 0
}

/// `bt_quickfix()`: the quickfix or location list buffer.
pub(crate) fn buf_is_quickfix(buffer: Option<Buf>) -> bool {
    buftype(buffer) == b'q'
}

/// `bt_terminal()`: a terminal buffer.
pub(crate) fn buf_is_terminal(buffer: Option<Buf>) -> bool {
    buftype(buffer) == b't'
}

/// `bt_nofilename()`: a "nofile", "acwrite", terminal or "prompt" buffer.
/// Its name may not be a file name, at least not one to write to.
pub(crate) fn buf_is_nofilename(buffer: Option<Buf>) -> bool {
    buffer.is_some_and(is_nofilename)
}

/// [`buf_is_nofilename`] over a buffer already in hand.
fn is_nofilename(buffer: Buf) -> bool {
    let bt = buftype(Some(buffer));
    bt == b'n' && buftype_2(buffer) == b'f' || bt == b'a' || has_terminal(buffer) || bt == b'p'
}

/// `bt_nofileread()`: a "nofile", "quickfix", terminal or "prompt" buffer,
/// not to be read from a file.
pub(crate) fn buf_is_nofileread(buffer: Option<Buf>) -> bool {
    buffer.is_some_and(|b| {
        let bt = buftype(Some(b));
        bt == b'n' && buftype_2(b) == b'f' || bt == b't' || bt == b'q' || bt == b'p'
    })
}

/// `bt_nofile()`: a "nofile" buffer.
pub(crate) fn buf_is_nofile(buffer: Option<Buf>) -> bool {
    buffer.is_some_and(|b| buftype(Some(b)) == b'n' && buftype_2(b) == b'f')
}

/// `bt_dontwrite()`: a "nowrite", "nofile", terminal or "prompt" buffer.
pub(crate) fn buf_is_dontwrite(buffer: Option<Buf>) -> bool {
    buffer.is_some_and(is_dontwrite)
}

/// [`buf_is_dontwrite`] over a buffer already in hand.
fn is_dontwrite(buffer: Buf) -> bool {
    let bt = buftype(Some(buffer));
    bt == b'n' || has_terminal(buffer) || bt == b'p'
}

/// `bt_dontwrite_msg()`: [`buf_is_dontwrite`], complaining when it is true.
pub(crate) fn buf_dontwrite_msg(buffer: Option<Buf>) -> bool {
    if buffer.is_some_and(is_dontwrite) {
        // SAFETY: a translated message literal.
        unsafe { emsg_ptr(tr(c"E382: Cannot write, 'buftype' option is set")) };
        return true;
    }
    false
}

/// Whether the buffer should be hidden rather than unloaded, according to
/// `'bufhidden'`, `'hidden'` and `:hide`.
pub fn buf_hide(buffer: Buf) -> bool {
    match buffer.b_p_bh.first_byte() {
        b'u' | b'w' | b'd' => return false, // "unload", "wipe", "delete"
        b'h' => return true,                // "hide"
        _ => {}
    }
    p_hid() || cmdmod_has(CmdModFlags::HIDE)
}

// ---------------------------------------------------------------------------
// The name a buffer without a file is shown under

/// The name to display for a special buffer, or null for an ordinary one.
pub fn buf_spname(buffer: Buf) -> *mut c_char {
    let b = buffer;
    if buf_is_quickfix(Some(b)) {
        if b.handle == qf_stack_get_bufnr() {
            return tr_raw(msg_qflist.as_ptr().cast_mut());
        }
        return tr_raw(msg_loclist.as_ptr().cast_mut());
    }
    if buf_is_nofilename(Some(b)) {
        if !b.name.is_unnamed() {
            return b.name.shown_ptr();
        }
        if cmdwin_buf.get() == Some(buffer.id()) {
            return tr(c"[Command Line]");
        }
        if buf_is_prompt(Some(b)) {
            return tr(c"[Prompt]");
        }
        return tr(c"[Scratch]");
    }
    if b.name.is_unnamed() {
        return tr(c"[No Name]");
    }
    ptr::null_mut()
}

pub fn buf_get_fname(buffer: Buf) -> *mut c_char {
    let name = buffer.name.shown_ptr();
    if name.is_null() {
        return tr(c"[No Name]");
    }
    name
}

// ---------------------------------------------------------------------------
// 'buflisted', emptiness and b:changedtick

/// Set `'buflisted'` for the current buffer, firing `BufAdd`/`BufDelete` if
/// it changed.
pub fn set_buflisted(on: c_int) {
    let mut buf = Buf::current();
    if on == buf.b_p_bl {
        return;
    }
    buf.b_p_bl = on;
    let event = if on != 0 {
        AutoEvent::BufAdd
    } else {
        AutoEvent::BufDelete
    };
    let __hoisted_0 = Buf::current_or_none();

    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, __hoisted_0) };
}

pub fn buf_is_empty(buffer: Buf) -> bool {
    let b = buffer;
    // SAFETY: line 1 exists in every buffer, and `ml_get_buf` answers a
    // NUL-terminated line.
    b.b_ml.ml_line_count == 1 as LineNr && buffer.lines().line(1).is_empty()
}

pub fn buf_inc_changedtick(buffer: Buf) {
    buf_set_changedtick(buffer, buf_get_changedtick(buffer) + 1 as VarNumber);
}

/// Set `b:changedtick`, telling any `b:` watcher about the change.
pub fn buf_set_changedtick(mut b: Buf, changedtick: VarNumber) {
    // `b:changedtick` is always a plain number, so the watcher's "old
    // value" is one too: a fresh typval, owning nothing.
    let old_val = TypVal::Number(b.changedtick_di.di_tv.number_or_zero());
    check_changedtick_item(b);
    b.changedtick_di.di_tv.write_number(changedtick);
    // SAFETY: `b_vars` is the buffer's own dictionary, allocated with it.
    if dict_is_watched(unsafe { (b.b_vars).as_ref() }) {
        b.b_locked += 1;
        let vars = b.b_vars;
        let key = b.changedtick_di.di_key.as_ptr().cast_mut();
        let new = &raw mut b.changedtick_di.di_tv;
        // SAFETY: the buffer's own dictionary and its `changedtick` entry,
        // plus a local holding the value it had.
        unsafe {
            dict_watcher_notify(
                vars,
                ::core::ffi::CStr::from_ptr(key),
                Some(&*new),
                Some(&old_val),
            )
        };
        b.b_locked -= 1;
    }
}

/// The consistency checks upstream wraps in `#ifndef NDEBUG`: `b:` must
/// still hold the fixed, read-only number `buf_init_changedtick` put there.
fn check_changedtick_item(buffer: Buf) {
    if !cfg!(debug_assertions) {
        return;
    }
    let vars = buffer.b_vars;
    let key = c"changedtick";
    // SAFETY: the buffer's own dictionary.
    let item = dict_find(unsafe { vars.as_ref() }, key.to_bytes()).expect("changedtick_di != NULL");
    assert!(
        item.di_tv.v_type() == VAR_NUMBER as _,
        "changedtick_di->di_tv.v_type() == VAR_NUMBER"
    );
    assert!(
        item.di_lock == VarLock::Fixed,
        "changedtick_di->di_lock == VarLock::Fixed"
    );
    assert!(
        item.di_flags as c_int == DI_FLAGS_RO as c_int | DI_FLAGS_FIX as c_int,
        "changedtick_di->di_flags == (DI_FLAGS_RO|DI_FLAGS_FIX)"
    );
    assert!(
        ::core::ptr::from_ref(item) == (&raw const buffer.changedtick_di).cast::<DictItem>(),
        "changedtick_di == (DictItem *)&buf->changedtick_di"
    );
}
