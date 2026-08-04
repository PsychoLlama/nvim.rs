//! Listing mappings: `:map` output and command-line completion.
//!
//! [`showmap`] prints one mapping in the four-column `:map` form.
//! [`translate_mapping`] is the same rendering for completion, which
//! [`ExpandMappings`] runs over the whole table for `:map <Tab>`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn showmap(mut mp: *mut mapblock_T, mut local: bool) {
    unsafe {
        if message_filtered((*mp).m_keys) as ::core::ffi::c_int != 0
            && message_filtered((*mp).m_str) as ::core::ffi::c_int != 0
            && ((*mp).m_desc.is_null() || message_filtered((*mp).m_desc) as ::core::ffi::c_int != 0)
        {
            return;
        }
        if msg_col.get() > 0 as ::core::ffi::c_int || msg_silent.get() != 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
            if got_int.get() {
                return;
            }
        }
        let mut mapchars: [::core::ffi::c_char; 7] = [0; 7];
        map_mode_to_chars((*mp).m_mode, &raw mut mapchars as *mut ::core::ffi::c_char);
        msg_puts(&raw mut mapchars as *mut ::core::ffi::c_char);
        let mut len: size_t = strlen(&raw mut mapchars as *mut ::core::ffi::c_char);
        loop {
            len = len.wrapping_add(1);
            if len > 3 as size_t {
                break;
            }
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        len = msg_outtrans_special((*mp).m_keys, true_0 != 0, 0 as ::core::ffi::c_int) as size_t;
        loop {
            msg_putchar(' ' as ::core::ffi::c_int);
            len = len.wrapping_add(1);
            if len >= 12 as size_t {
                break;
            }
        }
        if (*mp).m_noremap == REMAP_NONE as ::core::ffi::c_int {
            msg_puts_hl(
                b"*\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_8,
                false_0 != 0,
            );
        } else if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
            msg_puts_hl(
                b"&\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_8,
                false_0 != 0,
            );
        } else {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        if local {
            msg_putchar('@' as ::core::ffi::c_int);
        } else {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        if (*mp).m_luaref != LUA_NOREF {
            let mut str: *mut ::core::ffi::c_char =
                nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
            msg_puts_hl(str, HLF_8, false_0 != 0);
            xfree(str as *mut ::core::ffi::c_void);
        } else if *(*mp).m_str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            msg_puts_hl(
                b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_8,
                false_0 != 0,
            );
        } else {
            msg_outtrans_special((*mp).m_str, false_0 != 0, 0 as ::core::ffi::c_int);
        }
        if !(*mp).m_desc.is_null() {
            msg_puts(b"\n                 \0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts((*mp).m_desc);
        }
        if p_verbose.get() > 0 as OptInt {
            last_set_msg((*mp).m_script_ctx);
        }
        msg_clr_eos();
    }
}

