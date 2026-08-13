//! A window's configuration -- which buffer it shows, and where the UI is
//! told it sits.
//!
//! [`win_set_buf`] is the `nvim_win_set_buf()` half: switch to the window,
//! switch its buffer, and switch back, with the autocommands that implies.
//! [`ui_ext_win_position`] and [`ui_ext_win_viewport`] are the outbound half --
//! they tell an external UI where a floating window's grid is anchored and
//! which part of the buffer each window currently shows.
//! [`clear_float_config`] and [`merge_win_config`] normalise a `WinConfig`,
//! and the `check_split_disallowed` pair is the guard every layout change
//! asks first.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, cstr_as_string, find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::buffer::do_buffer;
use crate::src::nvim::decoration::clear_virttext;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::src::nvim::fold::getDeepestNesting;
use crate::src::nvim::grid::{grid_adjust, win_grid_alloc};
use crate::src::nvim::main::{
    Columns, RedrawingDisabled, Rows, curwin, default_grid, float_anchor_str, p_acd, p_ch,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::r#move::{textpos2screenpos, validate_cursor};
use crate::src::nvim::os::libc::{gettext, strncmp};
use crate::src::nvim::plines::win_text_height;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    Boolean, Error, Float, FloatAnchor, Integer, ScreenGrid, String_0, TryState, VirtText,
    VirtTextChunk, WinConfig, WinStyle, Window, buf_T, colnr_T, except_T, handle_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kFloatAnchorEast, kFloatAnchorSouth, kFloatRelativeEditor,
    kFloatRelativeLaststatus, kFloatRelativeTabline, kFloatRelativeWindow, linenr_T, lpos_T,
    msglist_T, pos_T, size_t, switchwin_T, tabpage_T, win_T,
};
use crate::src::nvim::ui::{
    ui_call_win_external_pos, ui_call_win_float_pos, ui_call_win_hide, ui_call_win_pos,
    ui_call_win_viewport, ui_check_cursor_grid, ui_has,
};
use crate::src::nvim::ui_compositor::{
    ui_comp_layers_adjust, ui_comp_put_grid, ui_comp_remove_grid,
};

