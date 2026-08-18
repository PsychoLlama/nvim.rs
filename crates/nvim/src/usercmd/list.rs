//! Showing user commands: `:command` with no definition after it, and
//! `nvim_get_commands()`.
//!
//! The two render the same facts differently. [`uc_list`] writes the
//! human's table -- flags, name, arguments, address, completion and the
//! definition, in fixed columns that shift left when a name overruns its
//! own -- and [`commands_array`] the API's dictionary-of-dictionaries,
//! whose keys and value shapes are a public interface.
//!
//! Both walk the buffer-local commands and then the global ones, and both
//! render the argument count, the address type and the completion from the
//! same three tables the parser uses.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::attr::named_addr_type;
use super::complete::command_complete_name;
use super::{
    EX_BANG, EX_COUNT, EX_DFLALL, EX_EXTRA, EX_KEEPSCRIPT, EX_NEEDARG, EX_NOSPC, EX_RANGE,
    EX_REGSTR, EX_TRLBAR, LUA_NOREF, NUL, Scope, ucmd_list, ucmd_name, ucmds,
};
use crate::api::private::helpers::{
    arena_dict, arena_string, cstr_as_string, dict_put, dict_put_str,
};
use crate::eval::last_set_msg;
use crate::highlight_group::{HLF_8, HLF_D};
use crate::lua::executor::{api_new_luaref, nlua_funcref_str};
use crate::main::{Columns, IObuff, got_int, p_verbose};
use crate::memory::xfree;
use crate::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_outtrans_special, msg_putchar,
    msg_puts, msg_puts_hl, msg_puts_title,
};
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::strings::arena_printf;
use crate::types::builders::static_cstring;
use crate::types::{
    Arena, Dict, LuaRef, Object, buf_T, garray_T, int64_t, size_t, ucmd_T, uint32_t,
};
use core::ffi::{CStr, c_char, c_int};
use core::fmt::Write as _;
use core::ptr;

/// The `-nargs=` spelling of a command's argument flags.
///
/// The five combinations upstream tests for are the only ones its parser
/// produces; anything else is left blank, as upstream's `switch` does.
fn nargs_str(argt: uint32_t) -> &'static CStr {
    match argt & (EX_EXTRA | EX_NOSPC | EX_NEEDARG) {
        0 => c"0",
        EX_EXTRA => c"*",
        x if x == EX_EXTRA | EX_NOSPC => c"?",
        x if x == EX_EXTRA | EX_NEEDARG => c"+",
        x if x == EX_EXTRA | EX_NOSPC | EX_NEEDARG => c"1",
        _ => c"",
    }
}

/// The fixed-width middle columns of one `:command` line, built in
/// `IObuff` as upstream does.
///
/// `over` is how far the name column overran; every following column is
/// pulled left by it, and a column that would then start before the text
/// already written still gets its one separating space.
struct Cols<'a> {
    buf: &'a mut [c_char],
    len: usize,
}

impl Cols<'_> {
    fn push(&mut self, byte: u8) {
        self.buf[self.len] = byte as c_char;
        self.len += 1;
    }

    fn put(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.push(byte);
        }
    }

    /// Pad with spaces up to column `col`, always writing at least one.
    fn pad_to(&mut self, col: int64_t, over: int64_t) {
        loop {
            self.push(b' ');
            if self.len as int64_t >= col - over {
                return;
            }
        }
    }
}

impl core::fmt::Write for Cols<'_> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        self.put(text.as_bytes());
        Ok(())
    }
}

/// `:command` -- list every user command whose name starts with `name`.
///
/// # Safety
/// Module contract; `name` must have `name_len` readable bytes.
pub(super) unsafe fn uc_list(name: *const c_char, name_len: size_t) {
    // SAFETY: module contract.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    // SAFETY: module contract.
    let wanted = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), name_len) };
    let mut found = false;

    for scope in Scope::BOTH {
        let mut interrupted = false;
        // SAFETY: module contract; nothing below adds or removes a command.
        for cmd in unsafe { ucmd_list(scope.table()) } {
            // SAFETY: module contract.
            let matches =
                unsafe { ucmd_name(cmd).starts_with(wanted) && !message_filtered(cmd.uc_name) };
            if !matches {
                continue;
            }
            if !found {
                // SAFETY: module contract.
                unsafe {
                    msg_puts_title(gettext(
                        c"\n    Name              Args Address Complete    Definition".as_ptr(),
                    ));
                }
            }
            found = true;
            // SAFETY: module contract.
            unsafe { msg_putchar(b'\n' as c_int) };
            if got_int.get() {
                interrupted = true;
                break;
            }
            // SAFETY: module contract.
            unsafe { list_one(cmd, scope, name_len) };
            line_breakcheck();
            if got_int.get() {
                interrupted = true;
                break;
            }
        }
        if interrupted {
            break;
        }
    }

    if !found {
        // SAFETY: module contract.
        unsafe { msg(gettext(c"No user-defined commands found".as_ptr()), 0) };
    }
}

