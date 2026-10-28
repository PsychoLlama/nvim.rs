//! User-defined commands: `:command`, `:comclear`, `:delcommand`, and the
//! lookup that turns a typed name back into one.
//!
//! A user command is a [`UserCmd`] -- a name, a replacement string, the
//! `EX_*` flags its attributes imply, and the completion it wants for its
//! arguments. They live in two sorted arrays: the global [`USER_COMMANDS`] and the
//! current buffer's `b_ucmds`. Everything that walks them looks at the
//! buffer-local table first and the global one second, so a `-buffer`
//! command shadows a global one of the same name; [`Scope`] is that walk.
//!
//! This file is the definition side. Its four neighbours are the rest:
//!
//! - [`attr`] parses `-nargs=`, `-range=`, `-count=`, `-addr=`, `-complete=`
//!   and the flag attributes into the `argt`/`def`/`compl` triple that
//!   [`uc_add_command`] stores.
//! - [`complete`] is command-line completion *of* `:command` itself, plus
//!   the name-to-`EXPAND_*` mapping that `-complete=` and the API share.
//! - [`expand`] runs a command: `<args>`, `<line1>`, `<mods>` and the rest
//!   of the `<...>` codes, expanded into the replacement string.
//! - [`list`] renders them, for `:command` with no arguments and for
//!   `nvim_get_commands()`.
//!
//! # Borrowing a table
//!
//! A table is lent to a closure ([`Table::with`]), never handed out as a
//! slice: growing a `Vec` moves its contents, so a borrow must not survive
//! anything that can add or remove a command. A closure that has to call
//! out -- to print, to run Lua, to run the command's body -- copies what it
//! needs first (the listing's rows, a Lua command's facts, a Vim command's
//! expanded body), and an entry leaves its table before it is dropped.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

mod attr;
mod complete;
mod expand;
mod list;
#[cfg(test)]
mod tests;

pub(crate) use attr::{parse_addr_type_arg, parse_compl_arg};
pub(crate) use complete::{
    cmdcomplete_str_to_type, cmdcomplete_type_to_str, get_user_cmd_addr_type,
    get_user_cmd_complete, get_user_cmd_flags, get_user_cmd_nargs, get_user_commands,
    set_context_in_user_cmd, set_context_in_user_cmdarg, user_command_name,
};
pub(crate) use expand::{
    LuaCommand, do_ucmd, uc_mods, uc_nargs_upper_bound, uc_split_args_iter, win_cmd_modifiers,
};
pub(crate) use list::commands_array;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::cstr;
use crate::ex_docmd::ends_excmd;
use crate::global_cell::GlobalCell;
use crate::keycodes::replace_termcodes;
use crate::lua::executor::nlua_set_sctx;
use crate::memory::XString;
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_bytes, msg_cstr};
use crate::option::vars::p_cpo;
use crate::os::cshim::gettext;
use crate::runtime::sourcing_lnum;
use crate::runtime::state::current_sctx;
use crate::semsg;
use crate::tr_c;
use crate::types::CmdIdx;
use crate::types::{
    CmdAddr, ExArg, ExArgt, Expand, ExpandContext, Failed, LuaRef, UserCmd, int64_t,
};
use crate::window::prevwin_curwin;
use crate::winlayer::Buf;
use core::cmp::Ordering;
use core::ffi::{CStr, c_int};
use core::mem;
use std::ffi::CString;
use std::rc::Rc;

pub(crate) const LUA_NOREF: c_int = -2;

/// The global user commands, sorted by name. A buffer's own live in its
/// `b_ucmds`, which is the same `Vec`; [`Table`] is the pair.
///
/// The cell's address is never handed out: a read borrows it just long
/// enough to answer a closure, and a write is a closure that cannot re-enter.
static USER_COMMANDS: GlobalCell<Vec<UserCmd>> = GlobalCell::new(Vec::new());

/// One command table: the global one, or the one inside a buffer.
///
/// The two are the same `Vec<UserCmd>` in different places, and every walker
/// here is written against this so that one function serves both. It is the
/// *where*; [`Scope`] is the *which of the two the current buffer sees*.
#[derive(Clone, Copy)]
pub(crate) enum Table {
    /// The global [`USER_COMMANDS`].
    Global,
    /// `buf`'s own `b_ucmds`.
    Buffer(Buf),
}

