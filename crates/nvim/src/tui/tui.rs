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
//! The state lives in one cell for the life of the process. Everything takes
//! `&mut TUIData`, borrowed from the cell for one step; the lifecycle steps
//! that turn the event loop (waiting for the terminal's answers) hold
//! nothing of it while they wait, because the loop's callbacks -- the input
//! layer's reads and timers, the resize watcher -- look it up themselves.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `tui/{tui,input,output,events}.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::{
    uv_close, uv_is_closing, uv_loop_close, uv_loop_init, uv_pipe_init, uv_pipe_open, uv_run,
    uv_timer_init, uv_timer_start, uv_tty_get_winsize, uv_tty_init, uv_tty_reset_mode,
    uv_tty_set_mode,
};
use crate::event::r#loop::{Drain, main_loop_raw, poll_main_loop, wait_on_main_loop};
use crate::event::objects::SignalWatcher;
use crate::event::signal::{
    signal_watcher_close, signal_watcher_init, signal_watcher_start, signal_watcher_stop,
};
use crate::event::stream::stream_set_blocking;
use crate::event::uv::{uv_file, uv_handle_t, uv_timer_t, uv_tty_mode_t};
use crate::global_cell::GlobalCell;
use crate::highlight::{HlAttrFlags, HlAttrs};
use crate::log::{LOGLVL_ERR, LOGLVL_WRN, logmsg};
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::option::{DFLT_COLS, DFLT_ROWS};
use crate::os::env::{env_get_within, getenv_owned};
use crate::os::fs::os_strerror;
use crate::os::input::os_isatty;
use crate::os::libc::STDOUT_FILENO;
use crate::os::uv_error::UV_EINTR;
use crate::startup::{UI_CLIENT_ERROR_EXIT, UI_CLIENT_EXIT_STATUS};
use crate::tui::events::{tui_mode_change, tui_mouse_off, tui_mouse_on, tui_set_title};
use crate::tui::input::{tinput_destroy, tinput_init, tinput_start, tinput_stop};
use crate::tui::negotiate::{
    BRACKETED_PASTE, GRAPHEME_CLUSTERS, LEFT_AND_RIGHT_MARGINS, RESIZE_EVENTS, SYNCHRONIZED_OUTPUT,
    THEME_UPDATES, tui_enable_extended_underline, tui_query_bg_color_noflush,
    tui_query_extended_underline, tui_query_kitty_keyboard, tui_request_term_mode,
    tui_reset_key_encoding, tui_set_term_mode,
};
use crate::tui::output::{flush, out, out_cstr, terminfo_out};
use crate::tui::paint::cursor_goto;
use crate::tui::quirks::{Terminal, augment_terminfo, patch_terminfo_bugs};
use crate::tui::state::{Staging, TUIData, TermInput, TerminfoExt};
use crate::tui::terminfo::caps::{
    TERM_CHANGE_SCROLL_REGION, TERM_CLEAR_SCREEN, TERM_CURSOR_NORMAL, TERM_DELETE_LINE,
    TERM_ENTER_CA_MODE, TERM_ERASE_CHARS, TERM_EXIT_ATTRIBUTE_MODE, TERM_EXIT_CA_MODE,
    TERM_INSERT_LINE, TERM_KEYPAD_LOCAL, TERM_KEYPAD_XMIT, TERM_PARM_DELETE_LINE,
    TERM_PARM_INSERT_LINE, TERM_RESET_CURSOR_COLOR, TERM_RESET_CURSOR_STYLE, TERM_SET_LR_MARGIN,
    TERM_SET_UNDERLINE_STYLE, TerminfoDef,
};
use crate::tui::terminfo::entry::TerminfoEntry;
use crate::tui::terminfo::{terminfo_from_builtin, terminfo_from_database};
use crate::types::ApiString;
use crate::ui::state::T_COLORS;
use crate::ui_client::{ui_client_attach, ui_client_detach, ui_client_set_size};
use ::libc::{abort, kill};
use core::ffi::{CStr, c_int, c_void};
use core::ptr;
use std::borrow::Cow;

// --------------------------------------------------------------- the state

// ----------------------------------------------------------- the constants

const UV_RUN_DEFAULT: core::ffi::c_uint = 0;
const UV_TTY_MODE_IO: uv_tty_mode_t = 2;

