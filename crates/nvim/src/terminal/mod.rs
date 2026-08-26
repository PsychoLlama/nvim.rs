//! `:terminal` buffers.
//!
//! Each one pairs a libvterm emulator with a buffer whose lines mirror what
//! that emulator holds. Bytes the child program writes go into vterm
//! ([`terminal_receive`]); vterm reports what changed
//! ([`callbacks`](self::callbacks)); a timer mirrors the changes into the
//! buffer's lines ([`refresh`](self::refresh)). Keys go the other way, from
//! the editor's terminal mode ([`mode`](self::mode)) through the key
//! translation in [`input`](self::input) and out of vterm's output
//! callback.
//!
//! What lives where: the scrollback and its buffer mirroring in
//! [`scrollback`](self::scrollback), unrecognised escape sequences in
//! [`termrequest`](self::termrequest). This module keeps the lifecycle —
//! allocating, opening, closing and destroying a terminal — and the one
//! thing the editor asks a terminal for while drawing:
//! [`terminal_get_line_attributes`], which turns a row of vterm cells into
//! the highlight ids the screen painter wants.
//!
//! A `Terminal` and its buffer can outlive each other in both directions.
//! The buffer can be wiped while the child is still running, and the child
//! can exit while the buffer is still on screen, so nothing here holds a
//! `buf_T` across anything that might run autocommands — `buf_handle` plus
//! [`buf_for_handle`] is the pattern throughout. `refcount` is the other
//! half: it is raised around anything that can run Vimscript, and
//! [`terminal_destroy`] only frees at zero. [`Term`] is how all of that
//! survives being written in safe code: it wraps the `*mut Terminal` the
//! editor passes around, makes the *construction* the unsafe step, and hands
//! out one short-lived borrow per use.

#![deny(unsafe_op_in_unsafe_fn)]

pub(crate) mod callbacks;
pub(crate) mod input;
pub(crate) mod mode;
pub(crate) mod refresh;
pub(crate) mod scrollback;
pub(crate) mod termrequest;

use crate::api::private::helpers::{
    api_clear_error, api_free_object, cstr_as_string, dict_get_value,
};
use crate::autocmd::{
    EVENT_TERMCLOSE, EVENT_TERMOPEN, apply_autocmds, apply_autocmds_group, aucmd_prepbuf,
    aucmd_restbuf, block_autocmds, is_aucmd_win, is_autocmd_blocked, unblock_autocmds,
};
use crate::change::deleted_lines_buf;
use crate::channel::main_loop_events;
use crate::cursor_shape::{SHAPE_IDX_TERM, shape_entry};
use crate::drawscreen::redraw_buf_line_later;
use crate::eval::typval::{tv_dict_add_nr, tv_dict_set_keys_readonly};
use crate::eval::vars::get_globvar_dict;
use crate::eval::{get_v_event, restore_v_event};
use crate::event::multiqueue::{multiqueue_free, multiqueue_new, multiqueue_put_event};
use crate::highlight::{HlAttrFlags, hl_combine_attr, hl_get_term_attr};
use crate::highlight_group::name_to_color;
use crate::main::{State, exiting};
use crate::memline::MlFlags;
use crate::memline::ml_delete_buf;
use crate::r#move::win_col_off;
use crate::option::set_option_value;
use crate::options::kOptBuftype;
use crate::types::builders::{DictBuf, static_cstring};
use crate::types::terminal_defs::SELECTIONBUF_SIZE;
use crate::types::{
    Arena, Buffer, Error, Event, ExtmarkOp, HlAttrs, MarkAdjustMode, Object, OptVal, OptValData,
    OptValType, OptionSetFlags, RgbValue, Terminal, TerminalOptions, VTermColor, VTermColor_rgb,
    VTermScreenCell, VTermScreenCellAttrs, VTermState, VTermValue, aco_save_T, buf_T, colnr_T,
    dict_T, exarg_T, handle_T, int16_t, kErrorTypeNone, kObjectTypeNil, kObjectTypeString,
    linenr_T, pos_T, save_v_event_T, size_t, uint8_t, varnumber_T, win_T,
};
use crate::vterm::parser::vterm_input_write;
use crate::vterm::pen::{convert_color_to_rgb, set_palette_color};
use crate::vterm::screen::{
    vterm_obtain_screen, vterm_screen_enable_altscreen, vterm_screen_enable_reflow,
    vterm_screen_flush_damage, vterm_screen_reset, vterm_screen_set_callbacks,
    vterm_screen_set_damage_merge, vterm_screen_set_unrecognised_fallbacks,
};
use crate::vterm::state::entry::{
    vterm_obtain_state, vterm_state_set_selection_callbacks, vterm_state_set_termprop,
};
use crate::vterm::vterm::{
    VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG, VTERM_COLOR_INDEXED, VTERM_COLOR_RGB,
    VTERM_COLOR_TYPE_MASK, VTERM_DAMAGE_SCROLL, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORSHAPE_BAR_LEFT, VTERM_PROP_CURSORSHAPE_BLOCK,
    VTERM_PROP_CURSORSHAPE_UNDERLINE, vterm_free, vterm_get_size, vterm_new,
    vterm_output_set_callback, vterm_set_size, vterm_set_utf8,
};
use crate::winlayer::{self, Buf, Win, tab_windows, windows};
use ::libc::{abort, strlen};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ops::{Deref, DerefMut};

