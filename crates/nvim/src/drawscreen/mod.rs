#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::AutoEvent;
use core::ffi::{c_char, c_int};

use crate::autocmd::apply_autocmds;
use crate::buffer::{buf_meta_total, maketitle};
use crate::charset::{vim_isprintc, vim_strsize};
use crate::cmdexpand::cmdline_pum_display;
use crate::decoration::{
    SCL_NUM, buf_signcols_count_range, decor_conceal_line, decor_range_add_virt,
    decor_redraw_reset, decor_virt_lines, kMTMetaSignText, win_lines_concealed,
};
use crate::decoration_provider::{
    decor_providers_invoke_buf, decor_providers_invoke_end, decor_providers_invoke_win,
    decor_providers_start,
};
use crate::diff::diff_redraw;
use crate::digraph::keymap_str;
use crate::drawline::win_line;
use crate::eval::vars::set_vim_var_nr;
use crate::ex_getln::{cmdline_screen_cleared, compute_cmdrow, redrawcmdline};
use crate::fold::{fold_info, foldmethod_is_syntax, has_any_folding, has_folding};
use crate::getchar::char_avail;
use crate::global_cell::GlobalCell;
use crate::grid::{
    default_grid_ref, default_gridview, grid_adjust, grid_alloc, grid_clear, grid_del_lines,
    grid_draw_border, grid_ins_lines, grid_line_clear_end, grid_line_fill, grid_line_flush,
    grid_line_getchar, grid_line_mirror, grid_line_put_schar, grid_line_start,
    schar_cache_clear_if_full, schar_from_ascii, win_grid_alloc,
};
use crate::highlight::{
    hl_combine_attr, update_window_hl, win_bg_attr, win_check_ns_hl, win_hl_attr,
};
use crate::highlight_group::{
    HLF_AT, HLF_C, HLF_CM, HLF_COUNT, HLF_EOB, HLF_FC, HLF_MSG, HLF_N, HLF_SC, highlight_changed,
};
use crate::insexpand::ins_compl_show_pum;
use crate::main::{
    Columns, KeyTyped, RedrawingDisabled, Rows, State, clear_cmdline, cmdline_row,
    cmdline_was_last_drawn, curbuf, curtab, curwin, display_tick, do_redraw, dollar_vcol, dy_flags,
    edit_submode, edit_submode_extra, edit_submode_highl, edit_submode_pre, exiting, exmode_active,
    global_busy, got_int, hl_attr_active, lines_left, mode_displayed, msg_col, msg_did_scroll,
    msg_didany, msg_didout, msg_grid_scroll_discount, msg_no_more, msg_row, msg_scrolled,
    msg_scrolled_at_flush, msg_silent, must_redraw, must_redraw_pum, need_diff_redraw,
    need_highlight_changed, need_maketitle, need_wait_return, no_hlsearch, ns_hl_fast, p_ch,
    p_columns, p_hls, p_icon, p_lines, p_lz, p_paste, p_rdt, p_ri, p_ru, p_sc, p_sloc, p_smd,
    p_title, p_wbr, p_wmw, redraw_cmdline, redraw_mode, redraw_not_allowed, redraw_tabline,
    reg_recording, resizing_screen, restart_edit, ru_col, ru_wid, sc_col, screen_search_hl,
    search_hl_has_cursor_lnum, starting, stl_syntax, tab_page_click_defs, tab_page_click_defs_size,
    updating_screen, win_extmark_arr,
};
use crate::r#match::{init_search_hl, prepare_search_hl};
use crate::mbyte::{utf_ptr2cells, utf_ptr2char};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::message::{
    msg_check_for_delay, msg_clr_cmdline, msg_clr_eos, msg_ext_flush_showmode, msg_ext_ui_flush,
    msg_grid_ref, msg_grid_set_pos, msg_grid_validate, msg_puts_hl, msg_reset_scroll,
    msg_scrollsize, msg_use_grid, repeat_message,
};
use crate::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, curs_columns,
    invalidate_botline_win, plines_correct_topline, set_empty_rows, update_curswant,
    update_topline, validate_cursor, validate_virtcol, win_col_off, win_col_off2,
};
use crate::normal::{clear_showcmd, do_check_scrollbind};
use crate::option::{get_ve_flags, shortmess};
use crate::options::{kOptDyFlagLastline, kOptDyFlagTruncate, kOptVeFlagAll, kOptVeFlagBlock};
use crate::plines::{getvcols, getvvcol, plines_m_win, plines_win, win_get_fill, win_may_fill};
use crate::popupmenu::{pum_check_clear, pum_drawn, pum_invalidate, pum_redraw};
use crate::pos::MAXLNUM;
use crate::profile::profile_setlimit;
use crate::regexp::vim_regfree;
use crate::search::last_pat_prog;
use crate::spell::spell_check_window;
use crate::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_EXTERNCMD, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP,
    MODE_NORMAL, MODE_SETWSIZE, MODE_TERMINAL, MODE_VISUAL, REPLACE_FLAG, VREPLACE_FLAG,
    get_real_state,
};
use crate::statusline::{
    draw_tabline, redraw_ruler, stl_alloc_click_defs, stl_clear_click_defs, win_redr_status,
    win_redr_winbar,
};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::syntax::{
    syn_set_timeout, syn_stack_apply_changes, syntax_check_changed, syntax_end_parsing,
    syntax_present,
};
use crate::terminal::{terminal_check_size, terminal_suspended};
use crate::types::ui::{kUICmdline, kUIMessages, kUIMultigrid};
use crate::types::{
    DecorPriority, DecorVirtText, DecorVirtText_data, Failed, Integer, OptInt, VirtText,
    VirtTextChunk, Window, buf_T, colnr_T, foldinfo_T, frame_T, handle_T, hlf_T, int64_t, linenr_T,
    match_T, pos_T, proftime_T, regmmatch_T, regprog_T, schar_T, size_t, spellvars_T, uint16_t,
    varnumber_T, win_T,
};
use crate::ui::{
    ui_call_grid_clear, ui_call_grid_resize, ui_call_msg_clear, ui_call_win_extmark, ui_flush,
    ui_grid_cursor_goto, ui_has,
};
use crate::ui_compositor::ui_comp_set_screen_valid;
use crate::version::{intro_message, may_show_intro};
use crate::window::{
    frame2win, global_stl_height, last_stl_height, min_rows, min_rows_for_all_tabpages,
    win_fdccol_count, win_new_screensize, win_ui_flush,
};
use crate::winlayer::{self, Cc, Win};

