//! The `:popup` terminal menu.
//!
//! [`pum_show_popupmenu`] runs its own key loop over a menu tree, drawing
//! through the same grid the completion menu uses and dispatching the
//! chosen entry. [`pum_select_mouse_pos`] maps a mouse position back to an
//! item for both this loop and the completion menu.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn pum_select_mouse_pos() {
    unsafe {
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        if grid == 0 as ::core::ffi::c_int {
            mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
        }
        if grid == (*pum_grid.ptr()).handle {
            let mut border_offset: ::core::ffi::c_int =
                if pum_border_width() == 2 as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            let mut item: ::core::ffi::c_int = row - border_offset;
            pum_selected.set(
                if item >= 0 as ::core::ffi::c_int && item < pum_height.get() {
                    item
                } else {
                    -1 as ::core::ffi::c_int
                },
            );
            return;
        }
        if grid != pum_anchor_grid.get()
            || col < pum_left_col.get() - pum_win_col_offset.get()
            || col >= pum_right_col.get() - pum_win_col_offset.get()
        {
            pum_selected.set(-1 as ::core::ffi::c_int);
            return;
        }
        let mut idx: ::core::ffi::c_int = row - (pum_row.get() - pum_win_row_offset.get());
        if idx < 0 as ::core::ffi::c_int || idx >= pum_height.get() {
            pum_selected.set(-1 as ::core::ffi::c_int);
        } else if *(*(*pum_array.ptr()).offset(idx as isize)).pum_text as ::core::ffi::c_int != NUL
        {
            pum_selected.set(idx);
        }
    }
}

pub(crate) unsafe extern "C" fn pum_execute_menu(
    mut menu: *mut vimmenu_T,
    mut mode: ::core::ffi::c_int,
) {
    unsafe {
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut mp: *mut vimmenu_T = (*menu).children;
        while !mp.is_null() {
            if (*mp).modes & (*mp).enabled & mode != 0 && {
                let c2rust_fresh7 = idx;
                idx = idx + 1;
                c2rust_fresh7 == pum_selected.get()
            } {
                memset(
                    &raw mut ea as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<exarg_T>(),
                );
                execute_menu(&raw mut ea, mp, -1 as ::core::ffi::c_int);
                break;
            } else {
                mp = (*mp).next;
            }
        }
    }
}

