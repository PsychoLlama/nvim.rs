//! Putting the command line on the screen.
//!
//! [`draw_cmdline`] writes the buffer out with the colour chunks
//! [`super::color`] computed; [`redrawcmd`] is the whole-line redraw and
//! [`cursorcmd`] the cursor placement.  [`put_on_cmdline`] is the other
//! direction — inserting text into the buffer and recomputing the screen
//! columns it now occupies.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cmdline_charsize(
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if cmdline_star.get() > 0 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        return ptr2cells((*ccline.ptr()).cmdbuff.offset(idx as isize));
    }
}

pub(crate) unsafe extern "C" fn cmd_startcol() -> ::core::ffi::c_int {
    unsafe {
        return (*ccline.ptr()).cmdindent
            + (if (*ccline.ptr()).cmdfirstc != NUL {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
    }
}

pub unsafe extern "C" fn cmd_screencol(mut bytepos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut m: ::core::ffi::c_int = 0;
        let mut col: ::core::ffi::c_int = cmd_startcol();
        if KeyTyped.get() {
            m = if !(*cmdline_win.ptr()).is_null() {
                (*cmdline_win.get()).w_view_width * (*cmdline_win.get()).w_view_height
            } else {
                Columns.get() * Rows.get()
            };
            if m < 0 as ::core::ffi::c_int {
                m = MAXCOL as ::core::ffi::c_int;
            }
        } else {
            m = MAXCOL as ::core::ffi::c_int;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*ccline.ptr()).cmdlen && i < bytepos {
            let mut c: ::core::ffi::c_int = cmdline_charsize(i);
            correct_screencol(i, c, &raw mut col);
            col += c;
            if col >= m {
                col -= c;
                break;
            } else {
                i += utfc_ptr2len((*ccline.ptr()).cmdbuff.offset(i as isize));
            }
        }
        return col;
    }
}

pub(crate) unsafe extern "C" fn correct_screencol(
    mut idx: ::core::ffi::c_int,
    mut cells: ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    unsafe {
        if utfc_ptr2len((*ccline.ptr()).cmdbuff.offset(idx as isize)) > 1 as ::core::ffi::c_int
            && utf_ptr2cells((*ccline.ptr()).cmdbuff.offset(idx as isize)) > 1 as ::core::ffi::c_int
            && *col % Columns.get() + cells > Columns.get()
        {
            *col += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn draw_cmdline(
    mut start: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
) {
    unsafe {
        if (*ccline.ptr()).cmdbuff.is_null() || !color_cmdline(ccline.ptr()) {
            return;
        }
        if ui_has(kUICmdline) {
            (*ccline.ptr()).special_char = NUL as ::core::ffi::c_char;
            (*ccline.ptr()).redraw_state = kCmdRedrawAll;
            return;
        }
        if cmdline_star.get() > 0 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < len {
                msg_putchar('*' as ::core::ffi::c_int);
                i += utfc_ptr2len(
                    (*ccline.ptr())
                        .cmdbuff
                        .offset(start as isize)
                        .offset(i as isize),
                ) - 1 as ::core::ffi::c_int;
                i += 1;
            }
        } else if (*ccline.ptr()).last_colors.colors.size != 0 {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*ccline.ptr()).last_colors.colors.size {
                let mut chunk: CmdlineColorChunk = *(*ccline.ptr())
                    .last_colors
                    .colors
                    .items
                    .offset(i_0 as isize);
                if chunk.end > start {
                    let chunk_start: ::core::ffi::c_int = if chunk.start > start {
                        chunk.start
                    } else {
                        start
                    };
                    msg_outtrans_len(
                        (*ccline.ptr()).cmdbuff.offset(chunk_start as isize),
                        chunk.end - chunk_start,
                        chunk.hl_id,
                        false_0 != 0,
                    );
                }
                i_0 = i_0.wrapping_add(1);
            }
        } else {
            msg_outtrans_len(
                (*ccline.ptr()).cmdbuff.offset(start as isize),
                len,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        };
    }
}

pub unsafe extern "C" fn putcmdline(mut c: ::core::ffi::c_char, mut shift: bool) {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        if !ui_has(kUICmdline) {
            msg_no_more.set(true_0 != 0);
            msg_putchar(c as ::core::ffi::c_int);
            if shift {
                draw_cmdline(
                    (*ccline.ptr()).cmdpos,
                    (*ccline.ptr()).cmdlen - (*ccline.ptr()).cmdpos,
                );
            }
            msg_no_more.set(false_0 != 0);
        } else if (*ccline.ptr()).redraw_state as ::core::ffi::c_uint
            != kCmdRedrawAll as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut charbuf: [::core::ffi::c_char; 2] = [c, 0 as ::core::ffi::c_char];
            ui_call_cmdline_special_char(
                cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
                shift as Boolean,
                (*ccline.ptr()).level as Integer,
            );
        }
        cursorcmd();
        (*ccline.ptr()).special_char = c;
        (*ccline.ptr()).special_shift = shift;
        ui_cursor_shape();
    }
}

pub unsafe extern "C" fn unputcmdline() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        msg_no_more.set(true_0 != 0);
        if (*ccline.ptr()).cmdlen == (*ccline.ptr()).cmdpos && !ui_has(kUICmdline) {
            msg_putchar(' ' as ::core::ffi::c_int);
        } else {
            draw_cmdline(
                (*ccline.ptr()).cmdpos,
                utfc_ptr2len(
                    (*ccline.ptr())
                        .cmdbuff
                        .offset((*ccline.ptr()).cmdpos as isize),
                ),
            );
        }
        msg_no_more.set(false_0 != 0);
        cursorcmd();
        (*ccline.ptr()).special_char = NUL as ::core::ffi::c_char;
        ui_cursor_shape();
    }
}

