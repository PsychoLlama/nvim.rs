//! `vim._with()`: run a function with the editor state temporarily swapped.
//!
//! One function, because unwinding it is one linear sequence: the option
//! table at slot 1 is read into a set of `:command` modifier flags plus at
//! most one of a buffer or a window, those are applied, the callback at slot
//! 2 is called through `lua_pcall`, and everything is put back in reverse —
//! through the same `try_enter`/`try_leave` bracket the API uses.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::TRY_STATE_INIT;
use crate::api::private::helpers::{try_enter, try_leave};
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::eval::window::{win_execute_after, win_execute_before};
use crate::ex_docmd::CmdModScope;
use crate::log::g_min_log_level;
use crate::lua::ffi::LUA_MULTRET;
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::narrow::number_as_int;
use crate::types::{AcoSave, CmdMod, CmdModFlags, Error, Pos, SwitchWin, WinExecute};
use crate::window::win_find_tabpage;
use crate::winlayer::{self, Buf, Win};

/// The context keys that are plain `:command` modifiers: a truthy value ors
/// the flag in, anything else is ignored.
const FLAG_KEYS: [(&CStr, CmdModFlags); 11] = [
    (c"sandbox", CmdModFlags::SANDBOX),
    (c"silent", CmdModFlags::SILENT),
    (c"emsg_silent", CmdModFlags::ERRSILENT),
    (c"unsilent", CmdModFlags::UNSILENT),
    (c"noautocmd", CmdModFlags::NOAUTOCMD),
    (c"hide", CmdModFlags::HIDE),
    (c"keepalt", CmdModFlags::KEEPALT),
    (c"keepmarks", CmdModFlags::KEEPMARKS),
    (c"keepjumps", CmdModFlags::KEEPJUMPS),
    (c"lockmarks", CmdModFlags::LOCKMARKS),
    (c"keeppatterns", CmdModFlags::KEEPPATTERNS),
];

/// An all-zero [`WinExecute`]; `win_execute_before` fills it.
const WIN_EXECUTE_INIT: WinExecute = WinExecute {
    wp: None,
    curpos: Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    },
    cwd: None,
    apply_acd: false,
    save_sfname: None,
    switchwin: SwitchWin {
        sw_curwin: None,
        sw_curtab: None,
        sw_same_win: false,
        sw_visual_active: false,
    },
};

/// `luaL_checkinteger` of the value on top, narrowed as the C cast does.
fn int_value(stack: &Stack) -> Result<c_int, Raise> {
    let n = stack.check_integer(At::TOP)?;
    Ok(number_as_int(i64::try_from(n).unwrap_or_default()))
}

/// `vim._with_c(context, fn)`: the context table at slot 1, the callback at
/// slot 2.
pub(crate) fn with(stack: &mut Stack) -> Result<c_int, Raise> {
    let mut flags = CmdModFlags::NONE;
    let mut buf: Option<Buf> = None;
    let mut win: Option<Win> = None;
    let mut log_level: c_int = -1;

    if stack.kind(At::arg(1)) != Kind::Table {
        return Err(Raise::argument(1, "table expected"));
    }
    stack.push_nil(); // [dict, ..., nil]
    while stack.next(At::arg(1)) {
        // [dict, ..., key, value]
        if stack.kind(At::from_top(2)) == Kind::String {
            let key: &CStr = stack.c_string(At::from_top(2)).unwrap_or_default();
            if key == c"buf" {
                buf = winlayer::buffer(int_value(stack)?);
            } else if key == c"win" {
                win = winlayer::window(int_value(stack)?);
            } else if key == c"log_level" {
                log_level = int_value(stack)?;
            } else if stack.to_boolean(At::TOP) {
                for (name, flag) in FLAG_KEYS {
                    if key == name {
                        flags |= flag;
                    }
                }
            }
        }
        // Pop the value; the walk pops the key.
        stack.pop(1); // [dict, ..., key]
    }
    let mut failed = false;
    let mut rets: c_int = 0;

    if flags.has(CmdModFlags::ERRSILENT) {
        // CmdModFlags::ERRSILENT must imply CmdModFlags::SILENT, or
        // apply_cmdmod() and undo_cmdmod() won't work properly.
        flags |= CmdModFlags::SILENT;
    }

    let save_min_log_level = g_min_log_level.get();
    if log_level >= 0 {
        g_min_log_level.set(log_level);
    }
    let mods = CmdModScope::enter(CmdMod {
        cmod_flags: flags,
        ..CmdMod::default()
    });

    let mut err = Error::none();
    let mut tstate = TRY_STATE_INIT;
    try_enter(&mut tstate);
    {
        let mut aco = AcoSave::default();
        let mut win_execute_args = WIN_EXECUTE_INIT;

        // A window that cannot be entered leaves everything below
        // untouched: no call, no results, and nothing to restore.
        let entered = if let Some(win) = win {
            let tabpage = win_find_tabpage(win.id()).expect("a live window is on a tab page");
            win_execute_before(&mut win_execute_args, win, tabpage)
        } else {
            if let Some(buf) = buf {
                aucmd_prepbuf(&mut aco, buf);
            }
            true
        };

        if entered {
            let s = stack.top().index();
            stack.push_value(At::arg(2));
            failed = stack.pcall(0, LUA_MULTRET, None).is_err();
            rets = stack.top().index() - s;

            if win.is_some() {
                win_execute_after(&mut win_execute_args);
            } else if buf.is_some() {
                aucmd_restbuf(&mut aco);
            }
        }
    }
    err.absorb(try_leave(&tstate));

    drop(mods);
    if log_level >= 0 {
        g_min_log_level.set(save_min_log_level);
    }

    if failed {
        return Err(Raise::top_value());
    } else if err.is_set() {
        return Err(Raise::message(err.message_or_empty().to_bytes()));
    }
    Ok(rets)
}
