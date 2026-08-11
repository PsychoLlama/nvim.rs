//! Sourcing the configuration: the system vimrc, the user's, the `exrc` in
//! the working directory, and the `--cmd`/`-c` commands around them.
//!
//! `-u NONE` and `--clean` are decided here, and so is the order the four
//! sources run in: `--cmd` commands, then the system vimrc, then the user's,
//! then `exrc`, and the `-c` commands last of all -- after the first file has
//! been loaded, which is why they are not in this module.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::lua::executor::{get_global_lstate, nlua_pcall};
use crate::src::nvim::lua::ffi::{lua_getfield, lua_pushstring, lua_tolstring};
use crate::src::nvim::main::args::execute_env;
use crate::src::nvim::main::{
    DOSO_NONE, DOSO_VIMRC, EDIT_QF, ETYPE_ARGS, FAIL, LUA_GLOBALSINDEX, OK, PATHSEP, SID_CARG,
    SID_CMDARG, SYS_VIMRC_FILE, VIMRC_FILE, current_sctx, curwin, e_cannot_read_from_str_2,
    e_conflicting_configs, exmode_active, kEqualFiles, kXDGConfigDirs, mparm_T, msg_scroll, p_exrc,
    silent_mode, time_msg_at,
};
use crate::src::nvim::memory::{strequal, xfree, xmalloc};
use crate::src::nvim::os::env::vim_env_iter;
use crate::src::nvim::os::fs::os_path_exists;
use crate::src::nvim::os::libc::{fprintf, gettext, memcpy, stderr, strlen};
use crate::src::nvim::os::stdpaths::get_appname;
use crate::src::nvim::os::stdpaths::{stdpaths_get_xdg_var, stdpaths_user_conf_subpath};
use crate::src::nvim::path::path_full_compare;
use crate::src::nvim::quickfix::qf_jump;
use crate::src::nvim::runtime::{do_source, estack_pop, estack_push};
use crate::src::nvim::types::{lua_State, qf_info_T, scid_T, size_t};

/// Run the `--cmd` commands, which come before any config.
pub(crate) unsafe fn exe_pre_commands(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block; the commands it
    // holds point into argv.
    unsafe {
        let count = (*parmp).n_pre_commands;
        if count <= 0 {
            return;
        }
        let cmds = &raw mut (*parmp).pre_commands as *mut *mut c_char;

        // Line 0 says "no line yet", so that a `--cmd` that moves the cursor
        // is not immediately overridden by the first file's position.
        (*curwin.get()).w_cursor.lnum = 0;
        estack_push(ETYPE_ARGS, gettext(c"pre-vimrc command line".as_ptr()), 0);
        (*current_sctx.ptr()).sc_sid = SID_CMDARG as scid_T;
        for i in 0..count {
            do_cmdline_cmd(*cmds.offset(i as isize));
        }
        estack_pop();
        (*current_sctx.ptr()).sc_sid = 0;

        time_msg_at(c"--cmd commands");
    }
}

/// Run the `-c` and `+cmd` commands, which come after the config and the
/// first file.
pub(crate) unsafe fn exe_commands(parmp: *mut mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block.
    unsafe {
        msg_scroll.set(1);
        if (*parmp).tagname.is_null() && (*curwin.get()).w_cursor.lnum <= 1 {
            // As in `exe_pre_commands`: let the commands decide the line.
            (*curwin.get()).w_cursor.lnum = 0;
        }

        // NB: not translated, unlike the pre-vimrc one above.
        estack_push(ETYPE_ARGS, c"command line".as_ptr() as *mut c_char, 0);
        (*current_sctx.ptr()).sc_sid = SID_CARG as scid_T;
        (*current_sctx.ptr()).sc_seq = 0;
        for i in 0..(*parmp).n_commands {
            do_cmdline_cmd((*parmp).commands[i as usize]);
            if (*parmp).cmds_tofree[i as usize] != 0 {
                xfree((*parmp).commands[i as usize] as *mut c_void);
            }
        }
        estack_pop();
        (*current_sctx.ptr()).sc_sid = 0;

        if (*curwin.get()).w_cursor.lnum == 0 {
            (*curwin.get()).w_cursor.lnum = 1;
        }
        if !exmode_active.get() {
            msg_scroll.set(0);
        }
        if (*parmp).edit_type == EDIT_QF as c_int {
            // `-q`: the commands may have changed the quickfix list.
            qf_jump(ptr::null_mut::<qf_info_T>(), 0, 0, 0);
        }

        time_msg_at(c"executing command arguments");
    }
}

