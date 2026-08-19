//! The UI events that are not painting.
//!
//! Everything the editor can tell the TUI that does not change what is on
//! the screen: whether it is busy, what the cursor should look like, whether
//! the mouse is being watched, the window title, and the handful of options
//! the TUI itself acts on.
//!
//! These are the sinks the UI client dispatches to; the painting ones live
//! in [`paint`](super::paint).

#![deny(unsafe_op_in_unsafe_fn)]

use crate::event::libuv::{uv_chdir, uv_run, uv_sleep, uv_strerror, uv_tty_set_mode, uv_write};
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::main::{stdin_isatty, ui_client_channel_id};
use crate::memory::{strequal, xfree};
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::tui::cursor::{
    cursor_style_enabled, decode_cursor_entry, reset_style as cursor_reset_style,
    set_mode as cursor_set_mode,
};
use crate::tui::negotiate::{
    MOUSE_ANY_EVENT, MOUSE_BUTTON_EVENT, MOUSE_SGR_EXT, tui_set_term_mode,
};
use crate::tui::output::{TERMINFO_SEQ_LIMIT, flush, out, out_raw, terminfo_out};
use crate::tui::paint::invalidate;
use crate::tui::terminfo::caps::{kTerm_from_status_line, kTerm_to_status_line};
use crate::tui::terminfo::terminfo_info_msg;
use crate::types::{
    Array, ArrayBuf, DictBuf, Integer, Object, ObjectType, String_0, TUIData, uv_buf_t,
    uv_stream_t, uv_tty_mode_t, uv_write_t,
};

const UV_TTY_MODE_NORMAL: uv_tty_mode_t = 0;
const UV_TTY_MODE_IO: uv_tty_mode_t = 2;
const UV_RUN_DEFAULT: core::ffi::c_uint = 0;

/// The longest title worth sending. Terminals truncate long ones anyway, and
/// a title that does not fit the staging buffer would be cut mid-sequence.
const MAX_TITLE: usize = 4096;

/// The menu the TUI does not draw.
pub fn tui_update_menu(_tui: &mut TUIData) {}

/// The editor is busy: hide the cursor until it says otherwise.
pub fn tui_busy_start(tui: &mut TUIData) {
    tui.busy = true;
}

/// The editor is idle again.
pub fn tui_busy_stop(tui: &mut TUIData) {
    tui.busy = false;
}

/// Start reporting mouse events.
pub fn tui_mouse_on(tui: &mut TUIData) {
    if tui.mouse_enabled {
        return;
    }
    tui_set_term_mode(tui, MOUSE_BUTTON_EVENT, true);
    // SGR coordinates, so columns past 223 are reportable at all.
    tui_set_term_mode(tui, MOUSE_SGR_EXT, true);
    if tui.mouse_move_enabled {
        tui_set_term_mode(tui, MOUSE_ANY_EVENT, true);
    }
    tui.mouse_enabled = true;
}

/// Stop reporting mouse events, in the reverse order they were turned on.
pub fn tui_mouse_off(tui: &mut TUIData) {
    if !tui.mouse_enabled {
        return;
    }
    if tui.mouse_move_enabled {
        tui_set_term_mode(tui, MOUSE_ANY_EVENT, false);
    }
    tui_set_term_mode(tui, MOUSE_BUTTON_EVENT, false);
    tui_set_term_mode(tui, MOUSE_SGR_EXT, false);
    tui.mouse_enabled = false;
}

/// The editor's `'guicursor'` was (re)parsed: take the cursor description
/// for every mode.
///
/// # Safety
/// `args` must be an array of mode dictionaries.
pub unsafe fn tui_mode_info_set(tui: &mut TUIData, guicursor_enabled: bool, args: Array) {
    cursor_style_enabled.set(guicursor_enabled);
    if !guicursor_enabled {
        cursor_reset_style(tui);
        return;
    }
    assert!(args.size != 0, "mode_info_set with no modes");
    // SAFETY: the caller guarantees `args`.
    unsafe {
        for i in 0..args.size {
            let item = &*args.items.add(i);
            assert!(
                item.type_0 == OBJECT_TYPE_DICT,
                "mode_info_set entry is not a dict"
            );
            tui.cursor_shapes[i] = decode_cursor_entry(item.data.dict);
        }
    }
    let showing = tui.showing_mode;
    cursor_set_mode(tui, showing);
}

/// The `Object` type tag for a dictionary.
const OBJECT_TYPE_DICT: ObjectType = 6;

