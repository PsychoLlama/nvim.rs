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
//! `Buffer` across anything that might run autocommands — `buf_handle` plus
//! [`buf_for_handle`] is the pattern throughout. `refcount` is the other
//! half: it is raised around anything that can run Vimscript, and
//! [`terminal_destroy`] only frees at zero. The terminal itself is reached
//! through [`Term`], the handle `winlayer` keeps beside `Win` and `Buf`,
//! which hands out one short-lived borrow per use; the emulator is a
//! [`VTerm`] the terminal owns and drives through its methods.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub(crate) mod callbacks;
pub(crate) mod input;
pub(crate) mod mode;
pub(crate) mod refresh;
pub(crate) mod scrollback;
pub(crate) mod termrequest;

use crate::api::private::helpers::{dict_get_value, scope_vars};
use crate::autocmd::{
    apply_autocmds, apply_autocmds_group, aucmd_prepbuf, aucmd_restbuf, block_autocmds,
    is_aucmd_win, is_autocmd_blocked, unblock_autocmds,
};
use crate::change::deleted_lines_buf;
use crate::cursor_shape::{SHAPE_IDX_TERM, shape_entry};
use crate::drawscreen::redraw_buf_line_later;
use crate::eval::typval::DictRef;
use crate::eval::vars::globvar_dict;
use crate::eval::{get_v_event, restore_v_event};
use crate::event_queue::{self, ParkingQueue, put_id_event_on_main_loop};
use crate::highlight::{HlAttrFlags, hl_combine_attr, hl_get_term_attr};
use crate::highlight_group::name_to_color;
use crate::memline::MlFlags;
use crate::memline::ml_delete_buf;
use crate::option::set_option_value;
use crate::options::kOptBuftype;
use crate::startup::exiting;
use crate::state::mode::State;
use crate::types::ApiString;
use crate::types::AutoEvent;
use crate::types::builders::DictBuf;
use crate::types::terminal_defs::SELECTIONBUF_SIZE;
use crate::types::{
    AcoSave, BufferHandle, ColNr, Event, Handle, HlAttrs, LineNr, Object, OptVal, OptionSetFlags,
    Pos, RefcountSize, RgbValue, SaveVEvent, Terminal, TerminalOptions, VTermColor, VTermColor_rgb,
    VTermScreenCell, VTermScreenCellAttrs, VarNumber, int16_t, uint8_t,
};
use crate::vterm::host::Value;
use crate::vterm::screen::ZERO_SCREEN_CELL;
use crate::vterm::vterm::{
    OutputSink, VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG, VTERM_COLOR_INDEXED,
    VTERM_COLOR_RGB, VTERM_COLOR_TYPE_MASK, VTERM_DAMAGE_SCROLL, VTERM_PROP_CURSORBLINK,
    VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORSHAPE_BAR_LEFT, VTERM_PROP_CURSORSHAPE_BLOCK,
    VTERM_PROP_CURSORSHAPE_UNDERLINE,
};
use crate::winlayer::{self, Buf, Win, tab_windows, windows};
use core::ffi::{CStr, c_int, c_uint};

use scrollback::{fetch_cell, refresh_scrollback, term_may_alloc_scrollback};

use crate::state::MODE_TERMINAL;
pub(crate) use crate::winlayer::Term;
pub(crate) use input::{buf_terminal_set_streamed_paste, terminal_paste};
pub(crate) use mode::{TerminalState, terminal_check, terminal_enter, terminal_execute};
pub(crate) use refresh::{
    on_scrollback_changed_in, terminal_check_refresh, terminal_init, terminal_teardown,
};

/// The largest `'scrollback'` that means anything; the option's negative
/// "unlimited" spelling becomes this.
const SB_MAX: c_int = 1000000;
/// The most columns [`terminal_get_line_attributes`] will resolve;
/// `drawline` sizes its array to match.
const TERM_ATTRS_MAX: c_int = 1024;
/// Every autocommand group, for the events dispatched from here.
const AUGROUP_ALL: c_int = -3;