/// `<dir>/<appname><suffix>`, freshly allocated.
///
/// `suffix` carries its own leading separator (`/sysinit.vim`), and the NUL
/// comes with it.
///
/// `dedup_sep` is the system path's rule and *not* the user path's: an
/// `$XDG_CONFIG_DIRS` entry that already ends in a separator would otherwise
/// produce a doubled one. Upstream only does this on the system side, so
/// this does too.
unsafe fn config_subpath(
    dir: *const c_char,
    dir_len: size_t,
    appname: *const c_char,
    appname_len: size_t,
    suffix: &CStr,
    dedup_sep: bool,
) -> *mut c_char {
    let tail = suffix.to_bytes_with_nul();
    // SAFETY: `dir[0..dir_len]` and `appname[0..appname_len]` are readable,
    // and the allocation below is large enough for the worst case (a `dir`
    // that does not end in a separator).
    unsafe {
        let path = xmalloc(dir_len + 1 + appname_len + tail.len()) as *mut c_char;
        memcpy(path as *mut c_void, dir as *const c_void, dir_len);
        let mut at = dir_len;
        if !dedup_sep || *path.add(at - 1) as c_int != PATHSEP {
            *path.add(at) = PATHSEP as c_char;
            at += 1;
        }
        memcpy(
            path.add(at) as *mut c_void,
            appname as *const c_void,
            appname_len,
        );
        at += appname_len;
        memcpy(
            path.add(at) as *mut c_void,
            tail.as_ptr() as *const c_void,
            tail.len(),
        );
        path
    }
}

/// Walk `$XDG_CONFIG_DIRS`, calling `visit` with each entry.
///
/// `visit` answers `true` to stop the walk. Answers whether it did.
unsafe fn for_each_config_dir(mut visit: impl FnMut(*const c_char, size_t) -> bool) -> bool {
    // SAFETY: `stdpaths_get_xdg_var` hands over an owned string, and
    // `vim_env_iter` hands back slices of it.
    unsafe {
        let config_dirs = stdpaths_get_xdg_var(kXDGConfigDirs);
        if config_dirs.is_null() {
            return false;
        }
        let mut iter: *const c_void = ptr::null();
        let mut stopped = false;
        loop {
            let mut dir: *const c_char = ptr::null();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 {
                break;
            }
            if visit(dir, dir_len) {
                stopped = true;
                break;
            }
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut c_void);
        stopped
    }
}

/// Source the system-wide vimrc: the first `<config dir>/<appname>/sysinit.vim`
/// that exists, or the compiled-in path if none do.
pub(crate) unsafe fn do_system_initialization() {
    // SAFETY: sources at most one file; `get_appname` hands over a borrowed
    // string that outlives the walk.
    unsafe {
        let appname = get_appname(false);
        let appname_len = strlen(appname);
        let sourced = for_each_config_dir(|dir, dir_len| {
            let vimrc = config_subpath(dir, dir_len, appname, appname_len, c"/sysinit.vim", true);
            let ok = do_source(vimrc, false, DOSO_NONE as c_int, ptr::null_mut()) != FAIL;
            xfree(vimrc as *mut c_void);
            ok
        });
        if sourced {
            return;
        }
        do_source(
            SYS_VIMRC_FILE.as_ptr() as *mut c_char,
            false,
            DOSO_NONE as c_int,
            ptr::null_mut(),
        );
    }
}

/// Try `init.lua` and then `init.vim` in one directory.
///
/// Answers `Some(do_exrc)` when one of them was sourced, and `None` when
/// neither was: `do_exrc` is off when the file that was sourced *is* the
/// `exrc` the working directory would offer, so it is not read twice.
unsafe fn source_init_pair(
    init_lua: *mut c_char,
    init_vim: *mut c_char,
    check_exrc_is_same: bool,
) -> Option<bool> {
    // SAFETY: both paths are owned NUL-terminated strings; freeing them is
    // the caller's job.
    unsafe {
        if os_path_exists(init_lua)
            && do_source(init_lua, true, DOSO_VIMRC as c_int, ptr::null_mut()) != 0
        {
            // Both present: the Lua one won, and the user should know.
            if os_path_exists(init_vim) {
                semsg_c!(e_conflicting_configs.as_ptr(), init_lua, init_vim,);
            }
            return Some(p_exrc.get() != 0);
        }
        if do_source(init_vim, true, DOSO_VIMRC as c_int, ptr::null_mut()) != FAIL {
            let mut do_exrc = p_exrc.get() != 0;
            if do_exrc && check_exrc_is_same {
                do_exrc =
                    path_full_compare(VIMRC_FILE.as_ptr() as *mut c_char, init_vim, false, true)
                        != kEqualFiles;
            }
            return Some(do_exrc);
        }
        None
    }
}

