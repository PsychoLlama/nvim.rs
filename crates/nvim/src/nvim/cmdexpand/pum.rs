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
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem::size_of;
use core::ptr;

/// Create the completion popup menu with items from `matches`.
pub(crate) unsafe fn cmdline_pum_create(
    ccline: *mut CmdlineInfo,
    xp: *mut expand_T,
    matches: *mut *mut c_char,
    numMatches: c_int,
    showtail: bool,
    noselect: bool,
) {
    unsafe {
        debug_assert!(numMatches >= 0);
        // Add all the completion matches.
        compl_match_array
            .set(xmalloc(size_of::<pumitem_T>() * numMatches as size_t) as *mut pumitem_T);
        compl_match_arraysize.set(numMatches);
        for i in 0..numMatches {
            let m = *matches.offset(i as isize);
            compl_match_array.get().offset(i as isize).write(pumitem_T {
                // C's SHOW_MATCH(i).
                pum_text: if showtail {
                    showmatches_gettail(m, false)
                } else {
                    m
                },
                pum_info: ptr::null_mut(),
                pum_extra: ptr::null_mut(),
                pum_kind: ptr::null_mut(),
                pum_cpt_source_idx: 0,
                pum_user_abbr_hlattr: -1,
                pum_user_kind_hlattr: -1,
            });
        }

        // Compute the popup menu starting column.
        let endpos = if showtail {
            showmatches_gettail((*xp).xp_pattern, noselect)
        } else {
            (*xp).xp_pattern
        };
        let col = endpos.offset_from((*ccline).cmdbuff) as c_int;
        compl_startcol.set(if ui_has(kUICmdline) && (*cmdline_win.ptr()).is_null() {
            col
        } else {
            cmd_screencol(col)
        });
    }
}

