//! The terminal user interface.
//!
//! This is the UI process's driver: it owns the connection to the terminal,
//! the terminfo entry describing what that terminal can do, and the state
//! needed to decide what to send it. The editor talks to it in UI events —
//! grid updates, mode changes, option changes — which arrive as the `tui_*`
//! functions the UI client dispatches to.
//!
//! What lives where: painting and the grid are in [`paint`](super::paint),
//! attribute state in [`attrs`](super::attrs), the write path in
//! [`output`](super::output), cursor shape in [`cursor`](super::cursor),
//! terminal quirks in [`quirks`](super::quirks), and input in
//! [`input`](super::input). This module keeps the shared state and the
//! lifecycle: starting the terminal up, negotiating what it supports,
//! and putting it back the way it was found.
//!
//! Almost everything here takes a raw `*mut TUIData`. That is deliberate:
//! the lifecycle pumps the event loop, which runs callbacks that reach the
//! same structure through the pointers libuv and the input layer hold, so a
//! long-lived `&mut` would be a lie. The painting side, which never pumps,
//! uses references.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::event::libuv::{
    uv_close, uv_is_closing, uv_loop_close, uv_loop_init, uv_pipe_init, uv_pipe_open, uv_run,
    uv_strerror, uv_timer_init, uv_timer_start, uv_tty_reset_mode,
};
use crate::src::nvim::event::r#loop::{loop_poll_events, process_events_until};
use crate::src::nvim::event::signal::{
    signal_watcher_close, signal_watcher_init, signal_watcher_start, signal_watcher_stop,
};
use crate::src::nvim::event::stream::stream_set_blocking;
use crate::src::nvim::log::{LOGLVL_ERR, LOGLVL_WRN, logmsg_c};
use crate::src::nvim::main::{main_loop, t_colors, ui_client_error_exit, ui_client_exit_status};
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, arena_strdup, xfree};
use crate::src::nvim::os::env::{os_getenv, os_getenv_noalloc};
use crate::src::nvim::os::input::os_isatty;
use crate::src::nvim::os::libc::{abort, kill, sscanf};
use crate::src::nvim::tui::events::{tui_mode_change, tui_mouse_off, tui_mouse_on, tui_set_title};
use crate::src::nvim::tui::input::{tinput_destroy, tinput_init, tinput_start, tinput_stop};
use crate::src::nvim::tui::negotiate::{
    BRACKETED_PASTE, GRAPHEME_CLUSTERS, LEFT_AND_RIGHT_MARGINS, RESIZE_EVENTS, SYNCHRONIZED_OUTPUT,
    THEME_UPDATES, tui_enable_extended_underline, tui_query_bg_color_noflush,
    tui_query_extended_underline, tui_query_kitty_keyboard, tui_request_term_mode,
    tui_reset_key_encoding, tui_set_term_mode, tui_tk_ti_getstr,
};
use crate::src::nvim::tui::output::{flush, out, out_cstr, terminfo_out};
use crate::src::nvim::tui::paint::cursor_goto;
use crate::src::nvim::tui::quirks::{Terminal, augment_terminfo, patch_terminfo_bugs};
use crate::src::nvim::tui::terminfo::caps::{
    TerminfoDef, kTerm_change_scroll_region, kTerm_clear_screen, kTerm_cursor_normal,
    kTerm_delete_line, kTerm_enter_ca_mode, kTerm_erase_chars, kTerm_exit_attribute_mode,
    kTerm_exit_ca_mode, kTerm_insert_line, kTerm_keypad_local, kTerm_keypad_xmit,
    kTerm_parm_delete_line, kTerm_parm_insert_line, kTerm_reset_cursor_color,
    kTerm_reset_cursor_style, kTerm_set_lr_margin, kTerm_set_underline_style,
};
use crate::src::nvim::tui::terminfo::{terminfo_from_builtin, terminfo_from_database};
use crate::src::nvim::types::libc::STDOUT_FILENO;
use crate::src::nvim::types::{
    HlAttrs, SignalWatcher, Staging, String_0, TUIData, TerminfoExt, uv_file, uv_handle_t,
    uv_loop_t, uv_timer_t, uv_tty_mode_t, uv_tty_t,
};
use crate::src::nvim::ui_client::{ui_client_attach, ui_client_detach, ui_client_set_size};
use core::ffi::{CStr, c_char, c_int, c_void};