// The carve of the transpiled module; see each child's docs.
mod resize;
pub use self::resize::*;
mod mode;
pub use self::mode::*;
mod separators;
pub use self::separators::*;
mod redraw;
pub use self::redraw::*;
mod winupdate;
pub(crate) use self::winupdate::*;
mod winlines;
pub use self::winlines::*;
use crate::regexp::re_multiline;
/// How much of a window has to be redrawn, ordered by severity. Each value
/// implies every lower one.
pub type RedrawType = ::core::ffi::c_int;
pub const UPD_CLEAR: RedrawType = 50;
pub const UPD_NOT_VALID: RedrawType = 40;
pub const UPD_SOME_VALID: RedrawType = 35;
pub const UPD_REDRAW_TOP: RedrawType = 30;
pub const UPD_INVERTED_ALL: RedrawType = 25;
pub const UPD_INVERTED: RedrawType = 20;
pub const UPD_VALID: RedrawType = 10;
/// Columns `'showcmd'` reserves at the right of the last line.
pub const SHOWCMD_COLS: ::core::ffi::c_int = 10;
/// The narrowest screen the editor will lay windows out on.
pub const MIN_COLUMNS: ::core::ffi::c_int = 12;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
/// The windows of the current tab page, in layout order.
///
/// `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`, i.e. [`winlayer::windows`] handing
/// back the raw pointer its callers still take.
///
/// Safe: handing out an address reads nothing, and [`winlayer::windows`] is
/// itself safe — it walks the registry, so the walk has the C's
/// `FOR_ALL_WINDOWS_IN_TAB` timing (the next link is read before the body).
/// What the *caller* does with the pointer is the unsafe part, and stays so.
pub(crate) fn windows_in_curtab() -> impl Iterator<Item = *mut win_T> {
    winlayer::windows().map(Win::raw)
}

