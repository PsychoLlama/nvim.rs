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
//! [`terminal_destroy`] only frees at zero.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod callbacks;
pub mod input;
pub mod mode;
pub mod refresh;
pub mod scrollback;
pub mod termrequest;

use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, cstr_as_string, dict_get_value,
};
use crate::src::nvim::autocmd::{
    EVENT_TERMCLOSE, EVENT_TERMOPEN, apply_autocmds, apply_autocmds_group, aucmd_prepbuf,
    aucmd_restbuf, block_autocmds, is_aucmd_win, is_autocmd_blocked, unblock_autocmds,
};
use crate::src::nvim::change::deleted_lines_buf;
use crate::src::nvim::cursor_shape::{SHAPE_IDX_TERM, shape_entry};
use crate::src::nvim::drawscreen::redraw_buf_line_later;
use crate::src::nvim::eval::typval::{tv_dict_add_nr, tv_dict_set_keys_readonly};
use crate::src::nvim::eval::vars::get_globvar_dict;
use crate::src::nvim::eval::{get_v_event, restore_v_event};
use crate::src::nvim::event::multiqueue::{multiqueue_free, multiqueue_new, multiqueue_put_event};
use crate::src::nvim::highlight::{
    HL_BG_INDEXED, HL_BLINK, HL_BOLD, HL_CONCEALED, HL_DIM, HL_FG_INDEXED, HL_INVERSE, HL_ITALIC,
    HL_OVERLINE, HL_STRIKETHROUGH, HL_UNDERCURL, HL_UNDERDOUBLE, HL_UNDERLINE, hl_combine_attr,
    hl_get_term_attr,
};
use crate::src::nvim::highlight_group::name_to_color;
use crate::src::nvim::main::{
    State, buffer_handles, curbuf, curtab, curwin, exiting, first_tabpage, firstwin, main_loop,
};
use crate::src::nvim::map::mh_get_int;
use crate::src::nvim::memline::ml_delete_buf;
use crate::src::nvim::memory::xfree;
use crate::src::nvim::r#move::win_col_off;
use crate::src::nvim::option::set_option_value;
use crate::src::nvim::options::kOptBuftype;
use crate::src::nvim::os::libc::{abort, strlen};
use crate::src::nvim::types::builders::{DictBuf, static_cstring};
use crate::src::nvim::types::terminal_defs::SELECTIONBUF_SIZE;
use crate::src::nvim::types::{
    Arena, Buffer, Error, Event, ExtmarkOp, HlAttrs, Map_int_ptr_t, MarkAdjustMode, Object, OptVal,
    OptValData, OptValType, RgbValue, Terminal, TerminalOptions, VTermColor, VTermColor_rgb,
    VTermScreenCell, VTermScreenCellAttrs, VTermState, VTermValue, aco_save_T, buf_T, colnr_T,
    exarg_T, handle_T, int16_t, kErrorTypeNone, kObjectTypeNil, kObjectTypeString, linenr_T, pos_T,
    ptr_t, save_v_event_T, size_t, tabpage_T, uint8_t, varnumber_T, win_T,
};
use crate::src::nvim::vterm::parser::vterm_input_write;
use crate::src::nvim::vterm::pen::{convert_color_to_rgb, set_palette_color};
use crate::src::nvim::vterm::screen::{
    vterm_obtain_screen, vterm_screen_enable_altscreen, vterm_screen_enable_reflow,
    vterm_screen_flush_damage, vterm_screen_reset, vterm_screen_set_callbacks,
    vterm_screen_set_damage_merge, vterm_screen_set_unrecognised_fallbacks,
};
use crate::src::nvim::vterm::state::{
    vterm_obtain_state, vterm_state_set_selection_callbacks, vterm_state_set_termprop,
};
use crate::src::nvim::vterm::vterm::{
    VTERM_COLOR_DEFAULT_BG, VTERM_COLOR_DEFAULT_FG, VTERM_COLOR_INDEXED, VTERM_COLOR_RGB,
    VTERM_COLOR_TYPE_MASK, VTERM_DAMAGE_SCROLL, VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_CURSORSHAPE_BAR_LEFT, VTERM_PROP_CURSORSHAPE_BLOCK,
    VTERM_PROP_CURSORSHAPE_UNDERLINE, vterm_free, vterm_get_size, vterm_new,
    vterm_output_set_callback, vterm_set_size, vterm_set_utf8,
};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