pub unsafe fn cmdline_pum_display(changed_array: bool) {
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

/// True if the cmdline completion popup menu is being displayed.
pub fn cmdline_pum_active() -> bool {
    pum_visible() && !compl_match_array.get().is_null()
}

/// Remove the cmdline completion popup menu (if present) and free the list of
/// items.
pub unsafe fn cmdline_pum_remove(defer_redraw: bool) {
    unsafe {
        pum_undisplay(!defer_redraw);
        xfree(compl_match_array.get() as *mut c_void);
        compl_match_array.set(ptr::null_mut());
        compl_match_arraysize.set(0);
    }
}

pub unsafe fn cmdline_pum_cleanup(cclp: *mut CmdlineInfo) {
    unsafe {
        cmdline_pum_remove(false);
        wildmenu_cleanup(cclp);
    }
}

/// The current cmdline completion pattern.
pub unsafe fn cmdline_compl_pattern() -> *mut c_char {
    unsafe {
        let xp = (*get_cmdline_info()).xpc;
        if xp.is_null() {
            ptr::null_mut()
        } else {
            (*xp).xp_orig
        }
    }
}

/// True if fuzzy cmdline completion is active.
pub unsafe fn cmdline_compl_is_fuzzy() -> bool {
    unsafe {
        let xp = (*get_cmdline_info()).xpc;
        !xp.is_null() && cmdline_fuzzy_completion_supported(xp)
    }
}

/// Whether the popup menu should be used for the cmdline completion wildmenu.
///
/// `need_wildmenu` says whether the current `'wildmode'` part wants one.
pub(crate) unsafe fn cmdline_compl_use_pum(need_wildmenu: bool) -> bool {
    unsafe {
        (need_wildmenu
            && wop_flags.get() & kOptWopFlagPum as c_uint != 0
            && !(ui_has(kUICmdline) && (*cmdline_win.ptr()).is_null()))
            || ui_has(kUIWildmenu)
            || (ui_has(kUICmdline) && ui_has(kUIPopupmenu))
    }
}

/// The number of characters that should be skipped in the wildmenu.
///
/// These are backslashes used for escaping.  Backslashes *are* shown in help
/// tags and in search pattern completion matches.
pub(crate) unsafe fn skip_wildmenu_char(xp: *mut expand_T, s: *mut c_char) -> c_int {
    unsafe {
        if (rem_backslash(s)
            && (*xp).xp_context != EXPAND_HELP
            && (*xp).xp_context != EXPAND_PATTERN_IN_BUF)
            || (((*xp).xp_context == EXPAND_MENUS || (*xp).xp_context == EXPAND_MENUNAMES)
                && (*s as c_int == '\t' as c_int
                    || (*s as c_int == '\\' as c_int && *s.add(1) as c_int != NUL)))
        {
            // TODO(bfredl): Why in the actual fuck are we special casing the
            // shell variety deep in the redraw logic?  Shell special
            // snowflakiness should already be eliminated multiple layers
            // before reaching the screen infrastructure.
            if (*xp).xp_shell
                && csh_like_shell()
                && *s.add(1) as c_int == '\\' as c_int
                && *s.add(2) as c_int == '!' as c_int
            {
                return 2;
            }
            return 1;
        }
        0
    }
}

/// The length of an item as it will be shown in the status line.
pub(crate) unsafe fn wildmenu_match_len(xp: *mut expand_T, s: *mut c_char) -> c_int {
    unsafe {
        let emenu = (*xp).xp_context == EXPAND_MENUS || (*xp).xp_context == EXPAND_MENUNAMES;

        // Check for menu separators - replace with '|'.
        if emenu && menu_is_separator(s) {
            return 1;
        }

        let mut len = 0;
        let mut s = s;
        while *s as c_int != NUL {
            s = s.add(skip_wildmenu_char(xp, s) as usize);
            len += ptr2cells(s);
            s = s.add(utfc_ptr2len(s) as usize);
        }

        len
    }
}

/// Show wildchar matches in the status line.
///
/// At least the `match_idx` item is shown.  We start at item `first_match` in
/// the list and show all matches that fit; if inversion is possible we use it,
/// else `=` characters are used.
pub(crate) unsafe fn redraw_wildmenu(
    xp: *mut expand_T,
    num_matches: c_int,
    matches: *mut *mut c_char,
    match_idx: c_int,
    showtail: bool,
) {
    unsafe {
        // Where the listing starts, remembered across redraws so that paging
        // through the matches does not jump.
        static first_match: GlobalCell<c_int> = GlobalCell::new(0);

        if matches.is_null() {
            // Interrupted completion?
            return;
        }

        // C's SHOW_MATCH().
        let show_match = |i: c_int| {
            let m = *matches.offset(i as isize);
            if showtail {
                showmatches_gettail(m, false)
            } else {
                m
            }
        };

        let mut highlight = true;
        let mut selstart: *mut c_char = ptr::null_mut();
        let mut selstart_col = 0;
        let mut selend: *mut c_char = ptr::null_mut();
        let mut add_left = false;
        let mut i;
        let mut l;

        let buf = xmalloc(Columns.get() as size_t * MB_MAXBYTES as size_t + 1) as *mut c_char;

        let mut match_idx = match_idx;
        if match_idx == -1 {
            // Don't show match but original text.
            match_idx = 0;
            highlight = false;
        }
        // Length in screen cells; count 1 for the ending ">".
        let mut clen = wildmenu_match_len(xp, show_match(match_idx)) + 3;
        if match_idx == 0 {
            first_match.set(0);
        } else if match_idx < first_match.get() {
            // Jumping left, as far as we can go.
            first_match.set(match_idx);
            add_left = true;
        } else {
            // Check if match fits on the screen.
            i = first_match.get();
            while i < match_idx {
                clen += wildmenu_match_len(xp, show_match(i)) + 2;
                i += 1;
            }
            if first_match.get() > 0 {
                clen += 2;
            }
            // Jumping right, put match at the left.
            if clen > Columns.get() {
                first_match.set(match_idx);
                // If showing the last match, we can add some on the left.
                clen = 2;
                i = match_idx;
                while i < num_matches {
                    clen += wildmenu_match_len(xp, show_match(i)) + 2;
                    if clen >= Columns.get() {
                        break;
                    }
                    i += 1;
                }
                if i == num_matches {
                    add_left = true;
                }
            }
        }
        if add_left {
            while first_match.get() > 0 {
                clen += wildmenu_match_len(xp, show_match(first_match.get() - 1)) + 2;
                if clen >= Columns.get() {
                    break;
                }
                first_match.set(first_match.get() - 1);
            }
        }

        let mut group: hlf_T = HLF_NONE;
        let fillchar = fillchar_status(&raw mut group, curwin.get());
        let attr = win_hl_attr(curwin.get(), group as c_int);

        let mut len;
        if first_match.get() == 0 {
            *buf = NUL as c_char;
            len = 0;
        } else {
            strcpy(buf, c"< ".as_ptr());
            len = 2;
        }
        clen = len;

        i = first_match.get();
        while clen + wildmenu_match_len(xp, show_match(i)) + 2 < Columns.get() {
            if i == match_idx {
                selstart = buf.offset(len as isize);
                selstart_col = clen;
            }

            let mut s = show_match(i);
            // Check for menu separators - replace with '|'.
            let emenu = (*xp).xp_context == EXPAND_MENUS || (*xp).xp_context == EXPAND_MENUNAMES;
            if emenu && menu_is_separator(s) {
                strcpy(buf.offset(len as isize), transchar('|' as c_int));
                l = strlen(buf.offset(len as isize)) as c_int;
                len += l;
                clen += l;
            } else {
                while *s as c_int != NUL {
                    s = s.add(skip_wildmenu_char(xp, s) as usize);
                    clen += ptr2cells(s);
                    l = utfc_ptr2len(s);
                    if l > 1 {
                        strncpy(buf.offset(len as isize), s, l as size_t);
                        s = s.add(l as usize - 1);
                        len += l;
                    } else {
                        strcpy(buf.offset(len as isize), transchar_byte(*s as u8 as c_int));
                        len += strlen(buf.offset(len as isize)) as c_int;
                    }
                    s = s.add(1);
                }
            }
            if i == match_idx {
                selend = buf.offset(len as isize);
            }

            *buf.offset(len as isize) = ' ' as c_char;
            len += 1;
            *buf.offset(len as isize) = ' ' as c_char;
            len += 1;
            clen += 2;
            i += 1;
            if i == num_matches {
                break;
            }
        }

        if i != num_matches {
            *buf.offset(len as isize) = '>' as c_char;
            len += 1;
            clen += 1;
        }

        *buf.offset(len as isize) = NUL as c_char;

        let mut row = cmdline_row.get() - 1;
        if row >= 0 {
            if wild_menu_showing.get() == 0 {
                if msg_scrolled.get() > 0 {
                    // Put the wildmenu just above the command line.  If there
                    // is no room, scroll the screen one line up.
                    if cmdline_row.get() == Rows.get() - 1 {
                        msg_scroll_up(false, false);
                        (*msg_scrolled.ptr()) += 1;
                    } else {
                        (*cmdline_row.ptr()) += 1;
                        row += 1;
                    }
                    wild_menu_showing.set(WM_SCROLLED);
                } else {
                    // Create status line if needed by setting 'laststatus' to
                    // 2.  Set 'winminheight' to zero to avoid that the window
                    // is resized.
                    if (*lastwin.get()).w_status_height == 0 && global_stl_height() == 0 {
                        save_p_ls.set(p_ls.get() as c_int);
                        save_p_wmh.set(p_wmh.get() as c_int);
                        p_ls.set(2 as OptInt);
                        p_wmh.set(0 as OptInt);
                        last_status(false);
                    }
                    wild_menu_showing.set(WM_SHOWN);
                }
            }

            // Tricky: the wildmenu can be drawn either over a status line, or
            // at empty scrolled space in the message output.
            grid_line_start(
                if wild_menu_showing.get() == WM_SCROLLED {
                    msg_grid_adj.ptr()
                } else {
                    default_gridview.ptr()
                },
                row,
            );

            grid_line_puts(0, buf, -1, attr);
            if !selstart.is_null() && highlight {
                *selend = NUL as c_char;
                grid_line_puts(
                    selstart_col,
                    selstart,
                    -1,
                    *(*hl_attr_active.ptr()).offset(HLF_WM as isize),
                );
            }

            grid_line_fill(clen, Columns.get(), fillchar, attr);

            grid_line_flush();
        }

        win_redraw_last_status(topframe.get());
        xfree(buf as *mut c_void);
    }
}
