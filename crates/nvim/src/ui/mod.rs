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

#![deny(unsafe_op_in_unsafe_fn)]

mod callbacks;
mod mouse;
mod sinks;

pub use callbacks::{ui_add_cb, ui_call_event, ui_cb_ext, ui_remove_cb};
pub use mouse::{ui_check_mouse, ui_mouse_has};
pub use sinks::*;

use crate::api::private::helpers::{arena_array, arena_dict, cstr_as_string};
use crate::api::private::validate::api_err_invalid;
use crate::api::ui::remote_ui_option_set;
use crate::autocmd::do_autocmd_uienter;
use crate::buffer::resettitle;
use crate::cursor_shape::{
    SHAPE_IDX_N, SHAPE_IDX_R, cursor_get_mode_idx, mode_style_array, shape_entry,
};
use crate::drawscreen::{conceal_check_cursor_line, screen_resize};
use crate::event::libuv::uv_cwd;
use crate::event::multiqueue::multiqueue_put_event;
use crate::ex_getln::cmdline_ui_flush;
use crate::global_cell::GlobalCell;
use crate::grid::get_win_by_grid_handle;
use crate::highlight::{highlight_use_hlstate, ui_send_all_hls};
use crate::highlight_group::HLF_W;
use crate::main::{
    State, called_vim_beep, cterm_normal_bg_color, cterm_normal_fg_color, curwin, default_grid,
    emsg_silent, exiting, expr_map_lock, first_tabpage, full_screen, in_assert_fails, msg_grid_adj,
    normal_bg, normal_fg, normal_sp, p_debug, p_guicursor, p_lz, p_tgc, p_vb, p_wd, rdb_flags,
    resize_events, starting, textlock, ui_client_channel_id, ui_ext_names, ui_refresh_cmdheight,
    updating_screen,
};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::message::{msg, msg_ext_ui_flush, msg_scroll_flush, msg_source, msg_ui_refresh};
use crate::option::{kOptValTypeNumber, set_option_value, ui_refresh_options};
use crate::options::{kOptBoFlagAll, kOptCmdheight, kOptRdbFlagFlush, kOptRdbFlagLine};
use crate::os::cshim::gettext;
use crate::os::time::{os_hrtime, os_sleep};
use crate::state::MODE_CMDLINE;
use crate::strings::vim_strchr;
use crate::types::builders::static_string;
use crate::types::ui::{
    kLineFlagInvalid, kLineFlagWrap, kUIExtCount, kUIFloatDebug, kUIHlState, kUILinegrid,
    kUIMessages, kUIMultigrid, kUITermColors,
};
use crate::types::{
    Arena, Array, Boolean, Dict, Error, Integer, KeyValuePair, LineFlags, Object, OptVal,
    OptValData, OptionSetFlags, RemoteUI, ScreenGrid, String_0, UIExtension, handle_T,
};
use crate::ui_compositor::{
    ui_comp_attach, ui_comp_detach, ui_comp_get_grid_at_coord, ui_comp_init, ui_comp_should_draw,
};
use crate::window::{win_set_inner_size, win_ui_flush};
use crate::winfloat::win_config_float;
use core::ffi::c_int;

/// The screen the editor draws into when nothing else claims a grid.
const DEFAULT_GRID_HANDLE: handle_T = 1;

/// How many UIs may be attached at once.
///
/// The table is a fixed array because the count is small and the sinks walk
/// it on every screen line; growing it would be a wire-protocol question,
/// not an allocation one.
pub const MAX_UI_COUNT: usize = 16;

/// The attached UIs, oldest first. Only the first [`ui_count`] are live.
static uis: GlobalCell<[*mut RemoteUI; MAX_UI_COUNT]> =
    GlobalCell::new([core::ptr::null_mut(); MAX_UI_COUNT]);
static attached: GlobalCell<usize> = GlobalCell::new(0);

/// The widgets every attached UI draws itself, plus whatever
/// [`ui_cb_ext`] adds. Read through [`ui_has`].
static ui_ext: GlobalCell<[bool; kUIExtCount as usize]> =
    GlobalCell::new([false; kUIExtCount as usize]);