/// The head of the current tab page's window list as the raw pointer the
/// transpiled redraw entry points still take, or a null before there is one.
fn first_win_raw() -> *mut win_T {
    winlayer::first_window().map_or(core::ptr::null_mut(), Win::raw)
}

static redraw_popupmenu: GlobalCell<bool> = GlobalCell::new(false);
static msg_grid_invalid: GlobalCell<bool> = GlobalCell::new(false);
static resizing_autocmd: GlobalCell<bool> = GlobalCell::new(false);
static conceal_cursor_used: GlobalCell<bool> = GlobalCell::new(false);
/// The screen row below window `wp`'s last one -- `W_ENDROW`.
///
/// That is where its horizontal separator or status line goes.
///
/// # Safety
/// `wp` must be a live window.
pub(crate) unsafe fn win_endrow(wp: *const win_T) -> c_int {
    // SAFETY: caller's promise.
    unsafe { (*wp).w_winrow + (*wp).w_height }
}

/// The screen column right of window `wp`'s last one -- `W_ENDCOL`.
///
/// That is where its vertical separator goes.
///
/// # Safety
/// `wp` must be a live window.
pub(crate) unsafe fn win_endcol(wp: *const win_T) -> c_int {
    // SAFETY: caller's promise.
    unsafe { (*wp).w_wincol + (*wp).w_width }
}

/// Redraw the cursor line if `'concealcursor'` changed what it does to it.
///
/// When the cursor also moved, both the old and the new line are redrawn
/// anyway, so this only matters when it did not.
pub unsafe fn conceal_check_cursor_line() {
    // SAFETY: `curwin` is the editor's current window, on the main thread.
    let wp = unsafe { Win::current() };
    let should_conceal = unsafe { conceal_cursor_line(wp.raw()) };
    if wp.w_onebuf_opt.wo_cole <= 0 || conceal_cursor_used.get() == should_conceal {
        return;
    }

    unsafe { redraw_win_line(wp.raw(), wp.w_cursor.lnum) };

    // Whether the line is displayed at all may have changed with it.
    if unsafe { decor_conceal_line(wp.raw(), wp.w_cursor.lnum - 1, true) } {
        changed_window_setting(unsafe { Win::new(wp.raw()) });
    }
    // The cursor column has to be recomputed, e.g. when entering Visual
    // mode stops the line being concealed.
    curs_columns(unsafe { Win::new(wp.raw()) }, c_int::from(true)); // may_scroll
}

/// Whether redrawing should happen right now.
///
/// `'lazyredraw'` postpones it while there is input waiting that was not typed
/// -- i.e. inside a mapping or a script -- unless something asked for a redraw
/// explicitly.
pub unsafe fn redrawing() -> bool {
    RedrawingDisabled.get() == 0
        && !(p_lz.get() != 0 && char_avail() && !KeyTyped.get() && !do_redraw.get())
}

