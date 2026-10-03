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
#![allow(unsafe_code)]

use super::attr::ADDR_TYPES;
use super::{Scope, ucmd_name};
use crate::charset::skip;
use crate::mapping::set_context_in_map_cmd;
use crate::mbyte::cluster_len;
use crate::memory::XString;
use crate::menu::set_context_in_menu_cmd;
use crate::types::CmdIdx;
use crate::types::{Candidate, ExArgt, Expand, ExpandContext, UserCmd};
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

/// Completion context for a `:command` line whose arguments start at `arg`
/// in the completion's line.
///
/// Answers where the rest of the line starts when what remains is an
/// ordinary command (the definition body), and `None` when the context has
/// been decided.
pub(crate) fn set_context_in_user_cmd(expand: &mut Expand, arg: usize) -> Option<usize> {
    let line = expand.line_cstr().to_bytes().to_vec();
    let at = |i: usize| line.get(i).copied().unwrap_or(0);
    let skipwhite = |i: usize| i + skip::white(line.get(i..).unwrap_or_default());
    let skiptowhite = |i: usize| i + skip::to_white(line.get(i..).unwrap_or_default());

    // The attributes come first.
    let mut arg = arg;
    while at(arg) == b'-' {
        arg += 1;
        let p = skiptowhite(arg);
        if at(p) != 0 {
            arg = skipwhite(p);
            continue;
        }
        // The cursor is still inside the attribute.
        let attr = &line[arg..];
        let Some(eq) = attr.iter().position(|&b| b == b'=') else {
            // No "=" yet, so complete attribute names.
            set_context(expand, ExpandContext::UserCmdFlags, arg);
            return None;
        };
        // `-complete=`, `-nargs=` and `-addr=` have values worth
        // completing too; any other attribute's value does not.
        let name = &attr[..eq];
        let value = arg + eq + 1;
        if abbreviates(name, "complete") {
            set_context(expand, ExpandContext::UserComplete, value);
        } else if abbreviates(name, "nargs") {
            set_context(expand, ExpandContext::UserNargs, value);
        } else if abbreviates(name, "addr") {
            set_context(expand, ExpandContext::UserAddrType, value);
        }
        return None;
    }

    // Then the name of the command being defined.
    let p = skiptowhite(arg);
    if at(p) == 0 {
        set_context(expand, ExpandContext::UserCommands, arg);
        return None;
    }
    // And finally an ordinary command, which the caller parses.
    Some(skipwhite(p))
}

/// Complete `context`'s values, from `pattern` in the completion's line.
fn set_context(expand: &mut Expand, context: ExpandContext, pattern: usize) {
    expand.context = context;
    expand.pattern = pattern;
}

/// Completion context for the *arguments* of a user command `cmd`, whose
/// `-complete=` chose `context`; the arguments start at `arg` in the
/// completion's line.
pub(crate) fn set_context_in_user_cmdarg(
    expand: &mut Expand,
    cmd: &CStr,
    arg: usize,
    argt: ExArgt,
    context: ExpandContext,
    forceit: bool,
) -> Option<usize> {
    if context == ExpandContext::Nothing {
        return None;
    }
    if argt.has(ExArgt::XFILE) {
        // ExArgt::XFILE: file names are handled before this call.
        return None;
    }
    if context == ExpandContext::Menus {
        return set_context_in_menu_cmd(expand, cmd, arg, forceit);
    }
    if context == ExpandContext::Commands {
        return Some(arg);
    }
    if context == ExpandContext::Mappings {
        return set_context_in_map_cmd(expand, c"map", arg, forceit, false, false, CmdIdx::map);
    }
    // The pattern is the last argument: walk to it, honouring escapes
    // and multibyte characters.
    let line = expand.line_cstr().to_bytes();
    let mut last = arg;
    let mut p = arg;
    while p < line.len() {
        if line[p] == b' ' {
            last = p + 1;
        } else if line[p] == b'\\' && p + 1 < line.len() {
            p += 1;
        }
        p += cluster_len(&line[p..]).max(1);
    }
    set_context(expand, context, last);
    None
}

