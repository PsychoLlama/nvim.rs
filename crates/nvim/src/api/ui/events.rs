//! Turning a UI event into bytes on one UI's wire.
//!
//! Every event ends at [`push_call`], which appends a name and an argument
//! array to that UI's outgoing buffer. What varies is which name and which
//! arguments, and the only thing that makes it vary is `ext_linegrid`.
//!
//! A UI that asked for `ext_linegrid` gets the modern protocol: grid events
//! carry a grid handle and screen contents arrive as `grid_line` runs. One
//! that did not gets the original Vim-era protocol, where there is a single
//! implicit grid, a scroll region, a current highlight and a cursor the
//! server has to track on the UI's behalf — which is why the legacy paths
//! below read and write `client_row`/`client_col`/`hl_id` on the
//! [`RemoteUI`] and the linegrid ones do not.
//!
//! The legacy protocol has not been the default since 0.3 and exists for
//! UIs too old to be rebuilt. It is kept exactly, not improved.

#![deny(unsafe_op_in_unsafe_fn)]

use super::packer::{push_call, ui_flush_buf};
use crate::api::private::helpers::cstr_as_string;
use crate::highlight::{HLATTRS_DICT_SIZE, hlattrs2dict, syn_attr2entry};
use crate::main::p_bg;
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::ui::{kUILinegrid, kUITermColors};
use crate::types::{Array, Boolean, Integer, Object, RemoteUI, String_0, Window};
use core::ffi::{c_char, c_int};

/// Queues `name(args...)` on `ui`'s buffer, with the argument array built
/// on the stack.
macro_rules! send {
    ($ui:expr, $name:expr $(, $arg:expr)* $(,)?) => {{
        let mut args = ArrayBuf::<{ count!($($arg),*) }>::new();
        $( args.push($arg); )*
        unsafe { push_call($ui, $name, args.array()) };
    }};
}

/// The number of arguments given, as a `usize` constant.
macro_rules! count {
    () => { 0usize };
    ($head:expr $(, $tail:expr)*) => { 1usize + count!($($tail),*) };
}

/// The [`Object`] constructor for a declared argument type.
///
/// A `Window` is not an `Integer` on the wire even though both are `i64` in
/// the signature, so the mapping has to be by token and cannot be a trait.
macro_rules! wire {
    (Integer, $v:expr) => {
        Object::integer($v)
    };
    (Boolean, $v:expr) => {
        Object::boolean($v)
    };
    (String_0, $v:expr) => {
        Object::string($v)
    };
    (Array, $v:expr) => {
        Object::array($v)
    };
    (Window, $v:expr) => {
        Object::window($v)
    };
    (Object, $v:expr) => {
        $v
    };
}