/// The editor changed mode: dress the cursor for it.
///
/// This is also where startup ends, because it is the first event that
/// arrives after the editor has finished reading its configuration.
///
pub fn tui_mode_change(tui: &mut TUIData, _mode: String_0, mode_idx: Integer) {
    // Reading from a pipe while writing to a tty leaves libuv's idea of the
    // terminal mode stale; setting it twice re-applies it.
    if tui.out_isatty && tui.is_starting && !stdin_isatty.get() {
        for mode in [UV_TTY_MODE_NORMAL, UV_TTY_MODE_IO] {
            // SAFETY: the handle is the TUI's own and libuv owns its state.
            let ret = unsafe { uv_tty_set_mode(&raw mut tui.output_handle.tty, mode) };
            if ret != 0 {
                // SAFETY: the format string holds the one `%s` filled here.
                unsafe {
                    logmsg_c!(
                        LOGLVL_ERR,
                        core::ptr::null(),
                        c"tui_mode_change".as_ptr(),
                        0,
                        true,
                        c"uv_tty_set_mode failed: %s".as_ptr(),
                        uv_strerror(ret),
                    )
                };
            }
        }
    }
    cursor_set_mode(tui, mode_idx as usize);
    if tui.is_starting && tui.verbose >= 3 {
        show_verbose_terminfo(tui);
    }
    tui.is_starting = false;
    tui.showing_mode = mode_idx as usize;
}

/// Ring the terminal's bell.
///
pub fn tui_bell(tui: &mut TUIData) {
    out(tui, b"\x07");
}

/// Flash the screen instead of ringing.
///
pub fn tui_visual_bell(tui: &mut TUIData) {
    if tui.screen_or_tmux {
        // screen and tmux have a visual bell of their own; reverse video
        // would be applied to their own status line as well.
        out(tui, b"\x1bg");
    } else {
        // Reverse the screen, hold it long enough to be seen, undo it.
        out(tui, b"\x1b[?5h");
        flush(tui);
        // SAFETY: libuv's sleep takes no state of ours.
        unsafe { uv_sleep(VISUAL_BELL_MS) };
        out(tui, b"\x1b[?5l");
    }
    flush(tui);
}

/// How long the screen stays reversed for a visual bell.
const VISUAL_BELL_MS: core::ffi::c_uint = 100;

/// Send `content` to the terminal verbatim, outside the staging buffer.
///
/// This is `ui_send`: whatever the editor wrote goes out as it stands,
/// synchronously, without the TUI interpreting any of it.
///
/// # Safety
/// `content` must be a valid API string.
pub unsafe fn tui_ui_send(tui: &mut TUIData, content: String_0) {
    // SAFETY: the caller guarantees `content`; `uv_write` fills in the
    // request, which lives until the loop below has run it.
    unsafe {
        let mut req: uv_write_t = core::mem::zeroed();
        let buf = uv_buf_t {
            base: content.data(),
            len: content.len(),
        };
        let ret = uv_write(
            &raw mut req,
            (&raw mut tui.output_handle).cast::<uv_stream_t>(),
            &raw const buf,
            1,
            None,
        );
        if ret != 0 {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"tui_ui_send".as_ptr(),
                0,
                true,
                c"uv_write failed: %s".as_ptr(),
                uv_strerror(ret),
            );
        }
        uv_run(&raw mut tui.write_loop, UV_RUN_DEFAULT);
    }
}

/// Set the terminal's window title, or put back the one it had.
///
/// The title is saved on the terminal's own stack the first time one is set,
/// and restored when the editor asks for an empty one — which is what stops
/// a terminal being left titled `nvim` after the editor exits.
///
/// # Safety
/// `title` must be a valid API string.
pub unsafe fn tui_set_title(tui: &mut TUIData, title: String_0) {
    if !tui.can_set_title {
        return;
    }
    let too_long = title.len() > MAX_TITLE;
    if too_long {
        // SAFETY: the format string takes no arguments.
        unsafe {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"tui_set_title".as_ptr(),
                0,
                true,
                c"set_title: title string too long!".as_ptr(),
            )
        };
    }
    if title.len() > 0 && !too_long {
        if !tui.title_enabled {
            out(tui, b"\x1b[22;0t");
            tui.title_enabled = true;
        }
        // The title and its brackets have to reach the terminal in one
        // piece, so make room for all of it before starting.
        if tui.staging.room() < title.len() + 2 * TERMINFO_SEQ_LIMIT {
            flush(tui);
        }
        terminfo_out(tui, kTerm_to_status_line);
        // SAFETY: the caller guarantees `title`.
        unsafe { out_raw(tui, title.data(), title.len()) };
        terminfo_out(tui, kTerm_from_status_line);
    } else if tui.title_enabled {
        out(tui, b"\x1b[23;0t");
        tui.title_enabled = false;
    }
}

