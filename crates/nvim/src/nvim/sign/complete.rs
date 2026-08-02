//! Command-line completion for `:sign`.
//!
//! [`set_context_in_sign_cmd`] decides, from how much of the line has been
//! typed, which of seven things the word under the cursor is: a
//! subcommand, an argument name for one of the four subcommands that take
//! them, a defined sign name, a placed sign group, or something with a
//! completion of its own (a highlight group, a file, a buffer).
//! [`get_sign_name`] is the `ExpandGeneric` callback that then enumerates
//! whichever list that answer named.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_nth_sign_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut name: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
        let mut current_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*sign_map.ptr()).set.h.n_keys {
            name = *(*sign_map.ptr()).set.keys.offset(__i as isize);
            let c2rust_fresh9 = current_idx;
            current_idx = current_idx + 1;
            if c2rust_fresh9 == idx {
                return name as *mut ::core::ffi::c_char;
            }
            __i = __i.wrapping_add(1);
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn get_nth_sign_group_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if idx < (*sign_ns.ptr()).size as ::core::ffi::c_int {
            return describe_ns(
                *(*sign_ns.ptr()).items.offset(idx as isize) as NS,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            ) as *mut ::core::ffi::c_char;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn get_sign_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        match expand_what.get() as ::core::ffi::c_uint {
            0 => return (*cmds.ptr())[idx as usize],
            1 => {
                let mut define_arg: [*mut ::core::ffi::c_char; 8] = [
                    b"culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"icon=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"text=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"priority=\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ];
                return define_arg[idx as usize];
            }
            2 => {
                let mut place_arg: [*mut ::core::ffi::c_char; 7] = [
                    b"line=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"name=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"priority=\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ];
                return place_arg[idx as usize];
            }
            3 => {
                let mut list_arg: [*mut ::core::ffi::c_char; 4] = [
                    b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ];
                return list_arg[idx as usize];
            }
            4 => {
                let mut unplace_arg: [*mut ::core::ffi::c_char; 4] = [
                    b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ];
                return unplace_arg[idx as usize];
            }
            5 => return get_nth_sign_name(idx),
            6 => return get_nth_sign_group_name(idx),
            _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
    }
}

pub unsafe extern "C" fn set_context_in_sign_cmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
) {
    unsafe {
        (*xp).xp_context = EXPAND_SIGN as ::core::ffi::c_int;
        expand_what.set(EXP_SUBCMD);
        (*xp).xp_pattern = arg;
        let mut end_subcmd: *mut ::core::ffi::c_char = skiptowhite(arg);
        if *end_subcmd as ::core::ffi::c_int == NUL {
            return;
        }
        let mut cmd_idx: ::core::ffi::c_int = sign_cmd_idx(arg, end_subcmd);
        let mut begin_subcmd_args: *mut ::core::ffi::c_char = skipwhite(end_subcmd);
        let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = begin_subcmd_args;
        loop {
            p = skipwhite(p);
            last = p;
            p = skiptowhite(p);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        }
        p = vim_strchr(last, '=' as ::core::ffi::c_int);
        if p.is_null() {
            (*xp).xp_pattern = last;
            match cmd_idx {
                SIGNCMD_DEFINE => {
                    expand_what.set(EXP_DEFINE);
                }
                SIGNCMD_PLACE => {
                    if ascii_isdigit(*begin_subcmd_args as ::core::ffi::c_int) {
                        expand_what.set(EXP_PLACE);
                    } else {
                        expand_what.set(EXP_LIST);
                    }
                }
                SIGNCMD_LIST | SIGNCMD_UNDEFINE => {
                    expand_what.set(EXP_SIGN_NAMES);
                }
                SIGNCMD_JUMP | SIGNCMD_UNPLACE => {
                    expand_what.set(EXP_UNPLACE);
                }
                _ => {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
        } else {
            (*xp).xp_pattern = p.offset(1 as ::core::ffi::c_int as isize);
            match cmd_idx {
                SIGNCMD_DEFINE => {
                    if strncmp(
                        last,
                        b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            last,
                            b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
                            6 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            last,
                            b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        || strncmp(
                            last,
                            b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
                    } else if strncmp(
                        last,
                        b"icon\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
                    } else {
                        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                    }
                }
                SIGNCMD_PLACE => {
                    if strncmp(
                        last,
                        b"name\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        expand_what.set(EXP_SIGN_NAMES);
                    } else if strncmp(
                        last,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        expand_what.set(EXP_SIGN_GROUPS);
                    } else if strncmp(
                        last,
                        b"file\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                    } else {
                        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                    }
                }
                SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
                    if strncmp(
                        last,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        expand_what.set(EXP_SIGN_GROUPS);
                    } else if strncmp(
                        last,
                        b"file\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                    } else {
                        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                    }
                }
                _ => {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
        };
    }
}
