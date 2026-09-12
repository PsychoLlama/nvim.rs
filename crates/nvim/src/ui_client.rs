//! The TUI as a client of a separate server process.
//!
//! `nvim` in a terminal is two processes: this one draws, and a headless
//! `--embed` server edits. This module is the client half. It starts or
//! connects to the server, attaches as a UI, and then does one thing
//! forever: turn each `redraw` event the server sends into a call on the
//! [`tui`](crate::tui) that owns the terminal.
//!
//! The event side is [`EVENT_HANDLERS`] and the wrappers it names. Each
//! wrapper takes the untyped [`Array`] the msgpack decoder produced,
//! checks it against the event's declared shape, and calls the TUI
//! function with typed arguments — the same shape upstream generates from
//! `ui_events.in.h`, spelled here as a table so that both names of every
//! event are greppable.
//!
//! `grid_line` is the exception and never reaches a wrapper: the decoder
//! recognises it and writes cells straight into the shared buffers
//! [`ui_client_event_raw_line`] reads, because building an `Array` per
//! cell is most of a redraw's cost.
//!
//! What is *not* here: anything about the terminal. This module knows only
//! the protocol.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::dispatch::key_dict_highlight_get_field;
use crate::api::private::helpers::{api_dict_to_keydict, api_metadata, cstr_to_string};
use crate::channel::{channel_connect, channel_job_start};
use crate::event::r#loop::process_events;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::socket::socket_address_is_tcp;
use crate::global_cell::GlobalCell;
use crate::highlight::{HLATTRS_INIT, dict2hlattrs};
use crate::log::{LOGLVL_ERR, LOGLVL_INF, logmsg};
use crate::memory::{strequal, xfree, xmalloc, xmemdupz, xstrdup};
use crate::message_fmt::{c_str, msg_cstr};
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::os::env::{os_env_exists, os_get_pid};
use crate::profile::time_fd;
use crate::profile::{time_finish, time_msg};
use crate::startup::{
    main_loop, os_exit, stderr_isatty, stdin_isatty, stdout_isatty, ui_client_attached,
    ui_client_channel_id, ui_client_error_exit, ui_client_exit_status, ui_client_forward_stdin,
};
use crate::tui::attrs::{tui_add_url, tui_default_colors_set, tui_hl_attr_define};
use crate::tui::events::{
    tui_bell, tui_busy_start, tui_busy_stop, tui_chdir, tui_mode_change, tui_mode_info_set,
    tui_mouse_off, tui_mouse_on, tui_option_set, tui_set_icon, tui_set_title, tui_ui_send,
    tui_update_menu, tui_visual_bell,
};
use crate::tui::paint::{
    tui_flush, tui_grid_clear, tui_grid_cursor_goto, tui_grid_resize, tui_grid_scroll,
    tui_raw_line, tui_screenshot,
};
use crate::tui::tui::{tui_is_stopped, tui_start, tui_stop, tui_suspend, tui_wait_ready};
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::channel::kChannelStdinPipe;
use crate::types::libc::{STDERR_FILENO, STDOUT_FILENO};
use crate::types::ui::kLineFlagWrap;
use crate::types::{
    ApiDict, Arena, Array, Callback, CallbackReader, Dict, Error, Event, GridLineEvent, HlAttrs,
    Integer, KeyDict_highlight, Object, ObjectType, ProfTime, String_0, TUIData, UIClientHandler,
    Unpacker, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger,
    kObjectTypeString, uint16_t,
};
use crate::ui::state::t_colors;
use ::libc::{close, dup};
use core::ffi::{CStr, c_char, c_int, c_void};

/// The descriptor the client hands the server as `stdin_fd` when the user
/// piped something in: stdin has been moved there so that the terminal can
/// take slot 0.
const FORWARDED_STDIN_FD: Integer = 3;

/// How long to wait for the server's socket, in milliseconds. Shorter than
/// `--server`'s own timeout: the server here was either just spawned by
/// this process or named by a `:restart` that has already bound it.
const UI_CONNECT_TIMEOUT_MS: c_int = 50;