const SIGSTOP: c_int = 19;
const SIGWINCH: c_int = 28;

/// The first entry in the highlight table: the editor's default highlight.
pub(crate) const DEFAULT_ATTRS: HlAttrs = HlAttrs {
    rgb_ae_attr: HlAttrFlags::NONE,
    cterm_ae_attr: HlAttrFlags::NONE,
    rgb_fg_color: -1,
    rgb_bg_color: -1,
    rgb_sp_color: -1,
    cterm_fg_color: 0,
    cterm_bg_color: 0,
    hl_blend: -1,
    url: -1,
};

// ------------------------------------------------------------- the lifecycle

/// The TUI's state, from [`Tui::start`] for the life of the process.
///
/// Boxed, because libuv keeps the addresses of the handles inside it; the
/// handles' callbacks carry no address back, they look the state up here.
/// Every step borrows it for its own length and none across a turn of the
/// event loop, so a callback the loop runs finds it free -- and the cell's
/// debug borrow tracking says so loudly if one ever does not.
static STATE: GlobalCell<Option<Box<TUIData>>> = GlobalCell::new(None);

/// Run `f` on the started TUI's state. `f` must not run the event loop.
fn with_tui<R>(f: impl FnOnce(&mut TUIData) -> R) -> R {
    let mut f = Some(f);
    let mut answer = None;
    lend_state(&mut |tui| answer = f.take().map(|f| f(tui)));
    answer.expect("the state was lent")
}

/// [`with_tui`]'s one body that names the cell: not generic, so the cell
/// is only ever reached from this module's code and stays unexported.
#[inline(never)]
fn lend_state(f: &mut dyn FnMut(&mut TUIData)) {
    STATE.with_mut(|state| f(state.as_mut().expect("the TUI is started")));
}

/// Read the started TUI's state.
fn read_tui<R>(f: impl FnOnce(&TUIData) -> R) -> R {
    with_tui(|tui| f(tui))
}

/// [`with_tui`], for the input layer's callbacks, which are reached from
/// libuv with nothing of the TUI borrowed.
pub(crate) fn with_started_tui<R>(f: impl FnOnce(&mut TUIData) -> R) -> R {
    with_tui(f)
}

/// The running TUI, as its client holds it: a token for the state in
/// [`STATE`], which the process exits with.
#[derive(Debug)]
pub struct Tui {
    _started: (),
}

impl Tui {
    /// Take over the terminal. Panics if a TUI was started before: there is
    /// one terminal.
    ///
    /// The queries whose answers the client needs — size, name, colour
    /// depth — have been sent but not answered when this returns;
    /// [`Tui::wait_ready`] collects them.
    pub fn start() -> Self {
        let fresh = STATE.with_mut(|state| state.replace(TUIData::new()).is_none());
        assert!(fresh, "the TUI is started once");
        with_tui(|tui| {
            // SAFETY: the editor's loop, live from startup to exit, and the
            // watcher inside the boxed state, which never moves; its callback
            // looks the state up again.
            unsafe {
                signal_watcher_init(main_loop_raw(), &raw mut tui.winch_handle, ptr::null_mut());
                signal_watcher_start(&raw mut tui.winch_handle, Some(sigwinch_cb), SIGWINCH);
            }
            tui_terminal_start(tui);
            // SAFETY: as above, for the startup timer.
            unsafe {
                uv_timer_init(
                    &raw mut (*main_loop_raw()).uv,
                    &raw mut tui.startup_delay_timer,
                );
                uv_timer_start(
                    &raw mut tui.startup_delay_timer,
                    Some(after_startup_cb),
                    STARTUP_DELAY_MS,
                    0,
                );
            }
        });
        Self { _started: () }
    }

    /// Wait for the terminal to answer what [`Tui::start`] asked it.
    ///
    /// One turn of the event loop, so that what the client is told is the
    /// terminal's answer rather than the guess. It is separate from
    /// starting because that turn can run any callback the loop has pending
    /// — including the one that stops this TUI when its channel is already
    /// at EOF — so the caller has to have published the TUI first.
    pub fn wait_ready(&self) -> TuiStart {
        poll_main_loop(1);
        read_tui(|tui| TuiStart {
            width: tui.width,
            height: tui.height,
            term: tui.term.as_deref().map(CStr::to_owned),
            rgb: tui.rgb,
        })
    }

