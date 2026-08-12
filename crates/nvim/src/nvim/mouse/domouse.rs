//! `do_mouse()` -- the Normal/Visual-mode mouse command.
//!
//! One state machine over the button, the modifiers, the click count and
//! `'mousemodel'`, deciding between: starting or extending a Visual selection,
//! setting the cursor, opening the popup menu, pasting the selection, dragging
//! a status line or vertical separator, closing or moving a tab page, folding,
//! and the "which window does this belong to" question that has to be answered
//! before any of them.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::buffer::bt_quickfix;
use crate::src::nvim::charset::vim_iswordc;
use crate::src::nvim::cursor::{coladvance, get_cursor_pos_ptr};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::eval::eval_has_provider;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, tabpage_new};
use crate::src::nvim::fold::{closeFold, openFold};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, safe_vgetc, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff, vpeekc,
    vungetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{
    Ctrl_G, Ctrl_O, Ctrl_P, Ctrl_R, Ctrl_RSB, Ctrl_T, Ctrl_V, KE_MIDDLEMOUSE, KE_MOUSEMOVE,
    get_mouse_button,
};
use crate::src::nvim::main::{
    Columns, KeyStuffed, State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect, VIsual_select,
    cmdwin_type, curbuf, curwin, firstwin, mod_mask, mode_displayed, mouse_col, mouse_dragging,
    mouse_grid, mouse_past_bottom, mouse_past_eol, mouse_row, msg_silent, p_sel, p_smd,
    redraw_cmdline, restart_edit, tab_page_click_defs, where_paste_started,
};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{gchar_pos, inc};
use crate::src::nvim::r#move::scroll_redraw;
use crate::src::nvim::normal::{
    clearop, clearopbeep, end_visual_mode, may_start_select, prep_redo,
};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::plines::getvcols;
use crate::src::nvim::pos::{equalpos, lt};
use crate::src::nvim::register::{do_put, insert_reg, yank_register_mline};
use crate::src::nvim::search::{BACKWARD, FORWARD, findmatch};
use crate::src::nvim::state::{MODE_INSERT, MODE_NORMAL, REPLACE_FLAG};
use crate::src::nvim::types::{
    OP_NOP, PUT_CURSEND, PUT_FIXINDENT, StlClickDefinition, colnr_T, linenr_T, oparg_T, pos_T,
    win_T, yankreg_T,
};
use crate::src::nvim::ui::ui_mouse_has;
use crate::src::nvim::window::{global_stl_height, goto_tabpage, tabpage_move};

