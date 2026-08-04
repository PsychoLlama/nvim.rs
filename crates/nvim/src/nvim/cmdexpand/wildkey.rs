//! The wildmenu's own key handling.
//!
//! While the wildmenu is up, some keys mean "move in the menu" rather than
//! what they usually mean.  [`wildmenu_translate_key`] does the remapping and
//! [`wildmenu_process_key`] applies it, with a different rule for menu names
//! than for file names.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn wildmenu_translate_key(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut did_wild_list: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = key;
        if cmdline_pum_active() as ::core::ffi::c_int != 0
            || did_wild_list as ::core::ffi::c_int != 0
            || wild_menu_showing.get() != 0
        {
            if c == K_LEFT {
                c = Ctrl_P;
            } else if c == K_RIGHT {
                c = Ctrl_N;
            }
        }
        if (*xp).xp_context == EXPAND_MENUNAMES
            && (*cclp).cmdpos > 1 as ::core::ffi::c_int
            && *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != '\\' as ::core::ffi::c_int
            && (c == '\n' as ::core::ffi::c_int || c == '\r' as ::core::ffi::c_int || c == K_KENTER)
        {
            c = K_DOWN;
        }
        return c;
    }
}

pub(crate) unsafe extern "C" fn cmdline_del(
    mut cclp: *mut CmdlineInfo,
    mut from: ::core::ffi::c_int,
) {
    unsafe {
        '_c2rust_label: {
            if (*cclp).cmdpos <= (*cclp).cmdlen {
            } else {
                __assert_fail(
                    b"cclp->cmdpos <= cclp->cmdlen\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3650 as ::core::ffi::c_uint,
                    b"void cmdline_del(CmdlineInfo *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            (*cclp).cmdbuff.offset(from as isize) as *mut ::core::ffi::c_void,
            (*cclp).cmdbuff.offset((*cclp).cmdpos as isize) as *const ::core::ffi::c_void,
            ((*cclp).cmdlen as size_t)
                .wrapping_sub((*cclp).cmdpos as size_t)
                .wrapping_add(1 as size_t),
        );
        (*cclp).cmdlen -= (*cclp).cmdpos - from;
        (*cclp).cmdpos = from;
    }
}

pub(crate) unsafe extern "C" fn wildmenu_process_key_menunames(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    unsafe {
        if key == K_DOWN
            && (*cclp).cmdpos > 0 as ::core::ffi::c_int
            && *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
        {
            key = p_wc.get() as ::core::ffi::c_int;
            KeyTyped.set(true_0 != 0);
        } else if key == K_UP {
            let mut found: bool = false_0 != 0;
            let mut j: ::core::ffi::c_int =
                (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            loop {
                j -= 1;
                if j <= 0 as ::core::ffi::c_int {
                    break;
                }
                if *(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
                    && *(*cclp)
                        .cmdbuff
                        .offset((j - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                {
                    i = j + 1 as ::core::ffi::c_int;
                    break;
                } else {
                    if !(*(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && *(*cclp)
                            .cmdbuff
                            .offset((j - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            != '\\' as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    if found {
                        i = j + 1 as ::core::ffi::c_int;
                        break;
                    } else {
                        found = true_0 != 0;
                    }
                }
            }
            if i > 0 as ::core::ffi::c_int {
                cmdline_del(cclp, i);
            }
            key = p_wc.get() as ::core::ffi::c_int;
            KeyTyped.set(true_0 != 0);
            (*xp).xp_context = EXPAND_NOTHING;
        }
        return key;
    }
}

pub(crate) unsafe extern "C" fn wildmenu_process_key_filenames(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut upseg: [::core::ffi::c_char; 5] = [0; 5];
        upseg[0 as ::core::ffi::c_int as usize] = PATHSEP as ::core::ffi::c_char;
        upseg[1 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
        upseg[2 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
        upseg[3 as ::core::ffi::c_int as usize] = PATHSEP as ::core::ffi::c_char;
        upseg[4 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        if key == K_DOWN
            && (*cclp).cmdpos > 0 as ::core::ffi::c_int
            && *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == PATHSEP
            && ((*cclp).cmdpos < 3 as ::core::ffi::c_int
                || *(*cclp)
                    .cmdbuff
                    .offset(((*cclp).cmdpos - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    != '.' as ::core::ffi::c_int
                || *(*cclp)
                    .cmdbuff
                    .offset(((*cclp).cmdpos - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    != '.' as ::core::ffi::c_int)
        {
            key = p_wc.get() as ::core::ffi::c_int;
            KeyTyped.set(true_0 != 0);
        } else if strncmp(
            (*xp).xp_pattern,
            (&raw mut upseg as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
            && key == K_DOWN
        {
            let mut found: bool = false_0 != 0;
            let mut j: ::core::ffi::c_int = (*cclp).cmdpos;
            let mut i: ::core::ffi::c_int =
                (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
            loop {
                j -= 1;
                if j <= i {
                    break;
                }
                j -= utf_head_off((*cclp).cmdbuff, (*cclp).cmdbuff.offset(j as isize));
                if !vim_ispathsep(*(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int) {
                    continue;
                }
                found = true_0 != 0;
                break;
            }
            if found as ::core::ffi::c_int != 0
                && *(*cclp)
                    .cmdbuff
                    .offset((j - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && *(*cclp)
                    .cmdbuff
                    .offset((j - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && (vim_ispathsep(
                    *(*cclp)
                        .cmdbuff
                        .offset((j - 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
                    || j == i + 2 as ::core::ffi::c_int)
            {
                cmdline_del(cclp, j - 2 as ::core::ffi::c_int);
                key = p_wc.get() as ::core::ffi::c_int;
                KeyTyped.set(true_0 != 0);
            }
        } else if key == K_UP {
            let mut found_0: bool = false_0 != 0;
            let mut j_0: ::core::ffi::c_int = (*cclp).cmdpos - 1 as ::core::ffi::c_int;
            let mut i_0: ::core::ffi::c_int =
                (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
            loop {
                j_0 -= 1;
                if j_0 <= i_0 {
                    break;
                }
                j_0 -= utf_head_off((*cclp).cmdbuff, (*cclp).cmdbuff.offset(j_0 as isize));
                if !vim_ispathsep(*(*cclp).cmdbuff.offset(j_0 as isize) as ::core::ffi::c_int) {
                    continue;
                }
                if found_0 {
                    i_0 = j_0 + 1 as ::core::ffi::c_int;
                    break;
                } else {
                    found_0 = true_0 != 0;
                }
            }
            if !found_0 {
                j_0 = i_0;
            } else if strncmp(
                (*cclp).cmdbuff.offset(j_0 as isize),
                &raw mut upseg as *mut ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                j_0 += 4 as ::core::ffi::c_int;
            } else if strncmp(
                (*cclp).cmdbuff.offset(j_0 as isize),
                (&raw mut upseg as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize),
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
                && j_0 == i_0
            {
                j_0 += 3 as ::core::ffi::c_int;
            } else {
                j_0 = 0 as ::core::ffi::c_int;
            }
            if j_0 > 0 as ::core::ffi::c_int {
                cmdline_del(cclp, j_0);
                put_on_cmdline(
                    (&raw mut upseg as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize),
                    3 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            } else if (*cclp).cmdpos > i_0 {
                cmdline_del(cclp, i_0);
            }
            key = p_wc.get() as ::core::ffi::c_int;
            KeyTyped.set(true_0 != 0);
        }
        return key;
    }
}

pub unsafe extern "C" fn wildmenu_process_key(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    unsafe {
        if (*xp).xp_context == EXPAND_MENUNAMES {
            return wildmenu_process_key_menunames(cclp, key, xp);
        }
        if (*xp).xp_context == EXPAND_FILES
            || (*xp).xp_context == EXPAND_DIRECTORIES
            || (*xp).xp_context == EXPAND_SHELLCMD
        {
            return wildmenu_process_key_filenames(cclp, key, xp);
        }
        return key;
    }
}

pub unsafe extern "C" fn wildmenu_cleanup(mut cclp: *mut CmdlineInfo) {
    unsafe {
        if p_wmnu.get() == 0 || wild_menu_showing.get() == 0 as ::core::ffi::c_int {
            return;
        }
        let skt: bool = KeyTyped.get();
        let old_RedrawingDisabled: ::core::ffi::c_int = RedrawingDisabled.get();
        if (*cclp).input_fn != 0 {
            RedrawingDisabled.set(0 as ::core::ffi::c_int);
        }
        set_no_hlsearch(true_0 != 0);
        if wild_menu_showing.get() == WM_SCROLLED {
            (*cmdline_row.ptr()) -= 1;
            redrawcmd();
            wild_menu_showing.set(0 as ::core::ffi::c_int);
        } else if save_p_ls.get() != -1 as ::core::ffi::c_int {
            p_ls.set(save_p_ls.get() as OptInt);
            p_wmh.set(save_p_wmh.get() as OptInt);
            last_status(false_0 != 0);
            update_screen();
            redrawcmd();
            save_p_ls.set(-1 as ::core::ffi::c_int);
            wild_menu_showing.set(0 as ::core::ffi::c_int);
        } else {
            win_redraw_last_status(topframe.get());
            wild_menu_showing.set(0 as ::core::ffi::c_int);
            redraw_statuslines();
        }
        KeyTyped.set(skt);
        if (*cclp).input_fn != 0 {
            RedrawingDisabled.set(old_RedrawingDisabled);
        }
    }
}