    /// Has the TUI been stopped?
    pub fn is_stopped(&self) -> bool {
        read_tui(|tui| tui.stopped)
    }

    /// Stop the TUI: reset the terminal, wait for it to say so, and let go
    /// of everything.
    pub fn stop(&self) {
        tui_stop();
    }

    /// Run `f`, one of the drawing sinks, on the TUI's state.
    pub fn with_state<R>(&self, f: impl FnOnce(&mut TUIData) -> R) -> R {
        with_tui(f)
    }
}

/// What a started TUI tells its client about the terminal it found.
#[derive(Debug)]
pub struct TuiStart {
    pub width: c_int,
    pub height: c_int,
    /// A copy of the `TERM` the TUI resolved: the [`TUIData`]'s own is freed
    /// when the terminal stops.
    pub term: Option<std::ffi::CString>,
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
        // libuv and the grid -- fill them in before
        // reading them, exactly as they did when this struct was xcalloc'ed.
        unsafe {
            Box::new(Self {
                staging: Staging::new(),
                input: TermInput::new(),
                write_loop: core::mem::zeroed(),
                ti: TerminfoEntry::default(),
                term: None,
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
                screenshot: None,
                cursor_shapes: crate::cursor_shape::CursorEntry::BLANK_TABLE,
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
            })
        }
    }
}

/// Bring the terminal up: resolve its terminfo, negotiate what it can do,
/// and open the write path.
fn terminfo_start(tui: &mut TUIData) {
    tui.staging.clear();
    tui.default_attr = false;
    tui.can_clear_attr = false;
    tui.is_invisible = true;
    tui.want_invisible = false;
    tui.busy = false;
    tui.set_cursor_color_as_str = false;
    tui.cursor_has_color = false;
    tui.resize_events_enabled = false;
    tui.modes.set_grapheme_clusters(false);
    tui.modes.set_resize_events(false);
    tui.modes.set_theme_updates(false);
    tui.showing_mode = 0;
    tui.terminfo_ext = TerminfoExt::default();
    tui.out_fd = STDOUT_FILENO;
    tui.out_isatty = os_isatty(tui.out_fd);
    assert!(tui.term.is_none(), "terminfo already resolved");

    let term = getenv_owned(c"TERM");
    let term_name = term.as_ref().map(XString::as_cstr);
    tui.terminfo_found_in_db = false;
    if let Some(name) = term_name
        && let Some(entry) = terminfo_from_database(name)
    {
        tui.ti = entry;
        tui.term = Some(Cow::Owned(name.to_owned()));
        tui.terminfo_found_in_db = true;
    }
    if !tui.terminfo_found_in_db {
        let (builtin_name, entry) = terminfo_from_builtin(term_name);
        tui.ti = entry;
        tui.term = Some(Cow::Borrowed(builtin_name));
    }

    let quirks = Terminal::identify(term_name);
    tui.screen_or_tmux = quirks.screen || quirks.tmux;
    tui.rgb = quirks.has_truecolor(&tui.ti);
    patch_terminfo_bugs(&mut tui.ti, &quirks);
    let augmented = augment_terminfo(&mut tui.ti, &quirks);
    tui.can_resize_screen = augmented.can_resize_screen;
    tui.can_set_title = augmented.can_set_title;
    tui.set_cursor_color_as_str = augmented.set_cursor_color_as_str;
    tui.terminfo_ext = augmented.ext;
    tui.input.key_encoding = augmented.key_encoding;
    if augmented.extended_underline {
        tui_enable_extended_underline(tui);
    }

    let defined = |tui: &TUIData, cap: TerminfoDef| tui.ti.has(cap);
    tui.can_change_scroll_region = defined(tui, TERM_CHANGE_SCROLL_REGION);
    tui.can_set_lr_margin = defined(tui, TERM_SET_LR_MARGIN);
    tui.can_scroll = defined(tui, TERM_DELETE_LINE)
        && defined(tui, TERM_PARM_DELETE_LINE)
        && defined(tui, TERM_INSERT_LINE)
        && defined(tui, TERM_PARM_INSERT_LINE);
    tui.can_erase_chars = defined(tui, TERM_ERASE_CHARS);
    tui.immediate_wrap_after_last_column = quirks.wraps_after_last_column;
    tui.bce = tui.ti.bce;
    T_COLORS.set(tui.ti.max_colors);

    terminfo_out(tui, TERM_ENTER_CA_MODE);
    terminfo_out(tui, TERM_KEYPAD_XMIT);
    terminfo_out(tui, TERM_CLEAR_SCREEN);
    tui_set_term_mode(tui, BRACKETED_PASTE, true);

    // What the terminal supports is only known once it answers; assume
    // nothing until then.
    tui.has_left_and_right_margin_mode = false;
    tui.has_sync_mode = false;
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
            tui_request_term_mode(tui, mode);
        }
    }
    if !defined(tui, TERM_SET_UNDERLINE_STYLE) && !quirks.screen && !quirks.tmux && !quirks.nsterm {
        tui_query_extended_underline(tui);
    }
    tui_query_kitty_keyboard(tui);
    tui_query_bg_color_noflush(tui);

    open_output(tui);
    flush(tui);
}

