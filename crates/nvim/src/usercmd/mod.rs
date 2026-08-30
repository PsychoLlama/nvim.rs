//! User-defined commands: `:command`, `:comclear`, `:delcommand`, and the
//! lookup that turns a typed name back into one.
//!
//! A user command is a [`ucmd_T`] -- a name, a replacement string, the
//! `EX_*` flags its attributes imply, and the completion it wants for its
//! arguments. They live in two sorted arrays: the global [`ucmds`] and the
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
//! # Safety
//!
//! Everything here runs on the main thread with the two command tables
//! live. `ucmd_T` fields are raw C pointers into memory the table owns:
//! `uc_name`, `uc_rep` and `uc_compl_arg` are NUL-terminated strings valid
//! until the entry is replaced or deleted, and the Lua references are owned
//! by the entry. That is the contract the `unsafe fn`s here share; each
//! states it once by reference rather than restating it.
//!
//! The one thing to watch is that a borrow of a table -- [`Table::list`]'s
//! slice, or a `&ucmd_T` taken out of it -- does not survive anything that
//! can add or remove a command, because growing a `Vec` moves its contents.
//! In practice only [`uc_add_command`] and [`ex_delcommand`] do that, and
//! both take their index before touching the table.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod attr;
mod complete;
mod expand;
mod list;

pub(crate) use attr::{parse_addr_type_arg, parse_compl_arg};
pub(crate) use complete::{
    cmdcomplete_str_to_type, cmdcomplete_type_to_str, expand_user_command_name,
    get_user_cmd_addr_type, get_user_cmd_complete, get_user_cmd_flags, get_user_cmd_nargs,
    get_user_command_name, get_user_commands, set_context_in_user_cmd, set_context_in_user_cmdarg,
};
pub(crate) use expand::{
    add_win_cmd_modifiers, do_ucmd, uc_mods, uc_nargs_upper_bound, uc_split_args_iter,
};
pub(crate) use list::commands_array;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{skiptowhite, skipwhite};
use crate::cstr;
use crate::ex_docmd::ends_excmd;
use crate::global_cell::GlobalCell;
use crate::keycodes::replace_termcodes;
use crate::lua::executor::{api_free_luaref, nlua_set_sctx};
use crate::main::{curbuf, current_sctx, p_cpo};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::message_fmt::{c_str, emsg_text};
use crate::os::cshim::gettext;
use crate::runtime::sourcing_lnum;
use crate::semsg;
use crate::strings::xstrnsave;
use crate::tr_c;
use crate::types::{
    CMD_USER, CMD_USER_BUF, CmdAddr, ExArgt, ExpandContext, FAIL, Failed, LuaRef, OK, buf_T,
    exarg_T, expand_T, int64_t, size_t, ucmd_T,
};
use crate::window::prevwin_curwin;
use core::cmp::Ordering;
use core::ffi::{CStr, c_char, c_int};
use core::{mem, ptr, slice};

pub(crate) const UC_BUFFER: c_int = 1;
pub(crate) const LUA_NOREF: c_int = -2;

/// The global user commands, sorted by name. A buffer's own live in its
/// `b_ucmds`, which is the same `Vec`; [`Table`] is the pair.
///
/// The cell's address is never handed out: a read borrows it just long
/// enough to answer a slice, and a write is a closure that cannot re-enter.
static ucmds: GlobalCell<Vec<ucmd_T>> = GlobalCell::new(Vec::new());

/// One command table: the global one, or the one inside a buffer.
///
/// The two are the same `Vec<ucmd_T>` in different places, and every walker
/// here is written against this so that one function serves both. It is the
/// *where*; [`Scope`] is the *which of the two the current buffer sees*.
#[derive(Clone, Copy)]
pub(crate) enum Table {
    /// The global [`ucmds`].
    Global,
    /// `buf`'s own `b_ucmds`.
    Buffer(*mut buf_T),
}

