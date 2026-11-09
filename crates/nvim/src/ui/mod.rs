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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

mod callbacks;
mod mouse;
mod sinks;
pub(crate) mod state;

pub use callbacks::{UI_CB_EXT, ui_add_cb, ui_call_event, ui_remove_cb};
pub(crate) use mouse::MOUSE_VISUAL;
pub use mouse::{ui_check_mouse, ui_mouse_has};
pub use sinks::*;

use crate::api::ui::{remote_ui_hl_attr_define, remote_ui_hl_group_set, remote_ui_option_set};
use crate::autocmd::do_autocmd_uienter;
use crate::buffer::resettitle;
use crate::cursor_shape::{
    SHAPE_IDX_N, SHAPE_IDX_R, cursor_get_mode_idx, mode_style_array, shape_entry,
};
use crate::drawscreen::state::UPDATING_SCREEN;
use crate::drawscreen::{conceal_check_cursor_line, screen_resize};
use crate::event_queue::put_job_in;
use crate::ex_getln::cmdline_ui_flush;
use crate::getchar::state::EXPR_MAP_LOCK;
use crate::global_cell::GlobalCell;
use crate::grid::{GridRef, default_grid_ref, get_win_by_grid_handle};
use crate::guard::TEXTLOCK;
use crate::highlight::state::{
    CTERM_NORMAL_BG_COLOR, CTERM_NORMAL_FG_COLOR, NORMAL_BG, NORMAL_FG, NORMAL_SP,
};
use crate::highlight::{highlight_use_hlstate, ui_send_all_hls};
use crate::highlight_group::HLF_W;
use crate::message::state::{EMSG_SILENT, IN_ASSERT_FAILS};
use crate::message::{msg, msg_ext_ui_flush, msg_scroll_flush, msg_source, msg_ui_refresh};
use crate::narrow::as_index;
use crate::option::defs::{OptVal, OptionSetFlags};
use crate::option::vars::{
    BO_FLAGS, P_GUICURSOR, P_LZ, RDB_FLAGS, p_debug, p_lz, p_tgc, p_vb, p_wd,
};
use crate::option::{set_option_value, ui_refresh_options};
use crate::options::{OPT_BO_FLAG_ALL, OPT_CMDHEIGHT, OPT_RDB_FLAG_FLUSH, OPT_RDB_FLAG_LINE};
use crate::os::cshim::gettext;
use crate::os::fs::current_dir;
use crate::os::time::{os_hrtime, os_sleep};
use crate::startup::{EXITING, FULL_SCREEN, UI_CLIENT_CHANNEL_ID, starting};
use crate::state::MODE_CMDLINE;
use crate::state::mode::STATE;
use crate::types::DictBuilder;
use crate::types::{ApiDict, ApiString, Array, Boolean, Error, Handle, Integer, Object};
use crate::ui::state::{CALLED_VIM_BEEP, RESIZE_EVENTS, UI_EXT_NAMES, UI_REFRESH_CMDHEIGHT};
use crate::ui::state::{
    LINE_FLAG_INVALID, LINE_FLAG_WRAP, UI_EXT_COUNT, UI_FLOAT_DEBUG, UI_HL_STATE, UI_LINEGRID,
    UI_MESSAGES, UI_MULTIGRID, UI_TERM_COLORS,
};
use crate::ui::state::{LineFlags, RemoteUI, UIExtension};
use crate::ui_compositor::{
    ui_comp_attach, ui_comp_detach, ui_comp_get_grid_at_coord, ui_comp_init, ui_comp_should_draw,
};
use crate::window::{win_set_inner_size, win_ui_flush};
use crate::winfloat::win_config_float;
use core::ffi::c_int;

use crate::api::private::validate::err_bad_number;
use crate::winlayer::{Win, tabs};
/// The screen the editor draws into when nothing else claims a grid.
const DEFAULT_GRID_HANDLE: Handle = Handle::new(1);

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
static UIS: GlobalCell<Vec<RemoteUI>> = GlobalCell::new(Vec::new());

/// The widgets every attached UI draws itself, plus whatever
/// [`UI_CB_EXT`] adds. Read through [`ui_has`].
static UI_EXT: GlobalCell<[bool; UI_EXT_COUNT as usize]> =
    GlobalCell::new([false; UI_EXT_COUNT as usize]);

