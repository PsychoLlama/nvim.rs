//! Completing `:command` itself, and the `-complete=` vocabulary.
//!
//! Two unrelated jobs share this file because they share one table.
//!
//! [`COMMAND_COMPLETE`] maps each `EXPAND_*` context to the name
//! `-complete=` knows it by. It is the vocabulary of `-complete=`, of
//! `nvim_create_user_command()`'s `complete` option, of `input()`'s third
//! argument and of what `:command` prints in its Complete column --
//! [`cmdcomplete_str_to_type`] and [`cmdcomplete_type_to_str`] are the two
//! directions. The table is indexed *by* the context, so its holes are
//! real: a context with no name is one `-complete=` cannot ask for.
//!
//! The rest is command-line completion of a `:command` line -- the
//! attribute names, their values, and the command name -- plus the
//! `expand_generic()` item getters those contexts are answered by. Each
//! getter is called with an increasing `idx` until it answers null, which
//! is why every bound here is "one past the last item" rather than a
//! length check the caller could have made.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::attr::ADDR_TYPES;
use super::{Scope, ucmd_name};
use crate::charset::{skiptowhite, skipwhite};
use crate::mapping::set_context_in_map_cmd;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xmalloc, xstrdup};
use crate::menu::set_context_in_menu_cmd;
use crate::os::cshim::snprintf;
use crate::types::{
    CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_map, ExArgt, ExpandContext, NUL, expand_T,
};
use ::libc::strlen;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The `-complete=` name of each completion context, indexed by the
/// `EXPAND_*` value. Must stay alphabetical by name: it is offered for
/// completion in table order.
///
/// The length is part of the contract -- every bound in this file is
/// `COMMAND_COMPLETE.len()` -- so the trailing `None`s are not padding to
/// be trimmed.
#[rustfmt::skip]
pub(super) static COMMAND_COMPLETE: [Option<&CStr>; 64] = [
    None,                     Some(c"command"),      Some(c"file"),
    Some(c"dir"),             Some(c"option"),       None,
    Some(c"tag"),             None,                  Some(c"help"),
    Some(c"buffer"),          Some(c"event"),        Some(c"menu"),
    None,                     Some(c"highlight"),    Some(c"augroup"),
    Some(c"var"),             Some(c"mapping"),      Some(c"tag_listfiles"),
    Some(c"function"),        None,                  Some(c"expression"),
    None,                     None,                  None,
    None,                     None,                  Some(c"environment"),
    None,                     Some(c"color"),        Some(c"compiler"),
    Some(c"custom"),          Some(c"customlist"),   Some(c"<Lua function>"),
    Some(c"shellcmd"),        Some(c"sign"),         None,
    Some(c"filetype"),        Some(c"file_in_path"), Some(c"syntax"),
    Some(c"locale"),          Some(c"history"),      Some(c"user"),
    Some(c"syntime"),         None,                  Some(c"packadd"),
    Some(c"messages"),        Some(c"mapclear"),     Some(c"arglist"),
    Some(c"diff_buffer"),     Some(c"breakpoint"),   Some(c"scriptnames"),
    Some(c"runtime"),         None,                  None,
    None,                     Some(c"keymap"),       Some(c"dir_in_path"),
    Some(c"shellcmdline"),    None,                  Some(c"filetypecmd"),
    None,                     Some(c"retab"),        Some(c"checkhealth"),
    Some(c"lua"),
];

/// The name completion context `arg` is known by, if it has one.
pub(super) fn command_complete_name(arg: ExpandContext) -> Option<&'static CStr> {
    usize::try_from(arg as c_int)
        .ok()
        .and_then(|arg| COMMAND_COMPLETE.get(arg).copied())
        .flatten()
}

/// C's `STRNICMP(arg, name, len) == 0`, the prefix test that lets `-com=`
/// stand for `-complete=`.
fn abbreviates(typed: &[u8], name: &str) -> bool {
    typed.len() <= name.len() && name.as_bytes()[..typed.len()].eq_ignore_ascii_case(typed)
}