static ui_mode_idx: GlobalCell<c_int> = GlobalCell::new(SHAPE_IDX_N);
static cursor_row: GlobalCell<c_int> = GlobalCell::new(0);
static cursor_col: GlobalCell<c_int> = GlobalCell::new(0);
static cursor_grid_handle: GlobalCell<handle_T> = GlobalCell::new(DEFAULT_GRID_HANDLE);
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
    attached.get()
}

/// The `index`th attached UI.
///
/// # Panics
///
/// Past [`ui_count`], where the entry is a stale pointer or null.
fn ui_at(index: usize) -> *mut RemoteUI {
    assert!(index < ui_count(), "UI index past the attached count");
    uis.with(|table| table[index])
}

/// Whether `reach` selects `ui`.
fn reaches(ui: *mut RemoteUI, reach: Reach) -> bool {
    match reach {
        Reach::All => true,
        // Safe by the table's invariant: entries below `ui_count` are live.
        Reach::Composed => unsafe { (*ui).composed },
        Reach::Uncomposed => !unsafe { (*ui).composed },
    }
}

/// # Safety
///
/// Call once, before any grid is drawn.
pub unsafe fn ui_init() {
    unsafe {
        (*default_grid.ptr()).handle = DEFAULT_GRID_HANDLE;
        (*msg_grid_adj.ptr()).target = default_grid.ptr();
        ui_comp_init();
    }
}

/// Whether the editor should resolve highlights to 24-bit colour.
///
/// `'termguicolors'` forces it. Otherwise it takes one non-terminal UI
/// asking for RGB: a terminal UI's `rgb` flag describes the terminal, which
/// the TUI has already accounted for.
pub fn ui_rgb_attached() -> bool {
    p_tgc.get() != 0 || each_ui().any(|ui| !is_tui(ui) && unsafe { (*ui).rgb })
}

/// Whether any attached UI is something other than a terminal.
pub fn ui_gui_attached() -> bool {
    each_ui().any(|ui| !is_tui(ui))
}

/// Whether any attached UI asked to be given every external widget whether
/// or not the others want them. `nvim --embed`'s debugging escape hatch.
pub fn ui_override() -> bool {
    each_ui().any(|ui| unsafe { (*ui).override_0 })
}

/// The number of attached UIs.
pub fn ui_active() -> usize {
    ui_count()
}

fn each_ui() -> impl Iterator<Item = *mut RemoteUI> {
    (0..ui_count()).map(ui_at)
}

fn is_tui(ui: *mut RemoteUI) -> bool {
    unsafe { (*ui).stdin_tty || (*ui).stdout_tty }
}

/// Renegotiates what the attached UIs can do, and resizes to match.
///
/// The screen becomes the smallest attached UI's, and a widget is external
/// only if every UI asked for it — a UI that cannot draw its own popup menu
/// must be sent one drawn into the grid, and the others cannot be sent a
/// different screen. [`ui_override`] opts out of the intersection.
///
/// # Safety
///
/// Not callable from the UI client process, which has no attach table.
pub unsafe fn ui_refresh() {
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
    for ui in each_ui() {
        let ui = unsafe { &*ui };
        width = width.min(ui.width);
        height = height.min(ui.height);
        for (widget, enabled) in ext_widgets[..kUIExtCount as usize].iter_mut().enumerate() {
            *enabled &= ui.ui_ext[widget] || inclusive;
        }
    }

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
            let name = ext_name(widget);
            ui_call_option_set(
                unsafe { cstr_as_string(name.cast_mut()) },
                Object::boolean(enabled),
            );
        }
    }

    if had_message != ui_has(kUIMessages) {
        // Messages moving in or out of the grid changes how many lines the
        // message area needs. `ui_refresh_cmdheight` is off while the
        // user's own 'cmdheight' is being restored.
        if ui_refresh_cmdheight.get() {
            set_option_value(
                kOptCmdheight,
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: had_message as _,
                    },
                },
                OptionSetFlags::NONE,
            );
            let mut tp = first_tabpage.get();
            while !tp.is_null() {
                unsafe {
                    (*tp).tp_ch_used = had_message as _;
                    tp = (*tp).tp_next;
                }
            }
        }
        unsafe { msg_scroll_flush() };
    }
    unsafe { msg_ui_refresh() };

    if ui_active() == 0 {
        return;
    }
    if updating_screen.get() {
        // Resizing mid-redraw would invalidate the grid being drawn into.
        ui_schedule_refresh();
        return;
    }
    unsafe {
        ui_default_colors_set();
        // 'lazyredraw' would defer the resize past the point the UIs are
        // told about it.
        let save_p_lz = p_lz.get();
        p_lz.set(0);
        screen_resize(width, height);
        p_lz.set(save_p_lz);
        ui_mode_info_set();
        pending_mode_update.set(true);
        ui_cursor_shape();
    }
    pending_has_mouse.set(-1);
}