static UI_MODE_IDX: GlobalCell<c_int> = GlobalCell::new(SHAPE_IDX_N);
static CURSOR_ROW: GlobalCell<c_int> = GlobalCell::new(0);
static CURSOR_COL: GlobalCell<c_int> = GlobalCell::new(0);
static CURSOR_GRID_HANDLE: GlobalCell<Handle> = GlobalCell::new(DEFAULT_GRID_HANDLE);
static PENDING_CURSOR_UPDATE: GlobalCell<bool> = GlobalCell::new(false);
static PENDING_MODE_INFO_UPDATE: GlobalCell<bool> = GlobalCell::new(false);
static PENDING_MODE_UPDATE: GlobalCell<bool> = GlobalCell::new(false);
static PENDING_DEFAULT_COLORS: GlobalCell<bool> = GlobalCell::new(false);
/// `-1` until the first [`ui_flush`], so that the initial state is sent
/// whichever way it goes.
static PENDING_HAS_MOUSE: GlobalCell<c_int> = GlobalCell::new(-1);
/// Nesting depth of [`ui_busy_start`]/[`ui_busy_stop`]; the UIs only hear
/// about the outermost pair.
static BUSY: GlobalCell<c_int> = GlobalCell::new(0);

/// Which of the attached UIs a sink reaches.
///
/// A UI that did not ask for `ext_multigrid` is `Composed`: it is shown the
/// compositor's single flattened grid rather than the real ones, so the
/// grid events it gets are the compositor's, sent later and from a
/// different entry point.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// Every attached UI. Used by everything that is not about a grid.
    All,
    /// Only the UIs the compositor draws for.
    Composed,
    /// Only the UIs that see the real grids.
    Uncomposed,
}

/// The number of attached UIs. Never inlined: the `pub` callers are small
/// enough to ship across crates, and would take the table's cell with them.
#[inline(never)]
fn ui_count() -> usize {
    UIS.with(Vec::len)
}

/// Runs `f` on the UI attached to `channel_id`, if there is one.
///
/// A leaf: see [`UIS`].
pub(crate) fn with_ui<R>(channel_id: u64, f: impl FnOnce(&mut RemoteUI) -> R) -> Option<R> {
    let (mut f, mut answer) = (Some(f), None);
    lend_ui(channel_id, &mut |ui| {
        let f = f.take().expect("lent once");
        answer = Some(f(ui));
    });
    answer
}

/// [`with_ui`]'s table step. Not generic and never inlined, so the table's
/// cell is named in this crate only (`just state-exports`).
#[inline(never)]
fn lend_ui(channel_id: u64, f: &mut dyn FnMut(&mut RemoteUI)) {
    UIS.with_mut(|table| {
        if let Some(ui) = table.iter_mut().find(|ui| ui.channel_id == channel_id) {
            f(ui);
        }
    });
}

/// Whether a UI is attached to `channel_id`.
pub(crate) fn ui_is_attached(channel_id: u64) -> bool {
    any_ui(|ui| ui.channel_id == channel_id)
}

/// Whether any attached UI satisfies `test`.
fn any_ui(mut test: impl FnMut(&RemoteUI) -> bool) -> bool {
    any_ui_dyn(&mut test)
}

/// [`any_ui`]'s table step; not generic, as [`lend_ui`].
#[inline(never)]
fn any_ui_dyn(test: &mut dyn FnMut(&RemoteUI) -> bool) -> bool {
    UIS.with(|table| table.iter().any(test))
}

/// Runs `send` on every attached UI that `reach` selects, in attach order,
/// answering whether there was one. A leaf, as [`with_ui`].
pub(crate) fn each_ui_reached(reach: Reach, mut send: impl FnMut(&mut RemoteUI)) -> bool {
    each_ui_reached_dyn(reach, &mut send)
}

