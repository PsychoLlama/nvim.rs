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

use crate::api::private::dispatch::key_dict_highlight_get_field;
use crate::api::private::helpers::{
    api_dict_to_keydict, api_free_array, api_metadata, api_set_error, copy_array, cstr_as_string,
};
use crate::channel::{channel_connect, channel_job_start};
use crate::eval::typval::kCallbackNone;
use crate::event::r#loop::process_events;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::socket::socket_address_is_tcp;
use crate::global_cell::GlobalCell;
use crate::highlight::{HLATTRS_INIT, dict2hlattrs};
use crate::log::{LOGLVL_ERR, LOGLVL_INF, logmsg_c};
use crate::main::{
    grid_line_buf_attr, grid_line_buf_char, grid_line_buf_size, main_loop, os_exit, stderr_isatty,
    stdin_isatty, stdout_isatty, t_colors, time_fd, ui_client_attached, ui_client_channel_id,
    ui_client_error_exit, ui_client_exit_status, ui_client_forward_stdin,
};
use crate::memory::{strequal, xfree, xmalloc, xmemdupz, xstrdup};
use crate::msgpack_rpc::channel::rpc_send_event;
use crate::os::env::{os_env_exists, os_get_pid};
use crate::profile::{time_finish, time_msg};
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
    Arena, Array, Callback, CallbackReader, Dict, Error, Event, GridLineEvent, HlAttrs, Integer,
    KeyDict_highlight, Object, ObjectType, TUIData, UIClientHandler, dict_T, garray_T,
    kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeDict,
    kObjectTypeInteger, kObjectTypeString, proftime_T, sattr_T, schar_T, uint16_t,
};
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
static tui_term: GlobalCell<*mut c_char> = GlobalCell::new(c"".as_ptr().cast_mut());
static tui_rgb: GlobalCell<bool> = GlobalCell::new(false);

/// A reader that discards what it is given, for the streams this client
/// does not read.
fn no_reader() -> CallbackReader {
    CallbackReader {
        cb: Callback {
            data: crate::types::Callback_data {
                funcref: core::ptr::null_mut(),
            },
            type_0: kCallbackNone,
        },
        self_0: core::ptr::null_mut::<dict_T>(),
        buffer: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 1,
            ga_data: core::ptr::null_mut(),
        },
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
    unsafe {
        let args = xmalloc((argc + 2) * size_of::<*mut c_char>()).cast::<*mut c_char>();
        *args = xstrdup(*argv);
        *args.add(1) = xstrdup(c"--embed".as_ptr());
        for i in 1..argc {
            *args.add(i + 1) = xstrdup(*argv.add(i));
        }
        *args.add(argc + 1) = core::ptr::null_mut();

        // The server's stderr is forwarded so that a Lua error at startup
        // is seen even though this process owns the terminal.
        let mut on_err = no_reader();
        on_err.fwd_err = true;

        let mut exit_status = 0;
        let channel = channel_job_start(
            args,
            exepath,
            no_reader(),
            on_err,
            Callback {
                data: crate::types::Callback_data {
                    funcref: core::ptr::null_mut(),
                },
                type_0: kCallbackNone,
            },
            false,
            true,
            true,
            true,
            kChannelStdinPipe,
            core::ptr::null(),
            0 as uint16_t,
            0 as uint16_t,
            core::ptr::null_mut::<dict_T>(),
            &raw mut exit_status,
        );
        if channel.is_null() {
            return 0;
        }
        if ui_client_forward_stdin.get() {
            // The user piped something in, which the server will read from
            // the descriptor `dup` lands on; this process needs slot 0 for
            // the terminal.
            close(0);
            dup(if stderr_isatty.get() {
                STDERR_FILENO
            } else {
                STDOUT_FILENO
            });
        }
        (*channel).id
    }
}