/// The TUI this client draws through, and the size and terminal it was
/// started with. Attaching needs all four, and so does re-attaching after
/// `:restart`, which is why they outlive `ui_client_run`.
static tui: GlobalCell<*mut TUIData> = GlobalCell::new(core::ptr::null_mut());
static tui_width: GlobalCell<c_int> = GlobalCell::new(0);
static tui_height: GlobalCell<c_int> = GlobalCell::new(0);
/// The terminal name, **owned**. `tui_wait_ready` answers a pointer into
/// the TUI's own arena, which `tui_terminal_stop` frees; a re-attach can
/// run after that (a `:restart` that fails, or a close event during
/// shutdown), so this keeps a copy rather than the TUI's pointer.
static tui_term: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
static tui_rgb: GlobalCell<bool> = GlobalCell::new(false);

/// A reader that discards what it is given, for the streams this client
/// does not read.
fn no_reader() -> CallbackReader {
    CallbackReader {
        cb: Callback::None,
        self_0: core::ptr::null_mut::<Dict>(),
        buffer: Vec::new(),
        eof: false,
        buffered: false,
        fwd_err: false,
        type_0: core::ptr::null(),
    }
}

/// Spawns the server this client will talk to, returning its channel.
///
/// The server gets this process's own argv with `--embed` inserted, so
/// that every option the user typed reaches the process that will act on
/// it.
///
/// # Safety
///
/// `argv` must have `argc` valid C strings, and `exepath` must be one.
pub(crate) unsafe fn ui_client_start_server(
    exepath: *const c_char,
    argc: usize,
    argv: *mut *mut c_char,
) -> u64 {
    let args = unsafe { xmalloc((argc + 2) * size_of::<*mut c_char>()) }.cast::<*mut c_char>();
    unsafe { *args = xstrdup(*argv) };
    unsafe { *args.add(1) = xstrdup(c"--embed".as_ptr()) };
    for i in 1..argc {
        unsafe { *args.add(i + 1) = xstrdup(*argv.add(i)) };
    }
    unsafe { *args.add(argc + 1) = core::ptr::null_mut() };

    // The server's stderr is forwarded so that a Lua error at startup
    // is seen even though this process owns the terminal.
    let mut on_err = no_reader();
    on_err.fwd_err = true;

    let mut exit_status = 0;
    // The wide argument list is bound out here so that the region below is
    // the call and nothing else: a call spread over rustfmt's line width is
    // one unchecked line per argument.
    let no_exit_cb = Callback::None;
    let stdin_mode = kChannelStdinPipe;
    let no_term = core::ptr::null();
    let no_env = core::ptr::null_mut::<Dict>();
    let (no_w, no_h) = (0 as uint16_t, 0 as uint16_t);
    let status = &raw mut exit_status;
    let (out, err) = (no_reader(), on_err);
    // SAFETY: `args` is the NULL-terminated vector just built and `exepath`
    // is the caller's C string; `exit_status` is this frame's own.
    let channel = unsafe {
        channel_job_start(
            args, exepath, out, err, no_exit_cb, false, true, true, true, stdin_mode, no_term,
            no_w, no_h, no_env, status,
        )
    };
    if channel.is_null() {
        return 0;
    }
    if ui_client_forward_stdin.get() {
        // The user piped something in, which the server will read from
        // the descriptor `dup` lands on; this process needs slot 0 for
        // the terminal.
        let keep = if stderr_isatty.get() {
            STDERR_FILENO
        } else {
            STDOUT_FILENO
        };
        // SAFETY: both are descriptors this process owns.
        unsafe {
            close(0);
            dup(keep);
        };
    }
    unsafe { (*channel).id }
}

/// Attaches this client to the server as a UI, remembering the terminal
/// description for a later [`ui_client_reattach`].
///
/// # Safety
///
/// `term` must be null or a valid C string, and a channel must be set.
pub(crate) unsafe fn ui_client_attach(width: c_int, height: c_int, term: *mut c_char, rgb: bool) {
    tui_width.set(width);
    tui_height.set(height);
    tui_rgb.set(rgb);
    // A copy, not the pointer: `term` names the TUI's own arena, which
    // stopping the terminal frees, and a re-attach can run after that.
    // SAFETY: the caller's promise.
    tui_term.set(unsafe { cstr_to_string(term) });
    ui_client_reattach();
}

