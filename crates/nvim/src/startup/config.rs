//! Sourcing the configuration: the system vimrc, the user's, the `exrc` in
//! the working directory, and the `--cmd`/`-c` commands around them.
//!
//! `-u NONE` and `--clean` are decided here, and so is the order the four
//! sources run in: `--cmd` commands, then the system vimrc, then the user's,
//! then `exrc`, and the `-c` commands last of all -- after the first file has
//! been loaded, which is why they are not in this module.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::ex_docmd::do_cmdline_cmd;
use crate::lua::executor::nlua_require_module;
use crate::message::state::msg_scroll;
use crate::option::vars::p_exrc;
use crate::os::cshim::{gettext, write_c_stderr};
use crate::os::fs::path_exists;
use crate::os::state::{kEqualFiles, kXDGConfigDirs};
use crate::os::stdpaths::{get_appname, stdpaths_user_conf_subpath, xdg_var};
use crate::path::PATHSEP;
use crate::path::path_full_compare;
use crate::profile::time_msg_at;
use crate::quickfix::qf_jump_to_current;
use crate::runtime::state::{
    DOSO_NONE, DOSO_VIMRC, ETYPE_ARGS, SID_CARG, SID_CMDARG, current_sctx,
};
use crate::runtime::{do_source, estack_pop, estack_push};
use crate::startup::args::execute_env;
use crate::startup::{EDIT_QF, MainParams, SYS_VIMRC_FILE, VIMRC_FILE, silent_mode};
use crate::state::mode::exmode_active;
use crate::types::{FAIL, OK, ScriptId};

/// Run the `--cmd` commands, which come before any config.
pub(crate) fn exe_pre_commands(parm: &MainParams) {
    if parm.pre_commands.is_empty() {
        return;
    }

    // Line 0 says "no line yet", so that a `--cmd` that moves the cursor
    // is not immediately overridden by the first file's position.
    Win::current().w_cursor.lnum = 0;
    estack_push(
        ETYPE_ARGS,
        crate::types::FrameName::Static(gettext(c"pre-vimrc command line")),
        0,
    );
    current_sctx.set(current_sctx.get().with_sid(SID_CMDARG as ScriptId));
    for cmd in &parm.pre_commands {
        let _ = do_cmdline_cmd(cmd);
    }
    estack_pop();
    current_sctx.set(current_sctx.get().with_sid(0));

    time_msg_at(c"--cmd commands");
}

/// Run the `-c` and `+cmd` commands, which come after the config and the
/// first file.
pub(crate) fn exe_commands(parm: &MainParams) {
    msg_scroll.set(1);
    if parm.tagname.is_none() && Win::current().w_cursor.lnum <= 1 {
        // As in `exe_pre_commands`: let the commands decide the line.
        Win::current().w_cursor.lnum = 0;
    }

    // NB: not translated, unlike the pre-vimrc one above.
    estack_push(
        ETYPE_ARGS,
        crate::types::FrameName::Static(c"command line"),
        0,
    );
    current_sctx.set(
        current_sctx
            .get()
            .with_sid(SID_CARG as ScriptId)
            .with_seq(0),
    );
    for cmd in &parm.commands {
        let _ = do_cmdline_cmd(cmd);
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
        qf_jump_to_current();
    }

    time_msg_at(c"executing command arguments");
}

/// `<dir>/<appname><suffix>`.
///
/// `suffix` carries its own leading separator (`/sysinit.vim`).
///
/// `dedup_sep` is the system path's rule and *not* the user path's: an
/// `$XDG_CONFIG_DIRS` entry that already ends in a separator would otherwise
/// produce a doubled one. Upstream only does this on the system side, so
/// this does too.
fn config_subpath(dir: &[u8], appname: &[u8], suffix: &[u8], dedup_sep: bool) -> CString {
    let sep = PATHSEP as u8;
    let mut path = dir.to_vec();
    if !dedup_sep || dir.last() != Some(&sep) {
        path.push(sep);
    }
    path.extend_from_slice(appname);
    path.extend_from_slice(suffix);
    CString::new(path).expect("an environment value and an app name hold no NUL")
}

