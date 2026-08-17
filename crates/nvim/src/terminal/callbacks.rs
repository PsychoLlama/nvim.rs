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

use crate::api::private::helpers::{api_clear_error, dict_set_var};
use crate::channel::main_loop_events;
use crate::drawscreen::status_redraw_buf;
use crate::eval::eval_call_provider;
use crate::eval::typval::{
    tv_list_alloc, tv_list_append_allocated_string, tv_list_append_list, tv_list_append_string,
};
use crate::event::multiqueue::multiqueue_put_event;
use crate::main::p_bg;
use crate::memory::xmemdupz;
use crate::options::kOptBoFlagTerm;
use crate::types::builders::static_cstring;
use crate::types::{
    Error, Event, Object, String_0, VTermPos, VTermProp, VTermRect, VTermScreenCallbacks,
    VTermSelectionCallbacks, VTermSelectionMask, VTermStringFragment, VTermValue, kErrorTypeNone,
    list_T, ptrdiff_t, ssize_t,
};
use crate::ui::vim_beep;
use crate::vterm::vterm::{
    VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORVISIBLE,
    VTERM_PROP_MOUSE, VTERM_PROP_SYNCOUTPUT, VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE,
    VTERM_SELECTION_PRIMARY,
};
use core::ffi::{c_char, c_int, c_void};

use crate::winlayer::Buf;

use super::refresh::invalidate_terminal;
use super::{Term, scrollback};

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
    // SAFETY: vterm hands back the terminal registered alongside this table.
    let term = unsafe { Term::new(data.cast()) };
    invalidate_terminal(term, Some((rect.start_row, rect.end_row)));
    1
}

unsafe extern "C" fn term_moverect(dest: VTermRect, src: VTermRect, data: *mut c_void) -> c_int {
    // SAFETY: as above.
    let term = unsafe { Term::new(data.cast()) };
    let rows = (
        dest.start_row.min(src.start_row),
        dest.end_row.max(src.end_row),
    );
    invalidate_terminal(term, Some(rows));
    1
}

unsafe extern "C" fn term_movecursor(
    new_pos: VTermPos,
    _old_pos: VTermPos,
    _visible: c_int,
    data: *mut c_void,
) -> c_int {
    // SAFETY: as above.
    let mut term = unsafe { Term::new(data.cast()) };
    term.cursor.row = new_pos.row;
    term.cursor.col = new_pos.col;
    invalidate_terminal(term, None);
    1
}

/// Publish `title` as the buffer's `b:term_title`.
///
/// Does nothing when the buffer is gone, which happens if the child sets a
/// title after its terminal buffer was wiped.
pub fn buf_set_term_title(buf: Option<Buf>, title: &[u8]) {
    let Some(mut buf) = buf else {
        return;
    };
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut(),
    };
    let title = Object::string(String_0 {
        data: title.as_ptr().cast::<c_char>().cast_mut(),
        size: title.len(),
    });
    let (vars, key, arena) = (
        buf.b_vars,
        static_cstring(c"term_title"),
        ::core::ptr::null_mut(),
    );
    // Setting a variable can run `BufModified`-ish machinery; the lock
    // keeps that from touching the buffer's lines mid-update.
    buf.b_locked += 1;
    // SAFETY: the buffer's own variable dictionary, and a string that
    // outlives the call, which copies it.
    unsafe { dict_set_var(vars, key, title, false, false, arena, &mut err) };
    buf.b_locked -= 1;
    // SAFETY: an error this function owns, cleared exactly once.
    unsafe { api_clear_error(&mut err) };
    // SAFETY: a live buffer, whose status line names the title.
    unsafe { status_redraw_buf(buf.raw()) };
}

/// Accumulate a fragmented title, publishing it once the last fragment
/// arrives.
///
/// # Safety
/// `frag` must be a fragment vterm handed over, as [`fragment_bytes`] wants.
unsafe fn term_set_title(mut term: Term, frag: &VTermStringFragment) {
    // SAFETY: the caller's promise.
    let bytes = unsafe { fragment_bytes(frag) };
    let buf = term.buf();
    if frag.initial() && frag.final_0() {
        buf_set_term_title(buf, bytes);
        return;
    }
    if frag.initial() {
        term.title.clear();
    }
    term.title.extend_from_slice(bytes);
    if frag.final_0() {
        // Taken rather than borrowed: publishing reaches the editor, and
        // the accumulator is done with either way.
        let title = ::core::mem::take(&mut term.title);
        buf_set_term_title(buf, &title);
    }
}