/// Attaches with the terminal description the last attach remembered.
pub(crate) fn ui_client_reattach() {
    let (width, height, term, rgb) = (
        tui_width.get(),
        tui_height.get(),
        tui_term.with(String_0::clone),
        tui_rgb.get(),
    );
    let mut opts = DictBuf::<8>::new();
    opts.insert(c"rgb", Object::boolean(rgb));
    // A TUI is always on the modern protocol and always owns its own
    // palette, so it never has to be told the fallback colours.
    opts.insert(c"ext_linegrid", Object::boolean(true));
    opts.insert(c"ext_termcolors", Object::boolean(true));
    if !term.is_null() {
        opts.insert(c"term_name", Object::string(term));
    }
    opts.insert(
        c"term_colors",
        Object::integer(Integer::from(t_colors.get())),
    );
    opts.insert(c"stdin_tty", Object::boolean(stdin_isatty.get()));
    opts.insert(c"stdout_tty", Object::boolean(stdout_isatty.get()));
    if ui_client_forward_stdin.get() {
        opts.insert(c"stdin_fd", Object::integer(FORWARDED_STDIN_FD));
        // Only the first attach forwards it; a re-attach after
        // `:restart` has nothing left to hand over.
        ui_client_forward_stdin.set(false);
    }

    let mut args = ArrayBuf::<3>::new();
    args.push(Object::integer(Integer::from(width)));
    args.push(Object::integer(Integer::from(height)));
    args.push(opts.object());
    unsafe {
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_ui_attach".as_ptr(),
            args.array(),
        )
    };
    ui_client_attached.set(true);
    log_startup_step(c"nvim_ui_attach");

    // Tell the server who is drawing for it, which is what
    // `nvim_get_chan_info` reports and what `:checkhealth` reads.
    let mut info = DictBuf::<3>::new();
    info.insert(
        c"website",
        Object::string(unsafe { cstr_to_string(c"https://neovim.io".as_ptr()) }),
    );
    info.insert(
        c"license",
        Object::string(unsafe { cstr_to_string(c"Apache 2".as_ptr()) }),
    );
    info.insert(c"pid", Object::integer(os_get_pid()));

    let mut client = ArrayBuf::<5>::new();
    client.push(Object::string(unsafe {
        cstr_to_string(c"nvim-tui".as_ptr())
    }));
    client.push(Object::dict(api_version()));
    client.push(Object::string(unsafe { cstr_to_string(c"ui".as_ptr()) }));
    // A UI exposes no methods of its own.
    client.push(Object::array(Array::EMPTY));
    client.push(info.object());
    unsafe {
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_set_client_info".as_ptr(),
            client.array(),
        )
    };
    log_startup_step(c"nvim_set_client_info");
}

/// This binary's own `version` dict, out of its API metadata.
fn api_version() -> ApiDict {
    let metadata = api_metadata();
    let metadata = metadata.as_dict().expect("API metadata is a dict");
    assert!(!metadata.is_empty(), "API metadata is empty");
    for entry in metadata {
        // SAFETY: a dictionary key is a NUL-terminated string.
        if unsafe { strequal(entry.key.as_ptr(), c"version".as_ptr()) } {
            // Copied: the metadata this walked is released on the way out.
            return entry
                .value
                .as_dict()
                .expect("API `version` is a dict")
                .clone();
        }
    }
    panic!("API metadata has no version");
}

/// Notes in `--startuptime` that `step` has been sent.
fn log_startup_step(step: &'static CStr) {
    if !time_fd.get().is_null() {
        unsafe { time_msg(step.as_ptr(), core::ptr::null::<ProfTime>()) };
    }
}

/// Detaches from the server without stopping this process.
pub(crate) fn ui_client_detach() {
    let (id, name, no_args) = (
        ui_client_channel_id.get(),
        c"nvim_ui_detach".as_ptr(),
        Array::EMPTY,
    );
    // SAFETY: the caller's promise -- a channel is set -- and the event
    // carries no arguments to own.
    unsafe { rpc_send_event(id, name, no_args) };
    ui_client_attached.set(false);
}

/// Starts the TUI, attaches, and runs until the process exits.
pub(crate) fn ui_client_run() -> ! {
    // Published before the loop turns: a callback that runs during
    // `tui_wait_ready` can reach `ui_client_stop`, which needs it.
    tui.set(unsafe { tui_start() });
    let started = unsafe { tui_wait_ready(tui.get()) };
    // SAFETY: the terminal reported all four, and the channel is set.
    unsafe { ui_client_attach(started.width, started.height, started.term, started.rgb) };

    // The test harness waits for a line in the log before it starts
    // driving the terminal, so that it is not racing startup.
    if unsafe { os_env_exists(c"__NVIM_TEST_LOG".as_ptr(), true) } {
        logmsg!(
            LOGLVL_ERR,
            c"ui_client_run",
            line!() as c_int,
            "test log message"
        );
    }
    time_finish();

    // Never returns: the client exits from a callback, either because
    // the server said so or because the channel closed.
    loop {
        unsafe { process_events(main_loop.ptr(), (*main_loop.ptr()).events, -1) };
    }
}