/// Put the screen back together after messages scrolled it up.
///
/// The message area grew over the windows below it; this clears the rows it
/// used and marks everything that reached into them for a redraw. With
/// multigrid the message grid is a grid of its own, so there is nothing on the
/// default grid to repair -- which is what the `kUIMultigrid` test is for.
///
/// # Safety
/// Called from [`update_screen`] with the screen grids allocated.
unsafe fn restore_scrolled_messages(redr_type: c_int, is_stl_global: bool) {
    // SAFETY: the screen and message grids, on the main thread.
    clear_cmdline.set(true);

    // `msg_scrollsize` is a pure function of `msg_scrolled` and 'cmdheight',
    // neither of which changes until the end of this function.
    let scrollsize = msg_scrollsize();
    let valid = (Rows.get() - scrollsize).max(0);

    // The part of the message grid that is not displayed is invalid.
    let mut mg = msg_grid_ref();
    if mg.is_allocated() {
        for i in 0..scrollsize.min(mg.rows) {
            let (off, cols) = (mg.row_start(i), mg.cols);
            mg.clear_line(off, cols, (i as OptInt) < p_ch.get());
        }
    }
    mg.throttled = false;

    let mut was_invalidated = false;
    // UPD_CLEAR is already handled by the caller.
    if redr_type == UPD_NOT_VALID && !ui_has(kUIMultigrid) && msg_scrolled.get() != 0 {
        was_invalidated = ui_comp_set_screen_valid(false);
        let mut dg = default_grid_ref();
        let mut row = valid;
        while (row as OptInt) < Rows.get() as OptInt - p_ch.get() {
            let off = dg.row_start(row);
            dg.clear_line(off, Columns.get(), false);
            row += 1;
        }
        for mut wp in winlayer::windows() {
            if wp.w_floating {
                continue;
            }
            if unsafe { win_endrow(wp.raw()) } > valid {
                // Pessimistic: `redr_type` could be UPD_NOT_VALID only
                // because of windows above the separator.
                wp.w_redr_type = wp.w_redr_type.max(UPD_NOT_VALID);
            }
            if !is_stl_global && unsafe { win_endrow(wp.raw()) } + wp.w_status_height > valid {
                wp.w_redr_status = true;
            }
        }
        if is_stl_global && Rows.get() as OptInt - p_ch.get() - 1 > valid as OptInt {
            unsafe { (*curwin.get()).w_redr_status = true };
        }
    }

    unsafe { msg_grid_set_pos(Rows.get() - p_ch.get() as c_int, false) };
    msg_grid_invalid.set(false);
    if was_invalidated {
        // Only the message area was invalid, not the floats.
        ui_comp_set_screen_valid(true);
    }

    msg_scrolled.set(0);
    msg_scrolled_at_flush.set(0);
    msg_grid_scroll_discount.set(0);
    need_wait_return.set(false);
}

/// Bring every displayed buffer's cached syntax and decoration state up to date
/// with the changes made since the last redraw.
///
/// Each buffer is done once however many windows show it, which is what the two
/// `display_tick` stamps are for.
///
/// # Safety
/// Called from [`update_screen`].
unsafe fn update_buffer_state(redr_type: c_int, hl_changed: bool) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for wp in winlayer::windows() {
        unsafe { update_window_hl(wp.raw(), redr_type >= UPD_NOT_VALID || hl_changed) };

        let buf = wp.w_buffer;
        if !unsafe { (*buf).b_mod_set } {
            continue;
        }
        if unsafe { (*buf).b_mod_tick_syn } < display_tick.get()
            && unsafe { syntax_present(wp.raw()) }
        {
            unsafe { syn_stack_apply_changes(buf) };
            unsafe { (*buf).b_mod_tick_syn = display_tick.get() };
        }
        if unsafe { (*buf).b_mod_tick_decor } < display_tick.get() {
            unsafe { decor_providers_invoke_buf(buf) };
            unsafe { (*buf).b_mod_tick_decor = display_tick.get() };
        }
    }
}

