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
//! The one thing to watch is that a borrow of a table -- [`ucmd_list`]'s
//! slice, or a `&ucmd_T` taken out of it -- does not survive anything that
//! can add or remove a command, because `ga_grow` reallocates. In practice
//! only [`uc_add_command`] and [`ex_delcommand`] do that, and both take
//! their index before touching the array.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod attr;
mod complete;
mod expand;
mod list;

pub use attr::{parse_addr_type_arg, parse_compl_arg};
pub use complete::{
    cmdcomplete_str_to_type, cmdcomplete_type_to_str, expand_user_command_name,
    get_user_cmd_addr_type, get_user_cmd_complete, get_user_cmd_flags, get_user_cmd_nargs,
    get_user_command_name, get_user_commands, set_context_in_user_cmd, set_context_in_user_cmdarg,
};
pub use expand::{
    add_win_cmd_modifiers, do_ucmd, uc_mods, uc_nargs_upper_bound, uc_split_args_iter,
};
pub use list::commands_array;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{skiptowhite, skipwhite};
use crate::ex_docmd::ends_excmd;
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::keycodes::replace_termcodes;
use crate::lua::executor::{api_free_luaref, nlua_set_sctx};
use crate::main::{curbuf, current_sctx, p_cpo};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::os::cshim::{gettext, memmove};
use crate::runtime::sourcing_lnum;
use crate::semsg_c;
use crate::strings::xstrnsave;
use crate::types::{
    CMD_USER, CMD_USER_BUF, FAIL, LuaRef, OK, cmd_addr_T, exarg_T, expand_T, garray_T, int64_t,
    size_t, ucmd_T, uint32_t,
};
use crate::window::prevwin_curwin;
use ::libc::strlen;
use core::cmp::Ordering;
use core::ffi::{CStr, c_char, c_int};
use core::{mem, ptr, slice};

pub const EXPAND_SHELLCMDLINE: c_int = 57;
pub const EXPAND_USER_ADDR_TYPE: c_int = 43;
pub const EXPAND_USER_LUA: c_int = 32;
pub const EXPAND_USER_LIST: c_int = 31;
pub const EXPAND_USER_DEFINED: c_int = 30;
pub const EXPAND_USER_COMPLETE: c_int = 25;
pub const EXPAND_USER_NARGS: c_int = 24;
pub const EXPAND_USER_CMD_FLAGS: c_int = 23;
pub const EXPAND_USER_COMMANDS: c_int = 22;
pub const EXPAND_MAPPINGS: c_int = 16;
pub const EXPAND_MENUS: c_int = 11;
pub const EXPAND_BUFFERS: c_int = 9;
pub const EXPAND_DIRECTORIES: c_int = 3;
pub const EXPAND_FILES: c_int = 2;
pub const EXPAND_COMMANDS: c_int = 1;
pub const EXPAND_NOTHING: c_int = 0;
pub const EXPAND_UNSUCCESSFUL: c_int = -2;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const DOCMD_KEYTYPED: u32 = 8;
pub const DOCMD_NOWAIT: u32 = 2;
pub const DOCMD_VERBOSE: u32 = 1;
pub const UC_BUFFER: c_int = 1;
pub const LUA_NOREF: c_int = -2;
pub const NUL: c_char = 0;
pub const EX_RANGE: u32 = 0x1;
pub const EX_BANG: u32 = 0x2;
pub const EX_EXTRA: u32 = 0x4;
pub const EX_XFILE: u32 = 0x8;
pub const EX_NOSPC: u32 = 0x10;
pub const EX_DFLALL: u32 = 0x20;
pub const EX_NEEDARG: u32 = 0x80;
pub const EX_TRLBAR: u32 = 0x100;
pub const EX_REGSTR: u32 = 0x200;
pub const EX_COUNT: u32 = 0x400;
pub const EX_ZEROR: u32 = 0x1000;
pub const EX_BUFNAME: u32 = 0x8000;
pub const EX_KEEPSCRIPT: u32 = 0x4000000;