/// The tallest popup menu every attached UI will draw, or zero if none of
/// them asked for a limit.
pub fn ui_pum_get_height() -> c_int {
    each_ui()
        .filter_map(|ui| Some(unsafe { (*ui).pum_nlines }).filter(|&n| n != 0))
        .min()
        .unwrap_or(0)
}

/// The geometry of the externally drawn popup menu, if a UI reported one.
///
/// # Safety
///
/// The four out-parameters must be writable.
pub unsafe fn ui_pum_get_pos(
    pwidth: *mut f64,
    pheight: *mut f64,
    prow: *mut f64,
    pcol: *mut f64,
) -> bool {
    let Some(ui) = each_ui().find(|&ui| unsafe { (*ui).pum_pos }) else {
        return false;
    };
    unsafe {
        *pwidth = (*ui).pum_width;
        *pheight = (*ui).pum_height;
        *prow = (*ui).pum_row;
        *pcol = (*ui).pum_col;
    }
    true
}

unsafe extern "C" fn ui_refresh_event(_argv: *mut *mut core::ffi::c_void) {
    unsafe { ui_refresh() };
}

/// Queues [`ui_refresh`] for after the current redraw.
pub fn ui_schedule_refresh() {
    unsafe {
        multiqueue_put_event(
            resize_events.get(),
            crate::types::Event::new(Some(ui_refresh_event), []),
        )
    };
}

/// Marks the default colours as needing to be re-sent.
///
/// # Safety
///
/// Reads the resolved `Normal` highlight.
pub unsafe fn ui_default_colors_set() {
    pending_default_colors.set(true);
    // Before startup finishes the colours are still being computed, and
    // the pending flag is enough — `ui_line` picks it up.
    if starting.get() == 0 {
        unsafe { ui_may_set_default_colors() };
    }
}

/// # Safety
///
/// As [`ui_default_colors_set`].
unsafe fn ui_may_set_default_colors() {
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
///
/// # Safety
///
/// Reads `'debug'` and may emit a message.
pub unsafe fn vim_beep(val: core::ffi::c_uint) {
    use crate::main::bo_flags;

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
            if p_vb.get() != 0 {
                ui_call_visual_bell();
            } else {
                ui_call_bell();
            }
        }
    }
    if !unsafe { vim_strchr(p_debug.get(), 'e' as c_int) }.is_null() {
        unsafe {
            msg_source(HLF_W);
            msg(gettext(c"Beep!".as_ptr()), HLF_W);
        }
    }
}

/// Fires `UIEnter` once per attached UI. Startup's catch-up, for the UIs
/// that attached before autocommands were running.
///
/// # Safety
///
/// Runs autocommands.
pub unsafe fn do_autocmd_uienter_all() {
    for ui in each_ui() {
        unsafe { do_autocmd_uienter((*ui).channel_id, true) };
    }
}

/// Whether another `nvim_ui_attach` would fit.
pub fn ui_can_attach_more() -> bool {
    ui_count() < MAX_UI_COUNT
}

