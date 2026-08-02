//! [`do_highlight`], the `:highlight` command.
//!
//! It parses `:hi [default] {group} key=value ...` — and the `clear` and
//! `link` forms — writing what it finds into the group's table entry. Each
//! key family has its own rules about what `default` and `init` mean for
//! an already-set value.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn set_gui_color(
    mut idx: ::core::ffi::c_int,
    mut init: bool,
    mut arg: *const ::core::ffi::c_char,
    mut color: *mut RgbValue,
    mut color_idx: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        if init as ::core::ffi::c_int != 0
            && (*(hl_table()).offset(idx as isize)).set & SG_GUI as ::core::ffi::c_int != 0
        {
            return false_0 != 0;
        }
        if !init {
            (*(hl_table()).offset(idx as isize)).set |= SG_GUI as ::core::ffi::c_int;
        }
        let mut old_color: RgbValue = *color;
        let mut old_idx: ::core::ffi::c_int = *color_idx;
        if strcmp(arg, b"NONE\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int
        {
            let (rgb, idx) = name_to_color(::core::ffi::CStr::from_ptr(arg));
            *color = rgb;
            *color_idx = idx;
        } else {
            *color = -1 as ::core::ffi::c_int as RgbValue;
            *color_idx = kColorIdxNone as ::core::ffi::c_int;
        }
        return *color != old_color || *color_idx != old_idx;
    }
}