/// The global user commands. A buffer's own live in its `b_ucmds`.
pub static ucmds: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: mem::size_of::<ucmd_T>() as c_int,
    ga_growsize: 4,
    ga_data: ptr::null_mut(),
});

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

    /// The array this scope names.
    ///
    /// # Safety
    /// Buffer scope reads `prevwin_curwin()`, which must have a buffer --
    /// true whenever there is a current window.
    pub(crate) unsafe fn table(self) -> *mut garray_T {
        match self {
            // SAFETY: caller contract.
            Scope::Buffer => unsafe { &raw mut (*(*prevwin_curwin()).w_buffer).b_ucmds },
            Scope::Global => ucmds.ptr(),
        }
    }
}

/// The commands in `gap`, as a slice.
///
/// # Safety
/// `gap` must be a live `garray_T` of `ucmd_T` -- one of the two tables --
/// and the borrow must not outlive anything that can add or remove a
/// command.
pub(crate) unsafe fn ucmd_list<'a>(gap: *const garray_T) -> &'a [ucmd_T] {
    // SAFETY: caller contract. An empty garray has a null `ga_data`, which
    // `from_raw_parts` will not accept even for a zero length.
    unsafe {
        if (*gap).ga_data.is_null() {
            return &[];
        }
        slice::from_raw_parts((*gap).ga_data.cast::<ucmd_T>(), (*gap).ga_len as usize)
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
pub unsafe fn find_ucmd(
    eap: *mut exarg_T,
    p: *mut c_char,
    full: *mut c_int,
    xp: *mut expand_T,
    complp: *mut c_int,
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
        let cmds = unsafe { ucmd_list(scope.table()) };
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
                unsafe {
                    (*xp).xp_luaref = uc.uc_compl_luaref;
                    (*xp).xp_arg = uc.uc_compl_arg;
                    (*xp).xp_script_ctx = uc.uc_script_ctx;
                    (*xp).xp_script_ctx.sc_lnum += sourcing_lnum();
                }
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
            unsafe { (*xp).xp_context = EXPAND_UNSUCCESSFUL };
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
pub unsafe fn uc_validate_name(name: *mut c_char) -> *mut c_char {
    let mut name = name;
    // SAFETY: caller contract; the walk stops at the NUL.
    unsafe {
        if (*name as u8).is_ascii_alphabetic() {
            while (*name as u8).is_ascii_alphanumeric() {
                name = name.offset(1);
            }
        }
        if ends_excmd(*name as c_int) == 0 && !ascii_iswhite(*name as c_int) {
            return ptr::null_mut();
        }
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
pub unsafe fn uc_add_command(
    name: *mut c_char,
    name_len: size_t,
    rep: *const c_char,
    argt: uint32_t,
    def: int64_t,
    flags: c_int,
    context: c_int,
    compl_arg: *mut c_char,
    compl_luaref: LuaRef,
    preview_luaref: LuaRef,
    addr_type: cmd_addr_T,
    luaref: LuaRef,
    force: bool,
) -> c_int {
    let mut rep_buf: *mut c_char = ptr::null_mut();
    // SAFETY: caller contract.
    unsafe {
        replace_termcodes(
            rep,
            strlen(rep),
            &raw mut rep_buf,
            0,
            0,
            ptr::null_mut(),
            p_cpo.get(),
        );
        if rep_buf.is_null() {
            rep_buf = xstrdup(rep);
        }
    }

    let gap = if flags & UC_BUFFER != 0 {
        // SAFETY: module contract.
        unsafe {
            let gap = &raw mut (*curbuf.get()).b_ucmds;
            if (*gap).ga_itemsize == 0 {
                ga_init(gap, mem::size_of::<ucmd_T>() as c_int, 4);
            }
            gap
        }
    } else {
        ucmds.ptr()
    };

    // SAFETY: caller contract.
    let new_name = unsafe { slice::from_raw_parts(name.cast::<u8>(), name_len) };
    // The tables are kept sorted by name, so the walk stops at the first
    // entry that is not smaller: either this very command, or where it goes.
    let mut idx = 0;
    let mut replacing = false;
    // SAFETY: module contract; nothing below reallocates before `ga_grow`.
    for cmd in unsafe { ucmd_list(gap) } {
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
        // SAFETY: `idx` indexes the entry the walk just compared.
        let cmd = unsafe { &mut *(*gap).ga_data.cast::<ucmd_T>().add(idx) };
        // A command may replace itself while the same script is still
        // sourcing (`sc_seq` differs), but two different scripts need the
        // bang.
        if !force
            && (cmd.uc_script_ctx.sc_sid != current_sctx.get().sc_sid
                || cmd.uc_script_ctx.sc_seq == current_sctx.get().sc_seq)
        {
            // SAFETY: `name` is the caller's; this call owns the other five.
            unsafe {
                semsg_c!(
                    gettext(c"E174: Command already exists: add ! to replace it: %s".as_ptr()),
                    name,
                );
                free_new_command(rep_buf, compl_arg, luaref, compl_luaref, preview_luaref);
            }
            return FAIL;
        }
        // SAFETY: the entry owns each of these.
        unsafe {
            xfree(cmd.uc_rep.cast());
            cmd.uc_rep = ptr::null_mut();
            xfree(cmd.uc_compl_arg.cast());
            cmd.uc_compl_arg = ptr::null_mut();
            free_luaref(&mut cmd.uc_luaref);
            free_luaref(&mut cmd.uc_compl_luaref);
            free_luaref(&mut cmd.uc_preview_luaref);
        }
    } else {
        // SAFETY: module contract; `idx <= ga_len`, so the tail move stays
        // inside the block `ga_grow` just made room in.
        unsafe {
            ga_grow(gap, 1);
            let slot = (*gap).ga_data.cast::<ucmd_T>().add(idx);
            memmove(
                slot.add(1).cast(),
                slot.cast(),
                (((*gap).ga_len as usize) - idx) * mem::size_of::<ucmd_T>(),
            );
            (*gap).ga_len += 1;
            (*slot).uc_name = xstrnsave(name, name_len);
        }
    }

    // SAFETY: `idx` is now a live entry either way.
    let cmd = unsafe { &mut *(*gap).ga_data.cast::<ucmd_T>().add(idx) };
    cmd.uc_rep = rep_buf;
    cmd.uc_argt = argt;
    cmd.uc_def = def;
    cmd.uc_compl = context;
    cmd.uc_script_ctx = current_sctx.get();
    cmd.uc_script_ctx.sc_lnum += sourcing_lnum();
    // SAFETY: the field is live for the call.
    unsafe { nlua_set_sctx(&raw mut cmd.uc_script_ctx) };
    cmd.uc_compl_arg = compl_arg;
    cmd.uc_compl_luaref = compl_luaref;
    cmd.uc_preview_luaref = preview_luaref;
    cmd.uc_addr_type = addr_type;
    cmd.uc_luaref = luaref;
    OK
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
    unsafe {
        xfree(rep_buf.cast());
        xfree(compl_arg.cast());
        for mut r in [luaref, compl_luaref, preview_luaref] {
            free_luaref(&mut r);
        }
    }
}

/// `:command` -- define one, or list them.
///
/// # Safety
/// Module contract; `eap` must be the command being executed.
pub unsafe fn ex_command(eap: *mut exarg_T) {
    let mut argt: uint32_t = 0;
    let mut def: c_int = -1;
    let mut flags: c_int = 0;
    let mut context: c_int = EXPAND_NOTHING;
    let mut compl_arg: *mut c_char = ptr::null_mut();
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;

    // SAFETY: caller contract.
    let (arg, forceit) = unsafe { ((*eap).arg, (*eap).forceit != 0) };
    // SAFETY: caller contract; `arg` is NUL-terminated.
    let has_attr = unsafe { *arg } == b'-' as c_char;
    let mut p = arg;
    // SAFETY: module contract; every step stays inside the NUL-terminated
    // argument.
    let name_end = unsafe {
        loop {
            if *p != b'-' as c_char {
                break uc_validate_name(p);
            }
            p = p.offset(1);
            let end = skiptowhite(p);
            let into = attr::Attributes {
                argt: &mut argt,
                def: &mut def,
                flags: &mut flags,
                complp: &mut context,
                compl_arg: &mut compl_arg,
                addr_type_arg: &mut addr_type_arg,
            };
            if attr::uc_scan_attr(p, end.offset_from(p) as size_t, into) == FAIL {
                xfree(compl_arg.cast());
                return;
            }
            p = skipwhite(end);
        }
    };

    let name = p;
    if name_end.is_null() {
        // SAFETY: module contract; this call owns `compl_arg`.
        unsafe {
            emsg(gettext(c"E182: Invalid command name".as_ptr()));
            xfree(compl_arg.cast());
        }
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
    } else if context > 0 && argt & EX_EXTRA == 0 {
        Some(c"E1208: -complete used without allowing arguments")
    } else {
        // SAFETY: module contract; `uc_add_command` takes `compl_arg`.
        unsafe {
            uc_add_command(
                name,
                name_len,
                rest,
                argt,
                def as int64_t,
                flags,
                context,
                compl_arg,
                LUA_NOREF,
                LUA_NOREF,
                addr_type_arg,
                LUA_NOREF,
                forceit,
            );
        }
        return;
    };

    // SAFETY: module contract; nothing above took `compl_arg`.
    unsafe {
        if let Some(message) = complaint {
            emsg(gettext(message.as_ptr()));
        }
        xfree(compl_arg.cast());
    }
}

/// `:comclear` -- forget every user command, global and buffer-local.
///
/// # Safety
/// Module contract.
pub unsafe fn ex_comclear(_eap: *mut exarg_T) {
    // SAFETY: module contract.
    unsafe {
        uc_clear(ucmds.ptr());
        if !curbuf.get().is_null() {
            uc_clear(&raw mut (*curbuf.get()).b_ucmds);
        }
    }
}

/// Release everything one entry owns. The entry itself is the caller's.
///
/// # Safety
/// Module contract; `cmd` must be a live entry that is being discarded.
pub unsafe fn free_ucmd(cmd: *mut ucmd_T) {
    // SAFETY: caller contract; the entry owns all six.
    unsafe {
        let cmd = &mut *cmd;
        xfree(cmd.uc_name.cast());
        xfree(cmd.uc_rep.cast());
        xfree(cmd.uc_compl_arg.cast());
        free_luaref(&mut cmd.uc_compl_luaref);
        free_luaref(&mut cmd.uc_luaref);
        free_luaref(&mut cmd.uc_preview_luaref);
    }
}

/// Empty one command table.
///
/// # Safety
/// Module contract; `gap` must be one of the two tables.
pub unsafe fn uc_clear(gap: *mut garray_T) {
    // SAFETY: caller contract.
    unsafe {
        for i in 0..ucmd_list(gap).len() {
            free_ucmd((*gap).ga_data.cast::<ucmd_T>().add(i));
        }
        ga_clear(gap);
    }
}

/// `:delcommand` -- remove one user command.
///
/// # Safety
/// Module contract; `eap` must be the command being executed.
pub unsafe fn ex_delcommand(eap: *mut exarg_T) {
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
        let (gap, cmds) = unsafe {
            let gap = scope.table();
            (gap, ucmd_list(gap))
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
            found = Some(gap);
            break;
        }
        if buffer_only {
            break;
        }
    }

    let Some(gap) = found else {
        // SAFETY: module contract.
        unsafe {
            semsg_c!(
                gettext(if buffer_only {
                    c"E1237: No such user-defined command in current buffer: %s".as_ptr()
                } else {
                    c"E184: No such user-defined command: %s".as_ptr()
                }),
                arg,
            );
        }
        return;
    };

    // SAFETY: module contract; `idx` is the entry just matched, and the
    // tail move stays inside the array.
    unsafe {
        let cmd = (*gap).ga_data.cast::<ucmd_T>().add(idx);
        free_ucmd(cmd);
        (*gap).ga_len -= 1;
        let tail = (*gap).ga_len as usize - idx;
        if tail > 0 {
            memmove(
                cmd.cast(),
                cmd.add(1).cast(),
                tail * mem::size_of::<ucmd_T>(),
            );
        }
    }
}
