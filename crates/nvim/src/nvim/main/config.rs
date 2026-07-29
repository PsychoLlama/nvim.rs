//! Sourcing the configuration: the system vimrc, the user's, the
//! `exrc` in the working directory, and the `--cmd`/`-c` commands around them.
//!
//! `-u NONE` and `--clean` are decided here, and so is the order the four
//! sources run in.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn exe_pre_commands(mut parmp: *mut mparm_T) {
    let mut cmds: *mut *mut c_char = &raw mut (*parmp).pre_commands as *mut *mut c_char;
    let mut cnt: c_int = (*parmp).n_pre_commands;
    if cnt <= 0 as c_int {
        return;
    }
    (*curwin.get()).w_cursor.lnum = 0 as c_int as linenr_T;
    estack_push(
        ETYPE_ARGS,
        gettext(b"pre-vimrc command line\0".as_ptr() as *const c_char),
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CMDARG as scid_T;
    let mut i: c_int = 0 as c_int;
    while i < cnt {
        do_cmdline_cmd(*cmds.offset(i as isize));
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as c_int as scid_T;
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"--cmd commands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn exe_commands(mut parmp: *mut mparm_T) {
    msg_scroll.set(true_0);
    if (*parmp).tagname.is_null() && (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 0 as c_int as linenr_T;
    }
    estack_push(
        ETYPE_ARGS,
        b"command line\0".as_ptr() as *const c_char as *mut c_char,
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CARG as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    while i < (*parmp).n_commands {
        do_cmdline_cmd((*parmp).commands[i as usize]);
        if (*parmp).cmds_tofree[i as usize] != 0 {
            xfree((*parmp).commands[i as usize] as *mut c_void);
        }
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as c_int as scid_T;
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    if (*parmp).edit_type == EDIT_QF as c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as c_int,
            0 as c_int,
            false_0,
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"executing command arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn do_system_initialization() {
    let config_dirs: *mut c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut iter: *const c_void = ::core::ptr::null::<c_void>();
        let mut appname: *const c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let sysinit_suffix: [c_char; 13] = [
            PATHSEP as c_char,
            's' as c_char,
            'y' as c_char,
            's' as c_char,
            'i' as c_char,
            'n' as c_char,
            'i' as c_char,
            't' as c_char,
            '.' as c_char,
            'v' as c_char,
            'i' as c_char,
            'm' as c_char,
            NUL as c_char,
        ];
        loop {
            let mut dir: *const c_char = ::core::ptr::null::<c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let mut path_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 13]>());
            let mut vimrc: *mut c_char = xmalloc(path_len) as *mut c_char;
            memcpy(vimrc as *mut c_void, dir as *const c_void, dir_len);
            if *vimrc.offset(dir_len.wrapping_sub(1 as size_t) as isize) as c_int != PATHSEP {
                *vimrc.offset(dir_len as isize) = PATHSEP as c_char;
                dir_len = dir_len.wrapping_add(1 as size_t);
            }
            memcpy(
                vimrc.offset(dir_len as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                vimrc.offset(dir_len as isize).offset(appname_len as isize) as *mut c_void,
                &raw const sysinit_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 13]>(),
            );
            if do_source(
                vimrc,
                false_0 != 0,
                DOSO_NONE as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != FAIL
            {
                xfree(vimrc as *mut c_void);
                xfree(config_dirs as *mut c_void);
                return;
            }
            xfree(vimrc as *mut c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut c_void);
    }
    do_source(
        SYS_VIMRC_FILE.as_ptr() as *mut c_char,
        false_0 != 0,
        DOSO_NONE as c_int,
        ::core::ptr::null_mut::<c_int>(),
    );
}

pub(crate) unsafe extern "C" fn do_user_initialization() -> bool {
    let mut do_exrc: bool = p_exrc.get() != 0;
    if execute_env(b"VIMINIT\0".as_ptr() as *const c_char as *mut c_char) == OK {
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    let mut init_lua_path: *mut c_char =
        stdpaths_user_conf_subpath(b"init.lua\0".as_ptr() as *const c_char);
    let mut user_vimrc: *mut c_char =
        stdpaths_user_conf_subpath(b"init.vim\0".as_ptr() as *const c_char);
    if os_path_exists(init_lua_path) as c_int != 0
        && do_source(
            init_lua_path,
            true_0 != 0,
            DOSO_VIMRC as c_int,
            ::core::ptr::null_mut::<c_int>(),
        ) != 0
    {
        if os_path_exists(user_vimrc) {
            semsg(
                (e_conflicting_configs.ptr() as *const _) as *const c_char,
                init_lua_path,
                user_vimrc,
            );
        }
        xfree(user_vimrc as *mut c_void);
        xfree(init_lua_path as *mut c_void);
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    xfree(init_lua_path as *mut c_void);
    if do_source(
        user_vimrc,
        true_0 != 0,
        DOSO_VIMRC as c_int,
        ::core::ptr::null_mut::<c_int>(),
    ) != FAIL
    {
        do_exrc = p_exrc.get() != 0;
        if do_exrc {
            do_exrc = path_full_compare(
                VIMRC_FILE.as_ptr() as *mut c_char,
                user_vimrc,
                false_0 != 0,
                true_0 != 0,
            ) as c_uint
                != kEqualFiles as c_int as c_uint;
        }
        xfree(user_vimrc as *mut c_void);
        return do_exrc;
    }
    xfree(user_vimrc as *mut c_void);
    let config_dirs: *mut c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut appname: *const c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let mut iter: *const c_void = ::core::ptr::null::<c_void>();
        loop {
            let mut dir: *const c_char = ::core::ptr::null::<c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let init_lua_suffix: [c_char; 10] = [
                PATHSEP as c_char,
                'i' as c_char,
                'n' as c_char,
                'i' as c_char,
                't' as c_char,
                '.' as c_char,
                'l' as c_char,
                'u' as c_char,
                'a' as c_char,
                NUL as c_char,
            ];
            let mut init_lua_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 10]>());
            let mut init_lua: *mut c_char = xmalloc(init_lua_len) as *mut c_char;
            memcpy(init_lua as *mut c_void, dir as *const c_void, dir_len);
            *init_lua.offset(dir_len as isize) = PATHSEP as c_char;
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize)
                    .offset(appname_len as isize) as *mut c_void,
                &raw const init_lua_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 10]>(),
            );
            let init_vim_suffix: [c_char; 10] = [
                PATHSEP as c_char,
                'i' as c_char,
                'n' as c_char,
                'i' as c_char,
                't' as c_char,
                '.' as c_char,
                'v' as c_char,
                'i' as c_char,
                'm' as c_char,
                NUL as c_char,
            ];
            let mut init_vim_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 10]>());
            let mut init_vim: *mut c_char = xmalloc(init_vim_len) as *mut c_char;
            memcpy(init_vim as *mut c_void, dir as *const c_void, dir_len);
            *init_vim.offset(dir_len as isize) = PATHSEP as c_char;
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize)
                    .offset(appname_len as isize) as *mut c_void,
                &raw const init_vim_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 10]>(),
            );
            if os_path_exists(init_lua) as c_int != 0
                && do_source(
                    init_lua,
                    true_0 != 0,
                    DOSO_VIMRC as c_int,
                    ::core::ptr::null_mut::<c_int>(),
                ) != 0
            {
                if os_path_exists(init_vim) {
                    semsg(
                        (e_conflicting_configs.ptr() as *const _) as *const c_char,
                        init_lua,
                        init_vim,
                    );
                }
                xfree(init_vim as *mut c_void);
                xfree(init_lua as *mut c_void);
                xfree(config_dirs as *mut c_void);
                do_exrc = p_exrc.get() != 0;
                return do_exrc;
            }
            xfree(init_lua as *mut c_void);
            if do_source(
                init_vim,
                true_0 != 0,
                DOSO_VIMRC as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != FAIL
            {
                do_exrc = p_exrc.get() != 0;
                if do_exrc {
                    do_exrc = path_full_compare(
                        VIMRC_FILE.as_ptr() as *mut c_char,
                        init_vim,
                        false_0 != 0,
                        true_0 != 0,
                    ) as c_uint
                        != kEqualFiles as c_int as c_uint;
                }
                xfree(init_vim as *mut c_void);
                xfree(config_dirs as *mut c_void);
                return do_exrc;
            }
            xfree(init_vim as *mut c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut c_void);
    }
    if execute_env(b"EXINIT\0".as_ptr() as *const c_char as *mut c_char) == OK {
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    return do_exrc;
}