use scrollback::{fetch_cell, refresh_scrollback, term_may_alloc_scrollback};

use crate::state::MODE_TERMINAL;
pub(crate) use input::{terminal_paste, terminal_set_streamed_paste};
pub(crate) use mode::terminal_enter;
pub(crate) use refresh::{
    on_scrollback_option_changed, terminal_check_refresh, terminal_init, terminal_teardown,
};

/// The largest `'scrollback'` that means anything; the option's negative
/// "unlimited" spelling becomes this.
const SB_MAX: c_int = 1000000;
/// Marks in trimmed scrollback move the way a terminal's do, not an edit's.
const kMarkAdjustTerm: MarkAdjustMode = 2;
const kExtmarkUndo: ExtmarkOp = 1;
/// `set_option_value` spellings.
const kOptValTypeString: OptValType = 2;
/// The most columns [`terminal_get_line_attributes`] will resolve;
/// `drawline` sizes its array to match.
const TERM_ATTRS_MAX: c_int = 1024;
/// Every autocommand group, for the events dispatched from here.
const AUGROUP_ALL: c_int = -3;

// ---------------------------------------------------------------------------
// The pointers, wrapped

/// A terminal the caller has promised is live.
///
/// The editor passes `*mut Terminal` around and the pointer has to stay raw:
/// vterm re-enters [`callbacks`](self::callbacks) with the same terminal
/// while [`terminal_receive`] is still inside `vterm_input_write`, and every
/// autocommand fired from here can run Vimscript that reaches it again. What
/// does not have to stay raw is the *dereference*. [`Term::new`] takes the
/// promise once — each `unsafe fn` below restates it in its own `# Safety`
/// section — and from there [`Deref`]/[`DerefMut`] give ordinary field
/// access, on [`winlayer`](crate::winlayer)'s pattern.
///
/// Being `Copy` is what keeps that honest: every borrow a `Term` produces
/// lives for one expression, so **no reference into a terminal is held
/// across a callback, an autocommand, a vterm write or a refresh**. Read the
/// slot out first and call afterwards.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Term(*mut Terminal);

/// The state machine of a live terminal's emulator.
///
/// [`Term::state`] is the only way to one, and it re-asks vterm every time
/// rather than caching, exactly as the C did. Two methods rather than a
/// `Deref`: the palette is all this module wants from a state.
#[derive(Clone, Copy)]
pub(crate) struct TermState(*mut VTermState);

impl Deref for Term {
    type Target = Terminal;

    #[inline(always)]
    fn deref(&self) -> &Terminal {
        // SAFETY: `Term::new`'s promise — the terminal is live.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Term {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Terminal {
        // SAFETY: as above.
        unsafe { &mut *self.0 }
    }
}

impl TermState {
    /// Point palette entry `index` at `color`.
    fn set_palette(self, index: c_int, color: &VTermColor) {
        // SAFETY: `Term::state`'s promise — the state belongs to a live
        // emulator and is freed with it.
        set_palette_color(unsafe { &mut *self.0 }, index, color);
    }

    /// `color` as a packed 24-bit RGB value, resolving the palette.
    fn rgb(self, mut color: VTermColor) -> c_int {
        // SAFETY: as above.
        convert_color_to_rgb(unsafe { &*self.0 }, &mut color);
        // SAFETY: `convert_color_to_rgb` leaves the value in its `rgb` arm.
        let rgb = unsafe { color.rgb };
        ((rgb.red as c_int) << 16) | ((rgb.green as c_int) << 8) | rgb.blue as c_int
    }
}

impl Term {
    /// # Safety
    /// `term` must stay a live terminal for as long as the value is used.
    #[inline(always)]
    pub(crate) const unsafe fn new(term: *mut Terminal) -> Self {
        Self(term)
    }

    #[inline(always)]
    pub(crate) fn raw(self) -> *mut Terminal {
        self.0
    }

    /// The buffer this terminal draws into, `None` once it has been wiped.
    fn buf(self) -> Option<Buf> {
        buf_for_handle(self.buf_handle)
    }

    /// The emulator's state machine.
    fn state(self) -> TermState {
        // SAFETY: the terminal's own emulator, which keeps the state it
        // builds on first ask until `vterm_free`.
        TermState(unsafe { vterm_obtain_state(self.vt) })
    }

    /// The emulator's size, rows first.
    fn size(self) -> (c_int, c_int) {
        let (mut rows, mut cols) = (0, 0);
        // SAFETY: the terminal's own emulator, and two live out-parameters.
        unsafe { vterm_get_size(self.vt, &raw mut rows, &raw mut cols) };
        (rows, cols)
    }

    /// Feed bytes of the child's output to the emulator. Re-entrant: vterm
    /// reports what changed through the screen callbacks, which reach this
    /// terminal again, so nothing of it is borrowed here.
    fn write(self, bytes: &[c_char]) {
        let vt = self.vt;
        // SAFETY: the terminal's own emulator, and a slice of its caller's
        // readable bytes.
        unsafe { vterm_input_write(vt, bytes.as_ptr(), bytes.len()) };
    }

