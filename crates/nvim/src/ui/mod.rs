//! What the editor tells the UIs, and the bookkeeping behind it.
//!
//! A UI is a msgpack-rpc peer that called `nvim_ui_attach`. Up to
//! [`MAX_UI_COUNT`] of them can be attached at once, and the editor draws
//! for all of them at the same time: every screen change is announced to
//! each one in turn by a sink in [`sinks`], which hands it to that UI's
//! serializer in [`api::ui`](crate::api::ui). Nothing here
//! writes bytes or knows about channels.
//!
//! The two things this module owns are the attach table and the negotiated
//! capabilities. Attaching or detaching changes what the editor can assume
//! about who is watching — the screen size becomes the smallest attached
//! UI's, and an external widget is only drawn externally if *every*
//! attached UI asked for it — so both go through [`ui_refresh`], which
//! recomputes the intersection and resizes the screen to match.
//!
//! [`ui_flush`] is the other half. Cursor position, mode and mouse state
//! are not sent as they change; they are marked pending and sent once at
//! the end of a redraw, because the intermediate values are never worth a
//! round trip. Everything with a `pending_` name below is that.
//!
//! What lives where: the per-event entry points in [`sinks`], the
//! `vim.ui_attach()` Lua handlers in [`callbacks`], and the flattening of
//! several grids into one for UIs that did not ask for `ext_multigrid` in
//! [`ui_compositor`](crate::ui_compositor).

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod callbacks;
mod mouse;
mod sinks;
pub(crate) mod state;

pub use callbacks::{ui_add_cb, ui_call_event, ui_cb_ext, ui_remove_cb};
pub use mouse::{ui_check_mouse, ui_mouse_has};
pub use sinks::*;

use crate::api::ui::{remote_ui_hl_attr_define, remote_ui_hl_group_set, remote_ui_option_set};
use crate::autocmd::do_autocmd_uienter;
use crate::buffer::resettitle;
use crate::cursor_shape::{
    SHAPE_IDX_N, SHAPE_IDX_R, cursor_get_mode_idx, mode_style_array, shape_entry,
};
use crate::drawscreen::state::updating_screen;
use crate::drawscreen::{conceal_check_cursor_line, screen_resize};
use crate::event::multiqueue::put_job_in;
use crate::ex_getln::cmdline_ui_flush;
use crate::getchar::state::expr_map_lock;
use crate::global_cell::GlobalCell;
use crate::grid::{GridRef, default_grid_ref, get_win_by_grid_handle};
use crate::guard::textlock;
use crate::highlight::state::{
    cterm_normal_bg_color, cterm_normal_fg_color, normal_bg, normal_fg, normal_sp,
};
use crate::highlight::{highlight_use_hlstate, ui_send_all_hls};
use crate::highlight_group::HLF_W;
use crate::message::state::{emsg_silent, in_assert_fails};
use crate::message::{msg, msg_ext_ui_flush, msg_scroll_flush, msg_source, msg_ui_refresh};
use crate::option::vars::{
    P_GUICURSOR, P_LZ, bo_flags, p_debug, p_lz, p_tgc, p_vb, p_wd, rdb_flags,
};
use crate::option::{set_option_value, ui_refresh_options};
use crate::options::{kOptBoFlagAll, kOptCmdheight, kOptRdbFlagFlush, kOptRdbFlagLine};
use crate::os::cshim::gettext;
use crate::os::fs::current_dir;
use crate::os::time::{os_hrtime, os_sleep};
use crate::startup::{exiting, full_screen, starting, ui_client_channel_id};
use crate::state::MODE_CMDLINE;
use crate::state::mode::State;
use crate::strings::has_char;
use crate::types::ui::{
    kLineFlagInvalid, kLineFlagWrap, kUIExtCount, kUIFloatDebug, kUIHlState, kUILinegrid,
    kUIMessages, kUIMultigrid, kUITermColors,
};
use crate::types::{
    ApiDict, Array, Boolean, Error, Handle, Integer, LineFlags, Object, OptVal, OptionSetFlags,
    RemoteUI, String_0, UIExtension,
};
use crate::ui::state::{called_vim_beep, resize_events, ui_ext_names, ui_refresh_cmdheight};
use crate::ui_compositor::{
    ui_comp_attach, ui_comp_detach, ui_comp_get_grid_at_coord, ui_comp_init, ui_comp_should_draw,
    win_grid,
};
use crate::window::{win_set_inner_size, win_ui_flush};
use crate::winfloat::win_config_float;
use core::ffi::c_int;

