//! Running a menu entry -- `:emenu`, `:popup` and the tooltip lookup.
//!
//! [`execute_menu`] is the one that decides which mode's right-hand side to
//! use: the mode the editor is really in, unless the command named one, and
//! with a special case for a menu invoked from a script.  The rhs is then fed
//! back into the typeahead as if the user had typed it.  [`ex_emenu`] parses
//! the command's argument, [`menu_getbyname`] and [`menu_find`] resolve a path
//! for it and for `:popup`.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::cursor::{check_cursor, gchar_cursor};
use crate::src::nvim::ex_docmd::{exec_normal_cmd, restore_current_state, save_current_state};
use crate::src::nvim::getchar::ins_typebuf;
use crate::src::nvim::main::{
    State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect, curbuf, current_sctx, curwin,
    e_invarg2, ex_normal_busy, p_sel, restart_edit,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_TERMINAL, MODE_VISUAL, get_real_state,
};
use crate::src::nvim::types::{
    String_0, buffblock, buffblock_T, buffheader_T, colnr_T, exarg_T, pos_T, save_state_T,
    tasave_T, typebuf_T, uint8_t, vimmenu_T,
};

pub unsafe extern "C" fn execute_menu(
    mut eap: *const exarg_T,
    mut menu: *mut vimmenu_T,
    mut mode_idx: ::core::ffi::c_int,
) {
    unsafe {
        let mut idx: ::core::ffi::c_int = mode_idx;
        if idx < 0 as ::core::ffi::c_int {
            if State.get() & MODE_TERMINAL != 0 {
                idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
            } else if State.get() & MODE_CMDLINE != 0 {
                idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
            } else if get_real_state() & MODE_VISUAL != 0 {
                idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
            } else if (State.get() & MODE_INSERT != 0 || restart_edit.get() != 0)
                && (*current_sctx.ptr()).sc_sid == 0 as ::core::ffi::c_int
            {
                idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
            } else if !eap.is_null() && (*eap).addr_count != 0 {
                let mut tpos: pos_T = pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                };
                idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
                if (*curbuf.get()).b_visual.vi_start.lnum == (*eap).line1
                    && (*curbuf.get()).b_visual.vi_end.lnum == (*eap).line2
                {
                    VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
                    tpos = (*curbuf.get()).b_visual.vi_end;
                    (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
                    (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
                } else {
                    VIsual_mode.set('V' as ::core::ffi::c_int);
                    (*curwin.get()).w_cursor.lnum = (*eap).line1;
                    (*curwin.get()).w_cursor.col = 1 as ::core::ffi::c_int as colnr_T;
                    tpos.lnum = (*eap).line2;
                    tpos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                    tpos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
                VIsual_active.set(true_0 != 0);
                VIsual_reselect.set(true_0);
                check_cursor(curwin.get());
                VIsual.set((*curwin.get()).w_cursor);
                (*curwin.get()).w_cursor = tpos;
                check_cursor(curwin.get());
                if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                    && gchar_cursor() != NUL
                {
                    (*curwin.get()).w_cursor.col += 1;
                }
            }
        }
        if idx == MENU_INDEX_INVALID as ::core::ffi::c_int || eap.is_null() {
            idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
        }
        if !(*menu).strings[idx as usize].is_null()
            && (*menu).modes & (1 as ::core::ffi::c_int) << idx != 0
        {
            if eap.is_null() || (*current_sctx.ptr()).sc_sid != 0 as ::core::ffi::c_int {
                let mut save_state: save_state_T = save_state_T {
                    save_msg_scroll: 0,
                    save_restart_edit: 0,
                    save_msg_didout: false,
                    save_State: 0,
                    save_finish_op: false,
                    save_opcount: 0,
                    save_reg_executing: 0,
                    save_pending_end_reg_executing: false,
                    tabuf: tasave_T {
                        save_typebuf: typebuf_T {
                            tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                            tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                            tb_buflen: 0,
                            tb_off: 0,
                            tb_len: 0,
                            tb_maplen: 0,
                            tb_silent: 0,
                            tb_no_abbr_cnt: 0,
                            tb_change_cnt: 0,
                        },
                        typebuf_valid: false,
                        old_char: 0,
                        old_mod_mask: 0,
                        save_readbuf1: buffheader_T {
                            bh_first: buffblock_T {
                                b_next: ::core::ptr::null_mut::<buffblock>(),
                                b_strlen: 0,
                                b_str: [0; 1],
                            },
                            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                            bh_index: 0,
                            bh_space: 0,
                            bh_create_newblock: false,
                        },
                        save_readbuf2: buffheader_T {
                            bh_first: buffblock_T {
                                b_next: ::core::ptr::null_mut::<buffblock>(),
                                b_strlen: 0,
                                b_str: [0; 1],
                            },
                            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                            bh_index: 0,
                            bh_space: 0,
                            bh_create_newblock: false,
                        },
                        save_inputbuf: String_0 {
                            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            size: 0,
                        },
                    },
                };
                (*ex_normal_busy.ptr()) += 1;
                if save_current_state(&raw mut save_state) {
                    exec_normal_cmd(
                        (*menu).strings[idx as usize],
                        (*menu).noremap[idx as usize],
                        (*menu).silent[idx as usize],
                    );
                }
                restore_current_state(&raw mut save_state);
                (*ex_normal_busy.ptr()) -= 1;
            } else {
                ins_typebuf(
                    (*menu).strings[idx as usize],
                    (*menu).noremap[idx as usize],
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    (*menu).silent[idx as usize],
                );
            }
        } else if !eap.is_null() {
            let mut mode: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            match idx {
                1 => {
                    mode = c"Visual".as_ptr() as *mut ::core::ffi::c_char;
                }
                2 => {
                    mode = c"Select".as_ptr() as *mut ::core::ffi::c_char;
                }
                3 => {
                    mode = c"Op-pending".as_ptr() as *mut ::core::ffi::c_char;
                }
                6 => {
                    mode = c"Terminal".as_ptr() as *mut ::core::ffi::c_char;
                }
                4 => {
                    mode = c"Insert".as_ptr() as *mut ::core::ffi::c_char;
                }
                5 => {
                    mode = c"Cmdline".as_ptr() as *mut ::core::ffi::c_char;
                }
                _ => {
                    mode = c"Normal".as_ptr() as *mut ::core::ffi::c_char;
                }
            }
            semsg_c!(
                gettext(c"E335: Menu not defined for %s mode".as_ptr()),
                mode,
            );
        }
    }
}

unsafe extern "C" fn menu_getbyname(mut name_arg: *mut ::core::ffi::c_char) -> *mut vimmenu_T {
    unsafe {
        let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name_arg);
        let mut menu: *mut vimmenu_T = *get_root_menu(saved_name);
        let mut name: *mut ::core::ffi::c_char = saved_name;
        let mut gave_emsg: bool = false_0 != 0;
        while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if *p as ::core::ffi::c_int == NUL && !(*menu).children.is_null() {
                        emsg(gettext(
                            c"E333: Menu path must lead to a menu item".as_ptr(),
                        ));
                        gave_emsg = true_0 != 0;
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                    } else if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                        emsg(gettext(e_notsubmenu.as_ptr()));
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                    }
                    break;
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() || *p as ::core::ffi::c_int == NUL {
                break;
            }
            menu = (*menu).children;
            name = p;
        }
        xfree(saved_name as *mut ::core::ffi::c_void);
        if menu.is_null() {
            if !gave_emsg {
                semsg_c!(gettext(c"E334: Menu not found: %s".as_ptr()), name_arg,);
            }
            return ::core::ptr::null_mut::<vimmenu_T>();
        }
        return menu;
    }
}