impl Table {
    /// The commands in this table, as a slice.
    ///
    /// # Safety
    /// A [`Table::Buffer`] must name a live buffer, and -- as the module
    /// docs say -- the borrow must not outlive anything that can add or
    /// remove a command, because growing a `Vec` moves its contents.
    pub(crate) unsafe fn list<'a>(self) -> &'a [ucmd_T] {
        let (data, len) = match self {
            Table::Global => ucmds.with(|cmds| (cmds.as_ptr(), cmds.len())),
            Table::Buffer(buf) => {
                // SAFETY: caller contract. The field is reached by raw
                // projection, so no reference to the whole buffer is formed.
                let cmds = unsafe { &(*buf).b_ucmds };
                (cmds.as_ptr(), cmds.len())
            }
        };
        // SAFETY: `Vec::as_ptr` answers an aligned, non-null pointer to
        // `len` initialised entries even when the table is empty; the
        // caller keeps the borrow short enough.
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Run `f` on this table, mutably.
    ///
    /// `f` holds the only `&mut` to the table, so ruling 6 makes it a leaf:
    /// it must not re-enter the editor. Releasing what an entry owns calls
    /// into Lua, so every caller here moves the entry *out* under `f` and
    /// frees it afterwards.
    ///
    /// # Safety
    /// A [`Table::Buffer`] must name a live buffer.
    unsafe fn with_mut<R>(self, f: impl FnOnce(&mut Vec<ucmd_T>) -> R) -> R {
        match self {
            Table::Global => ucmds.with_mut(f),
            // SAFETY: caller contract; the raw projection borrows the field
            // and not the buffer around it.
            Table::Buffer(buf) => unsafe { f(&mut (*buf).b_ucmds) },
        }
    }
}

/// Which of the two command tables a walk is standing on.
///
/// Upstream asks this by comparing the `garray_T *` it is walking against
/// `&ucmds`; the identity test is the only thing distinguishing "this entry
/// is buffer-local" from "this entry is global", and it appears in four
/// different walks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// The current buffer's `b_ucmds`, searched first.
    Buffer,
    /// The global [`ucmds`], searched second.
    Global,
}

impl Scope {
    /// Both tables in search order.
    pub(crate) const BOTH: [Scope; 2] = [Scope::Buffer, Scope::Global];

    /// The table this scope names.
    ///
    /// # Safety
    /// Buffer scope reads `prevwin_curwin()`, which must have a buffer --
    /// true whenever there is a current window.
    pub(crate) unsafe fn table(self) -> Table {
        match self {
            // SAFETY: caller contract.
            Scope::Buffer => Table::Buffer(unsafe { (*prevwin_curwin()).w_buffer }),
            Scope::Global => Table::Global,
        }
    }

    /// The commands this scope names, as a slice.
    ///
    /// # Safety
    /// As [`Scope::table`] and [`Table::list`].
    unsafe fn list<'a>(self) -> &'a [ucmd_T] {
        // SAFETY: caller contract.
        unsafe { self.table().list() }
    }
}

/// A command's name.
///
/// # Safety
/// As [`ucmd_list`]: `cmd` must be a live entry of one of the tables.
pub(crate) unsafe fn ucmd_name(cmd: &ucmd_T) -> &[u8] {
    // SAFETY: caller contract; `uc_name` is NUL-terminated for the life of
    // the entry.
    unsafe { CStr::from_ptr(cmd.uc_name).to_bytes() }
}