// ---------------------------------------------------------------------------
// The emulator, through the handle

impl Terminal {
    /// The emulator's size, rows first.
    fn size(&self) -> (c_int, c_int) {
        self.vt.size()
    }

    /// `color` as a packed 24-bit RGB value, resolving the palette.
    fn rgb(&mut self, color: VTermColor) -> c_int {
        let (red, green, blue) = self.vt.rgb_of(color).channels();
        (c_int::from(red) << 16) | (c_int::from(green) << 8) | c_int::from(blue)
    }
}

/// Feed bytes of the child's output to the emulator. Re-entrant: vterm
/// reports what changed through the screen callbacks, which reach this
/// terminal again through their own copy of the handle.
fn write_to_emulator(mut term: Term, bytes: &[u8]) {
    term.vt.input_write(bytes);
}

/// The buffer this terminal draws into, `None` once it has been wiped.
fn term_buf(term: Term) -> Option<Buf> {
    buf_for_handle(term.buf_handle)
}

/// The buffer a handle names, `None` once it has been wiped.
fn buf_for_handle(handle: Handle) -> Option<Buf> {
    winlayer::buffer(handle)
}

/// Create a terminal for `buffer` and wire it to vterm.
///
/// The buffer is emptied: its lines are about to become a mirror of the
/// emulator's screen, and anything already there would be taken for
/// scrollback.
///
/// The buffer owns the terminal from here; whoever drives it frees it with
/// [`terminal_destroy`].
pub(crate) fn terminal_alloc(mut buffer: Buf, opts: TerminalOptions) -> Term {
    // Owned by the buffer from here; every other reference reaches it
    // through `Buffer::terminal`, and `terminal_destroy` frees it.
    let mut term = Term::alloc(Terminal::new(opts, buffer.handle));
    buffer.terminal = term.raw();

    // None of the consumers being installed can run before this returns.
    term.vt.set_utf8(true);
    term.vt.obtain_screen();
    term.vt.screen_enable_altscreen();
    term.vt.obtain_screen().enable_reflow(true);
    let handle = term;
    term.vt
        .set_screen_host(Box::new(callbacks::ScreenReports(handle)));
    term.vt
        .set_fallbacks(Box::new(termrequest::Requests(handle)));
    // Merge damage reports up to a whole scrolled region before they are
    // delivered; the refresh works in row ranges anyway.
    term.vt.set_damage_merge(VTERM_DAMAGE_SCROLL);
    term.vt.screen_reset(true);
    term.vt
        .set_output(OutputSink::Rust(Box::new(move |bytes: &[u8]| {
            terminal_send(handle, bytes);
        })));
    term.vt
        .set_selection_host(Box::new(callbacks::Clipboard(handle)), SELECTIONBUF_SIZE);

    // Start the child off with the cursor the user configured for terminal
    // mode; it is free to change it.
    let shape = shape_entry(SHAPE_IDX_TERM);
    // vterm's shapes are DECSCUSR's rather than the editor's.
    let cursor_shape = match shape.shape {
        0 => VTERM_PROP_CURSORSHAPE_BLOCK,
        1 => VTERM_PROP_CURSORSHAPE_UNDERLINE,
        2 => VTERM_PROP_CURSORSHAPE_BAR_LEFT,
        // Not a shape vterm knows; leave its default alone.
        _ => 0,
    };
    let cursor_blink = shape.blinkon != 0 && shape.blinkoff != 0;
    let _ = term
        .vt
        .set_termprop(VTERM_PROP_CURSORSHAPE, Value::Int(cursor_shape));
    let _ = term
        .vt
        .set_termprop(VTERM_PROP_CURSORBLINK, Value::Bool(cursor_blink));

    term.invalid_start = 0;
    term.invalid_end = opts.height as c_int;
    term.pending.events = Some(ParkingQueue::detached());

    if !buffer.b_ml.ml_flags.has(MlFlags::EMPTY) {
        let line_count = buffer.line_count();
        // Not immutable: ml_delete_buf() mutates b_ml behind the pointer.
        #[allow(clippy::while_immutable_condition)]
        while !buffer.b_ml.ml_flags.has(MlFlags::EMPTY) {
            let _ = ml_delete_buf(buffer, LineNr::ONE, false);
        }
        deleted_lines_buf(buffer, LineNr::ONE, line_count.get());
    }
    term.old_height = 1;
    term
}