/// Attach the output file descriptor to the TUI's own write loop.
fn open_output(tui: &mut TUIData) {
    let out_fd = tui.out_fd as uv_file;
    // SAFETY: the write loop and the output handle are this TUI's own
    // fields, which do not move (the TUI is boxed for the process); libuv
    // keeps their addresses from here until `terminfo_stop` closes them.
    unsafe {
        uv_loop_init(&raw mut tui.write_loop);
        if !tui.out_isatty {
            let ret = uv_pipe_init(&raw mut tui.write_loop, &raw mut tui.output_handle.pipe, 0);
            log_uv_error(ret, "uv_pipe_init");
            let ret = uv_pipe_open(&raw mut tui.output_handle.pipe, out_fd);
            log_uv_error(ret, "uv_pipe_open");
            return;
        }
        let ret = uv_tty_init(
            &raw mut tui.write_loop,
            &raw mut tui.output_handle.tty,
            out_fd,
            0,
        );
        log_uv_error(ret, "uv_tty_init");
        // Setting the mode can be interrupted by a signal, which says
        // nothing about whether it would succeed.
        let mut retries = 10;
        let mut ret = UV_EINTR;
        while ret == UV_EINTR && retries > 0 {
            ret = uv_tty_set_mode(&raw mut tui.output_handle.tty, UV_TTY_MODE_IO);
            retries -= 1;
        }
        log_uv_error(ret, "uv_tty_set_mode");
    }
}

/// Log a libuv failure, if `ret` is one.
///
/// One `LOGLVL_ERR` line naming the libuv call that answered `ret`.
fn log_uv_error(ret: c_int, call: &str) {
    if ret == 0 {
        return;
    }
    let why = msg_cstr(os_strerror(ret));
    logmsg!(LOGLVL_ERR, c"tui", 0, "{call} failed: {why}");
}

/// Put the terminal back the way it was found, short of closing it.
///
/// Every mode this TUI turned on is turned off, in the reverse of the order
/// it was turned on, and the terminal is asked for its device attributes —
/// the reply is what tells the caller the resets have been processed.
fn terminfo_disable(tui: &mut TUIData) {
    if tui.modes.theme_updates() {
        tui_set_term_mode(tui, THEME_UPDATES, false);
    }
    tui_mode_change(tui, ApiString::NULL, 0);
    tui_mouse_off(tui);
    terminfo_out(tui, TERM_EXIT_ATTRIBUTE_MODE);
    terminfo_out(tui, TERM_CURSOR_NORMAL);
    terminfo_out(tui, TERM_RESET_CURSOR_STYLE);
    terminfo_out(tui, TERM_KEYPAD_LOCAL);
    tui_reset_key_encoding(tui);
    if tui.modes.resize_events() {
        tui_set_term_mode(tui, RESIZE_EVENTS, false);
    }
    if tui.modes.grapheme_clusters() {
        tui_set_term_mode(tui, GRAPHEME_CLUSTERS, false);
    }
    tui_set_title(tui, ApiString::NULL);
    if tui.cursor_has_color {
        terminfo_out(tui, TERM_RESET_CURSOR_COLOR);
    }
    tui_set_term_mode(tui, BRACKETED_PASTE, false);
    let focus_off = tui.terminfo_ext.disable_focus_reporting;
    out_cstr(tui, focus_off);
    out(tui, b"\x1b[c");
    flush(tui);
}

