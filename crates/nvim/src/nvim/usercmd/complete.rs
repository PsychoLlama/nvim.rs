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
//! `ExpandGeneric()` item getters those contexts are answered by. Each
//! getter is called with an increasing `idx` until it answers null, which
//! is why every bound here is "one past the last item" rather than a
//! length check the caller could have made.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::attr::ADDR_TYPES;
use super::{
    EX_XFILE, EXPAND_COMMANDS, EXPAND_MAPPINGS, EXPAND_MENUS, EXPAND_NOTHING,
    EXPAND_USER_ADDR_TYPE, EXPAND_USER_CMD_FLAGS, EXPAND_USER_COMMANDS, EXPAND_USER_COMPLETE,
    EXPAND_USER_DEFINED, EXPAND_USER_LIST, EXPAND_USER_LUA, EXPAND_USER_NARGS, NUL, Scope,
    ucmd_list, ucmd_name,
};
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::mapping::set_context_in_map_cmd;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xmalloc, xstrdup};
use crate::src::nvim::menu::set_context_in_menu_cmd;
use crate::src::nvim::os::libc::{snprintf, strlen};
use crate::src::nvim::types::{CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_map, expand_T, uint32_t};
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
pub(super) fn command_complete_name(arg: c_int) -> Option<&'static CStr> {
    usize::try_from(arg)
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
pub unsafe fn set_context_in_user_cmd(xp: *mut expand_T, arg_in: *const c_char) -> *const c_char {
    let mut arg = arg_in;
    // SAFETY: caller contract; every step stays inside the line.
    unsafe {
        // The attributes come first.
        while *arg == b'-' as c_char {
            arg = arg.offset(1);
            let p = skiptowhite(arg);
            if *p != NUL {
                arg = skipwhite(p);
                continue;
            }
            // The cursor is still inside the attribute.
            let Some(eq) = CStr::from_ptr(arg)
                .to_bytes()
                .iter()
                .position(|&b| b == b'=')
            else {
                // No "=" yet, so complete attribute names.
                set_context(xp, EXPAND_USER_CMD_FLAGS, arg);
                return ptr::null();
            };
            // `-complete=`, `-nargs=` and `-addr=` have values worth
            // completing too; any other attribute's value does not.
            let name = &CStr::from_ptr(arg).to_bytes()[..eq];
            let value = arg.add(eq + 1);
            if abbreviates(name, "complete") {
                set_context(xp, EXPAND_USER_COMPLETE, value);
            } else if abbreviates(name, "nargs") {
                set_context(xp, EXPAND_USER_NARGS, value);
            } else if abbreviates(name, "addr") {
                set_context(xp, EXPAND_USER_ADDR_TYPE, value);
            }
            return ptr::null();
        }

        // Then the name of the command being defined.
        let p = skiptowhite(arg);
        if *p == NUL {
            set_context(xp, EXPAND_USER_COMMANDS, arg);
            return ptr::null();
        }
        // And finally an ordinary command, which the caller parses.
        skipwhite(p)
    }
}

/// # Safety
/// `xp` must be writable and `pattern` must outlive it.
unsafe fn set_context(xp: *mut expand_T, context: c_int, pattern: *const c_char) {
    // SAFETY: caller contract.
    unsafe {
        (*xp).xp_context = context;
        (*xp).xp_pattern = pattern.cast_mut();
    }
}