use crate::api::private::validate::err_bad_number;
use crate::winlayer::{Win, tabs};
/// The screen the editor draws into when nothing else claims a grid.
const DEFAULT_GRID_HANDLE: Handle = 1;

/// How many UIs may be attached at once.
///
/// The table is a fixed array because the count is small and the sinks walk
/// it on every screen line; growing it would be a wire-protocol question,
/// not an allocation one.
pub const MAX_UI_COUNT: usize = 16;

/// The attached UIs, oldest first, found by their channel id.
///
/// The table owns them: nothing outside keeps a UI's address. A borrow of
/// the table is a leaf -- nothing run under one may reach the table again
/// or run user code -- so anything that does finds its UI again by id
/// afterwards, and copes with it having gone.
static uis: GlobalCell<Vec<RemoteUI>> = GlobalCell::new(Vec::new());

/// The widgets every attached UI draws itself, plus whatever
/// [`ui_cb_ext`] adds. Read through [`ui_has`].
static ui_ext: GlobalCell<[bool; kUIExtCount as usize]> =
    GlobalCell::new([false; kUIExtCount as usize]);

static ui_mode_idx: GlobalCell<c_int> = GlobalCell::new(SHAPE_IDX_N);
static cursor_row: GlobalCell<c_int> = GlobalCell::new(0);
static cursor_col: GlobalCell<c_int> = GlobalCell::new(0);
static cursor_grid_handle: GlobalCell<Handle> = GlobalCell::new(DEFAULT_GRID_HANDLE);
static pending_cursor_update: GlobalCell<bool> = GlobalCell::new(false);
static pending_mode_info_update: GlobalCell<bool> = GlobalCell::new(false);
static pending_mode_update: GlobalCell<bool> = GlobalCell::new(false);
static pending_default_colors: GlobalCell<bool> = GlobalCell::new(false);
/// `-1` until the first [`ui_flush`], so that the initial state is sent
/// whichever way it goes.
static pending_has_mouse: GlobalCell<c_int> = GlobalCell::new(-1);
/// Nesting depth of [`ui_busy_start`]/[`ui_busy_stop`]; the UIs only hear
/// about the outermost pair.
static busy: GlobalCell<c_int> = GlobalCell::new(0);

/// Which of the attached UIs a sink reaches.
///
/// A UI that did not ask for `ext_multigrid` is `Composed`: it is shown the
/// compositor's single flattened grid rather than the real ones, so the
/// grid events it gets are the compositor's, sent later and from a
/// different entry point.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Every attached UI. Used by everything that is not about a grid.
    All,
    /// Only the UIs the compositor draws for.
    Composed,
    /// Only the UIs that see the real grids.
    Uncomposed,
}

/// The number of attached UIs.
fn ui_count() -> usize {
    uis.with(Vec::len)
}

/// Runs `f` on the UI attached to `channel_id`, if there is one.
///
/// A leaf: see [`uis`].
pub(crate) fn with_ui<R>(channel_id: u64, f: impl FnOnce(&mut RemoteUI) -> R) -> Option<R> {
    uis.with_mut(|table| {
        let ui = table.iter_mut().find(|ui| ui.channel_id == channel_id)?;
        Some(f(ui))
    })
}

