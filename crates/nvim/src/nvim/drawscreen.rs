#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::autocmd::{EVENT_VIMRESIZED, apply_autocmds};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::buffer::maketitle;
use crate::src::nvim::charset::{vim_isprintc, vim_strsize};
use crate::src::nvim::cmdexpand::cmdline_pum_display;
use crate::src::nvim::decoration::{
    SCL_NUM, buf_signcols_count_range, decor_conceal_line, decor_range_add_virt,
    decor_redraw_reset, decor_virt_lines, kMTMetaSignText, win_lines_concealed,
};
use crate::src::nvim::decoration_provider::{
    decor_providers_invoke_buf, decor_providers_invoke_end, decor_providers_invoke_win,
    decor_providers_start,
};
use crate::src::nvim::diff::diff_redraw;
use crate::src::nvim::digraph::keymap_str;
use crate::src::nvim::drawline::win_line;
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::ex_getln::{
    cmdline_screen_cleared, compute_cmdrow, get_cmdline_info, redrawcmdline,
};
use crate::src::nvim::fold::{fold_info, foldmethodIsSyntax, hasAnyFolding, hasFolding};
use crate::src::nvim::getchar::char_avail;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_adjust, grid_alloc, grid_clear, grid_clear_line, grid_del_lines, grid_draw_border,
    grid_ins_lines, grid_invalidate, grid_line_clear_end, grid_line_fill, grid_line_flush,
    grid_line_getchar, grid_line_mirror, grid_line_put_schar, grid_line_start,
    schar_cache_clear_if_full, schar_from_ascii, win_grid_alloc,
};
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight::{
    hl_combine_attr, update_window_hl, win_bg_attr, win_check_ns_hl,
};
use crate::src::nvim::highlight_group::{
    HLF_AT, HLF_C, HLF_CM, HLF_COUNT, HLF_EOB, HLF_FC, HLF_MSG, HLF_N, HLF_SC, highlight_changed,
};
use crate::src::nvim::insexpand::ins_compl_show_pum;
use crate::src::nvim::main::{
    Columns, KeyTyped, NameBuff, RedrawingDisabled, Rows, State, VIsual, VIsual_active,
    VIsual_mode, VIsual_select, clear_cmdline, cmdline_row, cmdline_was_last_drawn, curbuf, curtab,
    curwin, decor_state, default_grid, default_gridview, display_tick, do_redraw, dollar_vcol,
    dy_flags, edit_submode, edit_submode_extra, edit_submode_highl, edit_submode_pre, exiting,
    exmode_active, first_tabpage, firstwin, global_busy, got_int, hl_attr_active, lines_left,
    mode_displayed, msg_col, msg_did_scroll, msg_didany, msg_didout, msg_grid,
    msg_grid_scroll_discount, msg_no_more, msg_row, msg_scrolled, msg_scrolled_at_flush,
    msg_silent, must_redraw, must_redraw_pum, need_diff_redraw, need_highlight_changed,
    need_maketitle, need_wait_return, no_hlsearch, ns_hl_fast, p_ch, p_columns, p_cpo, p_hls,
    p_icon, p_lines, p_lz, p_paste, p_rdt, p_ri, p_ru, p_sc, p_sloc, p_smd, p_title, p_wbr, p_wmw,
    redraw_cmdline, redraw_mode, redraw_not_allowed, redraw_tabline, reg_recording,
    resizing_screen, restart_edit, ru_col, ru_wid, sc_col, screen_search_hl,
    search_hl_has_cursor_lnum, starting, stl_syntax, tab_page_click_defs, tab_page_click_defs_size,
    updating_screen, win_extmark_arr,
};
use crate::src::nvim::r#match::{init_search_hl, prepare_search_hl};
use crate::src::nvim::mbyte::{utf_ptr2cells, utf_ptr2char};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::message::{
    msg_check_for_delay, msg_clr_cmdline, msg_clr_eos, msg_ext_flush_showmode, msg_ext_ui_flush,
    msg_grid_set_pos, msg_grid_validate, msg_puts_hl, msg_reset_scroll, msg_scrollsize,
    msg_use_grid, repeat_message,
};
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, curs_columns,
    invalidate_botline_win, plines_correct_topline, set_empty_rows, update_curswant,
    update_topline, validate_cursor, validate_virtcol, win_col_off, win_col_off2,
};
use crate::src::nvim::normal::{clear_showcmd, do_check_scrollbind};
use crate::src::nvim::option::{get_ve_flags, shortmess};
use crate::src::nvim::options::{
    kOptDyFlagLastline, kOptDyFlagTruncate, kOptVeFlagAll, kOptVeFlagBlock,
};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::plines::{
    getvcols, getvvcol, plines_m_win, plines_win, win_get_fill, win_may_fill,
};
use crate::src::nvim::popupmenu::{pum_check_clear, pum_drawn, pum_invalidate, pum_redraw};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::profile::profile_setlimit;
use crate::src::nvim::search::last_pat_prog;
use crate::src::nvim::spell::spell_check_window;
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_EXTERNCMD, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP,
    MODE_NORMAL, MODE_SETWSIZE, MODE_TERMINAL, MODE_VISUAL, REPLACE_FLAG, VREPLACE_FLAG,
    get_real_state,
};
use crate::src::nvim::statusline::{
    draw_tabline, redraw_ruler, stl_alloc_click_defs, stl_clear_click_defs, win_redr_status,
    win_redr_winbar,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::syntax::{
    syn_set_timeout, syn_stack_apply_changes, syntax_check_changed, syntax_end_parsing,
    syntax_present,
};
use crate::src::nvim::terminal::{terminal_check_size, terminal_suspended};
use crate::src::nvim::types::ui::{kUICmdline, kUIMessages, kUIMultigrid};
use crate::src::nvim::types::{
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Integer, OptInt,
    TriState, VimVarIndex, VirtText, VirtTextChunk, Window, buf_T, colnr_T, foldinfo_T, frame_T,
    handle_T, hlf_T, int64_t, linenr_T, pos_T, regprog_T, schar_T, size_t, spellvars_T, uint16_t,
    varnumber_T, win_T,
};
use crate::src::nvim::ui::{
    ui_call_grid_clear, ui_call_grid_resize, ui_call_msg_clear, ui_call_win_extmark, ui_flush,
    ui_grid_cursor_goto, ui_has,
};
use crate::src::nvim::ui_compositor::ui_comp_set_screen_valid;
use crate::src::nvim::version::{intro_message, may_show_intro};
use crate::src::nvim::window::{
    frame2win, global_stl_height, last_stl_height, min_rows, min_rows_for_all_tabpages,
    win_fdccol_count, win_new_screensize, win_ui_flush,
};

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
unsafe extern "C" {
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regfree(prog: *mut regprog_T);
}
pub const kFalse: TriState = 0;
/// A column past any real one -- "to the end of the line".
///
/// Still declared per module tree-wide (40 copies); `pos.rs` is the home it
/// wants, and that is the standing tree-wide constant job.
pub const MAXCOL: ::core::ffi::c_int = ::core::ffi::c_int::MAX;
/// `'shortmess'` flags this module tests.
pub const SHM_RECORDING: ::core::ffi::c_int = b'q' as ::core::ffi::c_int;
pub const SHM_COMPLETIONMENU: ::core::ffi::c_int = b'c' as ::core::ffi::c_int;
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
/// `v:echospace` -- how many columns a message may use before it wraps.
pub const VV_ECHOSPACE: VimVarIndex = 87;
/// Columns `'showcmd'` reserves at the right of the last line.
pub const SHOWCMD_COLS: ::core::ffi::c_int = 10;
/// The narrowest screen the editor will lay windows out on.
pub const MIN_COLUMNS: ::core::ffi::c_int = 12;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
/// The windows of the current tab page, in layout order.
///
/// `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`. The current tab page keeps its window
/// list in the global `firstwin` rather than in its own struct, which is why
/// c2rust rendered the macro as `if curtab == curtab { firstwin } else { … }` --
/// i.e. as `firstwin` and nothing else.
///
/// # Safety
/// The window list must not be restructured while the iterator is live.
pub(crate) unsafe fn windows_in_curtab() -> impl Iterator<Item = *mut win_T> {
    let mut wp = firstwin.get();
    ::core::iter::from_fn(move || {
        if wp.is_null() {
            return None;
        }
        let cur = wp;
        // SAFETY: the caller promises the list is not restructured under us.
        wp = unsafe { (*cur).w_next };
        Some(cur)
    })
}

static redraw_popupmenu: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_grid_invalid: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static resizing_autocmd: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static conceal_cursor_used: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
    unsafe {
        let wp = curwin.get();
        let should_conceal = conceal_cursor_line(wp);
        if (*wp).w_onebuf_opt.wo_cole <= 0 || conceal_cursor_used.get() == should_conceal {
            return;
        }

        redrawWinline(wp, (*wp).w_cursor.lnum);

        // Whether the line is displayed at all may have changed with it.
        if decor_conceal_line(wp, (*wp).w_cursor.lnum - 1, true) {
            changed_window_setting(wp);
        }
        // The cursor column has to be recomputed, e.g. when entering Visual
        // mode stops the line being concealed.
        curs_columns(wp, c_int::from(true)); // may_scroll
    }
}

