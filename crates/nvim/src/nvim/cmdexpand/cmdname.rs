//! The per-command context switch.
//!
//! [`set_context_by_cmdname`] is C's `set_context_by_cmdname`: one arm per
//! `CMD_*` whose argument has its own completion.  [`set_one_cmd_context`]
//! walks a single command's arguments to find the one the cursor is in, and
//! [`set_cmd_context`] / [`expand_cmdline`] are the two entry points over
//! both.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn set_context_by_cmdname(
    mut cmd: *const ::core::ffi::c_char,
    mut cmdidx: cmdidx_T,
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut context: ::core::ffi::c_int,
    mut forceit: bool,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut nextcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        's_685: {
            match cmdidx as ::core::ffi::c_int {
                158 | 403 | 457 => {
                    if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
                        (*xp).xp_context = if *get_findfunc() as ::core::ffi::c_int != NUL {
                            EXPAND_FINDFUNC as ::core::ffi::c_int
                        } else {
                            EXPAND_FILES_IN_PATH as ::core::ffi::c_int
                        };
                    }
                    break 's_685;
                }
                61 | 71 | 225 | 226 | 448 | 449 => {
                    if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
                        (*xp).xp_context = EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int;
                    }
                    break 's_685;
                }
                176 => {
                    (*xp).xp_context = EXPAND_HELP as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                3 | 10 | 26 | 31 | 38 | 40 | 62 | 66 | 97 | 111 | 165 | 164 | 181 | 183 | 209
                | 207 | 206 | 208 | 228 | 230 | 234 | 255 | 298 | 302 | 369 | 374 | 386 | 407
                | 453 | 455 | 484 | 502 | 506 | 507 | 528 => return arg,
                157 => return set_context_in_filter_cmd(xp, arg),
                278 => return set_context_in_match_cmd(xp, arg),
                93 => return set_context_in_user_cmd(xp, arg),
                114 => {
                    (*xp).xp_context = EXPAND_USER_COMMANDS as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                170 | 504 => {
                    nextcmd = find_cmd_after_global_cmd(arg);
                    if nextcmd.is_null() && may_expand_pattern.get() as ::core::ffi::c_int != 0 {
                        set_context_with_pattern(xp);
                    }
                    return nextcmd;
                }
                550 | 382 => {
                    nextcmd = find_cmd_after_substitute_cmd(arg);
                    if nextcmd.is_null() && may_expand_pattern.get() as ::core::ffi::c_int != 0 {
                        set_context_with_pattern(xp);
                    }
                    return nextcmd;
                }
                198 | 131 | 189 | 127 | 188 | 334 | 126 | 199 | 132 => {
                    return find_cmd_after_isearch_cmd(xp, arg);
                }
                17 => {
                    return set_context_in_autocmd(
                        xp,
                        arg as *mut ::core::ffi::c_char,
                        false_0 != 0,
                    );
                }
                128 | 129 => {
                    return set_context_in_autocmd(
                        xp,
                        arg as *mut ::core::ffi::c_char,
                        true_0 != 0,
                    );
                }
                399 => {
                    set_context_in_set_cmd(
                        xp,
                        arg as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                    );
                    break 's_685;
                }
                401 => {
                    set_context_in_set_cmd(
                        xp,
                        arg as *mut ::core::ffi::c_char,
                        OPT_GLOBAL as ::core::ffi::c_int,
                    );
                    break 's_685;
                }
                402 => {
                    set_context_in_set_cmd(
                        xp,
                        arg as *mut ::core::ffi::c_char,
                        OPT_LOCAL as ::core::ffi::c_int,
                    );
                    break 's_685;
                }
                451 | 431 | 335 | 262 | 489 | 437 | 343 | 474 | 436 | 338 => {
                    if wop_flags.get()
                        & kOptWopFlagTagfile as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        (*xp).xp_context = EXPAND_TAGS_LISTFILES as ::core::ffi::c_int;
                    } else {
                        (*xp).xp_context = EXPAND_TAGS as ::core::ffi::c_int;
                    }
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                18 => {
                    (*xp).xp_context = EXPAND_AUGROUP as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                444 => {
                    set_context_in_syntax_cmd(xp, arg);
                    break 's_685;
                }
                99 | 231 | 187 | 141 | 525 | 167 | 135 | 139 | 151 | 138 | 136 | 53 | 371 | 64
                | 50 | 70 | 232 | 216 | 238 => {
                    set_context_for_expression(xp, arg as *mut ::core::ffi::c_char, cmdidx);
                    break 's_685;
                }
                498 => return set_context_in_unlet_cmd(xp, arg),
                168 | 115 => {
                    (*xp).xp_context = EXPAND_USER_FUNC as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                137 => {
                    set_context_in_echohl_cmd(xp, arg);
                    break 's_685;
                }
                180 => {
                    set_context_in_highlight_cmd(xp, arg);
                    break 's_685;
                }
                406 => {
                    set_context_in_sign_cmd(xp, arg as *mut ::core::ffi::c_char);
                    break 's_685;
                }
                25 | 42 | 41 => loop {
                    (*xp).xp_pattern = strchr(arg, ' ' as ::core::ffi::c_int);
                    if (*xp).xp_pattern.is_null() {
                        break;
                    }
                    arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
                },
                20 | 388 | 321 | 75 => {}
                119 | 122 => {
                    (*xp).xp_context = EXPAND_DIFF_BUFFERS as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                -1 | -2 => {
                    return set_context_in_user_cmdarg(cmd, arg, argt, context, xp, forceit);
                }
                275 | 297 | 292 | 295 | 513 | 516 | 308 | 312 | 190 | 193 | 81 | 87 | 246 | 249
                | 411 | 416 | 539 | 542 => {
                    return set_context_in_map_cmd(
                        xp,
                        cmd as *mut ::core::ffi::c_char,
                        arg as *mut ::core::ffi::c_char,
                        forceit,
                        false_0 != 0,
                        false_0 != 0,
                        cmdidx,
                    );
                }
                500 | 305 | 520 | 315 | 200 | 105 | 263 | 439 | 544 => {
                    return set_context_in_map_cmd(
                        xp,
                        cmd as *mut ::core::ffi::c_char,
                        arg as *mut ::core::ffi::c_char,
                        forceit,
                        false_0 != 0,
                        true_0 != 0,
                        cmdidx,
                    );
                }
                276 | 293 | 514 | 309 | 191 | 82 | 247 | 412 | 540 => {
                    (*xp).xp_context = EXPAND_MAPCLEAR as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                1 | 300 | 46 | 88 | 185 | 194 => {
                    return set_context_in_map_cmd(
                        xp,
                        cmd as *mut ::core::ffi::c_char,
                        arg as *mut ::core::ffi::c_char,
                        forceit,
                        true_0 != 0,
                        false_0 != 0,
                        cmdidx,
                    );
                }
                495 | 106 | 201 => {
                    return set_context_in_map_cmd(
                        xp,
                        cmd as *mut ::core::ffi::c_char,
                        arg as *mut ::core::ffi::c_char,
                        forceit,
                        true_0 != 0,
                        true_0 != 0,
                        cmdidx,
                    );
                }
                279 | 301 | 501 | 5 | 6 | 19 | 294 | 296 | 306 | 515 | 518 | 521 | 310 | 313
                | 316 | 192 | 195 | 202 | 83 | 89 | 107 | 476 | 477 | 478 | 479 | 490 | 328
                | 142 => {
                    return set_context_in_menu_cmd(
                        xp,
                        cmd,
                        arg as *mut ::core::ffi::c_char,
                        forceit,
                    );
                }
                92 => {
                    (*xp).xp_context = EXPAND_COLORS as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                95 => {
                    (*xp).xp_context = EXPAND_COMPILER as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                317 => {
                    (*xp).xp_context = EXPAND_OWNSYNTAX as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                400 => {
                    (*xp).xp_context = EXPAND_FILETYPE as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                319 => {
                    (*xp).xp_context = EXPAND_PACKADD as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                376 => {
                    set_context_in_runtime_cmd(xp, arg);
                    break 's_685;
                }
                215 => return set_context_in_lang_cmd(xp, arg),
                332 => {
                    set_context_in_profile_cmd(xp, arg);
                    break 's_685;
                }
                73 => {
                    (*xp).xp_context = EXPAND_CHECKHEALTH as ::core::ffi::c_int;
                    break 's_685;
                }
                271 => {
                    (*xp).xp_context = EXPAND_LSP as ::core::ffi::c_int;
                    break 's_685;
                }
                370 => {
                    (*xp).xp_context = EXPAND_RETAB as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                281 => {
                    (*xp).xp_context = EXPAND_MESSAGES as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                182 => {
                    (*xp).xp_context = EXPAND_HISTORY as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                445 => {
                    (*xp).xp_context = EXPAND_SYNTIME as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                9 => {
                    loop {
                        (*xp).xp_pattern = vim_strchr(arg, ' ' as ::core::ffi::c_int);
                        if (*xp).xp_pattern.is_null() {
                            break;
                        }
                        arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
                    }
                    (*xp).xp_context = EXPAND_ARGLIST as ::core::ffi::c_int;
                    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                    break 's_685;
                }
                35 | 333 | 36 => return set_context_in_breakadd_cmd(xp, arg, cmdidx),
                397 => return set_context_in_scriptnames_cmd(xp, arg),
                156 => return set_context_in_filetype_cmd(xp, arg),
                264 | 552 => {
                    (*xp).xp_context = EXPAND_LUA as ::core::ffi::c_int;
                    break 's_685;
                }
                _ => {
                    break 's_685;
                }
            }
            (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
            (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn set_one_cmd_context(
    mut xp: *mut expand_T,
    mut buff: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = 0 as size_t;
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
        let mut forceit: bool = false_0 != 0;
        let mut usefilter: bool = false_0 != 0;
        ExpandInit(xp);
        (*xp).xp_pattern = buff as *mut ::core::ffi::c_char;
        (*xp).xp_line = buff as *mut ::core::ffi::c_char;
        (*xp).xp_context = EXPAND_COMMANDS as ::core::ffi::c_int;
        ea.argt = 0 as uint32_t;
        let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        cmd = buff;
        while !vim_strchr(
            b" \t:|\0".as_ptr() as *const ::core::ffi::c_char,
            *cmd as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            cmd = cmd.offset(1);
        }
        (*xp).xp_pattern = cmd as *mut ::core::ffi::c_char;
        if *cmd as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if *cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        cmd = skip_range(cmd, &raw mut (*xp).xp_context);
        (*xp).xp_pattern = cmd as *mut ::core::ffi::c_char;
        if *cmd as ::core::ffi::c_int == NUL {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if *cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if *cmd as ::core::ffi::c_int == '|' as ::core::ffi::c_int
            || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            return cmd.offset(1 as ::core::ffi::c_int as isize);
        }
        let mut p: *const ::core::ffi::c_char =
            set_cmd_index(cmd, &raw mut ea, xp, &raw mut context);
        if p.is_null() {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
            forceit = true_0 != 0;
            p = p.offset(1);
        }
        if !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
            ea.argt = excmd_get_argt(ea.cmdidx);
        }
        let mut arg: *const ::core::ffi::c_char = skipwhite(p);
        if ea.argt & EX_ARGOPT as uint32_t != 0 {
            while *arg as ::core::ffi::c_int != NUL
                && strncmp(
                    arg,
                    b"++\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                p = arg.offset(2 as ::core::ffi::c_int as isize);
                while *p as ::core::ffi::c_int != 0 && !ascii_isspace(*p as ::core::ffi::c_int) {
                    p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                }
                if *p as ::core::ffi::c_int == NUL {
                    if ea.argt & EX_ARGOPT as uint32_t != 0 {
                        return set_context_in_argopt(
                            xp,
                            arg.offset(2 as ::core::ffi::c_int as isize),
                        );
                    }
                }
                arg = skipwhite(p);
            }
        }
        if ea.cmdidx as ::core::ffi::c_int == CMD_write as ::core::ffi::c_int
            || ea.cmdidx as ::core::ffi::c_int == CMD_update as ::core::ffi::c_int
        {
            if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                arg = arg.offset(1);
                if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                    arg = arg.offset(1);
                }
                arg = skipwhite(arg);
            } else if *arg as ::core::ffi::c_int == '!' as ::core::ffi::c_int
                && ea.cmdidx as ::core::ffi::c_int == CMD_write as ::core::ffi::c_int
            {
                arg = arg.offset(1);
                usefilter = true_0 != 0;
            }
        }
        if ea.cmdidx as ::core::ffi::c_int == CMD_read as ::core::ffi::c_int {
            usefilter = forceit;
            if *arg as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                arg = arg.offset(1);
                usefilter = true_0 != 0;
            }
        }
        if ea.cmdidx as ::core::ffi::c_int == CMD_lshift as ::core::ffi::c_int
            || ea.cmdidx as ::core::ffi::c_int == CMD_rshift as ::core::ffi::c_int
        {
            while *arg as ::core::ffi::c_int == *cmd as ::core::ffi::c_int {
                arg = arg.offset(1);
            }
            arg = skipwhite(arg);
        }
        if ea.argt & EX_CMDARG as uint32_t != 0
            && !usefilter
            && *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        {
            p = arg.offset(1 as ::core::ffi::c_int as isize);
            arg = skip_cmd_arg(arg as *mut ::core::ffi::c_char, false_0 != 0);
            if *arg as ::core::ffi::c_int == NUL {
                return p;
            }
            arg = skipwhite(arg);
        }
        if ea.argt & EX_TRLBAR as uint32_t != 0 && !usefilter {
            p = arg;
            if ea.cmdidx as ::core::ffi::c_int == CMD_redir as ::core::ffi::c_int
                && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '@' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
            {
                p = p.offset(2 as ::core::ffi::c_int as isize);
            }
            while *p != 0 {
                if *p as ::core::ffi::c_int == Ctrl_V {
                    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                        p = p.offset(1);
                    }
                } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                    && ea.argt & EX_NOTRLCOM as uint32_t == 0
                    || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                {
                    if *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                    {
                        if *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                        {
                            return p.offset(1 as ::core::ffi::c_int as isize);
                        }
                        return ::core::ptr::null::<::core::ffi::c_char>();
                    }
                }
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
        }
        if ea.argt & EX_EXTRA as uint32_t == 0
            && *arg as ::core::ffi::c_int != NUL
            && strchr(
                b"|\"\0".as_ptr() as *const ::core::ffi::c_char,
                *arg as ::core::ffi::c_int,
            )
            .is_null()
        {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        p = buff;
        (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
        len = strlen(buff);
        while *p as ::core::ffi::c_int != 0 && p < buff.offset(len as isize) {
            if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == TAB
            {
                p = p.offset(1);
                (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
            } else {
                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                }
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
        }
        if ea.argt & EX_XFILE as uint32_t != 0 {
            set_context_for_wildcard_arg(&raw mut ea, arg, usefilter, xp, &raw mut context);
        }
        return set_context_by_cmdname(cmd, ea.cmdidx, xp, arg, ea.argt, context, forceit);
    }
}

pub unsafe extern "C" fn set_cmd_context(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut use_ccline: ::core::ffi::c_int,
) {
    unsafe {
        let ccline: *mut CmdlineInfo = get_cmdline_info();
        let mut old_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        if col < len {
            old_char = *str.offset(col as isize);
        }
        *str.offset(col as isize) = NUL as ::core::ffi::c_char;
        let mut nextcomm: *const ::core::ffi::c_char = str;
        if use_ccline != 0 && (*ccline).cmdfirstc == '=' as ::core::ffi::c_int {
            set_context_for_expression(xp, str, CMD_SIZE);
        } else if use_ccline != 0 && (*ccline).input_fn != 0 {
            (*xp).xp_context = (*ccline).xp_context;
            (*xp).xp_pattern = (*ccline).cmdbuff;
            (*xp).xp_arg = (*ccline).xp_arg;
            if (*xp).xp_context == EXPAND_SHELLCMDLINE as ::core::ffi::c_int {
                let mut context: ::core::ffi::c_int = (*xp).xp_context;
                set_context_for_wildcard_arg(
                    ::core::ptr::null_mut::<exarg_T>(),
                    (*xp).xp_pattern,
                    false_0 != 0,
                    xp,
                    &raw mut context,
                );
            }
        } else {
            while !nextcomm.is_null() {
                nextcomm = set_one_cmd_context(xp, nextcomm);
            }
        }
        (*xp).xp_line = str;
        (*xp).xp_col = col;
        *str.offset(col as isize) = old_char;
    }
}

pub unsafe extern "C" fn expand_cmdline(
    mut xp: *mut expand_T,
    mut str: *const ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut matchcount: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut file_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut options: ::core::ffi::c_int =
            WILD_ADD_SLASH as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int;
        if (*xp).xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
            beep_flush();
            return EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
        }
        if (*xp).xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
            return EXPAND_NOTHING as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if str.offset(col as isize).offset_from((*xp).xp_pattern) >= 0 as isize {
            } else {
                __assert_fail(
                    b"(str + col) - xp->xp_pattern >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2632 as ::core::ffi::c_uint,
                    b"int expand_cmdline(expand_T *, const char *, int, int *, char ***)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*xp).xp_pattern_len = str.offset(col as isize).offset_from((*xp).xp_pattern) as size_t;
        if cmdline_fuzzy_completion_supported(xp) {
            file_str = xstrdup((*xp).xp_pattern);
        } else {
            file_str = addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context);
        }
        if p_wic.get() != 0 {
            options += WILD_ICASE as ::core::ffi::c_int;
        }
        if ExpandFromContext(xp, file_str, matches, matchcount, options) == FAIL {
            *matchcount = 0 as ::core::ffi::c_int;
            *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        }
        xfree(file_str as *mut ::core::ffi::c_void);
        return EXPAND_OK as ::core::ffi::c_int;
    }
}