pub unsafe extern "C" fn do_mouse(
    mut oap: *mut oparg_T,
    mut c: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut fixindent: bool,
) -> bool {
    unsafe {
        let mut which_button: ::core::ffi::c_int = 0;
        let mut is_click: bool = false;
        let mut is_drag: bool = false;
        static in_tab_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        static orig_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        loop {
            which_button = get_mouse_button(
                (-c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint)
                    as ::core::ffi::c_int,
                &raw mut is_click,
                &raw mut is_drag,
            );
            if !is_drag {
                break;
            }
            if !(KeyStuffed.get() == 0 && vpeekc() != NUL) {
                break;
            }
            let mut nc: ::core::ffi::c_int = 0;
            let mut save_mouse_grid: ::core::ffi::c_int = mouse_grid.get();
            let mut save_mouse_row: ::core::ffi::c_int = mouse_row.get();
            let mut save_mouse_col: ::core::ffi::c_int = mouse_col.get();
            nc = safe_vgetc();
            if c == nc {
                continue;
            }
            vungetc(nc);
            mouse_grid.set(save_mouse_grid);
            mouse_row.set(save_mouse_row);
            mouse_col.set(save_mouse_col);
            break;
        }
        if c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            return false_0 != 0;
        }
        if is_click {
            got_click.set(true_0 != 0);
        } else {
            if !got_click.get() {
                return false_0 != 0;
            }
            if !is_drag {
                got_click.set(false_0 != 0);
                if in_tab_line.get() {
                    in_tab_line.set(false_0 != 0);
                    return false_0 != 0;
                }
            }
        }
        if is_click as ::core::ffi::c_int != 0
            && mod_mask.get() & MOD_MASK_CTRL != 0
            && which_button == MOUSE_RIGHT as ::core::ffi::c_int
        {
            if State.get() & MODE_INSERT != 0 {
                stuffcharReadbuff(Ctrl_O);
            }
            if count > 1 as ::core::ffi::c_int {
                stuffnumReadbuff(count);
            }
            stuffcharReadbuff(Ctrl_T);
            got_click.set(false_0 != 0);
            return false_0 != 0;
        }
        if mod_mask.get() & MOD_MASK_CTRL != 0 && which_button != MOUSE_LEFT as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT | MOD_MASK_META) != 0
            && (!is_click
                || mod_mask.get() & MOD_MASK_MULTI_CLICK != 0
                || which_button == MOUSE_MIDDLE as ::core::ffi::c_int)
            && !(mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0
                && mouse_model_popup() as ::core::ffi::c_int != 0
                && which_button == MOUSE_LEFT as ::core::ffi::c_int)
            && !(mod_mask.get() & MOD_MASK_ALT != 0
                && !mouse_model_popup()
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int)
        {
            return false_0 != 0;
        }
        if !is_click && which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut regname: ::core::ffi::c_int = if !oap.is_null() {
            (*oap).regname
        } else {
            0 as ::core::ffi::c_int
        };
        if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
            if State.get() == MODE_NORMAL {
                if !oap.is_null() && (*oap).op_type != OP_NOP {
                    clearopbeep(oap);
                    return false_0 != 0;
                }
                if VIsual_active.get() {
                    if VIsual_select.get() {
                        stuffcharReadbuff(Ctrl_G);
                        stuffReadbuff(c"\"+p".as_ptr());
                    } else {
                        stuffcharReadbuff('y' as ::core::ffi::c_int);
                        stuffcharReadbuff(
                            -(253 as ::core::ffi::c_int
                                + ((KE_MIDDLEMOUSE as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int)),
                        );
                    }
                    return false_0 != 0;
                }
            } else if State.get() & MODE_INSERT == 0 as ::core::ffi::c_int {
                return false_0 != 0;
            }
            if State.get() & MODE_INSERT != 0 {
                if regname == '.' as ::core::ffi::c_int {
                    insert_reg(regname, ::core::ptr::null_mut::<yankreg_T>(), true_0 != 0);
                } else {
                    if regname == 0 as ::core::ffi::c_int
                        && eval_has_provider(c"clipboard".as_ptr(), false_0 != 0)
                            as ::core::ffi::c_int
                            != 0
                    {
                        regname = '*' as ::core::ffi::c_int;
                    }
                    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
                    if State.get() & REPLACE_FLAG != 0
                        && !yank_register_mline(regname, &raw mut reg)
                    {
                        insert_reg(regname, reg, true_0 != 0);
                    } else {
                        do_put(
                            regname,
                            reg,
                            BACKWARD as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                            (if fixindent as ::core::ffi::c_int != 0 {
                                PUT_FIXINDENT as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            }) | PUT_CURSEND as ::core::ffi::c_int,
                        );
                        AppendCharToRedobuff(Ctrl_R);
                        AppendCharToRedobuff(if fixindent as ::core::ffi::c_int != 0 {
                            Ctrl_P
                        } else {
                            Ctrl_O
                        });
                        AppendCharToRedobuff(if regname == 0 as ::core::ffi::c_int {
                            '"' as ::core::ffi::c_int
                        } else {
                            regname
                        });
                    }
                }
                return false_0 != 0;
            }
        }
        let mut jump_flags: ::core::ffi::c_int = if is_click as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int
        };
        let mut old_curwin: *mut win_T = curwin.get();
        if !(*tab_page_click_defs.ptr()).is_null() {
            if mouse_grid.get() <= 1 as ::core::ffi::c_int
                && mouse_row.get() == 0 as ::core::ffi::c_int
                && (*firstwin.get()).w_winrow > 0 as ::core::ffi::c_int
            {
                if is_drag {
                    if in_tab_line.get() {
                        move_tab_to_mouse();
                    }
                    return false_0 != 0;
                }
                if is_click as ::core::ffi::c_int != 0
                    && cmdwin_type.get() == 0 as ::core::ffi::c_int
                    && mouse_col.get() < Columns.get()
                {
                    let mut tabnr: ::core::ffi::c_int =
                        (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize)).tabnr;
                    in_tab_line.set(true_0 != 0);
                    's_464: {
                        match (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize))
                            .type_0 as ::core::ffi::c_uint
                        {
                            1 => {
                                if which_button != MOUSE_MIDDLE as ::core::ffi::c_int {
                                    if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                                        end_visual_mode();
                                        tabpage_new();
                                        tabpage_move(if tabnr == 0 as ::core::ffi::c_int {
                                            9999 as ::core::ffi::c_int
                                        } else {
                                            tabnr - 1 as ::core::ffi::c_int
                                        });
                                    } else {
                                        goto_tabpage(tabnr);
                                        if curwin.get() != old_curwin {
                                            end_visual_mode();
                                        }
                                    }
                                    break 's_464;
                                }
                            }
                            2 => {}
                            3 => {
                                call_click_def_func(
                                    tab_page_click_defs.get(),
                                    mouse_col.get(),
                                    which_button,
                                );
                                break 's_464;
                            }
                            0 | _ => {
                                break 's_464;
                            }
                        }
                        mouse_tab_close(tabnr);
                    }
                }
                return true_0 != 0;
            } else if is_drag as ::core::ffi::c_int != 0
                && in_tab_line.get() as ::core::ffi::c_int != 0
            {
                move_tab_to_mouse();
                return false_0 != 0;
            }
        }
        let mut m_pos_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut m_pos: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0,
            coladd: 0,
        };
        if mouse_model_popup() {
            m_pos_flag = get_fpos_of_mouse(&raw mut m_pos);
            if m_pos_flag
                & (IN_STATUS_LINE as ::core::ffi::c_int
                    | MOUSE_WINBAR as ::core::ffi::c_int
                    | MOUSE_STATUSCOL as ::core::ffi::c_int)
                == 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
            {
                if !is_click {
                    return false_0 != 0;
                }
                return do_popup(which_button, m_pos_flag, m_pos)
                    & CURSOR_MOVED as ::core::ffi::c_int
                    != 0;
            }
            if m_pos_flag
                & (IN_STATUS_LINE as ::core::ffi::c_int
                    | MOUSE_WINBAR as ::core::ffi::c_int
                    | MOUSE_STATUSCOL as ::core::ffi::c_int)
                == 0
                && (which_button == MOUSE_LEFT as ::core::ffi::c_int
                    && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0)
            {
                which_button = MOUSE_RIGHT as ::core::ffi::c_int;
                (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
            }
        }
        let mut end_visual: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0,
            coladd: 0,
        };
        let mut start_visual: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0,
            coladd: 0,
        };
        let mut mouse_can_visual: bool = ui_mouse_has(MOUSE_VISUAL);
        if State.get() & (MODE_NORMAL | MODE_INSERT) != 0
            && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
        {
            if which_button == MOUSE_LEFT as ::core::ffi::c_int
                && mouse_can_visual as ::core::ffi::c_int != 0
            {
                if is_click {
                    if VIsual_active.get() {
                        jump_flags |= MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                    }
                } else {
                    jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
                }
            } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int
                && mouse_can_visual as ::core::ffi::c_int != 0
            {
                if is_click as ::core::ffi::c_int != 0
                    && VIsual_active.get() as ::core::ffi::c_int != 0
                {
                    if lt((*curwin.get()).w_cursor, VIsual.get()) {
                        start_visual = (*curwin.get()).w_cursor;
                        end_visual = VIsual.get();
                    } else {
                        start_visual = VIsual.get();
                        end_visual = (*curwin.get()).w_cursor;
                    }
                }
                jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
                jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
            } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
                jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
            }
        }
        if !is_drag && !oap.is_null() && (*oap).op_type != OP_NOP {
            got_click.set(false_0 != 0);
            (*oap).motion_type = kMTCharWise;
        }
        if !is_click && !is_drag {
            jump_flags |= MOUSE_RELEASED as ::core::ffi::c_int;
        }
        let mut old_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        if !VIsual_active.get() || mouse_can_visual as ::core::ffi::c_int != 0 {
            jump_flags = jump_to_mouse(
                jump_flags,
                if oap.is_null() {
                    ::core::ptr::null_mut::<bool>()
                } else {
                    &raw mut (*oap).inclusive
                },
                which_button,
            );
        }
        let mut moved: bool = jump_flags & CURSOR_MOVED as ::core::ffi::c_int != 0;
        let mut in_winbar: bool = jump_flags & MOUSE_WINBAR as ::core::ffi::c_int != 0;
        let mut in_statuscol: bool = jump_flags & MOUSE_STATUSCOL as ::core::ffi::c_int != 0;
        let mut in_status_line: bool = jump_flags & IN_STATUS_LINE as ::core::ffi::c_int != 0;
        let mut in_global_statusline: bool = in_status_line as ::core::ffi::c_int != 0
            && global_stl_height() > 0 as ::core::ffi::c_int;
        let mut in_sep_line: bool = jump_flags & IN_SEP_LINE as ::core::ffi::c_int != 0;
        if (in_winbar as ::core::ffi::c_int != 0
            || in_status_line as ::core::ffi::c_int != 0
            || in_statuscol as ::core::ffi::c_int != 0)
            && is_click as ::core::ffi::c_int != 0
        {
            let mut click_grid: ::core::ffi::c_int = mouse_grid.get();
            let mut click_row: ::core::ffi::c_int = mouse_row.get();
            let mut click_col: ::core::ffi::c_int = mouse_col.get();
            let mut wp: *mut win_T =
                mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col);
            if wp.is_null() {
                return false_0 != 0;
            }
            let mut click_defs: *mut StlClickDefinition =
                if in_status_line as ::core::ffi::c_int != 0 {
                    (*wp).w_status_click_defs
                } else if in_winbar as ::core::ffi::c_int != 0 {
                    (*wp).w_winbar_click_defs
                } else {
                    (*wp).w_statuscol_click_defs
                };
            if in_global_statusline {
                click_defs = (*curwin.get()).w_status_click_defs;
                click_col = mouse_col.get();
            }
            if in_statuscol as ::core::ffi::c_int != 0 && (*wp).w_onebuf_opt.wo_rl != 0 {
                click_col = (*wp).w_view_width - click_col - 1 as ::core::ffi::c_int;
            }
            if in_statuscol as ::core::ffi::c_int != 0
                && click_col >= (*wp).w_statuscol_click_defs_size as ::core::ffi::c_int
                || in_status_line as ::core::ffi::c_int != 0
                    && click_col
                        >= (*(if in_global_statusline as ::core::ffi::c_int != 0 {
                            curwin.get()
                        } else {
                            wp
                        }))
                        .w_status_click_defs_size as ::core::ffi::c_int
            {
                return false_0 != 0;
            }
            if !click_defs.is_null() {
                match (*click_defs.offset(click_col as isize)).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if in_statuscol as ::core::ffi::c_int != 0
                            && mouse_model_popup() as ::core::ffi::c_int != 0
                            && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                            && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
                        {
                            do_popup(which_button, m_pos_flag, m_pos);
                        }
                    }
                    3 => {
                        call_click_def_func(click_defs, click_col, which_button);
                    }
                    _ => {
                        debug_assert!(
                            false,
                            "false && \\\"winbar, statusline and statuscolumn only support %@ for clicks\\\""
                        );
                    }
                }
            }
            if !(in_statuscol as ::core::ffi::c_int != 0
                && jump_flags
                    & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int
                        | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
                    != 0)
            {
                return false_0 != 0;
            }
        } else if in_winbar as ::core::ffi::c_int != 0 || in_statuscol as ::core::ffi::c_int != 0 {
            return false_0 != 0;
        }
        if curwin.get() != old_curwin && !oap.is_null() && (*oap).op_type != OP_NOP {
            clearop(oap);
        }
        if mod_mask.get() == 0 as ::core::ffi::c_int
            && !is_drag
            && jump_flags
                & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
                != 0
            && which_button == MOUSE_LEFT as ::core::ffi::c_int
        {
            if jump_flags & MOUSE_FOLD_OPEN as ::core::ffi::c_int != 0 {
                openFold((*curwin.get()).w_cursor, 1 as ::core::ffi::c_int);
            } else {
                closeFold((*curwin.get()).w_cursor, 1 as ::core::ffi::c_int);
            }
            if curwin.get() == old_curwin {
                (*curwin.get()).w_cursor = save_cursor;
            }
        }
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && is_drag as ::core::ffi::c_int != 0
            && get_scrolloff_value(curwin.get()) != 0
        {
            if mouse_row.get() == 0 as ::core::ffi::c_int {
                mouse_dragging.set(2 as ::core::ffi::c_int);
            } else {
                mouse_dragging.set(1 as ::core::ffi::c_int);
            }
        }
        if is_drag as ::core::ffi::c_int != 0
            && mouse_row.get() < 0 as ::core::ffi::c_int
            && !in_status_line
        {
            scroll_redraw(false_0, 1 as linenr_T);
            mouse_row.set(0 as ::core::ffi::c_int);
        }
        let mut old_mode: ::core::ffi::c_int = VIsual_mode.get();
        if start_visual.lnum != 0 {
            let mut diff: linenr_T = 0;
            if mod_mask.get() & MOD_MASK_ALT != 0 {
                VIsual_mode.set(Ctrl_V);
            }
            if VIsual_mode.get() == Ctrl_V {
                let mut leftcol: colnr_T = 0;
                let mut rightcol: colnr_T = 0;
                getvcols(
                    curwin.get(),
                    &raw mut start_visual,
                    &raw mut end_visual,
                    &raw mut leftcol,
                    &raw mut rightcol,
                );
                if (*curwin.get()).w_curswant
                    > (leftcol as ::core::ffi::c_int + rightcol as ::core::ffi::c_int)
                        / 2 as ::core::ffi::c_int
                {
                    end_visual.col = leftcol;
                } else {
                    end_visual.col = rightcol;
                }
                if (*curwin.get()).w_cursor.lnum
                    >= (start_visual.lnum + end_visual.lnum) / 2 as linenr_T
                {
                    end_visual.lnum = start_visual.lnum;
                }
                start_visual = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = end_visual;
                coladvance(curwin.get(), end_visual.col);
                VIsual.set((*curwin.get()).w_cursor);
                (*curwin.get()).w_cursor = start_visual;
            } else if lt((*curwin.get()).w_cursor, start_visual) {
                VIsual.set(end_visual);
            } else if lt(end_visual, (*curwin.get()).w_cursor) {
                VIsual.set(start_visual);
            } else if end_visual.lnum == start_visual.lnum {
                if (*curwin.get()).w_cursor.col - start_visual.col
                    > end_visual.col - (*curwin.get()).w_cursor.col
                {
                    VIsual.set(start_visual);
                } else {
                    VIsual.set(end_visual);
                }
            } else {
                diff = (*curwin.get()).w_cursor.lnum
                    - start_visual.lnum
                    - (end_visual.lnum - (*curwin.get()).w_cursor.lnum);
                if diff > 0 as linenr_T {
                    VIsual.set(start_visual);
                } else if diff < 0 as linenr_T {
                    VIsual.set(end_visual);
                } else if (*curwin.get()).w_cursor.col
                    < (start_visual.col as ::core::ffi::c_int
                        + end_visual.col as ::core::ffi::c_int)
                        / 2 as ::core::ffi::c_int
                {
                    VIsual.set(end_visual);
                } else {
                    VIsual.set(start_visual);
                }
            }
        } else if State.get() & MODE_INSERT != 0 && VIsual_active.get() as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(Ctrl_O);
        }
        if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
            if regname == 0 as ::core::ffi::c_int
                && eval_has_provider(c"clipboard".as_ptr(), false_0 != 0) as ::core::ffi::c_int != 0
            {
                regname = '*' as ::core::ffi::c_int;
            }
            let mut reg_0: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
            if yank_register_mline(regname, &raw mut reg_0) {
                if mouse_past_bottom.get() {
                    dir = FORWARD as ::core::ffi::c_int;
                }
            } else if mouse_past_eol.get() {
                dir = FORWARD as ::core::ffi::c_int;
            }
            let mut c1: ::core::ffi::c_int = 0;
            let mut c2: ::core::ffi::c_int = 0;
            if fixindent {
                c1 = if dir == BACKWARD as ::core::ffi::c_int {
                    '[' as ::core::ffi::c_int
                } else {
                    ']' as ::core::ffi::c_int
                };
                c2 = 'p' as ::core::ffi::c_int;
            } else {
                c1 = if dir == FORWARD as ::core::ffi::c_int {
                    'p' as ::core::ffi::c_int
                } else {
                    'P' as ::core::ffi::c_int
                };
                c2 = NUL;
            }
            prep_redo(regname, count, NUL, c1, NUL, c2, NUL);
            if restart_edit.get() != 0 as ::core::ffi::c_int {
                where_paste_started.set((*curwin.get()).w_cursor);
            }
            do_put(
                regname,
                reg_0,
                dir,
                count,
                (if fixindent as ::core::ffi::c_int != 0 {
                    PUT_FIXINDENT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | PUT_CURSEND as ::core::ffi::c_int,
            );
        } else if (mod_mask.get() & MOD_MASK_CTRL != 0
            || mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK)
            && bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
        {
            if (*curwin.get()).w_llist_ref.is_null() {
                do_cmdline_cmd(c".cc".as_ptr());
            } else {
                do_cmdline_cmd(c".ll".as_ptr());
            }
            got_click.set(false_0 != 0);
        } else if mod_mask.get() & MOD_MASK_CTRL != 0
            || (*curbuf.get()).b_help as ::core::ffi::c_int != 0
                && mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK
        {
            if State.get() & MODE_INSERT != 0 {
                stuffcharReadbuff(Ctrl_O);
            }
            stuffcharReadbuff(Ctrl_RSB);
            got_click.set(false_0 != 0);
        } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
            if State.get() & MODE_INSERT != 0
                || VIsual_active.get() as ::core::ffi::c_int != 0
                    && VIsual_select.get() as ::core::ffi::c_int != 0
            {
                stuffcharReadbuff(Ctrl_O);
            }
            if which_button == MOUSE_LEFT as ::core::ffi::c_int {
                stuffcharReadbuff('*' as ::core::ffi::c_int);
            } else {
                stuffcharReadbuff('#' as ::core::ffi::c_int);
            }
        } else if !(in_status_line as ::core::ffi::c_int != 0
            || in_sep_line as ::core::ffi::c_int != 0)
        {
            if mod_mask.get() & MOD_MASK_MULTI_CLICK != 0
                && State.get() & (MODE_NORMAL | MODE_INSERT) != 0
                && mouse_can_visual as ::core::ffi::c_int != 0
            {
                if is_click as ::core::ffi::c_int != 0 || !VIsual_active.get() {
                    if VIsual_active.get() {
                        orig_cursor.set(VIsual.get());
                    } else {
                        VIsual.set((*curwin.get()).w_cursor);
                        orig_cursor.set(VIsual.get());
                        VIsual_active.set(true_0 != 0);
                        VIsual_reselect.set(true_0);
                        may_start_select('o' as ::core::ffi::c_int);
                        setmouse();
                    }
                    if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                        if mod_mask.get() & MOD_MASK_ALT != 0 {
                            VIsual_mode.set(Ctrl_V);
                        } else {
                            VIsual_mode.set('v' as ::core::ffi::c_int);
                        }
                    } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                        VIsual_mode.set('V' as ::core::ffi::c_int);
                    } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                        VIsual_mode.set(Ctrl_V);
                    }
                }
                if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
                    if is_click {
                        end_visual = (*curwin.get()).w_cursor;
                        let mut gc: ::core::ffi::c_int = 0;
                        loop {
                            gc = gchar_pos(&raw mut end_visual);
                            if !ascii_iswhite(gc) {
                                break;
                            }
                            inc(&raw mut end_visual);
                        }
                        if !oap.is_null() {
                            (*oap).motion_type = kMTCharWise;
                        }
                        if !oap.is_null()
                            && VIsual_mode.get() == 'v' as ::core::ffi::c_int
                            && !vim_iswordc(gchar_pos(&raw mut end_visual))
                            && equalpos((*curwin.get()).w_cursor, VIsual.get())
                                as ::core::ffi::c_int
                                != 0
                            && {
                                pos = findmatch(oap, NUL);
                                !pos.is_null()
                            }
                        {
                            (*curwin.get()).w_cursor = *pos;
                            if (*oap).motion_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                            {
                                VIsual_mode.set('V' as ::core::ffi::c_int);
                            } else if *p_sel.get() as ::core::ffi::c_int
                                == 'e' as ::core::ffi::c_int
                            {
                                if lt((*curwin.get()).w_cursor, VIsual.get()) {
                                    (*VIsual.ptr()).col += 1;
                                } else {
                                    (*curwin.get()).w_cursor.col += 1;
                                }
                            }
                        }
                    }
                    if pos.is_null()
                        && (is_click as ::core::ffi::c_int != 0
                            || is_drag as ::core::ffi::c_int != 0)
                    {
                        if lt((*curwin.get()).w_cursor, orig_cursor.get()) {
                            find_start_of_word(&raw mut (*curwin.get()).w_cursor);
                            find_end_of_word(VIsual.ptr());
                        } else {
                            find_start_of_word(VIsual.ptr());
                            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                                && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL
                            {
                                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                            }
                            find_end_of_word(&raw mut (*curwin.get()).w_cursor);
                        }
                    }
                    (*curwin.get()).w_set_curswant = true_0;
                }
                if is_click {
                    redraw_curbuf_later(UPD_INVERTED);
                }
            } else if VIsual_active.get() as ::core::ffi::c_int != 0 && old_active == 0 {
                if mod_mask.get() & MOD_MASK_ALT != 0 {
                    VIsual_mode.set(Ctrl_V);
                } else {
                    VIsual_mode.set('v' as ::core::ffi::c_int);
                }
            }
        }
        if !VIsual_active.get()
            && old_active != 0
            && mode_displayed.get() as ::core::ffi::c_int != 0
            || VIsual_active.get() as ::core::ffi::c_int != 0
                && p_smd.get() != 0
                && msg_silent.get() == 0 as ::core::ffi::c_int
                && (old_active == 0 || VIsual_mode.get() != old_mode)
        {
            redraw_cmdline.set(true_0 != 0);
        }
        return moved;
    }
}