/// Defines a serializer whose only variation is the name and arguments.
///
/// These are the events with no legacy spelling: a UI on the old protocol
/// is sent exactly the same call.
macro_rules! serialize {
    ($(
        $(#[$attr:meta])*
        fn $name:ident($($arg:ident: $ty:ident),* $(,)?) => $event:expr;
    )*) => {$(
        $(#[$attr])*
        ///
        /// # Safety
        ///
        /// `ui` must be live, and any string or array argument must stay
        /// valid until this returns.
        pub unsafe fn $name(ui: *mut RemoteUI $(, $arg: $ty)*) {
            send!(ui, $event $(, wire!($ty, $arg))*);
        }
    )*};
}

serialize! {
    /// The cursor shapes and blink timings for each mode.
    fn remote_ui_mode_info_set(enabled: Boolean, cursor_styles: Array) => c"mode_info_set";
    /// The menu changed; a UI drawing its own should re-read it.
    fn remote_ui_update_menu() => c"update_menu";
    /// The editor is busy and the cursor should not be drawn.
    fn remote_ui_busy_start() => c"busy_start";
    /// It is no longer busy.
    fn remote_ui_busy_stop() => c"busy_stop";
    /// Start reporting mouse events.
    fn remote_ui_mouse_on() => c"mouse_on";
    /// Stop.
    fn remote_ui_mouse_off() => c"mouse_off";
    /// The mode changed to the `mode_idx`th entry of the last
    /// `mode_info_set`.
    fn remote_ui_mode_change(mode: String_0, mode_idx: Integer) => c"mode_change";
    fn remote_ui_bell() => c"bell";
    fn remote_ui_visual_bell() => c"visual_bell";
    /// The user asked to suspend; a UI that can should background itself.
    fn remote_ui_suspend() => c"suspend";
    fn remote_ui_set_title(title: String_0) => c"set_title";
    fn remote_ui_set_icon(icon: String_0) => c"set_icon";
    /// Write a screenshot to `path`, for the screen tests.
    fn remote_ui_screenshot(path: String_0) => c"screenshot";
    /// A UI option changed. `value` keeps whatever type the option has.
    fn remote_ui_option_set(name: String_0, value: Object) => c"option_set";
    /// The server's working directory changed, so that a UI can resolve the
    /// relative paths the server sends.
    fn remote_ui_chdir(path: String_0) => c"chdir";
    /// A highlight group's id, for UIs that want to name what they draw.
    fn remote_ui_hl_group_set(name: String_0, id: Integer) => c"hl_group_set";
    /// Where the message grid sits, for UIs compositing it themselves.
    fn remote_ui_msg_set_pos(
        grid: Integer,
        row: Integer,
        scrolled: Boolean,
        sep_char: String_0,
        zindex: Integer,
        compindex: Integer,
    ) => c"msg_set_pos";
    /// What part of a buffer a window is showing.
    #[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
    fn remote_ui_win_viewport(
        grid: Integer,
        win: Window,
        topline: Integer,
        botline: Integer,
        curline: Integer,
        curcol: Integer,
        line_count: Integer,
        scroll_delta: Integer,
    ) => c"win_viewport";
    /// The margins inside that viewport that are not buffer text.
    fn remote_ui_win_viewport_margins(
        grid: Integer,
        win: Window,
        top: Integer,
        bottom: Integer,
        left: Integer,
        right: Integer,
    ) => c"win_viewport_margins";
    /// The editor is exiting because of an error, with `status`.
    fn remote_ui_error_exit(status: Integer) => c"error_exit";
}

/// Nothing. A UI is stopped by closing its channel, not by an event.
///
/// # Safety
///
/// Trivially. It exists so that the sink table has something to name.
pub unsafe fn remote_ui_stop(_ui: *mut RemoteUI) {}

/// Whether `ui` speaks the modern grid protocol.
///
/// # Safety
///
/// `ui` must be live.
pub(super) unsafe fn linegrid(ui: *mut RemoteUI) -> bool {
    unsafe { (*ui).ui_ext[kUILinegrid as usize] }
}

/// Clears a grid.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_grid_clear(ui: *mut RemoteUI, grid: Integer) {
    // The legacy protocol has one grid, so it takes no handle.
    if unsafe { linegrid(ui) } {
        send!(ui, c"grid_clear", Object::integer(grid));
    } else {
        send!(ui, c"clear");
    }
}

/// Resizes a grid.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_grid_resize(
    ui: *mut RemoteUI,
    grid: Integer,
    width: Integer,
    height: Integer,
) {
    if unsafe { linegrid(ui) } {
        send!(
            ui,
            c"grid_resize",
            Object::integer(grid),
            Object::integer(width),
            Object::integer(height),
        );
    } else {
        // A legacy resize leaves the cursor somewhere the server cannot
        // predict, so the tracked column is invalidated rather than
        // guessed: the next `put` will move it explicitly.
        unsafe { (*ui).client_col = -1 };
        send!(
            ui,
            c"resize",
            Object::integer(width),
            Object::integer(height),
        );
    }
}

/// Moves the cursor.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_grid_cursor_goto(
    ui: *mut RemoteUI,
    grid: Integer,
    row: Integer,
    col: Integer,
) {
    if unsafe { linegrid(ui) } {
        send!(
            ui,
            c"grid_cursor_goto",
            Object::integer(grid),
            Object::integer(row),
            Object::integer(col),
        );
        return;
    }
    // On the legacy protocol the cursor is where drawing happens, so the
    // intended position is remembered and re-sent at flush: putting a cell
    // moves it anyway, and a move now would only be undone.
    unsafe {
        (*ui).cursor_row = row;
        (*ui).cursor_col = col;
        remote_ui_cursor_goto(ui, row, col);
    }
}

/// Moves the legacy protocol's single cursor, if it is not already there.
///
/// # Safety
///
/// `ui` must be live.
pub(super) unsafe fn remote_ui_cursor_goto(ui: *mut RemoteUI, row: Integer, col: Integer) {
    unsafe {
        if (*ui).client_row == row && (*ui).client_col == col {
            return;
        }
        (*ui).client_row = row;
        (*ui).client_col = col;
    }
    send!(
        ui,
        c"cursor_goto",
        Object::integer(row),
        Object::integer(col),
    );
}

/// Writes one cell at the legacy cursor, which then advances.
///
/// # Safety
///
/// `ui` must be live and `cell` a valid C string.
pub(super) unsafe fn remote_ui_put(ui: *mut RemoteUI, cell: *const c_char) {
    unsafe { (*ui).client_col += 1 };
    send!(ui, c"put", Object::string(unsafe { cstr_as_string(cell) }));
}

