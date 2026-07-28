//! What the editor is right now: `has()`, `mode()`, `state()` and the
//! rest of the feature and status queries.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_api_info(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    object_to_vim(api_metadata(), rettv, ::core::ptr::null_mut::<Error>());
}
pub unsafe extern "C" fn f_did_filetype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curbuf.get()).b_did_filetype as varnumber_T;
}
pub unsafe extern "C" fn f_eventhandler(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = vgetc_busy.get() as varnumber_T;
}
pub unsafe extern "C" fn f_menu_get(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut modes: ::core::ffi::c_int = MENU_ALL_MODES as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let strmodes: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        modes = get_menu_cmd_modes(
            strmodes,
            false_0 != 0,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
    }
    menu_get(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        modes,
        (*rettv).vval.v_list,
    );
}
pub unsafe extern "C" fn f_foreground(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
}
pub unsafe extern "C" fn f_getfontname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn f_getpid(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = os_get_pid() as varnumber_T;
}
pub unsafe extern "C" fn f_has(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static has_list: GlobalCell<[*const ::core::ffi::c_char; 90]> = GlobalCell::new([
        b"linux\0".as_ptr() as *const ::core::ffi::c_char,
        b"unix\0".as_ptr() as *const ::core::ffi::c_char,
        b"fname_case\0".as_ptr() as *const ::core::ffi::c_char,
        b"acl\0".as_ptr() as *const ::core::ffi::c_char,
        b"autochdir\0".as_ptr() as *const ::core::ffi::c_char,
        b"arabic\0".as_ptr() as *const ::core::ffi::c_char,
        b"autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"browsefilter\0".as_ptr() as *const ::core::ffi::c_char,
        b"byte_offset\0".as_ptr() as *const ::core::ffi::c_char,
        b"cindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_compl\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_hist\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdwin\0".as_ptr() as *const ::core::ffi::c_char,
        b"comments\0".as_ptr() as *const ::core::ffi::c_char,
        b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursorbind\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursorshape\0".as_ptr() as *const ::core::ffi::c_char,
        b"dialog_con\0".as_ptr() as *const ::core::ffi::c_char,
        b"diff\0".as_ptr() as *const ::core::ffi::c_char,
        b"digraphs\0".as_ptr() as *const ::core::ffi::c_char,
        b"eval\0".as_ptr() as *const ::core::ffi::c_char,
        b"ex_extra\0".as_ptr() as *const ::core::ffi::c_char,
        b"extra_search\0".as_ptr() as *const ::core::ffi::c_char,
        b"file_in_path\0".as_ptr() as *const ::core::ffi::c_char,
        b"filterpipe\0".as_ptr() as *const ::core::ffi::c_char,
        b"find_in_path\0".as_ptr() as *const ::core::ffi::c_char,
        b"float\0".as_ptr() as *const ::core::ffi::c_char,
        b"folding\0".as_ptr() as *const ::core::ffi::c_char,
        b"fork\0".as_ptr() as *const ::core::ffi::c_char,
        b"gettext\0".as_ptr() as *const ::core::ffi::c_char,
        b"iconv\0".as_ptr() as *const ::core::ffi::c_char,
        b"insert_expand\0".as_ptr() as *const ::core::ffi::c_char,
        b"jumplist\0".as_ptr() as *const ::core::ffi::c_char,
        b"keymap\0".as_ptr() as *const ::core::ffi::c_char,
        b"lambda\0".as_ptr() as *const ::core::ffi::c_char,
        b"langmap\0".as_ptr() as *const ::core::ffi::c_char,
        b"libcall\0".as_ptr() as *const ::core::ffi::c_char,
        b"linebreak\0".as_ptr() as *const ::core::ffi::c_char,
        b"lispindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"listcmds\0".as_ptr() as *const ::core::ffi::c_char,
        b"localmap\0".as_ptr() as *const ::core::ffi::c_char,
        b"menu\0".as_ptr() as *const ::core::ffi::c_char,
        b"mksession\0".as_ptr() as *const ::core::ffi::c_char,
        b"modify_fname\0".as_ptr() as *const ::core::ffi::c_char,
        b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
        b"multi_byte\0".as_ptr() as *const ::core::ffi::c_char,
        b"multi_lang\0".as_ptr() as *const ::core::ffi::c_char,
        b"nanotime\0".as_ptr() as *const ::core::ffi::c_char,
        b"num64\0".as_ptr() as *const ::core::ffi::c_char,
        b"packages\0".as_ptr() as *const ::core::ffi::c_char,
        b"path_extra\0".as_ptr() as *const ::core::ffi::c_char,
        b"persistent_undo\0".as_ptr() as *const ::core::ffi::c_char,
        b"profile\0".as_ptr() as *const ::core::ffi::c_char,
        b"reltime\0".as_ptr() as *const ::core::ffi::c_char,
        b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
        b"rightleft\0".as_ptr() as *const ::core::ffi::c_char,
        b"scrollbind\0".as_ptr() as *const ::core::ffi::c_char,
        b"showcmd\0".as_ptr() as *const ::core::ffi::c_char,
        b"cmdline_info\0".as_ptr() as *const ::core::ffi::c_char,
        b"shada\0".as_ptr() as *const ::core::ffi::c_char,
        b"signs\0".as_ptr() as *const ::core::ffi::c_char,
        b"smartindent\0".as_ptr() as *const ::core::ffi::c_char,
        b"startuptime\0".as_ptr() as *const ::core::ffi::c_char,
        b"statusline\0".as_ptr() as *const ::core::ffi::c_char,
        b"spell\0".as_ptr() as *const ::core::ffi::c_char,
        b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
        b"tablineat\0".as_ptr() as *const ::core::ffi::c_char,
        b"tag_binary\0".as_ptr() as *const ::core::ffi::c_char,
        b"termguicolors\0".as_ptr() as *const ::core::ffi::c_char,
        b"terminfo\0".as_ptr() as *const ::core::ffi::c_char,
        b"termresponse\0".as_ptr() as *const ::core::ffi::c_char,
        b"textobjects\0".as_ptr() as *const ::core::ffi::c_char,
        b"timers\0".as_ptr() as *const ::core::ffi::c_char,
        b"title\0".as_ptr() as *const ::core::ffi::c_char,
        b"user-commands\0".as_ptr() as *const ::core::ffi::c_char,
        b"user_commands\0".as_ptr() as *const ::core::ffi::c_char,
        b"vartabs\0".as_ptr() as *const ::core::ffi::c_char,
        b"vertsplit\0".as_ptr() as *const ::core::ffi::c_char,
        b"vimscript-1\0".as_ptr() as *const ::core::ffi::c_char,
        b"virtualedit\0".as_ptr() as *const ::core::ffi::c_char,
        b"visual\0".as_ptr() as *const ::core::ffi::c_char,
        b"visualextra\0".as_ptr() as *const ::core::ffi::c_char,
        b"vreplace\0".as_ptr() as *const ::core::ffi::c_char,
        b"wildignore\0".as_ptr() as *const ::core::ffi::c_char,
        b"wildmenu\0".as_ptr() as *const ::core::ffi::c_char,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        b"winaltkeys\0".as_ptr() as *const ::core::ffi::c_char,
        b"writebackup\0".as_ptr() as *const ::core::ffi::c_char,
        b"xattr\0".as_ptr() as *const ::core::ffi::c_char,
        b"nvim\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut x: bool = false_0 != 0;
    let mut n: bool = false_0 != 0;
    let name: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if strncasecmp(
        name as *mut ::core::ffi::c_char,
        b"patch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        if *name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
            && strlen(name) >= 11 as size_t
            && (*name.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= '1' as ::core::ffi::c_int
                && *name.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    <= '9' as ::core::ffi::c_int)
        {
            let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut major: ::core::ffi::c_int = strtoul(
                name.offset(6 as ::core::ffi::c_int as isize),
                &raw mut end,
                10 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int;
            if *end as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && ascii_isdigit(*end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                && *end.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && ascii_isdigit(*end.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                let mut minor: ::core::ffi::c_int =
                    atoi(end.offset(1 as ::core::ffi::c_int as isize));
                n = has_vim_patch(
                    atoi(end.offset(3 as ::core::ffi::c_int as isize)),
                    major * 100 as ::core::ffi::c_int + minor,
                );
            }
        } else if ascii_isdigit(*name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            n = has_vim_patch(
                atoi(name.offset(5 as ::core::ffi::c_int as isize)),
                0 as ::core::ffi::c_int,
            );
        }
    } else if strncasecmp(
        name as *mut ::core::ffi::c_char,
        b"nvim-\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        5 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = has_nvim_version(name.offset(5 as ::core::ffi::c_int as isize));
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"vim_starting\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = starting.get() != 0 as ::core::ffi::c_int;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"ttyin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = stdin_isatty.get();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"ttyout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = stdout_isatty.get();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"multi_byte_encoding\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = true_0 != 0;
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"gui_running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = ui_gui_attached();
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"syntax_items\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = syntax_present(curwin.get());
    } else if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"wsl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = true_0 != 0;
        n = has_wsl();
    }
    if !x {
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 90]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 90]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if strcasecmp(
                name as *mut ::core::ffi::c_char,
                (*has_list.ptr())[i as usize] as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                x = true_0 != 0;
                n = true_0 != 0;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    if !x {
        let save_shell_error: ::core::ffi::c_int =
            get_vim_var_nr(VV_SHELL_ERROR) as ::core::ffi::c_int;
        if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"clipboard_working\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"unnamedplus\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"pythonx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            n = eval_has_provider(
                b"python3\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
        } else if eval_has_provider(name, true_0 != 0) {
            n = true_0 != 0;
        }
        set_vim_var_nr(VV_SHELL_ERROR, save_shell_error as varnumber_T);
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
unsafe extern "C" fn has_wsl() -> bool {
    static has_wsl_0: GlobalCell<TriState> = GlobalCell::new(kNone);
    if has_wsl_0.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut o: Object = nlua_exec(
            String_0 {
                data: b"return vim.uv.os_uname()['release']:lower():match('microsoft')\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 63]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        '_c2rust_label: {
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            } else {
                __assert_fail(
                    b"!ERROR_SET(&err)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2886 as ::core::ffi::c_uint,
                    b"_Bool has_wsl(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        has_wsl_0.set(
            (if o.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && o.data.boolean as ::core::ffi::c_int == true_0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState,
        );
    }
    return has_wsl_0.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_hostname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut hostname: [::core::ffi::c_char; 256] = [0; 256];
    os_get_hostname(&raw mut hostname as *mut ::core::ffi::c_char, 256 as size_t);
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(&raw mut hostname as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_mode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
    get_mode(&raw mut buf as *mut ::core::ffi::c_char);
    if !non_zero_arg(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
    (*rettv).v_type = VAR_STRING;
}
unsafe extern "C" fn may_add_state_char(
    mut gap: *mut garray_T,
    mut include: *const ::core::ffi::c_char,
    mut c: uint8_t,
) {
    if include.is_null() || !vim_strchr(include, c as ::core::ffi::c_int).is_null() {
        ga_append(gap, c);
    }
}
pub unsafe extern "C" fn f_state(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
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
        20 as ::core::ffi::c_int,
    );
    let mut include: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        include = tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    }
    if !(stuff_empty() as ::core::ffi::c_int != 0
        && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && using_script() == 0)
    {
        may_add_state_char(&raw mut ga, include, 'm' as uint8_t);
    }
    if op_pending() {
        may_add_state_char(&raw mut ga, include, 'o' as uint8_t);
    }
    if autocmd_busy.get() {
        may_add_state_char(&raw mut ga, include, 'x' as uint8_t);
    }
    if ins_compl_active() {
        may_add_state_char(&raw mut ga, include, 'a' as uint8_t);
    }
    if !get_was_safe_state() {
        may_add_state_char(&raw mut ga, include, 'S' as uint8_t);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < get_callback_depth() && i < 3 as ::core::ffi::c_int {
        may_add_state_char(&raw mut ga, include, 'c' as uint8_t);
        i += 1;
    }
    if msg_scrolled.get() > 0 as ::core::ffi::c_int {
        may_add_state_char(&raw mut ga, include, 's' as uint8_t);
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn f_nextnonblank(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = 0;
    lnum = tv_get_lnum(argvars);
    loop {
        if lnum < 0 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
            lnum = 0 as ::core::ffi::c_int as linenr_T;
            break;
        } else {
            if *skipwhite(ml_get(lnum)) as ::core::ffi::c_int != NUL {
                break;
            }
            lnum += 1;
        }
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
pub unsafe extern "C" fn f_prevnonblank(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        while lnum >= 1 as linenr_T && *skipwhite(ml_get(lnum)) as ::core::ffi::c_int == NUL {
            lnum -= 1;
        }
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
pub unsafe extern "C" fn f_pum_getpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    pum_set_event_info((*rettv).vval.v_dict);
}
pub unsafe extern "C" fn f_pumvisible(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if pum_visible() {
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_shiftwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut col: colnr_T =
            tv_get_number_chk(argvars, ::core::ptr::null_mut::<bool>()) as colnr_T;
        if col < 0 as ::core::ffi::c_int {
            return;
        }
        (*rettv).vval.v_number = get_sw_value_col(curbuf.get(), col, false_0 != 0) as varnumber_T;
        return;
    }
    (*rettv).vval.v_number = get_sw_value(curbuf.get()) as varnumber_T;
}
pub unsafe extern "C" fn f_tabpagebuflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = firstwin.get();
    } else {
        let tp: *mut tabpage_T = find_tabpage(tv_get_number(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        ) as ::core::ffi::c_int);
        if !tp.is_null() {
            wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
        }
    }
    if !wp.is_null() {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        while !wp.is_null() {
            tv_list_append_number(
                (*rettv).vval.v_list,
                (*(*wp).w_buffer).handle as varnumber_T,
            );
            wp = (*wp).w_next;
        }
    }
}
pub unsafe extern "C" fn f_visualmode(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut str: [::core::ffi::c_char; 2] = [0; 2];
    (*rettv).v_type = VAR_STRING;
    str[0 as ::core::ffi::c_int as usize] =
        (*curbuf.get()).b_visual_mode_eval as ::core::ffi::c_char;
    str[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    (*rettv).vval.v_string = xstrdup(&raw mut str as *mut ::core::ffi::c_char);
    if non_zero_arg(argvars.offset(0 as ::core::ffi::c_int as isize)) {
        (*curbuf.get()).b_visual_mode_eval = NUL;
    }
}
pub unsafe extern "C" fn f_wildmenumode(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if wild_menu_showing.get() != 0
        || State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
            && cmdline_pum_active() as ::core::ffi::c_int != 0
    {
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_windowsversion(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xstrdup(windowsVersion.ptr() as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_wordcount(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    cursor_pos_info((*rettv).vval.v_dict);
}