/// Make `buffer` look and behave like a terminal buffer, and announce it.
///
/// Runs `TermOpen`, which can wipe the buffer or close the terminal
/// outright — hence the re-check before touching either again: `attached`
/// answers whether the terminal's driver still holds it, and only then is
/// it reached.
pub(crate) fn terminal_open(mut term: Term, mut buffer: Buf, attached: &dyn Fn() -> bool) {
    let mut aco = AcoSave::default();
    // SAFETY: paired with the `aucmd_restbuf` below.
    aucmd_prepbuf(&mut aco, buffer);
    if term.sb.is_sized() {
        refresh_scrollback(term, buffer);
    } else {
        debug_assert!(term.invalid_start >= 0);
    }
    refresh::refresh_screen(term, buffer);

    // Locked because setting 'buftype' can run OptionSet, and the buffer's
    // lines are the emulator's to write.
    buffer.b_locked += 1;
    let _ = set_option_value(
        kOptBuftype,
        OptVal::static_string(c"terminal"),
        OptionSetFlags::LOCAL,
    );
    buffer.b_locked -= 1;

    // A copy: the title is set on the buffer whose name it is.
    if let Some(title) = buffer.name.full().map(|name| name.to_bytes().to_vec()) {
        callbacks::buf_set_term_title(Some(buffer), &title);
    }
    // Both would tie the terminal window's scroll position to another
    // window's, which fights the emulator for the topline.
    let mut win = Win::current();
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
    win.w_cursor = Pos {
        lnum: LineNr::ONE,
        col: ColNr::ZERO,
        coladd: 0,
    };
    // SAFETY: TermOpen against a live buffer. It may wipe the buffer or
    // close the terminal, which is what the re-check below is for, and
    // nothing of either is borrowed across it.
    apply_autocmds(AutoEvent::TermOpen, None, None, false, Some(buffer));
    // SAFETY: paired with the `aucmd_prepbuf` above.
    aucmd_restbuf(&mut aco);
    // TermOpen may have closed and freed the terminal; it is only reached
    // again once its driver says it still holds it.
    if !attached() || term.buf_handle == Handle::NONE {
        return;
    }
    if !term_may_alloc_scrollback(term, Some(buffer)) {
        // There is nothing to unwind; the scrollback could not be sized and
        // the buffer can no longer mirror the screen.
        std::process::abort();
    }

    // `g:terminal_color_0` .. `_15`, or a buffer-local override, set the
    // emulator's palette. Only the ones actually configured are taken: the
    // rest stay at vterm's defaults, and `color_set` records which, so that
    // the attribute code knows to leave them indexed for the UI to resolve.
    for i in 0..16 {
        let key = format!("terminal_color_{i}\0");
        let key = CStr::from_bytes_with_nul(key.as_bytes()).expect("one NUL, at the end");
        let Some(name) = get_config_string(buffer, key) else {
            continue;
        };
        let rgb = name_to_color(name.as_cstr()).0;
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
        term.vt.set_palette_color(i, &color);
        term.color_set[i as usize] = true;
    }
}