/// Whether a UI is attached to `channel_id`.
pub(crate) fn ui_is_attached(channel_id: u64) -> bool {
    uis.with(|table| table.iter().any(|ui| ui.channel_id == channel_id))
}

/// Whether any attached UI satisfies `test`.
fn any_ui(test: impl FnMut(&RemoteUI) -> bool) -> bool {
    uis.with(|table| table.iter().any(test))
}

/// Runs `send` on every attached UI that `reach` selects, in attach order,
/// answering whether there was one. A leaf, as [`with_ui`].
pub(crate) fn each_ui_reached(reach: Reach, mut send: impl FnMut(&mut RemoteUI)) -> bool {
    uis.with_mut(|table| {
        let mut any = false;
        for ui in table.iter_mut().filter(|ui| reaches(ui, reach)) {
            send(ui);
            any = true;
        }
        any
    })
}

/// The channel ids of the attached UIs, in attach order, for the callers
/// that run user code between one UI and the next.
fn ui_channels() -> Vec<u64> {
    uis.with(|table| table.iter().map(|ui| ui.channel_id).collect())
}

/// Whether `reach` selects `ui`.
fn reaches(ui: &RemoteUI, reach: Reach) -> bool {
    match reach {
        Reach::All => true,
        Reach::Composed => ui.composed,
        Reach::Uncomposed => !ui.composed,
    }
}

pub fn ui_init() {
    let mut grid = default_grid_ref();
    grid.handle = DEFAULT_GRID_HANDLE;
    ui_comp_init();
}

/// Whether the editor should resolve highlights to 24-bit colour.
///
/// `'termguicolors'` forces it. Otherwise it takes one non-terminal UI
/// asking for RGB: a terminal UI's `rgb` flag describes the terminal, which
/// the TUI has already accounted for.
pub fn ui_rgb_attached() -> bool {
    p_tgc() || any_ui(|ui| !is_tui(ui) && ui.rgb)
}

/// Whether any attached UI is something other than a terminal.
pub fn ui_gui_attached() -> bool {
    any_ui(|ui| !is_tui(ui))
}

/// Whether any attached UI asked to be given every external widget whether
/// or not the others want them. `nvim --embed`'s debugging escape hatch.
pub fn ui_override() -> bool {
    any_ui(|ui| ui.override_0)
}

/// The number of attached UIs.
pub fn ui_active() -> usize {
    ui_count()
}

fn is_tui(ui: &RemoteUI) -> bool {
    ui.stdin_tty || ui.stdout_tty
}

