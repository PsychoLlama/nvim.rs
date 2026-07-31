//! The UI event sinks: one entry point per event the editor can announce.
//!
//! Everything here is one table and the macro that expands it. Upstream
//! generates the same functions from `ui_events.in.h`; the transpiler
//! flattened that generator's output into 1,500 lines of six-line bodies
//! that differ only in a name and an argument list, which is what the table
//! below puts back.
//!
//! An event reaches attached UIs one of two ways.
//!
//! **[`Reach::All`] and friends** hand the arguments straight to the
//! serializer in [`api::ui`](crate::src::nvim::api::ui), which packs them
//! into that UI's outgoing msgpack buffer. This is the fast path and it is
//! what everything drawn on a screen uses.
//!
//! **[`event`] events** go out as a generic `redraw` call —
//! `(name, [args])` — because they first have to be offered to the Lua
//! callbacks registered by `vim.ui_attach()`, which can consume the event
//! and stop it reaching any UI at all. Each such sink carries a re-entry
//! guard: a callback is arbitrary Lua and can perfectly well provoke the
//! same event again.
//!
//! The [`Reach`] split is the compositor's. A UI that did not ask for
//! `ext_multigrid` sees a single grid, so its grid events are the ones the
//! compositor produced by flattening the real grids: those UIs are
//! [`Reach::Composed`] and are fed from `ui_composed_call_*`, which the
//! compositor calls when it has finished a flattened line. The rest see the
//! real grids and are fed directly from `ui_call_*`, which also passes the
//! event to the compositor on the way past.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{Reach, reaches, ui_at, ui_count};
use crate::src::nvim::api::ui::{
    remote_ui_bell, remote_ui_busy_start, remote_ui_busy_stop, remote_ui_chdir,
    remote_ui_default_colors_set, remote_ui_error_exit, remote_ui_flush, remote_ui_grid_clear,
    remote_ui_grid_cursor_goto, remote_ui_grid_resize, remote_ui_grid_scroll,
    remote_ui_hl_attr_define, remote_ui_hl_group_set, remote_ui_mode_change,
    remote_ui_mode_info_set, remote_ui_mouse_off, remote_ui_mouse_on, remote_ui_msg_set_pos,
    remote_ui_option_set, remote_ui_raw_line, remote_ui_screenshot, remote_ui_set_icon,
    remote_ui_set_title, remote_ui_stop, remote_ui_suspend, remote_ui_ui_send,
    remote_ui_update_menu, remote_ui_visual_bell, remote_ui_win_viewport,
    remote_ui_win_viewport_margins,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::types::builders::ArrayBuf;
use crate::src::nvim::types::{
    Array, Boolean, Buffer, Float, HlAttrs, Integer, LineFlags, Object, RemoteUI, String_0,
    Tabpage, Window, sattr_T, schar_T,
};
use crate::src::nvim::ui_compositor::{
    ui_comp_grid_cursor_goto, ui_comp_grid_resize, ui_comp_grid_scroll, ui_comp_msg_set_pos,
    ui_comp_raw_line,
};
use core::ffi::{CStr, c_int};

use super::callbacks::ui_call_event;

/// The number of argument names given, as a `usize` constant.
macro_rules! count {
    () => { 0usize };
    ($head:ident $($tail:ident)*) => { 1usize + count!($($tail)*) };
}

/// One argument of an [`event!`] sink, tagged for the wire.
///
/// The declared type picks the constructor: a `Window` is not an `Integer`
/// on the wire even though both are `i64` in the signature, so the mapping
/// has to be by token and cannot be a trait.
macro_rules! wire {
    (Integer, $v:expr) => {
        Object::integer($v)
    };
    (Boolean, $v:expr) => {
        Object::boolean($v)
    };
    (Float, $v:expr) => {
        Object::float($v)
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
    (Buffer, $v:expr) => {
        Object::buffer($v)
    };
    (Tabpage, $v:expr) => {
        Object::tabpage($v)
    };
    (Object, $v:expr) => {
        $v
    };
}

/// Defines a sink that hands its arguments to each UI's serializer.
///
/// `reach` selects which UIs — see [`Reach`]. `via` names a compositor
/// entry point to run first, before any UI is touched; it is what makes the
/// flattened grid exist for the [`Reach::Composed`] UIs to be sent later.
///
/// The trailing string is the name the debug log shows. It is passed as a
/// `&'static CStr` per sink, which is what lets [`log_event`] collapse a run
/// of the same event by comparing pointers rather than bytes.
macro_rules! broadcast {
    ($(
        $(#[$attr:meta])*
        fn $sink:ident($($arg:ident: $ty:ty),* $(,)?)
            => $serialize:ident, $reach:ident, $name:literal
            $(, via $comp:expr)?;
    )*) => {$(
        $(#[$attr])*
        pub fn $sink($($arg: $ty),*) {
            $( unsafe { $comp }; )?
            broadcast_to(Reach::$reach, $name, |ui| unsafe {
                $serialize(ui, $($arg),*)
            });
        }
    )*};
}

/// Notes that `name` was sent, collapsing a run of the same event.
///
/// Redraws produce thousands of `raw_line`s and a debug log listing each
/// one is unreadable, so a run becomes one line and a count. The comparison
/// is by pointer, which is why every sink passes the same `&'static CStr`
/// every time.
pub(super) fn log_event(name: &'static CStr) {
    static seen: GlobalCell<usize> = GlobalCell::new(0);
    static last_event: GlobalCell<Option<&'static CStr>> = GlobalCell::new(None);

    if last_event
        .get()
        .is_some_and(|previous| core::ptr::eq(previous, name))
    {
        seen.set(seen.get() + 1);
        return;
    }
    unsafe {
        if let Some(previous) = last_event.get()
            && seen.get() > 0
        {
            log(
                c"%s (+%zu times...)".as_ptr(),
                previous.as_ptr(),
                seen.get(),
            );
        }
        log(c"%s".as_ptr(), name.as_ptr(), 0);
    }
    seen.set(0);
    last_event.set(Some(name));
}

/// # Safety
///
/// `format` must be a valid format string for the two arguments.
unsafe fn log(format: *const core::ffi::c_char, name: *const core::ffi::c_char, count: usize) {
    const LOGLVL_DBG: c_int = 1;
    unsafe {
        logmsg(
            LOGLVL_DBG,
            c"UI: ".as_ptr(),
            core::ptr::null(),
            -1,
            true,
            format,
            name,
            count,
        )
    };
}

/// Hands every UI that `reach` selects to `send`, and logs `name` if there
/// was at least one.
///
/// The count is re-read every iteration because a serializer that cannot
/// write can disconnect its own UI, which compacts the table underneath us.
fn broadcast_to(reach: Reach, name: &'static CStr, mut send: impl FnMut(*mut RemoteUI)) {
    let mut any_call = false;
    let mut i = 0;
    while i < ui_count() {
        let ui = ui_at(i);
        if reaches(ui, reach) {
            send(ui);
            any_call = true;
        }
        i += 1;
    }
    if any_call {
        log_event(name);
    }
}

/// Defines a sink that goes out as a generic `redraw` call.
///
/// The re-entry guard is per sink and deliberately silent: C's generator
/// emits the same `if (entered) { return; }`, on the grounds that an event
/// provoked from inside a handler for that same event has nowhere sensible
/// to go.
macro_rules! event {
    ($(
        $(#[$attr:meta])*
        fn $sink:ident($($arg:ident: $ty:ident),* $(,)?) => $name:literal;
    )*) => {$(
        $(#[$attr])*
        pub fn $sink($($arg: $ty),*) {
            static ENTERED: GlobalCell<bool> = GlobalCell::new(false);
            if ENTERED.get() {
                return;
            }
            ENTERED.set(true);
            let mut args = ArrayBuf::<{ count!($($arg)*) }>::new();
            $( args.push(wire!($ty, $arg)); )*
            unsafe { ui_call_event($name, args.array()) };
            ENTERED.set(false);
        }
    )*};
}

broadcast! {
    fn ui_call_mode_info_set(enabled: Boolean, cursor_styles: Array)
        => remote_ui_mode_info_set, All, c"mode_info_set";
    fn ui_call_update_menu() => remote_ui_update_menu, All, c"update_menu";
    fn ui_call_busy_start() => remote_ui_busy_start, All, c"busy_start";
    fn ui_call_busy_stop() => remote_ui_busy_stop, All, c"busy_stop";
    fn ui_call_mouse_on() => remote_ui_mouse_on, All, c"mouse_on";
    fn ui_call_mouse_off() => remote_ui_mouse_off, All, c"mouse_off";
    fn ui_call_mode_change(mode: String_0, mode_idx: Integer)
        => remote_ui_mode_change, All, c"mode_change";
    fn ui_call_bell() => remote_ui_bell, All, c"bell";
    fn ui_call_visual_bell() => remote_ui_visual_bell, All, c"visual_bell";
    fn ui_call_flush() => remote_ui_flush, All, c"flush";
    fn ui_call_suspend() => remote_ui_suspend, All, c"suspend";
    fn ui_call_set_icon(icon: String_0) => remote_ui_set_icon, All, c"set_icon";
    fn ui_call_screenshot(path: String_0) => remote_ui_screenshot, All, c"screenshot";
    fn ui_call_option_set(name: String_0, value: Object)
        => remote_ui_option_set, All, c"option_set";
    fn ui_call_stop() => remote_ui_stop, All, c"stop";
    fn ui_call_ui_send(content: String_0) => remote_ui_ui_send, All, c"ui_send";
    fn ui_call_default_colors_set(
        rgb_fg: Integer,
        rgb_bg: Integer,
        rgb_sp: Integer,
        cterm_fg: Integer,
        cterm_bg: Integer,
    ) => remote_ui_default_colors_set, All, c"default_colors_set";
    fn ui_call_hl_attr_define(
        id: Integer,
        rgb_attrs: HlAttrs,
        cterm_attrs: HlAttrs,
        info: Array,
    ) => remote_ui_hl_attr_define, All, c"hl_attr_define";
    fn ui_call_hl_group_set(name: String_0, id: Integer)
        => remote_ui_hl_group_set, All, c"hl_group_set";
    fn ui_call_grid_clear(grid: Integer) => remote_ui_grid_clear, All, c"grid_clear";
    fn ui_call_win_viewport(
        grid: Integer,
        win: Window,
        topline: Integer,
        botline: Integer,
        curline: Integer,
        curcol: Integer,
        line_count: Integer,
        scroll_delta: Integer,
    ) => remote_ui_win_viewport, All, c"win_viewport";
    fn ui_call_win_viewport_margins(
        grid: Integer,
        win: Window,
        top: Integer,
        bottom: Integer,
        left: Integer,
        right: Integer,
    ) => remote_ui_win_viewport_margins, All, c"win_viewport_margins";
    fn ui_call_error_exit(status: Integer) => remote_ui_error_exit, All, c"error_exit";

    // Compositor-split events. `ui_call_*` runs the compositor and reaches
    // the UIs that see the real grids; `ui_composed_call_*` reaches the
    // rest, and is called by the compositor once it has flattened them.
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer)
        => remote_ui_grid_resize, Uncomposed, c"grid_resize", via ui_comp_grid_resize(grid, width, height);
    fn ui_composed_call_grid_resize(grid: Integer, width: Integer, height: Integer)
        => remote_ui_grid_resize, Composed, c"grid_resize";
    fn ui_call_grid_cursor_goto(grid: Integer, row: Integer, col: Integer)
        => remote_ui_grid_cursor_goto, Uncomposed, c"grid_cursor_goto", via ui_comp_grid_cursor_goto(grid, row, col);
    fn ui_composed_call_grid_cursor_goto(grid: Integer, row: Integer, col: Integer)
        => remote_ui_grid_cursor_goto, Composed, c"grid_cursor_goto";
    fn ui_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    ) => remote_ui_grid_scroll, Uncomposed, c"grid_scroll", via ui_comp_grid_scroll(grid, top, bot, left, right, rows, cols);
    fn ui_composed_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    ) => remote_ui_grid_scroll, Composed, c"grid_scroll";
    // The message grid is composited but never re-sent to the composed UIs:
    // they learn its position from the flattened grid itself.
    fn ui_call_msg_set_pos(
        grid: Integer,
        row: Integer,
        scrolled: Boolean,
        sep_char: String_0,
        zindex: Integer,
        compindex: Integer,
    ) => remote_ui_msg_set_pos, Uncomposed, c"msg_set_pos", via ui_comp_msg_set_pos(grid, row, scrolled, sep_char, zindex, compindex);
}

event! {
    fn ui_call_restart(listen_addr: String_0) => c"restart";
    fn ui_call_grid_destroy(grid: Integer) => c"grid_destroy";
    fn ui_call_win_pos(
        grid: Integer,
        win: Window,
        startrow: Integer,
        startcol: Integer,
        width: Integer,
        height: Integer,
    ) => c"win_pos";
    fn ui_call_win_float_pos(
        grid: Integer,
        win: Window,
        anchor: String_0,
        anchor_grid: Integer,
        anchor_row: Float,
        anchor_col: Float,
        mouse_enabled: Boolean,
        zindex: Integer,
        compindex: Integer,
        screen_row: Integer,
        screen_col: Integer,
    ) => c"win_float_pos";
    fn ui_call_win_external_pos(grid: Integer, win: Window) => c"win_external_pos";
    fn ui_call_win_hide(grid: Integer) => c"win_hide";
    fn ui_call_win_close(grid: Integer) => c"win_close";
    fn ui_call_win_extmark(
        grid: Integer,
        win: Window,
        ns_id: Integer,
        mark_id: Integer,
        row: Integer,
        col: Integer,
    ) => c"win_extmark";
    fn ui_call_popupmenu_show(
        items: Array,
        selected: Integer,
        row: Integer,
        col: Integer,
        grid: Integer,
    ) => c"popupmenu_show";
    fn ui_call_popupmenu_hide() => c"popupmenu_hide";
    fn ui_call_popupmenu_select(selected: Integer) => c"popupmenu_select";
    fn ui_call_tabline_update(
        current: Tabpage,
        tabs: Array,
        current_buffer: Buffer,
        buffers: Array,
    ) => c"tabline_update";
    fn ui_call_cmdline_show(
        content: Array,
        pos: Integer,
        firstc: String_0,
        prompt: String_0,
        indent: Integer,
        level: Integer,
        hl_id: Integer,
    ) => c"cmdline_show";
    fn ui_call_cmdline_pos(pos: Integer, level: Integer) => c"cmdline_pos";
    fn ui_call_cmdline_special_char(c: String_0, shift: Boolean, level: Integer)
        => c"cmdline_special_char";
    fn ui_call_cmdline_hide(level: Integer, abort: Boolean) => c"cmdline_hide";
    fn ui_call_cmdline_block_show(lines: Array) => c"cmdline_block_show";
    fn ui_call_cmdline_block_append(lines: Array) => c"cmdline_block_append";
    fn ui_call_cmdline_block_hide() => c"cmdline_block_hide";
    fn ui_call_msg_show(
        kind: String_0,
        content: Array,
        replace_last: Boolean,
        history: Boolean,
        append: Boolean,
        id: Object,
        trigger: String_0,
    ) => c"msg_show";
    fn ui_call_msg_clear() => c"msg_clear";
    fn ui_call_msg_showcmd(content: Array) => c"msg_showcmd";
    fn ui_call_msg_showmode(content: Array) => c"msg_showmode";
    fn ui_call_msg_ruler(content: Array) => c"msg_ruler";
    fn ui_call_msg_history_show(entries: Array, prev_cmd: Boolean) => c"msg_history_show";
}

/// A screen line, straight from the grid it was drawn into.
///
/// The only sink taking pointers rather than API values, and so the only
/// one the table above cannot hold: `chunk` and `attrs` are the grid's own
/// arrays, passed by address to keep the hot path free of copies.
///
/// # Safety
///
/// `chunk` and `attrs` must each address at least `endcol - startcol`
/// readable elements. The serializers read them without a length.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn ui_call_raw_line(
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    unsafe {
        ui_comp_raw_line(
            grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
        );
        raw_line_to(
            Reach::Uncomposed,
            grid,
            row,
            startcol,
            endcol,
            clearcol,
            clearattr,
            flags,
            chunk,
            attrs,
        );
    }
}

/// [`ui_call_raw_line`] for the UIs the compositor draws for, called by the
/// compositor once it has flattened the line.
///
/// # Safety
///
/// As [`ui_call_raw_line`].
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn ui_composed_call_raw_line(
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    unsafe {
        raw_line_to(
            Reach::Composed,
            grid,
            row,
            startcol,
            endcol,
            clearcol,
            clearattr,
            flags,
            chunk,
            attrs,
        )
    }
}

/// # Safety
///
/// As [`ui_call_raw_line`].
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
unsafe fn raw_line_to(
    reach: Reach,
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    broadcast_to(reach, c"raw_line", |ui| unsafe {
        remote_ui_raw_line(
            ui, grid, row, startcol, endcol, clearcol, clearattr, flags, chunk, attrs,
        )
    });
}

/// `set_title` is one of two sinks a test drives directly over the C ABI:
/// the 64 KiB title in `tui_spec.lua` has no other way in.
#[unsafe(no_mangle)]
pub extern "C" fn ui_call_set_title(title: String_0) {
    broadcast_to(Reach::All, c"set_title", |ui| unsafe {
        remote_ui_set_title(ui, title)
    });
}

/// As [`ui_call_set_title`]; `tui_spec.lua` drives this one to check that a
/// relative path sent by the server is resolved against the server's cwd.
#[unsafe(no_mangle)]
pub extern "C" fn ui_call_chdir(path: String_0) {
    broadcast_to(Reach::All, c"chdir", |ui| unsafe {
        remote_ui_chdir(ui, path)
    });
}