/// Adds `ui` to the attach table and brings it up to date.
///
/// # Safety
///
/// `ui` must be a live [`RemoteUI`] not already attached, and must outlive
/// its [`ui_detach_impl`].
pub unsafe fn ui_attach_impl(ui: *mut RemoteUI, chanid: u64) {
    assert!(ui_can_attach_more(), "attach table is full");
    // A UI that reads the real grids does not want the compositor's
    // flattened one. `_debug_float` is the same thing for debugging.
    let multigrid = unsafe { (*ui).ui_ext[kUIMultigrid as usize] };
    let float_debug = unsafe { (*ui).ui_ext[kUIFloatDebug as usize] };
    if !multigrid && !float_debug && ui_client_channel_id.get() == 0 {
        unsafe { ui_comp_attach(ui) };
    }
    uis.with_mut(|table| table[attached.get()] = ui);
    attached.set(attached.get() + 1);

    unsafe {
        ui_refresh_options();
        resettitle();
    }

    // Tell it where the server is, so that a UI on another machine can
    // resolve the paths the server sends.
    let mut cwd = [0; 4096];
    let mut cwdlen = cwd.len();
    if unsafe { uv_cwd(cwd.as_mut_ptr(), &raw mut cwdlen) } == 0 {
        ui_call_chdir(String_0::from_raw_parts(cwd.as_mut_ptr(), cwdlen));
    }

    for widget in kUILinegrid as usize..kUIExtCount as usize {
        unsafe { ui_set_ext_option(ui, widget as UIExtension, (*ui).ui_ext[widget]) };
    }

    // Highlights are sent as definitions plus ids, unless this is the
    // first UI to want that, in which case the whole table is rebuilt and
    // resent by `highlight_use_hlstate` itself.
    let sent = unsafe { (*ui).ui_ext[kUIHlState as usize] } && unsafe { highlight_use_hlstate() };
    if !sent {
        unsafe { ui_send_all_hls(ui) };
    }
    unsafe {
        ui_refresh();
        do_autocmd_uienter(chanid, true);
    }
}

/// Removes `ui` from the attach table.
///
/// # Safety
///
/// `ui` must currently be attached.
pub unsafe fn ui_detach_impl(ui: *mut RemoteUI, chanid: u64) {
    let index = each_ui()
        .position(|candidate| candidate == ui)
        .expect("detaching a UI that is not attached");
    uis.with_mut(|table| table.copy_within(index + 1..attached.get(), index));
    attached.set(attached.get() - 1);

    if ui_count() != 0 && !exiting.get() {
        // The screen size is the smallest attached UI's, so losing one can
        // make the screen bigger.
        ui_schedule_refresh();
    }
    let multigrid = unsafe { (*ui).ui_ext[kUIMultigrid as usize] };
    let float_debug = unsafe { (*ui).ui_ext[kUIFloatDebug as usize] };
    if !multigrid && !float_debug {
        unsafe { ui_comp_detach(ui) };
    }
    unsafe { do_autocmd_uienter(chanid, false) };
}

/// Tells `ui` that `ext` changed.
///
/// # Safety
///
/// `ui` must be live.
pub unsafe fn ui_set_ext_option(ui: *mut RemoteUI, ext: UIExtension, active: bool) {
    if ext < kUILinegrid {
        // An external widget changing changes the intersection, and every
        // UI has to be told the result rather than this one's request.
        unsafe { ui_refresh() };
        return;
    }
    let name = unsafe {
        *ui_ext_names
            .ptr()
            .cast::<*const core::ffi::c_char>()
            .add(ext as usize)
    };
    // A leading underscore marks an option that is not part of the
    // documented protocol; those are only mentioned when turned on.
    let private = unsafe { *name } == b'_' as core::ffi::c_char;
    if !private || active {
        unsafe {
            remote_ui_option_set(ui, cstr_as_string(name.cast_mut()), Object::boolean(active))
        };
    }
    if ext == kUITermColors {
        unsafe { ui_default_colors_set() };
    }
}