/// Renegotiates what the attached UIs can do, and resizes to match.
///
/// The screen becomes the smallest attached UI's, and a widget is external
/// only if every UI asked for it — a UI that cannot draw its own popup menu
/// must be sent one drawn into the grid, and the others cannot be sent a
/// different screen. [`ui_override`] opts out of the intersection.
pub fn ui_refresh() {
    assert!(
        ui_client_channel_id.get() == 0,
        "the UI client has no UIs of its own"
    );

    let mut width = c_int::MAX;
    let mut height = c_int::MAX;
    let inclusive = ui_override();
    // Start from all-true so the fold below is an intersection, but only if
    // there is something to intersect.
    let mut ext_widgets = [ui_active() != 0; kUIExtCount as usize];
    uis.with(|table| {
        for ui in table {
            width = width.min(ui.width);
            height = height.min(ui.height);
            for (widget, enabled) in ext_widgets[..kUIExtCount as usize].iter_mut().enumerate() {
                *enabled &= ui.ui_ext[widget] || inclusive;
            }
        }
    });

    cursor_row.set(0);
    cursor_col.set(0);
    pending_cursor_update.set(true);

    let had_message = ui_has(kUIMessages);
    for (widget, &wanted) in ext_widgets[..kUIExtCount as usize].iter().enumerate() {
        let enabled = wanted || ui_cb_ext.with(|cb| cb[widget]);
        ui_ext.with_mut(|widgets| widgets[widget] = enabled);
        // The widgets past `ext_linegrid` describe how a UI wants the
        // screen expressed rather than who draws what, and are not options
        // a UI can be told about.
        if widget < kUILinegrid as usize {
            let name = String_0::from_cstr(ui_ext_names[widget]);
            ui_call_option_set(name, Object::boolean(enabled));
        }
    }

    if had_message != ui_has(kUIMessages) {
        // Messages moving in or out of the grid changes how many lines the
        // message area needs. `ui_refresh_cmdheight` is off while the
        // user's own 'cmdheight' is being restored.
        if ui_refresh_cmdheight.get() {
            let _ = set_option_value(
                kOptCmdheight,
                OptVal::Number(had_message as _),
                OptionSetFlags::NONE,
            );
            for mut tp in tabs() {
                tp.tp_ch_used = had_message as _;
            }
        }
        msg_scroll_flush();
    }
    msg_ui_refresh();

    if ui_active() == 0 {
        return;
    }
    if updating_screen.get() {
        // Resizing mid-redraw would invalidate the grid being drawn into.
        ui_schedule_refresh();
        return;
    }
    ui_default_colors_set();
    // 'lazyredraw' would defer the resize past the point the UIs are
    // told about it.
    let save_p_lz = p_lz();
    P_LZ.set(false);
    screen_resize(width, height);
    P_LZ.set(save_p_lz);
    ui_mode_info_set();
    pending_mode_update.set(true);
    ui_cursor_shape();
    pending_has_mouse.set(-1);
}

/// The tallest popup menu every attached UI will draw, or zero if none of
/// them asked for a limit.
pub fn ui_pum_get_height() -> c_int {
    uis.with(|table| {
        let limits = table.iter().map(|ui| ui.pum_nlines).filter(|&n| n != 0);
        limits.min().unwrap_or(0)
    })
}

/// The geometry of the externally drawn popup menu, if a UI reported one:
/// width, height, row and column.
pub fn ui_pum_get_pos() -> Option<[f64; 4]> {
    uis.with(|table| {
        let ui = table.iter().find(|ui| ui.pum_pos)?;
        Some([ui.pum_width, ui.pum_height, ui.pum_row, ui.pum_col])
    })
}

/// Queues [`ui_refresh`] for after the current redraw.
pub fn ui_schedule_refresh() {
    put_job_in(&resize_events, Box::new(ui_refresh));
}

/// Marks the default colours as needing to be re-sent.
pub fn ui_default_colors_set() {
    pending_default_colors.set(true);
    // Before startup finishes the colours are still being computed, and
    // the pending flag is enough — `ui_line` picks it up.
    if starting.get() == 0 {
        ui_may_set_default_colors();
    }
}

fn ui_may_set_default_colors() {
    if !pending_default_colors.get() {
        return;
    }
    pending_default_colors.set(false);
    ui_call_default_colors_set(
        normal_fg.get() as Integer,
        normal_bg.get() as Integer,
        normal_sp.get() as Integer,
        cterm_normal_fg_color.get() as Integer,
        cterm_normal_bg_color.get() as Integer,
    );
}

/// Enters the busy state, in which UIs stop drawing a cursor.
pub fn ui_busy_start() {
    busy.set(busy.get() + 1);
    if busy.get() == 1 {
        ui_call_busy_start();
    }
}

/// Leaves it. Only the outermost [`ui_busy_start`] is announced.
pub fn ui_busy_stop() {
    busy.set(busy.get() - 1);
    if busy.get() == 0 {
        ui_call_busy_stop();
    }
}