/// One command's row, from the flag column to the definition.
///
/// # Safety
/// Module contract.
unsafe fn list_one(cmd: &ucmd_T, scope: Scope, name_len: size_t) {
    let a = cmd.uc_argt;
    // The flag column is right-aligned in four cells.
    let mut blank = 4;
    // SAFETY: module contract.
    unsafe {
        for (present, mark) in [
            (a & EX_BANG != 0, b'!'),
            (a & EX_REGSTR != 0, b'"'),
            (scope == Scope::Buffer, b'b'),
            (a & EX_TRLBAR != 0, b'|'),
        ] {
            if present {
                msg_putchar(mark as c_int);
                blank -= 1;
            }
        }
        if blank != 0 {
            msg_puts(c"    ".as_ptr().add(4 - blank));
        }

        msg_outtrans(cmd.uc_name, HLF_D, false);
        // The name column is 17 wide; a longer name pushes the rest left.
        let mut len = ucmd_name(cmd).len() + 4;
        if len < 21 {
            // Field padding spaces   12345678901234567
            static SPACES: &CStr = c"                 ";
            msg_puts(SPACES.as_ptr().add(len - 4));
            len = 21;
        }
        msg_putchar(b' ' as c_int);
        len += 1;
        let over = len as int64_t - 22;

        // The middle columns are assembled in one buffer and printed once.
        IObuff.with_mut(|buf| {
            let mut cols = Cols {
                buf: &mut buf[..],
                len: 0,
            };
            cols.put(nargs_str(a).to_bytes());
            cols.pad_to(5, over);

            if a & (EX_RANGE | EX_COUNT) != 0 {
                if a & EX_COUNT != 0 {
                    // -count=N
                    let _ = write!(cols, "{}c", cmd.uc_def);
                } else if a & EX_DFLALL != 0 {
                    cols.push(b'%');
                } else if cmd.uc_def >= 0 {
                    // -range=N
                    let _ = write!(cols, "{}", cmd.uc_def);
                } else {
                    cols.push(b'.');
                }
            }
            cols.pad_to(8, over);

            if let Some(row) = named_addr_type(cmd.uc_addr_type) {
                cols.put(row.shortname.to_bytes());
            }
            cols.pad_to(13, over);

            if let Some(name) = command_complete_name(cmd.uc_compl) {
                cols.put(name.to_bytes());
            }
            cols.pad_to(25, over);

            let end = cols.len;
            buf[end] = NUL;
        });
        msg_outtrans(IObuff.ptr().cast::<c_char>(), 0, false);

        if cmd.uc_luaref != LUA_NOREF {
            let text = nlua_funcref_str(cmd.uc_luaref, ptr::null_mut());
            msg_puts_hl(text, HLF_8, false);
            xfree(text.cast());
            // The definition goes on a line of its own.
            if *cmd.uc_rep != NUL {
                msg_puts(c"\n                                               ".as_ptr());
            }
        }
        msg_outtrans_special(
            cmd.uc_rep,
            false,
            if name_len == 0 { Columns.get() - 47 } else { 0 },
        );
        if p_verbose.get() > 0 {
            last_set_msg(cmd.uc_script_ctx);
        }
    }
}