/// Completion context for a `:command` line.
///
/// Answers the rest of the line when what remains is an ordinary command
/// (the definition body), and null when the context has been decided.
///
/// # Safety
/// `arg_in` must be NUL-terminated and `xp` writable.
pub(crate) unsafe fn set_context_in_user_cmd(
    xp: *mut expand_T,
    arg_in: *const c_char,
) -> *const c_char {
    let mut arg = arg_in;
    // SAFETY: caller contract; every step stays inside the line.
    // The attributes come first.
    while unsafe { *arg } == b'-' as c_char {
        arg = unsafe { arg.offset(1) };
        let p = unsafe { skiptowhite(arg) };
        if unsafe { *p } != NUL as c_char {
            arg = unsafe { skipwhite(p) };
            continue;
        }
        // The cursor is still inside the attribute.
        let Some(eq) = unsafe { CStr::from_ptr(arg) }
            .to_bytes()
            .iter()
            .position(|&b| b == b'=')
        else {
            // No "=" yet, so complete attribute names.
            unsafe { set_context(xp, ExpandContext::UserCmdFlags, arg) };
            return ptr::null();
        };
        // `-complete=`, `-nargs=` and `-addr=` have values worth
        // completing too; any other attribute's value does not.
        let name = &unsafe { CStr::from_ptr(arg) }.to_bytes()[..eq];
        let value = unsafe { arg.add(eq + 1) };
        if abbreviates(name, "complete") {
            unsafe { set_context(xp, ExpandContext::UserComplete, value) };
        } else if abbreviates(name, "nargs") {
            unsafe { set_context(xp, ExpandContext::UserNargs, value) };
        } else if abbreviates(name, "addr") {
            unsafe { set_context(xp, ExpandContext::UserAddrType, value) };
        }
        return ptr::null();
    }

    // Then the name of the command being defined.
    let p = unsafe { skiptowhite(arg) };
    if unsafe { *p } == NUL as c_char {
        unsafe { set_context(xp, ExpandContext::UserCommands, arg) };
        return ptr::null();
    }
    // And finally an ordinary command, which the caller parses.
    unsafe { skipwhite(p) }
}

/// # Safety
/// `xp` must be writable and `pattern` must outlive it.
unsafe fn set_context(xp: *mut expand_T, context: ExpandContext, pattern: *const c_char) {
    // SAFETY: caller contract.
    unsafe { (*xp).xp_context = context };
    unsafe { (*xp).xp_pattern = pattern.cast_mut() };
}

/// Completion context for the *arguments* of a user command, whose
/// `-complete=` chose `context`.
///
/// # Safety
/// `cmd` and `arg` must be NUL-terminated and `xp` writable.
pub(crate) unsafe fn set_context_in_user_cmdarg(
    cmd: *const c_char,
    arg: *const c_char,
    argt: ExArgt,
    context: ExpandContext,
    xp: *mut expand_T,
    forceit: bool,
) -> *const c_char {
    if context == ExpandContext::Nothing {
        return ptr::null();
    }
    if argt.has(ExArgt::XFILE) {
        // ExArgt::XFILE: file names are handled before this call.
        return ptr::null();
    }
    // SAFETY: caller contract.
    if context == ExpandContext::Menus {
        return unsafe { set_context_in_menu_cmd(xp, cmd, arg.cast_mut(), forceit) };
    }
    if context == ExpandContext::Commands {
        return arg;
    }
    if context == ExpandContext::Mappings {
        let (cmd, pat) = (c"map".as_ptr().cast_mut(), arg.cast_mut());
        // SAFETY: caller contract; `xp` is writable and `arg` outlives it.
        return unsafe { set_context_in_map_cmd(xp, cmd, pat, forceit, false, false, CMD_map) };
    }
    // The pattern is the last argument: walk to it, honouring escapes
    // and multibyte characters.
    let mut last = arg;
    let mut p = arg;
    while unsafe { *p } != NUL as c_char {
        if unsafe { *p } == b' ' as c_char {
            last = unsafe { p.offset(1) };
        } else if unsafe { *p } == b'\\' as c_char && unsafe { *p.offset(1) } != NUL as c_char {
            p = unsafe { p.offset(1) };
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    unsafe { set_context(xp, context, last) };
    ptr::null()
}

/// The `idx`th user command name, for the built-in command table's own
/// completion -- where user commands come after the `CMD_SIZE` built-ins.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn expand_user_command_name(idx: c_int) -> *mut c_char {
    // SAFETY: caller contract.
    unsafe { get_user_commands(ptr::null_mut(), idx - CMD_SIZE as c_int) }
}

/// The `idx`th user command name: buffer-local ones first, then global.
///
/// A global command shadowed by a buffer-local one of the same name is
/// answered as the empty string rather than skipped, so that the caller's
/// index keeps counting.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn get_user_commands(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    // SAFETY: module contract.
    let (local, global) = unsafe { (Scope::Buffer.list(), Scope::Global.list()) };
    let idx = idx as usize;
    if idx < local.len() {
        return local[idx].uc_name;
    }
    let Some(cmd) = global.get(idx - local.len()) else {
        return ptr::null_mut();
    };
    // SAFETY: module contract.
    let shadowed = unsafe { local.iter().any(|l| ucmd_name(l) == ucmd_name(cmd)) };
    if shadowed {
        c"".as_ptr().cast_mut()
    } else {
        cmd.uc_name
    }
}