/// Beeps, unless `'belloff'` covers `val`.
pub fn vim_beep(val: core::ffi::c_uint) {
    called_vim_beep.set(true);
    if emsg_silent.get() != 0 || in_assert_fails.get() {
        return;
    }
    if bo_flags.get() & val == 0 && bo_flags.get() & kOptBoFlagAll as core::ffi::c_uint == 0 {
        // At most three beeps per half second: a stuck macro that beeps on
        // every iteration should not make the terminal unusable.
        static beeps: GlobalCell<c_int> = GlobalCell::new(0);
        static start_time: GlobalCell<u64> = GlobalCell::new(0);
        if start_time.get() == 0 || os_hrtime() - start_time.get() > 500_000_000 {
            beeps.set(0);
            start_time.set(os_hrtime());
        }
        beeps.set(beeps.get() + 1);
        if beeps.get() <= 3 {
            if p_vb() {
                ui_call_visual_bell();
            } else {
                ui_call_bell();
            }
        }
    }
    if p_debug(|value| has_char(value, 'e' as c_int)) {
        msg_source(HLF_W);
        msg(gettext(c"Beep!"), HLF_W);
    }
}

/// Fires `UIEnter` once per attached UI. Startup's catch-up, for the UIs
/// that attached before autocommands were running.
pub fn do_autocmd_uienter_all() {
    for channel_id in ui_channels() {
        do_autocmd_uienter(channel_id, true);
    }
}

/// Whether another `nvim_ui_attach` would fit.
pub fn ui_can_attach_more() -> bool {
    ui_count() < MAX_UI_COUNT
}

/// Adds `ui` to the attach table and brings it up to date.
pub fn ui_attach_impl(mut ui: RemoteUI) {
    assert!(ui_can_attach_more(), "attach table is full");
    let chanid = ui.channel_id;
    // A UI that reads the real grids does not want the compositor's
    // flattened one. `_debug_float` is the same thing for debugging.
    let multigrid = ui.ui_ext[kUIMultigrid as usize];
    let float_debug = ui.ui_ext[kUIFloatDebug as usize];
    if !multigrid && !float_debug && ui_client_channel_id.get() == 0 {
        ui_comp_attach(&mut ui);
    }
    uis.with_mut(|table| table.push(ui));

    // Both of these can run user code (a 'titlestring' expression), which
    // can detach the UI again: from here on it is found by id each time.
    ui_refresh_options();
    resettitle();

    // Tell it where the server is, so that a UI on another machine can
    // resolve the paths the server sends.
    if let Some(cwd) = current_dir() {
        ui_call_chdir(String_0::from_xstring(cwd));
    }

    for widget in kUILinegrid as usize..kUIExtCount as usize {
        let Some(active) = with_ui(chanid, |ui| ui.ui_ext[widget]) else {
            break;
        };
        ui_set_ext_option(chanid, widget as UIExtension, active);
    }

    // Highlights are sent as definitions plus ids, unless this is the
    // first UI to want that, in which case the whole table is rebuilt and
    // resent by `highlight_use_hlstate` itself.
    let hl_state = with_ui(chanid, |ui| ui.ui_ext[kUIHlState as usize]).unwrap_or(false);
    let sent = hl_state && highlight_use_hlstate();
    if !sent {
        ui_send_all_hls(
            &mut |id, attr, info| {
                with_ui(chanid, |ui| {
                    remote_ui_hl_attr_define(ui, id, attr, attr, info)
                });
            },
            &mut |name, attr| {
                with_ui(chanid, |ui| remote_ui_hl_group_set(ui, name, attr));
            },
        );
    }
    ui_refresh();
    do_autocmd_uienter(chanid, true);
}

/// Removes the UI on `chanid` from the attach table, answering it.
pub fn ui_detach_impl(chanid: u64) -> Option<RemoteUI> {
    let mut ui = uis.with_mut(|table| {
        let index = table.iter().position(|ui| ui.channel_id == chanid)?;
        Some(table.remove(index))
    })?;

    if ui_count() != 0 && !exiting.get() {
        // The screen size is the smallest attached UI's, so losing one can
        // make the screen bigger.
        ui_schedule_refresh();
    }
    let multigrid = ui.ui_ext[kUIMultigrid as usize];
    let float_debug = ui.ui_ext[kUIFloatDebug as usize];
    if !multigrid && !float_debug {
        ui_comp_detach(&mut ui);
    }
    do_autocmd_uienter(chanid, false);
    Some(ui)
}