/// Stops drawing, on the way out.
pub(crate) fn ui_client_stop() {
    ui_client_attached.set(false);
    if !unsafe { tui_is_stopped(tui.get()) } {
        unsafe { tui_stop(tui.get()) };
    }
}

/// Reports a new terminal size to the server, and remembers it for a
/// re-attach.
pub(crate) fn ui_client_set_size(width: c_int, height: c_int) {
    if ui_client_attached.get() {
        let mut args = ArrayBuf::<2>::new();
        args.push(Object::integer(Integer::from(width)));
        args.push(Object::integer(Integer::from(height)));
        unsafe {
            rpc_send_event(
                ui_client_channel_id.get(),
                c"nvim_ui_try_resize".as_ptr(),
                args.array(),
            )
        };
    }
    tui_width.set(width);
    tui_height.set(height);
}

/// The wrapper for the `redraw` event named by `name`, or an empty handler.
///
/// Called by the msgpack decoder for every event in a batch, so the lookup
/// is on the hot path; a `match` on the bytes compiles to the same
/// length-then-prefix dispatch the generated perfect hash did.
///
/// # Safety
///
/// `name` must have `name_len` readable bytes.
pub(crate) unsafe fn ui_client_get_redraw_handler(
    name: *const c_char,
    name_len: usize,
) -> UIClientHandler {
    let name = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), name_len) };
    EVENT_HANDLERS
        .iter()
        .find(|handler| handler.event.to_bytes() == name)
        .map_or(
            UIClientHandler {
                name: core::ptr::null(),
                fn_0: None,
            },
            |handler| UIClientHandler {
                name: handler.event.as_ptr(),
                fn_0: Some(handler.wrapper),
            },
        )
}

/// Refuses a `redraw` sent as a request rather than a notification.
///
/// Its address is also what the decoder compares against to recognise the
/// `redraw` method at all, which is why this exists rather than a null
/// entry in the dispatch table.
///
/// # Safety
///
/// Takes the dispatcher's contract, which it uses none of.
pub(crate) unsafe fn handle_ui_client_redraw(
    _channel_id: u64,
    _args: Array,
    _arena: *mut Arena,
) -> Result<Object, Error> {
    Err(Error::validation(c"'redraw' cannot be sent as a request"))
}

/// One event's name and the wrapper that decodes it.
struct Handler {
    event: &'static CStr,
    wrapper: unsafe fn(Array),
}

/// Every `redraw` event this client understands.
///
/// Ordered by name length and then by name, which is the order the
/// generated perfect hash imposed; nothing depends on it now, but it keeps
/// the table diffable against upstream's.
static EVENT_HANDLERS: [Handler; 27] = {
    macro_rules! handlers {
        ($($event:literal => $wrapper:ident),* $(,)?) => {
            [$(Handler { event: $event, wrapper: $wrapper }),*]
        };
    }
    handlers! {
        c"bell" => ui_client_event_bell,
        c"chdir" => ui_client_event_chdir,
        c"flush" => ui_client_event_flush,
        c"connect" => ui_client_event_connect,
        c"restart" => ui_client_event_restart,
        c"suspend" => ui_client_event_suspend,
        c"ui_send" => ui_client_event_ui_send,
        c"mouse_on" => ui_client_event_mouse_on,
        c"set_icon" => ui_client_event_set_icon,
        c"busy_stop" => ui_client_event_busy_stop,
        c"grid_line" => ui_client_event_grid_line,
        c"mouse_off" => ui_client_event_mouse_off,
        c"set_title" => ui_client_event_set_title,
        c"busy_start" => ui_client_event_busy_start,
        c"error_exit" => ui_client_event_error_exit,
        c"grid_clear" => ui_client_event_grid_clear,
        c"option_set" => ui_client_event_option_set,
        c"screenshot" => ui_client_event_screenshot,
        c"mode_change" => ui_client_event_mode_change,
        c"update_menu" => ui_client_event_update_menu,
        c"visual_bell" => ui_client_event_visual_bell,
        c"grid_resize" => ui_client_event_grid_resize,
        c"grid_scroll" => ui_client_event_grid_scroll,
        c"mode_info_set" => ui_client_event_mode_info_set,
        c"hl_attr_define" => ui_client_event_hl_attr_define,
        c"grid_cursor_goto" => ui_client_event_grid_cursor_goto,
        c"default_colors_set" => ui_client_event_default_colors_set,
    }
};