/// Walk `$XDG_CONFIG_DIRS`, calling `visit` with each entry, up to the first
/// empty one.
///
/// `visit` answers `true` to stop the walk. Answers whether it did.
fn for_each_config_dir(visit: impl FnMut(&[u8]) -> bool) -> bool {
    let Some(config_dirs) = xdg_var(kXDGConfigDirs) else {
        return false;
    };
    config_dirs
        .as_bytes()
        .split(|&b| b == b':')
        .take_while(|dir| !dir.is_empty())
        .any(visit)
}

/// Source the system-wide vimrc: the first `<config dir>/<appname>/sysinit.vim`
/// that exists, or the compiled-in path if none do.
pub(crate) fn do_system_initialization() {
    let appname = get_appname(false);
    let appname = appname.to_bytes();
    let sourced = for_each_config_dir(|dir| {
        let vimrc = config_subpath(dir, appname, b"/sysinit.vim", true);
        do_source(&vimrc, false, DOSO_NONE as c_int, None) != FAIL
    });
    if sourced {
        return;
    }
    do_source(SYS_VIMRC_FILE, false, DOSO_NONE as c_int, None);
}

/// Try `init.lua` and then `init.vim` in one directory.
///
/// Answers `Some(do_exrc)` when one of them was sourced, and `None` when
/// neither was: `do_exrc` is off when the file that was sourced *is* the
/// `exrc` the working directory would offer, so it is not read twice.
fn source_init_pair(init_lua: &CStr, init_vim: &CStr, check_exrc_is_same: bool) -> Option<bool> {
    if path_exists(init_lua) && do_source(init_lua, true, DOSO_VIMRC as c_int, None) != 0 {
        // Both present: the Lua one won, and the user should know.
        if path_exists(init_vim) {
            let (init_lua, init_vim) = (msg_cstr(init_lua), msg_cstr(init_vim));
            semsg!("E5422: Conflicting configs: \"{init_lua}\" \"{init_vim}\"");
        }
        return Some(p_exrc());
    }
    if do_source(init_vim, true, DOSO_VIMRC as c_int, None) != FAIL {
        let mut do_exrc = p_exrc();
        if do_exrc && check_exrc_is_same {
            let same = path_full_compare(VIMRC_FILE, init_vim, false, true);
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
    // Read before anything is sourced: the fall-through at the bottom
    // answers with *this*, not with what a half-sourced config left
    // behind.
    let do_exrc = p_exrc();

    if execute_env(c"VIMINIT").is_ok() {
        return p_exrc();
    }

    let init_lua: XString = stdpaths_user_conf_subpath(b"init.lua");
    let init_vim: XString = stdpaths_user_conf_subpath(b"init.vim");
    if let Some(do_exrc) = source_init_pair(init_lua.as_cstr(), init_vim.as_cstr(), true) {
        return do_exrc;
    }

    let appname = get_appname(false);
    let appname = appname.to_bytes();
    let mut from_dirs: Option<bool> = None;
    for_each_config_dir(|dir| {
        let init_lua = config_subpath(dir, appname, b"/init.lua", false);
        let init_vim = config_subpath(dir, appname, b"/init.vim", false);
        from_dirs = source_init_pair(&init_lua, &init_vim, true);
        from_dirs.is_some()
    });
    if let Some(do_exrc) = from_dirs {
        return do_exrc;
    }

    if execute_env(c"EXINIT").is_ok() {
        return p_exrc();
    }
    do_exrc
}

/// Read the working directory's `exrc`, which is Lua's job.
pub(crate) fn do_exrc_initialization() {
    if let Err(text) = nlua_require_module(c"vim._core.exrc") {
        // What `fprintf("%s\n")` printed, a null string included.
        let text = text.unwrap_or_else(|| b"(null)".to_vec());
        write_c_stderr(&[&text[..], b"\n"].concat());
    }
}

/// The whole config phase: either the one file `-u` named, or the four
/// standard sources.
///
/// `-u NONE` and `-u NORC` name no file at all and source nothing; silent
/// (batch) mode skips the standard sources too.
pub(crate) fn source_startup_scripts(parm: &MainParams) {
    if let Some(vimrc) = parm.use_vimrc.as_deref() {
        let named_none = vimrc == c"NONE" || vimrc == c"NORC";
        if !named_none && do_source(vimrc, false, DOSO_NONE as c_int, None) != OK {
            let vimrc = msg_cstr(vimrc);
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