/// Whether redrawing should happen right now.
///
/// `'lazyredraw'` postpones it while there is input waiting that was not typed
/// -- i.e. inside a mapping or a script -- unless something asked for a redraw
/// explicitly.
pub unsafe fn redrawing() -> bool {
    // SAFETY: `char_avail` pumps the input layer on the main thread.
    unsafe {
        RedrawingDisabled.get() == 0
            && !(p_lz.get() != 0 && char_avail() && !KeyTyped.get() && !do_redraw.get())
    }
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
    unsafe {
        clear_cmdline.set(true);

        // `msg_scrollsize` is a pure function of `msg_scrolled` and 'cmdheight',
        // neither of which changes until the end of this function.
        let scrollsize = msg_scrollsize();
        let valid = (Rows.get() - scrollsize).max(0);

        // The part of the message grid that is not displayed is invalid.
        let mg = msg_grid.ptr();
        if !(*mg).chars.is_null() {
            for i in 0..scrollsize.min((*mg).rows) {
                grid_clear_line(
                    mg,
                    *(*mg).line_offset.add(i as usize),
                    (*mg).cols,
                    (i as OptInt) < p_ch.get(),
                );
            }
        }
        (*mg).throttled = false;

        let mut was_invalidated = false;
        // UPD_CLEAR is already handled by the caller.
        if redr_type == UPD_NOT_VALID && !ui_has(kUIMultigrid) && msg_scrolled.get() != 0 {
            was_invalidated = ui_comp_set_screen_valid(false);
            let dg = default_grid.ptr();
            let mut row = valid;
            while (row as OptInt) < Rows.get() as OptInt - p_ch.get() {
                grid_clear_line(
                    dg,
                    *(*dg).line_offset.add(row as usize),
                    Columns.get(),
                    false,
                );
                row += 1;
            }
            for wp in windows_in_curtab() {
                if (*wp).w_floating {
                    continue;
                }
                if win_endrow(wp) > valid {
                    // Pessimistic: `redr_type` could be UPD_NOT_VALID only
                    // because of windows above the separator.
                    (*wp).w_redr_type = (*wp).w_redr_type.max(UPD_NOT_VALID);
                }
                if !is_stl_global && win_endrow(wp) + (*wp).w_status_height > valid {
                    (*wp).w_redr_status = true;
                }
            }
            if is_stl_global && Rows.get() as OptInt - p_ch.get() - 1 > valid as OptInt {
                (*curwin.get()).w_redr_status = true;
            }
        }

        msg_grid_set_pos(Rows.get() - p_ch.get() as c_int, false);
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
    unsafe {
        for wp in windows_in_curtab() {
            update_window_hl(wp, redr_type >= UPD_NOT_VALID || hl_changed);

            let buf = (*wp).w_buffer;
            if !(*buf).b_mod_set {
                continue;
            }
            if (*buf).b_mod_tick_syn < display_tick.get() && syntax_present(wp) {
                syn_stack_apply_changes(buf);
                (*buf).b_mod_tick_syn = display_tick.get();
            }
            if (*buf).b_mod_tick_decor < display_tick.get() {
                decor_providers_invoke_buf(buf);
                (*buf).b_mod_tick_decor = display_tick.get();
            }
        }
    }
}

/// Redraw the parts of the screen that are marked for redraw.
///
/// Most code should not call this directly: [`redraw_later`] and
/// [`redraw_all_later`] mark what changed and the main loop gets here.
///
/// Answers `FAIL` when nothing was drawn -- the screen is not ready, redrawing
/// is disabled, or this is a recursive call.
pub unsafe fn update_screen() -> c_int {
    // The intro message is shown until something else claims the screen.
    static STILL_MAY_INTRO: GlobalCell<bool> = GlobalCell::new(true);

    // SAFETY: the whole screen pipeline, on the main thread.
    unsafe {
        if STILL_MAY_INTRO.get() && !may_show_intro() {
            redraw_later(firstwin.get(), UPD_NOT_VALID);
            STILL_MAY_INTRO.set(false);
        }

        let is_stl_global = global_stl_height() > 0;

        // A VimResized autocommand can redraw in the middle of a resize, which
        // would bypass the checks in `screen_resize`.
        if resizing_autocmd.get() || (*default_grid.ptr()).chars.is_null() {
            return FAIL;
        }

        // May have postponed updating diffs.
        if need_diff_redraw.get() {
            diff_redraw(true);
        }

        if !redrawing() || updating_screen.get() || cmdline_number_prompt() {
            return FAIL;
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
        if schar_cache_clear_if_full() {
            redr_type = redr_type.max(UPD_CLEAR);
        }

        // Tricky: other code can reset `msg_scrolled` behind our back, so this
        // is bookkept separately.
        if msg_did_scroll.get() {
            msg_did_scroll.set(false);
            msg_scrolled_at_flush.set(0);
        }

        if redr_type >= UPD_CLEAR || !(*default_grid.ptr()).valid {
            ui_comp_set_screen_valid(false);
        }

        if msg_scrolled.get() != 0 || msg_grid_invalid.get() {
            restore_scrolled_messages(redr_type, is_stl_global);
        }

        win_ui_flush(true);

        // `cmdline_row` may have been moved temporarily.
        compute_cmdrow();

        let mut hl_changed = false;
        if need_highlight_changed.get() {
            highlight_changed();
            hl_changed = true;
        }

        if redr_type == UPD_CLEAR {
            // Resets `clear_cmdline` and sets UPD_NOT_VALID on every window.
            screenclear();
            cmdline_screen_cleared();
            if ui_has(kUIMessages) {
                ui_call_msg_clear();
            }
            redr_type = UPD_NOT_VALID;
            // `must_redraw` may have been set indirectly; avoid another redraw.
            must_redraw.set(0);
        } else if !(*default_grid.ptr()).valid {
            grid_invalidate(default_grid.ptr());
            (*default_grid.ptr()).valid = true;
        }

        // May need to clear space on the default grid for the message area.
        if redr_type == UPD_NOT_VALID && clear_cmdline.get() && !ui_has(kUIMessages) {
            grid_clear(
                default_gridview.ptr(),
                Rows.get() - p_ch.get() as c_int,
                Rows.get(),
                0,
                Columns.get(),
                0,
            );
        }

        ui_comp_set_screen_valid(true);

        decor_providers_start();

        // The "start" callback may have changed highlights used by the global
        // elements.
        if win_check_ns_hl(::core::ptr::null_mut()) {
            redraw_cmdline.set(true);
            redraw_tabline.set(true);
        }

        if clear_cmdline.get() {
            msg_check_for_delay(false);
        }

        // Force a redraw when the width of the number column changed.
        //
        // Upstream special-cases `curwin` here and says so in a comment; either
        // every window should be checked or none should. Reproduced.
        let wp = curwin.get();
        // `number_width` is NOT pure -- it caches its answer in the window and
        // resets the 'statuscolumn' width estimate -- so it stays behind the
        // `w_redr_type` test, where upstream's `&&` puts it.
        if (*wp).w_redr_type < UPD_NOT_VALID {
            let nrwidth = if (*wp).w_onebuf_opt.wo_nu != 0
                || (*wp).w_onebuf_opt.wo_rnu != 0
                || *(*wp).w_onebuf_opt.wo_stc != 0
            {
                number_width(wp)
            } else {
                0
            };
            if (*wp).w_nrwidth != nrwidth {
                (*wp).w_redr_type = UPD_NOT_VALID;
            }
        }

        if (*wp).w_redr_type == UPD_INVERTED {
            // So the end of the Visual selection is right.
            update_curswant();
        }

        if redraw_tabline.get() || redr_type >= UPD_NOT_VALID {
            update_window_hl(curwin.get(), redr_type >= UPD_NOT_VALID);
            let mut tp = first_tabpage.get();
            while !tp.is_null() {
                if tp != curtab.get() {
                    update_window_hl((*tp).tp_curwin, redr_type >= UPD_NOT_VALID);
                }
                tp = (*tp).tp_next;
            }
            draw_tabline();
        }

        update_buffer_state(redr_type, hl_changed);

        // Top to bottom through the windows, redrawing the ones that need it.
        let mut did_one = false;
        (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut();

        for wp in windows_in_curtab() {
            if (*wp).w_redr_type == UPD_CLEAR
                && (*wp).w_floating
                && !(*wp).w_grid_alloc.chars.is_null()
            {
                grid_invalidate(&raw mut (*wp).w_grid_alloc);
                (*wp).w_redr_type = UPD_NOT_VALID;
            }

            win_check_ns_hl(wp);
            win_grid_alloc(wp);

            if (*wp).w_redr_border || (*wp).w_redr_type >= UPD_NOT_VALID {
                grid_draw_border(
                    &raw mut (*wp).w_grid_alloc,
                    &raw mut (*wp).w_config,
                    (&raw mut (*wp).w_border_adj).cast::<c_int>(),
                    (*wp).w_onebuf_opt.wo_winbl as c_int,
                    (*wp).w_ns_hl_attr,
                );
            }

            if (*wp).w_redr_type != 0 {
                if !did_one {
                    did_one = true;
                    start_search_hl();
                }
                win_update(wp);
            }

            // The status line and window bar go after the window, to minimise
            // cursor movement.
            if (*wp).w_redr_status {
                win_redr_winbar(wp);
                win_redr_status(wp);
            }
        }

        // Separator connectors go after every window update, so that a
        // connector is never overwritten by a neighbour's separator.
        if did_one {
            for wp in windows_in_curtab() {
                draw_sep_connectors_win(wp);
            }
        }

        end_search_hl();

        if pum_drawn() && must_redraw_pum.get() {
            win_check_ns_hl(curwin.get());
            pum_redraw();
        } else if State.get() & MODE_CMDLINE != 0 {
            pum_check_clear();
        }

        win_check_ns_hl(::core::ptr::null_mut());

        // Reset `b_mod_set`. Going through the windows is probably faster than
        // going through every buffer.
        for wp in windows_in_curtab() {
            (*(*wp).w_buffer).b_mod_set = false;
        }

        updating_screen.set(false);

        if need_maketitle.get() {
            maketitle();
        }

        // Last, because scrolling may mess the command line up.
        if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
            showmode();
        }

        if STILL_MAY_INTRO.get() {
            intro_message(false);
        }
        repeat_message();

        decor_providers_invoke_end();

        // Either the cmdline was cleared, not drawn, or the mode was drawn last.
        // This does not necessarily overwrite an external cmdline.
        if !ui_has(kUICmdline) {
            cmdline_was_last_drawn.set(false);
        }
        OK
    }
}

/// Compile the `'hlsearch'` pattern for the redraw that is starting.
pub unsafe fn start_search_hl() {
    // SAFETY: the screen's search-highlight state, on the main thread.
    unsafe {
        if p_hls.get() == 0 || no_hlsearch.get() {
            return;
        }
        end_search_hl(); // just in case it was not called before
        last_pat_prog(&raw mut (*screen_search_hl.ptr()).rm);
        // Bound the search by 'redrawtime'.
        (*screen_search_hl.ptr()).tm = profile_setlimit(p_rdt.get() as int64_t);
    }
}

/// Free the compiled `'hlsearch'` pattern.
pub unsafe fn end_search_hl() {
    // SAFETY: the screen's search-highlight state, on the main thread.
    unsafe {
        if (*screen_search_hl.ptr()).rm.regprog.is_null() {
            return;
        }
        vim_regfree((*screen_search_hl.ptr()).rm.regprog);
        (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut();
    }
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
    unsafe {
        if !force && !redrawing() {
            return;
        }
        validate_cursor(wp);

        let mut row = (*wp).w_wrow;
        let mut col = (*wp).w_wcol;
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            // With 'rightleft' and the cursor on a double-width character, the
            // cursor goes on its leftmost column.
            let cursor =
                ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).add((*wp).w_cursor.col as usize);
            let cells = if utf_ptr2cells(cursor) == 2 && vim_isprintc(utf_ptr2char(cursor)) {
                2
            } else {
                1
            };
            col = (*wp).w_view_width - (*wp).w_wcol - cells;
        }

        let grid = grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
        if !grid.is_null() {
            ui_grid_cursor_goto((*grid).handle, row, col);
        }
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
    unsafe {
        let fdc = win_fdccol_count(wp);
        let min_width = if wp == curwin.get() && p_wmw.get() == 0 {
            1
        } else {
            p_wmw.get() as c_int
        };
        fdc.min((*wp).w_view_width - (col + min_width))
    }
}

/// The width of window `wp`'s `'number'`/`'relativenumber'` column.
///
/// Callers check whether either option is set; this only decides how wide the
/// column would be. The answer is cached against the line count it was computed
/// for, since it only changes when that crosses a power of ten.
pub unsafe fn number_width(wp: *mut win_T) -> c_int {
    // SAFETY: a live window and its buffer, on the main thread.
    unsafe {
        // With 'relativenumber' alone the largest number shown is the window
        // height (the cursor line shows "0"); otherwise it is the line count.
        let largest = if (*wp).w_onebuf_opt.wo_rnu != 0 && (*wp).w_onebuf_opt.wo_nu == 0 {
            (*wp).w_view_height as linenr_T
        } else {
            (*(*wp).w_buffer).b_ml.ml_line_count
        };

        if largest == (*wp).w_nrwidth_line_count {
            return (*wp).w_nrwidth_width;
        }
        (*wp).w_nrwidth_line_count = largest;

        if *(*wp).w_onebuf_opt.wo_stc != 0 {
            // 'statuscolumn' draws the number itself, so all that is reserved
            // here is 'numberwidth'; the real width is re-estimated from the
            // expression's output.
            (*wp).w_statuscol_line_count = 0;
            (*wp).w_nrwidth_width =
                c_int::from((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                    * (*wp).w_onebuf_opt.wo_nuw as c_int;
            return (*wp).w_nrwidth_width;
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
        n = n.max((*wp).w_onebuf_opt.wo_nuw as c_int - 1);

        // With `'signcolumn'` "number" and a sign to show, the number column
        // needs room for the two-cell sign text.
        if n < 2
            && buf_meta_total((*wp).w_buffer, kMTMetaSignText) != 0
            && (*wp).w_minscwidth == SCL_NUM
        {
            n = 2;
        }

        (*wp).w_nrwidth_width = n;
        n
    }
}

/// Whether the cursor line in window `wp` may be concealed, per
/// `'concealcursor'`.
pub unsafe fn conceal_cursor_line(wp: *const win_T) -> bool {
    // SAFETY: a live window, on the main thread.
    unsafe {
        if *(*wp).w_onebuf_opt.wo_cocu == 0 {
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
        !vim_strchr((*wp).w_onebuf_opt.wo_cocu, mode as c_int).is_null()
    }
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
    unsafe {
        (*wp).w_cursorline = if win_cursorline_standout(wp) {
            (*wp).w_cursor.lnum
        } else {
            0
        };
        if (*wp).w_onebuf_opt.wo_cul != 0 {
            *foldinfo = fold_info(wp, (*wp).w_cursor.lnum);
            if (*foldinfo).fi_level != 0 && (*foldinfo).fi_lines > 0 {
                (*wp).w_cursorline = (*foldinfo).fi_lnum;
            }
        }
    }
}

pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