    /// Deliver whatever the emulator has been holding back. Re-entrant for
    /// the same reason as [`Term::write`].
    fn flush_damage(self) {
        let vts = self.vts;
        // SAFETY: the terminal's own screen.
        unsafe { vterm_screen_flush_damage(vts) };
    }
}

/// The buffer a handle names, `None` once it has been wiped.
fn buf_for_handle(handle: handle_T) -> Option<Buf> {
    winlayer::buffer(handle)
}

/// vterm's "here are bytes for the child" callback.
unsafe extern "C" fn term_output_callback(s: *const c_char, len: size_t, user_data: *mut c_void) {
    // SAFETY: vterm hands back the terminal registered alongside this
    // callback, and `s` points at `len` readable bytes.
    let bytes = unsafe { ::core::slice::from_raw_parts(s.cast::<u8>(), len) };
    // SAFETY: as above.
    terminal_send(unsafe { Term::new(user_data as *mut Terminal) }, bytes);
}

/// Create a terminal for `buf` and wire it to vterm.
///
/// The buffer is emptied: its lines are about to become a mirror of the
/// emulator's screen, and anything already there would be taken for
/// scrollback.
pub(crate) unsafe fn terminal_alloc(buf: *mut buf_T, opts: TerminalOptions) -> *mut Terminal {
    // SAFETY: the caller hands over a live buffer that has no terminal yet.
    let mut buf = unsafe { Buf::new(buf) };
    // Leaked here and reclaimed by terminal_destroy. The buffer is the
    // owner; every other reference reaches it through `buf_T::terminal`.
    let raw: *mut Terminal = Box::into_raw(Box::new(Terminal::new(opts, buf.handle)));
    buf.terminal = raw;
    // SAFETY: just allocated, and nothing else has reached it yet.
    let mut term = unsafe { Term::new(raw) };
    let user = raw.cast::<c_void>();

    // SAFETY: a fresh emulator of the requested size, and every call below
    // is against the emulator, screen and state it owns. None of the
    // consumers being installed can run before this returns.
    unsafe {
        term.vt = vterm_new(opts.height as c_int, opts.width as c_int);
        vterm_set_utf8(term.vt, 1);
        term.vts = vterm_obtain_screen(term.vt);
        vterm_screen_enable_altscreen(term.vts, 1);
        vterm_screen_enable_reflow(term.vts, true);
        vterm_screen_set_callbacks(term.vts, &raw const callbacks::SCREEN_CALLBACKS, user);
        vterm_screen_set_unrecognised_fallbacks(term.vts, &raw const termrequest::FALLBACKS, user);
        // Merge damage reports up to a whole scrolled region before they are
        // delivered; the refresh works in row ranges anyway.
        vterm_screen_set_damage_merge(term.vts, VTERM_DAMAGE_SCROLL);
        vterm_screen_reset(term.vts, 1);
        vterm_output_set_callback(term.vt, Some(term_output_callback), user);
    }

    let state = term.state();
    let selection_cbs = &raw const callbacks::SELECTION_CALLBACKS;
    let buffer = term.selection_buffer.as_mut_ptr();
    let size = SELECTIONBUF_SIZE as size_t;
    // SAFETY: the state is the emulator's own; the scratch buffer is the
    // terminal's and is never resized while vterm holds it.
    unsafe { vterm_state_set_selection_callbacks(state.0, selection_cbs, user, buffer, size) };

    // Start the child off with the cursor the user configured for terminal
    // mode; it is free to change it.
    let shape = shape_entry(SHAPE_IDX_TERM);
    let mut cursor_shape = VTermValue { boolean: 0 };
    // vterm's shapes are DECSCUSR's rather than the editor's.
    cursor_shape.number = match shape.shape {
        0 => VTERM_PROP_CURSORSHAPE_BLOCK,
        1 => VTERM_PROP_CURSORSHAPE_UNDERLINE,
        2 => VTERM_PROP_CURSORSHAPE_BAR_LEFT,
        // Not a shape vterm knows; leave its default alone.
        _ => 0,
    };
    let mut cursor_blink = VTermValue { boolean: 0 };
    cursor_blink.boolean = (shape.blinkon != 0 && shape.blinkoff != 0) as c_int;
    // SAFETY: the state is the emulator's own, and each property is read
    // through the arm of `VTermValue` its type names.
    unsafe { vterm_state_set_termprop(state.0, VTERM_PROP_CURSORSHAPE, &raw mut cursor_shape) };
    // SAFETY: as above.
    unsafe { vterm_state_set_termprop(state.0, VTERM_PROP_CURSORBLINK, &raw mut cursor_blink) };

    term.invalid_start = 0;
    term.invalid_end = opts.height as c_int;
    // SAFETY: a queue with no "on put" hook, freed by `terminal_destroy`.
    term.pending.events = unsafe { multiqueue_new(None, ::core::ptr::null_mut()) };

    if !buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
        let line_count = buf.line_count();
        // Not immutable: ml_delete_buf() mutates b_ml behind the pointer.
        #[allow(clippy::while_immutable_condition)]
        while !buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
            // SAFETY: a live buffer, deleting its own lines down to the one
            // empty line `MlFlags::EMPTY` stands for.
            unsafe { ml_delete_buf(buf.raw(), 1 as linenr_T, false) };
        }
        // SAFETY: as above, reporting what the deletion took away.
        unsafe { deleted_lines_buf(buf.raw(), 1 as linenr_T, line_count) };
    }
    term.old_height = 1;
    raw
}