/// Completion context for the *arguments* of a user command, whose
/// `-complete=` chose `context`.
///
/// # Safety
/// `cmd` and `arg` must be NUL-terminated and `xp` writable.
pub unsafe fn set_context_in_user_cmdarg(
    cmd: *const c_char,
    arg: *const c_char,
    argt: uint32_t,
    context: c_int,
    xp: *mut expand_T,
    forceit: bool,
) -> *const c_char {
    if context == EXPAND_NOTHING {
        return ptr::null();
    }
    if argt & EX_XFILE != 0 {
        // EX_XFILE: file names are handled before this call.
        return ptr::null();
    }
    // SAFETY: caller contract.
    unsafe {
        if context == EXPAND_MENUS {
            return set_context_in_menu_cmd(xp, cmd, arg.cast_mut(), forceit);
        }
        if context == EXPAND_COMMANDS {
            return arg;
        }
        if context == EXPAND_MAPPINGS {
            return set_context_in_map_cmd(
                xp,
                c"map".as_ptr().cast_mut(),
                arg.cast_mut(),
                forceit,
                false,
                false,
                CMD_map,
            );
        }
        // The pattern is the last argument: walk to it, honouring escapes
        // and multibyte characters.
        let mut last = arg;
        let mut p = arg;
        while *p != NUL {
            if *p == b' ' as c_char {
                last = p.offset(1);
            } else if *p == b'\\' as c_char && *p.offset(1) != NUL {
                p = p.offset(1);
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
        set_context(xp, context, last);
    }
    ptr::null()
}

/// The `idx`th user command name, for the built-in command table's own
/// completion -- where user commands come after the `CMD_SIZE` built-ins.
///
/// # Safety
/// Module contract.
pub unsafe fn expand_user_command_name(idx: c_int) -> *mut c_char {
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
pub unsafe extern "C" fn get_user_commands(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    // SAFETY: module contract.
    let (local, global) = unsafe {
        (
            ucmd_list(Scope::Buffer.table()),
            ucmd_list(Scope::Global.table()),
        )
    };
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
pub unsafe fn get_user_command_name(idx: c_int, cmdidx: c_int) -> *mut c_char {
    let scope = match cmdidx {
        c if c == CMD_USER as c_int => Scope::Global,
        c if c == CMD_USER_BUF as c_int => Scope::Buffer,
        _ => return ptr::null_mut(),
    };
    // SAFETY: module contract.
    unsafe {
        ucmd_list(scope.table())
            .get(idx as usize)
            .map_or(ptr::null_mut(), |cmd| cmd.uc_name)
    }
}

/// `ExpandGeneric()` item getter: the `-addr=` values.
///
/// # Safety
/// Nothing is dereferenced; the signature is the one the item-getter table
/// requires.
pub unsafe extern "C" fn get_user_cmd_addr_type(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    ADDR_TYPES
        .get(idx as usize)
        .map_or(ptr::null_mut(), |row| row.name.as_ptr().cast_mut())
}

/// `ExpandGeneric()` item getter: the attribute names.
///
/// # Safety
/// As [`get_user_cmd_addr_type`].
pub unsafe extern "C" fn get_user_cmd_flags(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
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

/// `ExpandGeneric()` item getter: the `-nargs=` values.
///
/// # Safety
/// As [`get_user_cmd_addr_type`].
pub unsafe extern "C" fn get_user_cmd_nargs(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    static USER_CMD_NARGS: [&CStr; 5] = [c"0", c"1", c"*", c"?", c"+"];
    USER_CMD_NARGS
        .get(idx as usize)
        .map_or(ptr::null_mut(), |name| name.as_ptr().cast_mut())
}

/// `ExpandGeneric()` item getter: the `-complete=` values.
///
/// The holes in [`COMMAND_COMPLETE`], and the Lua context that has a name
/// only for display, are answered as the empty string: the getter's null is
/// the end of the list, not a gap in it.
///
/// # Safety
/// As [`get_user_cmd_addr_type`].
pub unsafe extern "C" fn get_user_cmd_complete(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    if idx >= COMMAND_COMPLETE.len() as c_int {
        return ptr::null_mut();
    }
    match command_complete_name(idx) {
        Some(name) if idx != EXPAND_USER_LUA => name.as_ptr().cast_mut(),
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
pub unsafe fn cmdcomplete_type_to_str(expand: c_int, compl_arg: *const c_char) -> *mut c_char {
    let Some(name) = command_complete_name(expand).filter(|_| expand != EXPAND_USER_LUA) else {
        return ptr::null_mut();
    };
    if expand != EXPAND_USER_LIST && expand != EXPAND_USER_DEFINED {
        // SAFETY: `name` is a literal.
        return unsafe { xstrdup(name.as_ptr()) };
    }
    // SAFETY: caller contract.
    unsafe {
        let buflen = name.count_bytes() + strlen(compl_arg) + 2;
        let buffer = xmalloc(buflen).cast::<c_char>();
        snprintf(buffer, buflen, c"%s,%s".as_ptr(), name.as_ptr(), compl_arg);
        buffer
    }
}

/// The `EXPAND_*` context `complete_str` names, or `EXPAND_NOTHING`.
///
/// # Safety
/// `complete_str` must be NUL-terminated.
pub unsafe fn cmdcomplete_str_to_type(complete_str: *const c_char) -> c_int {
    // SAFETY: caller contract.
    let typed = unsafe { CStr::from_ptr(complete_str).to_bytes() };
    if typed.starts_with(b"custom,") {
        return EXPAND_USER_DEFINED;
    }
    if typed.starts_with(b"customlist,") {
        return EXPAND_USER_LIST;
    }
    COMMAND_COMPLETE
        .iter()
        .position(|name| name.is_some_and(|name| name.to_bytes() == typed))
        .map_or(EXPAND_NOTHING, |i| i as c_int)
}