unsafe extern "C" {
    fn uv_tty_init(_: *mut uv_loop_t, _: *mut uv_tty_t, fd: uv_file, readable: c_int) -> c_int;
    fn uv_tty_set_mode(_: *mut uv_tty_t, mode: uv_tty_mode_t) -> c_int;
    fn uv_tty_get_winsize(_: *mut uv_tty_t, width: *mut c_int, height: *mut c_int) -> c_int;
}

// --------------------------------------------------------------- the state

// ----------------------------------------------------------- the constants

const UV_RUN_DEFAULT: core::ffi::c_uint = 0;
const UV_EINTR: c_int = -4;
const UV_TTY_MODE_IO: uv_tty_mode_t = 2;

const SIGSTOP: c_int = 19;
const SIGWINCH: c_int = 28;

/// The size to fall back to when neither the terminal, the environment nor
/// terminfo says anything useful.
const DFLT_COLS: c_int = 80;
const DFLT_ROWS: c_int = 24;

/// The first entry in the highlight table: the editor's default highlight.
pub(crate) const DEFAULT_ATTRS: HlAttrs = HlAttrs {
    rgb_ae_attr: 0,
    cterm_ae_attr: 0,
    rgb_fg_color: -1,
    rgb_bg_color: -1,
    rgb_sp_color: -1,
    cterm_fg_color: 0,
    cterm_bg_color: 0,
    hl_blend: -1,
    url: -1,
};

// ------------------------------------------------------------- the lifecycle

/// Take over the terminal, returning the TUI that now owns it.
///
/// The queries whose answers the client needs — size, name, colour depth —
/// have been sent but not answered when this returns; [`tui_wait_ready`]
/// collects them.
///
/// # Safety
/// Must be called once, on the main thread.
pub unsafe fn tui_start() -> *mut TUIData {
    // SAFETY: called once, on the main thread, before anything is drawn.
    unsafe {
        let tui: *mut TUIData = Box::into_raw(TUIData::new());
        signal_watcher_init((*tui).loop_0, &raw mut (*tui).winch_handle, tui.cast());
        signal_watcher_start(&raw mut (*tui).winch_handle, Some(sigwinch_cb), SIGWINCH);
        (*tui).input.tk_ti_hook_fn = Some(tui_tk_ti_getstr);
        tui_terminal_start(tui);

        uv_timer_init(
            &raw mut (*(*tui).loop_0).uv,
            &raw mut (*tui).startup_delay_timer,
        );
        (*tui).startup_delay_timer.data = tui.cast();
        uv_timer_start(
            &raw mut (*tui).startup_delay_timer,
            Some(after_startup_cb),
            STARTUP_DELAY_MS,
            0,
        );

        tui
    }
}

/// Wait for the terminal to answer what [`tui_start`] asked it.
///
/// One turn of the event loop, so that what the client is told is the
/// terminal's answer rather than the guess. It is separate from
/// [`tui_start`] because that turn can run any callback the loop has
/// pending — including the one that stops this TUI when its channel is
/// already at EOF — so the caller has to have published the TUI first.
///
/// # Safety
/// `tui` must be the result of [`tui_start`].
pub unsafe fn tui_wait_ready(tui: *mut TUIData) -> TuiStart {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        loop_poll_events(main_loop.ptr(), 1);
        TuiStart {
            width: (*tui).width,
            height: (*tui).height,
            term: (*tui).term,
            rgb: (*tui).rgb,
        }
    }
}

/// What a started TUI tells its client about the terminal it found.
pub struct TuiStart {
    pub width: c_int,
    pub height: c_int,
    /// The `TERM` the TUI resolved, owned by the [`TUIData`].
    pub term: *mut c_char,
    /// Whether the terminal was found to do 24-bit colour.
    pub rgb: bool,
}