/// Close the terminal down and give back everything terminfo allocated.
fn terminfo_stop(tui: &mut TUIData) {
    // On a clean exit the cursor is left on the last line, so whatever the
    // shell prints next starts below the editor's output rather than over
    // it. After an error the screen is left as it is.
    if UI_CLIENT_EXIT_STATUS.get() == 0 && UI_CLIENT_ERROR_EXIT.get() > 0 {
        UI_CLIENT_EXIT_STATUS.set(UI_CLIENT_ERROR_EXIT.get());
    }
    if UI_CLIENT_EXIT_STATUS.get() == UI_CLIENT_ERROR_EXIT.get().max(0) {
        cursor_goto(tui, tui.height - 1, 0);
        terminfo_out(tui, TERM_EXIT_CA_MODE);
    }
    flush(tui);
    // SAFETY: the output handle and the write loop `open_output` opened;
    // the loop runs the close to completion before it is closed itself.
    unsafe {
        uv_tty_reset_mode();
        uv_close((&raw mut tui.output_handle).cast::<uv_handle_t>(), None);
        uv_run(&raw mut tui.write_loop, UV_RUN_DEFAULT);
        if uv_loop_close(&raw mut tui.write_loop) != 0 {
            // A loop that will not close has a handle still open, which
            // would mean writing to a terminal that is being reset.
            abort();
        }
    }
    tui.ti = TerminfoEntry::default();
    tui.term = None;
}

/// Start the terminal and the input layer on top of it.
fn tui_terminal_start(tui: &mut TUIData) {
    tui.print_attr_id = -1;
    terminfo_start(tui);
    if tui.input.tk.is_none() {
        tinput_init(&mut tui.input, &tui.ti);
    }
    tui_guess_size(tui);
    tinput_start(&mut tui.input);
}

/// The startup delay timer's callback.
///
/// # Safety
/// libuv's timer callback, on the TUI's startup timer.
unsafe extern "C" fn after_startup_cb(_handle: *mut uv_timer_t) {
    with_tui(tui_terminal_after_startup);
}

/// Send what had to wait until the terminal had settled.
fn tui_terminal_after_startup(tui: &mut TUIData) {
    let focus_on = tui.terminfo_ext.enable_focus_reporting;
    out_cstr(tui, focus_on);
    flush(tui);
}

/// Stop the TUI: reset the terminal, wait for it to say so, and let go of
/// everything.
///
/// The wait turns the event loop, whose read callback reaches the state, so
/// it holds nothing of it: the state is borrowed before and after.
fn tui_stop() {
    let stopping = with_tui(|tui| {
        // SAFETY: the TUI's own output handle.
        if unsafe { uv_is_closing((&raw mut tui.output_handle).cast::<uv_handle_t>()) } != 0 {
            logmsg!(LOGLVL_ERR, c"tui_stop", 0, "TUI already stopped (race?)");
            tui.stopped = true;
            return false;
        }
        // The terminal's answer to the device-attributes query terminfo
        // disable ends with is what says the resets have been processed.
        tui.input.callbacks.primary_device_attr = Some(tui_stop_cb);
        terminfo_disable(tui);
        true
    });
    if !stopping {
        return;
    }
    let answered = || read_tui(|tui| tui.stopped || tui.input.read_stream.did_eof);
    wait_on_main_loop(Drain::Events, DA1_TIMEOUT_MS, answered);
    with_tui(|tui| {
        if !tui.stopped && !tui.input.read_stream.did_eof {
            logmsg!(
                LOGLVL_WRN,
                c"tui_stop",
                0,
                "TUI: timed out waiting for DA1 response"
            );
        }
        tui.stopped = true;
        tui_terminal_stop(tui);
        stream_set_blocking(tui.input.in_fd, true);
        tinput_destroy(&mut tui.input);
        // SAFETY: the TUI's own watcher and timer, open until here.
        unsafe {
            signal_watcher_stop(&raw mut tui.winch_handle);
            signal_watcher_close(&raw mut tui.winch_handle, None);
            uv_close(
                (&raw mut tui.startup_delay_timer).cast::<uv_handle_t>(),
                None,
            );
        }
    });
}

/// How long to wait for the terminal to answer the shutdown query.
const DA1_TIMEOUT_MS: i64 = 1000;