use scrollback::{fetch_cell, refresh_scrollback, term_may_alloc_scrollback};

use crate::src::nvim::state::MODE_TERMINAL;
pub use input::{terminal_paste, terminal_set_streamed_paste};
pub use mode::terminal_enter;
pub use refresh::{
    on_scrollback_option_changed, terminal_check_refresh, terminal_init, terminal_teardown,
};

/// An `Error` that carries no error. The API calls made here cannot fail in
/// a way any caller could act on, so their errors are cleared and dropped.

/// "To the end of the buffer", for the mark adjustments.
const NUL: c_int = 0;
/// `ml_flags` bit meaning the buffer holds one empty line and nothing else.
const ML_EMPTY: c_int = 0x1;
/// The largest `'scrollback'` that means anything; the option's negative
/// "unlimited" spelling becomes this.
const SB_MAX: c_int = 1000000;
/// Marks in trimmed scrollback move the way a terminal's do, not an edit's.
const kMarkAdjustTerm: MarkAdjustMode = 2;
const kExtmarkUndo: ExtmarkOp = 1;
/// `set_option_value` spellings.
const kOptValTypeString: OptValType = 2;
const OPT_LOCAL: c_int = 2;
/// The most columns [`terminal_get_line_attributes`] will resolve;
/// `drawline` sizes its array to match.
const TERM_ATTRS_MAX: c_int = 1024;
/// `State` bit set while terminal mode is running.
/// `redraw_later` levels.
const AUGROUP_ALL: c_int = -3;

/// vterm's cursor shapes, which are DECSCUSR's rather than the editor's.
/// Merge damage reports up to a whole scrolled region before delivering
/// them; the refresh works in row ranges anyway.
/// An escape sequence ended with BEL rather than ST.

/// One entry of an `int -> ptr` map, or null.
///
/// The generated `map.h` accessor, of which this is the only instantiation
/// the terminal needs.
unsafe fn map_get_int_ptr_t(map: *mut Map_int_ptr_t, key: c_int) -> ptr_t {
    unsafe {
        let slot = mh_get_int(&raw mut (*map).set, key);
        // A miss is reported as the tombstone index.
        if slot == u32::MAX {
            // Nothing is stored under a missing key.
            ::core::ptr::null_mut()
        } else {
            *(*map).values.add(slot as usize)
        }
    }
}

/// The buffer a handle names, or null once it has been wiped.
unsafe fn buf_for_handle(handle: handle_T) -> *mut buf_T {
    unsafe { map_get_int_ptr_t(buffer_handles.ptr(), handle) as *mut buf_T }
}

