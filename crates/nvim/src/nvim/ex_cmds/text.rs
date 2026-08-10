//! Rearranging the text of a line in place -- `:left`, `:right`, `:center` and
//! the read-only `:ascii`.
//!
//! `ex_align` is the whole of the three alignment commands: it measures the
//! line with `linelen` (which is also `:sort`'s width oracle), works out the
//! new indent against 'textwidth'/'shiftwidth' and the `:right` argument, and
//! rewrites the leading whitespace.  `do_ascii` is `ga`: the code point under
//! the cursor spelled decimal, hex, octal and by digraph.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn do_ascii(mut _eap: *mut exarg_T) {
    unsafe {
        let mut data: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
        let mut len: size_t = utfc_ptr2len(data) as size_t;
        if len == 0 as size_t {
            msg(c"NUL".as_ptr(), 0 as ::core::ffi::c_int);
            return;
        }
        let mut need_clear: bool = true_0 != 0;
        msg_sb_eol();
        msg_start();
        let mut c: ::core::ffi::c_int = utf_ptr2char(data);
        let mut off: size_t = 0 as size_t;
        if c < 0x80 as ::core::ffi::c_int {
            if c == NL {
                c = NUL;
            }
            let cval: ::core::ffi::c_int = if c == CAR && get_fileformat(curbuf.get()) == EOL_MAC {
                NL
            } else {
                c
            };
            let mut buf1: [::core::ffi::c_char; 20] = [0; 20];
            if vim_isprintc(c) as ::core::ffi::c_int != 0
                && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
            {
                let mut buf3: [::core::ffi::c_char; 7] = [0; 7];
                transchar_nonprint(curbuf.get(), &raw mut buf3 as *mut ::core::ffi::c_char, c);
                vim_snprintf(
                    &raw mut buf1 as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                    c"  <%s>".as_ptr(),
                    &raw mut buf3 as *mut ::core::ffi::c_char,
                );
            } else {
                buf1[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            }
            let mut buf2: [::core::ffi::c_char; 20] = [0; 20];
            buf2[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            let dig = get_digraph_for_char(cval);
            if let Some(dig) = &dig {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    gettext(c"<%s>%s%s  %d,  Hex %02x,  Oct %03o, Digr %s".as_ptr()),
                    transchar(c),
                    &raw mut buf1 as *mut ::core::ffi::c_char,
                    &raw mut buf2 as *mut ::core::ffi::c_char,
                    cval,
                    cval,
                    cval,
                    dig.as_ptr(),
                );
            } else {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    gettext(c"<%s>%s%s  %d,  Hex %02x,  Octal %03o".as_ptr()),
                    transchar(c),
                    &raw mut buf1 as *mut ::core::ffi::c_char,
                    &raw mut buf2 as *mut ::core::ffi::c_char,
                    cval,
                    cval,
                    cval,
                );
            }
            msg_multiline(
                cstr_as_string(IObuff.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
                &raw mut need_clear,
            );
            off = off.wrapping_add(utf_ptr2len(data) as size_t);
        }
        while off < len {
            c = utf_ptr2char(data.add(off));
            let mut iobuff_len: size_t = 0 as size_t;
            if off > 0 as size_t {
                let c2rust_fresh0 = iobuff_len;
                iobuff_len = iobuff_len.wrapping_add(1);
                (*IObuff.ptr())[c2rust_fresh0 as usize] = ' ' as ::core::ffi::c_char;
            }
            let c2rust_fresh1 = iobuff_len;
            iobuff_len = iobuff_len.wrapping_add(1);
            (*IObuff.ptr())[c2rust_fresh1 as usize] = '<' as ::core::ffi::c_char;
            if utf_iscomposing_first(c) {
                let c2rust_fresh2 = iobuff_len;
                iobuff_len = iobuff_len.wrapping_add(1);
                (*IObuff.ptr())[c2rust_fresh2 as usize] = ' ' as ::core::ffi::c_char;
            }
            iobuff_len = iobuff_len.wrapping_add(utf_char2bytes(
                c,
                (IObuff.ptr() as *mut ::core::ffi::c_char).add(iobuff_len),
            ) as size_t);
            let dig_0 = get_digraph_for_char(c);
            if let Some(dig_0) = &dig_0 {
                vim_snprintf(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).add(iobuff_len),
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>().wrapping_sub(iobuff_len),
                    if c < 0x10000 as ::core::ffi::c_int {
                        gettext(c"> %d, Hex %04x, Oct %o, Digr %s".as_ptr())
                    } else {
                        gettext(c"> %d, Hex %08x, Oct %o, Digr %s".as_ptr())
                    },
                    c,
                    c,
                    c,
                    dig_0.as_ptr(),
                );
            } else {
                vim_snprintf(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).add(iobuff_len),
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>().wrapping_sub(iobuff_len),
                    if c < 0x10000 as ::core::ffi::c_int {
                        gettext(c"> %d, Hex %04x, Octal %o".as_ptr())
                    } else {
                        gettext(c"> %d, Hex %08x, Octal %o".as_ptr())
                    },
                    c,
                    c,
                    c,
                );
            }
            msg_multiline(
                cstr_as_string(IObuff.ptr() as *mut ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
                &raw mut need_clear,
            );
            off = off.wrapping_add(utf_ptr2len(data.add(off)) as size_t);
        }
        if need_clear {
            msg_clr_eos();
        }
        msg_end();
    }
}