/// The child exited, or the terminal is being torn down.
///
/// `status` is the child's exit status, or -1 when there was no child to
/// wait for. Runs `TermClose` unless autocommands are blocked; the buffer
/// stays, showing whatever the child left on screen, until something wipes
/// it.
///
pub(crate) fn terminal_close(mut term: Term, status: c_int) {
    if term.destroy {
        return;
    }
    let buf = term_buf(term);

    // Closing an already-closed terminal leaves only the freeing half.
    let only_destroy = term.closed;
    if !only_destroy {
        if !exiting.get() {
            // Show the child's last output before announcing its death.
            //
            block_autocmds();
            refresh::refresh_terminal(term);
            unblock_autocmds();
        }
        term.closed = true;
    }

    // Where the child left off, reported to `TermClose` as `v:event`-adjacent
    // data.
    let mut pos = buf.map_or(0, |buf| buf.line_count().get() - 1);
    if status == -1 || exiting.get() {
        // Nothing to report on: detach from the buffer straight away.
        term.buf_handle = Handle::NONE;
        if let Some(mut buf) = buf {
            buf.terminal = ::core::ptr::null_mut();
        }
        if term.refcount == RefcountSize::ZERO {
            term.destroy = true;
            // Read out before the call: the channel's close callback is free
            // to free the terminal.
            let (close_cb, owner) = (term.opts.close_cb, term.opts.owner);
            close_cb.expect("non-null function pointer")(owner);
        }
    } else if !only_destroy {
        // The status line says "running"; it no longer is.
        for mut wp in windows() {
            if Some(wp.w_buffer) == buf {
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
    // A plain save area `get_v_event` fills in, restored below.
    let mut save_v_event = SaveVEvent::default();
    let dict = get_v_event(&mut save_v_event);
    let _ = dict.edit().add_number(b"status", status as VarNumber);
    dict.edit().set_keys_readonly();
    let mut payload = DictBuf::<1>::new();
    payload.insert(c"pos", Object::integer(pos as i64));
    let data = payload.object();
    let exited = status >= 0;
    let (group, buf) = (AUGROUP_ALL, Some(buf));
    // TermClose against a live buffer; nothing of the terminal is borrowed
    // across it.
    let event = AutoEvent::TermClose;
    apply_autocmds_group(event, None, None, exited, group, buf, None, Some(&data));
    restore_v_event(dict, &mut save_v_event);
}

/// Redraw the last line of a terminal's buffer, where the "running" /
/// "suspended" marker is drawn.
///
/// A buffer handle as an id event's word.
fn handle_word(handle: Handle) -> u64 {
    u64::from(handle.get().cast_unsigned())
}

/// The buffer handle [`handle_word`] packed.
fn handle_of(word: u64) -> Handle {
    Handle::new(
        u32::try_from(word)
            .expect("`handle_word` packs a handle")
            .cast_signed(),
    )
}

/// The child behind buffer `handle`'s terminal was stopped or resumed:
/// redraw its last line, once the refresh queue gets here.
fn terminal_state_change_event(handle: u64, _: u64) {
    let buf = buf_for_handle(handle_of(handle));
    if let Some(buf) = buf
        && !buf.terminal.is_null()
    {
        let last = buf.line_count();
        redraw_buf_line_later(buf, last, false);
    }
}

/// Record that the child was stopped or resumed.
///
/// The redraw is deferred: this is reached from a process-status callback,
/// which is no place to touch the screen.
pub(crate) fn terminal_set_state(mut term: Term, suspended: bool) {
    if term.suspended != suspended
        && let Some(queue) = refresh::refresh_queue()
    {
        let event = Event::Ids(terminal_state_change_event, handle_word(term.buf_handle), 0);
        event_queue::put(queue, event);
    }
    term.suspended = suspended;
}

/// [`terminal_check_size`] for the terminal `buffer` holds, if it holds one.
pub(crate) fn buf_terminal_check_size(buffer: Buf) {
    if let Some(term) = buffer.terminal_handle() {
        terminal_check_size(term);
    }
}

/// Whether `buffer` holds a terminal whose job is still running.
pub(crate) fn buf_terminal_running(buffer: Buf) -> bool {
    buffer.terminal_handle().is_some_and(terminal_running)
}

/// Resize the emulator to fit the windows showing it.
///
/// The largest of them, not the smallest: a narrower window scrolls
/// sideways rather than making the child reflow for everyone else.
pub(crate) fn terminal_check_size(mut term: Term) {
    if term.closed {
        return;
    }
    let (curheight, curwidth) = term.size();

    let (mut width, mut height) = (0, 0);
    for wp in tab_windows() {
        // The autocommand window is a fiction with a nominal size.
        // SAFETY: a window of the current tab page's own list.
        if is_aucmd_win(wp) {
            continue;
        }
        if wp
            .buffer_or_none()
            .is_none_or(|buf| buf.terminal != term.raw())
        {
            continue;
        }
        let text_width = wp.w_view_width - wp.col_off();
        width = width.max(text_width.max(0));
        height = height.max(wp.w_view_height);
    }

    // Zero means no window is showing it; keep whatever size it had.
    if (curheight == height && curwidth == width) || height == 0 || width == 0 {
        return;
    }
    // The resize reaches the screen's consumer, which comes back to the
    // terminal through its own handle.
    term.vt.set_size(height, width);
    term.vt.flush_damage();
    term.pending.resize = true;
    refresh::invalidate_terminal(term, None);
}

/// Free the terminal, if nothing is still standing on it.
///
/// Reached repeatedly — from the close callback, from the buffer being
/// wiped — and does nothing until `refcount` reaches zero. Answers whether
/// it freed the terminal, after which the caller drops its handle.
pub(crate) fn terminal_destroy(mut term: Term) -> bool {
    if let Some(mut buf) = term_buf(term) {
        term.buf_handle = Handle::NONE;
        buf.terminal = ::core::ptr::null_mut();
    }
    if term.refcount != RefcountSize::ZERO {
        return false;
    }
    let events = term.pending.events.take();
    refresh::refresh_before_destroy(term);
    drop(events);
    // Everything the terminal owns goes with it, the emulator included, and
    // the caller drops its handle so that nothing reaches it again.
    term.free();
    true
}

/// Write `data` to the child, or hold it if a `TermRequest` handler is
/// running.
///
/// See [`TerminalPending::send`](crate::types::TerminalPending).
fn terminal_send(term: Term, data: &[u8]) {
    if term.closed {
        return;
    }
    if let Some(held) = &term.pending.send {
        if !data.is_empty() {
            held.borrow_mut().extend_from_slice(data);
        }
        return;
    }
    // Read out before the call: the channel's write callback may re-enter.
    let (write_cb, owner) = (term.opts.write_cb, term.opts.owner);
    write_cb.expect("non-null function pointer")(data, owner);
}

/// Redraw after the child closed a synchronized-output frame.
///
/// The child finished a synchronized frame in buffer `handle`'s terminal:
/// redraw it, from the main loop.
fn on_sync_flush(handle: u64, _: u64) {
    if exiting.get() {
        return;
    }
    let Some(term) = buf_for_handle(handle_of(handle)).and_then(Buf::terminal_handle) else {
        return;
    };
    block_autocmds();
    refresh::refresh_terminal(term);
    unblock_autocmds();
}

/// Feed `len` bytes of the child's output to the emulator.
///
/// `force_crlf` is for channels that are not a pty: a bare newline from
/// those means "next line, column zero", which to a terminal is CR LF.
///
pub(crate) fn terminal_receive(mut term: Term, bytes: &[u8]) {
    if term.opts.force_crlf {
        let mut crlf = Vec::with_capacity(bytes.len());
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' && (i == 0 || bytes[i - 1] != b'\r') {
                crlf.push(b'\r');
            }
            crlf.push(byte);
        }
        write_to_emulator(term, &crlf);
    } else {
        write_to_emulator(term, bytes);
    }
    term.vt.flush_damage();

    if term.sync_flush_pending {
        // The frame the child was assembling is complete: invalidate all of
        // it and redraw once, from the main loop rather than from the middle
        // of parsing its output.
        term.sync_flush_pending = false;
        let (height, _) = term.size();
        term.invalid_start = 0;
        term.invalid_end = height;
        put_id_event_on_main_loop(on_sync_flush, handle_word(term.buf_handle), 0);
    }
}

/// A vterm colour's type byte, which every arm of the union shares.
///
/// vterm fills a colour in whole, so the reads below all rest on the same
/// thing; it is booked here once instead of at each of the eight uses.
fn color_type(color: &VTermColor) -> c_uint {
    c_uint::from(color.type_byte())
}

/// The palette entry an indexed colour names.
fn color_index(color: &VTermColor) -> int16_t {
    int16_t::from(color.index())
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
///
/// `buffer` must hold a terminal.
pub(crate) fn terminal_get_line_attributes(
    buffer: Buf,
    linenr: c_int,
    term_attrs: &mut [c_int; TERM_ATTRS_MAX as usize],
) {
    let mut term = buffer.terminal_handle().expect("buf->terminal");
    let (height, width) = term.size();
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
        // `fetch_cell` fills it in whole either way.
        let mut cell: VTermScreenCell = ZERO_SCREEN_CELL;
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
                rgb_fg_color: if fg_default { -1 } else { term.rgb(cell.fg) },
                rgb_bg_color: if bg_default { -1 } else { term.rgb(cell.bg) },
                rgb_sp_color: -1 as RgbValue,
                cterm_fg_color: vt_fg_idx,
                cterm_bg_color: vt_bg_idx,
                hl_blend: -1,
                url: -1,
            };
            attr_id = hl_get_term_attr(resolved);
        }
        // A hyperlink is its own attribute, layered over the colours.
        if cell.uri > 0 {
            attr_id = hl_combine_attr(attr_id, cell.uri);
        }
        // `width` was clamped to the array.
        term_attrs[col as usize] = attr_id;
    }
}