/// Every window in every tabpage.
///
/// The current tabpage keeps its window list in `firstwin` rather than in
/// the tabpage struct, which is why this is not a plain walk of
/// `tp_firstwin`.
unsafe fn all_windows() -> impl Iterator<Item = *mut win_T> {
    let mut tp = first_tabpage.get() as *mut tabpage_T;
    let mut wp: *mut win_T = ::core::ptr::null_mut();
    ::core::iter::from_fn(move || {
        // SAFETY: walking the editor's window lists on the main thread. No
        // caller restructures them while iterating.
        unsafe {
            while wp.is_null() {
                if tp.is_null() {
                    return None;
                }
                wp = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            let found = wp;
            wp = (*found).w_next;
            Some(found)
        }
    })
}

/// vterm's "here are bytes for the child" callback.
unsafe extern "C" fn term_output_callback(s: *const c_char, len: size_t, user_data: *mut c_void) {
    unsafe { terminal_send(user_data as *mut Terminal, s, len) };
}

/// Create a terminal for `buf` and wire it to vterm.
///
/// The buffer is emptied: its lines are about to become a mirror of the
/// emulator's screen, and anything already there would be taken for
/// scrollback.
pub unsafe fn terminal_alloc(buf: *mut buf_T, opts: TerminalOptions) -> *mut Terminal {
    unsafe {
        // Leaked here and reclaimed by terminal_destroy. The buffer is the
        // owner; every other reference reaches it through `buf_T::terminal`.
        let term: *mut Terminal = Box::into_raw(Box::new(Terminal::new(opts, (*buf).handle)));
        (*buf).terminal = term;

        (*term).vt = vterm_new(opts.height as c_int, opts.width as c_int);
        vterm_set_utf8((*term).vt, 1);
        let state: *mut VTermState = vterm_obtain_state((*term).vt);
        (*term).vts = vterm_obtain_screen((*term).vt);
        vterm_screen_enable_altscreen((*term).vts, 1);
        vterm_screen_enable_reflow((*term).vts, true);
        vterm_screen_set_callbacks(
            (*term).vts,
            &raw const callbacks::SCREEN_CALLBACKS,
            term as *mut c_void,
        );
        vterm_screen_set_unrecognised_fallbacks(
            (*term).vts,
            &raw const termrequest::FALLBACKS,
            term as *mut c_void,
        );
        vterm_screen_set_damage_merge((*term).vts, VTERM_DAMAGE_SCROLL);
        vterm_screen_reset((*term).vts, 1);
        vterm_output_set_callback((*term).vt, Some(term_output_callback), term as *mut c_void);
        vterm_state_set_selection_callbacks(
            state,
            &raw const callbacks::SELECTION_CALLBACKS,
            term as *mut c_void,
            (*term).selection_buffer.as_mut_ptr(),
            SELECTIONBUF_SIZE as size_t,
        );

        // Start the child off with the cursor the user configured for
        // terminal mode; it is free to change it.
        let shape = shape_entry(SHAPE_IDX_TERM);
        let mut cursor_shape = VTermValue { boolean: 0 };
        cursor_shape.number = match shape.shape {
            0 => VTERM_PROP_CURSORSHAPE_BLOCK,
            1 => VTERM_PROP_CURSORSHAPE_UNDERLINE,
            2 => VTERM_PROP_CURSORSHAPE_BAR_LEFT,
            // Not a shape vterm knows; leave its default alone.
            _ => 0,
        };
        vterm_state_set_termprop(state, VTERM_PROP_CURSORSHAPE, &raw mut cursor_shape);
        let mut cursor_blink = VTermValue { boolean: 0 };
        cursor_blink.boolean = (shape.blinkon != 0 && shape.blinkoff != 0) as c_int;
        vterm_state_set_termprop(state, VTERM_PROP_CURSORBLINK, &raw mut cursor_blink);

        (*term).invalid_start = 0;
        (*term).invalid_end = opts.height as c_int;
        (*term).pending.events = multiqueue_new(None, ::core::ptr::null_mut());

        if (*buf).b_ml.ml_flags & ML_EMPTY == 0 {
            let line_count = (*buf).b_ml.ml_line_count;
            // Not immutable: ml_delete_buf() mutates (*buf).b_ml behind the raw pointer.
            #[allow(clippy::while_immutable_condition)]
            while (*buf).b_ml.ml_flags & ML_EMPTY == 0 {
                ml_delete_buf(buf, 1 as linenr_T, false);
            }
            deleted_lines_buf(buf, 1 as linenr_T, line_count);
        }
        (*term).old_height = 1;
        term
    }
}

/// Make `buf` look and behave like a terminal buffer, and announce it.
///
/// Runs `TermOpen`, which can wipe the buffer or close the terminal
/// outright — hence the re-check before touching either again.
pub unsafe fn terminal_open(termpp: *mut *mut Terminal, buf: *mut buf_T) {
    unsafe {
        let term: *mut Terminal = *termpp;
        assert!(!term.is_null(), "terminal_open without a terminal");

        let mut aco: aco_save_T = ::core::mem::zeroed();
        aucmd_prepbuf(&raw mut aco, buf);
        if (*term).sb.is_sized() {
            refresh_scrollback(term, buf);
        } else {
            debug_assert!((*term).invalid_start >= 0);
        }
        refresh::refresh_screen(term, buf);

        // Locked because setting 'buftype' can run OptionSet, and the
        // buffer's lines are the emulator's to write.
        (*buf).b_locked += 1;
        set_option_value(
            kOptBuftype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: static_cstring(c"terminal"),
                },
            },
            OPT_LOCAL,
        );
        (*buf).b_locked -= 1;

        if !(*buf).b_ffname.is_null() {
            callbacks::buf_set_term_title(
                buf,
                ::core::slice::from_raw_parts(
                    (*buf).b_ffname.cast::<u8>(),
                    strlen((*buf).b_ffname),
                ),
            );
        }
        // Both would tie the terminal window's scroll position to another
        // window's, which fights the emulator for the topline.
        (*curwin.get()).w_onebuf_opt.wo_scb = 0;
        (*curwin.get()).w_onebuf_opt.wo_crb = 0;
        (*curwin.get()).w_cursor = pos_T {
            lnum: 1 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        };
        apply_autocmds(
            EVENT_TERMOPEN,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            buf,
        );
        aucmd_restbuf(&raw mut aco);
        if (*termpp).is_null() || (*term).buf_handle == 0 {
            return;
        }
        if !term_may_alloc_scrollback(term, buf) {
            abort();
        }

        // `g:terminal_color_0` .. `_15`, or a buffer-local override, set the
        // emulator's palette. Only the ones actually configured are taken:
        // the rest stay at vterm's defaults, and `color_set` records which,
        // so that the attribute code knows to leave them indexed for the UI
        // to resolve.
        let state: *mut VTermState = vterm_obtain_state((*term).vt);
        for i in 0..16 {
            let key = format!("terminal_color_{i}\0");
            let name = get_config_string(buf, key.as_ptr().cast::<c_char>());
            if name.is_null() {
                continue;
            }
            let rgb = name_to_color(CStr::from_ptr(name)).0;
            xfree(name as *mut c_void);
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
            set_palette_color(&mut *state, i, &color);
            (*term).color_set[i as usize] = true;
        }
    }
}