/// Tells the UI on `chanid` that `ext` changed.
pub fn ui_set_ext_option(chanid: u64, ext: UIExtension, active: bool) {
    if ext < kUILinegrid {
        // An external widget changing changes the intersection, and every
        // UI has to be told the result rather than this one's request.
        ui_refresh();
        return;
    }
    let name = ui_ext_names[ext as usize];
    // A leading underscore marks an option that is not part of the
    // documented protocol; those are only mentioned when turned on.
    let private = name.to_bytes().first() == Some(&b'_');
    if !private || active {
        with_ui(chanid, |ui| {
            remote_ui_option_set(ui, String_0::from_cstr(name), Object::boolean(active));
        });
    }
    if ext == kUITermColors {
        ui_default_colors_set();
    }
}

/// Sends one drawn screen line. `row` must be within `grid`, which is
/// asserted.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub fn ui_line(
    grid: GridRef,
    row: c_int,
    invalid_row: bool,
    startcol: c_int,
    endcol: c_int,
    clearcol: c_int,
    clearattr: c_int,
    wrap: bool,
) {
    assert!((0..grid.rows).contains(&row), "row is outside the grid");

    let mut flags: LineFlags = if wrap { kLineFlagWrap } else { 0 };
    if startcol == 0 && invalid_row {
        flags |= kLineFlagInvalid;
    }
    ui_may_set_default_colors();

    let off = grid.cell_offset(row, startcol);
    let (chars, attrs) = grid.cells(off, (grid.cols - startcol) as usize);
    let handle = grid.handle as Integer;
    let (at_row, from) = (row as Integer, startcol as Integer);
    let (to, clear_from) = (endcol as Integer, clearcol as Integer);
    let clear_attr = clearattr as Integer;
    ui_call_raw_line(
        handle, at_row, from, to, clear_from, clear_attr, flags, chars, attrs,
    );

    // 'writedelay' with `redrawdebug=line`: park the cursor at the end of
    // the line just drawn and pause, so the draw order is watchable.
    if p_wd() != 0 && rdb_flags.get() & kOptRdbFlagLine as core::ffi::c_uint != 0 {
        ui_call_grid_cursor_goto(
            grid.handle as Integer,
            row as Integer,
            clearcol.min(grid.cols - 1) as Integer,
        );
        ui_call_flush();
        os_sleep(p_wd().unsigned_abs());
        pending_cursor_update.set(true);
    }
}

/// Moves the cursor on the default grid. See [`ui_grid_cursor_goto`].
pub fn ui_cursor_goto(new_row: c_int, new_col: c_int) {
    ui_grid_cursor_goto(DEFAULT_GRID_HANDLE, new_row, new_col);
}

/// Records where the cursor should end up.
///
/// Not sent here: a redraw moves the cursor many times and only the last
/// position matters, so [`ui_flush`] sends it once.
pub fn ui_grid_cursor_goto(grid_handle: Handle, new_row: c_int, new_col: c_int) {
    if new_row == cursor_row.get()
        && new_col == cursor_col.get()
        && grid_handle == cursor_grid_handle.get()
    {
        return;
    }
    cursor_row.set(new_row);
    cursor_col.set(new_col);
    cursor_grid_handle.set(grid_handle);
    pending_cursor_update.set(true);
}

/// Re-sends the cursor if it is on `grid_handle`, which has been redrawn
/// underneath it.
pub fn ui_check_cursor_grid(grid_handle: Handle) {
    if cursor_grid_handle.get() == grid_handle {
        pending_cursor_update.set(true);
    }
}