/// [`each_ui_reached`]'s table step; not generic, as [`lend_ui`].
#[inline(never)]
fn each_ui_reached_dyn(reach: Reach, send: &mut dyn FnMut(&mut RemoteUI)) -> bool {
    UIS.with_mut(|table| {
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
    UIS.with(|table| table.iter().map(|ui| ui.channel_id).collect())
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
        UI_CLIENT_CHANNEL_ID.get() == 0,
        "the UI client has no UIs of its own"
    );

    let mut width = c_int::MAX;
    let mut height = c_int::MAX;
    let inclusive = ui_override();
    // Start from all-true so the fold below is an intersection, but only if
    // there is something to intersect.
    let mut ext_widgets = [ui_active() != 0; UI_EXT_COUNT as usize];
    UIS.with(|table| {
        for ui in table {
            width = width.min(ui.width);
            height = height.min(ui.height);
            for (widget, enabled) in ext_widgets[..UI_EXT_COUNT as usize].iter_mut().enumerate() {
                *enabled &= ui.ui_ext[widget] || inclusive;
            }
        }
    });

    CURSOR_ROW.set(0);
    CURSOR_COL.set(0);
    PENDING_CURSOR_UPDATE.set(true);

    let had_message = ui_has(UI_MESSAGES);
    for (widget, &wanted) in ext_widgets[..UI_EXT_COUNT as usize].iter().enumerate() {
        let enabled = wanted || UI_CB_EXT.with(|cb| cb[widget]);
        UI_EXT.with_mut(|widgets| widgets[widget] = enabled);
        // The widgets past `ext_linegrid` describe how a UI wants the
        // screen expressed rather than who draws what, and are not options
        // a UI can be told about.
        if widget < UI_LINEGRID as usize {
            let name = ApiString::from_cstr(UI_EXT_NAMES[widget]);
            ui_call_option_set(name, Object::boolean(enabled));
        }
    }

    if had_message != ui_has(UI_MESSAGES) {
        // Messages moving in or out of the grid changes how many lines the
        // message area needs. `UI_REFRESH_CMDHEIGHT` is off while the
        // user's own 'cmdheight' is being restored.
        if UI_REFRESH_CMDHEIGHT.get() {
            let _ = set_option_value(
                OPT_CMDHEIGHT,
                OptVal::Number(had_message.into()),
                OptionSetFlags::NONE,
            );
            for mut tp in tabs() {
                tp.tp_ch_used = had_message.into();
            }
        }
        msg_scroll_flush();
    }
    msg_ui_refresh();

    if ui_active() == 0 {
        return;
    }
    if UPDATING_SCREEN.get() {
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
    PENDING_MODE_UPDATE.set(true);
    ui_cursor_shape();
    PENDING_HAS_MOUSE.set(-1);
}

/// The tallest popup menu every attached UI will draw, or zero if none of
/// them asked for a limit.
pub fn ui_pum_get_height() -> c_int {
    UIS.with(|table| {
        let limits = table.iter().map(|ui| ui.pum_nlines).filter(|&n| n != 0);
        limits.min().unwrap_or(0)
    })
}

/// The geometry of the externally drawn popup menu, if a UI reported one:
/// width, height, row and column.
pub fn ui_pum_get_pos() -> Option<[f64; 4]> {
    UIS.with(|table| {
        let ui = table.iter().find(|ui| ui.pum_pos)?;
        Some([ui.pum_width, ui.pum_height, ui.pum_row, ui.pum_col])
    })
}

/// Queues [`ui_refresh`] for after the current redraw.
pub fn ui_schedule_refresh() {
    put_job_in(&RESIZE_EVENTS, Box::new(ui_refresh));
}

/// Marks the default colours as needing to be re-sent.
pub fn ui_default_colors_set() {
    PENDING_DEFAULT_COLORS.set(true);
    // Before startup finishes the colours are still being computed, and
    // the pending flag is enough — `ui_line` picks it up.
    if starting.get() == 0 {
        ui_may_set_default_colors();
    }
}

fn ui_may_set_default_colors() {
    if !PENDING_DEFAULT_COLORS.get() {
        return;
    }
    PENDING_DEFAULT_COLORS.set(false);
    ui_call_default_colors_set(
        Integer::from(NORMAL_FG.get()),
        Integer::from(NORMAL_BG.get()),
        Integer::from(NORMAL_SP.get()),
        Integer::from(CTERM_NORMAL_FG_COLOR.get()),
        Integer::from(CTERM_NORMAL_BG_COLOR.get()),
    );
}

/// Enters the busy state, in which UIs stop drawing a cursor.
pub fn ui_busy_start() {
    BUSY.set(BUSY.get() + 1);
    if BUSY.get() == 1 {
        ui_call_busy_start();
    }
}

/// Leaves it. Only the outermost [`ui_busy_start`] is announced.
pub fn ui_busy_stop() {
    BUSY.set(BUSY.get() - 1);
    if BUSY.get() == 0 {
        ui_call_busy_stop();
    }
}

/// Beeps, unless `'belloff'` covers `val`.
pub fn vim_beep(val: core::ffi::c_uint) {
    CALLED_VIM_BEEP.set(true);
    if EMSG_SILENT.get() != 0 || IN_ASSERT_FAILS.get() {
        return;
    }
    if BO_FLAGS.get() & val == 0 && BO_FLAGS.get() & OPT_BO_FLAG_ALL == 0 {
        // At most three beeps per half second: a stuck macro that beeps on
        // every iteration should not make the terminal unusable.
        static BEEPS: GlobalCell<c_int> = GlobalCell::new(0);
        static START_TIME: GlobalCell<u64> = GlobalCell::new(0);
        if START_TIME.get() == 0 || os_hrtime() - START_TIME.get() > 500_000_000 {
            BEEPS.set(0);
            START_TIME.set(os_hrtime());
        }
        BEEPS.set(BEEPS.get() + 1);
        if BEEPS.get() <= 3 {
            if p_vb() {
                ui_call_visual_bell();
            } else {
                ui_call_bell();
            }
        }
    }
    if p_debug(|value| value.to_bytes().contains(&b'e')) {
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
    let multigrid = ui.ui_ext[UI_MULTIGRID as usize];
    let float_debug = ui.ui_ext[UI_FLOAT_DEBUG as usize];
    if !multigrid && !float_debug && UI_CLIENT_CHANNEL_ID.get() == 0 {
        ui_comp_attach(&mut ui);
    }
    UIS.with_mut(|table| table.push(ui));

    // Both of these can run user code (a 'titlestring' expression), which
    // can detach the UI again: from here on it is found by id each time.
    ui_refresh_options();
    resettitle();

    // Tell it where the server is, so that a UI on another machine can
    // resolve the paths the server sends.
    if let Some(cwd) = current_dir() {
        ui_call_chdir(ApiString::from_xstring(cwd));
    }

    for widget in UI_LINEGRID as usize..UI_EXT_COUNT as usize {
        let Some(active) = with_ui(chanid, |ui| ui.ui_ext[widget]) else {
            break;
        };
        ui_set_ext_option(
            chanid,
            UIExtension::try_from(widget).expect("a UI extension index"),
            active,
        );
    }

    // Highlights are sent as definitions plus ids, unless this is the
    // first UI to want that, in which case the whole table is rebuilt and
    // resent by `highlight_use_hlstate` itself.
    let hl_state = with_ui(chanid, |ui| ui.ui_ext[UI_HL_STATE as usize]).unwrap_or(false);
    let sent = hl_state && highlight_use_hlstate();
    if !sent {
        ui_send_all_hls(
            &mut |id, attr, info| {
                with_ui(chanid, |ui| {
                    remote_ui_hl_attr_define(ui, id, &attr, &attr, info)
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
    let mut ui = UIS.with_mut(|table| {
        let index = table.iter().position(|ui| ui.channel_id == chanid)?;
        Some(table.remove(index))
    })?;

    if ui_count() != 0 && !EXITING.get() {
        // The screen size is the smallest attached UI's, so losing one can
        // make the screen bigger.
        ui_schedule_refresh();
    }
    let multigrid = ui.ui_ext[UI_MULTIGRID as usize];
    let float_debug = ui.ui_ext[UI_FLOAT_DEBUG as usize];
    if !multigrid && !float_debug {
        ui_comp_detach(&mut ui);
    }
    do_autocmd_uienter(chanid, false);
    Some(ui)
}

/// Tells the UI on `chanid` that `ext` changed.
pub fn ui_set_ext_option(chanid: u64, ext: UIExtension, active: bool) {
    if ext < UI_LINEGRID {
        // An external widget changing changes the intersection, and every
        // UI has to be told the result rather than this one's request.
        ui_refresh();
        return;
    }
    let name = UI_EXT_NAMES[ext as usize];
    // A leading underscore marks an option that is not part of the
    // documented protocol; those are only mentioned when turned on.
    let private = name.to_bytes().first() == Some(&b'_');
    if !private || active {
        with_ui(chanid, |ui| {
            remote_ui_option_set(ui, ApiString::from_cstr(name), Object::boolean(active));
        });
    }
    if ext == UI_TERM_COLORS {
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

    let mut flags: LineFlags = if wrap { LINE_FLAG_WRAP } else { 0 };
    if startcol == 0 && invalid_row {
        flags |= LINE_FLAG_INVALID;
    }
    ui_may_set_default_colors();

    let off = grid.cell_offset(row, startcol);
    let (chars, attrs) = grid.cells(off, as_index(grid.cols - startcol));
    let handle = Integer::from(grid.handle);
    let (at_row, from) = (Integer::from(row), Integer::from(startcol));
    let (to, clear_from) = (Integer::from(endcol), Integer::from(clearcol));
    let clear_attr = Integer::from(clearattr);
    ui_call_raw_line(
        handle, at_row, from, to, clear_from, clear_attr, flags, chars, attrs,
    );

    // 'writedelay' with `redrawdebug=line`: park the cursor at the end of
    // the line just drawn and pause, so the draw order is watchable.
    if p_wd() != 0 && RDB_FLAGS.get() & OPT_RDB_FLAG_LINE != 0 {
        ui_call_grid_cursor_goto(
            Integer::from(grid.handle),
            Integer::from(row),
            Integer::from(clearcol.min(grid.cols - 1)),
        );
        ui_call_flush();
        os_sleep(p_wd().unsigned_abs());
        PENDING_CURSOR_UPDATE.set(true);
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
    if new_row == CURSOR_ROW.get()
        && new_col == CURSOR_COL.get()
        && grid_handle == CURSOR_GRID_HANDLE.get()
    {
        return;
    }
    CURSOR_ROW.set(new_row);
    CURSOR_COL.set(new_col);
    CURSOR_GRID_HANDLE.set(grid_handle);
    PENDING_CURSOR_UPDATE.set(true);
}

/// Re-sends the cursor if it is on `grid_handle`, which has been redrawn
/// underneath it.
pub fn ui_check_cursor_grid(grid_handle: Handle) {
    if CURSOR_GRID_HANDLE.get() == grid_handle {
        PENDING_CURSOR_UPDATE.set(true);
    }
}

/// Marks `'guicursor'` as needing to be re-sent.
pub fn ui_mode_info_set() {
    PENDING_MODE_INFO_UPDATE.set(true);
}

/// The row the cursor was last put on.
pub fn ui_current_row() -> c_int {
    CURSOR_ROW.get()
}

/// The column the cursor was last put on.
pub fn ui_current_col() -> c_int {
    CURSOR_COL.get()
}

/// Ends a redraw: sends everything held back, then `flush`.
///
/// Until a UI sees `flush` it is free to show nothing of what came before,
/// so this is what makes a redraw visible.
pub fn ui_flush() {
    assert!(
        UI_CLIENT_CHANNEL_ID.get() == 0,
        "the UI client has no UIs of its own"
    );
    if ui_active() == 0 {
        return;
    }

    // A hidden floating window is still the current window, and the cursor
    // is inside it, so there is nowhere to draw it: look busy instead.
    static WAS_BUSY: GlobalCell<bool> = GlobalCell::new(false);
    let cursor_nowhere = STATE.get() & MODE_CMDLINE == 0
        && Win::current().w_floating
        && Win::current().w_config.hide;
    if cursor_nowhere {
        if !WAS_BUSY.get() {
            ui_call_busy_start();
            WAS_BUSY.set(true);
        }
    } else if WAS_BUSY.get() {
        ui_call_busy_stop();
        WAS_BUSY.set(false);
    }

    win_ui_flush(false);
    if TEXTLOCK.get() == 0 && EXPR_MAP_LOCK.get() == 0 {
        // Both can run Lua handlers, which the locks exist to keep out.
        cmdline_ui_flush();
        msg_ext_ui_flush();
    }
    msg_scroll_flush();

    if PENDING_CURSOR_UPDATE.get() {
        ui_call_grid_cursor_goto(
            Integer::from(CURSOR_GRID_HANDLE.get()),
            Integer::from(CURSOR_ROW.get()),
            Integer::from(CURSOR_COL.get()),
        );
        PENDING_CURSOR_UPDATE.set(false);
        // Moving the cursor can uncover a window whose viewport changed.
        win_ui_flush(false);
    }

    if PENDING_MODE_INFO_UPDATE.get() {
        let style = mode_style_array();
        let enabled = P_GUICURSOR.first_byte() != 0;
        ui_call_mode_info_set(enabled as Boolean, style);
        PENDING_MODE_INFO_UPDATE.set(false);
    }

    // A cursor behind a floating window is reported as the "replace" shape,
    // which is the closest the protocol has to "do not draw one".
    static CURSOR_WAS_OBSCURED: GlobalCell<bool> = GlobalCell::new(false);
    let cursor_obscured = ui_cursor_is_behind_floatwin();
    if (cursor_obscured != CURSOR_WAS_OBSCURED.get() || PENDING_MODE_UPDATE.get())
        && starting.get() == 0
    {
        let idx = if cursor_obscured {
            SHAPE_IDX_R
        } else {
            UI_MODE_IDX.get()
        };
        let full_name = shape_entry(idx).full_name;
        ui_call_mode_change(ApiString::from_cstr(full_name), Integer::from(idx));
        PENDING_MODE_UPDATE.set(false);
        CURSOR_WAS_OBSCURED.set(cursor_obscured);
    }

    if PENDING_HAS_MOUSE.get() != c_int::from(mouse::wanted()) {
        if mouse::wanted() {
            ui_call_mouse_on();
        } else {
            ui_call_mouse_off();
        }
        PENDING_HAS_MOUSE.set(c_int::from(mouse::wanted()));
    }

    ui_call_flush();

    if p_wd() != 0 && RDB_FLAGS.get() & OPT_RDB_FLAG_FLUSH != 0 {
        os_sleep(p_wd().unsigned_abs());
    }
}

/// Recomputes the cursor shape for the current mode, without disturbing
/// `'conceallevel'`. For the paths that are about to redraw anyway.
pub fn ui_cursor_shape_no_check_conceal() {
    if !FULL_SCREEN.get() {
        return;
    }
    let new_mode_idx = cursor_get_mode_idx();
    if new_mode_idx != UI_MODE_IDX.get() {
        UI_MODE_IDX.set(new_mode_idx);
        PENDING_MODE_UPDATE.set(true);
    }
}

/// [`ui_cursor_shape_no_check_conceal`], plus the concealment recheck the
/// cursor line needs when the mode changed.
pub fn ui_cursor_shape() {
    ui_cursor_shape_no_check_conceal();
    conceal_check_cursor_line();
}

fn ui_cursor_is_behind_floatwin() -> bool {
    if STATE.get() & MODE_CMDLINE != 0 || !ui_comp_should_draw() {
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
    !top_grid.same(win.grid_alloc()) && !top_grid.same(default_grid_ref())
}

/// Whether `ext` is drawn by the UIs rather than into the grid.
pub fn ui_has(ext: UIExtension) -> bool {
    UI_EXT.with(|widgets| widgets[ext as usize])
}

/// Describes every attached UI, for `nvim_list_uis()`.
pub fn ui_array() -> Array {
    UIS.with(|table| table.iter().map(|ui| Object::dict(ui_info(ui))).collect())
}

/// One UI's entry in [`ui_array`].
fn ui_info(ui: &RemoteUI) -> ApiDict {
    let mut info = DictBuilder::with_capacity(10 + UI_EXT_COUNT as usize);
    let fixed: [(&'static str, Object); 9] = [
        ("width", Object::integer(Integer::from(ui.width))),
        ("height", Object::integer(Integer::from(ui.height))),
        ("rgb", Object::boolean(ui.rgb)),
        ("override", Object::boolean(ui.override_0)),
        ("term_name", Object::string(ui.term_name.clone())),
        // Reported empty rather than read back: the background a UI
        // sent is only meaningful to whoever sent it.
        ("term_background", Object::literal("")),
        (
            "term_colors",
            Object::integer(Integer::from(ui.term_colors)),
        ),
        ("stdin_tty", Object::boolean(ui.stdin_tty)),
        ("stdout_tty", Object::boolean(ui.stdout_tty)),
    ];
    for (key, value) in fixed {
        info.insert(key, value);
    }
    for (widget, name) in UI_EXT_NAMES.iter().enumerate().take(UI_EXT_COUNT as usize) {
        // A leading underscore marks an option outside the documented
        // protocol; those are only listed when turned on.
        let private = name.to_bytes().first() == Some(&b'_');
        if !private || ui.ui_ext[widget] {
            info.insert(name.to_bytes(), Object::boolean(ui.ui_ext[widget]));
        }
    }
    info.insert("chan", Object::integer(ui.channel_id.cast_signed()));
    info.into()
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
        return Err(err_bad_number(c"window handle", Integer::from(grid_handle)));
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