impl Table {
    /// Run `f` on the commands in this table.
    ///
    /// `f` holds a borrow of the table, so it must not add or remove a
    /// command -- nor run anything that might: copy out what is needed past
    /// the borrow.
    pub(crate) fn with<R>(self, f: impl FnOnce(&[UserCmd]) -> R) -> R {
        match self {
            Table::Global => USER_COMMANDS.with(|cmds| f(cmds)),
            Table::Buffer(buf) => f(&buf.b_ucmds),
        }
    }

    /// Run `f` on this table, mutably.
    ///
    /// `f` holds the only `&mut` to the table, so ruling 6 makes it a leaf:
    /// it must not re-enter the editor. Dropping an entry gives its Lua
    /// references back, which calls into Lua, so every caller here moves
    /// the entry *out* under `f` and drops it afterwards.
    fn with_mut<R>(self, f: impl FnOnce(&mut Vec<UserCmd>) -> R) -> R {
        match self {
            Table::Global => USER_COMMANDS.with_mut(f),
            Table::Buffer(mut buf) => f(&mut buf.b_ucmds),
        }
    }
}

/// Which of the two command tables a walk is standing on.
///
/// Upstream asks this by comparing the `GArray *` it is walking against
/// `&ucmds`; the identity test is the only thing distinguishing "this entry
/// is buffer-local" from "this entry is global", and it appears in four
/// different walks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// The current buffer's `b_ucmds`, searched first.
    Buffer,
    /// The global [`USER_COMMANDS`], searched second.
    Global,
}

impl Scope {
    /// Both tables in search order.
    pub(crate) const BOTH: [Scope; 2] = [Scope::Buffer, Scope::Global];

    /// The table this scope names.
    pub(crate) fn table(self) -> Table {
        match self {
            Scope::Buffer => Table::Buffer(prevwin_curwin().buffer()),
            Scope::Global => Table::Global,
        }
    }

    /// The scope a user command's `cmdidx` names, or `None` for a built-in.
    pub(crate) fn of(cmdidx: CmdIdx) -> Option<Scope> {
        match cmdidx {
            CmdIdx::USER => Some(Scope::Global),
            CmdIdx::USER_BUF => Some(Scope::Buffer),
            _ => None,
        }
    }

    /// [`Table::with`] on this scope's table.
    pub(crate) fn with<R>(self, f: impl FnOnce(&[UserCmd]) -> R) -> R {
        self.table().with(f)
    }
}

/// A command's name.
pub(crate) fn ucmd_name(cmd: &UserCmd) -> &[u8] {
    cmd.name.to_bytes()
}