/// Redraw the parts of the screen that are marked for redraw.
///
/// Most code should not call this directly: [`redraw_later`] and
/// [`redraw_all_later`] mark what changed and the main loop gets here.
///
/// Answers `Err` when nothing was drawn -- the screen is not ready, redrawing
/// is disabled, or this is a recursive call.
pub unsafe fn update_screen() -> Result<(), Failed> {
    // The intro message is shown until something else claims the screen.
    static STILL_MAY_INTRO: GlobalCell<bool> = GlobalCell::new(true);

    // SAFETY: the whole screen pipeline, on the main thread.
    if STILL_MAY_INTRO.get() && !unsafe { may_show_intro() } {
        unsafe { redraw_later(first_win_raw(), UPD_NOT_VALID) };
        STILL_MAY_INTRO.set(false);
    }

    let is_stl_global = global_stl_height() > 0;

    // A VimResized autocommand can redraw in the middle of a resize, which
    // would bypass the checks in `screen_resize`.
    if resizing_autocmd.get() || !default_grid_ref().is_allocated() {
        return Err(Failed);
    }

    // May have postponed updating diffs.
    if need_diff_redraw.get() {
        unsafe { diff_redraw(true) };
    }

    if !unsafe { redrawing() } || updating_screen.get() || cmdline_number_prompt() {
        return Err(Failed);
    }

    let mut redr_type = must_redraw.get();
    // Reset now, so that a redraw asked for while redrawing -- by
    // asynchronous scrolling, by `update_topline` in `win_update`, or by a
    // decoration provider -- happens later rather than being lost.
    must_redraw.set(0);

    updating_screen.set(true);
    display_tick.set(display_tick.get().wrapping_add(1));

    // Glyph cache full, very rare. The screen buffers cannot be compared
    // against their previous contents after this, so it has to be a CLEAR.
    if unsafe { schar_cache_clear_if_full() } {
        redr_type = redr_type.max(UPD_CLEAR);
    }

    // Tricky: other code can reset `msg_scrolled` behind our back, so this
    // is bookkept separately.
    if msg_did_scroll.get() {
        msg_did_scroll.set(false);
        msg_scrolled_at_flush.set(0);
    }

    if redr_type >= UPD_CLEAR || !default_grid_ref().valid {
        ui_comp_set_screen_valid(false);
    }

    if msg_scrolled.get() != 0 || msg_grid_invalid.get() {
        unsafe { restore_scrolled_messages(redr_type, is_stl_global) };
    }

    unsafe { win_ui_flush(true) };

    // `cmdline_row` may have been moved temporarily.
    unsafe { compute_cmdrow() };

    let mut hl_changed = false;
    if need_highlight_changed.get() {
        unsafe { highlight_changed() };
        hl_changed = true;
    }

    if redr_type == UPD_CLEAR {
        // Resets `clear_cmdline` and sets UPD_NOT_VALID on every window.
        unsafe { screenclear() };
        unsafe { cmdline_screen_cleared() };
        if ui_has(kUIMessages) {
            ui_call_msg_clear();
        }
        redr_type = UPD_NOT_VALID;
        // `must_redraw` may have been set indirectly; avoid another redraw.
        must_redraw.set(0);
    } else if !default_grid_ref().valid {
        default_grid_ref().revalidate();
    }

    // May need to clear space on the default grid for the message area.
    if redr_type == UPD_NOT_VALID && clear_cmdline.get() && !ui_has(kUIMessages) {
        unsafe {
            grid_clear(
                default_gridview(),
                Rows.get() - p_ch.get() as c_int,
                Rows.get(),
                0,
                Columns.get(),
                0,
            )
        };
    }

    ui_comp_set_screen_valid(true);

    unsafe { decor_providers_start() };

    // The "start" callback may have changed highlights used by the global
    // elements.
    if unsafe { win_check_ns_hl(::core::ptr::null_mut()) } {
        redraw_cmdline.set(true);
        redraw_tabline.set(true);
    }

    if clear_cmdline.get() {
        unsafe { msg_check_for_delay(false) };
    }

    // Force a redraw when the width of the number column changed.
    //
    // Upstream special-cases `curwin` here and says so in a comment; either
    // every window should be checked or none should. Reproduced.
    let mut wp = unsafe { Win::current() };
    // `number_width` is NOT pure -- it caches its answer in the window and
    // resets the 'statuscolumn' width estimate -- so it stays behind the
    // `w_redr_type` test, where upstream's `&&` puts it.
    if wp.w_redr_type < UPD_NOT_VALID {
        let nrwidth = if wp.w_onebuf_opt.wo_nu != 0
            || wp.w_onebuf_opt.wo_rnu != 0
            || unsafe { *wp.w_onebuf_opt.wo_stc } != 0
        {
            unsafe { number_width(wp.raw()) }
        } else {
            0
        };
        if wp.w_nrwidth != nrwidth {
            wp.w_redr_type = UPD_NOT_VALID;
        }
    }

    if wp.w_redr_type == UPD_INVERTED {
        // So the end of the Visual selection is right.
        unsafe { update_curswant() };
    }

    if redraw_tabline.get() || redr_type >= UPD_NOT_VALID {
        unsafe { update_window_hl(curwin.get(), redr_type >= UPD_NOT_VALID) };
        for tp in winlayer::tabs() {
            if !tp.is_current() {
                unsafe { update_window_hl(tp.tp_curwin, redr_type >= UPD_NOT_VALID) };
            }
        }
        unsafe { draw_tabline() };
    }

    unsafe { update_buffer_state(redr_type, hl_changed) };

    // Top to bottom through the windows, redrawing the ones that need it.
    let mut did_one = false;
    SearchHl::current().set_regprog(::core::ptr::null_mut());

    for mut wp in winlayer::windows() {
        if wp.w_redr_type == UPD_CLEAR && wp.w_floating && wp.w_grid_alloc.is_allocated() {
            wp.w_grid_alloc.invalidate();
            wp.w_redr_type = UPD_NOT_VALID;
        }

        unsafe { win_check_ns_hl(wp.raw()) };
        unsafe { win_grid_alloc(wp.raw()) };

        if wp.w_redr_border || wp.w_redr_type >= UPD_NOT_VALID {
            unsafe {
                grid_draw_border(
                    &raw mut wp.w_grid_alloc,
                    &raw mut wp.w_config,
                    (&raw mut wp.w_border_adj).cast::<c_int>(),
                    wp.w_onebuf_opt.wo_winbl as c_int,
                    wp.w_ns_hl_attr,
                )
            };
        }

        if wp.w_redr_type != 0 {
            if !did_one {
                did_one = true;
                unsafe { start_search_hl() };
            }
            unsafe { win_update(wp) };
        }

        // The status line and window bar go after the window, to minimise
        // cursor movement.
        if wp.w_redr_status {
            unsafe { win_redr_winbar(wp.raw()) };
            unsafe { win_redr_status(wp.raw()) };
        }
    }

    // Separator connectors go after every window update, so that a
    // connector is never overwritten by a neighbour's separator.
    if did_one {
        for wp in winlayer::windows() {
            unsafe { draw_sep_connectors_win(wp) };
        }
    }

    end_search_hl();

    if pum_drawn() && must_redraw_pum.get() {
        unsafe { win_check_ns_hl(curwin.get()) };
        unsafe { pum_redraw() };
    } else if State.get() & MODE_CMDLINE != 0 {
        unsafe { pum_check_clear() };
    }

    unsafe { win_check_ns_hl(::core::ptr::null_mut()) };

    // Reset `b_mod_set`. Going through the windows is probably faster than
    // going through every buffer.
    for mut wp in winlayer::windows() {
        unsafe { (*wp.w_buffer).b_mod_set = false };
    }

    updating_screen.set(false);

    if need_maketitle.get() {
        unsafe { maketitle() };
    }

    // Last, because scrolling may mess the command line up.
    if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
        unsafe { showmode() };
    }

    if STILL_MAY_INTRO.get() {
        unsafe { intro_message(false) };
    }
    unsafe { repeat_message() };

    unsafe { decor_providers_invoke_end() };

    // Either the cmdline was cleared, not drawn, or the mode was drawn last.
    // This does not necessarily overwrite an external cmdline.
    if !ui_has(kUICmdline) {
        cmdline_was_last_drawn.set(false);
    }
    Ok(())
}