pub unsafe extern "C" fn win_set_buf(
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut err: *mut Error,
) {
    unsafe {
        let win_handle: handle_T = (*win).handle;
        let mut tab: *mut tabpage_T = win_find_tabpage(win);
        (*RedrawingDisabled.ptr()) += 1;
        let mut switchwin: switchwin_T = switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        };
        let mut win_result: ::core::ffi::c_int = 0;
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        win_result = switch_win_noblock(&raw mut switchwin, win, tab, true);
        if win_result != 0 as ::core::ffi::c_int {
            let save_acd: ::core::ffi::c_int = p_acd.get();
            if !switchwin.sw_same_win {
                p_acd.set(0 as ::core::ffi::c_int);
            }
            do_buffer(
                DOBUF_GOTO as ::core::ffi::c_int,
                DOBUF_FIRST as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                (*buf).handle as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if !switchwin.sw_same_win {
                p_acd.set(save_acd);
            }
        }
        try_leave(&raw mut tstate, err);
        if win_result == FAIL
            && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to switch to window %d".as_ptr(),
                win_handle,
            );
        }
        validate_cursor(curwin.get());
        restore_win_noblock(&raw mut switchwin, true_0 != 0);
        (*RedrawingDisabled.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn win_fdccol_count(mut wp: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        let mut fdc: *const ::core::ffi::c_char = (*wp).w_onebuf_opt.wo_fdc;
        if strncmp(fdc, c"auto".as_ptr(), 4 as size_t) == 0 as ::core::ffi::c_int {
            let fdccol: ::core::ffi::c_int = if *fdc.offset(4 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                *fdc.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    - '0' as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
            let mut needed_fdccols: ::core::ffi::c_int = getDeepestNesting(wp);
            return if fdccol < needed_fdccols {
                fdccol
            } else {
                needed_fdccols
            };
        }
        return *fdc.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            - '0' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn merge_win_config(mut dst: *mut WinConfig, src: WinConfig) {
    unsafe {
        if (*dst).title_chunks.items != src.title_chunks.items {
            clear_virttext(&raw mut (*dst).title_chunks);
        }
        if (*dst).footer_chunks.items != src.footer_chunks.items {
            clear_virttext(&raw mut (*dst).footer_chunks);
        }
        *dst = src;
    }
}

pub unsafe extern "C" fn clear_float_config(mut fconfig: *mut WinConfig, mut free_fields: bool) {
    unsafe {
        let mut saved_style: WinStyle = (*fconfig).style;
        let mut saved_cmdline_offset: ::core::ffi::c_int = (*fconfig)._cmdline_offset;
        if free_fields {
            merge_win_config(
                fconfig,
                WinConfig {
                    window: 0,
                    bufpos: lpos_T {
                        lnum: -1 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    height: 0 as ::core::ffi::c_int,
                    width: 0 as ::core::ffi::c_int,
                    row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                    col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                    anchor: 0 as FloatAnchor,
                    relative: kFloatRelativeEditor,
                    external: false_0 != 0,
                    focusable: true_0 != 0,
                    mouse: true_0 != 0,
                    split: kWinSplitLeft,
                    zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                    style: kWinStyleUnused,
                    border: false,
                    shadow: false,
                    border_chars: [[0; 32]; 8],
                    border_hl_ids: [0; 8],
                    border_attr: [0; 8],
                    title: false,
                    title_pos: kAlignLeft,
                    title_chunks: VirtText {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<VirtTextChunk>(),
                    },
                    title_width: 0,
                    footer: false,
                    footer_pos: kAlignLeft,
                    footer_chunks: VirtText {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<VirtTextChunk>(),
                    },
                    footer_width: 0,
                    noautocmd: false_0 != 0,
                    fixed: false_0 != 0,
                    hide: false_0 != 0,
                    _cmdline_offset: INT_MAX,
                },
            );
        } else {
            *fconfig = WinConfig {
                window: 0,
                bufpos: lpos_T {
                    lnum: -1 as linenr_T,
                    col: 0 as colnr_T,
                },
                height: 0 as ::core::ffi::c_int,
                width: 0 as ::core::ffi::c_int,
                row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                anchor: 0 as FloatAnchor,
                relative: kFloatRelativeEditor,
                external: false_0 != 0,
                focusable: true_0 != 0,
                mouse: true_0 != 0,
                split: kWinSplitLeft,
                zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                style: kWinStyleUnused,
                border: false,
                shadow: false,
                border_chars: [[0; 32]; 8],
                border_hl_ids: [0; 8],
                border_attr: [0; 8],
                title: false,
                title_pos: kAlignLeft,
                title_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                title_width: 0,
                footer: false,
                footer_pos: kAlignLeft,
                footer_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                footer_width: 0,
                noautocmd: false_0 != 0,
                fixed: false_0 != 0,
                hide: false_0 != 0,
                _cmdline_offset: INT_MAX,
            };
        }
        (*fconfig).style = saved_style;
        (*fconfig)._cmdline_offset = saved_cmdline_offset;
    }
}

pub unsafe extern "C" fn ui_ext_win_position(mut wp: *mut win_T, mut validate: bool) {
    unsafe {
        (*wp).w_pos_changed = false_0 != 0;
        if !(*wp).w_floating {
            if ui_has(kUIMultigrid) {
                (*wp).w_grid_alloc.comp_col = (*wp).w_wincol;
                (*wp).w_grid_alloc.comp_row = (*wp).w_winrow;
            }
            ui_call_win_pos(
                (*wp).w_grid_alloc.handle as Integer,
                (*wp).handle as Window,
                (*wp).w_winrow as Integer,
                (*wp).w_wincol as Integer,
                (*wp).w_width as Integer,
                (*wp).w_height as Integer,
            );
            return;
        }
        let mut c: WinConfig = (*wp).w_config;
        if !c.external {
            let mut grid: *mut ScreenGrid = default_grid.ptr();
            let mut row: Float = c.row as Float;
            let mut col: Float = c.col as Float;
            if c.relative as ::core::ffi::c_uint
                == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut dummy: Error = Error {
                    type_0: kErrorTypeNone,
                    msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
                let mut win: *mut win_T = find_window_by_handle(c.window, &raw mut dummy);
                api_clear_error(&raw mut dummy);
                if !win.is_null() {
                    if (*win).w_pos_changed as ::core::ffi::c_int != 0
                        && !(*win).w_grid_alloc.chars.is_null()
                        && win_valid(win) as ::core::ffi::c_int != 0
                    {
                        ui_ext_win_position(win, validate);
                    }
                    let mut row_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut col_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    win_grid_alloc(win);
                    grid = grid_adjust(&raw mut (*win).w_grid, &raw mut row_off, &raw mut col_off);
                    row += row_off as Float;
                    col += col_off as Float;
                    if c.bufpos.lnum >= 0 as linenr_T {
                        let mut lnum: ::core::ffi::c_int = if (c.bufpos.lnum + 1 as linenr_T)
                            < (*(*win).w_buffer).b_ml.ml_line_count
                        {
                            c.bufpos.lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        } else {
                            (*(*win).w_buffer).b_ml.ml_line_count as ::core::ffi::c_int
                        };
                        let mut pos: pos_T = pos_T {
                            lnum: lnum as linenr_T,
                            col: c.bufpos.col,
                            coladd: 0 as colnr_T,
                        };
                        let mut trow: ::core::ffi::c_int = 0;
                        let mut tcol: ::core::ffi::c_int = 0;
                        let mut tcolc: ::core::ffi::c_int = 0;
                        let mut tcole: ::core::ffi::c_int = 0;
                        textpos2screenpos(
                            win,
                            &raw mut pos,
                            &raw mut trow,
                            &raw mut tcol,
                            &raw mut tcolc,
                            &raw mut tcole,
                            true_0 != 0,
                        );
                        row += (trow - 1 as ::core::ffi::c_int) as Float;
                        col += (tcol - 1 as ::core::ffi::c_int) as Float;
                    }
                }
            } else if c.relative as ::core::ffi::c_uint
                == kFloatRelativeLaststatus as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                row += (Rows.get()
                    - p_ch.get() as ::core::ffi::c_int
                    - last_stl_height(false_0 != 0)) as Float;
            } else if c.relative as ::core::ffi::c_uint
                == kFloatRelativeTabline as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                row += tabline_height() as Float;
            }
            let mut resort: bool = (*wp).w_grid_alloc.comp_index != 0 as size_t
                && (*wp).w_grid_alloc.zindex != (*wp).w_config.zindex;
            let mut raise: bool = resort as ::core::ffi::c_int != 0
                && (*wp).w_grid_alloc.zindex < (*wp).w_config.zindex;
            (*wp).w_grid_alloc.zindex = (*wp).w_config.zindex;
            if resort {
                ui_comp_layers_adjust((*wp).w_grid_alloc.comp_index, raise);
            }
            let mut valid: bool = (*wp).w_redr_type == 0 as ::core::ffi::c_int
                || ui_has(kUIMultigrid) as ::core::ffi::c_int != 0;
            if !valid && !validate {
                (*wp).w_pos_changed = true_0 != 0;
                return;
            }
            let mut east: bool =
                c.anchor as ::core::ffi::c_int & kFloatAnchorEast as ::core::ffi::c_int != 0;
            let mut south: bool =
                c.anchor as ::core::ffi::c_int & kFloatAnchorSouth as ::core::ffi::c_int != 0;
            let mut comp_row: ::core::ffi::c_int = row as ::core::ffi::c_int
                - (if south as ::core::ffi::c_int != 0 {
                    (*wp).w_height_outer
                } else {
                    0 as ::core::ffi::c_int
                });
            let mut comp_col_0: ::core::ffi::c_int = col as ::core::ffi::c_int
                - (if east as ::core::ffi::c_int != 0 {
                    (*wp).w_width_outer
                } else {
                    0 as ::core::ffi::c_int
                });
            let mut above_ch: ::core::ffi::c_int =
                if (*wp).w_config.zindex < kZIndexMessages as ::core::ffi::c_int {
                    p_ch.get() as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            comp_row += (*grid).comp_row;
            comp_col_0 += (*grid).comp_col;
            comp_row = if (if comp_row < Rows.get() - (*wp).w_height_outer - above_ch {
                comp_row
            } else {
                Rows.get() - (*wp).w_height_outer - above_ch
            }) > 0 as ::core::ffi::c_int
            {
                if comp_row < Rows.get() - (*wp).w_height_outer - above_ch {
                    comp_row
                } else {
                    Rows.get() - (*wp).w_height_outer - above_ch
                }
            } else {
                0 as ::core::ffi::c_int
            };
            if !c.fixed || east as ::core::ffi::c_int != 0 {
                comp_col_0 = if (if comp_col_0 < Columns.get() - (*wp).w_width_outer {
                    comp_col_0
                } else {
                    Columns.get() - (*wp).w_width_outer
                }) > 0 as ::core::ffi::c_int
                {
                    if comp_col_0 < Columns.get() - (*wp).w_width_outer {
                        comp_col_0
                    } else {
                        Columns.get() - (*wp).w_width_outer
                    }
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            (*wp).w_winrow = comp_row;
            (*wp).w_wincol = comp_col_0;
            if !c.hide {
                ui_comp_put_grid(
                    &raw mut (*wp).w_grid_alloc,
                    comp_row,
                    comp_col_0,
                    (*wp).w_height_outer,
                    (*wp).w_width_outer,
                    valid,
                    false_0 != 0,
                );
                if ui_has(kUIMultigrid) {
                    let mut anchor: String_0 = cstr_as_string(
                        *(&raw const float_anchor_str as *const *const ::core::ffi::c_char)
                            .offset(c.anchor as isize),
                    );
                    ui_call_win_float_pos(
                        (*wp).w_grid_alloc.handle as Integer,
                        (*wp).handle as Window,
                        anchor,
                        (*grid).handle as Integer,
                        row,
                        col,
                        c.mouse as Boolean,
                        (*wp).w_grid_alloc.zindex as Integer,
                        (*wp).w_grid_alloc.comp_index as ::core::ffi::c_int as Integer,
                        (*wp).w_winrow as Integer,
                        (*wp).w_wincol as Integer,
                    );
                }
                ui_check_cursor_grid((*wp).w_grid_alloc.handle);
                (*wp).w_grid_alloc.mouse_enabled = (*wp).w_config.mouse;
                if !valid {
                    (*wp).w_grid_alloc.valid = false_0 != 0;
                    redraw_later(wp, UPD_NOT_VALID);
                }
            } else {
                if ui_has(kUIMultigrid) {
                    ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
                }
                ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
            }
        } else {
            ui_call_win_external_pos((*wp).w_grid_alloc.handle as Integer, (*wp).handle as Window);
        };
    }
}

pub unsafe extern "C" fn ui_ext_win_viewport(mut wp: *mut win_T) {
    unsafe {
        if (wp == curwin.get() || ui_has(kUIMultigrid) as ::core::ffi::c_int != 0)
            && (*wp).w_viewport_invalid as ::core::ffi::c_int != 0
            && (*wp).w_redr_type == 0 as ::core::ffi::c_int
        {
            let line_count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
            let cur_topline: linenr_T = if (*wp).w_topline < line_count {
                (*wp).w_topline
            } else {
                line_count
            };
            let cur_botline: linenr_T = if (*wp).w_botline < line_count {
                (*wp).w_botline
            } else {
                line_count
            };
            let mut delta: int64_t = 0 as int64_t;
            let mut last_topline: linenr_T = (*wp).w_viewport_last_topline;
            let mut last_botline: linenr_T = (*wp).w_viewport_last_botline;
            let mut last_topfill: ::core::ffi::c_int =
                (*wp).w_viewport_last_topfill as ::core::ffi::c_int;
            let mut last_skipcol: int64_t = (*wp).w_viewport_last_skipcol as int64_t;
            if last_topline > line_count {
                delta -= (last_topline - line_count) as int64_t;
                last_topline = line_count;
                last_topfill = 0 as ::core::ffi::c_int;
                last_skipcol = MAXCOL as ::core::ffi::c_int as int64_t;
            }
            last_botline = if last_botline < line_count {
                last_botline
            } else {
                line_count
            };
            if cur_topline < last_topline
                || cur_topline == last_topline && ((*wp).w_skipcol as int64_t) < last_skipcol
            {
                let mut vcole: int64_t = last_skipcol;
                let mut lnume: linenr_T = last_topline;
                if last_topline > 0 as linenr_T && cur_botline < last_topline {
                    delta -= (last_topline - cur_botline) as int64_t;
                    lnume = cur_botline;
                    vcole = 0 as int64_t;
                }
                delta -= win_text_height(
                    wp,
                    cur_topline,
                    (*wp).w_skipcol as int64_t,
                    &raw mut lnume,
                    &raw mut vcole,
                    ::core::ptr::null_mut::<int64_t>(),
                    INT64_MAX as int64_t,
                );
            } else if cur_topline > last_topline
                || cur_topline == last_topline && (*wp).w_skipcol as int64_t > last_skipcol
            {
                let mut vcole_0: int64_t = (*wp).w_skipcol as int64_t;
                let mut lnume_0: linenr_T = cur_topline;
                if last_botline > 0 as linenr_T && cur_topline > last_botline {
                    delta += (cur_topline - last_botline) as int64_t;
                    lnume_0 = last_botline;
                    vcole_0 = 0 as int64_t;
                }
                delta += win_text_height(
                    wp,
                    last_topline,
                    last_skipcol,
                    &raw mut lnume_0,
                    &raw mut vcole_0,
                    ::core::ptr::null_mut::<int64_t>(),
                    INT64_MAX as int64_t,
                );
            }
            delta += last_topfill as int64_t;
            delta -= (*wp).w_topfill as int64_t;
            let mut ev_botline: linenr_T = (*wp).w_botline;
            if ev_botline == line_count + 1 as linenr_T
                && (*wp).w_empty_rows == 0 as ::core::ffi::c_int
            {
                ev_botline = line_count;
            }
            ui_call_win_viewport(
                (*wp).w_grid_alloc.handle as Integer,
                (*wp).handle as Window,
                ((*wp).w_topline - 1 as linenr_T) as Integer,
                ev_botline as Integer,
                ((*wp).w_cursor.lnum - 1 as linenr_T) as Integer,
                (*wp).w_cursor.col as Integer,
                line_count as Integer,
                delta as Integer,
            );
            (*wp).w_viewport_invalid = false_0 != 0;
            (*wp).w_viewport_last_topline = (*wp).w_topline;
            (*wp).w_viewport_last_botline = (*wp).w_botline;
            (*wp).w_viewport_last_topfill = (*wp).w_topfill as linenr_T;
            (*wp).w_viewport_last_skipcol = (*wp).w_skipcol as linenr_T;
        }
    }
}

pub unsafe extern "C" fn check_split_disallowed(mut wp: *const win_T) -> ::core::ffi::c_int {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let ok: bool = check_split_disallowed_err(wp, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            emsg(gettext(err.msg));
            api_clear_error(&raw mut err);
        }
        return if ok as ::core::ffi::c_int != 0 {
            OK
        } else {
            FAIL
        };
    }
}

pub unsafe extern "C" fn check_split_disallowed_err(
    mut wp: *const win_T,
    mut err: *mut Error,
) -> bool {
    unsafe {
        if split_disallowed.get() > 0 as ::core::ffi::c_int {
            api_set_error(
                err,
                kErrorTypeException,
                c"E242: Can't split a window while closing another".as_ptr(),
            );
            return false_0 != 0;
        }
        if (*(*wp).w_buffer).b_locked_split != 0 {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                e_cannot_split_window_when_closing_buffer.as_ptr(),
            );
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}