/// The child exited, or the terminal is being torn down.
///
/// `status` is the child's exit status, or -1 when there was no child to
/// wait for. Runs `TermClose` unless autocommands are blocked; the buffer
/// stays, showing whatever the child left on screen, until something wipes
/// it.
pub unsafe fn terminal_close(termpp: *mut *mut Terminal, status: c_int) {
    unsafe {
        let term: *mut Terminal = *termpp;
        if (*term).destroy {
            return;
        }
        let buf = buf_for_handle((*term).buf_handle);

        // Closing an already-closed terminal leaves only the freeing half.
        let only_destroy = (*term).closed;
        if !only_destroy {
            if !exiting.get() {
                // Show the child's last output before announcing its death.
                block_autocmds();
                refresh::refresh_terminal(term);
                unblock_autocmds();
            }
            (*term).closed = true;
        }

        // Where the child left off, reported to `TermClose` as
        // `v:event`-adjacent data.
        let mut pos = if buf.is_null() {
            0
        } else {
            (*buf).b_ml.ml_line_count as c_int - 1
        };
        if status == -1 || exiting.get() {
            // Nothing to report on: detach from the buffer straight away.
            (*term).buf_handle = 0 as handle_T;
            if !buf.is_null() {
                (*buf).terminal = ::core::ptr::null_mut();
            }
            if (*term).refcount == 0 {
                (*term).destroy = true;
                (*term).opts.close_cb.expect("non-null function pointer")((*term).opts.data);
            }
        } else if !only_destroy {
            // The status line says "running"; it no longer is.
            let mut wp = firstwin.get();
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    (*wp).w_redr_status = true;
                }
                wp = (*wp).w_next;
            }
            pos = pos.min(row_to_linenr(term, (*term).cursor.row));
        }

        if only_destroy || buf.is_null() || is_autocmd_blocked() {
            return;
        }
        let mut save_v_event: save_v_event_T = ::core::mem::zeroed();
        let dict = get_v_event(&raw mut save_v_event);
        tv_dict_add_nr(dict, c"status".as_ptr(), 6, status as varnumber_T);
        tv_dict_set_keys_readonly(dict);
        let mut data = DictBuf::<1>::new();
        data.insert(c"pos", Object::integer(pos as i64));
        apply_autocmds_group(
            EVENT_TERMCLOSE,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            status >= 0,
            AUGROUP_ALL,
            buf,
            ::core::ptr::null_mut::<exarg_T>(),
            &mut data.object(),
        );
        restore_v_event(dict, &raw mut save_v_event);
    }
}

/// Redraw the last line of a terminal's buffer, where the "running" /
/// "suspended" marker is drawn.
unsafe extern "C" fn terminal_state_change_event(argv: *mut *mut c_void) {
    unsafe {
        let buf = buf_for_handle((*argv.offset(0)).expose_provenance() as handle_T);
        if !buf.is_null() && !(*buf).terminal.is_null() {
            redraw_buf_line_later(buf, (*buf).b_ml.ml_line_count, false);
        }
    }
}

