//! "The next character is not a command": CTRL-V, CTRL-K, CTRL-E and
//! CTRL-Y.
//!
//! `ins_ctrl_v` and `get_literal` are CTRL-V: show a `^`, read one more
//! key, and insert it as itself -- with a decimal, octal, hex or Unicode
//! escape if what follows is digits.  `ins_digraph` is CTRL-K, the same
//! shape over two characters and the digraph table.  `insert_special` is
//! the shared tail that inserts the result, taking care of `K_SPECIAL`
//! escaping and of `mod_mask`.  `ins_copychar`/`ins_ctrl_ey` are CTRL-E and
//! CTRL-Y, which copy the character from the line below or above, and
//! `redo_literal` records what all of them produced so that `.` repeats the
//! character rather than the sequence.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_ctrl_v() {
    unsafe {
        let mut did_putchar: bool = false_0 != 0;
        ins_redraw(false_0 != 0);
        if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
            edit_putchar('^' as ::core::ffi::c_int, true_0 != 0);
            did_putchar = true_0 != 0;
        }
        AppendToRedobuff(CTRL_V_STR.as_ptr());
        add_to_showcmd_c(Ctrl_V);
        let mut c: ::core::ffi::c_int = get_literal(mod_mask.get() & MOD_MASK_SHIFT != 0);
        if did_putchar {
            edit_unputchar();
        }
        clear_showcmd();
        insert_special(c, true_0, true_0);
        (*revins_chars.ptr()) += 1;
        (*revins_legal.ptr()) += 1;
    }
}

pub unsafe extern "C" fn get_literal(mut no_simplify: bool) -> ::core::ffi::c_int {
    unsafe {
        let mut nc: ::core::ffi::c_int = 0;
        let mut hex: bool = false_0 != 0;
        let mut octal: bool = false_0 != 0;
        let mut unicode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if got_int.get() {
            return Ctrl_C;
        }
        (*no_mapping.ptr()) += 1;
        let mut cc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            nc = plain_vgetc();
            if !no_simplify {
                nc = merge_modifiers(nc, mod_mask.ptr());
            }
            if mod_mask.get() & !MOD_MASK_SHIFT != 0 as ::core::ffi::c_int {
                break;
            }
            if State.get() & MODE_CMDLINE == 0 as ::core::ffi::c_int
                && (if nc < 0 as ::core::ffi::c_int || nc > 255 as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    utf8len_tab[nc as usize] as ::core::ffi::c_int
                }) == 1 as ::core::ffi::c_int
            {
                add_to_showcmd(nc);
            }
            if nc == 'x' as ::core::ffi::c_int || nc == 'X' as ::core::ffi::c_int {
                hex = true_0 != 0;
            } else if nc == 'o' as ::core::ffi::c_int || nc == 'O' as ::core::ffi::c_int {
                octal = true_0 != 0;
            } else if nc == 'u' as ::core::ffi::c_int || nc == 'U' as ::core::ffi::c_int {
                unicode = nc;
            } else {
                if hex as ::core::ffi::c_int != 0 || unicode != 0 as ::core::ffi::c_int {
                    if !ascii_isxdigit(nc) {
                        break;
                    }
                    cc = cc * 16 as ::core::ffi::c_int + hex2nr(nc);
                } else if octal {
                    if nc < '0' as ::core::ffi::c_int || nc > '7' as ::core::ffi::c_int {
                        break;
                    }
                    cc = cc * 8 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
                } else {
                    if !ascii_isdigit(nc) {
                        break;
                    }
                    cc = cc * 10 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
                }
                i += 1;
            }
            if cc > 255 as ::core::ffi::c_int && unicode == 0 as ::core::ffi::c_int {
                cc = 255 as ::core::ffi::c_int;
            }
            nc = 0 as ::core::ffi::c_int;
            if hex {
                if i >= 2 as ::core::ffi::c_int {
                    break;
                }
            } else if unicode != 0 {
                if unicode == 'u' as ::core::ffi::c_int && i >= 4 as ::core::ffi::c_int
                    || unicode == 'U' as ::core::ffi::c_int && i >= 8 as ::core::ffi::c_int
                {
                    break;
                }
            } else if i >= 3 as ::core::ffi::c_int {
                break;
            }
        }
        if i == 0 as ::core::ffi::c_int {
            if nc == K_ZERO {
                cc = '\n' as ::core::ffi::c_int;
                nc = 0 as ::core::ffi::c_int;
            } else {
                cc = nc;
                nc = 0 as ::core::ffi::c_int;
            }
        }
        if cc == 0 as ::core::ffi::c_int {
            cc = '\n' as ::core::ffi::c_int;
        }
        (*no_mapping.ptr()) -= 1;
        if nc != 0 {
            vungetc(nc);
            mod_mask.set(0 as ::core::ffi::c_int);
        }
        got_int.set(false_0 != 0);
        return cc;
    }
}