pub unsafe extern "C" fn do_highlight(
    mut line: *const ::core::ffi::c_char,
    forceit: bool,
    init: bool,
) {
    unsafe {
        if !init && ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while i <= highlight_num_groups() && !got_int.get() {
                highlight_list_one(i);
                i += 1;
            }
            return;
        }
        let mut dodefault: bool = false_0 != 0;
        let mut name_end: *const ::core::ffi::c_char = skiptowhite(line);
        let mut linep: *const ::core::ffi::c_char = skipwhite(name_end);
        if strncmp(
            line,
            b"default\0".as_ptr() as *const ::core::ffi::c_char,
            name_end.offset_from(line) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            dodefault = true_0 != 0;
            line = linep;
            name_end = skiptowhite(line);
            linep = skipwhite(name_end);
        }
        let mut doclear: bool = false_0 != 0;
        let mut dolink: bool = false_0 != 0;
        if strncmp(
            line,
            b"clear\0".as_ptr() as *const ::core::ffi::c_char,
            name_end.offset_from(line) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            doclear = true_0 != 0;
        } else if strncmp(
            line,
            b"link\0".as_ptr() as *const ::core::ffi::c_char,
            name_end.offset_from(line) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            dolink = true_0 != 0;
        }
        if !doclear && !dolink && ends_excmd(*linep as uint8_t as ::core::ffi::c_int) != 0 {
            let mut id: ::core::ffi::c_int =
                syn_name2id_len(line, name_end.offset_from(line) as size_t);
            if id == 0 as ::core::ffi::c_int {
                semsg(
                    gettext(
                        (e_highlight_group_name_not_found_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    line,
                );
            } else {
                msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
                highlight_list_one(id);
            }
            return;
        }
        if dolink {
            let mut from_start: *const ::core::ffi::c_char = linep;
            let mut to_id: ::core::ffi::c_int = 0;
            let mut hlgroup: *mut HlGroup = ::core::ptr::null_mut::<HlGroup>();
            let mut from_end: *const ::core::ffi::c_char = skiptowhite(from_start);
            let mut to_start: *const ::core::ffi::c_char = skipwhite(from_end);
            let mut to_end: *const ::core::ffi::c_char = skiptowhite(to_start);
            if ends_excmd(*from_start as uint8_t as ::core::ffi::c_int) != 0
                || ends_excmd(*to_start as uint8_t as ::core::ffi::c_int) != 0
            {
                semsg(
                    gettext(
                        b"E412: Not enough arguments: \":highlight link %s\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    from_start,
                );
                return;
            }
            if ends_excmd(*skipwhite(to_end) as ::core::ffi::c_int) == 0 {
                semsg(
                    gettext(
                        b"E413: Too many arguments: \":highlight link %s\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    from_start,
                );
                return;
            }
            let mut from_id: ::core::ffi::c_int =
                syn_check_group(from_start, from_end.offset_from(from_start) as size_t);
            if strncmp(
                to_start,
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                to_id = 0 as ::core::ffi::c_int;
            } else {
                to_id = syn_check_group(to_start, to_end.offset_from(to_start) as size_t);
            }
            if from_id > 0 as ::core::ffi::c_int {
                hlgroup = (hl_table()).offset((from_id - 1 as ::core::ffi::c_int) as isize);
                if dodefault as ::core::ffi::c_int != 0
                    && (forceit as ::core::ffi::c_int != 0
                        || (*hlgroup).deflink == 0 as ::core::ffi::c_int)
                {
                    (*hlgroup).deflink = to_id;
                    (*hlgroup).deflink_sctx = current_sctx.get();
                    (*hlgroup).deflink_sctx.sc_lnum += (*((*exestack.ptr()).ga_data
                        as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum;
                    nlua_set_sctx(&raw mut (*hlgroup).deflink_sctx);
                }
            }
            if from_id > 0 as ::core::ffi::c_int
                && (!init || (*hlgroup).set == 0 as ::core::ffi::c_int)
            {
                if to_id > 0 as ::core::ffi::c_int
                    && !forceit
                    && !init
                    && hl_has_settings(from_id, dodefault) as ::core::ffi::c_int != 0
                {
                    if (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name
                    .is_null()
                        && !dodefault
                    {
                        emsg(gettext(
                            (e_group_has_settings_highlight_link_ignored.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                    }
                } else if (*hlgroup).link != to_id
                    || (*hlgroup).script_ctx.sc_sid != (*current_sctx.ptr()).sc_sid
                    || (*hlgroup).cleared as ::core::ffi::c_int != 0
                {
                    if !init {
                        (*hlgroup).set |= SG_LINK as ::core::ffi::c_int;
                    }
                    (*hlgroup).link = to_id;
                    (*hlgroup).script_ctx = current_sctx.get();
                    (*hlgroup).script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data
                        as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum;
                    nlua_set_sctx(&raw mut (*hlgroup).script_ctx);
                    (*hlgroup).cleared = false_0 != 0;
                    redraw_all_later(UPD_SOME_VALID);
                    need_highlight_changed.set(true_0 != 0);
                }
            }
            return;
        }
        if doclear {
            line = linep;
            if ends_excmd(*line as uint8_t as ::core::ffi::c_int) != 0 {
                do_unlet(
                    b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
                    true_0 != 0,
                );
                restore_cterm_colors();
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < highlight_num_groups() {
                    highlight_clear(j + 1 as ::core::ffi::c_int);
                    j += 1;
                }
                init_highlight(true_0 != 0, true_0 != 0);
                highlight_changed();
                redraw_all_later(UPD_NOT_VALID);
                return;
            }
            name_end = skiptowhite(line);
            linep = skipwhite(name_end);
        }
        let mut id_0: ::core::ffi::c_int =
            syn_check_group(line, name_end.offset_from(line) as size_t);
        if id_0 == 0 as ::core::ffi::c_int {
            return;
        }
        let mut idx: ::core::ffi::c_int = id_0 - 1 as ::core::ffi::c_int;
        if dodefault as ::core::ffi::c_int != 0
            && hl_has_settings(idx + 1 as ::core::ffi::c_int, true_0 != 0) as ::core::ffi::c_int
                != 0
        {
            return;
        }
        let mut item_before: HlGroup = *(hl_table()).offset(idx as isize);
        let mut is_normal_group: bool = strcmp(
            (*(hl_table()).offset(idx as isize))
                .name_u
                .as_ptr()
                .cast_mut(),
            b"NORMAL\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int;
        if doclear as ::core::ffi::c_int != 0
            || forceit as ::core::ffi::c_int != 0 && init as ::core::ffi::c_int != 0
        {
            highlight_clear(idx + 1 as ::core::ffi::c_int);
            if !doclear {
                (*(hl_table()).offset(idx as isize)).set = 0 as ::core::ffi::c_int;
            }
        }
        let mut did_change: bool = false_0 != 0;
        let mut error: bool = false_0 != 0;
        let mut key: [::core::ffi::c_char; 64] = [0; 64];
        let mut arg: [::core::ffi::c_char; 512] = [0; 512];
        if !doclear {
            let mut arg_start: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            while ends_excmd(*linep as uint8_t as ::core::ffi::c_int) == 0 {
                let mut key_start: *const ::core::ffi::c_char = linep;
                if *linep as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            (e_unexpected_equal_sign_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        key_start,
                    );
                    error = true_0 != 0;
                    break;
                } else {
                    while *linep as ::core::ffi::c_int != 0
                        && !ascii_iswhite(*linep as ::core::ffi::c_int)
                        && *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int
                    {
                        linep = linep.offset(1);
                    }
                    let mut key_len: size_t = linep.offset_from(key_start) as size_t;
                    if key_len
                        > ::core::mem::size_of::<[::core::ffi::c_char; 64]>()
                            .wrapping_sub(1 as usize)
                    {
                        emsg(gettext(
                            b"E423: Illegal argument\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        error = true_0 != 0;
                        break;
                    } else {
                        vim_memcpy_up(&raw mut key as *mut ::core::ffi::c_char, key_start, key_len);
                        key[key_len as usize] = NUL as ::core::ffi::c_char;
                        linep = skipwhite(linep);
                        if strcmp(
                            &raw mut key as *mut ::core::ffi::c_char,
                            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                        {
                            if !init
                                || (*(hl_table()).offset(idx as isize)).set
                                    == 0 as ::core::ffi::c_int
                            {
                                if !init {
                                    (*(hl_table()).offset(idx as isize)).set |= SG_CTERM
                                        as ::core::ffi::c_int
                                        + SG_GUI as ::core::ffi::c_int;
                                }
                                highlight_clear(idx + 1 as ::core::ffi::c_int);
                            }
                        } else if *linep as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_missing_equal_sign_str_2.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                key_start,
                            );
                            error = true_0 != 0;
                            break;
                        } else {
                            linep = linep.offset(1);
                            linep = skipwhite(linep);
                            if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                                linep = linep.offset(1);
                                arg_start = linep;
                                linep = strchr(linep, '\'' as ::core::ffi::c_int);
                                if linep.is_null() {
                                    semsg(
                                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                                        key_start,
                                    );
                                    error = true_0 != 0;
                                    break;
                                }
                            } else {
                                arg_start = linep;
                                linep = skiptowhite(linep);
                            }
                            if linep == arg_start {
                                semsg(
                                    gettext(
                                        (e_missing_argument_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    key_start,
                                );
                                error = true_0 != 0;
                                break;
                            } else {
                                let mut arg_len: size_t = linep.offset_from(arg_start) as size_t;
                                if arg_len
                                    > ::core::mem::size_of::<[::core::ffi::c_char; 512]>()
                                        .wrapping_sub(1 as usize)
                                {
                                    emsg(gettext(b"E423: Illegal argument\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                    error = true_0 != 0;
                                    break;
                                } else {
                                    memcpy(
                                        &raw mut arg as *mut ::core::ffi::c_char
                                            as *mut ::core::ffi::c_void,
                                        arg_start as *const ::core::ffi::c_void,
                                        arg_len,
                                    );
                                    arg[arg_len as usize] = NUL as ::core::ffi::c_char;
                                    if *linep as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                                        linep = linep.offset(1);
                                    }
                                    if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"TERM\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"CTERM\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        || strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"GUI\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                    {
                                        let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        let mut i_0: ::core::ffi::c_int = 0;
                                        while arg[off as usize] as ::core::ffi::c_int != NUL {
                                            i_0 =
                                                ::core::mem::size_of::<[::core::ffi::c_int; 18]>()
                                                    .wrapping_div(::core::mem::size_of::<
                                                        ::core::ffi::c_int,
                                                    >(
                                                    ))
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<
                                                            [::core::ffi::c_int; 18],
                                                        >(
                                                        )
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            ::core::ffi::c_int,
                                                        >(
                                                        )) == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    )
                                                    as ::core::ffi::c_int;
                                            loop {
                                                i_0 -= 1;
                                                if i_0 < 0 as ::core::ffi::c_int {
                                                    break;
                                                }
                                                let mut len: ::core::ffi::c_int = strlen(
                                                    (*hl_name_table.ptr())[i_0 as usize]
                                                        as *const ::core::ffi::c_char,
                                                )
                                                    as ::core::ffi::c_int;
                                                if strncasecmp(
                                                    (&raw mut arg as *mut ::core::ffi::c_char)
                                                        .offset(off as isize),
                                                    (*hl_name_table.ptr())[i_0 as usize],
                                                    len as size_t,
                                                ) != 0 as ::core::ffi::c_int
                                                {
                                                    continue;
                                                }
                                                if (*hl_attr_table.ptr())[i_0 as usize]
                                                    & HL_UNDERLINE_MASK
                                                    != 0
                                                {
                                                    attr &= !(HL_UNDERLINE_MASK);
                                                }
                                                attr |= (*hl_attr_table.ptr())[i_0 as usize];
                                                off += len;
                                                break;
                                            }
                                            if i_0 < 0 as ::core::ffi::c_int {
                                                semsg(
                                                    gettext(b"E418: Illegal value: %s\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                );
                                                error = true_0 != 0;
                                                break;
                                            } else if arg[off as usize] as ::core::ffi::c_int
                                                == ',' as ::core::ffi::c_int
                                            {
                                                off += 1;
                                            }
                                        }
                                        if error {
                                            break;
                                        }
                                        if *(&raw mut key as *mut ::core::ffi::c_char)
                                            as ::core::ffi::c_int
                                            == 'C' as ::core::ffi::c_int
                                        {
                                            if !init
                                                || (*(hl_table()).offset(idx as isize)).set
                                                    & SG_CTERM as ::core::ffi::c_int
                                                    == 0
                                            {
                                                if !init {
                                                    (*(hl_table()).offset(idx as isize)).set |=
                                                        SG_CTERM as ::core::ffi::c_int;
                                                }
                                                (*(hl_table()).offset(idx as isize)).cterm = attr;
                                                (*(hl_table()).offset(idx as isize)).cterm_bold =
                                                    false_0 != 0;
                                            }
                                        } else if *(&raw mut key as *mut ::core::ffi::c_char)
                                            as ::core::ffi::c_int
                                            == 'G' as ::core::ffi::c_int
                                        {
                                            if !init
                                                || (*(hl_table()).offset(idx as isize)).set
                                                    & SG_GUI as ::core::ffi::c_int
                                                    == 0
                                            {
                                                if !init {
                                                    (*(hl_table()).offset(idx as isize)).set |=
                                                        SG_GUI as ::core::ffi::c_int;
                                                }
                                                (*(hl_table()).offset(idx as isize)).gui = attr;
                                            }
                                        }
                                    } else if strcmp(
                                        &raw mut key as *mut ::core::ffi::c_char,
                                        b"FONT\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"CTERMFG\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                &raw mut key as *mut ::core::ffi::c_char,
                                                b"CTERMBG\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            if !init
                                                || (*(hl_table()).offset(idx as isize)).set
                                                    & SG_CTERM as ::core::ffi::c_int
                                                    == 0
                                            {
                                                if !init {
                                                    (*(hl_table()).offset(idx as isize)).set |=
                                                        SG_CTERM as ::core::ffi::c_int;
                                                }
                                                if key[5 as ::core::ffi::c_int as usize]
                                                    as ::core::ffi::c_int
                                                    == 'F' as ::core::ffi::c_int
                                                    && (*(hl_table()).offset(idx as isize))
                                                        .cterm_bold
                                                        as ::core::ffi::c_int
                                                        != 0
                                                {
                                                    (*(hl_table()).offset(idx as isize)).cterm &=
                                                        !(HL_BOLD);
                                                    (*(hl_table()).offset(idx as isize))
                                                        .cterm_bold = false_0 != 0;
                                                }
                                                let mut color: ::core::ffi::c_int = 0;
                                                if ascii_isdigit(
                                                    *(&raw mut arg as *mut ::core::ffi::c_char)
                                                        as ::core::ffi::c_int,
                                                ) {
                                                    color = atoi(
                                                        &raw mut arg as *mut ::core::ffi::c_char,
                                                    );
                                                } else if strcasecmp(
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                    b"fg\0".as_ptr() as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                ) == 0 as ::core::ffi::c_int
                                                {
                                                    if cterm_normal_fg_color.get() != 0 {
                                                        color = cterm_normal_fg_color.get()
                                                            - 1 as ::core::ffi::c_int;
                                                    } else {
                                                        emsg(gettext(
                                                            b"E419: FG color unknown\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ));
                                                        error = true_0 != 0;
                                                        break;
                                                    }
                                                } else if strcasecmp(
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                    b"bg\0".as_ptr() as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                ) == 0 as ::core::ffi::c_int
                                                {
                                                    if cterm_normal_bg_color.get()
                                                        > 0 as ::core::ffi::c_int
                                                    {
                                                        color = cterm_normal_bg_color.get()
                                                            - 1 as ::core::ffi::c_int;
                                                    } else {
                                                        emsg(gettext(
                                                            b"E420: BG color unknown\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ));
                                                        error = true_0 != 0;
                                                        break;
                                                    }
                                                } else {
                                                    let i_1 = match cterm_color_index(
                                                        ::core::ffi::CStr::from_ptr(
                                                            &raw const arg
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                    ) {
                                                        Some(i) => i as ::core::ffi::c_int,
                                                        None => -1,
                                                    };
                                                    if i_1 < 0 as ::core::ffi::c_int {
                                                        semsg(
                                                        gettext(
                                                            b"E421: Color name or number not recognized: %s\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ),
                                                        key_start,
                                                    );
                                                        error = true_0 != 0;
                                                        break;
                                                    } else {
                                                        let (c, bold) = lookup_color(
                                                            i_1 as usize,
                                                            key[5 as ::core::ffi::c_int as usize]
                                                                as ::core::ffi::c_int
                                                                == 'F' as ::core::ffi::c_int,
                                                        );
                                                        color = c;
                                                        if bold as ::core::ffi::c_int
                                                            == kTrue as ::core::ffi::c_int
                                                        {
                                                            (*(hl_table()).offset(idx as isize))
                                                                .cterm |= HL_BOLD;
                                                            (*(hl_table()).offset(idx as isize))
                                                                .cterm_bold = true_0 != 0;
                                                        } else if bold as ::core::ffi::c_int
                                                            == kFalse as ::core::ffi::c_int
                                                        {
                                                            (*(hl_table()).offset(idx as isize))
                                                                .cterm &= !(HL_BOLD);
                                                        }
                                                    }
                                                }
                                                if key[5 as ::core::ffi::c_int as usize]
                                                    as ::core::ffi::c_int
                                                    == 'F' as ::core::ffi::c_int
                                                {
                                                    (*(hl_table()).offset(idx as isize)).cterm_fg =
                                                        color + 1 as ::core::ffi::c_int;
                                                    if is_normal_group {
                                                        cterm_normal_fg_color
                                                            .set(color + 1 as ::core::ffi::c_int);
                                                    }
                                                } else {
                                                    (*(hl_table()).offset(idx as isize)).cterm_bg =
                                                        color + 1 as ::core::ffi::c_int;
                                                    if is_normal_group {
                                                        cterm_normal_bg_color
                                                            .set(color + 1 as ::core::ffi::c_int);
                                                        if !ui_rgb_attached() {
                                                            if color >= 0 as ::core::ffi::c_int {
                                                                let mut dark: ::core::ffi::c_int =
                                                                    -1 as ::core::ffi::c_int;
                                                                if t_colors.get()
                                                                    < 16 as ::core::ffi::c_int
                                                                {
                                                                    dark = (color
                                                                    == 0 as ::core::ffi::c_int
                                                                    || color
                                                                        == 4 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                                } else if color
                                                                    < 16 as ::core::ffi::c_int
                                                                {
                                                                    dark = (color
                                                                    < 7 as ::core::ffi::c_int
                                                                    || color
                                                                        == 8 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int;
                                                                }
                                                                if dark != -1 as ::core::ffi::c_int
                                                                    && dark != (*p_bg.get()
                                                                        as ::core::ffi::c_int
                                                                        == 'd'
                                                                            as ::core::ffi::c_int)
                                                                        as ::core::ffi::c_int
                                                                    && !option_was_set(
                                                                        kOptBackground,
                                                                    )
                                                                {
                                                                    set_option_value_give_err(
                                                                        kOptBackground,
                                                                        OptVal {
                                                                            type_0:
                                                                                kOptValTypeString,
                                                                            data: OptValData {
                                                                                string:
                                                                                    cstr_as_string(
                                                                                        if dark != 0
                                                                                        {
                                                                                            b"dark\0".as_ptr() as *const ::core::ffi::c_char
                                                                                        } else {
                                                                                            b"light\0".as_ptr() as *const ::core::ffi::c_char
                                                                                        },
                                                                                    ),
                                                                            },
                                                                        },
                                                                        0 as ::core::ffi::c_int,
                                                                    );
                                                                    reset_option_was_set(
                                                                        kOptBackground,
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"GUIFG\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            did_change = set_gui_color(
                                                idx,
                                                init,
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_fg,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_fg_idx,
                                            );
                                            if is_normal_group {
                                                normal_fg.set(
                                                    (*(hl_table()).offset(idx as isize)).rgb_fg,
                                                );
                                            }
                                        } else if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"GUIBG\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            did_change = set_gui_color(
                                                idx,
                                                init,
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_bg,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_bg_idx,
                                            );
                                            if is_normal_group {
                                                normal_bg.set(
                                                    (*(hl_table()).offset(idx as isize)).rgb_bg,
                                                );
                                            }
                                        } else if strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"GUISP\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            did_change = set_gui_color(
                                                idx,
                                                init,
                                                &raw mut arg as *mut ::core::ffi::c_char,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_sp,
                                                &raw mut (*(hl_table()).offset(idx as isize))
                                                    .rgb_sp_idx,
                                            );
                                            if is_normal_group {
                                                normal_sp.set(
                                                    (*(hl_table()).offset(idx as isize)).rgb_sp,
                                                );
                                            }
                                        } else if !(strcmp(
                                            &raw mut key as *mut ::core::ffi::c_char,
                                            b"START\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                &raw mut key as *mut ::core::ffi::c_char,
                                                b"STOP\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int)
                                        {
                                            if strcmp(
                                                &raw mut key as *mut ::core::ffi::c_char,
                                                b"BLEND\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if strcmp(
                                                    &raw mut arg as *mut ::core::ffi::c_char,
                                                    b"NONE\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ) != 0 as ::core::ffi::c_int
                                                {
                                                    (*(hl_table()).offset(idx as isize)).blend =
                                                        strtol(
                                                            &raw mut arg
                                                                as *mut ::core::ffi::c_char,
                                                            ::core::ptr::null_mut::<
                                                                *mut ::core::ffi::c_char,
                                                            >(
                                                            ),
                                                            10 as ::core::ffi::c_int,
                                                        )
                                                            as ::core::ffi::c_int;
                                                } else {
                                                    (*(hl_table()).offset(idx as isize)).blend =
                                                        -1 as ::core::ffi::c_int;
                                                }
                                            } else {
                                                semsg(
                                                    gettext(
                                                        b"E423: Illegal argument: %s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    key_start,
                                                );
                                                error = true_0 != 0;
                                                break;
                                            }
                                        }
                                    }
                                    (*(hl_table()).offset(idx as isize)).cleared = false_0 != 0;
                                    if !init
                                        || (*(hl_table()).offset(idx as isize)).set
                                            & SG_LINK as ::core::ffi::c_int
                                            == 0
                                    {
                                        (*(hl_table()).offset(idx as isize)).link =
                                            0 as ::core::ffi::c_int;
                                    }
                                    linep = skipwhite(linep);
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut did_highlight_changed: bool = false_0 != 0;
        if !error && is_normal_group as ::core::ffi::c_int != 0 {
            highlight_attr_set_all();
            if !ui_has(kUILinegrid) && starting.get() == 0 as ::core::ffi::c_int {
                ui_refresh();
            } else {
                ui_default_colors_set();
            }
            did_highlight_changed = true_0 != 0;
            redraw_all_later(UPD_NOT_VALID);
        } else {
            set_hl_attr(idx + 1 as ::core::ffi::c_int);
        }
        (*(hl_table()).offset(idx as isize)).script_ctx = current_sctx.get();
        (*(hl_table()).offset(idx as isize)).script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data
            as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*(hl_table()).offset(idx as isize)).script_ctx);
        if (did_change as ::core::ffi::c_int != 0
            || memcmp(
                (hl_table()).offset(idx as isize) as *const ::core::ffi::c_void,
                &raw mut item_before as *const ::core::ffi::c_void,
                ::core::mem::size_of::<HlGroup>(),
            ) != 0 as ::core::ffi::c_int)
            && !did_highlight_changed
        {
            if !updating_screen.get() {
                redraw_all_later(UPD_NOT_VALID);
            }
            need_highlight_changed.set(true_0 != 0);
        }
    }
}