/// Record that the child was stopped or resumed.
///
/// The redraw is deferred: this is reached from a process-status callback,
/// which is no place to touch the screen.
pub unsafe fn terminal_set_state(term: *mut Terminal, suspended: bool) {
    unsafe {
        if (*term).suspended != suspended {
            multiqueue_put_event(
                refresh::refresh_queue(),
                Event::new(
                    Some(terminal_state_change_event),
                    [::core::ptr::with_exposed_provenance_mut::<c_void>(
                        (*term).buf_handle as usize,
                    )],
                ),
            );
        }
        (*term).suspended = suspended;
    }
}

/// Resize the emulator to fit the windows showing it.
///
/// The largest of them, not the smallest: a narrower window scrolls
/// sideways rather than making the child reflow for everyone else.
pub unsafe fn terminal_check_size(term: *mut Terminal) {
    unsafe {
        if (*term).closed {
            return;
        }
        let mut curheight = 0;
        let mut curwidth = 0;
        vterm_get_size((*term).vt, &raw mut curheight, &raw mut curwidth);

        let mut width = 0;
        let mut height = 0;
        for wp in all_windows() {
            // The autocommand window is a fiction with a nominal size.
            if is_aucmd_win(wp) || (*wp).w_buffer.is_null() {
                continue;
            }
            if (*(*wp).w_buffer).terminal != term {
                continue;
            }
            width = width.max(((*wp).w_view_width - win_col_off(wp)).max(0));
            height = height.max((*wp).w_view_height);
        }

        // Zero means no window is showing it; keep whatever size it had.
        if (curheight == height && curwidth == width) || height == 0 || width == 0 {
            return;
        }
        vterm_set_size((*term).vt, height, width);
        vterm_screen_flush_damage((*term).vts);
        (*term).pending.resize = true;
        refresh::invalidate_terminal(term, None);
    }
}

/// Free the terminal, if nothing is still standing on it.
///
/// Reached repeatedly — from the close callback, from the buffer being
/// wiped — and does nothing until `refcount` reaches zero.
pub unsafe fn terminal_destroy(termpp: *mut *mut Terminal) {
    unsafe {
        let term: *mut Terminal = *termpp;
        let buf = buf_for_handle((*term).buf_handle);
        if !buf.is_null() {
            (*term).buf_handle = 0 as handle_T;
            (*buf).terminal = ::core::ptr::null_mut();
        }
        if (*term).refcount != 0 {
            return;
        }
        refresh::refresh_before_destroy(term);
        vterm_free((*term).vt);
        multiqueue_free((*term).pending.events);
        // The other half of terminal_alloc's `Box::into_raw`; everything
        // the terminal owns goes with it.
        drop(Box::from_raw(term));
        *termpp = ::core::ptr::null_mut();
    }
}

/// Write `data` to the child, or hold it if a `TermRequest` handler is
/// running.
///
/// See [`TerminalPending::send`](crate::src::nvim::types::TerminalPending).
unsafe fn terminal_send(term: *mut Terminal, data: *const c_char, size: size_t) {
    unsafe {
        if (*term).closed {
            return;
        }
        if let Some(held) = (*term).pending.send.as_mut() {
            if size > 0 {
                held.extend_from_slice(::core::slice::from_raw_parts(data.cast::<u8>(), size));
            }
            return;
        }
        (*term).opts.write_cb.expect("non-null function pointer")(data, size, (*term).opts.data);
    }
}

/// Redraw after the child closed a synchronized-output frame.
unsafe extern "C" fn on_sync_flush(argv: *mut *mut c_void) {
    unsafe {
        if exiting.get() {
            return;
        }
        let buf = buf_for_handle((*argv.offset(0)).expose_provenance() as handle_T);
        if buf.is_null() || (*buf).terminal.is_null() {
            return;
        }
        block_autocmds();
        refresh::refresh_terminal((*buf).terminal);
        unblock_autocmds();
    }
}