/// Search both tables for a command matching `args.cmd`.
///
/// Sets `args.cmdidx`, `args.argt`, `args.useridx` and `args.addr_type`,
/// and answers where the command name ends -- which may be *before* `end`,
/// because the match may be followed immediately by a count that `end` has
/// already skipped. Answers `None` when nothing matched.
///
/// `full` is set when the match was exact, `expand` filled in for completion
/// and `complp` given the command's completion type.
pub(crate) fn find_ucmd(
    excmd: &mut ExArg,
    end: usize,
    full: Option<&mut bool>,
    mut expand: Option<&mut Expand>,
    mut complp: Option<&mut ExpandContext>,
) -> Option<usize> {
    let cmd = excmd.line.cmd;
    let typed = excmd.line.slice_at(cmd, end - cmd);

    let typed_len = typed.len();
    let mut exact_match = false;
    let mut matchlen = 0;
    let mut found = false;
    let mut possible = false;
    // A buffer-local command matched ambiguously; only a full global match
    // is accepted then.
    let mut amb_local = false;

    for scope in Scope::BOTH {
        // Nothing in the walk can add or remove a command: it only copies
        // facts out.
        let walked = scope.with(|cmds| {
            for (j, uc) in cmds.iter().enumerate() {
                let name = ucmd_name(uc);
                let (k, at_nul) = match_prefix(typed, name);
                // A match up to a digit means there may be another command
                // *including* the digit that should be preferred.
                if !(k == typed.len() || (at_nul && ascii_isdigit(c_int::from(typed[k])))) {
                    continue;
                }
                if k == typed.len() && found && !at_nul {
                    if scope == Scope::Global {
                        return None;
                    }
                    amb_local = true;
                }
                if found && !(k == typed.len() && at_nul) {
                    continue;
                }
                if k == typed.len() {
                    found = true;
                } else {
                    possible = true;
                }
                excmd.cmdidx = if scope == Scope::Global {
                    CmdIdx::USER
                } else {
                    CmdIdx::USER_BUF
                };
                excmd.argt = uc.argt;
                excmd.useridx = c_int::try_from(j).expect("a table index fits");
                excmd.addr_type = uc.addr_type;
                if let Some(complp) = complp.as_deref_mut() {
                    *complp = uc.complete;
                }
                if let Some(expand) = expand.as_deref_mut() {
                    expand.luaref = uc.complete_luaref;
                    expand.arg = uc.complete_arg.as_deref().map(XString::from_cstr);
                    expand.script_ctx = uc.script_ctx;
                    expand.script_ctx.sc_lnum += sourcing_lnum();
                }
                // Do not look for further abbreviations of an exact match.
                matchlen = k;
                if k == typed.len() && at_nul {
                    exact_match = true;
                    amb_local = false;
                    return Some(true);
                }
            }
            Some(false)
        });
        match walked {
            // An ambiguous global match.
            None => return None,
            // Stop on a full match; otherwise fall through to the global table.
            Some(true) => break,
            Some(false) => {}
        }
    }

    if let Some(full) = full
        && exact_match
    {
        *full = true;
    }
    if amb_local {
        if let Some(expand) = expand {
            expand.context = ExpandContext::Unsuccessful;
        }
        return None;
    }
    if found || possible {
        // The match may be followed immediately by a number: move back onto
        // it. `matchlen <= typed.len()`, so this stays inside the line.
        return Some(end + matchlen - typed_len);
    }
    Some(end)
}

/// How far `typed` and `name` agree, and whether upstream's cursor into the
/// name is then sitting on its terminating NUL.
///
/// The second half is not simply `k == name.len()`. Upstream walks with
/// `*cp++ == *np++`, so a *mismatch* still advances `np`, leaving it one
/// past the byte that differed -- which is how `:A5` can name a command
/// called `Ab`: the mismatch at `5` puts `np` on `Ab`'s NUL, and the
/// "matched up to a digit" rule then accepts it as a partial match.
/// Faithful to upstream, quirk included.
fn match_prefix(typed: &[u8], name: &[u8]) -> (usize, bool) {
    let mut k = 0;
    while k < typed.len() && k < name.len() {
        if typed[k] != name[k] {
            return (k, name.len() == k + 1);
        }
        k += 1;
    }
    (k, name.len() == k)
}

/// Where the command name `name` starts with ends, or `None` when what
/// follows the name is neither whitespace nor the end of the command.
///
/// `name` is read as a C string: past its end is a NUL.
pub(crate) fn uc_validate_name(name: &[u8]) -> Option<usize> {
    let at = |i: usize| cstr::byte_at(name, i);
    let mut end = 0;
    if at(0).is_ascii_alphabetic() {
        while at(end).is_ascii_alphanumeric() {
            end += 1;
        }
    }
    let next = c_int::from(at(end));
    (ends_excmd(next) != 0 || ascii_iswhite(next)).then_some(end)
}

/// Everything a command is defined with bar its name and its replacement.
///
/// It owns the completion argument and the three Lua references: a
/// definition that is never installed gives them back when it is dropped.
pub(crate) struct Definition {
    pub(crate) argt: ExArgt,
    pub(crate) default_count: int64_t,
    pub(crate) complete: ExpandContext,
    pub(crate) complete_arg: Option<CString>,
    pub(crate) addr_type: CmdAddr,
    pub(crate) luaref: LuaRef,
    pub(crate) complete_luaref: LuaRef,
    pub(crate) preview_luaref: LuaRef,
}

impl Drop for Definition {
    fn drop(&mut self) {
        for reference in [self.luaref, self.complete_luaref, self.preview_luaref] {
            crate::lua::executor::release_luaref(reference);
        }
    }
}