/// Sends one drawn screen line.
///
/// # Safety
///
/// `grid` must be live and `row` within it.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn ui_line(
    grid: *mut ScreenGrid,
    row: c_int,
    invalid_row: bool,
    startcol: c_int,
    endcol: c_int,
    clearcol: c_int,
    clearattr: c_int,
    wrap: bool,
) {
    let grid = unsafe { &mut *grid };
    assert!((0..grid.rows).contains(&row), "row is outside the grid");

    let mut flags: LineFlags = if wrap { kLineFlagWrap } else { 0 };
    if startcol == 0 && invalid_row {
        flags |= kLineFlagInvalid;
    }
    unsafe { ui_may_set_default_colors() };

    let off = unsafe { *grid.line_offset.add(row as usize) } + startcol as usize;
    unsafe {
        ui_call_raw_line(
            grid.handle as Integer,
            row as Integer,
            startcol as Integer,
            endcol as Integer,
            clearcol as Integer,
            clearattr as Integer,
            flags,
            grid.chars.cast_const().add(off),
            grid.attrs.cast_const().add(off),
        )
    };

    // 'writedelay' with `redrawdebug=line`: park the cursor at the end of
    // the line just drawn and pause, so the draw order is watchable.
    if p_wd.get() != 0 && rdb_flags.get() & kOptRdbFlagLine as core::ffi::c_uint != 0 {
        ui_call_grid_cursor_goto(
            grid.handle as Integer,
            row as Integer,
            clearcol.min(grid.cols - 1) as Integer,
        );
        ui_call_flush();
        os_sleep(p_wd.get().unsigned_abs());
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
pub fn ui_grid_cursor_goto(grid_handle: handle_T, new_row: c_int, new_col: c_int) {
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
pub fn ui_check_cursor_grid(grid_handle: handle_T) {
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
///
/// # Safety
///
/// Not callable from the UI client process.
pub unsafe fn ui_flush() {
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
        && unsafe { (*curwin.get()).w_floating && (*curwin.get()).w_config.hide };
    if cursor_nowhere {
        if !was_busy.get() {
            ui_call_busy_start();
            was_busy.set(true);
        }
    } else if was_busy.get() {
        ui_call_busy_stop();
        was_busy.set(false);
    }

    unsafe { win_ui_flush(false) };
    if textlock.get() == 0 && expr_map_lock.get() == 0 {
        // Both can run Lua handlers, which the locks exist to keep out.
        unsafe {
            cmdline_ui_flush();
            msg_ext_ui_flush();
        }
    }
    unsafe { msg_scroll_flush() };

    if pending_cursor_update.get() {
        ui_call_grid_cursor_goto(
            cursor_grid_handle.get() as Integer,
            cursor_row.get() as Integer,
            cursor_col.get() as Integer,
        );
        pending_cursor_update.set(false);
        // Moving the cursor can uncover a window whose viewport changed.
        unsafe { win_ui_flush(false) };
    }

    if pending_mode_info_update.get() {
        let mut arena: Arena = ARENA_EMPTY;
        let style = unsafe { mode_style_array(&raw mut arena) };
        let enabled = unsafe { *p_guicursor.get() } != 0;
        ui_call_mode_info_set(enabled as Boolean, style);
        unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
        pending_mode_info_update.set(false);
    }

    // A cursor behind a floating window is reported as the "replace" shape,
    // which is the closest the protocol has to "do not draw one".
    static cursor_was_obscured: GlobalCell<bool> = GlobalCell::new(false);
    let cursor_obscured = unsafe { ui_cursor_is_behind_floatwin() };
    if (cursor_obscured != cursor_was_obscured.get() || pending_mode_update.get())
        && starting.get() == 0
    {
        let idx = if cursor_obscured {
            SHAPE_IDX_R
        } else {
            ui_mode_idx.get()
        };
        let full_name = shape_entry(idx).full_name;
        ui_call_mode_change(unsafe { cstr_as_string(full_name) }, idx as Integer);
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

    if p_wd.get() != 0 && rdb_flags.get() & kOptRdbFlagFlush as core::ffi::c_uint != 0 {
        os_sleep(p_wd.get().unsigned_abs());
    }
}

/// Recomputes the cursor shape for the current mode, without disturbing
/// `'conceallevel'`. For the paths that are about to redraw anyway.
///
/// # Safety
///
/// Reads the mode tables.
pub unsafe fn ui_cursor_shape_no_check_conceal() {
    if !full_screen.get() {
        return;
    }
    let new_mode_idx = unsafe { cursor_get_mode_idx() };
    if new_mode_idx != ui_mode_idx.get() {
        ui_mode_idx.set(new_mode_idx);
        pending_mode_update.set(true);
    }
}

/// [`ui_cursor_shape_no_check_conceal`], plus the concealment recheck the
/// cursor line needs when the mode changed.
///
/// # Safety
///
/// May redraw the cursor line.
pub unsafe fn ui_cursor_shape() {
    unsafe {
        ui_cursor_shape_no_check_conceal();
        conceal_check_cursor_line();
    }
}

/// # Safety
///
/// Reads the current window and the compositor's layout.
unsafe fn ui_cursor_is_behind_floatwin() -> bool {
    if State.get() & MODE_CMDLINE != 0 || !ui_comp_should_draw() {
        return false;
    }
    let win = unsafe { &mut *curwin.get() };
    let crow = win.w_winrow + win.w_winrow_off + win.w_wrow;
    let wcol = if win.w_onebuf_opt.wo_rl != 0 {
        win.w_view_width - win.w_wcol - 1
    } else {
        win.w_wcol
    };
    let ccol = win.w_wincol + win.w_wincol_off + wcol;
    let top_grid = unsafe { ui_comp_get_grid_at_coord(crow, ccol) };
    top_grid != &raw mut win.w_grid_alloc && top_grid != default_grid.ptr()
}

/// Whether `ext` is drawn by the UIs rather than into the grid.
pub fn ui_has(ext: UIExtension) -> bool {
    ui_ext.with(|widgets| widgets[ext as usize])
}

/// Describes every attached UI, for `nvim_list_uis()`.
///
/// # Safety
///
/// `arena` must be live; the result borrows it.
pub unsafe fn ui_array(arena: *mut Arena) -> Array {
    /// Appends `key: value`. Within the capacity asked for below: ten fixed
    /// keys plus at most one per extension.
    ///
    /// # Safety
    ///
    /// `info` must have room for another entry.
    unsafe fn push(info: &mut Dict, key: String_0, value: Object) {
        unsafe { *info.items.add(info.size) = KeyValuePair { key, value } };
        info.size += 1;
    }

    let mut all_uis = arena_array(arena, ui_count());
    for ui in each_ui() {
        let ui = unsafe { &*ui };
        let mut info = arena_dict(arena, 10 + kUIExtCount as usize);
        let fixed: [(&'static str, Object); 9] = [
            ("width", Object::integer(ui.width as Integer)),
            ("height", Object::integer(ui.height as Integer)),
            ("rgb", Object::boolean(ui.rgb)),
            ("override", Object::boolean(ui.override_0)),
            (
                "term_name",
                Object::string(unsafe { cstr_as_string(ui.term_name) }),
            ),
            // Reported empty rather than read back: the background a UI
            // sent is only meaningful to whoever sent it.
            ("term_background", Object::literal("")),
            ("term_colors", Object::integer(ui.term_colors as Integer)),
            ("stdin_tty", Object::boolean(ui.stdin_tty)),
            ("stdout_tty", Object::boolean(ui.stdout_tty)),
        ];
        for (key, value) in fixed {
            unsafe { push(&mut info, static_string(key), value) };
        }
        for widget in 0..kUIExtCount as usize {
            let name = ext_name(widget);
            // A leading underscore marks an option outside the documented
            // protocol; those are only listed when turned on.
            let private = unsafe { *name } == b'_' as core::ffi::c_char;
            if !private || ui.ui_ext[widget] {
                unsafe {
                    push(
                        &mut info,
                        cstr_as_string(name.cast_mut()),
                        Object::boolean(ui.ui_ext[widget]),
                    )
                };
            }
        }
        unsafe {
            push(
                &mut info,
                static_string("chan"),
                Object::integer(ui.channel_id as Integer),
            )
        };

        unsafe { *all_uis.items.add(all_uis.size) = Object::dict(info) };
        all_uis.size += 1;
    }
    all_uis
}

/// The protocol name of the `widget`th UI extension.
fn ext_name(widget: usize) -> *const core::ffi::c_char {
    unsafe {
        *ui_ext_names
            .ptr()
            .cast::<*const core::ffi::c_char>()
            .add(widget)
    }
}

/// Resizes the grid a UI asked about, for `nvim_ui_try_resize_grid`.
///
/// # Safety
///
/// `err` must be writable.
pub unsafe fn ui_grid_resize(grid_handle: handle_T, width: c_int, height: c_int, err: *mut Error) {
    if grid_handle == DEFAULT_GRID_HANDLE {
        unsafe { screen_resize(width, height) };
        return;
    }
    let wp = unsafe { get_win_by_grid_handle(grid_handle) };
    if wp.is_null() {
        unsafe {
            api_err_invalid(
                err,
                c"window handle".as_ptr(),
                core::ptr::null(),
                grid_handle as i64,
                false,
            )
        };
        return;
    }
    let wp = unsafe { &mut *wp };
    if wp.w_floating {
        if width != wp.w_width || height != wp.w_height {
            wp.w_config.width = width.max(1);
            wp.w_config.height = height.max(1);
            unsafe { win_config_float(wp, wp.w_config.clone()) };
        }
    } else {
        // A split's size is a request: the layout decides what it gets.
        wp.w_height_request = height.max(0);
        wp.w_width_request = width.max(0);
        unsafe { win_set_inner_size(wp, true) };
    }
}