/// The search-highlight matcher the whole redraw shares.
///
/// A `Copy` handle that *names* the cell rather than borrowing it: it is
/// handed to `drawline/`, which runs decoration providers that re-enter the
/// draw pass, so no borrow could span the walk.
#[derive(Clone, Copy)]
pub(crate) struct SearchHl(*mut match_T);

impl SearchHl {
    /// The one place the redraw matcher's address is taken.
    pub(crate) fn current() -> Self {
        SearchHl(screen_search_hl.ptr())
    }

    /// The address, for the `*_search_hl` helpers that take one.
    pub(crate) fn raw(self) -> *mut match_T {
        self.0
    }

    /// The multi-line regmatch, for `last_pat_prog` to compile into.
    pub(crate) fn regmatch(self) -> *mut regmmatch_T {
        // SAFETY: the only constructor names a `static`; no dereference here.
        unsafe { &raw mut (*self.0).rm }
    }

    /// The compiled `'hlsearch'` pattern, NULL when there is none.
    pub(crate) fn regprog(self) -> *mut regprog_T {
        // SAFETY: the only constructor names a `static`.
        unsafe { (*self.0).rm.regprog }
    }

    /// Install (or, with NULL, forget) the compiled pattern.
    pub(crate) fn set_regprog(self, prog: *mut regprog_T) {
        // SAFETY: as `regprog`.
        unsafe { (*self.0).rm.regprog = prog };
    }