/// Feed `len` bytes of the child's output to the emulator.
///
/// `force_crlf` is for channels that are not a pty: a bare newline from
/// those means "next line, column zero", which to a terminal is CR LF.
pub unsafe fn terminal_receive(term: *mut Terminal, data: *const c_char, len: size_t) {
    unsafe {
        if data.is_null() {
            return;
        }
        if (*term).opts.force_crlf {
            let bytes = ::core::slice::from_raw_parts(data.cast::<u8>(), len);
            let mut crlf = Vec::with_capacity(len);
            for (i, &byte) in bytes.iter().enumerate() {
                if byte == b'\n' && (i == 0 || bytes[i - 1] != b'\r') {
                    crlf.push(b'\r');
                }
                crlf.push(byte);
            }
            vterm_input_write((*term).vt, crlf.as_ptr().cast::<c_char>(), crlf.len());
        } else {
            vterm_input_write((*term).vt, data, len);
        }
        vterm_screen_flush_damage((*term).vts);

        if (*term).sync_flush_pending {
            // The frame the child was assembling is complete: invalidate
            // all of it and redraw once, from the main loop rather than
            // from the middle of parsing its output.
            (*term).sync_flush_pending = false;
            let mut height = 0;
            vterm_get_size((*term).vt, &raw mut height, ::core::ptr::null_mut());
            (*term).invalid_start = 0;
            (*term).invalid_end = height;
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event::new(
                    Some(on_sync_flush),
                    [::core::ptr::with_exposed_provenance_mut::<c_void>(
                        (*term).buf_handle as usize,
                    )],
                ),
            );
        }
    }
}

/// A vterm colour as a packed 24-bit RGB value, resolving the palette.
unsafe fn get_rgb(state: *mut VTermState, mut color: VTermColor) -> c_int {
    unsafe {
        convert_color_to_rgb(&*state, &mut color);
        ((color.rgb.red as c_int) << 16)
            | ((color.rgb.green as c_int) << 8)
            | color.rgb.blue as c_int
    }
}

/// The highlight bit for a cell's underline style. vterm names more styles
/// than the editor draws, so anything else becomes a plain underline.
fn get_underline_hl_flag(attrs: VTermScreenCellAttrs) -> c_int {
    match attrs.underline() {
        0 => 0,
        2 => HL_UNDERDOUBLE,
        3 => HL_UNDERCURL,
        _ => HL_UNDERLINE,
    }
}