/// How long to wait before sending the sequences that must not race the
/// terminal's own startup.
const STARTUP_DELAY_MS: u64 = 100;

impl TUIData {
    /// A TUI that has not been started yet: no terminal, no handles, and
    /// nothing on the screen.
    ///
    /// What is not zero here is what a zeroed C struct got wrong: the
    /// editor's own highlight has to exist before anything can be painted
    /// under it, and no hyperlink is -1 rather than 0.
    fn new() -> Box<Self> {
        // SAFETY: the fields left zeroed are C layouts whose owners --
        // libuv, the terminfo reader, the grid -- fill them in before
        // reading them, exactly as they did when this struct was xcalloc'ed.
        unsafe {
            Box::new(Self {
                loop_0: main_loop.ptr(),
                staging: Staging::new(),
                input: core::mem::zeroed(),
                write_loop: core::mem::zeroed(),
                ti: core::mem::zeroed(),
                term: core::ptr::null_mut(),
                output_handle: core::mem::zeroed(),
                out_isatty: false,
                winch_handle: core::mem::zeroed(),
                startup_delay_timer: core::mem::zeroed(),
                grid: core::mem::zeroed(),
                invalid_regions: Vec::new(),
                row: 0,
                col: 0,
                out_fd: 0,
                pending_resize_events: 0,
                terminfo_found_in_db: false,
                can_change_scroll_region: false,
                has_left_and_right_margin_mode: false,
                has_sync_mode: false,
                can_set_lr_margin: false,
                can_scroll: false,
                can_erase_chars: false,
                immediate_wrap_after_last_column: false,
                bce: false,
                mouse_enabled: false,
                mouse_move_enabled: false,
                mouse_enabled_save: false,
                title_enabled: false,
                sync_output: false,
                busy: false,
                is_invisible: false,
                want_invisible: false,
                set_cursor_color_as_str: false,
                cursor_has_color: false,
                is_starting: true,
                resize_events_enabled: false,
                modes: core::mem::zeroed(),
                screenshot: core::ptr::null_mut(),
                cursor_shapes: core::mem::zeroed(),
                clear_attrs: core::mem::zeroed(),
                attrs: vec![DEFAULT_ATTRS],
                print_attr_id: 0,
                default_attr: false,
                set_default_colors: false,
                can_clear_attr: false,
                showing_mode: 0,
                verbose: 0,
                terminfo_ext: TerminfoExt::default(),
                can_set_title: false,
                can_set_underline_color: false,
                can_resize_screen: false,
                stopped: false,
                width: 0,
                height: 0,
                rgb: false,
                screen_or_tmux: false,
                url: -1,
                ti_arena: ARENA_EMPTY,
            })
        }
    }
}