pub(crate) unsafe extern "C" fn do_exrc_initialization() {
    let L: *mut lua_State = get_global_lstate();
    '_c2rust_label: {
        if !L.is_null() {
        } else {
            __assert_fail(
                b"L\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                2207 as c_uint,
                b"void do_exrc_initialization(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    lua_getfield(L, LUA_GLOBALSINDEX, b"require\0".as_ptr() as *const c_char);
    lua_pushstring(L, b"vim._core.exrc\0".as_ptr() as *const c_char);
    if nlua_pcall(L, 1 as c_int, 0 as c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const c_char,
            lua_tolstring(L, -1 as c_int, ::core::ptr::null_mut::<size_t>()),
        );
    }
}

pub(crate) unsafe extern "C" fn source_startup_scripts(parmp: *const mparm_T) {
    if !(*parmp).use_vimrc.is_null() {
        if !(strequal((*parmp).use_vimrc, b"NONE\0".as_ptr() as *const c_char) as c_int != 0
            || strequal((*parmp).use_vimrc, b"NORC\0".as_ptr() as *const c_char) as c_int != 0)
        {
            if do_source(
                (*parmp).use_vimrc,
                false_0 != 0,
                DOSO_NONE as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != OK
            {
                semsg(
                    gettext((e_cannot_read_from_str_2.ptr() as *const _) as *const c_char),
                    (*parmp).use_vimrc,
                );
            }
        }
    } else if !silent_mode.get() {
        do_system_initialization();
        if do_user_initialization() {
            do_exrc_initialization();
        }
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"sourcing vimrc file(s)\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
