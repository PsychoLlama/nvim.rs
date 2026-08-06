//! Rebuilding a command *string* from the parsed pieces.
//!
//! `build_cmdline_str` is what `nvim_cmd` hands `execute_cmd` for the paths
//! that still want text: it writes the modifiers back in their canonical
//! order, then the range, the command name, the bang, the register and
//! each argument, recording where each one landed so `eap->args` can point
//! into the finished buffer.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn string_iswhite(mut str: String_0) -> bool {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < str.size {
            if !ascii_iswhite(*str.data.offset(i as isize) as ::core::ffi::c_int) {
                return false_0 != 0;
            } else {
                if *str.data.offset(i as isize) as ::core::ffi::c_int == NUL {
                    break;
                }
                i = i.wrapping_add(1);
            }
        }
        return true_0 != 0;
    }
}

/// Append `len` bytes to a [`StringBuilder`], growing it to the next power
/// of two when they do not fit: upstream's `kv_concat_len(cmdline, src,
/// len)`.  c2rust expanded that macro at all twenty-four of
/// [`build_cmdline_str`]'s call sites, ~40 lines apiece.
///
/// # Safety
/// `cmdline` points at a live builder and `src` at `len` readable bytes.
unsafe fn cmdline_concat(
    cmdline: *mut StringBuilder,
    src: *const ::core::ffi::c_char,
    len: size_t,
) {
    unsafe {
        if len == 0 as size_t {
            return;
        }
        if (*cmdline).capacity < (*cmdline).size.wrapping_add(len) {
            let mut capacity: size_t = (*cmdline).size.wrapping_add(len);
            capacity = capacity.wrapping_sub(1);
            capacity |= capacity >> 1 as ::core::ffi::c_int;
            capacity |= capacity >> 2 as ::core::ffi::c_int;
            capacity |= capacity >> 4 as ::core::ffi::c_int;
            capacity |= capacity >> 8 as ::core::ffi::c_int;
            capacity |= capacity >> 16 as ::core::ffi::c_int;
            (*cmdline).capacity = capacity.wrapping_add(1);
            (*cmdline).items = xrealloc(
                (*cmdline).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*cmdline).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        debug_assert!(!(*cmdline).items.is_null());
        memcpy(
            (*cmdline).items.offset((*cmdline).size as isize) as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(len),
        );
        (*cmdline).size = (*cmdline).size.wrapping_add(len);
    }
}

/// [`cmdline_concat`] for a string literal: upstream's `kv_concat`.
///
/// # Safety
/// `cmdline` points at a live builder.
unsafe fn cmdline_concat_str(cmdline: *mut StringBuilder, s: &::core::ffi::CStr) {
    unsafe { cmdline_concat(cmdline, s.as_ptr(), s.count_bytes()) }
}

pub(crate) unsafe extern "C" fn build_cmdline_str(
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut args: Array,
) {
    unsafe {
        let mut argc: size_t = args.size;
        let mut cmdline: StringBuilder = StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        cmdline.capacity = 32 as size_t;
        cmdline.items = xrealloc(
            cmdline.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
        ) as *mut ::core::ffi::c_char;
        if (*cmdinfo).cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%dtab \0".as_ptr() as *const ::core::ffi::c_char,
                (*cmdinfo).cmdmod.cmod_tab - 1 as ::core::ffi::c_int,
            );
        }
        if (*cmdinfo).cmdmod.cmod_verbose > 0 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%dverbose \0".as_ptr() as *const ::core::ffi::c_char,
                (*cmdinfo).cmdmod.cmod_verbose - 1 as ::core::ffi::c_int,
            );
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"silent! ");
        } else if (*cmdinfo).cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"silent ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"unsilent ");
        }
        match (*cmdinfo).cmdmod.cmod_split
            & (WSP_ABOVE as ::core::ffi::c_int
                | WSP_BELOW as ::core::ffi::c_int
                | WSP_TOP as ::core::ffi::c_int
                | WSP_BOT as ::core::ffi::c_int)
        {
            128 => {
                cmdline_concat_str(&raw mut cmdline, c"aboveleft ");
            }
            64 => {
                cmdline_concat_str(&raw mut cmdline, c"belowright ");
            }
            8 => {
                cmdline_concat_str(&raw mut cmdline, c"topleft ");
            }
            16 => {
                cmdline_concat_str(&raw mut cmdline, c"botright ");
            }
            _ => {}
        }
        if (*cmdinfo).cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"vertical ");
        }
        if (*cmdinfo).cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"horizontal ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"sandbox ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"noautocmd ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"browse ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"confirm ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"hide ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"keepalt ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"keepjumps ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"keepmarks ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"keeppatterns ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"lockmarks ");
        }
        if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
            cmdline_concat_str(&raw mut cmdline, c"noswapfile ");
        }
        if (*eap).argt & EX_RANGE as uint32_t != 0 {
            if (*eap).addr_count == 1 as ::core::ffi::c_int {
                kv_do_printf(
                    &raw mut cmdline,
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*eap).line2,
                );
            } else if (*eap).addr_count > 1 as ::core::ffi::c_int {
                kv_do_printf(
                    &raw mut cmdline,
                    b"%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*eap).line1,
                    (*eap).line2,
                );
                (*eap).addr_count = 2 as ::core::ffi::c_int;
            }
        }
        let mut cmdname_idx: size_t = cmdline.size;
        cmdline_concat(&raw mut cmdline, (*eap).cmd, strlen((*eap).cmd));
        if (*eap).argt & EX_BANG as uint32_t != 0 && (*eap).forceit != 0 {
            cmdline_concat_str(&raw mut cmdline, c"!");
        }
        if (*eap).argt & EX_REGSTR as uint32_t != 0 && (*eap).regname != 0 {
            kv_do_printf(
                &raw mut cmdline,
                b" %c\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).regname,
            );
        }
        (*eap).argc = argc;
        (*eap).arglens = (if (*eap).argc > 0 as size_t {
            xcalloc(argc, ::core::mem::size_of::<size_t>())
        } else {
            NULL
        }) as *mut size_t;
        let mut argstart_idx: size_t = cmdline.size;
        let mut i: size_t = 0 as size_t;
        while i < argc {
            let mut s: String_0 = (*args.items.offset(i as isize)).data.string;
            *(*eap).arglens.offset(i as isize) = s.size;
            cmdline_concat_str(&raw mut cmdline, c" ");
            cmdline_concat(&raw mut cmdline, s.data, s.size);
            i = i.wrapping_add(1);
        }
        if cmdline.size == cmdline.capacity {
            cmdline.capacity = if cmdline.capacity != 0 {
                cmdline.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            cmdline.items = xrealloc(
                cmdline.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
            ) as *mut ::core::ffi::c_char;
        } else {
        };
        let c2rust_fresh33 = cmdline.size;
        cmdline.size = cmdline.size.wrapping_add(1);
        *cmdline.items.offset(c2rust_fresh33 as isize) = '\0' as ::core::ffi::c_char;
        (*eap).cmd = cmdline.items.offset(cmdname_idx as isize);
        (*eap).args = (if (*eap).argc > 0 as size_t {
            xcalloc(argc, ::core::mem::size_of::<*mut ::core::ffi::c_char>())
        } else {
            NULL
        }) as *mut *mut ::core::ffi::c_char;
        let mut offset: size_t = argstart_idx;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < argc {
            offset = offset.wrapping_add(1);
            *(*eap).args.offset(i_0 as isize) = cmdline.items.offset(offset as isize);
            offset = offset.wrapping_add(*(*eap).arglens.offset(i_0 as isize));
            i_0 = i_0.wrapping_add(1);
        }
        (*eap).arg = if argc > 0 as size_t {
            *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
        } else {
            cmdline
                .items
                .offset(cmdline.size as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
        };
        *cmdlinep = cmdline.items;
        let mut p: *mut ::core::ffi::c_char = replace_makeprg(eap, (*eap).arg, cmdlinep);
        if p != (*eap).arg {
            (*eap).arg = p;
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*eap).args as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*eap).arglens as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
            (*eap).argc = 0 as size_t;
        }
    }
}