/// Attaches this client to the server as a UI.
///
/// # Safety
///
/// `term` must be null or a valid C string, and a channel must be set.
pub(crate) unsafe fn ui_client_attach(width: c_int, height: c_int, term: *mut c_char, rgb: bool) {
    unsafe {
        let mut opts = DictBuf::<8>::new();
        opts.insert(c"rgb", Object::boolean(rgb));
        // A TUI is always on the modern protocol and always owns its own
        // palette, so it never has to be told the fallback colours.
        opts.insert(c"ext_linegrid", Object::boolean(true));
        opts.insert(c"ext_termcolors", Object::boolean(true));
        if !term.is_null() {
            opts.insert(c"term_name", Object::string(cstr_as_string(term)));
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
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_ui_attach".as_ptr(),
            args.array(),
        );
        ui_client_attached.set(true);
        log_startup_step(c"nvim_ui_attach");

        // Tell the server who is drawing for it, which is what
        // `nvim_get_chan_info` reports and what `:checkhealth` reads.
        let mut info = DictBuf::<3>::new();
        info.insert(
            c"website",
            Object::string(cstr_as_string(c"https://neovim.io".as_ptr())),
        );
        info.insert(
            c"license",
            Object::string(cstr_as_string(c"Apache 2".as_ptr())),
        );
        info.insert(c"pid", Object::integer(os_get_pid()));

        let mut client = ArrayBuf::<5>::new();
        client.push(Object::string(cstr_as_string(c"nvim-tui".as_ptr())));
        client.push(Object::dict(api_version()));
        client.push(Object::string(cstr_as_string(c"ui".as_ptr())));
        // A UI exposes no methods of its own.
        client.push(Object::array(Array {
            size: 0,
            capacity: 0,
            items: core::ptr::null_mut(),
        }));
        client.push(info.object());
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_set_client_info".as_ptr(),
            client.array(),
        );
        log_startup_step(c"nvim_set_client_info");
    }
}

/// This binary's own `version` dict, out of its API metadata.
///
/// # Safety
///
/// The API metadata must be initialised.
unsafe fn api_version() -> Dict {
    unsafe {
        let metadata = api_metadata().data.dict;
        assert!(metadata.size > 0, "API metadata is empty");
        for i in 0..metadata.size {
            let entry = *metadata.items.add(i);
            if strequal(entry.key.data(), c"version".as_ptr()) {
                return entry.value.data.dict;
            }
        }
        panic!("API metadata has no version");
    }
}

/// Notes in `--startuptime` that `step` has been sent.
///
/// # Safety
///
/// `step` must outlive the log entry.
unsafe fn log_startup_step(step: &'static CStr) {
    if unsafe { !(*time_fd.ptr()).is_null() } {
        unsafe { time_msg(step.as_ptr(), core::ptr::null::<proftime_T>()) };
    }
}

/// Detaches from the server without stopping this process.
///
/// # Safety
///
/// A channel must be set.
pub(crate) unsafe fn ui_client_detach() {
    unsafe {
        rpc_send_event(
            ui_client_channel_id.get(),
            c"nvim_ui_detach".as_ptr(),
            Array {
                size: 0,
                capacity: 0,
                items: core::ptr::null_mut(),
            },
        )
    };
    ui_client_attached.set(false);
}

/// Starts the TUI, attaches, and runs until the process exits.
///
/// # Safety
///
/// Must be called once, on the main thread, with a server channel set.
pub(crate) unsafe fn ui_client_run() -> ! {
    unsafe {
        // Published before the loop turns: a callback that runs during
        // `tui_wait_ready` can reach `ui_client_stop`, which needs it.
        tui.set(tui_start());
        let started = tui_wait_ready(tui.get());
        tui_width.set(started.width);
        tui_height.set(started.height);
        tui_term.set(started.term);
        tui_rgb.set(started.rgb);
        ui_client_attach(started.width, started.height, started.term, started.rgb);

        // The test harness waits for a line in the log before it starts
        // driving the terminal, so that it is not racing startup.
        if os_env_exists(c"__NVIM_TEST_LOG".as_ptr(), true) {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"ui_client_run".as_ptr(),
                line!() as c_int,
                true,
                c"test log message".as_ptr(),
            );
        }
        time_finish();

        // Never returns: the client exits from a callback, either because
        // the server said so or because the channel closed.
        loop {
            process_events(main_loop.ptr(), (*main_loop.ptr()).events, -1);
        }
    }
}

