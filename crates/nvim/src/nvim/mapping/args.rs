//! Parsing `:map` arguments into a [`MapArguments`].
//!
//! [`str_to_mapargs`] splits `<buffer><expr>… {lhs} {rhs}` and hands both
//! halves to [`set_maparg_lhs_rhs`], which runs them through
//! `replace_termcodes`.  [`get_map_mode`] and [`get_map_mode_string`] are the
//! other direction of the same question — which modes a command name or a
//! `maparg()`-style mode string names.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn map_mode_to_chars(
    mut mode: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = buf;
        if mode & (MODE_INSERT | MODE_CMDLINE) == MODE_INSERT | MODE_CMDLINE {
            let c2rust_fresh0 = p;
            p = p.offset(1);
            *c2rust_fresh0 = '!' as ::core::ffi::c_char;
        } else if mode & MODE_INSERT != 0 {
            let c2rust_fresh1 = p;
            p = p.offset(1);
            *c2rust_fresh1 = 'i' as ::core::ffi::c_char;
        } else if mode & MODE_LANGMAP != 0 {
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = 'l' as ::core::ffi::c_char;
        } else if mode & MODE_CMDLINE != 0 {
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 = 'c' as ::core::ffi::c_char;
        } else if mode & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING)
            == MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
        {
            let c2rust_fresh4 = p;
            p = p.offset(1);
            *c2rust_fresh4 = ' ' as ::core::ffi::c_char;
        } else {
            if mode & MODE_NORMAL != 0 {
                let c2rust_fresh5 = p;
                p = p.offset(1);
                *c2rust_fresh5 = 'n' as ::core::ffi::c_char;
            }
            if mode & MODE_OP_PENDING != 0 {
                let c2rust_fresh6 = p;
                p = p.offset(1);
                *c2rust_fresh6 = 'o' as ::core::ffi::c_char;
            }
            if mode & MODE_TERMINAL != 0 {
                let c2rust_fresh7 = p;
                p = p.offset(1);
                *c2rust_fresh7 = 't' as ::core::ffi::c_char;
            }
            if mode & (MODE_VISUAL | MODE_SELECT) == MODE_VISUAL | MODE_SELECT {
                let c2rust_fresh8 = p;
                p = p.offset(1);
                *c2rust_fresh8 = 'v' as ::core::ffi::c_char;
            } else {
                if mode & MODE_VISUAL != 0 {
                    let c2rust_fresh9 = p;
                    p = p.offset(1);
                    *c2rust_fresh9 = 'x' as ::core::ffi::c_char;
                }
                if mode & MODE_SELECT != 0 {
                    let c2rust_fresh10 = p;
                    p = p.offset(1);
                    *c2rust_fresh10 = 's' as ::core::ffi::c_char;
                }
            }
        }
        *p = NUL as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn set_maparg_lhs_rhs(
    orig_lhs: *const ::core::ffi::c_char,
    orig_lhs_len: size_t,
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) -> bool {
    unsafe {
        (*mapargs).rhs_lua = rhs_lua;
        let mut lhs_buf: [::core::ffi::c_char; 128] = [0; 128];
        let mut did_simplify: bool = false_0 != 0;
        let flags: ::core::ffi::c_int =
            REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
        let mut bufarg: *mut ::core::ffi::c_char = &raw mut lhs_buf as *mut ::core::ffi::c_char;
        let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
            orig_lhs,
            orig_lhs_len,
            &raw mut bufarg,
            0 as scid_T,
            flags,
            &raw mut did_simplify,
            cpo_val,
        );
        if replaced.is_null() {
            return false_0 != 0;
        }
        (*mapargs).lhs_len = strlen(replaced);
        xstrlcpy(
            &raw mut (*mapargs).lhs as *mut ::core::ffi::c_char,
            replaced,
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
        );
        if did_simplify {
            replaced = replace_termcodes(
                orig_lhs,
                orig_lhs_len,
                &raw mut bufarg,
                0 as scid_T,
                flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
                ::core::ptr::null_mut::<bool>(),
                cpo_val,
            );
            if replaced.is_null() {
                return false_0 != 0;
            }
            (*mapargs).alt_lhs_len = strlen(replaced);
            xstrlcpy(
                &raw mut (*mapargs).alt_lhs as *mut ::core::ffi::c_char,
                replaced,
                ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
            );
        } else {
            (*mapargs).alt_lhs_len = 0 as size_t;
        }
        set_maparg_rhs(
            orig_rhs,
            orig_rhs_len,
            rhs_lua,
            0 as scid_T,
            cpo_val,
            mapargs,
        );
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn set_maparg_rhs(
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    sid: scid_T,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) {
    unsafe {
        (*mapargs).rhs_lua = rhs_lua;
        if rhs_lua == LUA_NOREF {
            (*mapargs).orig_rhs_len = orig_rhs_len;
            (*mapargs).orig_rhs = xcalloc(
                (*mapargs).orig_rhs_len.wrapping_add(1 as size_t),
                ::core::mem::size_of::<::core::ffi::c_char>(),
            ) as *mut ::core::ffi::c_char;
            xmemcpyz(
                (*mapargs).orig_rhs as *mut ::core::ffi::c_void,
                orig_rhs as *const ::core::ffi::c_void,
                (*mapargs).orig_rhs_len,
            );
            if strcasecmp(
                orig_rhs as *mut ::core::ffi::c_char,
                b"<nop>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*mapargs).rhs = xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
                    as *mut ::core::ffi::c_char;
                (*mapargs).rhs_len = 0 as size_t;
                (*mapargs).rhs_is_noop = true_0 != 0;
            } else {
                let mut rhs_buf: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
                    orig_rhs,
                    orig_rhs_len,
                    &raw mut rhs_buf,
                    sid,
                    REPTERM_DO_LT as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<bool>(),
                    cpo_val,
                );
                (*mapargs).rhs_len = strlen(replaced);
                (*mapargs).rhs_is_noop =
                    orig_rhs_len != 0 as size_t && (*mapargs).rhs_len == 0 as size_t;
                (*mapargs).rhs = replaced;
            }
        } else {
            let mut tmp_buf: [::core::ffi::c_char; 64] = [0; 64];
            (*mapargs).orig_rhs =
                xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
                    as *mut ::core::ffi::c_char;
            (*mapargs).orig_rhs_len = 0 as size_t;
            (*mapargs).rhs_len = vim_snprintf(
                &raw mut tmp_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as size_t),
                b"%c%c%c%d\r\0".as_ptr() as *const ::core::ffi::c_char,
                K_SPECIAL,
                KS_EXTRA,
                KE_LUA as ::core::ffi::c_int,
                rhs_lua,
            ) as size_t;
            (*mapargs).rhs = xstrdup(&raw mut tmp_buf as *mut ::core::ffi::c_char);
        };
    }
}