/// Make `buf` look and behave like a terminal buffer, and announce it.
///
/// Runs `TermOpen`, which can wipe the buffer or close the terminal
/// outright — hence the re-check before touching either again.
pub(crate) unsafe fn terminal_open(termpp: *mut *mut Terminal, buf: *mut buf_T) {
    // SAFETY: the caller hands over the buffer's own terminal slot.
    let mut term = unsafe { Term::new(*termpp) };
    assert!(!term.raw().is_null(), "terminal_open without a terminal");
    // SAFETY: the caller hands over a live buffer.
    let mut buf = unsafe { Buf::new(buf) };

    // SAFETY: a plain save area `aucmd_prepbuf` fills in, restored below.
    let mut aco: aco_save_T = unsafe { ::core::mem::zeroed() };
    // SAFETY: paired with the `aucmd_restbuf` below.
    unsafe { aucmd_prepbuf(&raw mut aco, buf.raw()) };
    if term.sb.is_sized() {
        refresh_scrollback(term, buf);
    } else {
        debug_assert!(term.invalid_start >= 0);
    }
    refresh::refresh_screen(term, buf);

    // Locked because setting 'buftype' can run OptionSet, and the buffer's
    // lines are the emulator's to write.
    buf.b_locked += 1;
    set_option_value(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: static_cstring(c"terminal"),
            },
        },
        OptionSetFlags::LOCAL,
    );
    buf.b_locked -= 1;

    let ffname = buf.b_ffname;
    if !ffname.is_null() {
        // SAFETY: a non-null `b_ffname` is a NUL-terminated file name, read
        // before anything here can free it.
        let title = unsafe { ::core::slice::from_raw_parts(ffname.cast(), strlen(ffname)) };
        callbacks::buf_set_term_title(Some(buf), title);
    }
    // Both would tie the terminal window's scroll position to another
    // window's, which fights the emulator for the topline.
    // SAFETY: `curwin` is set from startup to exit.
    let mut win = unsafe { Win::current() };
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
    win.w_cursor = pos_T {
        lnum: 1 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let none = ::core::ptr::null_mut();
    // SAFETY: TermOpen against a live buffer. It may wipe the buffer or
    // close the terminal, which is what the re-check below is for, and
    // nothing of either is borrowed across it.
    unsafe { apply_autocmds(EVENT_TERMOPEN, none, none, false, buf.raw()) };
    // SAFETY: paired with the `aucmd_prepbuf` above.
    unsafe { aucmd_restbuf(&raw mut aco) };
    // SAFETY: the caller's slot, which TermOpen may have emptied. The
    // terminal is only reached again once the slot says it is still there.
    if unsafe { (*termpp).is_null() } || term.buf_handle == 0 {
        return;
    }
    if !term_may_alloc_scrollback(term, Some(buf)) {
        // SAFETY: there is nothing to unwind; the scrollback could not be
        // sized and the buffer can no longer mirror the screen.
        unsafe { abort() };
    }

    // `g:terminal_color_0` .. `_15`, or a buffer-local override, set the
    // emulator's palette. Only the ones actually configured are taken: the
    // rest stay at vterm's defaults, and `color_set` records which, so that
    // the attribute code knows to leave them indexed for the UI to resolve.
    let state = term.state();
    for i in 0..16 {
        let key = format!("terminal_color_{i}\0");
        // SAFETY: `key` is NUL-terminated and `buf` is live.
        let name = unsafe { get_config_string(buf.raw(), key.as_ptr().cast::<c_char>()) };
        if name.is_null() {
            continue;
        }
        // SAFETY: the variable's own NUL-terminated bytes, borrowed for as
        // long as nothing assigns to it — and nothing here does.
        let rgb = name_to_color(unsafe { CStr::from_ptr(name) }).0;
        if rgb == -1 as RgbValue {
            continue;
        }
        let color = VTermColor {
            rgb: VTermColor_rgb {
                type_0: VTERM_COLOR_RGB as uint8_t,
                red: (rgb >> 16 & 0xff) as uint8_t,
                green: (rgb >> 8 & 0xff) as uint8_t,
                blue: (rgb & 0xff) as uint8_t,
            },
        };
        state.set_palette(i, &color);
        term.color_set[i as usize] = true;
    }
}

