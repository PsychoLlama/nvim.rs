//! The wildcard key press.
//!
//! [`command_line_wildchar_complete`] is what `<Tab>` (and `'wildchar'`)
//! reaches: it drives `'wildmode'` through [`check_opt_wim`], calls
//! `nextwild` once per configured stage, and decides whether the popup menu
//! or the wildmenu comes up.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn command_line_wildchar_complete(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    unsafe {
        let mut res: ::core::ffi::c_int = 0;
        let mut options: ::core::ffi::c_int = WILD_NO_BEEP as ::core::ffi::c_int;
        let mut escape: bool = (*s).firstc != '@' as ::core::ffi::c_int;
        let mut redraw_if_menu_empty: bool = (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        let mut wim_noselect: bool = p_wmnu.get() != 0
            && (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                & kOptWimFlagNoselect as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int;
        if (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
            & kOptWimFlagLastused as ::core::ffi::c_int
            != 0
        {
            options |= WILD_BUFLASTUSED as ::core::ffi::c_int;
        }
        if (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int {
            if (*s).xpc.xp_numfiles > 1 as ::core::ffi::c_int
                && !(*s).did_wild_list
                && (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
                    & kOptWimFlagList as ::core::ffi::c_int
                    != 0
            {
                showmatches(&raw mut (*s).xpc, false_0 != 0, true_0 != 0, wim_noselect);
                redrawcmd();
                (*s).did_wild_list = true_0 != 0;
            }
            if (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
                & kOptWimFlagLongest as ::core::ffi::c_int
                != 0
            {
                res = nextwild(
                    &raw mut (*s).xpc,
                    WILD_LONGEST as ::core::ffi::c_int,
                    options,
                    escape,
                );
            } else if (*wim_flags.ptr())[(*s).wim_index as usize] as ::core::ffi::c_int
                & kOptWimFlagFull as ::core::ffi::c_int
                != 0
            {
                res = nextwild(
                    &raw mut (*s).xpc,
                    WILD_NEXT as ::core::ffi::c_int,
                    options,
                    escape,
                );
            } else {
                res = OK;
            }
        } else {
            let mut wim_longest: bool = (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                & kOptWimFlagLongest as ::core::ffi::c_int
                != 0;
            let mut wim_list: bool = (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                & kOptWimFlagList as ::core::ffi::c_int
                != 0;
            let mut wim_full: bool = (*wim_flags.ptr())[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                & kOptWimFlagFull as ::core::ffi::c_int
                != 0;
            (*s).wim_index = 0 as ::core::ffi::c_int;
            if (*s).c as OptInt == p_wc.get()
                || (*s).c as OptInt == p_wcm.get()
                || (*s).c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || (*s).c == Ctrl_Z
            {
                options |= WILD_MAY_EXPAND_PATTERN as ::core::ffi::c_int;
                if (*s).c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    options |= WILD_FUNC_TRIGGER as ::core::ffi::c_int;
                }
                (*s).xpc.xp_pre_incsearch_pos = (*s).is_state.search_start;
            }
            let mut cmdpos_before: ::core::ffi::c_int = (*ccline.ptr()).cmdpos;
            if wim_longest {
                res = nextwild(
                    &raw mut (*s).xpc,
                    WILD_LONGEST as ::core::ffi::c_int,
                    options,
                    escape,
                );
            } else {
                if wim_noselect as ::core::ffi::c_int != 0 || wim_list as ::core::ffi::c_int != 0 {
                    options |= WILD_NOSELECT as ::core::ffi::c_int;
                }
                res = nextwild(
                    &raw mut (*s).xpc,
                    WILD_EXPAND_KEEP as ::core::ffi::c_int,
                    options,
                    escape,
                );
            }
            if redraw_if_menu_empty as ::core::ffi::c_int != 0
                && (*s).xpc.xp_numfiles <= 0 as ::core::ffi::c_int
            {
                pum_check_clear();
            }
            if got_int.get() {
                vpeekc();
                got_int.set(false_0 != 0);
                ExpandOne(
                    &raw mut (*s).xpc,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                    WILD_FREE as ::core::ffi::c_int,
                );
                (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                return CMDLINE_CHANGED as ::core::ffi::c_int;
            }
            if res == OK
                && (*s).xpc.xp_numfiles
                    > (if wim_noselect as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    })
            {
                if wim_longest {
                    let mut found_longest_prefix: bool = (*ccline.ptr()).cmdpos != cmdpos_before;
                    if wim_list as ::core::ffi::c_int != 0
                        || p_wmnu.get() != 0 && wim_full as ::core::ffi::c_int != 0
                    {
                        showmatches(&raw mut (*s).xpc, p_wmnu.get() != 0, wim_list, true_0 != 0);
                    } else if !found_longest_prefix {
                        let mut wim_list_next: bool = (*wim_flags.ptr())
                            [1 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int
                            & kOptWimFlagList as ::core::ffi::c_int
                            != 0;
                        let mut wim_full_next: bool = (*wim_flags.ptr())
                            [1 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int
                            & kOptWimFlagFull as ::core::ffi::c_int
                            != 0;
                        let mut wim_noselect_next: bool = (*wim_flags.ptr())
                            [1 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int
                            & kOptWimFlagNoselect as ::core::ffi::c_int
                            != 0;
                        if wim_list_next as ::core::ffi::c_int != 0
                            || p_wmnu.get() != 0
                                && (wim_full_next as ::core::ffi::c_int != 0
                                    || wim_noselect_next as ::core::ffi::c_int != 0)
                        {
                            if wim_full_next as ::core::ffi::c_int != 0 && !wim_noselect_next {
                                nextwild(
                                    &raw mut (*s).xpc,
                                    WILD_NEXT as ::core::ffi::c_int,
                                    options,
                                    escape,
                                );
                            } else {
                                showmatches(
                                    &raw mut (*s).xpc,
                                    p_wmnu.get() != 0,
                                    wim_list_next,
                                    wim_noselect_next,
                                );
                            }
                            if wim_list_next {
                                (*s).did_wild_list = true_0 != 0;
                            }
                        }
                    }
                } else if wim_list as ::core::ffi::c_int != 0
                    || p_wmnu.get() != 0
                        && (wim_full as ::core::ffi::c_int != 0
                            || wim_noselect as ::core::ffi::c_int != 0)
                {
                    showmatches(&raw mut (*s).xpc, p_wmnu.get() != 0, wim_list, wim_noselect);
                } else {
                    vim_beep(kOptBoFlagWildmode as ::core::ffi::c_int as ::core::ffi::c_uint);
                }
                redrawcmd();
                if wim_list {
                    (*s).did_wild_list = true_0 != 0;
                }
            } else if (*s).xpc.xp_numfiles == -1 as ::core::ffi::c_int {
                (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
        if (*s).wim_index < 3 as ::core::ffi::c_int {
            (*s).wim_index += 1;
        }
        if (*s).c == ESC {
            (*s).gotesc = true_0 != 0;
        }
        return if res == OK {
            CMDLINE_CHANGED as ::core::ffi::c_int
        } else {
            CMDLINE_NOT_CHANGED as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn check_opt_wim() -> ::core::ffi::c_int {
    unsafe {
        let mut new_wim_flags: [uint8_t; 4] = [0; 4];
        let mut i: ::core::ffi::c_int = 0;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        i = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            new_wim_flags[i as usize] = 0 as uint8_t;
            i += 1;
        }
        let mut p: *mut ::core::ffi::c_char = p_wim.get();
        while *p != 0 {
            i = 0 as ::core::ffi::c_int;
            while *p.offset(i as isize) as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p.offset(i as isize) as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p.offset(i as isize) as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p.offset(i as isize) as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            {
                i += 1;
            }
            if *p.offset(i as isize) as ::core::ffi::c_int != NUL
                && *p.offset(i as isize) as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *p.offset(i as isize) as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            {
                return FAIL;
            }
            if i == 7 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"longest\0".as_ptr() as *const ::core::ffi::c_char,
                    7 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                    | kOptWimFlagLongest as ::core::ffi::c_int)
                    as uint8_t;
            } else if i == 4 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"full\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                    | kOptWimFlagFull as ::core::ffi::c_int)
                    as uint8_t;
            } else if i == 4 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"list\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                    | kOptWimFlagList as ::core::ffi::c_int)
                    as uint8_t;
            } else if i == 8 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                    | kOptWimFlagLastused as ::core::ffi::c_int)
                    as uint8_t;
            } else if i == 8 as ::core::ffi::c_int
                && strncmp(
                    p,
                    b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                    | kOptWimFlagNoselect as ::core::ffi::c_int)
                    as uint8_t;
            } else {
                return FAIL;
            }
            p = p.offset(i as isize);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                if idx == 3 as ::core::ffi::c_int {
                    return FAIL;
                }
                idx += 1;
            }
            p = p.offset(1);
        }
        while idx < 3 as ::core::ffi::c_int {
            new_wim_flags[(idx + 1 as ::core::ffi::c_int) as usize] = new_wim_flags[idx as usize];
            idx += 1;
        }
        i = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            (*wim_flags.ptr())[i as usize] = new_wim_flags[i as usize];
            i += 1;
        }
        return OK;
    }
}