/// The icon name, which no terminal this runs on distinguishes from the
/// title.
///
pub fn tui_set_icon(_tui: &mut TUIData, _icon: String_0) {}

/// An option the TUI acts on changed.
///
/// # Safety
/// `name` must be a valid API string and `value` must hold the type that
/// option's name implies.
pub unsafe fn tui_option_set(tui: &mut TUIData, name: String_0, value: Object) {
    // SAFETY: the caller guarantees `name` and `value`'s type.
    let is = |option: &core::ffi::CStr| unsafe { strequal(name.data(), option.as_ptr()) };
    let boolean = || unsafe { value.data.boolean };
    let integer = || unsafe { value.data.integer };

    if is(c"mousemoveevent") {
        let wanted = boolean();
        if tui.mouse_move_enabled != wanted {
            // The mode is part of what mouse reporting turns on, so it can
            // only change while reporting is off.
            if tui.mouse_enabled {
                tui_mouse_off(tui);
                tui.mouse_move_enabled = wanted;
                tui_mouse_on(tui);
            } else {
                tui.mouse_move_enabled = wanted;
            }
        }
    } else if is(c"termguicolors") {
        tui.rgb = boolean();
        tui.print_attr_id = -1;
        let (height, width) = (tui.grid.height, tui.grid.width);
        invalidate(tui, 0, height, 0, width);
        // The editor decides what colours to send by what the UI says it
        // wants, so tell it this changed.
        if ui_client_channel_id.get() != 0 {
            let mut args = ArrayBuf::<2>::new();
            args.push(Object::literal("rgb"));
            args.push(Object::boolean(boolean()));
            // SAFETY: the event borrows `args`, which outlives the call.
            unsafe {
                rpc_send_event(
                    ui_client_channel_id.get(),
                    c"nvim_ui_set_option".as_ptr(),
                    args.array(),
                )
            };
        }
    } else if is(c"ttimeout") {
        tui.input.ttimeout = boolean();
    } else if is(c"ttimeoutlen") {
        tui.input.ttimeoutlen = integer();
    } else if is(c"verbose") {
        tui.verbose = integer();
    } else if is(c"termsync") {
        tui.sync_output = boolean();
    }
}

/// Follow the editor into its new working directory, so that relative paths
/// in what the terminal is told mean the same thing.
///
/// # Safety
/// `path` must be a valid API string.
pub unsafe fn tui_chdir(_tui: &mut TUIData, path: String_0) {
    // SAFETY: the caller guarantees `path`.
    unsafe {
        let err = uv_chdir(path.data());
        if err != 0 {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"tui_chdir".as_ptr(),
                0,
                true,
                c"Failed to chdir to %s: %s".as_ptr(),
                path.data(),
                uv_strerror(err),
            );
        }
    }
}

/// Echo what was resolved about this terminal, for `nvim -V3`.
///
/// `tui`'s terminfo must be resolved.
fn show_verbose_terminfo(tui: &TUIData) {
    // SAFETY: the message is owned here and freed once the event has been
    // serialised.
    let info = unsafe { terminfo_info_msg(&tui.ti, tui.term, tui.terminfo_found_in_db) };

    // Each chunk is [text] or [text, highlight group], as `nvim_echo` takes.
    let mut title = ArrayBuf::<2>::new();
    title.push(Object::literal("\n\n--- Terminal info --- {{{\n"));
    title.push(Object::literal("Title"));
    let mut body = ArrayBuf::<1>::new();
    body.push(Object::string(info));
    let mut end_fold = ArrayBuf::<2>::new();
    end_fold.push(Object::literal("}}}\n"));
    end_fold.push(Object::literal("Title"));
    let mut chunks = ArrayBuf::<3>::new();
    chunks.push(title.object());
    chunks.push(body.object());
    chunks.push(end_fold.object());

    let mut opts = DictBuf::<1>::new();
    opts.insert(c"verbose", Object::boolean(true));
    let mut args = ArrayBuf::<3>::new();
    args.push(chunks.object());
    args.push(Object::boolean(true));
    args.push(opts.object());

    // SAFETY: the event borrows the buffers above, which outlive the call.
    unsafe {
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_echo".as_ptr(),
            args.array(),
        );
        xfree(info.data().cast());
    }
}