/// The child exited, or the terminal is being torn down.
///
/// `status` is the child's exit status, or -1 when there was no child to
/// wait for. Runs `TermClose` unless autocommands are blocked; the buffer
/// stays, showing whatever the child left on screen, until something wipes
/// it.
pub(crate) unsafe fn terminal_close(termpp: *mut *mut Terminal, status: c_int) {
    // SAFETY: the caller hands over a slot holding a live terminal.
    let mut term = unsafe { Term::new(*termpp) };
    if term.destroy {
        return;
    }
    let buf = term.buf();

    // Closing an already-closed terminal leaves only the freeing half.
    let only_destroy = term.closed;
    if !only_destroy {
        if !exiting.get() {
            // Show the child's last output before announcing its death.
            //
            // SAFETY: a live terminal, with autocommands blocked around the
            // refresh because mirroring into the buffer would otherwise run
            // Vimscript from here.
            unsafe { block_autocmds() };
            refresh::refresh_terminal(term);
            // SAFETY: as above.
            unsafe { unblock_autocmds() };
        }
        term.closed = true;
    }

    // Where the child left off, reported to `TermClose` as `v:event`-adjacent
    // data.
    let mut pos = buf.map_or(0, |buf| buf.line_count() as c_int - 1);
    if status == -1 || exiting.get() {
        // Nothing to report on: detach from the buffer straight away.
        term.buf_handle = 0 as handle_T;
        if let Some(mut buf) = buf {
            buf.terminal = ::core::ptr::null_mut();
        }
        if term.refcount == 0 {
            term.destroy = true;
            // Read out before the call: the channel's close callback is free
            // to free the terminal.
            let (close_cb, data) = (term.opts.close_cb, term.opts.data);
            // SAFETY: the callback the channel registered, taking the data
            // it registered with it.
            unsafe { close_cb.expect("non-null function pointer")(data) };
        }
    } else if !only_destroy {
        // The status line says "running"; it no longer is.
        let shown = buf.map_or(::core::ptr::null_mut(), Buf::raw);
        for mut wp in windows() {
            if wp.w_buffer == shown {
                wp.w_redr_status = true;
            }
        }
        pos = pos.min(row_to_linenr(term, term.cursor.row));
    }

    if only_destroy {
        return;
    }
    let Some(buf) = buf else { return };
    if is_autocmd_blocked() {
        return;
    }
    // SAFETY: a plain save area `get_v_event` fills in, restored below.
    let mut save_v_event: save_v_event_T = unsafe { ::core::mem::zeroed() };
    // SAFETY: paired with the `restore_v_event` below.
    let dict = unsafe { get_v_event(&raw mut save_v_event) };
    // SAFETY: `dict` is `v:event`, which takes a number under a fixed key.
    unsafe { tv_dict_add_nr(dict, c"status".as_ptr(), 6, status as varnumber_T) };
    // SAFETY: as above.
    unsafe { tv_dict_set_keys_readonly(dict) };
    let mut payload = DictBuf::<1>::new();
    payload.insert(c"pos", Object::integer(pos as i64));
    let mut event = payload.object();
    // Pre-bound so that the eight-argument call still fits on one line.
    let (data, none) = (&mut event, ::core::ptr::null_mut());
    let (exarg, exited) = (::core::ptr::null_mut::<exarg_T>(), status >= 0);
    let (group, buf) = (AUGROUP_ALL, buf.raw());
    // SAFETY: TermClose against a live buffer; nothing of the terminal is
    // borrowed across it.
    unsafe { apply_autocmds_group(EVENT_TERMCLOSE, none, none, exited, group, buf, exarg, data) };
    // SAFETY: paired with the `get_v_event` above.
    unsafe { restore_v_event(dict, &raw mut save_v_event) };
}

/// Redraw the last line of a terminal's buffer, where the "running" /
/// "suspended" marker is drawn.
unsafe extern "C" fn terminal_state_change_event(argv: *mut *mut c_void) {
    // SAFETY: the event carries the buffer handle `terminal_set_state` put
    // in it.
    let handle = unsafe { (*argv).expose_provenance() as handle_T };
    let buf = buf_for_handle(handle);
    if let Some(buf) = buf
        && !buf.terminal.is_null()
    {
        let last = buf.line_count();
        // SAFETY: a live buffer and a line of it.
        unsafe { redraw_buf_line_later(buf.raw(), last, false) };
    }
}

/// Record that the child was stopped or resumed.
///
/// The redraw is deferred: this is reached from a process-status callback,
/// which is no place to touch the screen.
pub(crate) unsafe fn terminal_set_state(term: *mut Terminal, suspended: bool) {
    // SAFETY: the caller hands over a live terminal.
    let mut term = unsafe { Term::new(term) };
    if term.suspended != suspended {
        let event = Event::new(
            Some(terminal_state_change_event),
            [::core::ptr::with_exposed_provenance_mut::<c_void>(
                term.buf_handle as usize,
            )],
        );
        // SAFETY: the refresh queue is the editor's own, live from startup.
        unsafe { multiqueue_put_event(refresh::refresh_queue(), event) };
    }
    term.suspended = suspended;
}

