//! What the emulator tells the editor.
//!
//! vterm reports through the hosts installed when the terminal is allocated:
//! [`ScreenReports`] for everything about the screen, and [`Clipboard`] for
//! clipboard writes. Each holds the [`Term`] the reports belong to.
//!
//! Almost all of these do one of two things: record something on the
//! `Terminal` and mark rows invalid, or hand work to the main loop. Nothing
//! here draws — the refresh does that, later, in
//! [`refresh`](super::refresh)-land. The exceptions are the two that reach
//! the editor directly: `b:term_title` is a buffer variable, and a clipboard
//! write has to run the clipboard provider, so it is queued rather than done
//! here.

#![forbid(unsafe_code)]
use crate::drawscreen::status_redraw_buf;
use crate::eval::eval_call_provider;

use crate::api::private::helpers::{dict_set_var, scope_vars};
use crate::eval::typval::tv_list_alloc;
use crate::event_queue::put_job_on_main_loop;
use crate::option::vars::p_bg;
use crate::options::kOptBoFlagTerm;
use crate::types::{
    Error, Object, String_0, VTermPos, VTermProp, VTermRect, VTermScreenCell, VTermSelectionMask,
    kListLenMayKnow, ptrdiff_t,
};
use crate::ui::vim_beep;
use crate::vterm::host::{Fragment, ScreenHost, SelectionHost, Value};
use crate::vterm::vterm::{
    VTERM_PROP_ALTSCREEN, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORVISIBLE,
    VTERM_PROP_MOUSE, VTERM_PROP_SYNCOUTPUT, VTERM_PROP_THEMEUPDATES, VTERM_PROP_TITLE,
    VTERM_SELECTION_PRIMARY,
};
use core::ffi::{c_char, c_int};

use crate::winlayer::Buf;

use super::refresh::invalidate_terminal;
use super::{Term, scrollback, term_buf};

/// What the screen reports, for the terminal it belongs to.
pub(crate) struct ScreenReports(pub(crate) Term);

impl ScreenHost for ScreenReports {
    fn takes_moves(&self) -> bool {
        true
    }

    fn takes_scrollback(&self) -> bool {
        true
    }

    fn damage(&mut self, rect: VTermRect) -> Option<c_int> {
        invalidate_terminal(self.0, Some((rect.start_row, rect.end_row)));
        Some(1)
    }

    fn moverect(&mut self, dest: VTermRect, src: VTermRect) -> Option<c_int> {
        let rows = (
            dest.start_row.min(src.start_row),
            dest.end_row.max(src.end_row),
        );
        invalidate_terminal(self.0, Some(rows));
        Some(1)
    }

    fn movecursor(
        &mut self,
        new_pos: VTermPos,
        _old_pos: VTermPos,
        _visible: c_int,
    ) -> Option<c_int> {
        let mut term = self.0;
        term.cursor.row = new_pos.row;
        term.cursor.col = new_pos.col;
        invalidate_terminal(term, None);
        Some(1)
    }

    fn settermprop(&mut self, prop: VTermProp, val: &Value<'_>) -> Option<c_int> {
        Some(apply_termprop(self.0, prop, val))
    }

    fn bell(&mut self) -> Option<c_int> {
        vim_beep(kOptBoFlagTerm as ::core::ffi::c_uint);
        Some(1)
    }

    /// Answer vterm's "is the background dark?" query from `'background'`.
    fn theme(&mut self, dark: &mut bool) -> Option<c_int> {
        p_bg(|value| *dark = value.to_bytes().first() == Some(&b'd'));
        Some(1)
    }

    fn sb_pushline(&mut self, cells: &[VTermScreenCell]) -> Option<c_int> {
        Some(scrollback::scrollback_push(self.0, cells))
    }

    fn sb_popline(&mut self, cells: &mut [VTermScreenCell]) -> Option<c_int> {
        Some(scrollback::scrollback_pop(self.0, cells))
    }