pub unsafe fn ex_align(mut eap: *mut exarg_T) {
    unsafe {
        let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut new_indent: ::core::ffi::c_int = 0;
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_right as ::core::ffi::c_int {
                (*eap).cmdidx = CMD_left;
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
                (*eap).cmdidx = CMD_right;
            }
        }
        let mut width: ::core::ffi::c_int = atoi((*eap).arg);
        let mut save_curpos: pos_T = (*curwin.get()).w_cursor;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
            if width >= 0 as ::core::ffi::c_int {
                indent = width;
            }
        } else {
            if width <= 0 as ::core::ffi::c_int {
                width = (*curbuf.get()).b_p_tw as ::core::ffi::c_int;
            }
            if width == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_wm > 0 as OptInt {
                width = (*curwin.get()).w_view_width - (*curbuf.get()).b_p_wm as ::core::ffi::c_int;
            }
            if width <= 0 as ::core::ffi::c_int {
                width = 80 as ::core::ffi::c_int;
            }
        }
        if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
            return;
        }
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        while (*curwin.get()).w_cursor.lnum <= (*eap).line2 {
            's_118: {
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
                    new_indent = indent;
                } else {
                    let mut has_tab: ::core::ffi::c_int = false_0;
                    let mut len: ::core::ffi::c_int = linelen(
                        if (*eap).cmdidx as ::core::ffi::c_int == CMD_right as ::core::ffi::c_int {
                            &raw mut has_tab
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_int>()
                        },
                    ) - get_indent();
                    if len <= 0 as ::core::ffi::c_int {
                        break 's_118;
                    } else if (*eap).cmdidx as ::core::ffi::c_int
                        == CMD_center as ::core::ffi::c_int
                    {
                        new_indent = (width - len) / 2 as ::core::ffi::c_int;
                    } else {
                        new_indent = width - len;
                        if has_tab != 0 {
                            while new_indent > 0 as ::core::ffi::c_int {
                                set_indent(new_indent, 0 as ::core::ffi::c_int);
                                if linelen(::core::ptr::null_mut::<::core::ffi::c_int>()) <= width {
                                    loop {
                                        new_indent += 1;
                                        set_indent(new_indent, 0 as ::core::ffi::c_int);
                                        if linelen(::core::ptr::null_mut::<::core::ffi::c_int>())
                                            > width
                                        {
                                            break;
                                        }
                                    }
                                    new_indent -= 1;
                                    break;
                                } else {
                                    new_indent -= 1;
                                }
                            }
                        }
                    }
                }
                new_indent = if new_indent > 0 as ::core::ffi::c_int {
                    new_indent
                } else {
                    0 as ::core::ffi::c_int
                };
                set_indent(new_indent, 0 as ::core::ffi::c_int);
            }
            (*curwin.get()).w_cursor.lnum += 1;
        }
        changed_lines(
            curbuf.get(),
            (*eap).line1,
            0 as colnr_T,
            (*eap).line2 + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        (*curwin.get()).w_cursor = save_curpos;
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
}

unsafe extern "C" fn linelen(mut has_tab: *mut ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if *line as ::core::ffi::c_int == NUL {
            return 0 as ::core::ffi::c_int;
        }
        let mut first: *mut ::core::ffi::c_char = skipwhite(line);
        last = first.add(strlen(first));
        while last > first
            && ascii_iswhite(*last.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            last = last.offset(-1);
        }
        let mut save: ::core::ffi::c_char = *last;
        *last = NUL as ::core::ffi::c_char;
        let mut len: ::core::ffi::c_int = linetabsize_str(line);
        if !has_tab.is_null() {
            *has_tab = !vim_strchr(first, TAB).is_null() as ::core::ffi::c_int;
        }
        *last = save;
        return len;
    }
}