/// Bring the terminal up: resolve its terminfo, negotiate what it can do,
/// and open the write path.
///
/// # Safety
/// `tui` must point to a live [`TUIData`] whose terminfo is not resolved.
unsafe fn terminfo_start(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`; the terminfo lookups either
    // borrow from the arena below or from static tables.
    unsafe {
        (*tui).staging.clear();
        (*tui).default_attr = false;
        (*tui).can_clear_attr = false;
        (*tui).is_invisible = true;
        (*tui).want_invisible = false;
        (*tui).busy = false;
        (*tui).set_cursor_color_as_str = false;
        (*tui).cursor_has_color = false;
        (*tui).resize_events_enabled = false;
        (*tui).modes.set_grapheme_clusters(false);
        (*tui).modes.set_resize_events(false);
        (*tui).modes.set_theme_updates(false);
        (*tui).showing_mode = 0;
        (*tui).terminfo_ext = TerminfoExt::default();
        (*tui).out_fd = STDOUT_FILENO;
        (*tui).out_isatty = os_isatty((*tui).out_fd);
        (*tui).input.tui_data = tui;
        (*tui).ti_arena = ARENA_EMPTY;
        assert!((*tui).term.is_null(), "terminfo already resolved");

        let term = os_getenv(c"TERM".as_ptr());
        let term_name = (!term.is_null()).then(|| CStr::from_ptr(term));
        (*tui).terminfo_found_in_db = false;
        if let Some(name) = term_name
            && let Some(entry) = terminfo_from_database(name, &raw mut (*tui).ti_arena)
        {
            (*tui).ti = entry;
            (*tui).term = arena_strdup(&raw mut (*tui).ti_arena, term);
            (*tui).terminfo_found_in_db = true;
        }
        if !(*tui).terminfo_found_in_db {
            let (builtin_name, entry) = terminfo_from_builtin(term_name);
            (*tui).ti = entry;
            (*tui).term = builtin_name.as_ptr().cast_mut();
        }

        let quirks = Terminal::identify(term_name);
        (*tui).screen_or_tmux = quirks.screen || quirks.tmux;
        (*tui).rgb = quirks.has_truecolor(&(*tui).ti);
        patch_terminfo_bugs(&mut (*tui).ti, &raw mut (*tui).ti_arena, &quirks);
        let augmented = augment_terminfo(&mut (*tui).ti, &quirks);
        (*tui).can_resize_screen = augmented.can_resize_screen;
        (*tui).can_set_title = augmented.can_set_title;
        (*tui).set_cursor_color_as_str = augmented.set_cursor_color_as_str;
        (*tui).terminfo_ext = augmented.ext;
        (*tui).input.key_encoding = augmented.key_encoding;
        if augmented.extended_underline {
            tui_enable_extended_underline(tui);
        }

        let defs = (*tui).ti.defs;
        let defined = |cap: TerminfoDef| !defs[cap as usize].is_null();
        (*tui).can_change_scroll_region = defined(kTerm_change_scroll_region);
        (*tui).can_set_lr_margin = defined(kTerm_set_lr_margin);
        (*tui).can_scroll = defined(kTerm_delete_line)
            && defined(kTerm_parm_delete_line)
            && defined(kTerm_insert_line)
            && defined(kTerm_parm_insert_line);
        (*tui).can_erase_chars = defined(kTerm_erase_chars);
        (*tui).immediate_wrap_after_last_column = quirks.wraps_after_last_column;
        (*tui).bce = (*tui).ti.bce;
        t_colors.set((*tui).ti.max_colors);

        terminfo_out(&mut *tui, kTerm_enter_ca_mode);
        terminfo_out(&mut *tui, kTerm_keypad_xmit);
        terminfo_out(&mut *tui, kTerm_clear_screen);
        tui_set_term_mode(&mut *tui, BRACKETED_PASTE, true);

        // What the terminal supports is only known once it answers; assume
        // nothing until then.
        (*tui).has_left_and_right_margin_mode = false;
        (*tui).has_sync_mode = false;
        if !quirks.nsterm {
            // Terminal.app answers mode queries with gibberish that ends up
            // in the editor as typed input, so it is never asked.
            for mode in [
                LEFT_AND_RIGHT_MARGINS,
                SYNCHRONIZED_OUTPUT,
                GRAPHEME_CLUSTERS,
                THEME_UPDATES,
                RESIZE_EVENTS,
            ] {
                tui_request_term_mode(&mut *tui, mode);
            }
        }
        if !defined(kTerm_set_underline_style) && !quirks.screen && !quirks.tmux && !quirks.nsterm {
            tui_query_extended_underline(tui);
        }
        tui_query_kitty_keyboard(tui);
        tui_query_bg_color_noflush(tui);

        open_output(tui);
        flush(&mut *tui);
        xfree(term.cast());
    }
}

/// Attach the output file descriptor to the TUI's own write loop.
///
/// # Safety
/// `tui` must point to a live [`TUIData`] with `out_fd` set.
unsafe fn open_output(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`; the handles live in it.
    unsafe {
        uv_loop_init(&raw mut (*tui).write_loop);
        if !(*tui).out_isatty {
            let ret = uv_pipe_init(
                &raw mut (*tui).write_loop,
                &raw mut (*tui).output_handle.pipe,
                0,
            );
            log_uv_error(ret, c"uv_pipe_init failed: %s");
            let ret = uv_pipe_open(&raw mut (*tui).output_handle.pipe, (*tui).out_fd as uv_file);
            log_uv_error(ret, c"uv_pipe_open failed: %s");
            return;
        }
        let ret = uv_tty_init(
            &raw mut (*tui).write_loop,
            &raw mut (*tui).output_handle.tty,
            (*tui).out_fd as uv_file,
            0,
        );
        log_uv_error(ret, c"uv_tty_init failed: %s");
        // Setting the mode can be interrupted by a signal, which says
        // nothing about whether it would succeed.
        let mut retries = 10;
        let mut ret = UV_EINTR;
        while ret == UV_EINTR && retries > 0 {
            ret = uv_tty_set_mode(&raw mut (*tui).output_handle.tty, UV_TTY_MODE_IO);
            retries -= 1;
        }
        log_uv_error(ret, c"uv_tty_set_mode failed: %s");
    }
}