/// The name of user command `idx` in the table `cmdidx` names.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn get_user_command_name(idx: c_int, cmdidx: c_int) -> *mut c_char {
    let scope = match cmdidx {
        c if c == CMD_USER as c_int => Scope::Global,
        c if c == CMD_USER_BUF as c_int => Scope::Buffer,
        _ => return ptr::null_mut(),
    };
    // SAFETY: module contract.
    unsafe { scope.list() }
        .get(idx as usize)
        .map_or(ptr::null_mut(), |cmd| cmd.uc_name)
}

/// `expand_generic()` item getter: the `-addr=` values.
pub(crate) fn get_user_cmd_addr_type(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    ADDR_TYPES
        .get(idx as usize)
        .map_or(ptr::null_mut(), |row| row.name.as_ptr().cast_mut())
}

/// `expand_generic()` item getter: the attribute names.
pub(crate) fn get_user_cmd_flags(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    /// Must stay alphabetical bar the last, which upstream appended.
    static USER_CMD_FLAGS: [&CStr; 10] = [
        c"addr",
        c"bang",
        c"bar",
        c"buffer",
        c"complete",
        c"count",
        c"nargs",
        c"range",
        c"register",
        c"keepscript",
    ];
    USER_CMD_FLAGS
        .get(idx as usize)
        .map_or(ptr::null_mut(), |name| name.as_ptr().cast_mut())
}

/// `expand_generic()` item getter: the `-nargs=` values.
pub(crate) fn get_user_cmd_nargs(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    static USER_CMD_NARGS: [&CStr; 5] = [c"0", c"1", c"*", c"?", c"+"];
    USER_CMD_NARGS
        .get(idx as usize)
        .map_or(ptr::null_mut(), |name| name.as_ptr().cast_mut())
}

/// `expand_generic()` item getter: the `-complete=` values.
///
/// The holes in [`COMMAND_COMPLETE`], and the Lua context that has a name
/// only for display, are answered as the empty string: the getter's null is
/// the end of the list, not a gap in it.
pub(crate) fn get_user_cmd_complete(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    if idx >= COMMAND_COMPLETE.len() as c_int {
        return ptr::null_mut();
    }
    match ExpandContext::try_from(idx)
        .ok()
        .and_then(command_complete_name)
    {
        Some(name) if idx != ExpandContext::UserLua as c_int => name.as_ptr().cast_mut(),
        _ => c"".as_ptr().cast_mut(),
    }
}

/// The name of completion type `expand`, as an allocated string, or null
/// when it has none.
///
/// `custom`/`customlist` render as `custom,{func}`, which is the spelling
/// `-complete=` accepts back.
///
/// # Safety
/// `compl_arg` must be NUL-terminated when `expand` is one of the two
/// custom types.
pub(crate) unsafe fn cmdcomplete_type_to_str(
    expand: ExpandContext,
    compl_arg: *const c_char,
) -> *mut c_char {
    let Some(name) = command_complete_name(expand).filter(|_| expand != ExpandContext::UserLua)
    else {
        return ptr::null_mut();
    };
    if expand != ExpandContext::UserList && expand != ExpandContext::UserDefined {
        // SAFETY: `name` is a literal.
        return unsafe { xstrdup(name.as_ptr()) };
    }
    // SAFETY: caller contract.
    let buflen = name.count_bytes() + unsafe { strlen(compl_arg) } + 2;
    let buffer = unsafe { xmalloc(buflen) }.cast::<c_char>();
    unsafe { snprintf(buffer, buflen, c"%s,%s".as_ptr(), name.as_ptr(), compl_arg) };
    buffer
}

/// The `EXPAND_*` context `complete_str` names, or `ExpandContext::Nothing`.
///
/// # Safety
/// `complete_str` must be NUL-terminated.
pub(crate) unsafe fn cmdcomplete_str_to_type(complete_str: *const c_char) -> ExpandContext {
    // SAFETY: caller contract.
    let typed = unsafe { CStr::from_ptr(complete_str).to_bytes() };
    if typed.starts_with(b"custom,") {
        return ExpandContext::UserDefined;
    }
    if typed.starts_with(b"customlist,") {
        return ExpandContext::UserList;
    }
    COMMAND_COMPLETE
        .iter()
        .position(|name| name.is_some_and(|name| name.to_bytes() == typed))
        .and_then(|i| ExpandContext::try_from(i as c_int).ok())
        .unwrap_or(ExpandContext::Nothing)
}