/// Collect the entries that are present into an arena Dict.
///
/// `capacity` is upstream's, not `N`: sizing and filling from one array is
/// what makes the puts sound, and the two must agree, so the assertion is
/// the contract rather than a hope.
fn dict_of<const N: usize>(
    arena: *mut Arena,
    capacity: size_t,
    entries: [(&'static CStr, Option<Object>); N],
) -> Dict {
    debug_assert!(N <= capacity, "dict_of past capacity");
    let mut dict = arena_dict(arena, capacity);
    // SAFETY: the dict was reserved for `capacity` pairs, at most `N` are
    // written, and this is the only thing that writes to it.
    unsafe {
        for (key, value) in entries {
            if let Some(value) = value {
                dict_put(&mut dict, key, value);
            }
        }
    }
    dict
}

/// `nvim_get_commands()`: every user command of `buf`, or every global one
/// when `buf` is null, as a map from name to description.
///
/// # Safety
/// Module contract; `buf` must be null or a live buffer, and `arena` the
/// dispatcher's.
pub unsafe fn commands_array(buf: *mut buf_T, arena: *mut Arena) -> Dict {
    let gap: *mut garray_T = if buf.is_null() {
        ucmds.ptr()
    } else {
        // SAFETY: caller contract.
        unsafe { &raw mut (*buf).b_ucmds }
    };
    // SAFETY: caller contract.
    let cmds = unsafe { ucmd_list(gap) };
    let mut rv = arena_dict(arena, cmds.len());
    for cmd in cmds {
        // SAFETY: module contract.
        let d = unsafe { describe(cmd, arena) };
        // SAFETY: `rv` was reserved for exactly one pair per command, and
        // this is the only thing that writes to it.
        unsafe {
            dict_put_str(&mut rv, cstr_as_string(cmd.uc_name), Object::dict(d));
        }
    }
    rv
}

/// One command as the API describes it. The key order is upstream's and is
/// what the wire format carries.
///
/// # Safety
/// Module contract.
unsafe fn describe(cmd: &ucmd_T, arena: *mut Arena) -> Dict {
    let a = cmd.uc_argt;
    // SAFETY: module contract; the entry owns each reference, and
    // `api_new_luaref` takes a fresh one for the caller to own.
    let luaref = |r: LuaRef| (r != LUA_NOREF).then(|| Object::luaref(unsafe { api_new_luaref(r) }));
    // SAFETY: module contract; the three strings outlive the arena copy.
    let (name, definition, complete_arg) = unsafe {
        (
            cstr_as_string(cmd.uc_name),
            cstr_as_string(cmd.uc_rep),
            if cmd.uc_compl_arg.is_null() {
                Object::NIL
            } else {
                Object::string(cstr_as_string(cmd.uc_compl_arg))
            },
        )
    };
    // The completion is a Lua reference when the command was given one,
    // and the `-complete=` name otherwise.
    let complete = match luaref(cmd.uc_compl_luaref) {
        Some(callback) => callback,
        None => match command_complete_name(cmd.uc_compl) {
            Some(text) => Object::string(static_cstring(text)),
            None => Object::NIL,
        },
    };
    let count = (a & EX_COUNT != 0).then(|| {
        if cmd.uc_def >= 0 {
            // SAFETY: `arena` is the dispatcher's.
            Object::string(unsafe { arena_printf(arena, c"%ld".as_ptr(), cmd.uc_def) })
        } else {
            Object::string(static_cstring(c"0"))
        }
    });
    let range = (a & EX_RANGE != 0).then(|| {
        if a & EX_DFLALL != 0 {
            Object::string(static_cstring(c"%"))
        } else if cmd.uc_def >= 0 {
            // SAFETY: `arena` is the dispatcher's.
            Object::string(unsafe { arena_printf(arena, c"%ld".as_ptr(), cmd.uc_def) })
        } else {
            Object::string(static_cstring(c"."))
        }
    });
    // SAFETY: `arena` is the dispatcher's and `nargs_str` is a literal.
    let nargs = unsafe { arena_string(arena, static_cstring(nargs_str(a))) };

    dict_of(
        arena,
        16,
        [
            (c"name", Some(Object::string(name))),
            (c"definition", Some(Object::string(definition))),
            (
                c"script_id",
                Some(Object::integer(cmd.uc_script_ctx.sc_sid.into())),
            ),
            (c"bang", Some(Object::boolean(a & EX_BANG != 0))),
            (c"bar", Some(Object::boolean(a & EX_TRLBAR != 0))),
            (c"register", Some(Object::boolean(a & EX_REGSTR != 0))),
            (c"keepscript", Some(Object::boolean(a & EX_KEEPSCRIPT != 0))),
            (c"preview", luaref(cmd.uc_preview_luaref)),
            (c"callback", luaref(cmd.uc_luaref)),
            (c"nargs", Some(Object::string(nargs))),
            (c"complete", Some(complete)),
            (c"complete_arg", Some(complete_arg)),
            (c"count", Some(count.unwrap_or(Object::NIL))),
            (c"range", Some(range.unwrap_or(Object::NIL))),
            (c"addr", Some(addr_object(cmd))),
        ],
    )
}

/// The `addr` field: the name of the command's address type, when it has
/// one that is not the default.
fn addr_object(cmd: &ucmd_T) -> Object {
    match named_addr_type(cmd.uc_addr_type) {
        Some(row) => Object::string(static_cstring(row.name)),
        None => Object::NIL,
    }
}