/// The `index`th argument of `args`, if it is there and has type `want`.
///
/// `want` of `None` accepts anything, for the one event whose argument is
/// declared as an untyped `Object`.
///
/// The value is **taken** out of the array: an event's arguments are handed
/// on to the sink, which owns what it is given.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
unsafe fn take_arg(args: &mut Array, index: usize, want: Option<ObjectType>) -> Option<Object> {
    let value = args.get_mut(index)?;
    match want {
        Some(ty) if value.kind() != ty => None,
        _ => Some(value.take()),
    }
}

/// Notes that an event arrived with arguments it could not have.
///
/// The server and the client are the same build, so this means a corrupt
/// stream rather than a version mismatch; there is nothing to do about it
/// but skip the event.
fn bad_event(event: &'static CStr, wrapper: &'static CStr) {
    let name = msg_cstr(event);
    logmsg!(
        LOGLVL_ERR,
        wrapper,
        line!() as c_int,
        "Error handling ui event '{name}'"
    );
}

/// A `&'static CStr` from a name the macros have as a `&str`.
macro_rules! cstr {
    ($text:expr) => {
        match CStr::from_bytes_with_nul(concat!($text, "\0").as_bytes()) {
            Ok(name) => name,
            Err(_) => panic!("a Rust identifier holds no NUL"),
        }
    };
}

/// The [`ObjectType`] an argument declared as `$ty` must arrive as.
macro_rules! tag {
    (Boolean) => {
        Some(kObjectTypeBoolean)
    };
    (Integer) => {
        Some(kObjectTypeInteger)
    };
    (String_0) => {
        Some(kObjectTypeString)
    };
    (Array) => {
        Some(kObjectTypeArray)
    };
    (Dict) => {
        Some(kObjectTypeDict)
    };
    (Object) => {
        None
    };
}

/// The payload of a checked argument, as the declared type. [`arg`] has
/// already compared the tag against [`tag`], so the variant is the one the
/// declaration names.
macro_rules! payload {
    (Boolean, $v:expr) => {
        $v.as_boolean().expect("`take_arg` checked the tag")
    };
    (Integer, $v:expr) => {
        $v.as_integer().expect("`take_arg` checked the tag")
    };
    (String_0, $v:expr) => {
        $v.into_string().expect("`take_arg` checked the tag")
    };
    (Array, $v:expr) => {
        $v.into_array().expect("`take_arg` checked the tag")
    };
    (Dict, $v:expr) => {
        $v.into_dict().expect("`arg` checked the tag")
    };
    (Object, $v:expr) => {
        $v
    };
}