/// Search both tables for a command matching `eap->cmd`.
///
/// Sets `eap->cmdidx`, `eap->argt`, `eap->useridx` and `eap->addr_type`,
/// and answers a pointer to just after the command name -- which may be
/// *before* `p`, because the match may be followed immediately by a count
/// that `p` has already skipped. Answers null when nothing matched.
///
/// `full` is set when the match was exact, `xp` filled in for completion
/// and `complp` given the command's completion type; each may be null.
///
/// # Safety
/// Module contract; `eap` must be the command being looked up, and `full`,
/// `xp` and `complp` null or writable.
pub(crate) unsafe fn find_ucmd(
    eap: *mut exarg_T,
    p: *mut c_char,
    full: *mut c_int,
    xp: *mut expand_T,
    complp: *mut ExpandContext,
) -> *mut c_char {
    // SAFETY: caller contract.
    let eap = unsafe { &mut *eap };
    // SAFETY: caller contract; `p` points into the same line as `eap.cmd`.
    let typed = unsafe { slice::from_raw_parts(eap.cmd.cast::<u8>(), p.offset_from(eap.cmd) as _) };

    let mut matchlen = 0;
    let mut found = false;
    let mut possible = false;
    // A buffer-local command matched ambiguously; only a full global match
    // is accepted then.
    let mut amb_local = false;

    for scope in Scope::BOTH {
        // SAFETY: module contract.
        let cmds = unsafe { scope.list() };
        let mut exact = false;
        for (j, uc) in cmds.iter().enumerate() {
            // SAFETY: module contract.
            let name = unsafe { ucmd_name(uc) };
            let (k, at_nul) = match_prefix(typed, name);
            // A match up to a digit means there may be another command
            // *including* the digit that should be preferred.
            if !(k == typed.len() || (at_nul && ascii_isdigit(typed[k] as c_int))) {
                continue;
            }
            if k == typed.len() && found && !at_nul {
                if scope == Scope::Global {
                    return ptr::null_mut();
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
            eap.cmdidx = if scope == Scope::Global {
                CMD_USER
            } else {
                CMD_USER_BUF
            };
            eap.argt = uc.uc_argt;
            eap.useridx = j as c_int;
            eap.addr_type = uc.uc_addr_type;
            if !complp.is_null() {
                // SAFETY: caller contract.
                unsafe { *complp = uc.uc_compl };
            }
            if !xp.is_null() {
                // SAFETY: caller contract.
                unsafe { (*xp).xp_luaref = uc.uc_compl_luaref };
                unsafe { (*xp).xp_arg = uc.uc_compl_arg };
                unsafe { (*xp).xp_script_ctx = uc.uc_script_ctx };
                unsafe { (*xp).xp_script_ctx.sc_lnum += sourcing_lnum() };
            }
            // Do not look for further abbreviations of an exact match.
            matchlen = k;
            if k == typed.len() && at_nul {
                if !full.is_null() {
                    // SAFETY: caller contract.
                    unsafe { *full = true as c_int };
                }
                amb_local = false;
                exact = true;
                break;
            }
        }
        // Stop on a full match; otherwise fall through to the global table.
        if exact {
            break;
        }
    }

    if amb_local {
        if !xp.is_null() {
            // SAFETY: caller contract.
            unsafe { (*xp).xp_context = ExpandContext::Unsuccessful };
        }
        return ptr::null_mut();
    }
    if found || possible {
        // The match may be followed immediately by a number: move back onto
        // it.
        // SAFETY: `matchlen <= typed.len()`, so this stays inside the line.
        return unsafe { p.offset(matchlen as isize - typed.len() as isize) };
    }
    p
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

/// The end of the command name `name` starts with, or null when what
/// follows the name is neither whitespace nor the end of the command.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn uc_validate_name(name: *mut c_char) -> *mut c_char {
    let mut name = name;
    // SAFETY: caller contract; the walk stops at the NUL.
    if (unsafe { *name } as u8).is_ascii_alphabetic() {
        while (unsafe { *name } as u8).is_ascii_alphanumeric() {
            name = unsafe { name.offset(1) };
        }
    }
    if ends_excmd(unsafe { *name } as c_int) == 0 && !ascii_iswhite(unsafe { *name } as c_int) {
        return ptr::null_mut();
    }
    name
}

/// Define, or redefine, one user command.
///
/// Takes ownership of `compl_arg` and of the three Lua references: on
/// failure they are freed here.
///
/// # Safety
/// Module contract; `name` must have `name_len` readable bytes and `rep`
/// must be NUL-terminated.
#[expect(clippy::too_many_arguments, reason = "one per ucmd_T field")]
pub(crate) unsafe fn uc_add_command(
    name: *mut c_char,
    name_len: size_t,
    rep: *const c_char,
    argt: ExArgt,
    def: int64_t,
    flags: c_int,
    context: ExpandContext,
    compl_arg: *mut c_char,
    compl_luaref: LuaRef,
    preview_luaref: LuaRef,
    addr_type: CmdAddr,
    luaref: LuaRef,
    force: bool,
) -> Result<(), Failed> {
    let mut rep_buf: *mut c_char = ptr::null_mut();
    let out = &raw mut rep_buf;
    let (no_flags, no_did_simplify, cpo) = (0, ptr::null_mut(), p_cpo.get());
    // SAFETY: caller contract; `rep_buf` is this frame's own.
    unsafe {
        let len = cstr::bytes_at(rep).len();
        replace_termcodes(rep, len, out, 0, no_flags, no_did_simplify, cpo)
    };
    if rep_buf.is_null() {
        rep_buf = unsafe { xstrdup(rep) };
    }

    let table = if flags & UC_BUFFER != 0 {
        Table::Buffer(curbuf.get())
    } else {
        Table::Global
    };

    // SAFETY: caller contract.
    let new_name = unsafe { slice::from_raw_parts(name.cast::<u8>(), name_len) };
    // The tables are kept sorted by name, so the walk stops at the first
    // entry that is not smaller: either this very command, or where it goes.
    let mut idx = 0;
    let mut replacing = false;
    // SAFETY: module contract; the borrow ends with the walk.
    for cmd in unsafe { table.list() } {
        // SAFETY: module contract.
        match new_name.cmp(unsafe { ucmd_name(cmd) }) {
            Ordering::Equal => {
                replacing = true;
                break;
            }
            Ordering::Less => break,
            Ordering::Greater => idx += 1,
        }
    }

    if replacing {
        // SAFETY: module contract; `idx` indexes the entry the walk just
        // compared, and the borrow ends with the copy.
        let existing = unsafe { table.list()[idx].uc_script_ctx };
        // A command may replace itself while the same script is still
        // sourcing (`sc_seq` differs), but two different scripts need the
        // bang.
        if !force
            && (existing.sc_sid != current_sctx.get().sc_sid
                || existing.sc_seq == current_sctx.get().sc_seq)
        {
            // SAFETY: `name` is the caller's; this call owns the other five.
            let name = unsafe { c_str(name) };
            semsg!("E174: Command already exists: add ! to replace it: {name}");
            unsafe { free_new_command(rep_buf, compl_arg, luaref, compl_luaref, preview_luaref) };
            return Err(Failed);
        }
        // Everything the old entry owned bar its name, taken out of the
        // table before it is released: freeing a Lua reference re-enters,
        // and no borrow of the table may be live when it does.
        let steal = |cmds: &mut Vec<ucmd_T>| {
            let cmd = &mut cmds[idx];
            (
                mem::replace(&mut cmd.uc_rep, ptr::null_mut()),
                mem::replace(&mut cmd.uc_compl_arg, ptr::null_mut()),
                [
                    mem::replace(&mut cmd.uc_luaref, LUA_NOREF),
                    mem::replace(&mut cmd.uc_compl_luaref, LUA_NOREF),
                    mem::replace(&mut cmd.uc_preview_luaref, LUA_NOREF),
                ],
            )
        };
        // SAFETY: module contract; `steal` is a leaf.
        let (old_rep, old_compl_arg, old_luarefs) = unsafe { table.with_mut(steal) };
        // SAFETY: the entry owned all five and no longer names any of them.
        unsafe { xfree(old_rep.cast()) };
        unsafe { xfree(old_compl_arg.cast()) };
        for mut luaref in old_luarefs {
            unsafe { free_luaref(&mut luaref) };
        }
    }

    let mut script_ctx = current_sctx.get();
    script_ctx.sc_lnum += sourcing_lnum();
    // SAFETY: the local is live for the call.
    unsafe { nlua_set_sctx(&raw mut script_ctx) };
    // A new entry needs a name of its own; a replaced one keeps the name it
    // was found by, which is the same string.
    // SAFETY: caller contract; `name` has `name_len` readable bytes.
    let fresh_name = (!replacing).then(|| unsafe { xstrnsave(name, name_len) });

    let store = |cmds: &mut Vec<ucmd_T>| {
        let entry = |uc_name| ucmd_T {
            uc_name,
            uc_argt: argt,
            uc_rep: rep_buf,
            uc_def: def,
            uc_compl: context,
            uc_addr_type: addr_type,
            uc_script_ctx: script_ctx,
            uc_compl_arg: compl_arg,
            uc_compl_luaref: compl_luaref,
            uc_preview_luaref: preview_luaref,
            uc_luaref: luaref,
        };
        match fresh_name {
            // `idx` is where the walk stopped, which keeps it sorted.
            Some(uc_name) => cmds.insert(idx, entry(uc_name)),
            None => {
                let uc_name = cmds[idx].uc_name;
                cmds[idx] = entry(uc_name);
            }
        }
    };
    // SAFETY: module contract; `store` is a leaf.
    unsafe { table.with_mut(store) };
    Ok(())
}

/// Release a reference and mark the field free. No-op on `LUA_NOREF`.
///
/// # Safety
/// The reference must be owned by whatever holds `slot`.
unsafe fn free_luaref(slot: &mut LuaRef) {
    if *slot != LUA_NOREF {
        // SAFETY: caller contract.
        unsafe { api_free_luaref(*slot) };
        *slot = LUA_NOREF;
    }
}

/// Everything [`uc_add_command`] was handed but could not install.
///
/// # Safety
/// The caller must own all five.
unsafe fn free_new_command(
    rep_buf: *mut c_char,
    compl_arg: *mut c_char,
    luaref: LuaRef,
    compl_luaref: LuaRef,
    preview_luaref: LuaRef,
) {
    // SAFETY: caller contract.
    unsafe { xfree(rep_buf.cast()) };
    unsafe { xfree(compl_arg.cast()) };
    for mut r in [luaref, compl_luaref, preview_luaref] {
        unsafe { free_luaref(&mut r) };
    }
}

/// `:command` -- define one, or list them.
///
/// # Safety
/// Module contract; `eap` must be the command being executed.
pub(crate) unsafe fn ex_command(eap: *mut exarg_T) {
    let mut argt = ExArgt::NONE;
    let mut def: c_int = -1;
    let mut flags: c_int = 0;
    let mut context = ExpandContext::Nothing;
    let mut compl_arg: *mut c_char = ptr::null_mut();
    let mut addr_type_arg: CmdAddr = CmdAddr::NoRange;

    // SAFETY: caller contract.
    let (arg, forceit) = unsafe { ((*eap).arg, (*eap).forceit != 0) };
    // SAFETY: caller contract; `arg` is NUL-terminated.
    let has_attr = unsafe { *arg } == b'-' as c_char;
    let mut p = arg;
    // SAFETY: module contract; every step stays inside the NUL-terminated
    // argument.
    let name_end = loop {
        // SAFETY: module contract; every step stays inside the argument.
        if unsafe { *p } != b'-' as c_char {
            // SAFETY: as above.
            break unsafe { uc_validate_name(p) };
        }
        // SAFETY: as above -- the byte just read was not the NUL.
        let (attr_start, attr_end) = unsafe {
            let start = p.offset(1);
            (start, skiptowhite(start))
        };
        let into = attr::Attributes {
            argt: &mut argt,
            def: &mut def,
            flags: &mut flags,
            complp: &mut context,
            compl_arg: &mut compl_arg,
            addr_type_arg: &mut addr_type_arg,
        };
        // SAFETY: as above; `into` names this frame's own locals.
        let len = unsafe { attr_end.offset_from(attr_start) } as size_t;
        // SAFETY: as above.
        if unsafe { attr::uc_scan_attr(attr_start, len, into) } == FAIL {
            // SAFETY: nothing above took `compl_arg`.
            unsafe { xfree(compl_arg.cast()) };
            return;
        }
        // SAFETY: as above.
        p = unsafe { skipwhite(attr_end) };
    };

    let name = p;
    if name_end.is_null() {
        emsg(gettext(c"E182: Invalid command name"));
        unsafe { xfree(compl_arg.cast()) };
        return;
    }
    // SAFETY: module contract.
    let (name_len, rest) = unsafe { (name_end.offset_from(name) as size_t, skipwhite(name_end)) };
    // SAFETY: module contract.
    let name_bytes = unsafe { slice::from_raw_parts(name.cast::<u8>(), name_len) };
    // Not `name_bytes[0]`: an attribute-only line (`:command -nargs=1 |`)
    // leaves the name empty, and upstream still reads the byte there.
    // SAFETY: module contract; `name` points into the NUL-terminated
    // argument, so there is always a byte to read.
    let first = unsafe { *name } as u8;

    // SAFETY: module contract.
    let complaint = if !has_attr && unsafe { ends_excmd(*rest as c_int) } != 0 {
        // SAFETY: module contract.
        unsafe { list::uc_list(name, name_len) };
        None
    } else if !first.is_ascii_uppercase() {
        Some(c"E183: User defined commands must start with an uppercase letter")
    } else if b"Next".starts_with(name_bytes) {
        Some(c"E841: Reserved name, cannot be used for user defined command")
    } else if context != ExpandContext::Nothing && !argt.has(ExArgt::EXTRA) {
        Some(c"E1208: -complete used without allowing arguments")
    } else {
        let def = def as int64_t;
        let (no_compl, no_preview, no_cb) = (LUA_NOREF, LUA_NOREF, LUA_NOREF);
        // SAFETY: module contract; `uc_add_command` takes `compl_arg`.
        let _ = unsafe {
            uc_add_command(
                name,
                name_len,
                rest,
                argt,
                def,
                flags,
                context,
                compl_arg,
                no_compl,
                no_preview,
                addr_type_arg,
                no_cb,
                forceit,
            )
        };
        return;
    };

    if let Some(message) = complaint {
        emsg(gettext(message));
    }
    unsafe { xfree(compl_arg.cast()) };
}

/// `:comclear` -- forget every user command, global and buffer-local.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_comclear(_eap: *mut exarg_T) {
    // SAFETY: module contract.
    unsafe { uc_clear(Table::Global) };
    if !curbuf.get().is_null() {
        unsafe { uc_clear(Table::Buffer(curbuf.get())) };
    }
}

/// Release everything one entry owns.
///
/// The entry is taken by value, which is what makes the release exactly
/// once: `ucmd_T` is neither `Copy` nor `Clone`, so the caller has had to
/// move it out of the table to get here.
///
/// # Safety
/// Module contract; `cmd` must be an entry that has been taken out of a
/// table and is being discarded.
unsafe fn free_ucmd(mut cmd: ucmd_T) {
    // SAFETY: caller contract; the entry owns all six.
    unsafe { xfree(cmd.uc_name.cast()) };
    unsafe { xfree(cmd.uc_rep.cast()) };
    unsafe { xfree(cmd.uc_compl_arg.cast()) };
    unsafe { free_luaref(&mut cmd.uc_compl_luaref) };
    unsafe { free_luaref(&mut cmd.uc_luaref) };
    unsafe { free_luaref(&mut cmd.uc_preview_luaref) };
}

/// Empty one command table, leaving it usable again.
///
/// The entries are moved out first and released afterwards: `free_ucmd`
/// re-enters Lua, so it must not run with the table borrowed, and a table
/// that has been emptied cannot free the same entry twice.
///
/// # Safety
/// Module contract; a [`Table::Buffer`] must name a live buffer.
pub(crate) unsafe fn uc_clear(table: Table) {
    // SAFETY: caller contract; `mem::take` cannot re-enter.
    let cmds = unsafe { table.with_mut(mem::take) };
    for cmd in cmds {
        // SAFETY: the entry is out of the table and owns what it names.
        unsafe { free_ucmd(cmd) };
    }
}

/// `:delcommand` -- remove one user command.
///
/// # Safety
/// Module contract; `eap` must be the command being executed.
pub(crate) unsafe fn ex_delcommand(eap: *mut exarg_T) {
    // SAFETY: caller contract; `eap.arg` is NUL-terminated.
    let (mut arg, buffer_only) = unsafe {
        let arg = (*eap).arg.cast_const();
        let local = CStr::from_ptr(arg).to_bytes().starts_with(b"-buffer")
            && ascii_iswhite(*arg.add(7) as c_int);
        (arg, local)
    };
    if buffer_only {
        // SAFETY: the seven bytes were just matched.
        arg = unsafe { skipwhite(arg.add(7)) };
    }
    // SAFETY: module contract.
    let wanted = unsafe { CStr::from_ptr(arg).to_bytes() };

    // Upstream carries `res` across both passes, so an empty table leaves
    // the previous pass's answer standing; an empty table cannot match.
    let mut res = Ordering::Less;
    let mut idx = 0;
    let mut found = None;
    for scope in Scope::BOTH {
        // SAFETY: module contract.
        let (table, cmds) = unsafe {
            let table = scope.table();
            (table, table.list())
        };
        idx = 0;
        for cmd in cmds {
            // SAFETY: module contract.
            res = wanted.cmp(unsafe { ucmd_name(cmd) });
            if res != Ordering::Greater {
                break;
            }
            idx += 1;
        }
        if res == Ordering::Equal {
            found = Some(table);
            break;
        }
        if buffer_only {
            break;
        }
    }

    let Some(table) = found else {
        let untranslated = if buffer_only {
            c"E1237: No such user-defined command in current buffer: %s"
        } else {
            c"E184: No such user-defined command: %s"
        };
        // SAFETY: module contract -- `arg` is NUL-terminated.
        let arg = unsafe { c_str(arg) };
        emsg_text(tr_c!(untranslated, arg));
        return;
    };

    // SAFETY: module contract; `idx` is the entry just matched.
    unsafe { uc_del_command(table, idx) };
}

/// Delete the `idx`th entry of `table`, releasing everything it owns.
///
/// The entry is moved out under the borrow and freed once the borrow has
/// ended: `free_ucmd` re-enters Lua, which must not happen while the table
/// is borrowed, and an entry that has left the table cannot be freed twice.
///
/// # Safety
/// Module contract; `idx` must index `table`, and a [`Table::Buffer`] must
/// name a live buffer.
pub(crate) unsafe fn uc_del_command(table: Table, idx: usize) {
    // SAFETY: caller contract; the closure is a leaf.
    let cmd = unsafe { table.with_mut(|cmds| cmds.remove(idx)) };
    // SAFETY: the entry is out of the table and owns what it names.
    unsafe { free_ucmd(cmd) };
}
