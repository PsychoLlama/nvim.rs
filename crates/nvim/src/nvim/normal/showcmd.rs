//! The 'showcmd' area: the partial command echoed while it is still
//! being typed.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn clear_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    if VIsual_active.get() as c_int != 0 && !char_avail() {
        let mut cursor_bot: bool = lt(VIsual.get(), (*curwin.get()).w_cursor);
        let mut lines: c_int = 0;
        let mut leftcol: colnr_T = 0;
        let mut rightcol: colnr_T = 0;
        let mut top: linenr_T = 0;
        let mut bot: linenr_T = 0;
        if cursor_bot {
            top = (*VIsual.ptr()).lnum;
            bot = (*curwin.get()).w_cursor.lnum;
        } else {
            top = (*curwin.get()).w_cursor.lnum;
            bot = (*VIsual.ptr()).lnum;
        }
        hasFolding(
            curwin.get(),
            top,
            &raw mut top,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        hasFolding(
            curwin.get(),
            bot,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut bot,
        );
        lines = (bot - top + 1 as linenr_T) as c_int;
        if VIsual_mode.get() == Ctrl_V {
            let saved_sbr: *mut c_char = p_sbr.get();
            let saved_w_sbr: *mut c_char = (*curwin.get()).w_onebuf_opt.wo_sbr;
            p_sbr.set(empty_string_option.ptr() as *mut c_char);
            (*curwin.get()).w_onebuf_opt.wo_sbr = empty_string_option.ptr() as *mut c_char;
            getvcols(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                VIsual.ptr(),
                &raw mut leftcol,
                &raw mut rightcol,
            );
            p_sbr.set(saved_sbr);
            (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
            snprintf(
                showcmd_buf.ptr() as *mut c_char,
                SHOWCMD_BUFLEN as c_int as size_t,
                b"%ldx%ld\0".as_ptr() as *const c_char,
                lines as int64_t,
                rightcol as int64_t - leftcol as int64_t + 1 as int64_t,
            );
        } else if VIsual_mode.get() == 'V' as c_int
            || (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum
        {
            snprintf(
                showcmd_buf.ptr() as *mut c_char,
                SHOWCMD_BUFLEN as c_int as size_t,
                b"%ld\0".as_ptr() as *const c_char,
                lines as int64_t,
            );
        } else {
            let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut e: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut bytes: c_int = 0 as c_int;
            let mut chars: c_int = 0 as c_int;
            if cursor_bot {
                s = ml_get_pos(VIsual.ptr());
                e = get_cursor_pos_ptr();
            } else {
                s = get_cursor_pos_ptr();
                e = ml_get_pos(VIsual.ptr());
            }
            while if *p_sel.get() as c_int != 'e' as c_int {
                (s <= e) as c_int
            } else {
                (s < e) as c_int
            } != 0
            {
                let mut l: c_int = utfc_ptr2len(s);
                if l == 0 as c_int {
                    bytes += 1;
                    chars += 1;
                    break;
                } else {
                    bytes += l;
                    chars += 1;
                    s = s.offset(l as isize);
                }
            }
            if bytes == chars {
                snprintf(
                    showcmd_buf.ptr() as *mut c_char,
                    SHOWCMD_BUFLEN as c_int as size_t,
                    b"%d\0".as_ptr() as *const c_char,
                    chars,
                );
            } else {
                snprintf(
                    showcmd_buf.ptr() as *mut c_char,
                    SHOWCMD_BUFLEN as c_int as size_t,
                    b"%d-%d\0".as_ptr() as *const c_char,
                    chars,
                    bytes,
                );
            }
        }
        let mut limit: c_int = if ui_has(kUIMessages) as c_int != 0 {
            SHOWCMD_BUFLEN as c_int - 1 as c_int
        } else {
            SHOWCMD_COLS as c_int
        };
        (*showcmd_buf.ptr())[limit as usize] = NUL as c_char;
        showcmd_visual.set(true_0 != 0);
    } else {
        (*showcmd_buf.ptr())[0 as c_int as usize] = NUL as c_char;
        showcmd_visual.set(false_0 != 0);
        if showcmd_is_clear.get() {
            return;
        }
    }
    display_showcmd();
}

pub unsafe extern "C" fn add_to_showcmd(mut c: c_int) -> bool {
    static ignore: GlobalCell<[c_int; 23]> = GlobalCell::new([
        -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTRELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLEMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLEDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLERELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEDOWN as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEUP as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSELEFT as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSERIGHT as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1MOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1DRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1RELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2MOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2DRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2RELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)),
        0 as c_int,
    ]);
    if p_sc.get() == 0 || msg_silent.get() != 0 as c_int || ex_normal_busy.get() != 0 {
        return false_0 != 0;
    }
    if showcmd_visual.get() {
        (*showcmd_buf.ptr())[0 as c_int as usize] = NUL as c_char;
        showcmd_visual.set(false_0 != 0);
    }
    if c < 0 as c_int {
        let mut i: c_int = 0 as c_int;
        while (*ignore.ptr())[i as usize] != 0 as c_int {
            if (*ignore.ptr())[i as usize] == c {
                return false_0 != 0;
            }
            i += 1;
        }
    }
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut mbyte_buf: [c_char; 7] = [0; 7];
    if c <= 0x7f as c_int || !vim_isprintc(c) {
        p = transchar(c);
        if *p as c_int == ' ' as c_int {
            strcpy(p, b"<20>\0".as_ptr() as *const c_char as *mut c_char);
        }
    } else {
        mbyte_buf[utf_char2bytes(c, &raw mut mbyte_buf as *mut c_char) as usize] = NUL as c_char;
        p = &raw mut mbyte_buf as *mut c_char;
    }
    let mut old_len: size_t = strlen(showcmd_buf.ptr() as *mut c_char);
    let mut extra_len: size_t = strlen(p);
    let mut limit: size_t = (if ui_has(kUIMessages) as c_int != 0 {
        SHOWCMD_BUFLEN as c_int - 1 as c_int
    } else {
        SHOWCMD_COLS as c_int
    }) as size_t;
    if old_len.wrapping_add(extra_len) > limit {
        let mut overflow: size_t = old_len.wrapping_add(extra_len).wrapping_sub(limit);
        memmove(
            showcmd_buf.ptr() as *mut c_char as *mut c_void,
            (showcmd_buf.ptr() as *mut c_char).offset(overflow as isize) as *const c_void,
            old_len.wrapping_sub(overflow).wrapping_add(1 as size_t),
        );
    }
    strcat(showcmd_buf.ptr() as *mut c_char, p);
    if char_avail() {
        return false_0 != 0;
    }
    display_showcmd();
    return true_0 != 0;
}