/// Log a libuv failure, if `ret` is one.
///
/// # Safety
/// `message` must hold exactly one `%s`.
unsafe fn log_uv_error(ret: c_int, message: &CStr) {
    if ret == 0 {
        return;
    }
    // SAFETY: the caller guarantees the format string's one `%s`, which
    // `uv_strerror` fills with a static string.
    unsafe {
        logmsg_c!(
            LOGLVL_ERR,
            core::ptr::null(),
            c"tui".as_ptr(),
            0,
            true,
            message.as_ptr(),
            uv_strerror(ret),
        );
    }
}

/// Put the terminal back the way it was found, short of closing it.
///
/// Every mode this TUI turned on is turned off, in the reverse of the order
/// it was turned on, and the terminal is asked for its device attributes —
/// the reply is what tells the caller the resets have been processed.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn terminfo_disable(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        if (*tui).modes.theme_updates() {
            tui_set_term_mode(&mut *tui, THEME_UPDATES, false);
        }
        tui_mode_change(&mut *tui, NULL_STRING, 0);
        tui_mouse_off(&mut *tui);
        terminfo_out(&mut *tui, kTerm_exit_attribute_mode);
        terminfo_out(&mut *tui, kTerm_cursor_normal);
        terminfo_out(&mut *tui, kTerm_reset_cursor_style);
        terminfo_out(&mut *tui, kTerm_keypad_local);
        tui_reset_key_encoding(tui);
        if (*tui).modes.resize_events() {
            tui_set_term_mode(&mut *tui, RESIZE_EVENTS, false);
        }
        if (*tui).modes.grapheme_clusters() {
            tui_set_term_mode(&mut *tui, GRAPHEME_CLUSTERS, false);
        }
        tui_set_title(&mut *tui, NULL_STRING);
        if (*tui).cursor_has_color {
            terminfo_out(&mut *tui, kTerm_reset_cursor_color);
        }
        tui_set_term_mode(&mut *tui, BRACKETED_PASTE, false);
        let focus_off = (*tui).terminfo_ext.disable_focus_reporting;
        out_cstr(&mut *tui, focus_off);
        out(&mut *tui, b"\x1b[c");
        flush(&mut *tui);
    }
}

/// An empty API string, for the sinks that take one and are being called to
/// mean "nothing".
const NULL_STRING: String_0 = String_0 {
    data: core::ptr::null_mut(),
    size: 0,
};

/// Close the terminal down and give back everything terminfo allocated.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn terminfo_stop(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        // On a clean exit the cursor is left on the last line, so whatever
        // the shell prints next starts below the editor's output rather
        // than over it. After an error the screen is left as it is.
        if ui_client_exit_status.get() == 0 && ui_client_error_exit.get() > 0 {
            ui_client_exit_status.set(ui_client_error_exit.get());
        }
        if ui_client_exit_status.get() == ui_client_error_exit.get().max(0) {
            cursor_goto(&mut *tui, (*tui).height - 1, 0);
            terminfo_out(&mut *tui, kTerm_exit_ca_mode);
        }
        flush(&mut *tui);
        uv_tty_reset_mode();
        uv_close((&raw mut (*tui).output_handle).cast::<uv_handle_t>(), None);
        uv_run(&raw mut (*tui).write_loop, UV_RUN_DEFAULT);
        if uv_loop_close(&raw mut (*tui).write_loop) != 0 {
            // A loop that will not close has a handle still open, which
            // would mean writing to a terminal that is being reset.
            abort();
        }
        arena_mem_free(arena_finish(&raw mut (*tui).ti_arena));
        (*tui).ti = core::mem::zeroed();
        (*tui).term = core::ptr::null_mut();
    }
}