    /// Bound the matching by `'redrawtime'`.
    fn set_time_limit(self, tm: proftime_T) {
        // SAFETY: as `regprog`.
        unsafe { (*self.0).tm = tm };
    }
}

/// Compile the `'hlsearch'` pattern for the redraw that is starting.
pub unsafe fn start_search_hl() {
    if p_hls.get() == 0 || no_hlsearch.get() {
        return;
    }
    end_search_hl(); // just in case it was not called before
    let hl = SearchHl::current();
    // SAFETY: the search history's own last pattern, compiled into the
    // redraw's matcher.
    unsafe { last_pat_prog(hl.regmatch()) };
    // Bound the search by 'redrawtime'.
    hl.set_time_limit(profile_setlimit(p_rdt.get() as int64_t));
}

/// Free the compiled `'hlsearch'` pattern.
pub fn end_search_hl() {
    let hl = SearchHl::current();
    let prog = hl.regprog();
    if prog.is_null() {
        return;
    }
    // SAFETY: the pattern `start_search_hl` compiled, freed once.
    unsafe { vim_regfree(prog) };
    hl.set_regprog(::core::ptr::null_mut());
}

/// Put the terminal cursor where the cursor is in the current window.
pub unsafe fn setcursor() {
    // SAFETY: `curwin` is the editor's current window.
    unsafe { setcursor_mayforce(curwin.get(), false) }
}

/// Put the terminal cursor where the cursor is in window `wp`.
///
/// `force` positions it even when not redrawing.
pub unsafe fn setcursor_mayforce(wp: *mut win_T, force: bool) {
    // SAFETY: a live window; `grid_adjust` maps its coordinates onto whichever
    // grid actually carries them.
    if !force && !unsafe { redrawing() } {
        return;
    }
    let wp = unsafe { Win::new(wp) };
    validate_cursor(wp);

    let mut row = wp.w_wrow;
    let mut col = wp.w_wcol;
    if wp.w_onebuf_opt.wo_rl != 0 {
        // With 'rightleft' and the cursor on a double-width character, the
        // cursor goes on its leftmost column.
        let cursor =
            unsafe { ml_get_buf(wp.w_buffer, wp.w_cursor.lnum).add(wp.w_cursor.col as usize) };
        let cells = if unsafe { utf_ptr2cells(cursor) } == 2
            && unsafe { vim_isprintc(utf_ptr2char(cursor)) }
        {
            2
        } else {
            1
        };
        col = wp.w_view_width - wp.w_wcol - cells;
    }

    let grid = unsafe { grid_adjust(wp.w_grid, &mut row, &mut col) };
    if !grid.is_unresolved() {
        ui_grid_cursor_goto(grid.handle, row, col);
    }
}

/// The width of window `wp`'s fold column, given `col` columns are already
/// spoken for.
///
/// `'foldcolumn'` asks for a width; what it gets is bounded by the room left
/// beside the text, which must be at least one column ('winminwidth' of 0 still
/// leaves one for the current window).
pub unsafe fn compute_foldcolumn(wp: *mut win_T, col: c_int) -> c_int {
    // SAFETY: a live window, on the main thread.
    let wp = unsafe { Win::new(wp) };
    let fdc = unsafe { win_fdccol_count(wp.raw()) };
    let min_width = if wp.raw() == curwin.get() && p_wmw.get() == 0 {
        1
    } else {
        p_wmw.get() as c_int
    };
    fdc.min(wp.w_view_width - (col + min_width))
}

