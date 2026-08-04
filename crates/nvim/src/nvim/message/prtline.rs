//! `:print` and `:list`: one buffer line onto the message area.
//!
//! [`msg_prt_line`] is the only message path that knows about `'listchars'`,
//! `'tabstop'` and the lead/trail/multispace distinctions, which is why it
//! duplicates so much of the drawing code's character loop.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_prt_line(mut s: *const ::core::ffi::c_char, mut list: bool) {
    unsafe {
        let mut sc: schar_T = 0;
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut n_extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sc_extra: schar_T = 0 as schar_T;
        let mut sc_final: schar_T = 0 as schar_T;
        let mut p_extra: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut n: ::core::ffi::c_int = 0;
        let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lead: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut in_multispace: bool = false_0 != 0;
        let mut multispace_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut trail: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut l: ::core::ffi::c_int = 0;
        if (*curwin.get()).w_onebuf_opt.wo_list != 0 {
            list = true_0 != 0;
        }
        if list {
            if (*curwin.get()).w_p_lcs_chars.trail != 0 {
                trail = s.offset(strlen(s) as isize);
                while trail > s
                    && ascii_iswhite(
                        *trail.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                {
                    trail = trail.offset(-1);
                }
            }
            if (*curwin.get()).w_p_lcs_chars.lead != 0
                || !(*curwin.get()).w_p_lcs_chars.leadmultispace.is_null()
                || (*curwin.get()).w_p_lcs_chars.leadtab1 != NUL as schar_T
            {
                lead = s;
                while ascii_iswhite(
                    *lead.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) {
                    lead = lead.offset(1);
                }
                if *lead as ::core::ffi::c_int == NUL {
                    lead = ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
        }
        if *s as ::core::ffi::c_int == NUL
            && !(list as ::core::ffi::c_int != 0
                && (*curwin.get()).w_p_lcs_chars.eol != NUL as schar_T)
        {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        while !got_int.get() {
            if n_extra > 0 as ::core::ffi::c_int {
                n_extra -= 1;
                if n_extra == 0 as ::core::ffi::c_int && sc_final != 0 {
                    sc = sc_final;
                } else if sc_extra != 0 {
                    sc = sc_extra;
                } else {
                    '_c2rust_label: {
                        if !p_extra.is_null() {
                        } else {
                            __assert_fail(
                                b"p_extra != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                2209 as ::core::ffi::c_uint,
                                b"void msg_prt_line(const char *, _Bool)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    let c2rust_fresh34 = p_extra;
                    p_extra = p_extra.offset(1);
                    sc = *c2rust_fresh34 as ::core::ffi::c_uchar as schar_T;
                }
            } else {
                l = utfc_ptr2len(s);
                if l > 1 as ::core::ffi::c_int {
                    col += utf_ptr2cells(s);
                    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
                    if l >= MB_MAXBYTES as ::core::ffi::c_int {
                        xstrlcpy(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            b"?\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 22]>(),
                        );
                    } else if (*curwin.get()).w_p_lcs_chars.nbsp != NUL as schar_T
                        && list as ::core::ffi::c_int != 0
                        && (utf_ptr2char(s) == 160 as ::core::ffi::c_int
                            || utf_ptr2char(s) == 0x202f as ::core::ffi::c_int)
                    {
                        schar_get(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            (*curwin.get()).w_p_lcs_chars.nbsp,
                        );
                    } else {
                        memmove(
                            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                            s as *const ::core::ffi::c_void,
                            l as size_t,
                        );
                        buf[l as usize] = NUL as ::core::ffi::c_char;
                    }
                    msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
                    s = s.offset(l as isize);
                    continue;
                } else {
                    hl_id = 0 as ::core::ffi::c_int;
                    let c2rust_fresh35 = s;
                    s = s.offset(1);
                    let mut c: ::core::ffi::c_int =
                        *c2rust_fresh35 as uint8_t as ::core::ffi::c_int;
                    if c >= 0x80 as ::core::ffi::c_int {
                        col += utf_char2cells(c);
                        msg_putchar(c);
                        continue;
                    } else {
                        sc_extra = NUL as schar_T;
                        sc_final = NUL as schar_T;
                        if list {
                            in_multispace = c == ' ' as ::core::ffi::c_int
                                && (*s as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                    || col > 0 as ::core::ffi::c_int
                                        && *s.offset(-2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == ' ' as ::core::ffi::c_int);
                            if !in_multispace {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                        }
                        if c == TAB && (!list || (*curwin.get()).w_p_lcs_chars.tab1 != 0) {
                            n_extra = tabstop_padding(
                                col as colnr_T,
                                (*curbuf.get()).b_p_ts,
                                (*curbuf.get()).b_p_vts_array,
                            ) - 1 as ::core::ffi::c_int;
                            if !list {
                                sc = ' ' as ::core::ffi::c_int as schar_T;
                                sc_extra = ' ' as ::core::ffi::c_int as schar_T;
                            } else {
                                let mut lcs_tab1: schar_T = (*curwin.get()).w_p_lcs_chars.tab1;
                                let mut lcs_tab2: schar_T = (*curwin.get()).w_p_lcs_chars.tab2;
                                let mut lcs_tab3: schar_T = (*curwin.get()).w_p_lcs_chars.tab3;
                                if !lead.is_null()
                                    && s <= lead
                                    && (*curwin.get()).w_p_lcs_chars.leadtab1 != NUL as schar_T
                                {
                                    lcs_tab1 = (*curwin.get()).w_p_lcs_chars.leadtab1;
                                    lcs_tab2 = (*curwin.get()).w_p_lcs_chars.leadtab2;
                                    lcs_tab3 = (*curwin.get()).w_p_lcs_chars.leadtab3;
                                }
                                sc = if n_extra == 0 as ::core::ffi::c_int && lcs_tab3 != 0 {
                                    lcs_tab3
                                } else {
                                    lcs_tab1
                                };
                                sc_extra = lcs_tab2;
                                sc_final = lcs_tab3;
                                hl_id = HLF_0;
                            }
                        } else if c == NUL
                            && list as ::core::ffi::c_int != 0
                            && (*curwin.get()).w_p_lcs_chars.eol != NUL as schar_T
                        {
                            p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char;
                            n_extra = 1 as ::core::ffi::c_int;
                            sc = (*curwin.get()).w_p_lcs_chars.eol;
                            hl_id = HLF_AT;
                            s = s.offset(-1);
                        } else if c != NUL && {
                            n = byte2cells(c);
                            n > 1 as ::core::ffi::c_int
                        } {
                            n_extra = n - 1 as ::core::ffi::c_int;
                            p_extra = transchar_byte_buf(::core::ptr::null::<buf_T>(), c);
                            let c2rust_fresh36 = p_extra;
                            p_extra = p_extra.offset(1);
                            sc = *c2rust_fresh36 as schar_T;
                            hl_id = HLF_0;
                        } else if c == ' ' as ::core::ffi::c_int {
                            if !lead.is_null()
                                && s <= lead
                                && in_multispace as ::core::ffi::c_int != 0
                                && !(*curwin.get()).w_p_lcs_chars.leadmultispace.is_null()
                            {
                                let c2rust_fresh37 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                sc = *(*curwin.get())
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(c2rust_fresh37 as isize);
                                if *(*curwin.get())
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                                hl_id = HLF_0;
                            } else if !lead.is_null()
                                && s <= lead
                                && (*curwin.get()).w_p_lcs_chars.lead != NUL as schar_T
                            {
                                sc = (*curwin.get()).w_p_lcs_chars.lead;
                                hl_id = HLF_0;
                            } else if !trail.is_null() && s > trail {
                                sc = (*curwin.get()).w_p_lcs_chars.trail;
                                hl_id = HLF_0;
                            } else if in_multispace as ::core::ffi::c_int != 0
                                && !(*curwin.get()).w_p_lcs_chars.multispace.is_null()
                            {
                                let c2rust_fresh38 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                sc = *(*curwin.get())
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(c2rust_fresh38 as isize);
                                if *(*curwin.get())
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                                hl_id = HLF_0;
                            } else if list as ::core::ffi::c_int != 0
                                && (*curwin.get()).w_p_lcs_chars.space != NUL as schar_T
                            {
                                sc = (*curwin.get()).w_p_lcs_chars.space;
                                hl_id = HLF_0;
                            } else {
                                sc = ' ' as ::core::ffi::c_int as schar_T;
                            }
                        } else {
                            sc = c as schar_T;
                        }
                    }
                }
            }
            if sc == NUL as schar_T {
                break;
            }
            let mut buf_0: [::core::ffi::c_char; 32] = [0; 32];
            schar_get(&raw mut buf_0 as *mut ::core::ffi::c_char, sc);
            msg_puts_hl(
                &raw mut buf_0 as *mut ::core::ffi::c_char,
                hl_id,
                false_0 != 0,
            );
            col += 1;
        }
        msg_clr_eos();
    }
}