/// Resize the emulator to fit the windows showing it.
///
/// The largest of them, not the smallest: a narrower window scrolls
/// sideways rather than making the child reflow for everyone else.
pub(crate) unsafe fn terminal_check_size(term: *mut Terminal) {
    // SAFETY: the caller hands over a live terminal.
    let mut term = unsafe { Term::new(term) };
    if term.closed {
        return;
    }
    let (curheight, curwidth) = term.size();

    let (mut width, mut height) = (0, 0);
    for wp in tab_windows() {
        // The autocommand window is a fiction with a nominal size.
        // SAFETY: a window of the current tab page's own list.
        if unsafe { is_aucmd_win(wp.raw()) } {
            continue;
        }
        if wp
            .buffer_or_none()
            .is_none_or(|buf| buf.terminal != term.raw())
        {
            continue;
        }
        // SAFETY: as above.
        let text_width = wp.w_view_width - unsafe { win_col_off(wp.raw()) };
        width = width.max(text_width.max(0));
        height = height.max(wp.w_view_height);
    }

    // Zero means no window is showing it; keep whatever size it had.
    if (curheight == height && curwidth == width) || height == 0 || width == 0 {
        return;
    }
    let vt = term.vt;
    // SAFETY: the terminal's own emulator. The resize reaches the screen's
    // consumer, so nothing of the terminal is borrowed across it.
    unsafe { vterm_set_size(vt, height, width) };
    term.flush_damage();
    term.pending.resize = true;
    refresh::invalidate_terminal(term, None);
}

/// Free the terminal, if nothing is still standing on it.
///
/// Reached repeatedly — from the close callback, from the buffer being
/// wiped — and does nothing until `refcount` reaches zero.
pub(crate) unsafe fn terminal_destroy(termpp: *mut *mut Terminal) {
    // SAFETY: the caller hands over a slot holding a live terminal.
    let mut term = unsafe { Term::new(*termpp) };
    if let Some(mut buf) = term.buf() {
        term.buf_handle = 0 as handle_T;
        buf.terminal = ::core::ptr::null_mut();
    }
    if term.refcount != 0 {
        return;
    }
    let (raw, vt, events) = (term.raw(), term.vt, term.pending.events);
    refresh::refresh_before_destroy(term);
    // SAFETY: the terminal is the last thing standing on all of these, and
    // each is released exactly once.
    unsafe { vterm_free(vt) };
    // SAFETY: as above.
    unsafe { multiqueue_free(events) };
    // SAFETY: the other half of `terminal_alloc`'s `Box::into_raw`;
    // everything the terminal owns goes with it, and the caller's slot is
    // emptied so that nothing reaches it again.
    unsafe {
        drop(Box::from_raw(raw));
        *termpp = ::core::ptr::null_mut();
    }
}

/// Write `data` to the child, or hold it if a `TermRequest` handler is
/// running.
///
/// See [`TerminalPending::send`](crate::types::TerminalPending).
fn terminal_send(term: Term, data: &[u8]) {
    if term.closed {
        return;
    }
    let held = term.pending.send;
    if !held.is_null() {
        if !data.is_empty() {
            // SAFETY: the buffer is borrowed from the in-flight request,
            // which outlives it.
            unsafe { (*held).extend_from_slice(data) };
        }
        return;
    }
    // Read out before the call: the channel's write callback may re-enter.
    let (write_cb, user) = (term.opts.write_cb, term.opts.data);
    let (bytes, size) = (data.as_ptr().cast::<c_char>(), data.len());
    // SAFETY: the callback the channel registered, taking the data it
    // registered with it and the caller's own bytes.
    unsafe { write_cb.expect("non-null function pointer")(bytes, size, user) };
}

/// Redraw after the child closed a synchronized-output frame.
unsafe extern "C" fn on_sync_flush(argv: *mut *mut c_void) {
    if exiting.get() {
        return;
    }
    // SAFETY: the event carries the buffer handle `terminal_receive` put in
    // it.
    let handle = unsafe { (*argv).expose_provenance() as handle_T };
    let buf = buf_for_handle(handle);
    let Some(buf) = buf.filter(|buf| !buf.terminal.is_null()) else {
        return;
    };
    // SAFETY: a buffer that still has its terminal.
    let term = unsafe { Term::new(buf.terminal) };
    // SAFETY: autocommands are blocked around the refresh because
    // mirroring into the buffer would otherwise run Vimscript from the
    // middle of the event loop; paired with the unblock below.
    unsafe { block_autocmds() };
    refresh::refresh_terminal(term);
    // SAFETY: as above.
    unsafe { unblock_autocmds() };
}