/// Start the terminal and the input layer on top of it.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn tui_terminal_start(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        (*tui).print_attr_id = -1;
        terminfo_start(tui);
        if (*tui).input.loop_0.is_null() {
            tinput_init(&raw mut (*tui).input, main_loop.ptr(), &raw mut (*tui).ti);
        }
        tui_guess_size(tui);
        tinput_start(&raw mut (*tui).input);
    }
}

/// The startup delay timer's callback.
///
/// # Safety
/// Called by libuv with the timer this TUI started.
unsafe extern "C" fn after_startup_cb(handle: *mut uv_timer_t) {
    // SAFETY: the timer's `data` is the TUI that started it.
    unsafe { tui_terminal_after_startup((*handle).data.cast()) };
}

/// Send what had to wait until the terminal had settled.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn tui_terminal_after_startup(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        let focus_on = (*tui).terminfo_ext.enable_focus_reporting;
        out_cstr(&mut *tui, focus_on);
        flush(&mut *tui);
    }
}

/// Stop the TUI: reset the terminal, wait for it to say so, and let go of
/// everything.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_stop(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        if uv_is_closing((&raw mut (*tui).output_handle).cast::<uv_handle_t>()) != 0 {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"tui_stop".as_ptr(),
                0,
                true,
                c"TUI already stopped (race?)".as_ptr(),
            );
            (*tui).stopped = true;
            return;
        }
        // The terminal's answer to the device-attributes query terminfo
        // disable ends with is what says the resets have been processed.
        (*tui).input.callbacks.primary_device_attr = Some(tui_stop_cb);
        terminfo_disable(tui);
        process_events_until(
            (*tui).loop_0,
            (*(*tui).loop_0).events,
            DA1_TIMEOUT_MS,
            || (*tui).stopped || (*tui).input.read_stream.did_eof,
        );
        if !(*tui).stopped && !(*tui).input.read_stream.did_eof {
            logmsg_c!(
                LOGLVL_WRN,
                core::ptr::null(),
                c"tui_stop".as_ptr(),
                0,
                true,
                c"TUI: timed out waiting for DA1 response".as_ptr(),
            );
        }
        (*tui).stopped = true;
        tui_terminal_stop(tui);
        stream_set_blocking((*tui).input.in_fd, true);
        tinput_destroy(&raw mut (*tui).input);
        signal_watcher_stop(&raw mut (*tui).winch_handle);
        signal_watcher_close(&raw mut (*tui).winch_handle, None);
        uv_close(
            (&raw mut (*tui).startup_delay_timer).cast::<uv_handle_t>(),
            None,
        );
    }
}

/// How long to wait for the terminal to answer the shutdown query.
const DA1_TIMEOUT_MS: i64 = 1000;

/// The device-attributes reply that ends [`tui_stop`]'s wait.
///
/// # Safety
/// Called by the input layer with the TUI it belongs to.
unsafe fn tui_stop_cb(tui: *mut TUIData) {
    // SAFETY: the input layer holds this TUI's own pointer.
    unsafe { (*tui).stopped = true };
}

/// Stop reading input and close the terminal.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn tui_terminal_stop(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        tinput_stop(&raw mut (*tui).input);
        terminfo_stop(tui);
    }
}

/// Has the TUI been stopped?
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_is_stopped(tui: *mut TUIData) -> bool {
    // SAFETY: the caller guarantees `tui`.
    unsafe { (*tui).stopped }
}

