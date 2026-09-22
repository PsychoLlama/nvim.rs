//! Sourcing the configuration: the system vimrc, the user's, the `exrc` in
//! the working directory, and the `--cmd`/`-c` commands around them.
//!
//! `-u NONE` and `--clean` are decided here, and so is the order the four
//! sources run in: `--cmd` commands, then the system vimrc, then the user's,
//! then `exrc`, and the `-c` commands last of all -- after the first file has
//! been loaded, which is why they are not in this module.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::{Live, Win};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::ex_docmd::do_cmdline_cmd;
use crate::lua::executor::{get_global_lstate, nlua_pcall};
use crate::lua::ffi::{lua_getfield, lua_pushstring, lua_tolstring};
use crate::lua::state::LUA_GLOBALSINDEX;
use crate::memory::{strequal, xfree, xmalloc};
use crate::message::state::msg_scroll;
use crate::option::vars::p_exrc;
use crate::os::cshim::{gettext, stderr};
use crate::os::env::vim_env_iter;
use crate::os::fs::os_path_exists;
use crate::os::state::{kEqualFiles, kXDGConfigDirs};
use crate::os::stdpaths::{get_appname, stdpaths_get_xdg_var, stdpaths_user_conf_subpath};
use crate::path::PATHSEP;
use crate::path::path_full_compare;
use crate::profile::time_msg_at;
use crate::quickfix::qf_jump;
use crate::runtime::state::{
    DOSO_NONE, DOSO_VIMRC, ETYPE_ARGS, SID_CARG, SID_CMDARG, current_sctx,
};
use crate::runtime::{do_source, estack_pop, estack_push};
use crate::startup::args::execute_env;
use crate::startup::{EDIT_QF, MainParams, SYS_VIMRC_FILE, VIMRC_FILE, silent_mode};
use crate::state::mode::exmode_active;
use crate::types::{FAIL, OK, QfInfo, ScriptId, lua_State, size_t};
use ::libc::fprintf;

/// The parameter block `main` filled in, which outlives every call here.
type Mp = Live<MainParams>;

/// Run the `--cmd` commands, which come before any config.
///
/// # Safety
///
/// `parmp` must point at the startup parameters.
pub(crate) unsafe fn exe_pre_commands(parmp: *mut MainParams) {
    // SAFETY: `parmp` is the caller's live parameter block; the commands it
    // holds point into argv.
    let count = unsafe { (*parmp).n_pre_commands };
    if count <= 0 {
        return;
    }
    let cmds = unsafe { &raw mut (*parmp).pre_commands } as *mut *mut c_char;

    // Line 0 says "no line yet", so that a `--cmd` that moves the cursor
    // is not immediately overridden by the first file's position.
    Win::current().w_cursor.lnum = 0;
    estack_push(
        ETYPE_ARGS,
        gettext(c"pre-vimrc command line").as_ptr().cast_mut(),
        0,
    );
    current_sctx.set(current_sctx.get().with_sid(SID_CMDARG as ScriptId));
    for i in 0..count {
        // SAFETY: the caller's array of `count` NUL-terminated commands.
        let _ = do_cmdline_cmd(unsafe { cstr::at(*cmds.offset(i as isize)) });
    }
    estack_pop();
    current_sctx.set(current_sctx.get().with_sid(0));

    time_msg_at(c"--cmd commands");
}

/// Run the `-c` and `+cmd` commands, which come after the config and the
/// first file.
///
/// # Safety
///
/// `parmp` must point at the startup parameters.
pub(crate) unsafe fn exe_commands(parmp: *mut MainParams) {
    // SAFETY: `parmp` is the caller's live parameter block.
    let parm = unsafe { Mp::new(parmp) };
    msg_scroll.set(1);
    if parm.tagname.is_null() && Win::current().w_cursor.lnum <= 1 {
        // As in `exe_pre_commands`: let the commands decide the line.
        Win::current().w_cursor.lnum = 0;
    }

    // NB: not translated, unlike the pre-vimrc one above.
    estack_push(ETYPE_ARGS, c"command line".as_ptr() as *mut c_char, 0);
    current_sctx.set(
        current_sctx
            .get()
            .with_sid(SID_CARG as ScriptId)
            .with_seq(0),
    );
    for i in 0..parm.n_commands {
        let cmd = parm.commands[i as usize];
        // SAFETY: a NUL-terminated command out of the argument vector.
        let _ = do_cmdline_cmd(unsafe { cstr::at(cmd) });
        if parm.cmds_tofree[i as usize] != 0 {
            unsafe { xfree(cmd as *mut c_void) };
        }
    }
    estack_pop();
    current_sctx.set(current_sctx.get().with_sid(0));

    if Win::current().w_cursor.lnum == 0 {
        Win::current().w_cursor.lnum = 1;
    }
    if !exmode_active.get() {
        msg_scroll.set(0);
    }
    if parm.edit_type == EDIT_QF as c_int {
        // `-q`: the commands may have changed the quickfix list.
        unsafe { qf_jump(ptr::null_mut::<QfInfo>(), 0, 0, 0) };
    }

    time_msg_at(c"executing command arguments");
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
///
/// # Safety
///
/// `dir` must point at a NUL-terminated string. `appname` must point at a
/// NUL-terminated string.
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
    let path = unsafe { xmalloc(dir_len + 1 + appname_len + tail.len()) } as *mut c_char;
    let into = path.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(dir.cast(), dir_len) };
    let mut at = dir_len;
    if !dedup_sep || unsafe { *path.add(at - 1) } as c_int != PATHSEP {
        unsafe { *path.add(at) = PATHSEP as c_char };
        at += 1;
    }
    let into = unsafe { path.add(at) } as *mut c_void;
    let into = into.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(appname.cast(), appname_len) };
    at += appname_len;
    let into = unsafe { path.add(at) } as *mut c_void;
    let into = into.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(tail.as_ptr().cast(), tail.len()) };
    path
}