/// Defines a wrapper that checks an event's arguments and forwards them.
///
/// Both names are spelled out: the event as it arrives on the wire, and
/// the TUI function it ends at. Anything that needs to do more than
/// forward is written out below instead.
macro_rules! forward {
    ($(
        $(#[$attr:meta])*
        fn $wrapper:ident($($arg:ident: $ty:ident),* $(,)?) => $sink:ident, $event:expr;
    )*) => {$(
        $(#[$attr])*
        ///
        /// # Safety
        ///
        /// `args` must be the array the decoder produced for this event.
        // `pub(crate)`, not `pub`: the module is not reachable from outside
        // the crate, so a `pub` here is `unreachable_pub` at the macro's own
        // line, once for all of its expansions.
        pub(crate) unsafe fn $wrapper(mut args: Array) {
            // An event may take no arguments, in which case neither of
            // these is read; borrowing them keeps that case warning-free
            // without an allow on every wrapper.
            let mut index = 0usize;
            let _ = (&mut args, &mut index);
            $(
                let Some($arg) = (unsafe { take_arg(&mut args, index, tag!($ty)) }) else {
                    return bad_event($event, cstr!(stringify!($wrapper)));
                };
                index += 1;
            )*
            // The last repetition's `index += 1` has no reader; this is it,
            // rather than an `unused_assignments` allow per wrapper.
            let _ = index;
            unsafe { $sink(&mut *tui.get() $(, payload!($ty, $arg))*) };
        }
    )*};
}

forward! {
    /// The cursor shapes and blink timings for each mode.
    fn ui_client_event_mode_info_set(enabled: Boolean, cursor_styles: Array)
        => tui_mode_info_set, c"mode_info_set";
    fn ui_client_event_update_menu() => tui_update_menu, c"update_menu";
    fn ui_client_event_busy_start() => tui_busy_start, c"busy_start";
    fn ui_client_event_busy_stop() => tui_busy_stop, c"busy_stop";
    fn ui_client_event_mouse_on() => tui_mouse_on, c"mouse_on";
    fn ui_client_event_mouse_off() => tui_mouse_off, c"mouse_off";
    fn ui_client_event_mode_change(mode: String_0, mode_idx: Integer)
        => tui_mode_change, c"mode_change";
    fn ui_client_event_bell() => tui_bell, c"bell";
    fn ui_client_event_visual_bell() => tui_visual_bell, c"visual_bell";
    /// Everything since the last flush may now be shown.
    fn ui_client_event_flush() => tui_flush, c"flush";
    fn ui_client_event_suspend() => tui_suspend, c"suspend";
    fn ui_client_event_set_title(title: String_0) => tui_set_title, c"set_title";
    fn ui_client_event_set_icon(icon: String_0) => tui_set_icon, c"set_icon";
    /// Write the screen to a file, for the screen tests.
    fn ui_client_event_screenshot(path: String_0) => tui_screenshot, c"screenshot";
    /// A UI option changed. Its value keeps whatever type the option has,
    /// so it is the one argument that is not type-checked here.
    fn ui_client_event_option_set(name: String_0, value: Object) => tui_option_set, c"option_set";
    /// The server's working directory changed.
    fn ui_client_event_chdir(path: String_0) => tui_chdir, c"chdir";
    /// A control sequence the server produced, to be written through.
    fn ui_client_event_ui_send(content: String_0) => tui_ui_send, c"ui_send";
    /// The colours `Normal` resolves to.
    fn ui_client_event_default_colors_set(
        rgb_fg: Integer,
        rgb_bg: Integer,
        rgb_sp: Integer,
        cterm_fg: Integer,
        cterm_bg: Integer,
    ) => tui_default_colors_set, c"default_colors_set";
    fn ui_client_event_grid_clear(grid: Integer) => tui_grid_clear, c"grid_clear";
    fn ui_client_event_grid_cursor_goto(grid: Integer, row: Integer, col: Integer)
        => tui_grid_cursor_goto, c"grid_cursor_goto";
    fn ui_client_event_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    ) => tui_grid_scroll, c"grid_scroll";
}

/// Resizes a grid, and the buffers a `grid_line` is decoded into.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_grid_resize(mut args: Array) {
    let (Some(grid), Some(width), Some(height)) = (
        unsafe { take_arg(&mut args, 0, Some(kObjectTypeInteger)) },
        unsafe { take_arg(&mut args, 1, Some(kObjectTypeInteger)) },
        unsafe { take_arg(&mut args, 2, Some(kObjectTypeInteger)) },
    ) else {
        return bad_event(c"grid_resize", c"ui_client_event_grid_resize");
    };
    let expect = "`arg` checked the tag";
    let (grid, width, height) = (
        grid.as_integer().expect(expect),
        width.as_integer().expect(expect),
        height.as_integer().expect(expect),
    );
    tui_grid_resize(unsafe { &mut *tui.get() }, grid, width, height);

    // The decoder writes cells straight into these rather than
    // building an array, so they have to hold the widest grid.
    Unpacker::widen_grid_line_buf(width as usize);
}

/// Never called: the decoder recognises `grid_line` by this function's
/// address and decodes the cells itself, ending at
/// [`ui_client_event_raw_line`].
pub(crate) fn ui_client_event_grid_line(_args: Array) {
    unreachable!("grid_line is decoded by the unpacker, not dispatched");
}