/// Feed `len` bytes of the child's output to the emulator.
///
/// `force_crlf` is for channels that are not a pty: a bare newline from
/// those means "next line, column zero", which to a terminal is CR LF.
pub(crate) unsafe fn terminal_receive(term: *mut Terminal, data: *const c_char, len: size_t) {
    // SAFETY: the caller hands over a live terminal.
    let mut term = unsafe { Term::new(term) };
    if data.is_null() {
        return;
    }
    if term.opts.force_crlf {
        // SAFETY: `data` points at `len` readable bytes.
        let bytes = unsafe { ::core::slice::from_raw_parts(data.cast::<u8>(), len) };
        let mut crlf = Vec::with_capacity(len);
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' && (i == 0 || bytes[i - 1] != b'\r') {
                crlf.push(b'\r' as c_char);
            }
            crlf.push(byte as c_char);
        }
        term.write(&crlf);
    } else {
        // SAFETY: as above.
        term.write(unsafe { ::core::slice::from_raw_parts(data, len) });
    }
    term.flush_damage();

    if term.sync_flush_pending {
        // The frame the child was assembling is complete: invalidate all of
        // it and redraw once, from the main loop rather than from the middle
        // of parsing its output.
        term.sync_flush_pending = false;
        let (height, _) = term.size();
        term.invalid_start = 0;
        term.invalid_end = height;
        let event = Event::new(
            Some(on_sync_flush),
            [::core::ptr::with_exposed_provenance_mut::<c_void>(
                term.buf_handle as usize,
            )],
        );
        // SAFETY: the main loop's queue is the editor's own, live until it
        // stops.
        unsafe { multiqueue_put_event(main_loop_events(), event) };
    }
}

/// A vterm colour's type byte, which every arm of the union shares.
///
/// vterm fills a colour in whole, so the reads below all rest on the same
/// thing; it is booked here once instead of at each of the eight uses.
fn color_type(color: &VTermColor) -> c_uint {
    // SAFETY: `type_0` is an arm of its own, at offset 0 of the others.
    c_uint::from(unsafe { color.type_0 })
}

/// The palette entry an indexed colour names.
fn color_index(color: &VTermColor) -> int16_t {
    // SAFETY: as above; the caller has checked the type says indexed.
    int16_t::from(unsafe { color.indexed.idx })
}

/// The highlight bit for a cell's underline style. vterm names more styles
/// than the editor draws, so anything else becomes a plain underline.
fn get_underline_hl_flag(attrs: VTermScreenCellAttrs) -> HlAttrFlags {
    match attrs.underline() {
        0 => HlAttrFlags::NONE,
        2 => HlAttrFlags::UNDERDOUBLE,
        3 => HlAttrFlags::UNDERCURL,
        _ => HlAttrFlags::UNDERLINE,
    }
}

/// Resolve one buffer line of a terminal into per-column highlight ids.
///
/// The screen painter calls this for every visible line of a terminal
/// buffer; `term_attrs` is its scratch array, `TERM_ATTRS_MAX` wide. Lines
/// that are scrollback rather than screen resolve through the scrollback,
/// and lines below the screen are left alone.
pub(crate) unsafe fn terminal_get_line_attributes(
    term: *mut Terminal,
    _wp: *mut win_T,
    linenr: c_int,
    term_attrs: *mut c_int,
) {
    // SAFETY: the caller hands over a live terminal.
    let term = unsafe { Term::new(term) };
    let (height, width) = term.size();
    let state = term.state();
    debug_assert!(linenr != 0, "buffer line numbers are one-based");

    let row = linenr_to_row(term, linenr);
    if row >= height {
        return;
    }
    let width = width.min(TERM_ATTRS_MAX);

    for col in 0..width {
        // False for a scrollback cell past the end of a row stored while
        // the terminal was narrower; such a cell has no colours at all.
        //
        // SAFETY: all-zeroes is a valid cell — every field is a scalar or a
        // union of them — and `fetch_cell` fills it in whole either way.
        let mut cell: VTermScreenCell = unsafe { ::core::mem::zeroed() };
        let color_valid = fetch_cell(term, row, col, &mut cell);
        let fg_default = !color_valid || color_type(&cell.fg) & VTERM_COLOR_DEFAULT_FG != 0;
        let bg_default = !color_valid || color_type(&cell.bg) & VTERM_COLOR_DEFAULT_BG != 0;
        let fg_indexed = color_type(&cell.fg) & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED;
        let bg_indexed = color_type(&cell.bg) & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED;

        // The cterm colour is one-based so that zero can mean "unset".
        let vt_fg_idx: int16_t = if !fg_default && fg_indexed {
            color_index(&cell.fg) + 1
        } else {
            0
        };
        let vt_bg_idx: int16_t = if !bg_default && bg_indexed {
            color_index(&cell.bg) + 1
        } else {
            0
        };
        // A palette entry the user configured resolves to a real RGB value
        // here; one they did not stays indexed, so that the UI's own palette
        // applies instead.
        let fg_set = vt_fg_idx != 0 && vt_fg_idx <= 16 && term.color_set[(vt_fg_idx - 1) as usize];
        let bg_set = vt_bg_idx != 0 && vt_bg_idx <= 16 && term.color_set[(vt_bg_idx - 1) as usize];

        let attrs = cell.attrs;
        let mut hl_attrs = get_underline_hl_flag(attrs);
        for (set, bit) in [
            (attrs.bold() != 0, HlAttrFlags::BOLD),
            (attrs.dim() != 0, HlAttrFlags::DIM),
            (attrs.blink() != 0, HlAttrFlags::BLINK),
            (attrs.conceal() != 0, HlAttrFlags::CONCEALED),
            (attrs.overline() != 0, HlAttrFlags::OVERLINE),
            (attrs.italic() != 0, HlAttrFlags::ITALIC),
            (attrs.reverse() != 0, HlAttrFlags::INVERSE),
            (attrs.strike() != 0, HlAttrFlags::STRIKETHROUGH),
            (fg_indexed && !fg_set, HlAttrFlags::FG_INDEXED),
            (bg_indexed && !bg_set, HlAttrFlags::BG_INDEXED),
        ] {
            if set {
                hl_attrs |= bit;
            }
        }

        let mut attr_id = 0;
        if !hl_attrs.is_empty() || !fg_default || !bg_default {
            let resolved = HlAttrs {
                rgb_ae_attr: hl_attrs,
                cterm_ae_attr: hl_attrs,
                rgb_fg_color: if fg_default { -1 } else { state.rgb(cell.fg) },
                rgb_bg_color: if bg_default { -1 } else { state.rgb(cell.bg) },
                rgb_sp_color: -1 as RgbValue,
                cterm_fg_color: vt_fg_idx,
                cterm_bg_color: vt_bg_idx,
                hl_blend: -1,
                url: -1,
            };
            // SAFETY: reads the highlight tables, which are the editor's own.
            attr_id = unsafe { hl_get_term_attr(resolved) };
        }
        // A hyperlink is its own attribute, layered over the colours.
        if cell.uri > 0 {
            // SAFETY: as above.
            attr_id = unsafe { hl_combine_attr(attr_id, cell.uri) };
        }
        // SAFETY: the caller's scratch array is `TERM_ATTRS_MAX` wide and
        // `width` was clamped to it.
        unsafe { *term_attrs.add(col as usize) = attr_id };
    }
}