pub unsafe extern "C" fn add_to_showcmd_c(mut c: c_int) {
    add_to_showcmd(c);
    setcursor();
}

pub(crate) unsafe extern "C" fn del_from_showcmd(mut len: c_int) {
    if p_sc.get() == 0 {
        return;
    }
    let mut old_len: c_int = strlen(showcmd_buf.ptr() as *mut c_char) as c_int;
    len = if len < old_len { len } else { old_len };
    (*showcmd_buf.ptr())[(old_len - len) as usize] = NUL as c_char;
    if !char_avail() {
        display_showcmd();
    }
}

pub unsafe extern "C" fn push_showcmd() {
    if p_sc.get() != 0 {
        strcpy(
            old_showcmd_buf.ptr() as *mut c_char,
            showcmd_buf.ptr() as *mut c_char,
        );
    }
}

pub unsafe extern "C" fn pop_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    strcpy(
        showcmd_buf.ptr() as *mut c_char,
        old_showcmd_buf.ptr() as *mut c_char,
    );
    display_showcmd();
}

pub(crate) unsafe extern "C" fn display_showcmd() {
    showcmd_is_clear.set((*showcmd_buf.ptr())[0 as c_int as usize] as c_int == NUL);
    if *p_sloc.get() as c_int == 's' as c_int {
        if showcmd_is_clear.get() {
            (*curwin.get()).w_redr_status = true_0 != 0;
        } else {
            win_redr_status(curwin.get());
            setcursor();
        }
        return;
    }
    if *p_sloc.get() as c_int == 't' as c_int {
        if showcmd_is_clear.get() {
            redraw_tabline.set(true_0 != 0);
        } else {
            draw_tabline();
            setcursor();
        }
        return;
    }
    if ui_has(kUIMessages) {
        let mut content: Array = ARRAY_DICT_INIT;
        let mut content__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 1];
        content.capacity = 1 as size_t;
        content.items = &raw mut content__items as *mut Object;
        let mut chunk: Array = ARRAY_DICT_INIT;
        let mut chunk__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 3];
        chunk.capacity = 3 as size_t;
        chunk.items = &raw mut chunk__items as *mut Object;
        if !showcmd_is_clear.get() {
            let c2rust_fresh6 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh7 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_0 {
                    string: cstr_as_string(showcmd_buf.ptr() as *mut c_char),
                },
            };
            let c2rust_fresh8 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh9 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_0 { array: chunk },
            };
        }
        ui_call_msg_showcmd(content);
        return;
    }
    if p_ch.get() == 0 as OptInt {
        return;
    }
    msg_grid_validate();
    let mut showcmd_row: c_int = Rows.get() - 1 as c_int;
    grid_line_start(msg_grid_adj.ptr(), showcmd_row);
    let mut len: c_int = 0 as c_int;
    if !showcmd_is_clear.get() {
        len = grid_line_puts(
            sc_col.get(),
            showcmd_buf.ptr() as *mut c_char,
            -1 as c_int,
            *(*hl_attr_active.ptr()).offset(HLF_MSG as c_int as isize),
        );
    }
    grid_line_puts(
        sc_col.get() + len,
        (b"          \0".as_ptr() as *const c_char as *mut c_char).offset(len as isize),
        -1 as c_int,
        *(*hl_attr_active.ptr()).offset(HLF_MSG as c_int as isize),
    );
    grid_line_flush();
}
