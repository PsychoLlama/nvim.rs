//! Showing the matches: the command-line popup menu and the wildmenu.
//!
//! The two renderings of the same match array.  [`cmdline_pum_create`] turns
//! it into `pum_display` items; [`redraw_wildmenu`] draws the one-line
//! statusline form instead.  [`cmdline_compl_use_pum`] is the choice between
//! them, and the `cmdline_compl_*` accessors are what `cmdcomplete_info()`
//! and the `ext_cmdline` UI read.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn cmdline_pum_create(
    mut ccline: *mut CmdlineInfo,
    mut xp: *mut expand_T,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut numMatches: ::core::ffi::c_int,
    mut showtail: bool,
    mut noselect: bool,
) {
    unsafe {
        '_c2rust_label: {
            if numMatches >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                b"numMatches >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                389 as ::core::ffi::c_uint,
                b"void cmdline_pum_create(CmdlineInfo *, expand_T *, char **, int, _Bool, _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        compl_match_array.set(xmalloc(
            ::core::mem::size_of::<pumitem_T>().wrapping_mul(numMatches as size_t),
        ) as *mut pumitem_T);
        compl_match_arraysize.set(numMatches);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < numMatches {
            *(*compl_match_array.ptr()).offset(i as isize) = pumitem_T {
                pum_text: if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                } else {
                    *matches.offset(i as isize)
                },
                pum_kind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                pum_extra: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                pum_info: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                pum_cpt_source_idx: 0,
                pum_user_abbr_hlattr: -1 as ::core::ffi::c_int,
                pum_user_kind_hlattr: -1 as ::core::ffi::c_int,
            };
            i += 1;
        }
        let mut endpos: *mut ::core::ffi::c_char = if showtail as ::core::ffi::c_int != 0 {
            showmatches_gettail((*xp).xp_pattern, noselect)
        } else {
            (*xp).xp_pattern
        };
        if ui_has(kUICmdline) as ::core::ffi::c_int != 0 && (*cmdline_win.ptr()).is_null() {
            compl_startcol.set(endpos.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int);
        } else {
            compl_startcol.set(cmd_screencol(
                endpos.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int
            ));
        };
    }
}

pub unsafe extern "C" fn cmdline_pum_display(mut changed_array: bool) {
    unsafe {
        pum_display(
            compl_match_array.get(),
            compl_match_arraysize.get(),
            compl_selected.get(),
            changed_array,
            compl_startcol.get(),
        );
    }
}

pub unsafe extern "C" fn cmdline_pum_active() -> bool {
    unsafe {
        return pum_visible() as ::core::ffi::c_int != 0 && !(*compl_match_array.ptr()).is_null();
    }
}

