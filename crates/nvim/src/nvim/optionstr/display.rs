//! The callbacks for options that decide what the screen looks like.
//!
//! They are `pub` only so the generated option table can name them.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_ambiwidth(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    return check_chars_options();
}

pub unsafe extern "C" fn did_set_emoji(mut _args: *mut optset_T) -> *const c_char {
    if check_str_opt(kOptAmbiwidth, ::core::ptr::null_mut::<*mut c_char>()) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return check_chars_options();
}

pub unsafe extern "C" fn did_set_background(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if *(*args).os_oldval.string.data.offset(0 as c_int as isize) as c_int == *p_bg.get() as c_int {
        return ::core::ptr::null::<c_char>();
    }
    let mut dark: c_int = (*p_bg.get() as c_int == 'd' as c_int) as c_int;
    init_highlight(false_0 != 0, false_0 != 0);
    if dark != (*p_bg.get() as c_int == 'd' as c_int) as c_int
        && !get_var_value(b"g:colors_name\0".as_ptr() as *const c_char).is_null()
    {
        do_unlet(
            b"g:colors_name\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 14]>().wrapping_sub(1 as size_t),
            true_0 != 0,
        );
        free_string_option(p_bg.get());
        p_bg.set(xstrdup(if dark != 0 {
            b"dark\0".as_ptr() as *const c_char
        } else {
            b"light\0".as_ptr() as *const c_char
        }));
        check_string_option(p_bg.ptr());
        init_highlight(false_0 != 0, false_0 != 0);
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).terminal.is_null() {
            terminal_notify_theme((*buf).terminal, dark != 0);
        }
        buf = (*buf).b_next;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_breakat(mut _args: *mut optset_T) -> *const c_char {
    let mut i: c_int = 0 as c_int;
    while i < 256 as c_int {
        (*breakat_flags.ptr())[i as usize] = false_0 as c_char;
        i += 1;
    }
    if !(*p_breakat.ptr()).is_null() {
        let mut p: *mut c_char = p_breakat.get();
        while *p != 0 {
            (*breakat_flags.ptr())[*p as uint8_t as usize] = true_0 as c_char;
            p = p.offset(1);
        }
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_breakindentopt(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if briopt_check(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_briopt {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) as c_int
        == FAIL
    {
        return &raw const e_invarg as *const c_char;
    }
    if varp == &raw mut (*win).w_onebuf_opt.wo_briopt && (*win).w_briopt_list != 0 {
        redraw_all_later(UPD_NOT_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_colorcolumn(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return check_colorcolumn(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_cc {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    );
}

pub unsafe extern "C" fn did_set_concealcursor(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        COCU_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_cursorlineopt(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if **varp as c_int == NUL || fill_culopt_flags(*varp, win) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_display(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    init_chartab();
    msg_grid_validate();
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_guicursor(mut _args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = parse_shape_opt(SHAPE_CURSOR);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active.get() {
        redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_highlight(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if strcmp(*varp, HIGHLIGHT_INIT.as_ptr()) != 0 as c_int {
        return &raw const e_unsupportedoption as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_inccommand(mut args: *mut optset_T) -> *const c_char {
    if cmdpreview.get() {
        return &raw const e_invarg as *const c_char;
    }
    return did_set_str_generic(args);
}

pub unsafe extern "C" fn did_set_keymodel(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    km_stopsel.set(!vim_strchr(p_km.get(), 'o' as c_int).is_null());
    km_startsel.set(!vim_strchr(p_km.get(), 'a' as c_int).is_null());
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_messagesopt(mut _args: *mut optset_T) -> *const c_char {
    if messagesopt_changed() == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_mouse(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        MOUSE_ALL.as_ptr() as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_mousescroll(mut _args: *mut optset_T) -> *const c_char {
    let mut vertical: OptInt = -1 as OptInt;
    let mut horizontal: OptInt = -1 as OptInt;
    let mut string: *mut c_char = p_mousescroll.get();
    loop {
        let mut end: *mut c_char = vim_strchr(string, ',' as c_int);
        let mut length: size_t = if !end.is_null() {
            end.offset_from(string) as size_t
        } else {
            strlen(string)
        };
        if length <= 4 as size_t {
            return &raw const e_invarg as *const c_char;
        }
        let mut direction: *mut OptInt = ::core::ptr::null_mut::<OptInt>();
        if memcmp(
            string as *const c_void,
            b"ver:\0".as_ptr() as *const c_char as *const c_void,
            4 as size_t,
        ) == 0 as c_int
        {
            direction = &raw mut vertical;
        } else if memcmp(
            string as *const c_void,
            b"hor:\0".as_ptr() as *const c_char as *const c_void,
            4 as size_t,
        ) == 0 as c_int
        {
            direction = &raw mut horizontal;
        } else {
            return &raw const e_invarg as *const c_char;
        }
        if *direction != -1 as OptInt {
            return &raw const e_invarg as *const c_char;
        }
        let mut i: size_t = 4 as size_t;
        while i < length {
            if !ascii_isdigit(*string.offset(i as isize) as c_int) {
                return b"E5080: Digit expected\0".as_ptr() as *const c_char;
            }
            i = i.wrapping_add(1);
        }
        string = string.offset(4 as c_int as isize);
        *direction = getdigits_int(&raw mut string, false_0 != 0, -1 as c_int) as OptInt;
        if *direction == -1 as OptInt {
            return &raw const e_invarg as *const c_char;
        }
        if end.is_null() {
            break;
        }
        string = end.offset(1 as c_int as isize);
    }
    p_mousescroll_vert.set(if vertical == -1 as OptInt {
        MOUSESCROLL_VERT_DFLT as OptInt
    } else {
        vertical
    });
    p_mousescroll_hor.set(if horizontal == -1 as OptInt {
        MOUSESCROLL_HOR_DFLT as OptInt
    } else {
        horizontal
    });
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_selection(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active.get() {
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_showbreak(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut s: *mut c_char = *varp;
    while *s != 0 {
        if ptr2cells(s) != 1 as c_int {
            return (e_showbreak_contains_unprintable_or_wide_character.ptr() as *const _)
                as *const c_char;
        }
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_showcmdloc(mut args: *mut optset_T) -> *const c_char {
    let mut errmsg: *const c_char = did_set_str_generic(args);
    if errmsg.is_null() {
        comp_col();
    }
    return errmsg;
}

pub unsafe extern "C" fn did_set_signcolumn(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut oldval: *const c_char = (*args).os_oldval.string.data;
    if check_signcolumn(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_scl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    }
    if *oldval as c_int == 'n' as c_int
        && *oldval.offset(1 as c_int as isize) as c_int == 'u' as c_int
        || (*win).w_minscwidth == SCL_NUM
    {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_virtualedit(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut ve: *mut c_char = p_ve.get();
    let mut flags: *mut c_uint = ve_flags.ptr();
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        ve = (*win).w_onebuf_opt.wo_ve;
        flags = &raw mut (*win).w_onebuf_opt.wo_ve_flags;
    }
    if (*args).os_flags & OPT_LOCAL as c_int != 0 && *ve as c_int == NUL {
        *flags = 0 as c_uint;
    } else if opt_strings_flags(
        ve,
        opt_ve_values.ptr() as *mut *const c_char,
        flags,
        true_0 != 0,
    ) != OK
    {
        return &raw const e_invarg as *const c_char;
    } else if strcmp(ve, (*args).os_oldval.string.data) != 0 as c_int {
        validate_virtcol(win);
        coladvance(win, (*win).w_virtcol);
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_whichwrap(mut args: *mut optset_T) -> *const c_char {
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    return did_set_option_listflag(
        *varp,
        b"bshl<>[]~,\0".as_ptr() as *const c_char as *mut c_char,
        (*args).os_errbuf,
        (*args).os_errbuflen,
    );
}

pub unsafe extern "C" fn did_set_wildmode(mut _args: *mut optset_T) -> *const c_char {
    if check_opt_wim() == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_winbar(mut args: *mut optset_T) -> *const c_char {
    return did_set_statustabline_rulerformat(args, false_0 != 0, false_0 != 0);
}

pub(crate) unsafe extern "C" fn parse_border_opt(mut border_opt: *mut c_char) -> bool {
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as c_int,
        width: 0 as c_int,
        row: 0 as c_int as c_double,
        col: 0 as c_int as c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut result: bool = true_0 != 0;
    if !parse_winborder(&raw mut fconfig, border_opt, &raw mut err) {
        result = false_0 != 0;
    }
    api_clear_error(&raw mut err);
    return result;
}

pub unsafe extern "C" fn did_set_winborder(mut _args: *mut optset_T) -> *const c_char {
    if !parse_border_opt(p_winborder.get()) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_pumborder(mut _args: *mut optset_T) -> *const c_char {
    if !parse_border_opt(p_pumborder.get()) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn did_set_winhighlight(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    if !parse_winhl_opt(
        *varp,
        if varp == &raw mut (*win).w_onebuf_opt.wo_winhl {
            win
        } else {
            ::core::ptr::null_mut::<win_T>()
        },
    ) {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