pub(crate) unsafe extern "C" fn str_to_mapargs(
    mut strargs: *const ::core::ffi::c_char,
    mut is_unmap: bool,
    mut mapargs: *mut MapArguments,
) -> ::core::ffi::c_int {
    unsafe {
        let mut to_parse: *const ::core::ffi::c_char = strargs;
        to_parse = skipwhite(to_parse);
        memset(
            mapargs as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<MapArguments>(),
        );
        loop {
            if strncmp(
                to_parse,
                b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
                (*mapargs).buffer = true_0 != 0;
            } else if strncmp(
                to_parse,
                b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
                (*mapargs).nowait = true_0 != 0;
            } else if strncmp(
                to_parse,
                b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
                (*mapargs).silent = true_0 != 0;
            } else if strncmp(
                to_parse,
                b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(9 as ::core::ffi::c_int as isize));
            } else if strncmp(
                to_parse,
                b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
                (*mapargs).script = true_0 != 0;
            } else if strncmp(
                to_parse,
                b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_parse = skipwhite(to_parse.offset(6 as ::core::ffi::c_int as isize));
                (*mapargs).expr = true_0 != 0;
            } else {
                if strncmp(
                    to_parse,
                    b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
                to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
                (*mapargs).unique = true_0 != 0;
            }
        }
        let mut lhs_end: *const ::core::ffi::c_char = to_parse;
        let mut do_backslash: bool = vim_strchr(p_cpo.get(), CPO_BSLASH).is_null();
        while *lhs_end as ::core::ffi::c_int != 0
            && (is_unmap as ::core::ffi::c_int != 0
                || !ascii_iswhite(*lhs_end as ::core::ffi::c_int))
        {
            if (*lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == Ctrl_V
                || do_backslash as ::core::ffi::c_int != 0
                    && *lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
                && *lhs_end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                lhs_end = lhs_end.offset(1);
            }
            lhs_end = lhs_end.offset(1);
        }
        let mut rhs_start: *const ::core::ffi::c_char = skipwhite(lhs_end);
        let mut orig_lhs_len: size_t = lhs_end.offset_from(to_parse) as size_t;
        if orig_lhs_len >= 256 as size_t {
            return 1 as ::core::ffi::c_int;
        }
        let mut lhs_to_replace: [::core::ffi::c_char; 256] = [0; 256];
        xmemcpyz(
            &raw mut lhs_to_replace as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            to_parse as *const ::core::ffi::c_void,
            orig_lhs_len,
        );
        let mut orig_rhs_len: size_t = strlen(rhs_start);
        if !set_maparg_lhs_rhs(
            &raw mut lhs_to_replace as *mut ::core::ffi::c_char,
            orig_lhs_len,
            rhs_start,
            orig_rhs_len,
            LUA_NOREF,
            p_cpo.get(),
            mapargs,
        ) {
            return 1 as ::core::ffi::c_int;
        }
        if (*mapargs).lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn get_map_mode(
    mut cmdp: *mut *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut mode: ::core::ffi::c_int = 0;
        let mut p: *mut ::core::ffi::c_char = *cmdp;
        let c2rust_fresh11 = p;
        p = p.offset(1);
        let mut modec: ::core::ffi::c_int = *c2rust_fresh11 as uint8_t as ::core::ffi::c_int;
        if modec == 'i' as ::core::ffi::c_int {
            mode = MODE_INSERT;
        } else if modec == 'l' as ::core::ffi::c_int {
            mode = MODE_LANGMAP;
        } else if modec == 'c' as ::core::ffi::c_int {
            mode = MODE_CMDLINE;
        } else if modec == 'n' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
        {
            mode = MODE_NORMAL;
        } else if modec == 'v' as ::core::ffi::c_int {
            mode = MODE_VISUAL | MODE_SELECT;
        } else if modec == 'x' as ::core::ffi::c_int {
            mode = MODE_VISUAL;
        } else if modec == 's' as ::core::ffi::c_int {
            mode = MODE_SELECT;
        } else if modec == 'o' as ::core::ffi::c_int {
            mode = MODE_OP_PENDING;
        } else if modec == 't' as ::core::ffi::c_int {
            mode = MODE_TERMINAL;
        } else {
            p = p.offset(-1);
            if forceit {
                mode = MODE_INSERT | MODE_CMDLINE;
            } else {
                mode = MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
            }
        }
        *cmdp = p;
        return mode;
    }
}

pub(crate) unsafe extern "C" fn get_map_mode_string(
    mode_string: *const ::core::ffi::c_char,
    abbr: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = mode_string;
        let MASK_V: ::core::ffi::c_int = MODE_VISUAL | MODE_SELECT;
        let MASK_MAP: ::core::ffi::c_int =
            MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
        let MASK_BANG: ::core::ffi::c_int = MODE_INSERT | MODE_CMDLINE;
        if *p as ::core::ffi::c_int == NUL {
            p = b" \0".as_ptr() as *const ::core::ffi::c_char;
        }
        let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut modec: ::core::ffi::c_int = 0;
        loop {
            let c2rust_fresh20 = p;
            p = p.offset(1);
            modec = *c2rust_fresh20 as uint8_t as ::core::ffi::c_int;
            if modec == 0 {
                break;
            }
            let mut tmode: ::core::ffi::c_int = 0;
            match modec {
                105 => {
                    tmode = MODE_INSERT;
                }
                108 => {
                    tmode = MODE_LANGMAP;
                }
                99 => {
                    tmode = MODE_CMDLINE;
                }
                110 => {
                    tmode = MODE_NORMAL;
                }
                120 => {
                    tmode = MODE_VISUAL;
                }
                115 => {
                    tmode = MODE_SELECT;
                }
                111 => {
                    tmode = MODE_OP_PENDING;
                }
                116 => {
                    tmode = MODE_TERMINAL;
                }
                118 => {
                    tmode = MASK_V;
                }
                33 => {
                    tmode = MASK_BANG;
                }
                32 => {
                    tmode = MASK_MAP;
                }
                _ => return 0 as ::core::ffi::c_int,
            }
            mode |= tmode;
        }
        if abbr as ::core::ffi::c_int != 0 && mode & !MASK_BANG != 0 as ::core::ffi::c_int
            || !abbr
                && mode & mode - 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                && !(mode & MASK_BANG != 0 as ::core::ffi::c_int
                    && mode & !MASK_BANG == 0 as ::core::ffi::c_int
                    || mode & MASK_MAP != 0 as ::core::ffi::c_int
                        && mode & !MASK_MAP == 0 as ::core::ffi::c_int)
        {
            return 0 as ::core::ffi::c_int;
        }
        return mode;
    }
}