/// Marks `'guicursor'` as needing to be re-sent.
pub fn ui_mode_info_set() {
    pending_mode_info_update.set(true);
}

/// The row the cursor was last put on.
pub fn ui_current_row() -> c_int {
    cursor_row.get()
}

/// The column the cursor was last put on.
pub fn ui_current_col() -> c_int {
    cursor_col.get()
}

/// Ends a redraw: sends everything held back, then `flush`.
///
/// Until a UI sees `flush` it is free to show nothing of what came before,
/// so this is what makes a redraw visible.
pub fn ui_flush() {
    assert!(
        ui_client_channel_id.get() == 0,
        "the UI client has no UIs of its own"
    );
    if ui_active() == 0 {
        return;
    }

    // A hidden floating window is still the current window, and the cursor
    // is inside it, so there is nowhere to draw it: look busy instead.
    static was_busy: GlobalCell<bool> = GlobalCell::new(false);
    let cursor_nowhere = State.get() & MODE_CMDLINE == 0
        && Win::current().w_floating
        && Win::current().w_config.hide;
    if cursor_nowhere {
        if !was_busy.get() {
            ui_call_busy_start();
            was_busy.set(true);
        }
    } else if was_busy.get() {
        ui_call_busy_stop();
        was_busy.set(false);
    }

    win_ui_flush(false);
    if textlock.get() == 0 && expr_map_lock.get() == 0 {
        // Both can run Lua handlers, which the locks exist to keep out.
        cmdline_ui_flush();
        msg_ext_ui_flush();
    }
    msg_scroll_flush();

    if pending_cursor_update.get() {
        ui_call_grid_cursor_goto(
            cursor_grid_handle.get() as Integer,
            cursor_row.get() as Integer,
            cursor_col.get() as Integer,
        );
        pending_cursor_update.set(false);
        // Moving the cursor can uncover a window whose viewport changed.
        win_ui_flush(false);
    }

    if pending_mode_info_update.get() {
        let style = mode_style_array();
        let enabled = P_GUICURSOR.first_byte() != 0;
        ui_call_mode_info_set(enabled as Boolean, style);
        pending_mode_info_update.set(false);
    }

    // A cursor behind a floating window is reported as the "replace" shape,
    // which is the closest the protocol has to "do not draw one".
    static cursor_was_obscured: GlobalCell<bool> = GlobalCell::new(false);
    let cursor_obscured = ui_cursor_is_behind_floatwin();
    if (cursor_obscured != cursor_was_obscured.get() || pending_mode_update.get())
        && starting.get() == 0
    {
        let idx = if cursor_obscured {
            SHAPE_IDX_R
        } else {
            ui_mode_idx.get()
        };
        let full_name = shape_entry(idx).full_name;
        ui_call_mode_change(String_0::from_cstr(full_name), idx as Integer);
        pending_mode_update.set(false);
        cursor_was_obscured.set(cursor_obscured);
    }

    if pending_has_mouse.get() != mouse::wanted() as c_int {
        if mouse::wanted() {
            ui_call_mouse_on();
        } else {
            ui_call_mouse_off();
        }
        pending_has_mouse.set(mouse::wanted() as c_int);
    }

    ui_call_flush();

    if p_wd() != 0 && rdb_flags.get() & kOptRdbFlagFlush as core::ffi::c_uint != 0 {
        os_sleep(p_wd().unsigned_abs());
    }
}

/// Recomputes the cursor shape for the current mode, without disturbing
/// `'conceallevel'`. For the paths that are about to redraw anyway.
pub fn ui_cursor_shape_no_check_conceal() {
    if !full_screen.get() {
        return;
    }
    let new_mode_idx = cursor_get_mode_idx();
    if new_mode_idx != ui_mode_idx.get() {
        ui_mode_idx.set(new_mode_idx);
        pending_mode_update.set(true);
    }
}

