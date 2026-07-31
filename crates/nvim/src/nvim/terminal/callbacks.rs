//! What the emulator tells the editor.
//!
//! vterm reports through callback tables installed when the terminal is
//! allocated: [`SCREEN_CALLBACKS`] for everything about the screen, and
//! [`SELECTION_CALLBACKS`] for clipboard writes. Each entry's `data` is the
//! [`Terminal`] the report belongs to.
//!
//! Almost all of these do one of two things: record something on the
//! `Terminal` and mark rows invalid, or hand work to the main loop. Nothing
//! here draws — the refresh does that, later, in
//! [`refresh`](super::refresh)-land. The exceptions are the two that reach
//! the editor directly: `b:term_title` is a buffer variable, and a clipboard
//! write has to run the clipboard provider, so it is queued rather than done
//! here.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{api_clear_error, dict_set_var};
use crate::src::nvim::drawscreen::status_redraw_buf;
use crate::src::nvim::eval::eval_call_provider;
use crate::src::nvim::eval::typval::{
    tv_list_alloc, tv_list_append_allocated_string, tv_list_append_list, tv_list_append_string,
};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::main::{main_loop, p_bg};
use crate::src::nvim::memory::xmemdupz;
use crate::src::nvim::options::kOptBoFlagTerm;
use crate::src::nvim::types::builders::static_cstring;
use crate::src::nvim::types::{
    Arena, Error, Event, Object, String_0, Terminal, VTermPos, VTermProp, VTermRect,
    VTermScreenCallbacks, VTermSelectionCallbacks, VTermSelectionMask, VTermStringFragment,
    VTermValue, buf_T, list_T, ptrdiff_t, ssize_t,
};
use crate::src::nvim::ui::vim_beep;
use crate::src::nvim::vterm::vterm::{
    VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORVISIBLE,
    VTERM_PROP_MOUSE, VTERM_PROP_SYNCOUTPUT, VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE,
    VTERM_SELECTION_PRIMARY,
};
use core::ffi::{c_char, c_int, c_void};

use super::refresh::invalidate_terminal;
use super::{buf_for_handle, scrollback};
use crate::src::nvim::types::api::kErrorTypeNone;

pub static SCREEN_CALLBACKS: VTermScreenCallbacks = VTermScreenCallbacks {
    damage: Some(term_damage),
    moverect: Some(term_moverect),
    movecursor: Some(term_movecursor),
    settermprop: Some(term_settermprop),
    bell: Some(term_bell),
    resize: None,
    theme: Some(term_theme),
    sb_pushline: Some(scrollback::term_sb_push),
    sb_popline: Some(scrollback::term_sb_pop),
    sb_clear: Some(scrollback::term_sb_clear),
};

pub static SELECTION_CALLBACKS: VTermSelectionCallbacks = VTermSelectionCallbacks {
    set: Some(term_selection_set),
    query: None,
};

/// The bytes of a fragment vterm handed over.
///
/// # Safety
/// `frag.str` must point at `frag.len()` readable bytes, which is vterm's
/// contract for a fragment callback.
unsafe fn fragment_bytes(frag: &VTermStringFragment) -> &[u8] {
    if frag.str.is_null() || frag.len() == 0 {
        return &[];
    }
    unsafe { ::core::slice::from_raw_parts(frag.str.cast::<u8>(), frag.len()) }
}

unsafe extern "C" fn term_damage(rect: VTermRect, data: *mut c_void) -> c_int {
    unsafe { invalidate_terminal(data as *mut Terminal, Some((rect.start_row, rect.end_row))) };
    1
}

unsafe extern "C" fn term_moverect(dest: VTermRect, src: VTermRect, data: *mut c_void) -> c_int {
    unsafe {
        invalidate_terminal(
            data as *mut Terminal,
            Some((
                dest.start_row.min(src.start_row),
                dest.end_row.max(src.end_row),
            )),
        )
    };
    1
}

unsafe extern "C" fn term_movecursor(
    new_pos: VTermPos,
    _old_pos: VTermPos,
    _visible: c_int,
    data: *mut c_void,
) -> c_int {
    unsafe {
        let term = data as *mut Terminal;
        (*term).cursor.row = new_pos.row;
        (*term).cursor.col = new_pos.col;
        invalidate_terminal(term, None);
    }
    1
}

/// Publish `title` as the buffer's `b:term_title`.
///
/// Does nothing when the buffer is gone, which happens if the child sets a
/// title after its terminal buffer was wiped.
pub unsafe fn buf_set_term_title(buf: *mut buf_T, title: &[u8]) {
    unsafe {
        if buf.is_null() {
            return;
        }
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut(),
        };
        // Setting a variable can run `BufModified`-ish machinery; the lock
        // keeps that from touching the buffer's lines mid-update.
        (*buf).b_locked += 1;
        dict_set_var(
            (*buf).b_vars,
            static_cstring(c"term_title"),
            Object::string(String_0 {
                data: title.as_ptr().cast::<c_char>().cast_mut(),
                size: title.len(),
            }),
            false,
            false,
            ::core::ptr::null_mut::<Arena>(),
            &mut err,
        );
        (*buf).b_locked -= 1;
        api_clear_error(&mut err);
        status_redraw_buf(buf);
    }
}