/// The width of window `wp`'s `'number'`/`'relativenumber'` column.
///
/// Callers check whether either option is set; this only decides how wide the
/// column would be. The answer is cached against the line count it was computed
/// for, since it only changes when that crosses a power of ten.
pub unsafe fn number_width(wp: *mut win_T) -> c_int {
    // SAFETY: a live window and its buffer, on the main thread.
    let mut wp = unsafe { Win::new(wp) };
    // With 'relativenumber' alone the largest number shown is the window
    // height (the cursor line shows "0"); otherwise it is the line count.
    let largest = if wp.w_onebuf_opt.wo_rnu != 0 && wp.w_onebuf_opt.wo_nu == 0 {
        wp.w_view_height as linenr_T
    } else {
        unsafe { (*wp.w_buffer).b_ml.ml_line_count }
    };

    if largest == wp.w_nrwidth_line_count {
        return wp.w_nrwidth_width;
    }
    wp.w_nrwidth_line_count = largest;

    if unsafe { *wp.w_onebuf_opt.wo_stc } != 0 {
        // 'statuscolumn' draws the number itself, so all that is reserved
        // here is 'numberwidth'; the real width is re-estimated from the
        // expression's output.
        wp.w_statuscol_line_count = 0;
        wp.w_nrwidth_width = c_int::from(wp.w_onebuf_opt.wo_nu != 0 || wp.w_onebuf_opt.wo_rnu != 0)
            * wp.w_onebuf_opt.wo_nuw as c_int;
        return wp.w_nrwidth_width;
    }

    // Digits in `largest`, at least one -- upstream's do-while, which
    // answers 1 for a line count of 0.
    let mut n = 0;
    let mut rest = largest;
    loop {
        rest /= 10;
        n += 1;
        if rest <= 0 {
            break;
        }
    }

    // 'numberwidth' is the minimal width plus one.
    n = n.max(wp.w_onebuf_opt.wo_nuw as c_int - 1);

    // With `'signcolumn'` "number" and a sign to show, the number column
    // needs room for the two-cell sign text.
    if n < 2
        && buf_meta_total(unsafe { Win::new(wp.raw()) }.buffer(), kMTMetaSignText) != 0
        && wp.w_minscwidth == SCL_NUM
    {
        n = 2;
    }

    wp.w_nrwidth_width = n;
    n
}

/// Whether the cursor line in window `wp` may be concealed, per
/// `'concealcursor'`.
pub unsafe fn conceal_cursor_line(wp: *const win_T) -> bool {
    // SAFETY: a live window, on the main thread.
    if unsafe { *(*wp).w_onebuf_opt.wo_cocu } == 0 {
        return false;
    }
    let mode = if get_real_state() & MODE_VISUAL != 0 {
        b'v'
    } else if State.get() & MODE_INSERT != 0 {
        b'i'
    } else if State.get() & MODE_NORMAL != 0 {
        b'n'
    } else if State.get() & MODE_CMDLINE != 0 {
        b'c'
    } else {
        return false;
    };
    !unsafe { vim_strchr((*wp).w_onebuf_opt.wo_cocu, mode as c_int) }.is_null()
}

/// Whether the cursor line of window `wp` is drawn differently from any other.
///
/// When it is, moving the cursor within the window means redrawing both the old
/// cursor line and the new one.
pub unsafe fn win_cursorline_standout(wp: *const win_T) -> bool {
    // SAFETY: a live window, on the main thread.
    unsafe {
        (*wp).w_onebuf_opt.wo_cul != 0
            || (wp == curwin.get() && (*wp).w_onebuf_opt.wo_cole > 0 && !conceal_cursor_line(wp))
    }
}

/// Update `w_cursorline`, and answer the cursor line's fold info through
/// `foldinfo`.
///
/// On a closed fold the whole fold is the cursor line, so `w_cursorline` is
/// moved to its first line -- otherwise the fold would not be redrawn when the
/// cursor moves onto it.
pub unsafe fn win_update_cursorline(wp: *mut win_T, foldinfo: *mut foldinfo_T) {
    // SAFETY: a live window; `foldinfo` is the caller's out-parameter.
    let mut wp = unsafe { Win::new(wp) };
    unsafe {
        wp.w_cursorline = if win_cursorline_standout(wp.raw()) {
            wp.w_cursor.lnum
        } else {
            0
        }
    };
    if wp.w_onebuf_opt.wo_cul != 0 {
        unsafe { *foldinfo = fold_info(Win::new(wp.raw()), wp.w_cursor.lnum) };
        if unsafe { (*foldinfo).fi_level } != 0 && unsafe { (*foldinfo).fi_lines } > 0 {
            unsafe { wp.w_cursorline = (*foldinfo).fi_lnum };
        }
    }
}

pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
