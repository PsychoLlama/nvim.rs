//! Drawing while inserting: postponing it, and the two things drawn
//! directly.
//!
//! `ins_redraw` is the postponement -- Insert mode does not redraw after
//! each character but just before the next key is *waited for*, which is
//! what makes a long CTRL-R or a mapping fast and is also where the
//! `TextChangedI`/`CursorMovedI` autocommands and the completion popup's
//! update live.
//!
//! `edit_putchar`/`edit_unputchar` bypass all of that: they write one
//! character straight onto the grid and remember what was under it, which is
//! how CTRL-V and CTRL-K show a `^` or a `?` at the cursor while they wait
//! for the rest of the sequence.  `display_dollar`/`undisplay_dollar` are
//! the same trick for the `$` that 'cpoptions' `$` puts at the end of a
//! changed region.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ins_redraw(mut ready: bool) {
    unsafe {
        if char_avail() {
            return;
        }
        if ready as ::core::ffi::c_int != 0
            && has_event(EVENT_CURSORMOVEDI) as ::core::ffi::c_int != 0
            && (last_cursormoved_win.get() != curwin.get()
                || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
            && !pum_visible()
        {
            if syntax_present(curwin.get()) as ::core::ffi::c_int != 0 && must_redraw.get() != 0 {
                update_screen();
            }
            update_curswant();
            ins_apply_autocmds(EVENT_CURSORMOVEDI);
            last_cursormoved_win.set(curwin.get());
            last_cursormoved.set((*curwin.get()).w_cursor);
        }
        if ready as ::core::ffi::c_int != 0
            && has_event(EVENT_TEXTCHANGEDI) as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_last_changedtick_i != buf_get_changedtick(curbuf.get())
            && !pum_visible()
        {
            let mut aco: aco_save_T = aco_save_T::default();
            let mut tick: varnumber_T = buf_get_changedtick(curbuf.get());
            aucmd_prepbuf(&raw mut aco, curbuf.get());
            apply_autocmds(
                EVENT_TEXTCHANGEDI,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            aucmd_restbuf(&raw mut aco);
            (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
            if tick != buf_get_changedtick(curbuf.get()) {
                u_save(
                    (*curwin.get()).w_cursor.lnum,
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                );
            }
        }
        if ready as ::core::ffi::c_int != 0
            && has_event(EVENT_TEXTCHANGEDP) as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_last_changedtick_pum != buf_get_changedtick(curbuf.get())
            && pum_visible() as ::core::ffi::c_int != 0
        {
            let mut aco_0: aco_save_T = aco_save_T::default();
            let mut tick_0: varnumber_T = buf_get_changedtick(curbuf.get());
            aucmd_prepbuf(&raw mut aco_0, curbuf.get());
            apply_autocmds(
                EVENT_TEXTCHANGEDP,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            aucmd_restbuf(&raw mut aco_0);
            (*curbuf.get()).b_last_changedtick_pum = buf_get_changedtick(curbuf.get());
            if tick_0 != buf_get_changedtick(curbuf.get()) {
                u_save(
                    (*curwin.get()).w_cursor.lnum,
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                );
            }
        }
        if ready {
            may_trigger_win_scrolled_resized();
        }
        if ready as ::core::ffi::c_int != 0
            && has_event(EVENT_BUFMODIFIEDSET) as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_changed_invalid as ::core::ffi::c_int == true_0
            && !pum_visible()
        {
            apply_autocmds(
                EVENT_BUFMODIFIEDSET,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            (*curbuf.get()).b_changed_invalid = false_0 != 0;
        }
        may_trigger_safestate(
            ready as ::core::ffi::c_int != 0 && !ins_compl_active() && !pum_visible(),
        );
        pum_check_clear();
        show_cursor_info_later(false_0 != 0);
        if must_redraw.get() != 0 {
            update_screen();
        } else {
            redraw_statuslines();
            if clear_cmdline.get() as ::core::ffi::c_int != 0
                || redraw_cmdline.get() as ::core::ffi::c_int != 0
                || redraw_mode.get() as ::core::ffi::c_int != 0
            {
                showmode();
            }
        }
        setcursor();
        emsg_on_display.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn edit_putchar(mut c: ::core::ffi::c_int, mut highlight: bool) {
    unsafe {
        if (*curwin.get()).w_grid_alloc.chars.is_null() && (*default_grid.ptr()).chars.is_null() {
            return;
        }
        let mut attr: ::core::ffi::c_int = 0;
        update_topline(curwin.get());
        validate_cursor(curwin.get());
        if highlight {
            attr = *(*hl_attr_active.ptr()).offset(HLF_8 as isize);
        } else {
            attr = 0 as ::core::ffi::c_int;
        }
        pc_row.set((*curwin.get()).w_wrow);
        pc_status.set(PC_STATUS_UNSET);
        grid_line_start(&raw mut (*curwin.get()).w_grid, pc_row.get());
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            pc_col.set(
                (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int - (*curwin.get()).w_wcol,
            );
            if grid_line_getchar(pc_col.get(), ::core::ptr::null_mut::<::core::ffi::c_int>())
                == NUL as schar_T
            {
                grid_line_put_schar(
                    pc_col.get() - 1 as ::core::ffi::c_int,
                    ' ' as ::core::ffi::c_int as schar_T,
                    attr,
                );
                (*curwin.get()).w_wcol -= 1;
                pc_status.set(PC_STATUS_RIGHT);
            }
        } else {
            pc_col.set((*curwin.get()).w_wcol);
            if grid_line_getchar(
                pc_col.get() + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) == NUL as schar_T
            {
                pc_status.set(PC_STATUS_LEFT);
            }
        }
        if pc_status.get() == PC_STATUS_UNSET {
            pc_schar.set(grid_line_getchar(pc_col.get(), pc_attr.ptr()));
            pc_status.set(PC_STATUS_SET);
        }
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        grid_line_puts(
            pc_col.get(),
            &raw mut buf as *mut ::core::ffi::c_char,
            utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char),
            attr,
        );
        grid_line_flush();
    }
}

pub unsafe extern "C" fn edit_unputchar() {
    unsafe {
        if pc_status.get() != PC_STATUS_UNSET {
            if pc_status.get() == PC_STATUS_RIGHT {
                (*curwin.get()).w_wcol += 1;
            }
            if pc_status.get() == PC_STATUS_RIGHT || pc_status.get() == PC_STATUS_LEFT {
                redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
            } else {
                grid_line_start(&raw mut (*curwin.get()).w_grid, pc_row.get());
                grid_line_put_schar(pc_col.get(), pc_schar.get(), pc_attr.get());
                grid_line_flush();
            }
        }
    }
}

pub unsafe extern "C" fn display_dollar(mut col_arg: colnr_T) {
    unsafe {
        let mut col: colnr_T = if col_arg > 0 as ::core::ffi::c_int {
            col_arg
        } else {
            0 as colnr_T
        };
        if !redrawing() {
            return;
        }
        let mut save_col: colnr_T = (*curwin.get()).w_cursor.col;
        (*curwin.get()).w_cursor.col = col;
        let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        (*curwin.get()).w_cursor.col -= utf_head_off(p, p.offset(col as isize));
        curs_columns(curwin.get(), false_0);
        if (*curwin.get()).w_wcol < (*curwin.get()).w_view_width {
            edit_putchar('$' as ::core::ffi::c_int, false_0 != 0);
            dollar_vcol.set((*curwin.get()).w_virtcol);
        }
        (*curwin.get()).w_cursor.col = save_col;
    }
}

pub unsafe extern "C" fn undisplay_dollar() {
    unsafe {
        if dollar_vcol.get() < 0 as ::core::ffi::c_int {
            return;
        }
        dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
        redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
}

pub unsafe extern "C" fn get_nolist_virtcol() -> colnr_T {
    unsafe {
        if (*curwin.get()).w_buffer.is_null()
            || (*(*curwin.get()).w_buffer).b_ml.ml_mfp.is_null()
            || (*curwin.get()).w_cursor.lnum > (*(*curwin.get()).w_buffer).b_ml.ml_line_count
        {
            return 0 as colnr_T;
        }
        if (*curwin.get()).w_onebuf_opt.wo_list != 0
            && vim_strchr(p_cpo.get(), CPO_LISTWM).is_null()
        {
            return getvcol_nolist(&raw mut (*curwin.get()).w_cursor);
        }
        validate_virtcol(curwin.get());
        return (*curwin.get()).w_virtcol;
    }
}