/// Paints the cells the decoder wrote into the shared buffers.
///
/// # Safety
///
/// `g` must be the decoder's event, and the shared buffers must hold the
/// cells it counted.
pub(crate) unsafe fn ui_client_event_raw_line(g: *mut GridLineEvent) {
    let [grid, row, startcol] = unsafe { (*g).args };
    let endcol = Integer::from(startcol + unsafe { (*g).coloff });
    let clearcol = endcol + Integer::from(unsafe { (*g).clear_width });
    let flags = if unsafe { (*g).wrap } {
        kLineFlagWrap
    } else {
        0
    };
    let (chars, attrs) = Unpacker::grid_line_cells();
    let (grid, row, startcol) = (
        Integer::from(grid),
        Integer::from(row),
        Integer::from(startcol),
    );
    // SAFETY: the decoder filled `g` for this event, and the TUI is started.
    unsafe {
        let (t, attr) = (&mut *tui.get(), Integer::from((*g).cur_attr));
        tui_raw_line(
            t, grid, row, startcol, endcol, clearcol, attr, flags, chars, attrs,
        )
    };
}

/// Announces a highlight attribute id, converting both dicts back into the
/// attribute entry the TUI keeps.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_hl_attr_define(mut args: Array) {
    let (Some(id), Some(rgb), Some(cterm), Some(info)) = (
        unsafe { take_arg(&mut args, 0, Some(kObjectTypeInteger)) },
        unsafe { take_arg(&mut args, 1, Some(kObjectTypeDict)) },
        unsafe { take_arg(&mut args, 2, Some(kObjectTypeDict)) },
        unsafe { take_arg(&mut args, 3, Some(kObjectTypeArray)) },
    ) else {
        return bad_event(c"hl_attr_define", c"ui_client_event_hl_attr_define");
    };
    let expect = "`arg` checked the tag";
    let (id, rgb, cterm, info) = (
        id.as_integer().expect(expect),
        rgb.into_dict().expect(expect),
        cterm.into_dict().expect(expect),
        info.into_array().expect(expect),
    );
    tui_hl_attr_define(
        unsafe { &mut *tui.get() },
        id,
        unsafe { dict_to_hlattrs(&rgb, true) },
        unsafe { dict_to_hlattrs(&cterm, false) },
        info,
    );
}

/// The attribute entry `d` describes, as the server's `hl_attr_define`
/// spelled it.
///
/// # Safety
///
/// `d` must be a valid dict.
unsafe fn dict_to_hlattrs(d: &ApiDict, rgb: bool) -> HlAttrs {
    // Every key unset, which is the state `api_dict_to_keydict` fills in
    // over -- and the only one that lets `dict2hlattrs` tell "the UI said
    // `bold = false`" from "the UI said nothing about `bold`".
    let mut dict = KeyDict_highlight::default();
    if unsafe {
        api_dict_to_keydict(
            (&raw mut dict).cast::<c_void>(),
            Some(key_dict_highlight_get_field),
            d.clone(),
        )
    }
    .is_err()
    {
        return HLATTRS_INIT;
    }
    let Ok(mut attrs) = (unsafe { dict2hlattrs(&dict, rgb, None, None) }) else {
        return HLATTRS_INIT;
    };
    // A URL is not an attribute the terminal understands; the TUI
    // interns it and the entry keeps the index.
    if let Some(url) = &dict.url {
        attrs.url = unsafe { tui_add_url(&mut *tui.get(), url.data()) };
    }
    attrs
}

/// Records the exit status the server asked for; the client exits when the
/// channel closes.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_error_exit(mut args: Array) {
    let Some(status) = (unsafe { take_arg(&mut args, 0, Some(kObjectTypeInteger)) }) else {
        return bad_event(c"error_exit", c"ui_client_event_error_exit");
    };
    let status = status.as_integer().expect("`arg` checked the tag");
    ui_client_error_exit.set(status as c_int);
}

/// Moves this client to the server listening at the given address.
///
/// Sent by `:restart`, whose new server is a different process; the
/// connect is queued rather than done here because this runs from inside
/// the decode of the old server's stream.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_connect(mut args: Array) {
    let Some(address) = (unsafe { take_arg(&mut args, 0, Some(kObjectTypeString)) }) else {
        return bad_event(c"connect", c"ui_client_event_connect");
    };
    let address = address.as_string().expect("`arg` checked the tag");
    let server_addr = unsafe { xmemdupz(address.data().cast(), address.len()).cast::<c_char>() };
    unsafe {
        multiqueue_put_event(
            (*main_loop.ptr()).fast_events,
            Event::new(Some(channel_connect_event), [server_addr.cast::<c_void>()]),
        )
    };
    // No channel until the queued connect runs; anything sent meanwhile
    // would go to the server that is going away.
    ui_client_channel_id.set(u64::MAX);
}