pub unsafe extern "C" fn put_on_cmdline(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut redraw: bool,
) {
    unsafe {
        if len < 0 as ::core::ffi::c_int {
            len = strlen(str) as ::core::ffi::c_int;
        }
        realloc_cmdbuff((*ccline.ptr()).cmdlen + len + 1 as ::core::ffi::c_int);
        if (*ccline.ptr()).overstrike == 0 {
            memmove(
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize)
                    .offset(len as isize) as *mut ::core::ffi::c_void,
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize)
                    as *const ::core::ffi::c_void,
                ((*ccline.ptr()).cmdlen - (*ccline.ptr()).cmdpos) as size_t,
            );
            (*ccline.ptr()).cmdlen += len;
        } else {
            let mut m: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            while i < len {
                m += 1;
                i += utfc_ptr2len(str.offset(i as isize));
            }
            i = (*ccline.ptr()).cmdpos;
            while i < (*ccline.ptr()).cmdlen && m > 0 as ::core::ffi::c_int {
                m -= 1;
                i += utfc_ptr2len((*ccline.ptr()).cmdbuff.offset(i as isize));
            }
            if i < (*ccline.ptr()).cmdlen {
                memmove(
                    (*ccline.ptr())
                        .cmdbuff
                        .offset((*ccline.ptr()).cmdpos as isize)
                        .offset(len as isize) as *mut ::core::ffi::c_void,
                    (*ccline.ptr()).cmdbuff.offset(i as isize) as *const ::core::ffi::c_void,
                    ((*ccline.ptr()).cmdlen - i) as size_t,
                );
                (*ccline.ptr()).cmdlen += (*ccline.ptr()).cmdpos + len - i;
            } else {
                (*ccline.ptr()).cmdlen = (*ccline.ptr()).cmdpos + len;
            }
        }
        memmove(
            (*ccline.ptr())
                .cmdbuff
                .offset((*ccline.ptr()).cmdpos as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
        *(*ccline.ptr())
            .cmdbuff
            .offset((*ccline.ptr()).cmdlen as isize) = NUL as ::core::ffi::c_char;
        if (*ccline.ptr()).cmdpos > 0 as ::core::ffi::c_int
            && *(*ccline.ptr())
                .cmdbuff
                .offset((*ccline.ptr()).cmdpos as isize) as uint8_t
                as ::core::ffi::c_int
                >= 0x80 as ::core::ffi::c_int
        {
            let mut i_0: ::core::ffi::c_int = utf_head_off(
                (*ccline.ptr()).cmdbuff,
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize),
            );
            if i_0 != 0 as ::core::ffi::c_int {
                (*ccline.ptr()).cmdpos -= i_0;
                len += i_0;
                (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
            }
        }
        if redraw as ::core::ffi::c_int != 0 && !cmd_silent.get() {
            msg_no_more.set(true_0 != 0);
            let mut i_1: ::core::ffi::c_int = cmdline_row.get();
            cursorcmd();
            draw_cmdline(
                (*ccline.ptr()).cmdpos,
                (*ccline.ptr()).cmdlen - (*ccline.ptr()).cmdpos,
            );
            if cmdline_row.get() != i_1 || (*ccline.ptr()).overstrike != 0 {
                msg_clr_eos();
            }
            msg_no_more.set(false_0 != 0);
        }
        let mut m_0: ::core::ffi::c_int = 0;
        if KeyTyped.get() {
            m_0 = Columns.get() * Rows.get();
            if m_0 < 0 as ::core::ffi::c_int {
                m_0 = MAXCOL as ::core::ffi::c_int;
            }
        } else {
            m_0 = MAXCOL as ::core::ffi::c_int;
        }
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < len {
            let mut c: ::core::ffi::c_int = cmdline_charsize((*ccline.ptr()).cmdpos);
            correct_screencol((*ccline.ptr()).cmdpos, c, &raw mut (*ccline.ptr()).cmdspos);
            if (*ccline.ptr()).cmdspos + c < m_0 {
                (*ccline.ptr()).cmdspos += c;
            }
            c = utfc_ptr2len(
                (*ccline.ptr())
                    .cmdbuff
                    .offset((*ccline.ptr()).cmdpos as isize),
            ) - 1 as ::core::ffi::c_int;
            c = if c < len - i_2 - 1 as ::core::ffi::c_int {
                c
            } else {
                len - i_2 - 1 as ::core::ffi::c_int
            };
            (*ccline.ptr()).cmdpos += c;
            i_2 += c;
            (*ccline.ptr()).cmdpos += 1;
            i_2 += 1;
        }
        if redraw {
            msg_check();
        }
    }
}