pub(crate) unsafe extern "C" fn translate_mapping(
    str_in: *const ::core::ffi::c_char,
    cpo_val: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut str: *const uint8_t = str_in as *const uint8_t;
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            40 as ::core::ffi::c_int,
        );
        let cpo_bslash: bool = !vim_strchr(cpo_val, CPO_BSLASH).is_null();
        while *str != 0 {
            let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
            's_13: {
                if c == K_SPECIAL
                    && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_MODIFIER
                    {
                        str = str.offset(1);
                        str = str.offset(1);
                        modifiers = *str as ::core::ffi::c_int;
                        str = str.offset(1);
                        c = *str as ::core::ffi::c_int;
                    }
                    if c == K_SPECIAL
                        && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                        && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                    {
                        c = if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == KS_SPECIAL
                        {
                            K_SPECIAL
                        } else if *str.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == KS_ZERO
                        {
                            K_ZERO
                        } else {
                            -(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int))
                        };
                        if c == K_ZERO {
                            c = NUL;
                        }
                        str = str.offset(2 as ::core::ffi::c_int as isize);
                    }
                    if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                        ga_concat(&raw mut ga, get_special_key_name(c, modifiers));
                        break 's_13;
                    }
                }
                if c == ' ' as ::core::ffi::c_int
                    || c == '\t' as ::core::ffi::c_int
                    || c == Ctrl_J
                    || c == Ctrl_V
                    || c == '<' as ::core::ffi::c_int
                    || c == '\\' as ::core::ffi::c_int && !cpo_bslash
                {
                    ga_append(
                        &raw mut ga,
                        (if cpo_bslash as ::core::ffi::c_int != 0 {
                            Ctrl_V
                        } else {
                            '\\' as ::core::ffi::c_int
                        }) as uint8_t,
                    );
                }
                if c != 0 {
                    ga_append(&raw mut ga, c as uint8_t);
                }
            }
            str = str.offset(1);
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn set_context_in_map_cmd(
    mut xp: *mut expand_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
    mut isabbrev: bool,
    mut isunmap: bool,
    mut cmdidx: cmdidx_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if forceit as ::core::ffi::c_int != 0
            && cmdidx as ::core::ffi::c_int != CMD_map as ::core::ffi::c_int
            && cmdidx as ::core::ffi::c_int != CMD_unmap as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        } else {
            if isunmap {
                expand_mapmodes.set(get_map_mode(
                    &raw mut cmd,
                    forceit as ::core::ffi::c_int != 0 || isabbrev as ::core::ffi::c_int != 0,
                ));
            } else {
                expand_mapmodes.set(MODE_INSERT | MODE_CMDLINE);
                if !isabbrev {
                    (*expand_mapmodes.ptr()) |=
                        MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
                }
            }
            expand_isabbrev.set(isabbrev);
            (*xp).xp_context = EXPAND_MAPPINGS as ::core::ffi::c_int;
            expand_buffer.set(false_0 != 0);
            loop {
                if strncmp(
                    arg,
                    b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_buffer.set(true_0 != 0);
                    arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
                } else if strncmp(
                    arg,
                    b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
                } else if strncmp(
                    arg,
                    b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
                } else if strncmp(
                    arg,
                    b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
                } else if strncmp(
                    arg,
                    b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                    9 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
                } else if strncmp(
                    arg,
                    b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
                } else {
                    if strncmp(
                        arg,
                        b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
                }
            }
            (*xp).xp_pattern = arg;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn ExpandMappings(
    mut pat: *mut ::core::ffi::c_char,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let fuzzy: bool = cmdline_fuzzy_complete(pat);
        *numMatches = 0 as ::core::ffi::c_int;
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        if !fuzzy {
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        } else {
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 7 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            's_34: {
                if i == 0 as ::core::ffi::c_int {
                    p = b"<silent>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 1 as ::core::ffi::c_int {
                    p = b"<unique>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 2 as ::core::ffi::c_int {
                    p = b"<script>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 3 as ::core::ffi::c_int {
                    p = b"<expr>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 4 as ::core::ffi::c_int && !expand_buffer.get() {
                    p = b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 5 as ::core::ffi::c_int {
                    p = b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else if i == 6 as ::core::ffi::c_int {
                    p = b"<special>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    break 's_34;
                }
                let mut match_0: bool = false;
                let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if !fuzzy {
                    match_0 = vim_regexec(regmatch, p, 0 as colnr_T);
                } else {
                    score = fuzzy_match_str(p, pat);
                    match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                }
                if match_0 {
                    if fuzzy {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                            fuzmatch_str_T {
                                idx: ga.ga_len,
                                str: xstrdup(p),
                                score: score,
                            };
                        ga.ga_len += 1;
                    } else {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                            xstrdup(p);
                        ga.ga_len += 1;
                    }
                }
            }
            i += 1;
        }
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if expand_isabbrev.get() {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                } else {
                    mp = FIRST_ABBR.get();
                }
            } else if expand_buffer.get() {
                mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = (*MAPHASH.ptr())[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if !((*mp).m_simplified != 0 || (*mp).m_mode & expand_mapmodes.get() == 0) {
                    let mut p_0: *mut ::core::ffi::c_char =
                        translate_mapping((*mp).m_keys, p_cpo.get());
                    if !p_0.is_null() {
                        let mut match_1: bool = false;
                        let mut score_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if !fuzzy {
                            match_1 = vim_regexec(regmatch, p_0, 0 as colnr_T);
                        } else {
                            score_0 = fuzzy_match_str(p_0, pat);
                            match_1 = score_0 != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                        }
                        if !match_1 {
                            xfree(p_0 as *mut ::core::ffi::c_void);
                        } else if fuzzy {
                            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                            *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                                fuzmatch_str_T {
                                    idx: ga.ga_len,
                                    str: p_0,
                                    score: score_0,
                                };
                            ga.ga_len += 1;
                        } else {
                            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                            *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                                .offset(ga.ga_len as isize) = p_0;
                            ga.ga_len += 1;
                        }
                    }
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        if ga.ga_len == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        if !fuzzy {
            *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
            *numMatches = ga.ga_len;
        } else {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                false_0 != 0,
            );
            *numMatches = ga.ga_len;
        }
        let mut count: ::core::ffi::c_int = *numMatches;
        if count > 1 as ::core::ffi::c_int {
            if !fuzzy {
                sort_strings(*matches, count);
            }
            let mut ptr1: *mut *mut ::core::ffi::c_char = *matches;
            let mut ptr2: *mut *mut ::core::ffi::c_char =
                ptr1.offset(1 as ::core::ffi::c_int as isize);
            let mut ptr3: *mut *mut ::core::ffi::c_char = ptr1.offset(count as isize);
            while ptr2 < ptr3 {
                if strcmp(*ptr1, *ptr2) != 0 as ::core::ffi::c_int {
                    let c2rust_fresh12 = ptr2;
                    ptr2 = ptr2.offset(1);
                    ptr1 = ptr1.offset(1);
                    let c2rust_lvalue_ptr = &raw mut *ptr1;
                    *c2rust_lvalue_ptr = *c2rust_fresh12;
                } else {
                    let c2rust_fresh13 = ptr2;
                    ptr2 = ptr2.offset(1);
                    xfree(*c2rust_fresh13 as *mut ::core::ffi::c_void);
                    count -= 1;
                }
            }
        }
        *numMatches = count;
        return if count == 0 as ::core::ffi::c_int {
            FAIL
        } else {
            OK
        };
    }
}
