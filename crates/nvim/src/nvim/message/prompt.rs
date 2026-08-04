//! The two prompts: hit-enter and the `--More--` pager.
//!
//! [`wait_return`] is what puts `Press ENTER or type command to continue` up
//! after a message that scrolled; [`do_more_prompt`] is the pager the
//! `'more'` option raises when the message area fills. Both run their own key
//! loop, which is why almost nothing else in this module blocks.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_end_prompt() {
    unsafe {
        need_wait_return.set(false_0 != 0);
        emsg_on_display.set(false_0 != 0);
        cmdline_row.set(msg_row.get());
        msg_col.set(0 as ::core::ffi::c_int);
        msg_clr_eos();
        lines_left.set(-1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn wait_return(mut redraw: ::core::ffi::c_int) {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        let mut had_got_int: ::core::ffi::c_int = 0;
        let mut save_scriptout: *mut FILE = ::core::ptr::null_mut::<FILE>();
        if redraw == true_0 {
            redraw_all_later(UPD_NOT_VALID);
        }
        if ui_has(kUIMessages) {
            prompt_for_input(
                b"Press any key to continue\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                HLF_M,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            return;
        }
        if msg_silent.get() != 0 as ::core::ffi::c_int {
            return;
        }
        if headless_mode.get() as ::core::ffi::c_int != 0 && ui_active() == 0 {
            return;
        }
        if vgetc_busy.get() > 0 as ::core::ffi::c_int {
            return;
        }
        need_wait_return.set(true_0 != 0);
        if no_wait_return.get() != 0 {
            if !exmode_active.get() {
                cmdline_row.set(msg_row.get());
            }
            return;
        }
        redir_off.set(true_0 != 0);
        let mut oldState: ::core::ffi::c_int = State.get();
        if quit_more.get() {
            c = CAR;
            quit_more.set(false_0 != 0);
            got_int.set(false_0 != 0);
        } else if exmode_active.get() {
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
            c = CAR;
            got_int.set(false_0 != 0);
        } else if !stuff_empty() {
            c = CAR;
        } else {
            State.set(MODE_HITRETURN);
            setmouse();
            cmdline_row.set(msg_row.get());
            if need_check_timestamps.get() {
                check_timestamps(false_0);
            }
            if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
                msg_grid_validate();
                msg_scroll_up(false_0 != 0, true_0 != 0);
                (*msg_scrolled.ptr()) += 1;
                cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            }
            if msg_flags.get() & kOptMoptFlagHitEnter as ::core::ffi::c_int != 0 {
                hit_return_msg(true_0 != 0);
                loop {
                    had_got_int = got_int.get() as ::core::ffi::c_int;
                    (*no_mapping.ptr()) += 1;
                    (*allow_keys.ptr()) += 1;
                    let save_reg_recording: ::core::ffi::c_int = reg_recording.get();
                    save_scriptout = scriptout.get();
                    reg_recording.set(0 as ::core::ffi::c_int);
                    scriptout.set(::core::ptr::null_mut::<FILE>());
                    c = safe_vgetc();
                    if had_got_int != 0 && global_busy.get() == 0 {
                        got_int.set(false_0 != 0);
                    }
                    (*no_mapping.ptr()) -= 1;
                    (*allow_keys.ptr()) -= 1;
                    reg_recording.set(save_reg_recording);
                    scriptout.set(save_scriptout);
                    if p_more.get() != 0 {
                        if c == 'b' as ::core::ffi::c_int
                            || c == Ctrl_B
                            || c == 'k' as ::core::ffi::c_int
                            || c == 'u' as ::core::ffi::c_int
                            || c == 'g' as ::core::ffi::c_int
                            || c == K_UP
                            || c == K_PAGEUP
                        {
                            if msg_scrolled.get() > Rows.get() {
                                do_more_prompt(c);
                            } else {
                                msg_didout.set(false_0 != 0);
                                c = -(253 as ::core::ffi::c_int
                                    + ((KE_IGNORE as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int));
                                msg_col.set(0 as ::core::ffi::c_int);
                            }
                            if quit_more.get() {
                                c = CAR;
                                quit_more.set(false_0 != 0);
                                got_int.set(false_0 != 0);
                            } else if c
                                != -(253 as ::core::ffi::c_int
                                    + ((KE_IGNORE as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int))
                            {
                                c = -(253 as ::core::ffi::c_int
                                    + ((KE_IGNORE as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int));
                                hit_return_msg(false_0 != 0);
                            }
                        } else if msg_scrolled.get() > Rows.get() - 2 as ::core::ffi::c_int
                            && (c == 'j' as ::core::ffi::c_int
                                || c == 'd' as ::core::ffi::c_int
                                || c == 'f' as ::core::ffi::c_int
                                || c == Ctrl_F
                                || c == K_DOWN
                                || c == K_PAGEDOWN)
                        {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                        }
                    }
                    if !(had_got_int != 0 && c == Ctrl_C
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MIDDLERELEASE as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        || c == -(253 as ::core::ffi::c_int
                            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                    {
                        break;
                    }
                }
                os_breakcheck();
                if c == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    jump_to_mouse(
                        MOUSE_SETPOS as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<bool>(),
                        0 as ::core::ffi::c_int,
                    );
                } else if vim_strchr(b"\r\n \0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                    && c != Ctrl_C
                    && c != 'q' as ::core::ffi::c_int
                {
                    ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
                    do_redraw.set(true_0 != 0);
                }
            } else {
                c = CAR;
                do_sleep(msg_wait.get() as int64_t, true_0 != 0);
            }
        }
        redir_off.set(false_0 != 0);
        if c == ':' as ::core::ffi::c_int
            || c == '?' as ::core::ffi::c_int
            || c == '/' as ::core::ffi::c_int
        {
            if !exmode_active.get() {
                cmdline_row.set(msg_row.get());
            }
            skip_redraw.set(true_0 != 0);
            do_redraw.set(false_0 != 0);
        }
        let mut tmpState: ::core::ffi::c_int = State.get();
        State.set(oldState);
        setmouse();
        msg_check();
        need_wait_return.set(false_0 != 0);
        did_wait_return.set(true_0 != 0);
        emsg_on_display.set(false_0 != 0);
        lines_left.set(-1 as ::core::ffi::c_int);
        reset_last_sourcing();
        if !(*keep_msg.ptr()).is_null()
            && vim_strsize(keep_msg.get())
                >= (Rows.get() - cmdline_row.get() - 1 as ::core::ffi::c_int) * Columns.get()
                    + sc_col.get()
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        if tmpState == MODE_SETWSIZE {
            ui_refresh();
        } else if !skip_redraw.get() {
            if redraw == true_0
                || msg_scrolled.get() != 0 as ::core::ffi::c_int
                    && redraw != -1 as ::core::ffi::c_int
            {
                redraw_later(curwin.get(), UPD_VALID);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn hit_return_msg(mut newline_sb: bool) {
    unsafe {
        let mut save_p_more: ::core::ffi::c_int = p_more.get();
        if !newline_sb {
            p_more.set(false_0);
        }
        if msg_didout.get() {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        p_more.set(false_0);
        if got_int.get() {
            msg_puts(gettext(
                b"Interrupt: \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        msg_puts_hl(
            gettext(
                b"Press ENTER or type command to continue\0".as_ptr() as *const ::core::ffi::c_char
            ),
            HLF_R,
            false_0 != 0,
        );
        if msg_use_printf() == 0 {
            msg_clr_eos();
        }
        p_more.set(save_p_more);
    }
}

pub(crate) unsafe extern "C" fn do_more_prompt(mut typed_char: ::core::ffi::c_int) -> bool {
    unsafe {
        static entered: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut used_typed_char: ::core::ffi::c_int = typed_char;
        let mut oldState: ::core::ffi::c_int = State.get();
        let mut c: ::core::ffi::c_int = 0;
        let mut retval: bool = false_0 != 0;
        let mut to_redraw: bool = false_0 != 0;
        let mut mp_last: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
        let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
        let mut no_need_more: bool = headless_mode.get() as ::core::ffi::c_int != 0
            && !embedded_mode.get()
            && ui_active() == 0;
        if no_need_more as ::core::ffi::c_int != 0
            || entered.get() as ::core::ffi::c_int != 0
            || State.get() == MODE_HITRETURN && typed_char == 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        entered.set(true_0 != 0);
        if typed_char == 'G' as ::core::ffi::c_int {
            mp_last = msg_sb_start(last_msgchunk.get());
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < Rows.get() - 2 as ::core::ffi::c_int
                && !mp_last.is_null()
                && !(*mp_last).sb_prev.is_null()
            {
                mp_last = msg_sb_start((*mp_last).sb_prev);
                i += 1;
            }
        }
        State.set(MODE_ASKMORE);
        setmouse();
        if typed_char == NUL {
            msg_moremsg(false_0 != 0);
        }
        's_528: loop {
            if used_typed_char != NUL {
                c = used_typed_char;
                used_typed_char = NUL;
            } else {
                c = get_keystroke(resize_events.get());
            }
            let mut toscroll: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            's_276: {
                match c {
                    BS | K_BS | 107 | K_UP => {
                        toscroll = -1 as ::core::ffi::c_int;
                        break 's_276;
                    }
                    CAR | NL | 106 | K_DOWN => {
                        toscroll = 1 as ::core::ffi::c_int;
                        break 's_276;
                    }
                    117 => {
                        toscroll = -(Rows.get() / 2 as ::core::ffi::c_int);
                        break 's_276;
                    }
                    100 => {
                        toscroll = Rows.get() / 2 as ::core::ffi::c_int;
                        break 's_276;
                    }
                    98 | Ctrl_B | K_PAGEUP => {
                        toscroll = -(Rows.get() - 1 as ::core::ffi::c_int);
                        break 's_276;
                    }
                    32 | 102 | Ctrl_F | K_PAGEDOWN | -11517 => {
                        toscroll = Rows.get() - 1 as ::core::ffi::c_int;
                        break 's_276;
                    }
                    103 => {
                        toscroll = -999999 as ::core::ffi::c_int;
                        break 's_276;
                    }
                    71 => {
                        toscroll = 999999 as ::core::ffi::c_int;
                        lines_left.set(999999 as ::core::ffi::c_int);
                        break 's_276;
                    }
                    58 => {
                        if confirm_msg_used.get() == 0 {
                            typeahead_noflush(':' as ::core::ffi::c_int);
                            cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                            skip_redraw.set(true_0 != 0);
                            need_wait_return.set(false_0 != 0);
                        }
                    }
                    113 | Ctrl_C | ESC => {}
                    K_EVENT => {
                        multiqueue_process_events(resize_events.get());
                        to_redraw = true_0 != 0;
                        break 's_276;
                    }
                    _ => {
                        msg_moremsg(true_0 != 0);
                        continue 's_528;
                    }
                }
                if confirm_msg_used.get() != 0 {
                    retval = true_0 != 0;
                } else {
                    got_int.set(true_0 != 0);
                    quit_more.set(true_0 != 0);
                }
                lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
            }
            '_c2rust_label: {
                if toscroll == 0 as ::core::ffi::c_int || !to_redraw {
                } else {
                    __assert_fail(
                        b"(toscroll == 0) || !to_redraw\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3168 as ::core::ffi::c_uint,
                        b"_Bool do_more_prompt(int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if !(toscroll != 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0) {
                break;
            }
            if toscroll < 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0 {
                if mp_last.is_null() {
                    mp = msg_sb_start(last_msgchunk.get());
                } else if !(*mp_last).sb_prev.is_null() {
                    mp = msg_sb_start((*mp_last).sb_prev);
                } else {
                    mp = ::core::ptr::null_mut::<msgchunk_T>();
                }
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < Rows.get() - 2 as ::core::ffi::c_int
                    && !mp.is_null()
                    && !(*mp).sb_prev.is_null()
                {
                    mp = msg_sb_start((*mp).sb_prev);
                    i_0 += 1;
                }
                if !mp.is_null()
                    && (!(*mp).sb_prev.is_null() || to_redraw as ::core::ffi::c_int != 0)
                {
                    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_1 > toscroll {
                        if mp.is_null() || (*mp).sb_prev.is_null() {
                            break;
                        }
                        mp = msg_sb_start((*mp).sb_prev);
                        if mp_last.is_null() {
                            mp_last = msg_sb_start(last_msgchunk.get());
                        } else {
                            mp_last = msg_sb_start((*mp_last).sb_prev);
                        }
                        i_1 -= 1;
                    }
                    if toscroll == -1 as ::core::ffi::c_int && !to_redraw {
                        grid_ins_lines(
                            msg_grid.ptr(),
                            0 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                            Rows.get(),
                            0 as ::core::ffi::c_int,
                            Columns.get(),
                        );
                        grid_clear(
                            msg_grid_adj.ptr(),
                            0 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            Columns.get(),
                            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                        );
                        disp_sb_line(0 as ::core::ffi::c_int, mp);
                    } else {
                        grid_clear(
                            msg_grid_adj.ptr(),
                            0 as ::core::ffi::c_int,
                            Rows.get(),
                            0 as ::core::ffi::c_int,
                            Columns.get(),
                            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                        );
                        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while !mp.is_null() && i_2 < Rows.get() - 1 as ::core::ffi::c_int {
                            mp = disp_sb_line(i_2, mp);
                            (*msg_scrolled.ptr()) += 1;
                            i_2 += 1;
                        }
                        to_redraw = false_0 != 0;
                    }
                    toscroll = 0 as ::core::ffi::c_int;
                }
            } else {
                if cmdline_row.get() >= Rows.get() && !ui_has(kUIMessages) {
                    msg_scroll_up(true_0 != 0, false_0 != 0);
                    (*msg_scrolled.ptr()) += 1;
                }
                while toscroll > 0 as ::core::ffi::c_int && !mp_last.is_null() {
                    if msg_do_throttle() as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).throttled
                    {
                        (*msg_scrolled_at_flush.ptr()) -= 1;
                        (*msg_grid_scroll_discount.ptr()) += 1;
                    }
                    msg_scroll_up(true_0 != 0, false_0 != 0);
                    inc_msg_scrolled();
                    grid_clear(
                        msg_grid_adj.ptr(),
                        Rows.get() - 2 as ::core::ffi::c_int,
                        Rows.get() - 1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        Columns.get(),
                        *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                    );
                    mp_last = disp_sb_line(Rows.get() - 2 as ::core::ffi::c_int, mp_last);
                    toscroll -= 1;
                }
            }
            if toscroll <= 0 as ::core::ffi::c_int {
                grid_clear(
                    msg_grid_adj.ptr(),
                    Rows.get() - 1 as ::core::ffi::c_int,
                    Rows.get(),
                    0 as ::core::ffi::c_int,
                    Columns.get(),
                    *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                );
                msg_moremsg(false_0 != 0);
            } else {
                lines_left.set(toscroll);
                break;
            }
        }
        grid_clear(
            msg_grid_adj.ptr(),
            Rows.get() - 1 as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
        );
        redraw_cmdline.set(true_0 != 0);
        clear_cmdline.set(false_0 != 0);
        mode_displayed.set(false_0 != 0);
        State.set(oldState);
        setmouse();
        if quit_more.get() {
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            msg_col.set(0 as ::core::ffi::c_int);
        }
        entered.set(false_0 != 0);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn msg_moremsg(mut full: bool) {
    unsafe {
        let mut attr: ::core::ffi::c_int = hl_combine_attr(
            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
            *(*hl_attr_active.ptr()).offset(HLF_M as isize),
        );
        grid_line_start(msg_grid_adj.ptr(), Rows.get() - 1 as ::core::ffi::c_int);
        let mut len: ::core::ffi::c_int = grid_line_puts(
            0 as ::core::ffi::c_int,
            gettext(b"-- More --\0".as_ptr() as *const ::core::ffi::c_char),
            -1 as ::core::ffi::c_int,
            attr,
        );
        if full {
            len += grid_line_puts(
                len,
                gettext(
                    b" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit \0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                -1 as ::core::ffi::c_int,
                attr,
            );
        }
        grid_line_cursor_goto(len);
        grid_line_flush();
    }
}

pub unsafe extern "C" fn repeat_message() {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        if State.get() == MODE_ASKMORE {
            msg_moremsg(true_0 != 0);
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        } else if State.get() & MODE_CMDLINE != 0 && !(*confirm_msg.ptr()).is_null() {
            display_confirm_msg();
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        } else if State.get() == MODE_EXTERNCMD {
            ui_cursor_goto(msg_row.get(), msg_col.get());
        } else if State.get() == MODE_HITRETURN || State.get() == MODE_SETWSIZE {
            if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int {
                msg_didout.set(false_0 != 0);
                msg_col.set(0 as ::core::ffi::c_int);
                msg_clr_eos();
            }
            hit_return_msg(false_0 != 0);
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        }
    }
}

pub unsafe extern "C" fn msg_delay(mut ms: uint64_t, mut ignoreinput: bool) {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        if nvim_testing.get() {
            ms = 100 as uint64_t;
        }
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"msg_delay\0".as_ptr() as *const ::core::ffi::c_char,
            4047 as ::core::ffi::c_int,
            true_0 != 0,
            b"%lu ms%s\0".as_ptr() as *const ::core::ffi::c_char,
            ms,
            if nvim_testing.get() as ::core::ffi::c_int != 0 {
                b" (skipped by NVIM_TEST)\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        ui_flush();
        os_delay(ms, ignoreinput);
    }
}

pub unsafe extern "C" fn msg_check_for_delay(mut check_msg_scroll: bool) {
    unsafe {
        if (emsg_on_display.get() as ::core::ffi::c_int != 0
            || check_msg_scroll as ::core::ffi::c_int != 0 && msg_scroll.get() != 0)
            && !did_wait_return.get()
            && emsg_silent.get() == 0 as ::core::ffi::c_int
            && !in_assert_fails.get()
            && !ui_has(kUIMessages)
        {
            msg_delay(1006 as uint64_t, true_0 != 0);
            emsg_on_display.set(false_0 != 0);
            if check_msg_scroll {
                msg_scroll.set(false_0);
            }
        }
    }
}