pub unsafe fn ex_emenu(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut mode_idx: ::core::ffi::c_int = MENU_INDEX_INVALID as ::core::ffi::c_int;
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && ascii_iswhite(*arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            match *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                110 => {
                    mode_idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
                }
                118 => {
                    mode_idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
                }
                115 => {
                    mode_idx = MENU_INDEX_SELECT as ::core::ffi::c_int;
                }
                111 => {
                    mode_idx = MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
                }
                116 => {
                    mode_idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
                }
                105 => {
                    mode_idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
                }
                99 => {
                    mode_idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
                }
                _ => {
                    semsg_c!(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        arg,
                    );
                    return;
                }
            }
            arg = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
        }
        let mut menu: *mut vimmenu_T = menu_getbyname(arg);
        if menu.is_null() {
            return;
        }
        execute_menu(eap, menu, mode_idx);
    }
}

pub unsafe extern "C" fn menu_find(mut path_name: *const ::core::ffi::c_char) -> *mut vimmenu_T {
    unsafe {
        let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
        let mut saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
        let mut name: *mut ::core::ffi::c_char = saved_name;
        '_theend: {
            while *name != 0 {
                let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
                while !menu.is_null() {
                    if menu_name_equal(name, menu) {
                        if (*menu).children.is_null() {
                            if *p as ::core::ffi::c_int == NUL {
                                emsg(gettext(c"E336: Menu path must lead to a sub-menu".as_ptr()));
                            } else {
                                emsg(gettext(e_notsubmenu.as_ptr()));
                            }
                            menu = ::core::ptr::null_mut::<vimmenu_T>();
                            break '_theend;
                        } else if *p as ::core::ffi::c_int == NUL {
                            break '_theend;
                        } else {
                            break;
                        }
                    } else {
                        menu = (*menu).next;
                    }
                }
                if menu.is_null() {
                    break;
                }
                menu = (*menu).children;
                name = p;
            }
            if menu.is_null() {
                emsg(gettext(c"E337: Menu not found - check menu names".as_ptr()));
            }
        }
        xfree(saved_name as *mut ::core::ffi::c_void);
        return menu;
    }
}