    fn sb_clear(&mut self) -> Option<c_int> {
        Some(scrollback::scrollback_clear(self.0))
    }
}

/// Publish `title` as the buffer's `b:term_title`.
///
/// Does nothing when the buffer is gone, which happens if the child sets a
/// title after its terminal buffer was wiped.
pub(crate) fn buf_set_term_title(buffer: Option<Buf>, title: &[u8]) {
    let Some(mut buf) = buffer else {
        return;
    };
    let mut err = Error::none();
    let title = Object::string(String_0::from_bytes(title));
    let (vars, key) = (
        scope_vars(&buf.b_bufvar),
        String_0::from_cstr(c"term_title"),
    );
    // Setting a variable can run `BufModified`-ish machinery; the lock
    // keeps that from touching the buffer's lines mid-update.
    buf.b_locked += 1;
    drop(dict_set_var(&vars, &key, title, false, false));
    buf.b_locked -= 1;
    err.clear();
    status_redraw_buf(buf);
}

/// Accumulate a fragmented title, publishing it once the last fragment
/// arrives.
fn accumulate_title(mut term: Term, frag: &Fragment<'_>) {
    let bytes = frag.bytes();
    let buf = term_buf(term);
    if frag.initial && frag.last {
        buf_set_term_title(buf, bytes);
        return;
    }
    if frag.initial {
        term.title.clear();
    }
    term.title.extend_from_slice(bytes);
    if frag.last {
        // Taken rather than borrowed: publishing reaches the editor, and
        // the accumulator is done with either way.
        let title = ::core::mem::take(&mut term.title);
        buf_set_term_title(buf, &title);
    }
}

fn apply_termprop(mut term: Term, prop: VTermProp, val: &Value<'_>) -> c_int {
    if prop == VTERM_PROP_TITLE {
        if let Value::Str(frag) = val {
            accumulate_title(term, frag);
        }
        return 1;
    }
    // Every property below is integer-typed.
    let number = val.int();
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

/// Hand a finished OSC 52 write to the clipboard provider: `payload` (up
/// to its first NUL, as the C string it was) into the register `mask`
/// names.
fn term_clipboard_set(mask: VTermSelectionMask, payload: &[u8]) {
    let regname = if mask == VTERM_SELECTION_PRIMARY {
        b'*' as c_char
    } else {
        b'+' as c_char
    };
    let payload = payload.split(|&byte| byte == 0).next().unwrap_or_default();
    // A readfile()-style list of lines (`:h chansend()`), the last one kept
    // even when empty so that a trailing newline survives: a newline inside
    // one item would reach a command-line provider as a NUL.
    let lines = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    for line in payload.split(|&byte| byte == b'\n') {
        lines.edit().push_bytes(Some(line));
    }
    let mut held = tv_list_alloc(3 as ptrdiff_t);
    let args = &mut *held;
    args.push_list(Some(lines));
    args.push_bytes(Some(b"v"));
    args.push_bytes(Some(&[regname.cast_unsigned()]));
    // The provider is Vimscript, which is why this runs on the main loop.
    eval_call_provider(c"clipboard", c"set", Some(held), true);
}

/// OSC 52 writes, for the terminal they arrive at.
pub(crate) struct Clipboard(pub(crate) Term);

impl SelectionHost for Clipboard {
    fn takes_set(&self) -> bool {
        true
    }

    /// Accumulate a clipboard write, queueing it once complete.
    fn set(&mut self, mask: VTermSelectionMask, frag: Fragment<'_>) -> Option<c_int> {
        let mut term = self.0;
        if frag.initial {
            term.selection.clear();
        }
        term.selection.extend_from_slice(frag.bytes());
        if frag.last {
            let payload = term.selection.clone();
            put_job_on_main_loop(move || term_clipboard_set(mask, &payload));
        }
        Some(1)
    }

    fn query(&mut self, _mask: VTermSelectionMask) -> Option<c_int> {
        None
    }
}