/// Accumulate a fragmented title, publishing it once the last fragment
/// arrives.
unsafe fn term_set_title(term: *mut Terminal, frag: &VTermStringFragment) {
    unsafe {
        let buf = buf_for_handle((*term).buf_handle);
        if frag.initial() && frag.final_0() {
            buf_set_term_title(buf, fragment_bytes(frag));
            return;
        }
        if frag.initial() {
            (*term).title.clear();
        }
        (*term).title.extend_from_slice(fragment_bytes(frag));
        if frag.final_0() {
            // Taken rather than borrowed: publishing reaches the editor,
            // and the accumulator is done with either way.
            let title = ::core::mem::take(&mut (*term).title);
            buf_set_term_title(buf, &title);
        }
    }
}

unsafe extern "C" fn term_settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    data: *mut c_void,
) -> c_int {
    unsafe {
        let term = data as *mut Terminal;
        match prop {
            VTERM_PROP_ALTSCREEN => (*term).in_altscreen = (*val).boolean != 0,
            VTERM_PROP_CURSORVISIBLE => {
                (*term).cursor.visible = (*val).boolean != 0;
                invalidate_terminal(term, None);
            }
            VTERM_PROP_TITLE => term_set_title(term, &(*val).string),
            VTERM_PROP_MOUSE => (*term).forward_mouse = (*val).number != 0,
            VTERM_PROP_CURSORBLINK => {
                (*term).cursor.blink = (*val).boolean != 0;
                (*term).pending.cursor = true;
                invalidate_terminal(term, None);
            }
            VTERM_PROP_CURSORSHAPE => {
                (*term).cursor.shape = (*val).number;
                (*term).pending.cursor = true;
                invalidate_terminal(term, None);
            }
            VTERM_PROP_THEMEUPDATES => (*term).theme_updates = (*val).boolean != 0,
            VTERM_PROP_SYNCOUTPUT => {
                // While synchronized output is on, damage is recorded but
                // not refreshed; leaving it owes the screen one flush.
                (*term).synchronized_output = (*val).boolean != 0;
                if (*val).boolean == 0 {
                    (*term).sync_flush_pending = true;
                }
            }
            _ => return 0,
        }
        1
    }
}

unsafe extern "C" fn term_bell(_data: *mut c_void) -> c_int {
    unsafe { vim_beep(kOptBoFlagTerm as ::core::ffi::c_uint) };
    1
}

/// Answer vterm's "is the background dark?" query from `'background'`.
unsafe extern "C" fn term_theme(dark: *mut bool, _data: *mut c_void) -> c_int {
    unsafe { *dark = *p_bg.get() == b'd' as c_char };
    1
}

/// Hand `data` to the clipboard provider. Runs on the main loop, because
/// the provider is Vimscript.
unsafe extern "C" fn term_clipboard_set(argv: *mut *mut c_void) {
    unsafe {
        let mask = (*argv.offset(0)).expose_provenance() as VTermSelectionMask;
        // Allocated by `term_selection_set`; the list takes ownership.
        let data = *argv.offset(1) as *mut c_char;
        let mut regname = if mask == VTERM_SELECTION_PRIMARY {
            b'*' as c_char
        } else {
            b'+' as c_char
        };

        let lines: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
        tv_list_append_allocated_string(lines, data);
        let args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_list(args, lines);
        let regtype = b'v' as c_char;
        tv_list_append_string(args, &raw const regtype, 1 as ssize_t);
        tv_list_append_string(args, &raw mut regname, 1 as ssize_t);
        eval_call_provider(
            c"clipboard".as_ptr().cast_mut(),
            c"set".as_ptr().cast_mut(),
            args,
            true,
        );
    }
}

/// Accumulate an OSC 52 clipboard write, queueing it once complete.
unsafe extern "C" fn term_selection_set(
    mask: VTermSelectionMask,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    unsafe {
        let term = user as *mut Terminal;
        if frag.initial() {
            (*term).selection.clear();
        }
        (*term).selection.extend_from_slice(fragment_bytes(&frag));
        if frag.final_0() {
            // The event handler hands this to a list, which frees it.
            let data = xmemdupz(
                (*term).selection.as_ptr().cast::<c_void>(),
                (*term).selection.len(),
            );
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event::new(
                    Some(term_clipboard_set),
                    [
                        ::core::ptr::with_exposed_provenance_mut::<c_void>(mask as usize),
                        data,
                    ],
                ),
            );
        }
        1
    }
}