pub(crate) unsafe extern "C" fn insert_special(
    mut c: ::core::ffi::c_int,
    mut allow_modmask: ::core::ffi::c_int,
    mut ctrlv: ::core::ffi::c_int,
) {
    unsafe {
        if mod_mask.get() & MOD_MASK_CMD != 0 {
            allow_modmask = true_0;
        }
        if c < 0 as ::core::ffi::c_int || mod_mask.get() != 0 && allow_modmask != 0 {
            let mut p: *mut ::core::ffi::c_char = get_special_key_name(c, mod_mask.get());
            let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
            c = *p.offset((len - 1 as ::core::ffi::c_int) as isize) as uint8_t
                as ::core::ffi::c_int;
            if len > 2 as ::core::ffi::c_int {
                if stop_arrow() == FAIL {
                    return;
                }
                *p.offset((len - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
                ins_str(p, (len - 1 as ::core::ffi::c_int) as size_t);
                AppendToRedobuffLit(p, -1 as ::core::ffi::c_int);
                ctrlv = false_0;
            }
        }
        if stop_arrow() == OK {
            insertchar(
                c,
                if ctrlv != 0 {
                    INSCHAR_CTRLV as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                -1 as ::core::ffi::c_int,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn redo_literal(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 10] = [0; 10];
        if ascii_isdigit(c) {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
                b"%03d\0".as_ptr() as *const ::core::ffi::c_char,
                c,
            );
            AppendToRedobuff(&raw mut buf as *mut ::core::ffi::c_char);
        } else {
            AppendCharToRedobuff(c);
        };
    }
}

pub(crate) unsafe extern "C" fn ins_digraph() -> ::core::ffi::c_int {
    unsafe {
        let mut did_putchar: bool = false_0 != 0;
        pc_status.set(PC_STATUS_UNSET);
        if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
            ins_redraw(false_0 != 0);
            edit_putchar('?' as ::core::ffi::c_int, true_0 != 0);
            did_putchar = true_0 != 0;
            add_to_showcmd_c(Ctrl_K);
        }
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut c: ::core::ffi::c_int = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if did_putchar {
            edit_unputchar();
        }
        if c < 0 as ::core::ffi::c_int || mod_mask.get() != 0 {
            clear_showcmd();
            insert_special(c, true_0, false_0);
            return NUL;
        }
        if c != ESC {
            did_putchar = false_0 != 0;
            if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
                ins_redraw(false_0 != 0);
                if char2cells(c) == 1 as ::core::ffi::c_int {
                    ins_redraw(false_0 != 0);
                    edit_putchar(c, true_0 != 0);
                    did_putchar = true_0 != 0;
                }
                add_to_showcmd_c(c);
            }
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
            let mut cc: ::core::ffi::c_int = plain_vgetc();
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if did_putchar {
                edit_unputchar();
            }
            if cc != ESC {
                AppendToRedobuff(CTRL_V_STR.as_ptr());
                c = digraph_get(c, cc, true_0 != 0);
                clear_showcmd();
                return c;
            }
        }
        clear_showcmd();
        return NUL;
    }
}

pub unsafe extern "C" fn ins_copychar(mut lnum: linenr_T) -> ::core::ffi::c_int {
    unsafe {
        if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
            vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
            return NUL;
        }
        validate_virtcol(curwin.get());
        let end_vcol: ::core::ffi::c_int = (*curwin.get()).w_virtcol as ::core::ffi::c_int;
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut csarg: CharsizeArg = CharsizeArg::default();
        let mut cstype: CharsizeKind = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while vcol < end_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
            vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            if vcol > end_vcol {
                break;
            }
            ci = utfc_next(ci);
        }
        let mut c: ::core::ffi::c_int = if ci.chr.value < 0 as int32_t {
            *ci.ptr as uint8_t as ::core::ffi::c_int
        } else {
            ci.chr.value as ::core::ffi::c_int
        };
        if c == NUL {
            vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        return c;
    }
}

pub(crate) unsafe extern "C" fn ins_ctrl_ey(mut tc: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = tc;
        if ctrl_x_mode_scroll() {
            if c == Ctrl_Y {
                scrolldown_clamp();
            } else {
                scrollup_clamp();
            }
            redraw_later(curwin.get(), UPD_VALID);
        } else {
            c = ins_copychar(
                (*curwin.get()).w_cursor.lnum
                    + (if c == Ctrl_Y {
                        -1 as linenr_T
                    } else {
                        1 as linenr_T
                    }),
            );
            if c != NUL {
                if c < 256 as ::core::ffi::c_int
                    && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                        & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                {
                    AppendToRedobuff(CTRL_V_STR.as_ptr());
                }
                let mut tw_save: OptInt = (*curbuf.get()).b_p_tw;
                (*curbuf.get()).b_p_tw = -1 as OptInt;
                insert_special(c, true_0, false_0);
                (*curbuf.get()).b_p_tw = tw_save;
                (*revins_chars.ptr()) += 1;
                (*revins_legal.ptr()) += 1;
                c = Ctrl_V;
                auto_format(false_0 != 0, true_0 != 0);
            }
        }
        return c;
    }
}