/// The buffer `term` draws into, 0 once it has been detached.
pub(crate) fn terminal_buf(term: Term) -> BufferHandle {
    term.buf_handle as BufferHandle
}

/// [`terminal_close`] of `buffer`'s terminal, which must be set.
pub(crate) fn close_buf_terminal(buffer: Buf) {
    terminal_close(buffer.terminal_handle().expect("buf->terminal"), -1);
}

/// [`terminal_check_size`] of `buffer`'s terminal, when it has one.
pub(crate) fn check_buf_terminal_size(buffer: Buf) {
    if let Some(term) = buffer.terminal_handle() {
        terminal_check_size(term);
    }
}

/// Whether `term`'s child is still running.
pub(crate) fn terminal_running(term: Term) -> bool {
    !term.closed
}

/// Whether `buffer` holds a terminal whose job is suspended.
pub(crate) fn buf_terminal_suspended(buffer: Buf) -> bool {
    buffer.terminal_handle().is_some_and(|term| term.suspended)
}

/// Tell the child of `buffer`'s terminal, if it has one and asked for theme
/// updates, that `'background'` changed.
pub(crate) fn terminal_notify_theme(buffer: Buf, dark: bool) {
    let Some(term) = buffer.terminal_handle() else {
        return;
    };
    if !term.theme_updates {
        return;
    }
    let report: &[u8] = if dark { b"\x1b[997;1n" } else { b"\x1b[997;2n" };
    terminal_send(term, report);
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
    State.get() & MODE_TERMINAL != 0 && Buf::current().terminal == term.raw()
}

/// What `dict` holds under `key`, or nil.
///
/// The lookup cannot fail in a way this module's caller could act on, so the
/// error is cleared and dropped. **The answer BORROWS `dict`**:
/// The answer is a copy of the variable's value.
///
/// A key that is not there answers nil rather than a refusal.
fn dict_lookup(dict: &DictRef, key: &CStr) -> Object {
    dict_get_value(dict, &ApiString::from_cstr(key)).unwrap_or(Object::Nil)
}

/// `b:<key>`, falling back to `g:<key>`, if it is a string.
fn get_config_string(buffer: Buf, key: &CStr) -> Option<ApiString> {
    let mut obj = dict_lookup(&scope_vars(&buffer.b_bufvar), key);
    if obj.is_nil() {
        obj = dict_lookup(&globvar_dict(), key);
    }
    obj.into_string()
}