pub(crate) unsafe fn terminal_buf(term: *const Terminal) -> Buffer {
    // SAFETY: the caller hands over a live terminal.
    unsafe { (*term).buf_handle as Buffer }
}

pub(crate) unsafe fn terminal_running(term: *const Terminal) -> bool {
    // SAFETY: the caller hands over a live terminal.
    unsafe { !(*term).closed }
}

pub(crate) unsafe fn terminal_suspended(term: *const Terminal) -> bool {
    // SAFETY: the caller hands over a live terminal.
    unsafe { (*term).suspended }
}

/// Tell a child that asked for theme updates that `'background'` changed.
pub(crate) unsafe fn terminal_notify_theme(term: *mut Terminal, dark: bool) {
    // SAFETY: the caller hands over a live terminal.
    if !unsafe { Term::new(term) }.theme_updates {
        return;
    }
    let report: &[u8] = if dark { b"\x1b[997;1n" } else { b"\x1b[997;2n" };
    // SAFETY: as above.
    terminal_send(unsafe { Term::new(term) }, report);
}

/// The buffer line an emulator row appears on, counting the scrollback
/// above it. The "nothing invalid" sentinel passes through unchanged.
fn row_to_linenr(term: Term, row: c_int) -> c_int {
    if row == c_int::MAX {
        return c_int::MAX;
    }
    row + term.sb.len() as c_int + 1
}

/// The inverse of [`row_to_linenr`]. Negative for a scrollback line.
fn linenr_to_row(term: Term, linenr: c_int) -> c_int {
    linenr - term.sb.len() as c_int - 1
}

/// Whether the user is typing at this terminal right now.
fn is_focused(term: Term) -> bool {
    // SAFETY: `curbuf` is set from startup to exit.
    State.get() & MODE_TERMINAL != 0 && unsafe { Buf::current() }.terminal == term.raw()
}

/// What `dict` holds under `key`, or nil.
///
/// The lookup cannot fail in a way this module's caller could act on, so the
/// error is cleared and dropped. **The answer BORROWS `dict`**:
/// `dict_get_value` converts with `reuse_strdata`, so a string in it points
/// at the variable's own bytes rather than at a copy.
///
/// # Safety
/// `dict` must be a live dictionary and `key` NUL-terminated.
unsafe fn dict_lookup(dict: *mut dict_T, key: *const c_char) -> Object {
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut(),
    };
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: forwarded to this function's own caller; `err` is this
    // function's and is released before it returns.
    unsafe {
        let obj = dict_get_value(dict, cstr_as_string(key), no_arena, &raw mut err);
        api_clear_error(&raw mut err);
        obj
    }
}

/// `b:<key>`, falling back to `g:<key>`, if it is a string.
///
/// The result BORROWS the variable's own bytes, or is null. It must not be
/// freed, and it stays valid only until something assigns to or unsets the
/// variable.
unsafe fn get_config_string(buf: *mut buf_T, key: *const c_char) -> *mut c_char {
    // SAFETY: `buf` is a live buffer and `key` is NUL-terminated.
    let mut obj = unsafe { dict_lookup((*buf).b_vars, key) };
    if obj.type_0 == kObjectTypeNil {
        // SAFETY: as above, against the global variables.
        obj = unsafe { dict_lookup(get_globvar_dict(), key) };
    }
    if obj.type_0 == kObjectTypeString {
        // SAFETY: the object is a string, so this is the live arm — and the
        // bytes are the variable's, so nothing here owns them.
        return unsafe { obj.data.string.data() };
    }
    // SAFETY: not a string, so there is no borrowed `String` to release.
    unsafe { api_free_object(obj) };
    ::core::ptr::null_mut()
}
