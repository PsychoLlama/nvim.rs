//! `insertchar` -- putting one typed character into the buffer.
//!
//! The common path, and the one that has to be fast: everything that is not
//! a special key ends here.  Three things make it more than an insert.  It
//! batches -- while more plain characters are already available and nothing
//! needs formatting, it collects them into one `ins_str` rather than one
//! call apiece.  It decides whether this character triggers a wrap, which
//! is 'textwidth', 'formatoptions' and 'formatexpr' and is handed to
//! `internal_format`.  And it runs the `InsertCharPre` autocommand, which
//! may replace the character with a whole string -- `do_insert_char_pre` is
//! that, and it is why the function cannot simply take an `int`.
//!
//! `echeck_abbr` is here because an abbreviation is triggered by the
//! *non*-word character that ends the word, which is the character being
//! inserted.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn insertchar(
    mut c: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut force_format: ::core::ffi::c_int = flags & INSCHAR_FORMAT as ::core::ffi::c_int;
        let textwidth: ::core::ffi::c_int = comp_textwidth(force_format != 0);
        let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
        if textwidth > 0 as ::core::ffi::c_int
            && (force_format != 0
                || !ascii_iswhite(c)
                    && !(State.get() & REPLACE_FLAG != 0
                        && State.get() & VREPLACE_FLAG == 0
                        && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL)
                    && ((*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
                        || (!has_format_option(FO_INS_LONG)
                            || Insstart_textlen.get() <= textwidth)
                            && (!fo_ins_blank || Insstart_blank_vcol.get() <= textwidth)))
        {
            let mut do_internal: bool = true_0 != 0;
            let mut virtcol: colnr_T =
                get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });
            if *(*curbuf.get()).b_p_fex as ::core::ffi::c_int != NUL
                && flags & INSCHAR_NO_FEX as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                && (force_format != 0 || virtcol > textwidth)
            {
                do_internal =
                    fex_format((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_long, c)
                        != 0 as ::core::ffi::c_int;
                ins_need_undo.set(true_0 != 0);
            }
            if do_internal {
                internal_format(textwidth, second_indent, flags, c == NUL, c);
            }
        }
        if c == NUL {
            return;
        }
        if did_ai.get() as ::core::ffi::c_int != 0 && c == end_comment_pending.get() {
            let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let mut i: ::core::ffi::c_int =
                get_leader_len(line, &raw mut p, false_0 != 0, true_0 != 0);
            if i > 0 as ::core::ffi::c_int && !vim_strchr(p, COM_MIDDLE).is_null() {
                while *p as ::core::ffi::c_int != 0
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ':' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
                let mut middle_len: ::core::ffi::c_int = copy_option_part(
                    &raw mut p,
                    &raw mut lead_end as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
                while middle_len > 0 as ::core::ffi::c_int
                    && ascii_iswhite(
                        lead_end[(middle_len - 1 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    middle_len -= 1;
                }
                while *p as ::core::ffi::c_int != 0
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ':' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
                let mut end_len: ::core::ffi::c_int = copy_option_part(
                    &raw mut p,
                    &raw mut lead_end as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
                i = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                loop {
                    i -= 1;
                    if !(i >= 0 as ::core::ffi::c_int
                        && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0)
                    {
                        break;
                    }
                }
                i += 1;
                i -= middle_len;
                if i >= 0 as ::core::ffi::c_int
                    && end_len > 0 as ::core::ffi::c_int
                    && lead_end[(end_len - 1 as ::core::ffi::c_int) as usize] as uint8_t
                        as ::core::ffi::c_int
                        == end_comment_pending.get()
                {
                    backspace_until_column(i);
                    ins_bytes_len(
                        &raw mut lead_end as *mut ::core::ffi::c_char,
                        (end_len - 1 as ::core::ffi::c_int) as size_t,
                    );
                }
            }
        }
        end_comment_pending.set(NUL);
        did_ai.set(false_0 != 0);
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        if !(c < ' ' as ::core::ffi::c_int
            || c >= DEL
            || c == '0' as ::core::ffi::c_int
            || c == '^' as ::core::ffi::c_int)
            && utf_char2len(c) == 1 as ::core::ffi::c_int
            && !has_event(EVENT_INSERTCHARPRE)
            && !test_disable_char_avail.get()
            && vpeekc() != NUL
            && State.get() & REPLACE_FLAG == 0
            && !cindent_on()
            && p_ri.get() == 0
        {
            let mut buf: [::core::ffi::c_char; 101] = [0; 101];
            let mut virtcol_0: colnr_T = 0 as colnr_T;
            buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
            let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            if textwidth > 0 as ::core::ffi::c_int {
                virtcol_0 = get_nolist_virtcol();
            }
            loop {
                c = vpeekc();
                if !(c != NUL
                    && !(c < ' ' as ::core::ffi::c_int
                        || c >= DEL
                        || c == '0' as ::core::ffi::c_int
                        || c == '^' as ::core::ffi::c_int)
                    && utf8len_tab[c as usize] as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && i_0 < INPUT_BUFLEN
                    && (textwidth == 0 as ::core::ffi::c_int || {
                        virtcol_0 += byte2cells(
                            buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                                as ::core::ffi::c_int,
                        );
                        virtcol_0 < textwidth
                    })
                    && !(!no_abbr.get()
                        && !vim_iswordc(c)
                        && vim_iswordc(
                            buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                                as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0))
                {
                    break;
                }
                c = vgetc();
                let c2rust_fresh0 = i_0;
                i_0 = i_0 + 1;
                buf[c2rust_fresh0 as usize] = c as ::core::ffi::c_char;
            }
            do_digraph(-1 as ::core::ffi::c_int);
            do_digraph(
                buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t as ::core::ffi::c_int,
            );
            buf[i_0 as usize] = NUL as ::core::ffi::c_char;
            ins_str(&raw mut buf as *mut ::core::ffi::c_char, i_0 as size_t);
            if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
                redo_literal(
                    *(&raw mut buf as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
                );
                i_0 = 1 as ::core::ffi::c_int;
            } else {
                i_0 = 0 as ::core::ffi::c_int;
            }
            if buf[i_0 as usize] as ::core::ffi::c_int != NUL {
                AppendToRedobuffLit(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(i_0 as isize),
                    -1 as ::core::ffi::c_int,
                );
            }
        } else {
            let mut cc: ::core::ffi::c_int = 0;
            cc = utf_char2len(c);
            if cc > 1 as ::core::ffi::c_int {
                let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
                utf_char2bytes(c, &raw mut buf_0 as *mut ::core::ffi::c_char);
                buf_0[cc as usize] = NUL as ::core::ffi::c_char;
                ins_char_bytes(&raw mut buf_0 as *mut ::core::ffi::c_char, cc as size_t);
                AppendCharToRedobuff(c);
            } else {
                ins_char(c);
                if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
                    redo_literal(c);
                } else {
                    AppendCharToRedobuff(c);
                }
            }
        };
    }
}

pub(crate) unsafe extern "C" fn echeck_abbr(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        if p_paste.get() != 0
            || no_abbr.get() as ::core::ffi::c_int != 0
            || arrow_used.get() as ::core::ffi::c_int != 0
        {
            return false_0 != 0;
        }
        return check_abbr(
            c,
            get_cursor_line_ptr(),
            (*curwin.get()).w_cursor.col as ::core::ffi::c_int,
            if (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum {
                (*Insstart.ptr()).col as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
    }
}

pub(crate) unsafe extern "C" fn do_insert_char_pre(
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut buf: [::core::ffi::c_char; 22] = [0; 22];
        let save_State: ::core::ffi::c_int = State.get();
        if c == Ctrl_RSB {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !has_event(EVENT_INSERTCHARPRE) {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut buflen: size_t =
            utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as size_t;
        buf[buflen as usize] = NUL as ::core::ffi::c_char;
        (*textlock.ptr()) += 1;
        set_vim_var_string(
            VV_CHAR,
            &raw mut buf as *mut ::core::ffi::c_char,
            buflen as ptrdiff_t,
        );
        let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if ins_apply_autocmds(EVENT_INSERTCHARPRE) != 0 {
            if strcmp(
                &raw mut buf as *mut ::core::ffi::c_char,
                get_vim_var_str(VV_CHAR),
            ) != 0 as ::core::ffi::c_int
            {
                res = xstrdup(get_vim_var_str(VV_CHAR));
            }
        }
        set_vim_var_string(
            VV_CHAR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        (*textlock.ptr()) -= 1;
        State.set(save_State);
        return res;
    }
}