/// Walk `$XDG_CONFIG_DIRS`, calling `visit` with each entry.
///
/// `visit` answers `true` to stop the walk. Answers whether it did.
fn for_each_config_dir(mut visit: impl FnMut(*const c_char, size_t) -> bool) -> bool {
    // SAFETY: `stdpaths_get_xdg_var` hands over an owned string, and
    // `vim_env_iter` hands back slices of it.
    let config_dirs = stdpaths_get_xdg_var(kXDGConfigDirs);
    if config_dirs.is_null() {
        return false;
    }
    let mut iter: *const c_void = ptr::null();
    let mut stopped = false;
    loop {
        let mut dir: *const c_char = ptr::null();
        let mut dir_len: size_t = 0;
        let (at, len) = (&raw mut dir, &raw mut dir_len);
        iter = unsafe { vim_env_iter(':' as c_char, config_dirs, iter, at, len) };
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
    unsafe { xfree(config_dirs as *mut c_void) };
    stopped
}

/// Source the system-wide vimrc: the first `<config dir>/<appname>/sysinit.vim`
/// that exists, or the compiled-in path if none do.
pub(crate) fn do_system_initialization() {
    let appname = get_appname(false);
    let appname_len = appname.count_bytes();
    // SAFETY: sources at most one file; `appname` outlives the walk.
    let appname = appname.as_ptr();
    let sourced = unsafe {
        for_each_config_dir(|dir, dir_len| {
            let vimrc = config_subpath(dir, dir_len, appname, appname_len, c"/sysinit.vim", true);
            let ok = do_source(vimrc, false, DOSO_NONE as c_int, ptr::null_mut()) != FAIL;
            xfree(vimrc as *mut c_void);
            ok
        })
    };
    if sourced {
        return;
    }
    let sys_vimrc = SYS_VIMRC_FILE.as_ptr() as *mut c_char;
    unsafe { do_source(sys_vimrc, false, DOSO_NONE as c_int, ptr::null_mut()) };
}

/// Try `init.lua` and then `init.vim` in one directory.
///
/// Answers `Some(do_exrc)` when one of them was sourced, and `None` when
/// neither was: `do_exrc` is off when the file that was sourced *is* the
/// `exrc` the working directory would offer, so it is not read twice.
///
/// # Safety
///
/// `init_lua` must point at a NUL-terminated string, unaliased for the call.
/// `init_vim` must point at a NUL-terminated string, unaliased for the call.
unsafe fn source_init_pair(
    init_lua: *mut c_char,
    init_vim: *mut c_char,
    check_exrc_is_same: bool,
) -> Option<bool> {
    // SAFETY: both paths are owned NUL-terminated strings; freeing them is
    // the caller's job.
    if unsafe { os_path_exists(init_lua) }
        && unsafe { do_source(init_lua, true, DOSO_VIMRC as c_int, ptr::null_mut()) } != 0
    {
        // Both present: the Lua one won, and the user should know.
        if unsafe { os_path_exists(init_vim) } {
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (init_lua, init_vim) = unsafe { (c_str(init_lua), c_str(init_vim)) };
            semsg!("E5422: Conflicting configs: \"{init_lua}\" \"{init_vim}\"");
        }
        return Some(p_exrc());
    }
    if unsafe { do_source(init_vim, true, DOSO_VIMRC as c_int, ptr::null_mut()) } != FAIL {
        let mut do_exrc = p_exrc();
        if do_exrc && check_exrc_is_same {
            let vimrc = VIMRC_FILE.as_ptr() as *mut c_char;
            let same = unsafe { path_full_compare(vimrc, init_vim, false, true) };
            do_exrc = same != kEqualFiles;
        }
        return Some(do_exrc);
    }
    None
}

/// Source the user's config, and say whether the working directory's `exrc`
/// should be read afterwards.
///
/// The sources are tried in order and the first that works wins: `$VIMINIT`,
/// `$XDG_CONFIG_HOME/<appname>/init.{lua,vim}`, then the same pair under each
/// `$XDG_CONFIG_DIRS` entry, then `$EXINIT`.
pub(crate) fn do_user_initialization() -> bool {
    // SAFETY: sources at most one config; every path built here is freed on
    // every way out.
    // Read before anything is sourced: the fall-through at the bottom
    // answers with *this*, not with what a half-sourced config left
    // behind.
    let do_exrc = p_exrc();

    if unsafe { execute_env(c"VIMINIT".as_ptr() as *mut c_char) }.is_ok() {
        return p_exrc();
    }

    let init_lua_path = unsafe { stdpaths_user_conf_subpath(c"init.lua".as_ptr()) };
    let user_vimrc = unsafe { stdpaths_user_conf_subpath(c"init.vim".as_ptr()) };
    let home = unsafe { source_init_pair(init_lua_path, user_vimrc, true) };
    unsafe { xfree(init_lua_path as *mut c_void) };
    unsafe { xfree(user_vimrc as *mut c_void) };
    if let Some(do_exrc) = home {
        return do_exrc;
    }

    let appname = get_appname(false);
    let appname_len = appname.count_bytes();
    let appname = appname.as_ptr();
    let mut from_dirs: Option<bool> = None;
    unsafe {
        for_each_config_dir(|dir, dir_len| {
            let init_lua = config_subpath(dir, dir_len, appname, appname_len, c"/init.lua", false);
            let init_vim = config_subpath(dir, dir_len, appname, appname_len, c"/init.vim", false);
            from_dirs = source_init_pair(init_lua, init_vim, true);
            xfree(init_lua as *mut c_void);
            xfree(init_vim as *mut c_void);
            from_dirs.is_some()
        })
    };
    if let Some(do_exrc) = from_dirs {
        return do_exrc;
    }

    if unsafe { execute_env(c"EXINIT".as_ptr() as *mut c_char) }.is_ok() {
        return p_exrc();
    }
    do_exrc
}

/// Read the working directory's `exrc`, which is Lua's job.
pub(crate) fn do_exrc_initialization() {
    // SAFETY: the Lua state exists by now -- `nlua_init` ran in `main_0`.
    let lstate: *mut lua_State = get_global_lstate();
    // Deliberately a hard failure, not a `debug_assert!`: every line
    // below dereferences `lstate`, so a release build that carried on
    // would fault instead of saying what went wrong.
    assert!(!lstate.is_null(), "the Lua state is not initialised");
    unsafe { lua_getfield(lstate, LUA_GLOBALSINDEX, c"require".as_ptr()) };
    unsafe { lua_pushstring(lstate, c"vim._core.exrc".as_ptr()) };
    if unsafe { nlua_pcall(lstate, 1, 0) } != 0 {
        let msg = unsafe { lua_tolstring(lstate, -1, ptr::null_mut::<size_t>()) };
        unsafe { fprintf(stderr, c"%s\n".as_ptr(), msg) };
    }
}

/// The whole config phase: either the one file `-u` named, or the four
/// standard sources.
///
/// `-u NONE` and `-u NORC` name no file at all and source nothing; silent
/// (batch) mode skips the standard sources too.
///
/// # Safety
///
/// `parmp` must point at the startup parameters.
pub(crate) unsafe fn source_startup_scripts(parmp: *const MainParams) {
    // SAFETY: `parmp` is the caller's live parameter block.
    if !unsafe { (*parmp).use_vimrc }.is_null() {
        let named_none = unsafe { strequal((*parmp).use_vimrc, c"NONE".as_ptr()) }
            || unsafe { strequal((*parmp).use_vimrc, c"NORC".as_ptr()) };
        let vimrc = unsafe { (*parmp).use_vimrc };
        if !named_none
            && unsafe { do_source(vimrc, false, DOSO_NONE as c_int, ptr::null_mut()) } != OK
        {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let vimrc = unsafe { c_str(vimrc) };
            semsg!("E282: Cannot read from \"{vimrc}\"");
        }
    } else if !silent_mode.get() {
        do_system_initialization();
        if do_user_initialization() {
            do_exrc_initialization();
        }
    }

    time_msg_at(c"sourcing vimrc file(s)");
}