/// The `idx`th user command name: buffer-local ones first, then global.
///
/// A global command shadowed by a buffer-local one of the same name is
/// answered as the empty string rather than skipped, so that the caller's
/// index keeps counting. The built-in command table's own completion
/// reaches this past its last built-in.
pub(crate) fn get_user_commands(_expand: &Expand, idx: usize) -> Option<Candidate> {
    // SAFETY: the borrow ends before anything can add or remove a command.
    let (local, global) = unsafe { (Scope::Buffer.list(), Scope::Global.list()) };
    if let Some(cmd) = local.get(idx) {
        return Some(name_candidate(cmd));
    }
    let cmd = global.get(idx - local.len())?;
    let shadowed = local.iter().any(|l| ucmd_name(l) == ucmd_name(cmd));
    Some(if shadowed {
        Candidate::Borrowed(c"")
    } else {
        name_candidate(cmd)
    })
}

/// A copy of a command's name, which lives no longer than the command.
fn name_candidate(cmd: &UserCmd) -> Candidate {
    // SAFETY: `uc_name` is NUL-terminated for the life of the entry.
    Candidate::Owned(unsafe { CStr::from_ptr(cmd.uc_name) }.to_owned())
}

/// The name of user command `idx` in the table `cmdidx` names.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn get_user_command_name(idx: c_int, cmdidx: CmdIdx) -> *mut c_char {
    let scope = match cmdidx {
        CmdIdx::USER => Scope::Global,
        CmdIdx::USER_BUF => Scope::Buffer,
        _ => return ptr::null_mut(),
    };
    // SAFETY: module contract.
    unsafe { scope.list() }
        .get(idx as usize)
        .map_or(ptr::null_mut(), |cmd| cmd.uc_name)
}

/// The `idx`th of a fixed list.
fn nth(list: &[&'static CStr], idx: usize) -> Option<Candidate> {
    list.get(idx).map(|&name| Candidate::Borrowed(name))
}

/// `expand_generic()` item getter: the `-addr=` values.
pub(crate) fn get_user_cmd_addr_type(_expand: &Expand, idx: usize) -> Option<Candidate> {
    ADDR_TYPES.get(idx).map(|row| Candidate::Borrowed(row.name))
}

/// `expand_generic()` item getter: the attribute names `:command` takes.
pub(crate) fn get_user_cmd_flags(_expand: &Expand, idx: usize) -> Option<Candidate> {
    /// Must stay alphabetical bar the last, which upstream appended.
    const USER_CMD_FLAGS: [&CStr; 10] = [
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
    nth(&USER_CMD_FLAGS, idx)
}

/// `expand_generic()` item getter: the `-nargs=` values.
pub(crate) fn get_user_cmd_nargs(_expand: &Expand, idx: usize) -> Option<Candidate> {
    nth(&[c"0", c"1", c"*", c"?", c"+"], idx)
}

/// `expand_generic()` item getter: the `-complete=` values.
///
/// The holes in [`COMMAND_COMPLETE`], and the Lua context that has a name
/// only for display, are answered as the empty string: the getter's `None`
/// is the end of the list, not a gap in it.
pub(crate) fn get_user_cmd_complete(_expand: &Expand, idx: usize) -> Option<Candidate> {
    if idx >= COMMAND_COMPLETE.len() {
        return None;
    }
    let context = c_int::try_from(idx)
        .ok()
        .and_then(|i| ExpandContext::try_from(i).ok());
    Some(match context.and_then(command_complete_name) {
        Some(name) if context != Some(ExpandContext::UserLua) => Candidate::Borrowed(name),
        _ => Candidate::Borrowed(c""),
    })
}

/// The name of completion type `expand`, or `None` when it has none.
///
/// `custom`/`customlist` render as `custom,{func}`, which is the spelling
/// `-complete=` accepts back.
pub(crate) fn cmdcomplete_type_to_str(
    expand: ExpandContext,
    compl_arg: Option<&CStr>,
) -> Option<XString> {
    let name = command_complete_name(expand).filter(|_| expand != ExpandContext::UserLua)?;
    let mut text = XString::from_cstr(name);
    if expand == ExpandContext::UserList || expand == ExpandContext::UserDefined {
        text.push_byte(b',');
        text.push_bytes(compl_arg.map_or(&b"(null)"[..], CStr::to_bytes));
    }
    Some(text)
}

/// The `EXPAND_*` context `complete_str` names, or `ExpandContext::Nothing`.
pub(crate) fn cmdcomplete_str_to_type(complete_str: &CStr) -> ExpandContext {
    let typed = complete_str.to_bytes();
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