unsafe extern "C" fn term_settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    data: *mut c_void,
) -> c_int {
    // SAFETY: vterm hands back the terminal registered alongside this table.
    let mut term = unsafe { Term::new(data.cast()) };
    if prop == VTERM_PROP_TITLE {
        // SAFETY: the value carries the arm the property's type names, and
        // the fragment is vterm's own.
        unsafe { term_set_title(term, &(*val).string) };
        return 1;
    }
    // SAFETY: as above; every property below is integer-typed, and
    // `boolean` and `number` are the same `c_int` at offset 0.
    let number = unsafe { (*val).number };
    let flag = number != 0;
    match prop {
        VTERM_PROP_ALTSCREEN => term.in_altscreen = flag,
        VTERM_PROP_CURSORVISIBLE => {
            term.cursor.visible = flag;
            invalidate_terminal(term, None);
        }
        VTERM_PROP_MOUSE => term.forward_mouse = flag,
        VTERM_PROP_CURSORBLINK => {
            term.cursor.blink = flag;
            term.pending.cursor = true;
            invalidate_terminal(term, None);
        }
        VTERM_PROP_CURSORSHAPE => {
            term.cursor.shape = number;
            term.pending.cursor = true;
            invalidate_terminal(term, None);
        }
        VTERM_PROP_THEMEUPDATES => term.theme_updates = flag,
        VTERM_PROP_SYNCOUTPUT => {
            // While synchronized output is on, damage is recorded but not
            // refreshed; leaving it owes the screen one flush.
            term.synchronized_output = flag;
            if !flag {
                term.sync_flush_pending = true;
            }
        }
        _ => return 0,
    }
    1
}

unsafe extern "C" fn term_bell(_data: *mut c_void) -> c_int {
    // SAFETY: the editor's own beep, which takes no pointer.
    unsafe { vim_beep(kOptBoFlagTerm as ::core::ffi::c_uint) };
    1
}

/// Answer vterm's "is the background dark?" query from `'background'`.
unsafe extern "C" fn term_theme(dark: *mut bool, _data: *mut c_void) -> c_int {
    // SAFETY: vterm's own out-parameter, and `'background'` is a live
    // option string.
    unsafe { *dark = *p_bg.get() == b'd' as c_char };
    1
}

unsafe extern "C" fn term_clipboard_set(argv: *mut *mut c_void) {
    // SAFETY: the event's own two arguments, as `term_selection_set` left
    // them: a selection mask, and the string it allocated.
    let (mask, data) = unsafe { ((*argv).expose_provenance(), *argv.add(1) as *mut c_char) };
    let mut regname = if mask as VTermSelectionMask == VTERM_SELECTION_PRIMARY {
        b'*' as c_char
    } else {
        b'+' as c_char
    };
    // SAFETY: a fresh list, which takes ownership of `data`.
    let lines: *mut list_T = unsafe { tv_list_alloc(1 as ptrdiff_t) };
    // SAFETY: as above.
    unsafe { tv_list_append_allocated_string(lines, data) };
    // SAFETY: a fresh list, which takes ownership of `lines`.
    let args: *mut list_T = unsafe { tv_list_alloc(3 as ptrdiff_t) };
    // SAFETY: as above.
    unsafe { tv_list_append_list(args, lines) };
    let regtype = b'v' as c_char;
    // SAFETY: as above, over one byte of this frame each, which the list
    // copies.
    unsafe { tv_list_append_string(args, &raw const regtype, 1 as ssize_t) };
    // SAFETY: as above.
    unsafe { tv_list_append_string(args, &raw mut regname, 1 as ssize_t) };
    let (provider, method) = (c"clipboard".as_ptr().cast_mut(), c"set".as_ptr().cast_mut());
    // SAFETY: two names of this crate's own, and the arguments built above.
    // The provider is Vimscript, which is why this runs on the main loop.
    unsafe { eval_call_provider(provider, method, args, true) };
}

/// Accumulate an OSC 52 clipboard write, queueing it once complete.
unsafe extern "C" fn term_selection_set(
    mask: VTermSelectionMask,
    frag: VTermStringFragment,
    user: *mut c_void,
) -> c_int {
    // SAFETY: vterm hands back the terminal registered alongside this table.
    let mut term = unsafe { Term::new(user.cast()) };
    if frag.initial() {
        term.selection.clear();
    }
    // SAFETY: a fragment vterm handed over.
    term.selection
        .extend_from_slice(unsafe { fragment_bytes(&frag) });
    if frag.final_0() {
        let (bytes, len) = (
            term.selection.as_ptr().cast::<c_void>(),
            term.selection.len(),
        );
        // The event handler hands this to a list, which frees it.
        //
        // SAFETY: a slice of the terminal's own accumulator, copied.
        let data = unsafe { xmemdupz(bytes, len) };
        let mask = ::core::ptr::with_exposed_provenance_mut::<c_void>(mask as usize);
        let event = Event::new(Some(term_clipboard_set), [mask, data]);
        // SAFETY: the main loop's queue, live from startup to exit.
        unsafe { multiqueue_put_event(main_loop_events(), event) };
    }
    1
}