impl Definition {
    /// The entry this definition makes, the references moving into it.
    fn into_command(mut self, name: Rc<CStr>, replacement: Rc<CStr>) -> UserCmd {
        let mut script_ctx = current_sctx.get();
        script_ctx.sc_lnum += sourcing_lnum();
        nlua_set_sctx(&mut script_ctx);
        UserCmd {
            name,
            argt: self.argt,
            replacement,
            default_count: self.default_count,
            complete: self.complete,
            addr_type: self.addr_type,
            script_ctx,
            complete_arg: self.complete_arg.take().map(Rc::from),
            complete_luaref: mem::replace(&mut self.complete_luaref, LUA_NOREF),
            preview_luaref: mem::replace(&mut self.preview_luaref, LUA_NOREF),
            luaref: mem::replace(&mut self.luaref, LUA_NOREF),
        }
    }
}

/// Define, or redefine, one user command: the first `name_len` bytes of
/// `text` with replacement text `rep` (each read as a C string),
/// buffer-local when `buffer_local`. The refusal quotes `text` whole, as
/// upstream's `%s` of the name's pointer does -- for `:command` that is the
/// rest of the line.
///
/// Takes ownership of `def`: on failure what it owns is given back here.
pub(crate) fn uc_add_command(
    text: &[u8],
    name_len: usize,
    rep: &[u8],
    def: Definition,
    buffer_local: bool,
    force: bool,
) -> Result<(), Failed> {
    let (text, rep) = (cstr::until_nul(text), cstr::until_nul(rep));
    let name = &text[..name_len.min(text.len())];
    let replaced = p_cpo(|cpo| replace_termcodes(rep, 0, 0, None, cpo, None));
    let replacement = Rc::from(cstr::owned(cstr::until_nul(&replaced.unwrap_or_default())));

    let table = if buffer_local {
        Table::Buffer(Buf::current())
    } else {
        Table::Global
    };

    // The tables are kept sorted by name, so the walk stops at the first
    // entry that is not smaller: either this very command, or where it goes.
    let (idx, existing) = table.with(|cmds| {
        let mut idx = 0;
        for cmd in cmds {
            match name.cmp(ucmd_name(cmd)) {
                Ordering::Equal => return (idx, Some(cmd.script_ctx)),
                Ordering::Less => break,
                Ordering::Greater => idx += 1,
            }
        }
        (idx, None)
    });

    if let Some(existing) = existing
        // A command may replace itself while the same script is still
        // sourcing (`sc_seq` differs), but two different scripts need the
        // bang.
        && !force
        && (existing.sc_sid != current_sctx.get().sc_sid
            || existing.sc_seq == current_sctx.get().sc_seq)
    {
        let name = msg_bytes(text);
        semsg!("E174: Command already exists: add ! to replace it: {name}");
        return Err(Failed);
    }

    let entry = def.into_command(Rc::from(cstr::owned(name)), replacement);
    // Both closures are leaves; the entry a redefinition replaces is
    // dropped -- its references given back -- once the table is free.
    let replaced = if existing.is_some() {
        Some(table.with_mut(|cmds| mem::replace(&mut cmds[idx], entry)))
    } else {
        // `idx` is where the walk stopped, which keeps it sorted.
        table.with_mut(|cmds| cmds.insert(idx, entry));
        None
    };
    drop(replaced);
    Ok(())
}