/// The device-attributes reply that ends [`tui_stop`]'s wait.
fn tui_stop_cb(tui: &mut TUIData) {
    tui.stopped = true;
}

/// Stop reading input and close the terminal.
fn tui_terminal_stop(tui: &mut TUIData) {
    tinput_stop(&mut tui.input);
    terminfo_stop(tui);
}

/// Has the TUI been stopped?
pub fn tui_is_stopped(tui: &TUIData) -> bool {
    tui.stopped
}

/// Suspend the editor, giving the terminal back to the shell.
///
/// The terminal is reset first, and — as in [`tui_stop`] — the actual stop
/// waits for the terminal to answer, so the shell does not get the terminal
/// mid-reset.
pub fn tui_suspend(tui: &mut TUIData) {
    ui_client_detach();
    tui.mouse_enabled_save = tui.mouse_enabled;
    tui.input.callbacks.primary_device_attr = Some(tui_suspend_cb);
    terminfo_disable(tui);
}

/// Actually suspend, once the terminal has confirmed the reset.
fn tui_suspend_cb(tui: &mut TUIData) {
    tui_terminal_stop(tui);
    stream_set_blocking(tui.input.in_fd, true);
    // Stop the whole process group, which is what returns to the shell.
    // SAFETY: signals this process's own group.
    unsafe { kill(0, SIGSTOP) };
    // Execution resumes here on SIGCONT. Restarting the terminal
    // re-registers its handles, which keep their addresses in the box.
    tui_terminal_start(tui);
    tui_terminal_after_startup(tui);
    if tui.mouse_enabled_save {
        tui_mouse_on(tui);
    }
    stream_set_blocking(tui.input.in_fd, false);
    let term = tui.term.clone();
    ui_client_attach(tui.width, tui.height, term.as_deref(), tui.rgb);
}

// ------------------------------------------------------------------- sizing

/// The terminal was resized.
///
/// # Safety
/// The signal watcher's callback; the TUI's state is looked up again.
unsafe fn sigwinch_cb(_watcher: *mut SignalWatcher, _signum: c_int, _cbdata: *mut c_void) {
    with_tui(|tui| {
        // Terminals that report resizes themselves have already told us.
        if tui_is_stopped(tui) || tui.resize_events_enabled {
            return;
        }
        tui_guess_size(tui);
    });
}

/// Tell the editor the terminal is `width` by `height`.
pub fn tui_set_size(tui: &mut TUIData, width: c_int, height: c_int) {
    tui.pending_resize_events += 1;
    tui.width = width;
    tui.height = height;
    ui_client_set_size(width, height);
}

/// Work out how big the terminal is, from the best source that answers.
///
/// The terminal itself is authoritative; `$LINES`/`$COLUMNS` come next (they
/// are how a terminal-less run says what to pretend), then whatever terminfo
/// claimed, and finally a plain 80x24.
fn tui_guess_size(tui: &mut TUIData) {
    let (mut width, mut height) = (0, 0);
    // SAFETY: the tty handle `open_output` opened when `out_isatty`, and two
    // ints of this frame's for libuv to fill.
    let from_tty = tui.out_isatty
        && unsafe {
            uv_tty_get_winsize(
                &raw mut tui.output_handle.tty,
                &raw mut width,
                &raw mut height,
            )
        } == 0;
    if !from_tty {
        match (env_size(c"LINES"), env_size(c"COLUMNS")) {
            (Some(lines), Some(columns)) => (height, width) = (lines, columns),
            _ => (height, width) = (tui.ti.lines, tui.ti.columns),
        }
    }
    if width <= 0 || height <= 0 {
        width = DFLT_COLS;
        height = DFLT_ROWS;
    }
    tui_set_size(tui, width, height);
}

/// Read a size out of the environment variable `name`: what `sscanf`'s
/// `%d` reads -- leading whitespace, a sign, at least one digit -- or `None`.
/// A number past `c_int`'s range is `None` too, where `sscanf` was undefined.
fn env_size(name: &CStr) -> Option<c_int> {
    let value = env_get_within(name, crate::path::MAXPATHL as usize)?;
    let text = value.to_vec();
    let text = text.trim_ascii_start();
    let sign = usize::from(matches!(text.first(), Some(b'+' | b'-')));
    let digits = text[sign..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }
    core::str::from_utf8(&text[..sign + digits])
        .ok()?
        .parse()
        .ok()
}