/// [`ui_cursor_shape_no_check_conceal`], plus the concealment recheck the
/// cursor line needs when the mode changed.
pub fn ui_cursor_shape() {
    ui_cursor_shape_no_check_conceal();
    conceal_check_cursor_line();
}

fn ui_cursor_is_behind_floatwin() -> bool {
    if State.get() & MODE_CMDLINE != 0 || !ui_comp_should_draw() {
        return false;
    }
    let win = Win::current();
    let crow = win.w_winrow + win.w_winrow_off + win.w_wrow;
    let wcol = if win.w_onebuf_opt.wo_rl != 0 {
        win.w_view_width - win.w_wcol - 1
    } else {
        win.w_wcol
    };
    let ccol = win.w_wincol + win.w_wincol_off + wcol;
    let top_grid = ui_comp_get_grid_at_coord(crow, ccol);
    !top_grid.same(win_grid(win)) && !top_grid.same(default_grid_ref())
}

/// Whether `ext` is drawn by the UIs rather than into the grid.
pub fn ui_has(ext: UIExtension) -> bool {
    ui_ext.with(|widgets| widgets[ext as usize])
}

/// Describes every attached UI, for `nvim_list_uis()`.
pub fn ui_array() -> Array {
    uis.with(|table| {
        let mut all_uis = Array::with_capacity(table.len());
        for ui in table {
            all_uis.push(Object::dict(ui_info(ui)));
        }
        all_uis
    })
}

/// One UI's entry in [`ui_array`].
fn ui_info(ui: &RemoteUI) -> ApiDict {
    let mut info = ApiDict::with_capacity(10 + kUIExtCount as usize);
    let fixed: [(&'static str, Object); 9] = [
        ("width", Object::integer(ui.width as Integer)),
        ("height", Object::integer(ui.height as Integer)),
        ("rgb", Object::boolean(ui.rgb)),
        ("override", Object::boolean(ui.override_0)),
        ("term_name", Object::string(ui.term_name.clone())),
        // Reported empty rather than read back: the background a UI
        // sent is only meaningful to whoever sent it.
        ("term_background", Object::literal("")),
        ("term_colors", Object::integer(ui.term_colors as Integer)),
        ("stdin_tty", Object::boolean(ui.stdin_tty)),
        ("stdout_tty", Object::boolean(ui.stdout_tty)),
    ];
    for (key, value) in fixed {
        info.insert(key, value);
    }
    for (widget, name) in ui_ext_names.iter().enumerate().take(kUIExtCount as usize) {
        // A leading underscore marks an option outside the documented
        // protocol; those are only listed when turned on.
        let private = name.to_bytes().first() == Some(&b'_');
        if !private || ui.ui_ext[widget] {
            info.insert(name.to_bytes(), Object::boolean(ui.ui_ext[widget]));
        }
    }
    info.insert("chan", Object::integer(ui.channel_id as Integer));
    info
}

/// Resizes the grid a UI asked about, for `nvim_ui_try_resize_grid`.
///
/// Safe: the grid handle is looked up here, and every `unsafe` below rests on
/// that lookup rather than on anything the caller promised.
pub fn ui_grid_resize(grid_handle: Handle, width: c_int, height: c_int) -> Result<(), Error> {
    if grid_handle == DEFAULT_GRID_HANDLE {
        screen_resize(width, height);
        return Ok(());
    }
    let Some(mut wp) = get_win_by_grid_handle(grid_handle) else {
        return Err(err_bad_number(c"window handle", grid_handle as i64));
    };
    if wp.w_floating {
        if width != wp.w_width || height != wp.w_height {
            wp.w_config.width = width.max(1);
            wp.w_config.height = height.max(1);
            win_config_float(wp, wp.w_config.clone());
        }
    } else {
        // A split's size is a request: the layout decides what it gets.
        wp.w_height_request = height.max(0);
        wp.w_width_request = width.max(0);
        win_set_inner_size(wp, true);
    }
    Ok(())
}