/// `:command` -- define one, or list them.
pub(crate) fn ex_command(excmd: &mut ExArg) {
    let mut def = Definition {
        argt: ExArgt::NONE,
        default_count: -1,
        complete: ExpandContext::Nothing,
        complete_arg: None,
        addr_type: CmdAddr::NoRange,
        luaref: LUA_NOREF,
        complete_luaref: LUA_NOREF,
        preview_luaref: LUA_NOREF,
    };
    let mut default_count: c_int = -1;
    let mut buffer_local = false;

    let line = &excmd.line;
    let has_attr = line.byte_at(line.arg) == b'-';
    let mut p = line.arg;
    let name_end = loop {
        if line.byte_at(p) != b'-' {
            break uc_validate_name(line.rest_of(p)).map(|len| p + len);
        }
        let attr_start = p + 1;
        let attr_end = line.skip_to_white(attr_start);
        let into = attr::Attributes {
            argt: &mut def.argt,
            def: &mut default_count,
            buffer_local: &mut buffer_local,
            complp: &mut def.complete,
            compl_arg: &mut def.complete_arg,
            addr_type_arg: &mut def.addr_type,
        };
        if attr::uc_scan_attr(line.rest_of(attr_start), attr_end - attr_start, into).is_err() {
            return;
        }
        p = line.skip_white(attr_end);
    };

    let name = p;
    let Some(name_end) = name_end else {
        emsg(gettext(c"E182: Invalid command name"));
        return;
    };
    let rest = line.skip_white(name_end);
    let name_bytes = line.slice_at(name, name_end - name);
    // Not `name_bytes[0]`: an attribute-only line (`:command -nargs=1 |`)
    // leaves the name empty, and upstream still reads the byte there.
    let first = line.byte_at(name);

    let complaint = if !has_attr && ends_excmd(c_int::from(line.byte_at(rest))) != 0 {
        list::uc_list(name_bytes);
        None
    } else if !first.is_ascii_uppercase() {
        Some(c"E183: User defined commands must start with an uppercase letter")
    } else if b"Next".starts_with(name_bytes) {
        Some(c"E841: Reserved name, cannot be used for user defined command")
    } else if def.complete != ExpandContext::Nothing && !def.argt.has(ExArgt::EXTRA) {
        Some(c"E1208: -complete used without allowing arguments")
    } else {
        def.default_count = int64_t::from(default_count);
        let (text, len) = (line.rest_of(name), name_bytes.len());
        let _ = uc_add_command(
            text,
            len,
            line.rest_of(rest),
            def,
            buffer_local,
            excmd.forceit,
        );
        return;
    };

    if let Some(message) = complaint {
        emsg(gettext(message));
    }
}

/// `:comclear` -- forget every user command, global and buffer-local.
pub(crate) fn ex_comclear(_excmd: &mut ExArg) {
    uc_clear(Table::Global);
    if let Some(buffer) = Buf::current_or_none() {
        uc_clear(Table::Buffer(buffer));
    }
}

/// Empty one command table, leaving it usable again.
///
/// The entries are moved out first and dropped afterwards: dropping one
/// re-enters Lua, so it must not happen with the table borrowed, and a
/// table that has been emptied cannot drop the same entry twice.
pub(crate) fn uc_clear(table: Table) {
    // `mem::take` cannot re-enter.
    drop(table.with_mut(mem::take));
}

/// `:delcommand` -- remove one user command.
pub(crate) fn ex_delcommand(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    let buffer_only = excmd.line.arg().starts_with(b"-buffer")
        && ascii_iswhite(c_int::from(excmd.line.byte_at(at + 7)));
    if buffer_only {
        at = excmd.line.skip_white(at + 7);
    }
    let arg = excmd.line.cstr_from(at);
    let wanted = arg.to_bytes();

    // Upstream carries `res` across both passes, so an empty table leaves
    // the previous pass's answer standing; an empty table cannot match.
    let mut res = Ordering::Less;
    let mut found = None;
    for scope in Scope::BOTH {
        let table = scope.table();
        let idx = table.with(|cmds| {
            let mut idx = 0;
            for cmd in cmds {
                res = wanted.cmp(ucmd_name(cmd));
                if res != Ordering::Greater {
                    break;
                }
                idx += 1;
            }
            idx
        });
        if res == Ordering::Equal {
            found = Some((table, idx));
            break;
        }
        if buffer_only {
            break;
        }
    }

    let Some((table, idx)) = found else {
        let untranslated = if buffer_only {
            c"E1237: No such user-defined command in current buffer: %s"
        } else {
            c"E184: No such user-defined command: %s"
        };
        let arg = msg_cstr(arg);
        emsg_text(tr_c!(untranslated, arg));
        return;
    };
    uc_del_command(table, idx);
}

/// Delete the `idx`th entry of `table`, giving back everything it owns.
///
/// The entry is moved out under the borrow and dropped once the borrow has
/// ended: dropping it re-enters Lua, which must not happen while the table
/// is borrowed.
///
/// # Panics
/// If `idx` does not index `table`.
fn uc_del_command(table: Table, idx: usize) {
    // The closure is a leaf.
    drop(table.with_mut(|cmds| cmds.remove(idx)));
}

/// Delete the command called `name` from `table`; `false` when it has none.
pub(crate) fn uc_del_named(table: Table, name: &CStr) -> bool {
    let wanted = name.to_bytes();
    match table.with(|cmds| cmds.iter().position(|cmd| ucmd_name(cmd) == wanted)) {
        Some(idx) => {
            uc_del_command(table, idx);
            true
        }
        None => false,
    }
}