/// Source the user's config, and say whether the working directory's `exrc`
/// should be read afterwards.
///
/// The sources are tried in order and the first that works wins: `$VIMINIT`,
/// `$XDG_CONFIG_HOME/<appname>/init.{lua,vim}`, then the same pair under each
/// `$XDG_CONFIG_DIRS` entry, then `$EXINIT`.
pub(crate) unsafe fn do_user_initialization() -> bool {
    // SAFETY: sources at most one config; every path built here is freed on
    // every way out.
    unsafe {
        // Read before anything is sourced: the fall-through at the bottom
        // answers with *this*, not with what a half-sourced config left
        // behind.
        let do_exrc = p_exrc.get() != 0;

        if execute_env(c"VIMINIT".as_ptr() as *mut c_char) == OK {
            return p_exrc.get() != 0;
        }

        let init_lua_path = stdpaths_user_conf_subpath(c"init.lua".as_ptr());
        let user_vimrc = stdpaths_user_conf_subpath(c"init.vim".as_ptr());
        let home = source_init_pair(init_lua_path, user_vimrc, true);
        xfree(init_lua_path as *mut c_void);
        xfree(user_vimrc as *mut c_void);
        if let Some(do_exrc) = home {
            return do_exrc;
        }

        let appname = get_appname(false);
        let appname_len = strlen(appname);
        let mut from_dirs: Option<bool> = None;
        for_each_config_dir(|dir, dir_len| {
            let init_lua = config_subpath(dir, dir_len, appname, appname_len, c"/init.lua", false);
            let init_vim = config_subpath(dir, dir_len, appname, appname_len, c"/init.vim", false);
            from_dirs = source_init_pair(init_lua, init_vim, true);
            xfree(init_lua as *mut c_void);
            xfree(init_vim as *mut c_void);
            from_dirs.is_some()
        });
        if let Some(do_exrc) = from_dirs {
            return do_exrc;
        }

        if execute_env(c"EXINIT".as_ptr() as *mut c_char) == OK {
            return p_exrc.get() != 0;
        }
        do_exrc
    }
}

/// Read the working directory's `exrc`, which is Lua's job.
pub(crate) unsafe fn do_exrc_initialization() {
    // SAFETY: the Lua state exists by now -- `nlua_init` ran in `main_0`.
    unsafe {
        let lstate: *mut lua_State = get_global_lstate();
        // Deliberately a hard failure, not a `debug_assert!`: every line
        // below dereferences `lstate`, so a release build that carried on
        // would fault instead of saying what went wrong.
        assert!(!lstate.is_null(), "the Lua state is not initialised");
        lua_getfield(lstate, LUA_GLOBALSINDEX, c"require".as_ptr());
        lua_pushstring(lstate, c"vim._core.exrc".as_ptr());
        if nlua_pcall(lstate, 1, 0) != 0 {
            fprintf(
                stderr,
                c"%s\n".as_ptr(),
                lua_tolstring(lstate, -1, ptr::null_mut::<size_t>()),
            );
        }
    }
}

/// The whole config phase: either the one file `-u` named, or the four
/// standard sources.
///
/// `-u NONE` and `-u NORC` name no file at all and source nothing; silent
/// (batch) mode skips the standard sources too.
pub(crate) unsafe fn source_startup_scripts(parmp: *const mparm_T) {
    // SAFETY: `parmp` is the caller's live parameter block.
    unsafe {
        if !(*parmp).use_vimrc.is_null() {
            let named_none = strequal((*parmp).use_vimrc, c"NONE".as_ptr())
                || strequal((*parmp).use_vimrc, c"NORC".as_ptr());
            if !named_none
                && do_source(
                    (*parmp).use_vimrc,
                    false,
                    DOSO_NONE as c_int,
                    ptr::null_mut(),
                ) != OK
            {
                semsg_c!(
                    gettext(e_cannot_read_from_str_2.as_ptr()),
                    (*parmp).use_vimrc,
                );
            }
        } else if !silent_mode.get() {
            do_system_initialization();
            if do_user_initialization() {
                do_exrc_initialization();
            }
        }

        time_msg_at(c"sourcing vimrc file(s)");
    }
}