/// Suspend the editor, giving the terminal back to the shell.
///
/// The terminal is reset first, and — as in [`tui_stop`] — the actual stop
/// waits for the terminal to answer, so the shell does not get the terminal
/// mid-reset.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_suspend(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        ui_client_detach();
        (*tui).mouse_enabled_save = (*tui).mouse_enabled;
        (*tui).input.callbacks.primary_device_attr = Some(tui_suspend_cb);
        terminfo_disable(tui);
    }
}

/// Actually suspend, once the terminal has confirmed the reset.
///
/// # Safety
/// Called by the input layer with the TUI it belongs to.
unsafe fn tui_suspend_cb(tui: *mut TUIData) {
    // SAFETY: the input layer holds this TUI's own pointer.
    unsafe {
        tui_terminal_stop(tui);
        stream_set_blocking((*tui).input.in_fd, true);
        // Stop the whole process group, which is what returns to the shell.
        kill(0, SIGSTOP);
        // Execution resumes here on SIGCONT.
        tui_terminal_start(tui);
        tui_terminal_after_startup(tui);
        if (*tui).mouse_enabled_save {
            tui_mouse_on(&mut *tui);
        }
        stream_set_blocking((*tui).input.in_fd, false);
        ui_client_attach((*tui).width, (*tui).height, (*tui).term, (*tui).rgb);
    }
}

// ------------------------------------------------------------------- sizing

/// The terminal was resized.
///
/// # Safety
/// Called by the signal watcher with the TUI as its data.
unsafe extern "C" fn sigwinch_cb(
    _watcher: *mut SignalWatcher,
    _signum: c_int,
    cbdata: *mut c_void,
) {
    let tui: *mut TUIData = cbdata.cast();
    // SAFETY: the watcher holds this TUI's own pointer.
    unsafe {
        // Terminals that report resizes themselves have already told us.
        if tui_is_stopped(tui) || (*tui).resize_events_enabled {
            return;
        }
        tui_guess_size(tui);
    }
}

/// Tell the editor the terminal is `width` by `height`.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
pub unsafe fn tui_set_size(tui: *mut TUIData, width: c_int, height: c_int) {
    // SAFETY: the caller guarantees `tui`.
    unsafe {
        (*tui).pending_resize_events += 1;
        (*tui).width = width;
        (*tui).height = height;
        ui_client_set_size(width, height);
    }
}

/// Work out how big the terminal is, from the best source that answers.
///
/// The terminal itself is authoritative; `$LINES`/`$COLUMNS` come next (they
/// are how a terminal-less run says what to pretend), then whatever terminfo
/// claimed, and finally a plain 80x24.
///
/// # Safety
/// `tui` must point to a live [`TUIData`].
unsafe fn tui_guess_size(tui: *mut TUIData) {
    // SAFETY: the caller guarantees `tui`; the environment reads borrow.
    unsafe {
        let (mut width, mut height) = (0, 0);
        let from_tty = (*tui).out_isatty
            && uv_tty_get_winsize(
                &raw mut (*tui).output_handle.tty,
                &raw mut width,
                &raw mut height,
            ) == 0;
        if !from_tty
            && !(env_size(c"LINES", &raw mut height) && env_size(c"COLUMNS", &raw mut width))
        {
            height = (*tui).ti.lines;
            width = (*tui).ti.columns;
        }
        if width <= 0 || height <= 0 {
            width = DFLT_COLS;
            height = DFLT_ROWS;
        }
        tui_set_size(tui, width, height);
    }
}

/// Read a size out of the environment variable `name`.
///
/// # Safety
/// `out` must be valid for writes.
unsafe fn env_size(name: &CStr, out: *mut c_int) -> bool {
    // SAFETY: `os_getenv_noalloc` returns null or a NUL-terminated string,
    // and `%d%n` writes one int and one int.
    unsafe {
        let value = os_getenv_noalloc(name.as_ptr());
        if value.is_null() {
            return false;
        }
        let mut consumed: c_int = 0;
        sscanf(value, c"%d%n".as_ptr(), out, &raw mut consumed) != EOF && consumed != 0
    }
}

const EOF: c_int = -1;