pub unsafe extern "C" fn redrawcmdline() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        need_wait_return.set(false_0 != 0);
        compute_cmdrow();
        redrawcmd();
        cursorcmd();
        ui_cursor_shape();
    }
}

pub(crate) unsafe extern "C" fn redrawcmdprompt() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        if ui_has(kUICmdline) {
            (*ccline.ptr()).redraw_state = kCmdRedrawAll;
            return;
        }
        if (*ccline.ptr()).cmdfirstc != NUL {
            msg_putchar((*ccline.ptr()).cmdfirstc);
        }
        if !(*ccline.ptr()).cmdprompt.is_null() {
            msg_puts_hl(
                (*ccline.ptr()).cmdprompt,
                (*ccline.ptr()).hl_id,
                false_0 != 0,
            );
            (*ccline.ptr()).cmdindent =
                msg_col.get() + (msg_row.get() - cmdline_row.get()) * Columns.get();
            if (*ccline.ptr()).cmdfirstc != NUL {
                (*ccline.ptr()).cmdindent -= 1;
            }
        } else {
            let mut i: ::core::ffi::c_int = (*ccline.ptr()).cmdindent;
            while i > 0 as ::core::ffi::c_int {
                msg_putchar(' ' as ::core::ffi::c_int);
                i -= 1;
            }
        };
    }
}

pub unsafe extern "C" fn redrawcmd() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        if ui_has(kUICmdline) {
            draw_cmdline(0 as ::core::ffi::c_int, (*ccline.ptr()).cmdlen);
            return;
        }
        if (*ccline.ptr()).cmdbuff.is_null() {
            msg_cursor_goto(cmdline_row.get(), 0 as ::core::ffi::c_int);
            msg_clr_eos();
            return;
        }
        redrawing_cmdline.set(true_0 != 0);
        sb_text_restart_cmdline();
        msg_start();
        redrawcmdprompt();
        msg_no_more.set(true_0 != 0);
        draw_cmdline(0 as ::core::ffi::c_int, (*ccline.ptr()).cmdlen);
        msg_clr_eos();
        msg_no_more.set(false_0 != 0);
        (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
        if (*ccline.ptr()).special_char as ::core::ffi::c_int != NUL {
            putcmdline((*ccline.ptr()).special_char, (*ccline.ptr()).special_shift);
        }
        msg_scroll.set(false_0);
        skip_redraw.set(false_0 != 0);
        cmdline_was_last_drawn.set(true_0 != 0);
        redrawing_cmdline.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn compute_cmdrow() {
    unsafe {
        if exmode_active.get() as ::core::ffi::c_int != 0
            || msg_scrolled.get() != 0 as ::core::ffi::c_int
        {
            cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        } else {
            let mut wp: *mut win_T = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
            cmdline_row.set(
                (*wp).w_winrow
                    + (*wp).w_height
                    + (*wp).w_hsep_height
                    + (*wp).w_status_height
                    + global_stl_height(),
            );
        }
        if cmdline_row.get() == Rows.get() && p_ch.get() > 0 as OptInt {
            (*cmdline_row.ptr()) -= 1;
        }
        lines_left.set(cmdline_row.get());
    }
}

pub unsafe extern "C" fn cursorcmd() {
    unsafe {
        if cmd_silent.get() as ::core::ffi::c_int != 0
            || ui_has(kUICmdline) as ::core::ffi::c_int != 0
        {
            return;
        }
        msg_row.set(cmdline_row.get() + (*ccline.ptr()).cmdspos / Columns.get());
        msg_col.set((*ccline.ptr()).cmdspos % Columns.get());
        msg_row.set(if msg_row.get() < Rows.get() - 1 as ::core::ffi::c_int {
            msg_row.get()
        } else {
            Rows.get() - 1 as ::core::ffi::c_int
        });
        msg_cursor_goto(msg_row.get(), msg_col.get());
    }
}

pub unsafe extern "C" fn gotocmdline(mut clr: bool) {
    unsafe {
        if ui_has(kUICmdline) {
            return;
        }
        msg_start();
        msg_col.set(0 as ::core::ffi::c_int);
        if clr {
            msg_clr_eos();
        }
        msg_cursor_goto(cmdline_row.get(), 0 as ::core::ffi::c_int);
    }
}