/// Stops drawing, on the way out.
///
/// # Safety
///
/// The TUI must have been started.
pub(crate) unsafe fn ui_client_stop() {
    ui_client_attached.set(false);
    unsafe {
        if !tui_is_stopped(tui.get()) {
            tui_stop(tui.get());
        }
    }
}

/// Reports a new terminal size to the server, and remembers it for a
/// re-attach.
///
/// # Safety
///
/// A channel must be set if this client is attached.
pub(crate) unsafe fn ui_client_set_size(width: c_int, height: c_int) {
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
    _error: *mut Error,
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
/// `error` must be writable.
pub(crate) unsafe fn handle_ui_client_redraw(
    _channel_id: u64,
    _args: Array,
    _arena: *mut Arena,
    error: *mut Error,
) -> Object {
    unsafe {
        api_set_error(
            error,
            kErrorTypeValidation,
            c"'redraw' cannot be sent as a request".as_ptr(),
        )
    };
    Object::NIL
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
/// # Safety
///
/// `args` must be a valid array.
unsafe fn arg(args: Array, index: usize, want: Option<ObjectType>) -> Option<Object> {
    if index >= args.size {
        return None;
    }
    let value = unsafe { *args.items.add(index) };
    match want {
        Some(ty) if value.type_0 != ty => None,
        _ => Some(value),
    }
}

/// Notes that an event arrived with arguments it could not have.
///
/// The server and the client are the same build, so this means a corrupt
/// stream rather than a version mismatch; there is nothing to do about it
/// but skip the event.
fn bad_event(event: &'static CStr, wrapper: &'static CStr) {
    // SAFETY: both names are static C strings and the format takes one.
    unsafe {
        logmsg_c!(
            LOGLVL_ERR,
            core::ptr::null(),
            wrapper.as_ptr(),
            line!() as c_int,
            true,
            c"Error handling ui event '%s'".as_ptr(),
            event.as_ptr(),
        )
    };
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

/// The payload of a checked argument, as the declared type.
macro_rules! payload {
    (Boolean, $v:expr) => {
        $v.data.boolean
    };
    (Integer, $v:expr) => {
        $v.data.integer
    };
    (String_0, $v:expr) => {
        $v.data.string
    };
    (Array, $v:expr) => {
        $v.data.array
    };
    (Dict, $v:expr) => {
        $v.data.dict
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
        pub(crate) unsafe fn $wrapper(args: Array) {
            // An event may take no arguments, in which case neither of
            // these is read; borrowing them keeps that case warning-free
            // without an allow on every wrapper.
            let mut index = 0usize;
            let _ = (&args, &mut index);
            $(
                let Some($arg) = (unsafe { arg(args, index, tag!($ty)) }) else {
                    return bad_event($event, cstr!(stringify!($wrapper)));
                };
                index += 1;
            )*
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
pub(crate) unsafe fn ui_client_event_grid_resize(args: Array) {
    unsafe {
        let (Some(grid), Some(width), Some(height)) = (
            arg(args, 0, Some(kObjectTypeInteger)),
            arg(args, 1, Some(kObjectTypeInteger)),
            arg(args, 2, Some(kObjectTypeInteger)),
        ) else {
            return bad_event(c"grid_resize", c"ui_client_event_grid_resize");
        };
        let (grid, width, height) = (grid.data.integer, width.data.integer, height.data.integer);
        tui_grid_resize(&mut *tui.get(), grid, width, height);

        // The decoder writes cells straight into these rather than
        // building an array, so they have to hold the widest grid.
        if grid_line_buf_size.get() < width as usize {
            xfree(grid_line_buf_char.get().cast());
            xfree(grid_line_buf_attr.get().cast());
            grid_line_buf_size.set(width as usize);
            grid_line_buf_char
                .set(xmalloc(width as usize * size_of::<schar_T>()).cast::<schar_T>());
            grid_line_buf_attr
                .set(xmalloc(width as usize * size_of::<sattr_T>()).cast::<sattr_T>());
        }
    }
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
    unsafe {
        let [grid, row, startcol] = (*g).args;
        let endcol = Integer::from(startcol + (*g).coloff);
        let clearcol = endcol + Integer::from((*g).clear_width);
        let flags = if (*g).wrap { kLineFlagWrap } else { 0 };
        tui_raw_line(
            &mut *tui.get(),
            Integer::from(grid),
            Integer::from(row),
            Integer::from(startcol),
            endcol,
            clearcol,
            Integer::from((*g).cur_attr),
            flags,
            grid_line_buf_char.get().cast_const(),
            grid_line_buf_attr.get(),
        );
    }
}

/// Announces a highlight attribute id, converting both dicts back into the
/// attribute entry the TUI keeps.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_hl_attr_define(args: Array) {
    unsafe {
        let (Some(id), Some(rgb), Some(cterm), Some(info)) = (
            arg(args, 0, Some(kObjectTypeInteger)),
            arg(args, 1, Some(kObjectTypeDict)),
            arg(args, 2, Some(kObjectTypeDict)),
            arg(args, 3, Some(kObjectTypeArray)),
        ) else {
            return bad_event(c"hl_attr_define", c"ui_client_event_hl_attr_define");
        };
        tui_hl_attr_define(
            &mut *tui.get(),
            id.data.integer,
            dict_to_hlattrs(rgb.data.dict, true),
            dict_to_hlattrs(cterm.data.dict, false),
            info.data.array,
        );
    }
}

/// The attribute entry `d` describes, as the server's `hl_attr_define`
/// spelled it.
///
/// # Safety
///
/// `d` must be a valid dict.
unsafe fn dict_to_hlattrs(d: Dict, rgb: bool) -> HlAttrs {
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: core::ptr::null_mut(),
        };
        // Every field of a keyset is zero when nothing is set: the flags
        // are a bitmask, `kObjectTypeNil` is 0, and the rest are C layouts
        // whose null is their empty value.
        let mut dict: KeyDict_highlight = core::mem::zeroed();
        if !api_dict_to_keydict(
            (&raw mut dict).cast::<c_void>(),
            Some(key_dict_highlight_get_field),
            d,
            &raw mut err,
        ) {
            return HLATTRS_INIT;
        }
        let mut attrs = dict2hlattrs(&dict, rgb, None, None, &raw mut err);
        // A URL is not an attribute the terminal understands; the TUI
        // interns it and the entry keeps the index.
        if dict.is_set__highlight_ & (1 << KEYSET_OPTIDX_highlight__url) != 0 {
            attrs.url = tui_add_url(&mut *tui.get(), dict.url.data());
        }
        attrs
    }
}

/// Records the exit status the server asked for; the client exits when the
/// channel closes.
///
/// # Safety
///
/// `args` must be the array the decoder produced for this event.
pub(crate) unsafe fn ui_client_event_error_exit(args: Array) {
    let Some(status) = (unsafe { arg(args, 0, Some(kObjectTypeInteger)) }) else {
        return bad_event(c"error_exit", c"ui_client_event_error_exit");
    };
    ui_client_error_exit.set(unsafe { status.data.integer } as c_int);
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
pub(crate) unsafe fn ui_client_event_connect(args: Array) {
    let Some(address) = (unsafe { arg(args, 0, Some(kObjectTypeString)) }) else {
        return bad_event(c"connect", c"ui_client_event_connect");
    };
    let address = unsafe { address.data.string };
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
    unsafe {
        let server_addr = (*argv).cast::<c_char>();
        let mut err = c"".as_ptr();
        let is_tcp = socket_address_is_tcp(CStr::from_ptr(server_addr));
        let chan = channel_connect(
            is_tcp,
            server_addr,
            true,
            no_reader(),
            UI_CONNECT_TIMEOUT_MS,
            &raw mut err,
        );
        if !strequal(err, c"".as_ptr()) {
            logmsg_c!(
                LOGLVL_ERR,
                core::ptr::null(),
                c"channel_connect_event".as_ptr(),
                line!() as c_int,
                true,
                c"Cannot connect to server %s: %s".as_ptr(),
                server_addr,
                err,
            );
            xfree(server_addr.cast());
            ui_client_exit_status.set(1);
            os_exit(1);
        }
        ui_client_channel_id.set(chan);
        ui_client_attach(
            tui_width.get(),
            tui_height.get(),
            tui_term.get(),
            tui_rgb.get(),
        );
        logmsg_c!(
            LOGLVL_INF,
            core::ptr::null(),
            c"channel_connect_event".as_ptr(),
            line!() as c_int,
            true,
            c"Connected to server %s on channel %ld".as_ptr(),
            server_addr,
            chan,
        );
        xfree(server_addr.cast());
    }
}

/// The address the restarted server will listen on, kept until the old
/// server's channel has finished closing.
static restart_args: GlobalCell<Array> = GlobalCell::new(Array {
    size: 0,
    capacity: 0,
    items: core::ptr::null_mut(),
});
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
    unsafe {
        api_free_array(restart_args.get());
        restart_args.set(copy_array(args, core::ptr::null_mut::<Arena>()));
    }
    restart_pending.set(true);
}

/// Connects to the restarted server, if one was announced.
///
/// Called once the old channel is gone, which is why this is separate from
/// [`ui_client_event_restart`].
///
/// # Safety
///
/// Must run on the main thread with the old channel closed.
pub(crate) unsafe fn ui_client_attach_to_restarted_server() {
    if !restart_pending.get() {
        return;
    }
    restart_pending.set(false);
    unsafe {
        let args = restart_args.get();
        let address = arg(args, 0, Some(kObjectTypeString));
        match address {
            None => bad_event(c"restart", c"ui_client_attach_to_restarted_server"),
            Some(address) => {
                let listen_addr = address.data.string.data();
                let mut err = c"".as_ptr();
                let chan_id = channel_connect(
                    socket_address_is_tcp(CStr::from_ptr(listen_addr)),
                    listen_addr,
                    true,
                    no_reader(),
                    UI_CONNECT_TIMEOUT_MS,
                    &raw mut err,
                );
                if !strequal(err, c"".as_ptr()) {
                    logmsg_c!(
                        LOGLVL_ERR,
                        core::ptr::null(),
                        c"ui_client_attach_to_restarted_server".as_ptr(),
                        line!() as c_int,
                        true,
                        c"cannot connect to server %s: %s".as_ptr(),
                        listen_addr,
                        err,
                    );
                } else {
                    ui_client_channel_id.set(chan_id);
                    ui_client_attach(
                        tui_width.get(),
                        tui_height.get(),
                        tui_term.get(),
                        tui_rgb.get(),
                    );
                    logmsg_c!(
                        LOGLVL_INF,
                        core::ptr::null(),
                        c"ui_client_attach_to_restarted_server".as_ptr(),
                        line!() as c_int,
                        true,
                        c"restarted server address=%s id=%ld".as_ptr(),
                        listen_addr,
                        chan_id,
                    );
                }
            }
        }
        api_free_array(restart_args.get());
    }
    restart_args.set(Array {
        size: 0,
        capacity: 0,
        items: core::ptr::null_mut(),
    });
}

/// The bit `KeyDict_highlight` sets when the dict carried a `url`.
const KEYSET_OPTIDX_highlight__url: u32 = 5;