/// Resolve one buffer line of a terminal into per-column highlight ids.
///
/// The screen painter calls this for every visible line of a terminal
/// buffer; `term_attrs` is its scratch array, `TERM_ATTRS_MAX` wide. Lines
/// that are scrollback rather than screen resolve through the scrollback,
/// and lines below the screen are left alone.
pub unsafe fn terminal_get_line_attributes(
    term: *mut Terminal,
    _wp: *mut win_T,
    linenr: c_int,
    term_attrs: *mut c_int,
) {
    unsafe {
        let mut height = 0;
        let mut width = 0;
        vterm_get_size((*term).vt, &raw mut height, &raw mut width);
        let state: *mut VTermState = vterm_obtain_state((*term).vt);
        debug_assert!(linenr != 0, "buffer line numbers are one-based");

        let row = linenr_to_row(term, linenr);
        if row >= height {
            return;
        }
        let width = width.min(TERM_ATTRS_MAX);

        for col in 0..width {
            let mut cell: VTermScreenCell = ::core::mem::zeroed();
            // False for a scrollback cell past the end of a row stored while
            // the terminal was narrower; such a cell has no colours at all.
            let color_valid = fetch_cell(term, row, col, &raw mut cell);
            let fg_default =
                !color_valid || c_uint::from(cell.fg.type_0) & VTERM_COLOR_DEFAULT_FG != 0;
            let bg_default =
                !color_valid || c_uint::from(cell.bg.type_0) & VTERM_COLOR_DEFAULT_BG != 0;
            let fg_indexed =
                c_uint::from(cell.fg.type_0) & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED;
            let bg_indexed =
                c_uint::from(cell.bg.type_0) & VTERM_COLOR_TYPE_MASK == VTERM_COLOR_INDEXED;

            // The cterm colour is one-based so that zero can mean "unset".
            let vt_fg_idx: int16_t = if !fg_default && fg_indexed {
                cell.fg.indexed.idx as int16_t + 1
            } else {
                0
            };
            let vt_bg_idx: int16_t = if !bg_default && bg_indexed {
                cell.bg.indexed.idx as int16_t + 1
            } else {
                0
            };
            // A palette entry the user configured resolves to a real RGB
            // value here; one they did not stays indexed, so that the UI's
            // own palette applies instead.
            let fg_set =
                vt_fg_idx != 0 && vt_fg_idx <= 16 && (*term).color_set[(vt_fg_idx - 1) as usize];
            let bg_set =
                vt_bg_idx != 0 && vt_bg_idx <= 16 && (*term).color_set[(vt_bg_idx - 1) as usize];

            let attrs = cell.attrs;
            let mut hl_attrs = get_underline_hl_flag(attrs);
            for (set, bit) in [
                (attrs.bold() != 0, HL_BOLD),
                (attrs.dim() != 0, HL_DIM),
                (attrs.blink() != 0, HL_BLINK),
                (attrs.conceal() != 0, HL_CONCEALED),
                (attrs.overline() != 0, HL_OVERLINE),
                (attrs.italic() != 0, HL_ITALIC),
                (attrs.reverse() != 0, HL_INVERSE),
                (attrs.strike() != 0, HL_STRIKETHROUGH),
                (fg_indexed && !fg_set, HL_FG_INDEXED),
                (bg_indexed && !bg_set, HL_BG_INDEXED),
            ] {
                if set {
                    hl_attrs |= bit as c_int;
                }
            }

            let mut attr_id = 0;
            if hl_attrs != 0 || !fg_default || !bg_default {
                let mut resolved = HlAttrs {
                    rgb_ae_attr: hl_attrs,
                    cterm_ae_attr: hl_attrs,
                    rgb_fg_color: if fg_default {
                        -1
                    } else {
                        get_rgb(state, cell.fg)
                    },
                    rgb_bg_color: if bg_default {
                        -1
                    } else {
                        get_rgb(state, cell.bg)
                    },
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
            *term_attrs.add(col as usize) = attr_id;
        }
    }
}

pub unsafe fn terminal_buf(term: *const Terminal) -> Buffer {
    unsafe { (*term).buf_handle as Buffer }
}

pub unsafe fn terminal_running(term: *const Terminal) -> bool {
    unsafe { !(*term).closed }
}

pub unsafe fn terminal_suspended(term: *const Terminal) -> bool {
    unsafe { (*term).suspended }
}

/// Tell a child that asked for theme updates that `'background'` changed.
pub unsafe fn terminal_notify_theme(term: *mut Terminal, dark: bool) {
    unsafe {
        if !(*term).theme_updates {
            return;
        }
        let report: &[u8] = if dark { b"\x1b[997;1n" } else { b"\x1b[997;2n" };
        terminal_send(term, report.as_ptr().cast::<c_char>(), report.len());
    }
}

/// The buffer line an emulator row appears on, counting the scrollback
/// above it. The "nothing invalid" sentinel passes through unchanged.
unsafe fn row_to_linenr(term: *mut Terminal, row: c_int) -> c_int {
    unsafe {
        if row == c_int::MAX {
            c_int::MAX
        } else {
            row + (*term).sb.len() as c_int + 1
        }
    }
}

/// The inverse of [`row_to_linenr`]. Negative for a scrollback line.
unsafe fn linenr_to_row(term: *mut Terminal, linenr: c_int) -> c_int {
    unsafe { linenr - (*term).sb.len() as c_int - 1 }
}

/// Whether the user is typing at this terminal right now.
unsafe fn is_focused(term: *mut Terminal) -> bool {
    unsafe { State.get() & MODE_TERMINAL != 0 && (*curbuf.get()).terminal == term }
}

/// `b:<key>`, falling back to `g:<key>`, if it is a string.
///
/// The result is an owned C string the caller frees, or null.
unsafe fn get_config_string(buf: *mut buf_T, key: *const c_char) -> *mut c_char {
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut(),
        };
        let mut obj = dict_get_value(
            (*buf).b_vars,
            cstr_as_string(key),
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        if obj.type_0 == kObjectTypeNil {
            obj = dict_get_value(
                get_globvar_dict(),
                cstr_as_string(key),
                ::core::ptr::null_mut::<Arena>(),
                &raw mut err,
            );
            api_clear_error(&raw mut err);
        }
        if obj.type_0 == kObjectTypeString {
            // The string outlives the object; the caller owns it now.
            return obj.data.string.data;
        }
        api_free_object(obj);
        ::core::ptr::null_mut()
    }
}
