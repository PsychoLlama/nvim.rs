use core::ffi::{c_char, c_int};

use crate::src::nvim::autocmd::{EVENT_VIMRESIZED, apply_autocmds};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::buffer::maketitle;
use crate::src::nvim::charset::{vim_isprintc, vim_strsize};
use crate::src::nvim::cmdexpand::cmdline_pum_display;
use crate::src::nvim::decoration::{
    buf_signcols_count_range, decor_conceal_line, decor_range_add_virt, decor_redraw_reset,
    decor_virt_lines, win_lines_concealed,
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
    schar_cache_clear_if_full, win_grid_alloc,
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
    updating_screen,
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
use crate::src::nvim::os::libc::{__assert_fail, abs, gettext};
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
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Integer, MetaIndex,
    OptInt, ScreenGrid, TriState, VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinExtmark, Window, buf_T, colnr_T, foldinfo_T, frame_T, handle_T, hlf_T, int64_t, linenr_T,
    matchitem_T, pos_T, proftime_T, regprog_T, schar_T, size_t, spellvars_T, tabpage_T, uint16_t,
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
unsafe extern "C" {
    static win_extmark_arr: GlobalCell<C2Rust_Unnamed_23>;
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regfree(prog: *mut regprog_T);
}
pub const kFalse: TriState = 0;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
pub const kVPosWinCol: VirtTextPos = 5;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub const kMTMetaSignText: MetaIndex = 3;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SHM_RECORDING: C2Rust_Unnamed_18 = 113;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_18 = 99;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
/// How much of a window has to be redrawn, ordered by severity.
pub type RedrawType = ::core::ffi::c_int;
pub const UPD_CLEAR: RedrawType = 50;
pub const UPD_NOT_VALID: RedrawType = 40;
pub const UPD_SOME_VALID: RedrawType = 35;
pub const UPD_REDRAW_TOP: RedrawType = 30;
pub const UPD_INVERTED_ALL: RedrawType = 25;
pub const UPD_INVERTED: RedrawType = 20;
pub const UPD_VALID: RedrawType = 10;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const DID_FOLD: C2Rust_Unnamed_25 = 3;
pub const DID_LINE: C2Rust_Unnamed_25 = 2;
pub const DID_NONE: C2Rust_Unnamed_25 = 1;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const SHOWCMD_COLS: C2Rust_Unnamed_27 = 10;
pub const MIN_COLUMNS: C2Rust_Unnamed_28 = 12;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
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
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
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
pub unsafe extern "C" fn conceal_check_cursor_line() {
    let mut should_conceal: bool = conceal_cursor_line(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_cole <= 0 as OptInt
        || conceal_cursor_used.get() as ::core::ffi::c_int == should_conceal as ::core::ffi::c_int
    {
        return;
    }
    redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    if decor_conceal_line(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        true_0 != 0,
    ) {
        changed_window_setting(curwin.get());
    }
    curs_columns(curwin.get(), true_0);
}
pub unsafe extern "C" fn redrawing() -> bool {
    return RedrawingDisabled.get() == 0
        && !(p_lz.get() != 0
            && char_avail() as ::core::ffi::c_int != 0
            && !KeyTyped.get()
            && !do_redraw.get());
}
pub unsafe extern "C" fn update_screen() -> ::core::ffi::c_int {
    static still_may_intro: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
    if still_may_intro.get() {
        if !may_show_intro() {
            redraw_later(firstwin.get(), UPD_NOT_VALID);
            still_may_intro.set(false_0 != 0);
        }
    }
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    if resizing_autocmd.get() as ::core::ffi::c_int != 0 || (*default_grid.ptr()).chars.is_null() {
        return FAIL;
    }
    if need_diff_redraw.get() {
        diff_redraw(true_0 != 0);
    }
    if !redrawing()
        || updating_screen.get() as ::core::ffi::c_int != 0
        || cmdline_number_prompt() as ::core::ffi::c_int != 0
    {
        return FAIL;
    }
    let mut type_0: ::core::ffi::c_int = must_redraw.get();
    must_redraw.set(0 as ::core::ffi::c_int);
    updating_screen.set(true_0 != 0);
    display_tick.set((*display_tick.ptr()).wrapping_add(1));
    if schar_cache_clear_if_full() {
        type_0 = if type_0 > UPD_CLEAR {
            type_0
        } else {
            UPD_CLEAR
        };
    }
    if msg_did_scroll.get() {
        msg_did_scroll.set(false_0 != 0);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
    }
    if type_0 >= UPD_CLEAR || !(*default_grid.ptr()).valid {
        ui_comp_set_screen_valid(false_0 != 0);
    }
    if msg_scrolled.get() != 0 || msg_grid_invalid.get() as ::core::ffi::c_int != 0 {
        clear_cmdline.set(true_0 != 0);
        let mut valid: ::core::ffi::c_int =
            if Rows.get() - msg_scrollsize() > 0 as ::core::ffi::c_int {
                Rows.get() - msg_scrollsize()
            } else {
                0 as ::core::ffi::c_int
            };
        if !(*msg_grid.ptr()).chars.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i
                < (if msg_scrollsize() < (*msg_grid.ptr()).rows {
                    msg_scrollsize()
                } else {
                    (*msg_grid.ptr()).rows
                })
            {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset.offset(i as isize),
                    (*msg_grid.ptr()).cols,
                    (i as OptInt) < p_ch.get(),
                );
                i += 1;
            }
        }
        (*msg_grid.ptr()).throttled = false_0 != 0;
        let mut was_invalidated: bool = false_0 != 0;
        if type_0 == UPD_NOT_VALID && !ui_has(kUIMultigrid) && msg_scrolled.get() != 0 {
            was_invalidated = ui_comp_set_screen_valid(false_0 != 0);
            let mut i_0: ::core::ffi::c_int = valid;
            while (i_0 as OptInt) < Rows.get() as OptInt - p_ch.get() {
                grid_clear_line(
                    default_grid.ptr(),
                    *(*default_grid.ptr()).line_offset.offset(i_0 as isize),
                    Columns.get(),
                    false_0 != 0,
                );
                i_0 += 1;
            }
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if !(*wp).w_floating {
                    if (*wp).w_winrow + (*wp).w_height > valid {
                        (*wp).w_redr_type = if (*wp).w_redr_type > UPD_NOT_VALID {
                            (*wp).w_redr_type
                        } else {
                            UPD_NOT_VALID
                        };
                    }
                    if !is_stl_global
                        && (*wp).w_winrow + (*wp).w_height + (*wp).w_status_height > valid
                    {
                        (*wp).w_redr_status = true_0 != 0;
                    }
                }
                wp = (*wp).w_next;
            }
            if is_stl_global as ::core::ffi::c_int != 0
                && Rows.get() as OptInt - p_ch.get() - 1 as OptInt > valid as OptInt
            {
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
        }
        msg_grid_set_pos(Rows.get() - p_ch.get() as ::core::ffi::c_int, false_0 != 0);
        msg_grid_invalid.set(false_0 != 0);
        if was_invalidated {
            ui_comp_set_screen_valid(true_0 != 0);
        }
        msg_scrolled.set(0 as ::core::ffi::c_int);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
        msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
        need_wait_return.set(false_0 != 0);
    }
    win_ui_flush(true_0 != 0);
    compute_cmdrow();
    let mut hl_changed: bool = false_0 != 0;
    if need_highlight_changed.get() {
        highlight_changed();
        hl_changed = true_0 != 0;
    }
    if type_0 == UPD_CLEAR {
        screenclear();
        cmdline_screen_cleared();
        if ui_has(kUIMessages) {
            ui_call_msg_clear();
        }
        type_0 = UPD_NOT_VALID;
        must_redraw.set(0 as ::core::ffi::c_int);
    } else if !(*default_grid.ptr()).valid {
        grid_invalidate(default_grid.ptr());
        (*default_grid.ptr()).valid = true_0 != 0;
    }
    if type_0 == UPD_NOT_VALID
        && clear_cmdline.get() as ::core::ffi::c_int != 0
        && !ui_has(kUIMessages)
    {
        grid_clear(
            default_gridview.ptr(),
            Rows.get() - p_ch.get() as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            0 as ::core::ffi::c_int,
        );
    }
    ui_comp_set_screen_valid(true_0 != 0);
    decor_providers_start();
    if win_check_ns_hl(::core::ptr::null_mut::<win_T>()) {
        redraw_cmdline.set(true_0 != 0);
        redraw_tabline.set(true_0 != 0);
    }
    if clear_cmdline.get() {
        msg_check_for_delay(false_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_NOT_VALID
        && (*curwin.get()).w_nrwidth
            != (if (*curwin.get()).w_onebuf_opt.wo_nu != 0
                || (*curwin.get()).w_onebuf_opt.wo_rnu != 0
                || *(*curwin.get()).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
            {
                number_width(curwin.get())
            } else {
                0 as ::core::ffi::c_int
            })
    {
        (*curwin.get()).w_redr_type = UPD_NOT_VALID;
    }
    if (*curwin.get()).w_redr_type == UPD_INVERTED {
        update_curswant();
    }
    if redraw_tabline.get() as ::core::ffi::c_int != 0 || type_0 >= UPD_NOT_VALID {
        update_window_hl(curwin.get(), type_0 >= UPD_NOT_VALID);
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                update_window_hl((*tp).tp_curwin, type_0 >= UPD_NOT_VALID);
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        draw_tabline();
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        update_window_hl(
            wp_0,
            type_0 >= UPD_NOT_VALID || hl_changed as ::core::ffi::c_int != 0,
        );
        let mut buf: *mut buf_T = (*wp_0).w_buffer;
        if (*buf).b_mod_set {
            if (*buf).b_mod_tick_syn < display_tick.get()
                && syntax_present(wp_0) as ::core::ffi::c_int != 0
            {
                syn_stack_apply_changes(buf);
                (*buf).b_mod_tick_syn = display_tick.get();
            }
            if (*buf).b_mod_tick_decor < display_tick.get() {
                decor_providers_invoke_buf(buf);
                (*buf).b_mod_tick_decor = display_tick.get();
            }
        }
        wp_0 = (*wp_0).w_next;
    }
    let mut did_one: bool = false_0 != 0;
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut wp_1: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_1.is_null() {
        if (*wp_1).w_redr_type == UPD_CLEAR
            && (*wp_1).w_floating as ::core::ffi::c_int != 0
            && !(*wp_1).w_grid_alloc.chars.is_null()
        {
            grid_invalidate(&raw mut (*wp_1).w_grid_alloc);
            (*wp_1).w_redr_type = UPD_NOT_VALID;
        }
        win_check_ns_hl(wp_1);
        win_grid_alloc(wp_1);
        if (*wp_1).w_redr_border as ::core::ffi::c_int != 0 || (*wp_1).w_redr_type >= UPD_NOT_VALID
        {
            grid_draw_border(
                &raw mut (*wp_1).w_grid_alloc,
                &raw mut (*wp_1).w_config,
                &raw mut (*wp_1).w_border_adj as *mut ::core::ffi::c_int,
                (*wp_1).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
                (*wp_1).w_ns_hl_attr,
            );
        }
        if (*wp_1).w_redr_type != 0 as ::core::ffi::c_int {
            if !did_one {
                did_one = true_0 != 0;
                start_search_hl();
            }
            win_update(wp_1);
        }
        if (*wp_1).w_redr_status {
            win_redr_winbar(wp_1);
            win_redr_status(wp_1);
        }
        wp_1 = (*wp_1).w_next;
    }
    if did_one {
        let mut wp_2: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp_2.is_null() {
            draw_sep_connectors_win(wp_2);
            wp_2 = (*wp_2).w_next;
        }
    }
    end_search_hl();
    if pum_drawn() as ::core::ffi::c_int != 0 && must_redraw_pum.get() as ::core::ffi::c_int != 0 {
        win_check_ns_hl(curwin.get());
        pum_redraw();
    } else if State.get() & MODE_CMDLINE != 0 {
        pum_check_clear();
    }
    win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    let mut wp_3: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_3.is_null() {
        (*(*wp_3).w_buffer).b_mod_set = false_0 != 0;
        wp_3 = (*wp_3).w_next;
    }
    updating_screen.set(false_0 != 0);
    if need_maketitle.get() {
        maketitle();
    }
    if clear_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_mode.get() as ::core::ffi::c_int != 0
    {
        showmode();
    }
    if still_may_intro.get() {
        intro_message(false_0 != 0);
    }
    repeat_message();
    decor_providers_invoke_end();
    if !ui_has(kUICmdline) {
        cmdline_was_last_drawn.set(false_0 != 0);
    }
    return OK;
}
pub unsafe extern "C" fn start_search_hl() {
    if p_hls.get() == 0 || no_hlsearch.get() as ::core::ffi::c_int != 0 {
        return;
    }
    end_search_hl();
    last_pat_prog(&raw mut (*screen_search_hl.ptr()).rm);
    (*screen_search_hl.ptr()).tm = profile_setlimit(p_rdt.get() as int64_t);
}
pub unsafe extern "C" fn end_search_hl() {
    if (*screen_search_hl.ptr()).rm.regprog.is_null() {
        return;
    }
    vim_regfree((*screen_search_hl.ptr()).rm.regprog);
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
}
pub unsafe extern "C" fn setcursor() {
    setcursor_mayforce(curwin.get(), false_0 != 0);
}
pub unsafe extern "C" fn setcursor_mayforce(mut wp: *mut win_T, mut force: bool) {
    if force as ::core::ffi::c_int != 0 || redrawing() as ::core::ffi::c_int != 0 {
        validate_cursor(wp);
        let mut row: ::core::ffi::c_int = (*wp).w_wrow;
        let mut col: ::core::ffi::c_int = (*wp).w_wcol;
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            let mut cursor: *mut ::core::ffi::c_char =
                ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
            col = (*wp).w_view_width
                - (*wp).w_wcol
                - (if utf_ptr2cells(cursor) == 2 as ::core::ffi::c_int
                    && vim_isprintc(utf_ptr2char(cursor)) as ::core::ffi::c_int != 0
                {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                });
        }
        let mut grid: *mut ScreenGrid =
            grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
        if !grid.is_null() {
            ui_grid_cursor_goto((*grid).handle, row, col);
        }
    }
}
unsafe extern "C" fn win_update(mut wp: *mut win_T) {
    let mut old_botline: linenr_T = 0;
    if (*wp).w_grid.target == default_grid.ptr() && (*wp).w_wincol >= Columns.get() {
        return;
    }
    let mut top_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mid_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut mid_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bot_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut scrolled_down: bool = false_0 != 0;
    let mut scrolled_for_mod: bool = false_0 != 0;
    let mut top_to_mod: bool = false_0 != 0;
    let mut bot_scroll_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut did_update: C2Rust_Unnamed_25 = DID_NONE;
    let mut syntax_last_parsed: linenr_T = 0 as linenr_T;
    let mut mod_top: linenr_T = 0 as linenr_T;
    let mut mod_bot: linenr_T = 0 as linenr_T;
    let mut type_0: ::core::ffi::c_int = (*wp).w_redr_type;
    if type_0 >= UPD_NOT_VALID {
        (*wp).w_redr_status = true_0 != 0;
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_view_height == 0 as ::core::ffi::c_int {
        draw_hsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        draw_vsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false);
    let mut syntax_tm: proftime_T = profile_setlimit(p_rdt.get() as int64_t);
    syn_set_timeout(&raw mut syntax_tm);
    (*win_extmark_arr.ptr()).size = 0 as size_t;
    decor_redraw_reset(wp, decor_state.ptr());
    decor_providers_invoke_win(wp);
    if !(*buf).terminal.is_null() && terminal_suspended((*buf).terminal) as ::core::ffi::c_int != 0
    {
        static chunk: GlobalCell<VirtTextChunk> = GlobalCell::new(VirtTextChunk {
            text: b"[Process suspended]\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            hl_id: -1 as ::core::ffi::c_int,
        });
        static virt_text: GlobalCell<DecorVirtText> = GlobalCell::new(DecorVirtText {
            flags: 0,
            hl_mode: 0,
            priority: DECOR_PRIORITY_BASE as DecorPriority,
            width: 0,
            col: 0,
            pos: kVPosWinCol,
            data: C2Rust_Unnamed_2 {
                virt_text: VirtText {
                    size: 1 as size_t,
                    capacity: 0,
                    items: (chunk.as_raw() as *const _) as *mut VirtTextChunk,
                },
            },
            next: ::core::ptr::null_mut::<DecorVirtText>(),
        });
        decor_range_add_virt(
            decor_state.ptr(),
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            virt_text.ptr(),
            false_0 != 0,
        );
    }
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_buffer == (*wp).w_buffer && win_redraw_signcols(win) as ::core::ffi::c_int != 0
        {
            changed_line_abv_curs_win(win);
            redraw_later(win, UPD_NOT_VALID);
        }
        win = (*win).w_next;
    }
    (*buf).b_signcols.last_max = (*buf).b_signcols.max;
    validate_virtcol(wp);
    type_0 = (*wp).w_redr_type;
    init_search_hl(wp, screen_search_hl.ptr());
    if (*wp).w_skipcol > 0 as ::core::ffi::c_int && (*wp).w_view_width > win_col_off(wp) {
        let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut add: ::core::ffi::c_int = width1;
        while w < (*wp).w_skipcol {
            if w > 0 as ::core::ffi::c_int {
                add = width2;
            }
            w += add;
        }
        if w != (*wp).w_skipcol {
            (*wp).w_skipcol = (w - add) as colnr_T;
        }
    }
    let nrwidth_before: ::core::ffi::c_int = (*wp).w_nrwidth;
    let mut nrwidth_new: ::core::ffi::c_int = if (*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
    {
        number_width(wp)
    } else {
        0 as ::core::ffi::c_int
    };
    if (*wp).w_nrwidth != nrwidth_new {
        type_0 = UPD_NOT_VALID;
        changed_line_abv_curs_win(wp);
        (*wp).w_nrwidth = nrwidth_new;
    } else {
        mod_top = (*wp).w_redraw_top;
        if (*wp).w_redraw_bot != 0 as linenr_T {
            mod_bot = (*wp).w_redraw_bot + 1 as linenr_T;
        } else {
            mod_bot = 0 as ::core::ffi::c_int as linenr_T;
        }
        if (*buf).b_mod_set {
            if mod_top == 0 as linenr_T || mod_top > (*buf).b_mod_top {
                mod_top = (*buf).b_mod_top;
                if syntax_present(wp) {
                    mod_top -= (*buf).b_s.b_syn_sync_linebreaks;
                    mod_top = if mod_top > 1 as linenr_T {
                        mod_top
                    } else {
                        1 as linenr_T
                    };
                }
            }
            if mod_bot == 0 as linenr_T || mod_bot < (*buf).b_mod_bot {
                mod_bot = (*buf).b_mod_bot;
            }
            if !(*screen_search_hl.ptr()).rm.regprog.is_null()
                && re_multiline((*screen_search_hl.ptr()).rm.regprog) != 0
            {
                top_to_mod = true_0 != 0;
            } else {
                let mut cur: *const matchitem_T = (*wp).w_match_head;
                while !cur.is_null() {
                    if !(*cur).mit_match.regprog.is_null()
                        && re_multiline((*cur).mit_match.regprog) != 0
                    {
                        top_to_mod = true_0 != 0;
                        break;
                    } else {
                        cur = (*cur).mit_next;
                    }
                }
            }
        }
        if search_hl_has_cursor_lnum.get() > 0 as linenr_T {
            if mod_top == 0 as linenr_T || mod_top > search_hl_has_cursor_lnum.get() {
                mod_top = search_hl_has_cursor_lnum.get();
            }
            if mod_bot == 0 as linenr_T || mod_bot < search_hl_has_cursor_lnum.get() + 1 as linenr_T
            {
                mod_bot = search_hl_has_cursor_lnum.get() + 1 as linenr_T;
            }
        }
        if mod_top != 0 as linenr_T && win_lines_concealed(wp) as ::core::ffi::c_int != 0 {
            let mut lnumt: linenr_T = (*wp).w_topline;
            let mut lnumb: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(i as isize)).wl_valid {
                    if (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < mod_top {
                        lnumt = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                    }
                    if lnumb == MAXLNUM as ::core::ffi::c_int as linenr_T
                        && (*(*wp).w_lines.offset(i as isize)).wl_lnum >= mod_bot
                    {
                        lnumb = (*(*wp).w_lines.offset(i as isize)).wl_lnum;
                        if compute_foldcolumn(wp, 0 as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
                        {
                            lnumb += 1;
                        }
                    }
                }
                i += 1;
            }
            hasFolding(
                wp,
                mod_top,
                &raw mut mod_top,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            mod_top = if mod_top < lnumt { mod_top } else { lnumt };
            mod_bot -= 1;
            hasFolding(
                wp,
                mod_bot,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut mod_bot,
            );
            mod_bot += 1;
            mod_bot = if mod_bot > lnumb { mod_bot } else { lnumb };
        }
        if mod_top != 0 as linenr_T && mod_top < (*wp).w_topline {
            if mod_bot > (*wp).w_topline {
                mod_top = (*wp).w_topline;
            } else if syntax_present(wp) {
                top_end = 1 as ::core::ffi::c_int;
            }
        }
    }
    (*wp).w_redraw_top = 0 as ::core::ffi::c_int as linenr_T;
    (*wp).w_redraw_bot = 0 as ::core::ffi::c_int as linenr_T;
    search_hl_has_cursor_lnum.set(0 as ::core::ffi::c_int as linenr_T);
    if type_0 == UPD_REDRAW_TOP {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*wp).w_lines_valid {
            j += (*(*wp).w_lines.offset(i_0 as isize)).wl_size as ::core::ffi::c_int;
            if j >= (*wp).w_upd_rows {
                top_end = j;
                break;
            } else {
                i_0 += 1;
            }
        }
        if top_end == 0 as ::core::ffi::c_int {
            type_0 = UPD_NOT_VALID;
        } else {
            type_0 = UPD_VALID;
        }
    }
    let mut topline_conceal: linenr_T = (*wp).w_topline;
    while topline_conceal < (*buf).b_ml.ml_line_count
        && decor_conceal_line(
            wp,
            topline_conceal as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        topline_conceal += 1;
        hasFolding(
            wp,
            topline_conceal,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut topline_conceal,
        );
    }
    if (type_0 == UPD_VALID
        || type_0 == UPD_SOME_VALID
        || type_0 == UPD_INVERTED
        || type_0 == UPD_INVERTED_ALL)
        && !(*wp).w_botfill
        && !(*wp).w_old_botfill
    {
        if !(mod_top != 0 as linenr_T
            && (*wp).w_topline == mod_top
            && (!(*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                || topline_conceal
                    == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum))
        {
            if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                as ::core::ffi::c_int
                != 0
                && (topline_conceal
                    < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                    || topline_conceal
                        == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        && (*wp).w_topfill > (*wp).w_old_topfill)
            {
                let mut j_0: ::core::ffi::c_int = 0;
                if win_lines_concealed(wp) {
                    j_0 = 0 as ::core::ffi::c_int;
                    let mut ln: linenr_T = (*wp).w_topline;
                    while ln < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum {
                        j_0 += !decor_conceal_line(
                            wp,
                            ln as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            false_0 != 0,
                        ) as ::core::ffi::c_int;
                        if j_0 >= (*wp).w_view_height - 2 as ::core::ffi::c_int {
                            break;
                        }
                        hasFolding(wp, ln, ::core::ptr::null_mut::<linenr_T>(), &raw mut ln);
                        ln += 1;
                    }
                } else {
                    j_0 = ((*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        - (*wp).w_topline) as ::core::ffi::c_int;
                }
                if j_0 < (*wp).w_view_height - 2 as ::core::ffi::c_int {
                    let mut i_1: ::core::ffi::c_int = plines_m_win(
                        wp,
                        (*wp).w_topline,
                        (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                            - 1 as linenr_T,
                        (*wp).w_view_height,
                    );
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        != (*wp).w_topline
                    {
                        i_1 += win_get_fill(
                            wp,
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum,
                        ) - (*wp).w_old_topfill;
                    }
                    if i_1 != 0 as ::core::ffi::c_int
                        && i_1 < (*wp).w_view_height - 2 as ::core::ffi::c_int
                    {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, i_1);
                        bot_scroll_start = 0 as ::core::ffi::c_int;
                        if (*wp).w_lines_valid != 0 as ::core::ffi::c_int {
                            top_end = i_1;
                            scrolled_down = true_0 != 0;
                            (*wp).w_lines_valid += j_0 as linenr_T as ::core::ffi::c_int;
                            if (*wp).w_lines_valid > (*wp).w_view_height {
                                (*wp).w_lines_valid = (*wp).w_view_height;
                            }
                            let mut idx: ::core::ffi::c_int = 0;
                            idx = (*wp).w_lines_valid;
                            while idx - j_0 >= 0 as ::core::ffi::c_int {
                                *(*wp).w_lines.offset(idx as isize) =
                                    *(*wp).w_lines.offset((idx - j_0) as isize);
                                idx -= 1;
                            }
                            while idx >= 0 as ::core::ffi::c_int {
                                let c2rust_fresh0 = idx;
                                idx = idx - 1;
                                (*(*wp).w_lines.offset(c2rust_fresh0 as isize)).wl_valid =
                                    false_0 != 0;
                            }
                        }
                    } else {
                        mid_start = 0 as ::core::ffi::c_int;
                    }
                } else {
                    mid_start = 0 as ::core::ffi::c_int;
                }
            } else {
                let mut j_1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_2 < (*wp).w_lines_valid {
                    if (*(*wp).w_lines.offset(i_2 as isize)).wl_valid as ::core::ffi::c_int != 0
                        && (*(*wp).w_lines.offset(i_2 as isize)).wl_lnum == (*wp).w_topline
                    {
                        j_1 = i_2;
                        break;
                    } else {
                        row += (*(*wp).w_lines.offset(i_2 as isize)).wl_size as ::core::ffi::c_int;
                        i_2 += 1;
                    }
                }
                if j_1 == -1 as ::core::ffi::c_int {
                    mid_start = 0 as ::core::ffi::c_int;
                } else {
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        == (*wp).w_topline
                    {
                        row += (*wp).w_old_topfill;
                    } else {
                        row += win_get_fill(wp, (*wp).w_topline);
                    }
                    row -= (*wp).w_topfill;
                    if row > 0 as ::core::ffi::c_int {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, -row);
                        bot_start = (*wp).w_view_height - row;
                        bot_scroll_start = bot_start;
                    }
                    if (row == 0 as ::core::ffi::c_int || bot_start < 999 as ::core::ffi::c_int)
                        && (*wp).w_lines_valid != 0 as ::core::ffi::c_int
                    {
                        bot_start = 0 as ::core::ffi::c_int;
                        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        loop {
                            *(*wp).w_lines.offset(idx_0 as isize) =
                                *(*wp).w_lines.offset(j_1 as isize);
                            if row > 0 as ::core::ffi::c_int
                                && bot_start
                                    + row
                                    + (*(*wp).w_lines.offset(j_1 as isize)).wl_size
                                        as ::core::ffi::c_int
                                    > (*wp).w_view_height
                            {
                                (*wp).w_lines_valid = idx_0 + 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                let c2rust_fresh1 = idx_0;
                                idx_0 = idx_0 + 1;
                                bot_start += (*(*wp).w_lines.offset(c2rust_fresh1 as isize)).wl_size
                                    as ::core::ffi::c_int;
                                j_1 += 1;
                                if j_1 < (*wp).w_lines_valid {
                                    continue;
                                }
                                (*wp).w_lines_valid = idx_0;
                                break;
                            }
                        }
                        if win_may_fill(wp) as ::core::ffi::c_int != 0
                            && bot_start > 0 as ::core::ffi::c_int
                        {
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_size =
                                plines_correct_topline(
                                    wp,
                                    (*wp).w_topline,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    true_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                ) as uint16_t;
                        }
                    }
                }
            }
        }
        if mid_start == 0 as ::core::ffi::c_int {
            mid_end = (*wp).w_view_height;
        }
    } else {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
    }
    if type_0 == UPD_SOME_VALID {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
        type_0 = UPD_NOT_VALID;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer
        || (*wp).w_old_cursor_lnum != 0 as linenr_T && type_0 != UPD_NOT_VALID
    {
        let mut from: linenr_T = 0;
        let mut to: linenr_T = 0;
        if VIsual_active.get() {
            if VIsual_mode.get() != (*wp).w_old_visual_mode as ::core::ffi::c_int
                || type_0 == UPD_INVERTED_ALL
            {
                if (*curwin.get()).w_cursor.lnum < (*VIsual.ptr()).lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*VIsual.ptr()).lnum;
                } else {
                    from = (*VIsual.ptr()).lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                }
                from = if (if from < (*wp).w_old_cursor_lnum {
                    from
                } else {
                    (*wp).w_old_cursor_lnum
                }) < (*wp).w_old_visual_lnum
                {
                    if from < (*wp).w_old_cursor_lnum {
                        from
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
                to = if (if to > (*wp).w_old_cursor_lnum {
                    to
                } else {
                    (*wp).w_old_cursor_lnum
                }) > (*wp).w_old_visual_lnum
                {
                    if to > (*wp).w_old_cursor_lnum {
                        to
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
            } else {
                if (*curwin.get()).w_cursor.lnum < (*wp).w_old_cursor_lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*wp).w_old_cursor_lnum;
                } else {
                    from = (*wp).w_old_cursor_lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                    if from == 0 as linenr_T {
                        from = to;
                    }
                }
                if (*VIsual.ptr()).lnum != (*wp).w_old_visual_lnum
                    || (*VIsual.ptr()).col != (*wp).w_old_visual_col
                {
                    if (*wp).w_old_visual_lnum < from && (*wp).w_old_visual_lnum != 0 as linenr_T {
                        from = (*wp).w_old_visual_lnum;
                    }
                    to = if (if to > (*wp).w_old_visual_lnum {
                        to
                    } else {
                        (*wp).w_old_visual_lnum
                    }) > (*VIsual.ptr()).lnum
                    {
                        if to > (*wp).w_old_visual_lnum {
                            to
                        } else {
                            (*wp).w_old_visual_lnum
                        }
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
            }
            if VIsual_mode.get() == Ctrl_V {
                let mut fromc: colnr_T = 0;
                let mut toc: colnr_T = 0;
                let mut save_ve_flags: ::core::ffi::c_uint =
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags;
                if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 {
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags =
                        kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint;
                }
                getvcols(
                    wp,
                    VIsual.ptr(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut fromc,
                    &raw mut toc,
                );
                toc += 1;
                (*curwin.get()).w_onebuf_opt.wo_ve_flags = save_ve_flags;
                if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                    if get_ve_flags(curwin.get())
                        & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        let mut pos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        let mut cursor_above: ::core::ffi::c_int = ((*curwin.get()).w_cursor.lnum
                            < (*VIsual.ptr()).lnum)
                            as ::core::ffi::c_int;
                        toc = 0 as ::core::ffi::c_int as colnr_T;
                        pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        pos.lnum = (*curwin.get()).w_cursor.lnum;
                        while if cursor_above != 0 {
                            (pos.lnum <= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } else {
                            (pos.lnum >= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } != 0
                        {
                            let mut t: colnr_T = 0;
                            pos.col = ml_get_buf_len((*wp).w_buffer, pos.lnum);
                            getvvcol(
                                wp,
                                &raw mut pos,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut t,
                            );
                            toc = if toc > t { toc } else { t };
                            pos.lnum = (pos.lnum as ::core::ffi::c_int
                                + if cursor_above != 0 {
                                    1 as ::core::ffi::c_int
                                } else {
                                    -1 as ::core::ffi::c_int
                                }) as linenr_T;
                        }
                        toc += 1;
                    } else {
                        toc = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                }
                if fromc != (*wp).w_old_cursor_fcol || toc != (*wp).w_old_cursor_lcol {
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    to = if to > (*VIsual.ptr()).lnum {
                        to
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
                (*wp).w_old_cursor_fcol = fromc;
                (*wp).w_old_cursor_lcol = toc;
            }
        } else if (*wp).w_old_cursor_lnum < (*wp).w_old_visual_lnum {
            from = (*wp).w_old_cursor_lnum;
            to = (*wp).w_old_visual_lnum;
        } else {
            from = (*wp).w_old_visual_lnum;
            to = (*wp).w_old_cursor_lnum;
        }
        from = if from > (*wp).w_topline {
            from
        } else {
            (*wp).w_topline
        };
        if (*wp).w_valid & VALID_BOTLINE != 0 {
            from = if from < (*wp).w_botline - 1 as linenr_T {
                from
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
            to = if to < (*wp).w_botline - 1 as linenr_T {
                to
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
        }
        if mid_start > 0 as ::core::ffi::c_int {
            let mut lnum: linenr_T = (*wp).w_topline;
            let mut idx_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut srow: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if scrolled_down {
                mid_start = top_end;
            } else {
                mid_start = 0 as ::core::ffi::c_int;
            }
            while lnum < from && idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid {
                    mid_start +=
                        (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                } else if !scrolled_down {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                }
                idx_1 += 1;
                if idx_1 < (*wp).w_lines_valid
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                {
                    lnum = (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum;
                } else {
                    lnum += 1;
                }
            }
            srow += mid_start;
            mid_end = (*wp).w_view_height;
            while idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum >= to + 1 as linenr_T
                {
                    mid_end = srow;
                    break;
                } else {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                    idx_1 += 1;
                }
            }
        }
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer {
        (*wp).w_old_visual_mode = VIsual_mode.get() as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*wp).w_old_visual_lnum = (*VIsual.ptr()).lnum;
        (*wp).w_old_visual_col = (*VIsual.ptr()).col;
        (*wp).w_old_curswant = (*curwin.get()).w_curswant;
    } else {
        (*wp).w_old_visual_mode = 0 as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_col = 0 as ::core::ffi::c_int as colnr_T;
    }
    let mut cursorline_fi: foldinfo_T = foldinfo_T {
        fi_lnum: 0 as linenr_T,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    win_update_cursorline(wp, &raw mut cursorline_fi);
    if wp == curwin.get() {
        conceal_cursor_used.set(conceal_cursor_line(curwin.get()));
    }
    win_check_ns_hl(wp);
    let mut spv: spellvars_T = spellvars_T {
        spv_has_spell: false,
        spv_unchanged: false,
        spv_checked_col: 0,
        spv_checked_lnum: 0,
        spv_cap_col: 0,
        spv_capcol_lnum: 0,
    };
    let mut lnum_0: linenr_T = (*wp).w_topline;
    if spell_check_window(wp) {
        spv.spv_has_spell = true_0 != 0;
        spv.spv_unchanged = mod_top == 0 as linenr_T;
    }
    let mut idx_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut srow_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eof: bool = false_0 != 0;
    let mut didline: bool = false_0 != 0;
    's_2363: {
        's_2327: loop {
            '_redr_statuscol: {
                's_2139: {
                    if row_0 == (*wp).w_view_height {
                        didline = true_0 != 0;
                    } else if lnum_0 > (*buf).b_ml.ml_line_count {
                        eof = true_0 != 0;
                    } else {
                        srow_0 = row_0;
                        if row_0 < top_end
                            || row_0 >= mid_start && row_0 < mid_end
                            || top_to_mod as ::core::ffi::c_int != 0
                            || idx_2 >= (*wp).w_lines_valid
                            || row_0
                                + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                    as ::core::ffi::c_int
                                > bot_start
                            || mod_top != 0 as linenr_T
                                && (lnum_0 == mod_top
                                    || lnum_0 >= mod_top
                                        && (lnum_0 < mod_bot
                                            || did_update as ::core::ffi::c_uint
                                                == DID_FOLD as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || did_update as ::core::ffi::c_uint
                                                == DID_LINE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && syntax_present(wp) as ::core::ffi::c_int != 0
                                                && (foldmethodIsSyntax(wp) as ::core::ffi::c_int
                                                    != 0
                                                    && hasAnyFolding(wp) != 0
                                                    || syntax_check_changed(lnum_0)
                                                        as ::core::ffi::c_int
                                                        != 0)
                                            || !(*wp).w_match_head.is_null()
                                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                                && (*buf).b_mod_xlines != 0 as linenr_T))
                            || lnum_0 == (*wp).w_cursorline
                            || lnum_0 == (*wp).w_last_cursorline
                        {
                            if lnum_0 == mod_top {
                                top_to_mod = false_0 != 0;
                            }
                            let mut foldinfo: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                && lnum_0 == (*wp).w_cursor.lnum
                            {
                                cursorline_fi
                            } else {
                                fold_info(wp, lnum_0)
                            };
                            let mut concealed: bool = decor_conceal_line(
                                wp,
                                lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                            if concealed as ::core::ffi::c_int != 0
                                && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                            {
                                if lnum_0 == mod_top && lnum_0 < mod_bot {
                                    mod_top = (mod_top as ::core::ffi::c_int
                                        + (if foldinfo.fi_lines != 0 {
                                            foldinfo.fi_lines
                                        } else {
                                            1 as linenr_T
                                        })
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                }
                                lnum_0 = (lnum_0 as ::core::ffi::c_int
                                    + (if foldinfo.fi_lines != 0 {
                                        foldinfo.fi_lines
                                    } else {
                                        1 as linenr_T
                                    }) as ::core::ffi::c_int)
                                    as linenr_T;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                continue 's_2327;
                            } else {
                                if !scrolled_for_mod
                                    && mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                    && lnum_0 >= mod_top
                                    && lnum_0
                                        < (if mod_bot > mod_top + 1 as linenr_T {
                                            mod_bot
                                        } else {
                                            mod_top + 1 as linenr_T
                                        })
                                    && (!scrolled_down || row_0 >= top_end)
                                {
                                    scrolled_for_mod = true_0 != 0;
                                    let mut old_cline_height: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let mut old_rows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut l: linenr_T = 0;
                                    let mut i_3: ::core::ffi::c_int = 0;
                                    i_3 = idx_2;
                                    while i_3 < (*wp).w_lines_valid {
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                                == mod_bot
                                        {
                                            break;
                                        }
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                            == (*wp).w_cursor.lnum
                                        {
                                            old_cline_height = (*(*wp).w_lines.offset(i_3 as isize))
                                                .wl_size
                                                as ::core::ffi::c_int;
                                        }
                                        old_rows += (*(*wp).w_lines.offset(i_3 as isize)).wl_size
                                            as ::core::ffi::c_int;
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lastlnum
                                                + 1 as linenr_T
                                                == mod_bot
                                        {
                                            i_3 += 1;
                                            while i_3 < (*wp).w_lines_valid
                                                && !(*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            {
                                                let c2rust_fresh2 = i_3;
                                                i_3 = i_3 + 1;
                                                old_rows +=
                                                    (*(*wp).w_lines.offset(c2rust_fresh2 as isize))
                                                        .wl_size
                                                        as ::core::ffi::c_int;
                                            }
                                            break;
                                        } else {
                                            i_3 += 1;
                                        }
                                    }
                                    if i_3 >= (*wp).w_lines_valid {
                                        bot_start = 0 as ::core::ffi::c_int;
                                        bot_scroll_start = 0 as ::core::ffi::c_int;
                                    } else {
                                        let mut new_rows: ::core::ffi::c_int =
                                            0 as ::core::ffi::c_int;
                                        let mut j_2: ::core::ffi::c_int = idx_2;
                                        l = lnum_0;
                                        while l < mod_bot {
                                            if dollar_vcol.get() >= 0 as ::core::ffi::c_int
                                                && wp == curwin.get()
                                                && old_cline_height > 0 as ::core::ffi::c_int
                                                && l == (*wp).w_cursor.lnum
                                            {
                                                new_rows += old_cline_height;
                                                j_2 += 1;
                                            } else {
                                                let mut n: ::core::ffi::c_int =
                                                    plines_correct_topline(
                                                        wp,
                                                        l,
                                                        &raw mut l,
                                                        true_0 != 0,
                                                        ::core::ptr::null_mut::<bool>(),
                                                    );
                                                new_rows += n;
                                                j_2 += (n > 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            }
                                            if new_rows
                                                > (*wp).w_view_height
                                                    - row_0
                                                    - 2 as ::core::ffi::c_int
                                            {
                                                new_rows = 9999 as ::core::ffi::c_int;
                                                break;
                                            } else {
                                                l += 1;
                                            }
                                        }
                                        let mut xtra_rows: ::core::ffi::c_int = new_rows - old_rows;
                                        if xtra_rows < 0 as ::core::ffi::c_int {
                                            if row_0 - xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0, xtra_rows);
                                                bot_start = (*wp).w_view_height + xtra_rows;
                                                bot_scroll_start = bot_start;
                                            }
                                        } else if xtra_rows > 0 as ::core::ffi::c_int {
                                            if row_0 + xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0 + old_rows, xtra_rows);
                                                bot_scroll_start = 0 as ::core::ffi::c_int;
                                                if top_end > row_0 + old_rows {
                                                    top_end += xtra_rows;
                                                }
                                            }
                                        }
                                        if mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                            && i_3 != j_2
                                        {
                                            if j_2 < i_3 {
                                                let mut x: ::core::ffi::c_int = row_0 + new_rows;
                                                loop {
                                                    if i_3 >= (*wp).w_lines_valid {
                                                        (*wp).w_lines_valid = j_2;
                                                        break;
                                                    } else {
                                                        *(*wp).w_lines.offset(j_2 as isize) =
                                                            *(*wp).w_lines.offset(i_3 as isize);
                                                        if x + (*(*wp).w_lines.offset(j_2 as isize))
                                                            .wl_size
                                                            as ::core::ffi::c_int
                                                            > (*wp).w_view_height
                                                        {
                                                            (*wp).w_lines_valid =
                                                                j_2 + 1 as ::core::ffi::c_int;
                                                            break;
                                                        } else {
                                                            let c2rust_fresh3 = j_2;
                                                            j_2 = j_2 + 1;
                                                            x += (*(*wp)
                                                                .w_lines
                                                                .offset(c2rust_fresh3 as isize))
                                                            .wl_size
                                                                as ::core::ffi::c_int;
                                                            i_3 += 1;
                                                        }
                                                    }
                                                }
                                                bot_start =
                                                    if bot_start < x { bot_start } else { x };
                                            } else {
                                                j_2 -= i_3;
                                                (*wp).w_lines_valid +=
                                                    j_2 as linenr_T as ::core::ffi::c_int;
                                                (*wp).w_lines_valid =
                                                    if (*wp).w_lines_valid < (*wp).w_view_height {
                                                        (*wp).w_lines_valid
                                                    } else {
                                                        (*wp).w_view_height
                                                    };
                                                i_3 = (*wp).w_lines_valid;
                                                while i_3 - j_2 >= idx_2 {
                                                    *(*wp).w_lines.offset(i_3 as isize) =
                                                        *(*wp).w_lines.offset((i_3 - j_2) as isize);
                                                    i_3 -= 1;
                                                }
                                                while i_3 >= idx_2 {
                                                    (*(*wp).w_lines.offset(i_3 as isize)).wl_size =
                                                        0 as uint16_t;
                                                    let c2rust_fresh4 = i_3;
                                                    i_3 = i_3 - 1;
                                                    (*(*wp)
                                                        .w_lines
                                                        .offset(c2rust_fresh4 as isize))
                                                    .wl_valid = false_0 != 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                if foldinfo.fi_lines == 0 as linenr_T
                                    && idx_2 < (*wp).w_lines_valid
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid
                                        as ::core::ffi::c_int
                                        != 0
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum == lnum_0
                                    && lnum_0 > (*wp).w_topline
                                    && dy_flags.get()
                                        & (kOptDyFlagLastline as ::core::ffi::c_int
                                            | kOptDyFlagTruncate as ::core::ffi::c_int)
                                            as ::core::ffi::c_uint
                                        == 0
                                    && srow_0
                                        + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                            as ::core::ffi::c_int
                                        > (*wp).w_view_height
                                    && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                                {
                                    row_0 = (*wp).w_view_height + 1 as ::core::ffi::c_int;
                                } else {
                                    prepare_search_hl(wp, screen_search_hl.ptr(), lnum_0);
                                    if syntax_last_parsed != 0 as linenr_T
                                        && (syntax_last_parsed + 1 as linenr_T) < lnum_0
                                        && syntax_present(wp) as ::core::ffi::c_int != 0
                                    {
                                        syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                                    }
                                    let mut display_buf_line: bool = !concealed
                                        && (foldinfo.fi_lines == 0 as linenr_T
                                            || *(*wp).w_onebuf_opt.wo_fdt as ::core::ffi::c_int
                                                == NUL);
                                    let mut zero_spv: spellvars_T = spellvars_T {
                                        spv_has_spell: false,
                                        spv_unchanged: false,
                                        spv_checked_col: 0,
                                        spv_checked_lnum: 0,
                                        spv_cap_col: 0,
                                        spv_capcol_lnum: 0,
                                    };
                                    row_0 = win_line(
                                        wp,
                                        lnum_0,
                                        srow_0,
                                        (*wp).w_view_height,
                                        0 as ::core::ffi::c_int,
                                        concealed,
                                        if display_buf_line as ::core::ffi::c_int != 0 {
                                            &raw mut spv
                                        } else {
                                            &raw mut zero_spv
                                        },
                                        foldinfo,
                                    );
                                    if display_buf_line {
                                        syntax_last_parsed = lnum_0;
                                    } else {
                                        spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    }
                                    let mut lastlnum: linenr_T = lnum_0 + foldinfo.fi_lines
                                        - (foldinfo.fi_lines > 0 as linenr_T) as ::core::ffi::c_int;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_folded =
                                        foldinfo.fi_lines > 0 as linenr_T;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_foldend = lastlnum;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum = lastlnum;
                                    did_update = (if foldinfo.fi_lines > 0 as linenr_T {
                                        DID_FOLD as ::core::ffi::c_int
                                    } else {
                                        DID_LINE as ::core::ffi::c_int
                                    })
                                        as C2Rust_Unnamed_25;
                                    let mut virt_below: bool = decor_virt_lines(
                                        wp,
                                        lastlnum as ::core::ffi::c_int,
                                        lastlnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        ::core::ptr::null_mut::<VirtLines>(),
                                        true_0 != 0,
                                    ) > 0 as ::core::ffi::c_int;
                                    while !virt_below
                                        && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                            < (*buf).b_ml.ml_line_count
                                        && decor_conceal_line(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                                as ::core::ffi::c_int,
                                            false_0 != 0,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        virt_below = false_0 != 0;
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum += 1;
                                        hasFolding(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            &raw mut (*(*wp).w_lines.offset(idx_2 as isize))
                                                .wl_lastlnum,
                                        );
                                    }
                                }
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum = lnum_0;
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid = true_0 != 0;
                                let mut is_curline: bool =
                                    wp == curwin.get() && lnum_0 == (*wp).w_cursor.lnum;
                                if row_0 > (*wp).w_view_height {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            plines_win(wp, lnum_0, true_0 != 0) as uint16_t;
                                    }
                                    idx_2 += 1;
                                    break 's_2139;
                                } else {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            (row_0 - srow_0) as uint16_t;
                                    }
                                    let c2rust_fresh5 = idx_2;
                                    idx_2 = idx_2 + 1;
                                    lnum_0 = (*(*wp).w_lines.offset(c2rust_fresh5 as isize))
                                        .wl_lastlnum
                                        + 1 as linenr_T;
                                }
                            }
                        } else {
                            if (*wp).w_onebuf_opt.wo_nu != 0
                                && mod_top != 0 as linenr_T
                                && lnum_0 >= mod_bot
                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                && (*buf).b_mod_xlines != 0 as linenr_T
                                || (*wp).w_onebuf_opt.wo_rnu != 0
                                    && (*wp).w_last_cursor_lnum_rnu != (*wp).w_cursor.lnum
                            {
                                let mut info: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                    && lnum_0 == (*wp).w_cursor.lnum
                                {
                                    cursorline_fi
                                } else {
                                    fold_info(wp, lnum_0)
                                };
                                win_line(
                                    wp,
                                    lnum_0,
                                    srow_0,
                                    (*wp).w_view_height,
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                        as ::core::ffi::c_int,
                                    false_0 != 0,
                                    &raw mut spv,
                                    info,
                                );
                            }
                            let c2rust_fresh6 = idx_2;
                            idx_2 = idx_2 + 1;
                            row_0 += (*(*wp).w_lines.offset(c2rust_fresh6 as isize)).wl_size
                                as ::core::ffi::c_int;
                            if row_0 > (*wp).w_view_height {
                                break 's_2139;
                            } else {
                                lnum_0 = (*(*wp)
                                    .w_lines
                                    .offset((idx_2 - 1 as ::core::ffi::c_int) as isize))
                                .wl_lastlnum
                                    + 1 as linenr_T;
                                did_update = DID_NONE;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                        if (*wp).w_redr_statuscol {
                            break '_redr_statuscol;
                        } else {
                            if lnum_0 <= (*buf).b_ml.ml_line_count {
                                continue 's_2327;
                            }
                            eof = true_0 != 0;
                        }
                    }
                }
                (*wp).w_last_cursorline = (*wp).w_cursorline;
                (*wp).w_last_cursor_lnum_rnu = if (*wp).w_onebuf_opt.wo_rnu != 0 {
                    (*wp).w_cursor.lnum
                } else {
                    0 as linenr_T
                };
                (*wp).w_lines_valid = if (*wp).w_lines_valid > idx_2 {
                    (*wp).w_lines_valid
                } else {
                    idx_2
                };
                (*wp).w_display_tick = display_tick.get();
                if syntax_last_parsed != 0 as linenr_T
                    && syntax_present(wp) as ::core::ffi::c_int != 0
                {
                    syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                }
                old_botline = (*wp).w_botline;
                (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
                (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
                if !eof && !didline {
                    let mut at_attr: ::core::ffi::c_int =
                        hl_combine_attr(win_bg_attr(wp), win_hl_attr(wp, HLF_AT));
                    if lnum_0 == (*wp).w_topline {
                        (*wp).w_botline = lnum_0 + 1 as linenr_T;
                    } else if win_get_fill(wp, lnum_0) >= (*wp).w_view_height - srow_0 {
                        (*wp).w_botline = lnum_0;
                        (*wp).w_filler_rows = (*wp).w_view_height - srow_0;
                    } else if dy_flags.get()
                        & kOptDyFlagTruncate as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        grid_line_fill(
                            0 as ::core::ffi::c_int,
                            if (*wp).w_view_width < 3 as ::core::ffi::c_int {
                                (*wp).w_view_width
                            } else {
                                3 as ::core::ffi::c_int
                            },
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_fill(
                            3 as ::core::ffi::c_int,
                            (*wp).w_view_width,
                            ' ' as ::core::ffi::c_int as schar_T,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else if dy_flags.get()
                        & kOptDyFlagLastline as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        let mut width: ::core::ffi::c_int = if grid_line_getchar(
                            if (*wp).w_view_width - 3 as ::core::ffi::c_int
                                > 0 as ::core::ffi::c_int
                            {
                                (*wp).w_view_width - 3 as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ) == NUL as schar_T
                        {
                            4 as ::core::ffi::c_int
                        } else {
                            3 as ::core::ffi::c_int
                        };
                        grid_line_fill(
                            if (*wp).w_view_width - width > 0 as ::core::ffi::c_int {
                                (*wp).w_view_width - width
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            (*wp).w_view_width,
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else {
                        win_draw_end(
                            wp,
                            (*wp).w_p_fcs_chars.lastline,
                            true_0 != 0,
                            srow_0,
                            (*wp).w_view_height,
                            HLF_AT,
                        );
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2363;
                } else if eof {
                    (*wp).w_botline = (*buf).b_ml.ml_line_count + 1 as linenr_T;
                    let mut j_3: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_botline);
                    if !(j_3 > 0 as ::core::ffi::c_int
                        && !(*wp).w_botfill
                        && row_0 < (*wp).w_view_height)
                    {
                        break 's_2327;
                    }
                    let mut zero_spv_0: spellvars_T = spellvars_T {
                        spv_has_spell: false,
                        spv_unchanged: false,
                        spv_checked_col: 0,
                        spv_checked_lnum: 0,
                        spv_cap_col: 0,
                        spv_capcol_lnum: 0,
                    };
                    let mut zero_foldinfo: foldinfo_T = foldinfo_T {
                        fi_lnum: 0 as linenr_T,
                        fi_level: 0,
                        fi_low_level: 0,
                        fi_lines: 0,
                    };
                    row_0 = win_line(
                        wp,
                        (*wp).w_botline,
                        row_0,
                        (*wp).w_view_height,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        &raw mut zero_spv_0,
                        zero_foldinfo,
                    );
                    if !(*wp).w_redr_statuscol {
                        break 's_2327;
                    }
                    eof = false_0 != 0;
                } else {
                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2327;
                }
            }
            (*wp).w_redr_statuscol = false_0 != 0;
            idx_2 = 0 as ::core::ffi::c_int;
            row_0 = 0 as ::core::ffi::c_int;
            lnum_0 = (*wp).w_topline;
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
            (*wp).w_valid &= !VALID_WCOL;
            decor_redraw_reset(wp, decor_state.ptr());
            decor_providers_invoke_win(wp);
        }
        let mut lastline: ::core::ffi::c_int = bot_scroll_start;
        if mid_end >= row_0 {
            lastline = if lastline < mid_start {
                lastline
            } else {
                mid_start
            };
        }
        if mod_bot > (*buf).b_ml.ml_line_count {
            lastline = 0 as ::core::ffi::c_int;
        }
        win_draw_end(
            wp,
            (*wp).w_p_fcs_chars.eob,
            false_0 != 0,
            if lastline > row_0 { lastline } else { row_0 },
            (*wp).w_view_height,
            HLF_EOB,
        );
        set_empty_rows(wp, row_0);
    }
    if (*wp).w_redr_type >= UPD_REDRAW_TOP {
        draw_vsep_win(wp);
        draw_hsep_win(wp);
    }
    syn_set_timeout(::core::ptr::null_mut::<proftime_T>());
    (*wp).w_redr_type = 0 as ::core::ffi::c_int;
    (*wp).w_old_topfill = (*wp).w_topfill;
    (*wp).w_old_botfill = (*wp).w_botfill;
    let mut n_0: size_t = 0 as size_t;
    while n_0 < (*win_extmark_arr.ptr()).size {
        ui_call_win_extmark(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).ns_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).mark_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_row as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_col as Integer,
        );
        n_0 = n_0.wrapping_add(1);
    }
    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
        (*wp).w_valid |= VALID_BOTLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
        if wp == curwin.get() && (*wp).w_botline != old_botline && !recursive.get() {
            recursive.set(true_0 != 0);
            (*curwin.get()).w_valid &= !VALID_TOPLINE;
            update_topline(curwin.get());
            if must_redraw.get() != 0 as ::core::ffi::c_int {
                let mut mod_set: ::core::ffi::c_int =
                    (*curbuf.get()).b_mod_set as ::core::ffi::c_int;
                (*curbuf.get()).b_mod_set = false_0 != 0;
                curs_columns(curwin.get(), true_0);
                win_update(curwin.get());
                must_redraw.set(0 as ::core::ffi::c_int);
                (*curbuf.get()).b_mod_set = mod_set != 0;
            }
            recursive.set(false_0 != 0);
        }
    }
    if nrwidth_before != (*wp).w_nrwidth && !(*buf).terminal.is_null() {
        terminal_check_size((*buf).terminal);
    }
    if !got_int.get() {
        got_int.set(save_got_int != 0);
    }
}
pub unsafe extern "C" fn win_scroll_lines(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
) {
    if !redrawing() || line_count == 0 as ::core::ffi::c_int {
        return;
    }
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid =
        grid_adjust(&raw mut (*wp).w_grid, &raw mut row_off, &raw mut col);
    let mut checked_width: ::core::ffi::c_int = if (*grid).cols - col < (*wp).w_view_width {
        (*grid).cols - col
    } else {
        (*wp).w_view_width
    };
    let mut checked_height: ::core::ffi::c_int = if (*grid).rows - row_off < (*wp).w_view_height {
        (*grid).rows - row_off
    } else {
        (*wp).w_view_height
    };
    if row + abs(line_count) >= checked_height {
        return;
    }
    if line_count < 0 as ::core::ffi::c_int {
        grid_del_lines(
            grid,
            row + row_off,
            -line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    } else {
        grid_ins_lines(
            grid,
            row + row_off,
            line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    };
}
pub unsafe extern "C" fn win_draw_end(
    mut wp: *mut win_T,
    mut c1: schar_T,
    mut draw_margin: bool,
    mut startrow: ::core::ffi::c_int,
    mut endrow: ::core::ffi::c_int,
    mut hl: hlf_T,
) {
    '_c2rust_label: {
        if hl as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
            && (hl as ::core::ffi::c_uint) < HLF_COUNT as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"hl >= 0 && hl < HLF_COUNT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2513 as ::core::ffi::c_uint,
                b"void win_draw_end(win_T *, schar_T, _Bool, int, int, hlf_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let view_width: ::core::ffi::c_int = (*wp).w_view_width;
    let fdc: ::core::ffi::c_int = compute_foldcolumn(wp, 0 as ::core::ffi::c_int);
    let scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
    let mut row: ::core::ffi::c_int = startrow;
    while row < endrow {
        grid_line_start(&raw mut (*wp).w_grid, row);
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if draw_margin {
            if fdc > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + fdc {
                        view_width
                    } else {
                        n + fdc
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_FC),
                );
            }
            if scwidth > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + scwidth * SIGN_WIDTH as ::core::ffi::c_int {
                        view_width
                    } else {
                        n + scwidth * SIGN_WIDTH as ::core::ffi::c_int
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_SC),
                );
            }
            if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                && vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
            {
                let mut width: ::core::ffi::c_int = number_width(wp) + 1 as ::core::ffi::c_int;
                n = grid_line_fill(
                    n,
                    if view_width < n + width {
                        view_width
                    } else {
                        n + width
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_N),
                );
            }
        }
        let mut attr: ::core::ffi::c_int = win_hl_attr(wp, hl as ::core::ffi::c_int);
        if n < view_width {
            grid_line_put_schar(n, c1, attr);
            n += 1;
        }
        grid_line_clear_end(n, view_width, win_bg_attr(wp), attr);
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            grid_line_mirror(view_width);
        }
        grid_line_flush();
        row += 1;
    }
}
pub unsafe extern "C" fn compute_foldcolumn(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut fdc: ::core::ffi::c_int = win_fdccol_count(wp);
    let mut wmw: ::core::ffi::c_int = if wp == curwin.get() && p_wmw.get() == 0 as OptInt {
        1 as ::core::ffi::c_int
    } else {
        p_wmw.get() as ::core::ffi::c_int
    };
    let mut n: ::core::ffi::c_int = (*wp).w_view_width - (col + wmw);
    return if fdc < n { fdc } else { n };
}
pub unsafe extern "C" fn number_width(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = 0;
    if (*wp).w_onebuf_opt.wo_rnu != 0 && (*wp).w_onebuf_opt.wo_nu == 0 {
        lnum = (*wp).w_view_height as linenr_T;
    } else {
        lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
    }
    if lnum == (*wp).w_nrwidth_line_count {
        return (*wp).w_nrwidth_width;
    }
    (*wp).w_nrwidth_line_count = lnum;
    if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL {
        (*wp).w_statuscol_line_count = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_nrwidth_width = ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
            as ::core::ffi::c_int
            * (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int;
        return (*wp).w_nrwidth_width;
    }
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        lnum = (lnum as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as linenr_T;
        n += 1;
        if lnum <= 0 as linenr_T {
            break;
        }
    }
    n = if n > (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        n
    } else {
        (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    if n < 2 as ::core::ffi::c_int
        && buf_meta_total((*wp).w_buffer, kMTMetaSignText) != 0
        && (*wp).w_minscwidth == SCL_NUM
    {
        n = 2 as ::core::ffi::c_int;
    }
    (*wp).w_nrwidth_width = n;
    return n;
}
pub unsafe extern "C" fn conceal_cursor_line(mut wp: *const win_T) -> bool {
    let mut c: ::core::ffi::c_int = 0;
    if *(*wp).w_onebuf_opt.wo_cocu as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if get_real_state() & MODE_VISUAL != 0 {
        c = 'v' as ::core::ffi::c_int;
    } else if State.get() & MODE_INSERT != 0 {
        c = 'i' as ::core::ffi::c_int;
    } else if State.get() & MODE_NORMAL != 0 {
        c = 'n' as ::core::ffi::c_int;
    } else if State.get() & MODE_CMDLINE != 0 {
        c = 'c' as ::core::ffi::c_int;
    } else {
        return false_0 != 0;
    }
    return !vim_strchr((*wp).w_onebuf_opt.wo_cocu, c).is_null();
}
pub unsafe extern "C" fn win_cursorline_standout(mut wp: *const win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_cul != 0
        || wp == curwin.get() as *const win_T
            && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
            && !conceal_cursor_line(wp);
}
pub unsafe extern "C" fn win_update_cursorline(mut wp: *mut win_T, mut foldinfo: *mut foldinfo_T) {
    (*wp).w_cursorline = if win_cursorline_standout(wp) as ::core::ffi::c_int != 0 {
        (*wp).w_cursor.lnum
    } else {
        0 as linenr_T
    };
    if (*wp).w_onebuf_opt.wo_cul != 0 {
        *foldinfo = fold_info(wp, (*wp).w_cursor.lnum);
        if (*foldinfo).fi_level != 0 as ::core::ffi::c_int && (*foldinfo).fi_lines > 0 as linenr_T {
            (*wp).w_cursorline = (*foldinfo).fi_lnum;
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