/// Sets the highlight subsequent [`remote_ui_put`]s draw with.
///
/// # Safety
///
/// `ui` must be live and `id` a resolved highlight attribute id.
pub(super) unsafe fn remote_ui_highlight_set(ui: *mut RemoteUI, id: c_int) {
    // The legacy protocol has no attribute table, so every change resends
    // the whole set; skipping the no-op is the only compression there is.
    unsafe {
        if (*ui).hl_id == id {
            return;
        }
        (*ui).hl_id = id;
    }

    let mut buf = DictBuf::<HLATTRS_DICT_SIZE>::new();
    let mut dict = buf.dict();
    unsafe {
        hlattrs2dict(&mut dict, None, syn_attr2entry(id), (*ui).rgb, false);
    }
    send!(ui, c"highlight_set", Object::dict(dict));
}

/// Scrolls a rectangle of a grid.
///
/// # Safety
///
/// `ui` must be live.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn remote_ui_grid_scroll(
    ui: *mut RemoteUI,
    grid: Integer,
    top: Integer,
    bot: Integer,
    left: Integer,
    right: Integer,
    rows: Integer,
    cols: Integer,
) {
    if unsafe { linegrid(ui) } {
        send!(
            ui,
            c"grid_scroll",
            Object::integer(grid),
            Object::integer(top),
            Object::integer(bot),
            Object::integer(left),
            Object::integer(right),
            Object::integer(rows),
            Object::integer(cols),
        );
        return;
    }
    // The legacy protocol scrolls whatever the current region is, so the
    // region has to be set, used, and put back — a UI left with a stale
    // region would scroll the wrong rows on the next command that did not
    // set one. The bounds are inclusive there and exclusive here.
    send!(
        ui,
        c"set_scroll_region",
        Object::integer(top),
        Object::integer(bot - 1),
        Object::integer(left),
        Object::integer(right - 1),
    );
    send!(ui, c"scroll", Object::integer(rows));
    let (width, height) = unsafe { ((*ui).width, (*ui).height) };
    send!(
        ui,
        c"set_scroll_region",
        Object::integer(0),
        Object::integer(Integer::from(height - 1)),
        Object::integer(0),
        Object::integer(Integer::from(width - 1)),
    );
}

/// The colours `Normal` resolves to.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_default_colors_set(
    ui: *mut RemoteUI,
    mut rgb_fg: Integer,
    mut rgb_bg: Integer,
    mut rgb_sp: Integer,
    cterm_fg: Integer,
    cterm_bg: Integer,
) {
    // A UI that did not ask for `ext_termcolors` has no colours of its own
    // to fall back on, so "unset" has to be resolved to something, and
    // 'background' is the only hint there is.
    if !unsafe { (*ui).ui_ext[kUITermColors as usize] } {
        let dark = unsafe { *p_bg.get() } == b'd' as c_char;
        if rgb_fg == -1 {
            rgb_fg = if dark { 0xffffff } else { 0 };
        }
        if rgb_bg == -1 {
            rgb_bg = if dark { 0 } else { 0xffffff };
        }
        if rgb_sp == -1 {
            rgb_sp = 0xff0000;
        }
    }
    send!(
        ui,
        c"default_colors_set",
        Object::integer(rgb_fg),
        Object::integer(rgb_bg),
        Object::integer(rgb_sp),
        Object::integer(cterm_fg),
        Object::integer(cterm_bg),
    );

    // The legacy protocol has one colour per channel, sent either as RGB or
    // as a terminal palette index — one below what the modern protocol
    // uses, which counts "unset" as zero. There is no cterm special colour,
    // so `update_sp` has nothing to fall back on.
    if !unsafe { linegrid(ui) } {
        let rgb = unsafe { (*ui).rgb };
        for (name, value) in [
            (c"update_fg", if rgb { rgb_fg } else { cterm_fg - 1 }),
            (c"update_bg", if rgb { rgb_bg } else { cterm_bg - 1 }),
            (c"update_sp", if rgb { rgb_sp } else { -1 }),
        ] {
            send!(ui, name, Object::integer(value));
        }
    }
}

/// Ends a batch: the UI may now show everything since the last flush.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_flush(ui: *mut RemoteUI) {
    unsafe {
        if (*ui).nevents == 0 && !(*ui).flushed_events {
            return;
        }
        if !linegrid(ui) {
            // The cursor move deferred by `remote_ui_grid_cursor_goto`.
            let (row, col) = ((*ui).cursor_row, (*ui).cursor_col);
            remote_ui_cursor_goto(ui, row, col);
        }
    }
    send!(ui, c"flush");
    unsafe {
        ui_flush_buf(ui, false);
        (*ui).flushed_events = false;
    }
}

/// Forwards a terminal control sequence the editor produced.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn remote_ui_ui_send(ui: *mut RemoteUI, content: String_0) {
    // Only meaningful to a UI that owns a terminal to write it to.
    if !unsafe { (*ui).stdout_tty } {
        return;
    }
    send!(ui, c"ui_send", Object::string(content));
}

pub(super) use {count, send};