pub unsafe extern "C" fn cmdline_pum_remove(mut defer_redraw: bool) {
    unsafe {
        pum_undisplay(!defer_redraw);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            compl_match_array.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        compl_match_arraysize.set(0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn cmdline_pum_cleanup(mut cclp: *mut CmdlineInfo) {
    unsafe {
        cmdline_pum_remove(false_0 != 0);
        wildmenu_cleanup(cclp);
    }
}

pub unsafe extern "C" fn cmdline_compl_pattern() -> *mut ::core::ffi::c_char {
    unsafe {
        let mut xp: *mut expand_T = (*get_cmdline_info()).xpc;
        return if xp.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*xp).xp_orig
        };
    }
}

pub unsafe extern "C" fn cmdline_compl_is_fuzzy() -> bool {
    unsafe {
        let mut xp: *mut expand_T = (*get_cmdline_info()).xpc;
        return !xp.is_null() && cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0;
    }
}

pub(crate) unsafe extern "C" fn cmdline_compl_use_pum(mut need_wildmenu: bool) -> bool {
    unsafe {
        return need_wildmenu as ::core::ffi::c_int != 0
            && wop_flags.get() & kOptWopFlagPum as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && !(ui_has(kUICmdline) as ::core::ffi::c_int != 0 && (*cmdline_win.ptr()).is_null())
            || ui_has(kUIWildmenu) as ::core::ffi::c_int != 0
            || ui_has(kUICmdline) as ::core::ffi::c_int != 0
                && ui_has(kUIPopupmenu) as ::core::ffi::c_int != 0;
    }
}

pub(crate) unsafe extern "C" fn skip_wildmenu_char(
    mut xp: *mut expand_T,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if rem_backslash(s) as ::core::ffi::c_int != 0
            && (*xp).xp_context != EXPAND_HELP as ::core::ffi::c_int
            && (*xp).xp_context != EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int
            || ((*xp).xp_context == EXPAND_MENUS as ::core::ffi::c_int
                || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
                && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\t' as ::core::ffi::c_int
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL)
        {
            if (*xp).xp_shell as ::core::ffi::c_int != 0
                && csh_like_shell()
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '!' as ::core::ffi::c_int
            {
                return 2 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn wildmenu_match_len(
    mut xp: *mut expand_T,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut emenu: ::core::ffi::c_int = ((*xp).xp_context == EXPAND_MENUS as ::core::ffi::c_int
            || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if emenu != 0 && menu_is_separator(s) as ::core::ffi::c_int != 0 {
            return 1 as ::core::ffi::c_int;
        }
        while *s as ::core::ffi::c_int != NUL {
            s = s.offset(skip_wildmenu_char(xp, s) as isize);
            len += ptr2cells(s);
            s = s.offset(utfc_ptr2len(s) as isize);
        }
        return len;
    }
}

pub(crate) unsafe extern "C" fn redraw_wildmenu(
    mut xp: *mut expand_T,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut match_0: ::core::ffi::c_int,
    mut showtail: bool,
) {
    unsafe {
        let mut highlight: bool = true_0 != 0;
        let mut selstart: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut selstart_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut selend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        static first_match: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        let mut add_left: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0;
        let mut l: ::core::ffi::c_int = 0;
        if matches.is_null() {
            return;
        }
        let mut buf: *mut ::core::ffi::c_char = xmalloc(
            (Columns.get() as size_t)
                .wrapping_mul(MB_MAXBYTES as ::core::ffi::c_int as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        if match_0 == -1 as ::core::ffi::c_int {
            match_0 = 0 as ::core::ffi::c_int;
            highlight = false_0 != 0;
        }
        let mut clen: ::core::ffi::c_int = wildmenu_match_len(
            xp,
            if showtail as ::core::ffi::c_int != 0 {
                showmatches_gettail(*matches.offset(match_0 as isize), false_0 != 0)
            } else {
                *matches.offset(match_0 as isize)
            },
        ) + 3 as ::core::ffi::c_int;
        if match_0 == 0 as ::core::ffi::c_int {
            first_match.set(0 as ::core::ffi::c_int);
        } else if match_0 < first_match.get() {
            first_match.set(match_0);
            add_left = true_0 != 0;
        } else {
            i = first_match.get();
            while i < match_0 {
                clen += wildmenu_match_len(
                    xp,
                    if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                    } else {
                        *matches.offset(i as isize)
                    },
                ) + 2 as ::core::ffi::c_int;
                i += 1;
            }
            if first_match.get() > 0 as ::core::ffi::c_int {
                clen += 2 as ::core::ffi::c_int;
            }
            if clen > Columns.get() {
                first_match.set(match_0);
                clen = 2 as ::core::ffi::c_int;
                i = match_0;
                while i < num_matches {
                    clen += wildmenu_match_len(
                        xp,
                        if showtail as ::core::ffi::c_int != 0 {
                            showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                        } else {
                            *matches.offset(i as isize)
                        },
                    ) + 2 as ::core::ffi::c_int;
                    if clen >= Columns.get() {
                        break;
                    }
                    i += 1;
                }
                if i == num_matches {
                    add_left = true_0 != 0;
                }
            }
        }
        if add_left {
            while first_match.get() > 0 as ::core::ffi::c_int {
                clen += wildmenu_match_len(
                    xp,
                    if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(
                            *matches.offset((first_match.get() - 1 as ::core::ffi::c_int) as isize),
                            false_0 != 0,
                        )
                    } else {
                        *matches.offset((first_match.get() - 1 as ::core::ffi::c_int) as isize)
                    },
                ) + 2 as ::core::ffi::c_int;
                if clen >= Columns.get() {
                    break;
                }
                (*first_match.ptr()) -= 1;
            }
        }
        let mut len: ::core::ffi::c_int = 0;
        let mut group: hlf_T = HLF_NONE;
        let mut fillchar: schar_T = fillchar_status(&raw mut group, curwin.get());
        let mut attr: ::core::ffi::c_int = win_hl_attr(curwin.get(), group as ::core::ffi::c_int);
        if first_match.get() == 0 as ::core::ffi::c_int {
            *buf = NUL as ::core::ffi::c_char;
            len = 0 as ::core::ffi::c_int;
        } else {
            strcpy(
                buf,
                b"< \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            len = 2 as ::core::ffi::c_int;
        }
        clen = len;
        i = first_match.get();
        while (clen
            + wildmenu_match_len(
                xp,
                if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                } else {
                    *matches.offset(i as isize)
                },
            )
            + 2 as ::core::ffi::c_int)
            < Columns.get()
        {
            if i == match_0 {
                selstart = buf.offset(len as isize);
                selstart_col = clen;
            }
            let mut s: *mut ::core::ffi::c_char = if showtail as ::core::ffi::c_int != 0 {
                showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
            } else {
                *matches.offset(i as isize)
            };
            let mut emenu: ::core::ffi::c_int = ((*xp).xp_context
                == EXPAND_MENUS as ::core::ffi::c_int
                || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            if emenu != 0 && menu_is_separator(s) as ::core::ffi::c_int != 0 {
                strcpy(
                    buf.offset(len as isize),
                    transchar('|' as ::core::ffi::c_int),
                );
                l = strlen(buf.offset(len as isize)) as ::core::ffi::c_int;
                len += l;
                clen += l;
            } else {
                while *s as ::core::ffi::c_int != NUL {
                    s = s.offset(skip_wildmenu_char(xp, s) as isize);
                    clen += ptr2cells(s);
                    l = utfc_ptr2len(s);
                    if l > 1 as ::core::ffi::c_int {
                        strncpy(buf.offset(len as isize), s, l as size_t);
                        s = s.offset((l - 1 as ::core::ffi::c_int) as isize);
                        len += l;
                    } else {
                        strcpy(
                            buf.offset(len as isize),
                            transchar_byte(*s as uint8_t as ::core::ffi::c_int),
                        );
                        len += strlen(buf.offset(len as isize)) as ::core::ffi::c_int;
                    }
                    s = s.offset(1);
                }
            }
            if i == match_0 {
                selend = buf.offset(len as isize);
            }
            let c2rust_fresh3 = len;
            len = len + 1;
            *buf.offset(c2rust_fresh3 as isize) = ' ' as ::core::ffi::c_char;
            let c2rust_fresh4 = len;
            len = len + 1;
            *buf.offset(c2rust_fresh4 as isize) = ' ' as ::core::ffi::c_char;
            clen += 2 as ::core::ffi::c_int;
            i += 1;
            if i == num_matches {
                break;
            }
        }
        if i != num_matches {
            let c2rust_fresh5 = len;
            len = len + 1;
            *buf.offset(c2rust_fresh5 as isize) = '>' as ::core::ffi::c_char;
            clen += 1;
        }
        *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
        let mut row: ::core::ffi::c_int = cmdline_row.get() - 1 as ::core::ffi::c_int;
        if row >= 0 as ::core::ffi::c_int {
            if wild_menu_showing.get() == 0 as ::core::ffi::c_int {
                if msg_scrolled.get() > 0 as ::core::ffi::c_int {
                    if cmdline_row.get() == Rows.get() - 1 as ::core::ffi::c_int {
                        msg_scroll_up(false_0 != 0, false_0 != 0);
                        (*msg_scrolled.ptr()) += 1;
                    } else {
                        (*cmdline_row.ptr()) += 1;
                        row += 1;
                    }
                    wild_menu_showing.set(WM_SCROLLED as ::core::ffi::c_int);
                } else {
                    if (*lastwin.get()).w_status_height == 0 as ::core::ffi::c_int
                        && global_stl_height() == 0 as ::core::ffi::c_int
                    {
                        save_p_ls.set(p_ls.get() as ::core::ffi::c_int);
                        save_p_wmh.set(p_wmh.get() as ::core::ffi::c_int);
                        p_ls.set(2 as OptInt);
                        p_wmh.set(0 as OptInt);
                        last_status(false_0 != 0);
                    }
                    wild_menu_showing.set(WM_SHOWN as ::core::ffi::c_int);
                }
            }
            grid_line_start(
                if wild_menu_showing.get() == WM_SCROLLED as ::core::ffi::c_int {
                    msg_grid_adj.ptr()
                } else {
                    default_gridview.ptr()
                },
                row,
            );
            grid_line_puts(0 as ::core::ffi::c_int, buf, -1 as ::core::ffi::c_int, attr);
            if !selstart.is_null() && highlight as ::core::ffi::c_int != 0 {
                *selend = NUL as ::core::ffi::c_char;
                grid_line_puts(
                    selstart_col,
                    selstart,
                    -1 as ::core::ffi::c_int,
                    *(*hl_attr_active.ptr()).offset(HLF_WM as isize),
                );
            }
            grid_line_fill(clen, Columns.get(), fillchar, attr);
            grid_line_flush();
        }
        win_redraw_last_status(topframe.get());
        xfree(buf as *mut ::core::ffi::c_void);
    }
}