/// Connects to the address `ui_client_event_connect` queued, and re-attaches.
///
/// # Safety
///
/// `argv[0]` must be an owned C string.
unsafe extern "C" fn channel_connect_event(argv: *mut *mut c_void) {
    let server_addr = unsafe { *argv }.cast::<c_char>();
    let mut err = c"".as_ptr();
    let is_tcp = socket_address_is_tcp(unsafe { CStr::from_ptr(server_addr) });
    let chan = unsafe {
        channel_connect(
            is_tcp,
            server_addr,
            true,
            no_reader(),
            UI_CONNECT_TIMEOUT_MS,
            &raw mut err,
        )
    };
    if !unsafe { strequal(err, c"".as_ptr()) } {
        let line = line!() as c_int;
        logmsg!(
            LOGLVL_ERR,
            c"channel_connect_event",
            line,
            "Cannot connect to server {}: {}",
            unsafe { c_str(server_addr) },
            unsafe { c_str(err) }
        );
        unsafe { xfree(server_addr.cast()) };
        ui_client_exit_status.set(1);
        os_exit(1);
    }
    ui_client_channel_id.set(chan);
    ui_client_reattach();
    let line = line!() as c_int;
    logmsg!(
        LOGLVL_INF,
        c"channel_connect_event",
        line,
        "Connected to server {} on channel {}",
        unsafe { c_str(server_addr) },
        chan
    );
    unsafe { xfree(server_addr.cast()) };
}

/// The copied `restart` event arguments, owned.
///
/// Deliberately not `Copy`: `restart_args.get()` handed every caller a
/// second owner of the same `items` pointer, so the cell and the caller both
/// believed they had to free it. Owning them here makes the free this type's
/// `Drop` and the one genuine move `GlobalCell::take`.
struct RestartArgs(Array);

impl RestartArgs {
    const EMPTY: RestartArgs = RestartArgs(Array::EMPTY);
}

impl Default for RestartArgs {
    fn default() -> Self {
        RestartArgs::EMPTY
    }
}

/// The address the restarted server will listen on, kept until the old
/// server's channel has finished closing.
static restart_args: GlobalCell<RestartArgs> = GlobalCell::new(RestartArgs::EMPTY);
static restart_pending: GlobalCell<bool> = GlobalCell::new(false);

/// Remembers where to reconnect after `:restart`.
///
/// The arguments are copied because they live in the decoder's arena,
/// which is reused as soon as this returns.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_restart(args: Array) {
    // `set` drops what the cell held, which frees the previous copy.
    // SAFETY: the caller's promise -- the decoder's array for this event.
    let copied = args.clone();
    restart_args.set(RestartArgs(copied));
    restart_pending.set(true);
}

/// Connects to the restarted server, if one was announced.
///
/// Called once the old channel is gone, which is why this is separate from
/// [`ui_client_event_restart`].
pub(crate) fn ui_client_attach_to_restarted_server() {
    if !restart_pending.get() {
        return;
    }
    restart_pending.set(false);
    // The arguments move out here, so the cell has nothing left to free and
    // dropping `args` at the end of the scope is the only free.
    let mut args = restart_args.take();
    let address = unsafe { take_arg(&mut args.0, 0, Some(kObjectTypeString)) };
    match address {
        None => bad_event(c"restart", c"ui_client_attach_to_restarted_server"),
        Some(address) => {
            let listen_addr = address
                .as_string()
                .expect("`take_arg` checked the tag")
                .data();
            let mut err = c"".as_ptr();
            let chan_id = unsafe {
                channel_connect(
                    socket_address_is_tcp(CStr::from_ptr(listen_addr)),
                    listen_addr,
                    true,
                    no_reader(),
                    UI_CONNECT_TIMEOUT_MS,
                    &raw mut err,
                )
            };
            if !unsafe { strequal(err, c"".as_ptr()) } {
                let line = line!() as c_int;
                logmsg!(
                    LOGLVL_ERR,
                    c"ui_client_attach_to_restarted_server",
                    line,
                    "cannot connect to server {}: {}",
                    unsafe { c_str(listen_addr) },
                    unsafe { c_str(err) }
                );
            } else {
                ui_client_channel_id.set(chan_id);
                ui_client_reattach();
                let line = line!() as c_int;
                logmsg!(
                    LOGLVL_INF,
                    c"ui_client_attach_to_restarted_server",
                    line,
                    "restarted server address={} id={}",
                    unsafe { c_str(listen_addr) },
                    chan_id
                );
            }
        }
    }
}