pub unsafe extern "C" fn pum_show_popupmenu(mut menu: *mut vimmenu_T) {
    unsafe {
        pum_undisplay(true_0 != 0);
        pum_size.set(0 as ::core::ffi::c_int);
        let mut mode: ::core::ffi::c_int = get_menu_mode_flag();
        let mut mp: *mut vimmenu_T = (*menu).children;
        while !mp.is_null() {
            if menu_is_separator((*mp).dname) as ::core::ffi::c_int != 0
                || (*mp).modes & (*mp).enabled & mode != 0
            {
                (*pum_size.ptr()) += 1;
            }
            mp = (*mp).next;
        }
        if pum_size.get() <= 0 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut array: *mut pumitem_T = xcalloc(
            pum_size.get() as size_t,
            ::core::mem::size_of::<pumitem_T>(),
        ) as *mut pumitem_T;
        let mut mp_0: *mut vimmenu_T = (*menu).children;
        while !mp_0.is_null() {
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if menu_is_separator((*mp_0).dname) {
                s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if (*mp_0).modes & (*mp_0).enabled & mode != 0 {
                s = (*mp_0).dname;
            }
            if !s.is_null() {
                s = xstrdup(s);
                let c2rust_fresh6 = idx;
                idx = idx + 1;
                let c2rust_lvalue_ptr = &raw mut (*array.offset(c2rust_fresh6 as isize)).pum_text;
                *c2rust_lvalue_ptr = s;
            }
            mp_0 = (*mp_0).next;
        }
        pum_array.set(array);
        pum_compute_size();
        pum_scrollbar.set(0 as ::core::ffi::c_int);
        pum_height.set(pum_size.get());
        pum_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);
        pum_position_at_mouse(20 as ::core::ffi::c_int);
        pum_selected.set(-1 as ::core::ffi::c_int);
        pum_first.set(0 as ::core::ffi::c_int);
        if p_mousemev.get() == 0 {
            ui_call_option_set(
                String_0 {
                    data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>()
                        .wrapping_sub(1 as size_t),
                },
                object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_12 { boolean: true },
                },
            );
        }
        loop {
            pum_is_visible.set(true_0 != 0);
            pum_is_drawn.set(true_0 != 0);
            (*pum_grid.ptr()).zindex = kZIndexCmdlinePopupMenu as ::core::ffi::c_int;
            pum_redraw();
            setcursor_mayforce(curwin.get(), true_0 != 0);
            let mut c: ::core::ffi::c_int = vgetc();
            if c == ESC || c == Ctrl_C || (*pum_array.ptr()).is_null() {
                break;
            }
            if c == CAR || c == NL {
                pum_execute_menu(menu, mode);
                break;
            } else if c == 'k' as ::core::ffi::c_int
                || c == K_UP
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                while pum_selected.get() > 0 as ::core::ffi::c_int {
                    (*pum_selected.ptr()) -= 1;
                    if *(*array.offset(pum_selected.get() as isize)).pum_text as ::core::ffi::c_int
                        != NUL
                    {
                        break;
                    }
                }
            } else if c == 'j' as ::core::ffi::c_int
                || c == K_DOWN
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                while pum_selected.get() < pum_size.get() - 1 as ::core::ffi::c_int {
                    (*pum_selected.ptr()) += 1;
                    if *(*array.offset(pum_selected.get() as isize)).pum_text as ::core::ffi::c_int
                        != NUL
                    {
                        break;
                    }
                }
            } else if c
                == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                vungetc(c);
                break;
            } else if c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                pum_select_mouse_pos();
            } else {
                if !(c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                {
                    continue;
                }
                pum_select_mouse_pos();
                if pum_selected.get() >= 0 as ::core::ffi::c_int {
                    pum_execute_menu(menu, mode);
                    break;
                } else if c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    break;
                }
            }
        }
        idx = 0 as ::core::ffi::c_int;
        while idx < pum_size.get() {
            xfree((*array.offset(idx as isize)).pum_text as *mut ::core::ffi::c_void);
            idx += 1;
        }
        xfree(array as *mut ::core::ffi::c_void);
        pum_undisplay(true_0 != 0);
        if p_mousemev.get() == 0 {
            ui_call_option_set(
                String_0 {
                    data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>()
                        .wrapping_sub(1 as size_t),
                },
                object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_12 { boolean: false },
                },
            );
        }
    }
}

pub unsafe extern "C" fn pum_make_popup(
    mut path_name: *const ::core::ffi::c_char,
    mut use_mouse_pos: ::core::ffi::c_int,
) {
    unsafe {
        if use_mouse_pos == 0 {
            mouse_row.set((*curwin.get()).w_grid.row_offset + (*curwin.get()).w_wrow);
            mouse_col.set(
                (*curwin.get()).w_grid.col_offset
                    + (if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
                        (*curwin.get()).w_view_width
                            - (*curwin.get()).w_wcol
                            - 1 as ::core::ffi::c_int
                    } else {
                        (*curwin.get()).w_wcol
                    }),
            );
            if ui_has(kUIMultigrid) {
                mouse_grid.set((*(*curwin.get()).w_grid.target).handle as ::core::ffi::c_int);
            } else if (*curwin.get()).w_grid.target != default_grid.ptr() {
                mouse_grid.set(0 as ::core::ffi::c_int);
                (*mouse_row.ptr()) += (*curwin.get()).w_winrow;
                (*mouse_col.ptr()) += (*curwin.get()).w_wincol;
            }
        }
        let mut menu: *mut vimmenu_T = menu_find(path_name);
        if !menu.is_null() {
            pum_show_popupmenu(menu);
        }
    }
}
