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
use super::{LUA_NOREF, Scope, Table, ucmd_name};
use crate::api::private::helpers::{
    arena_dict, arena_string, cstr_as_string, dict_put, dict_put_str,
};
use crate::eval::last_set_msg;
use crate::highlight_group::{HLF_8, HLF_D};
use crate::lua::executor::{api_new_luaref, nlua_funcref_str};
use crate::main::{Columns, got_int, p_verbose};
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
    Arena, Dict, ExArgt, IOSIZE, LuaRef, NUL, Object, buf_T, int64_t, size_t, ucmd_T,
};
use core::ffi::{CStr, c_char, c_int};
use core::fmt::Write as _;
use core::ptr;

/// The `-nargs=` spelling of a command's argument flags.
///
/// The five combinations upstream tests for are the only ones its parser
/// produces; anything else is left blank, as upstream's `switch` does.
fn nargs_str(argt: ExArgt) -> &'static CStr {
    match argt.masked(ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG) {
        x if x.is_empty() => c"0",
        x if x == ExArgt::EXTRA => c"*",
        x if x == ExArgt::EXTRA | ExArgt::NOSPC => c"?",
        x if x == ExArgt::EXTRA | ExArgt::NEEDARG => c"+",
        x if x == ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG => c"1",
        _ => c"",
    }
}

/// The fixed-width middle columns of one `:command` line, built in a
/// buffer of the row's own — upstream uses the shared `IObuff`, which
/// `msg_outtrans` writes again.
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
        for cmd in unsafe { scope.list() } {
            // SAFETY: module contract.
            let matches =
                unsafe { ucmd_name(cmd).starts_with(wanted) && !message_filtered(cmd.uc_name) };
            if !matches {
                continue;
            }
            if !found {
                let heading =
                    c"\n    Name              Args Address Complete    Definition".as_ptr();
                // SAFETY: module contract; the heading is a static string.
                unsafe { msg_puts_title(gettext(heading)) };
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
    let mut middle = [0 as c_char; IOSIZE as usize];
    let a = cmd.uc_argt;
    // The flag column is right-aligned in four cells.
    let mut blank = 4;
    // SAFETY: module contract.
    for (present, mark) in [
        (a.has(ExArgt::BANG), b'!'),
        (a.has(ExArgt::REGSTR), b'"'),
        (scope == Scope::Buffer, b'b'),
        (a.has(ExArgt::TRLBAR), b'|'),
    ] {
        if present {
            unsafe { msg_putchar(mark as c_int) };
            blank -= 1;
        }
    }
    if blank != 0 {
        unsafe { msg_puts(c"    ".as_ptr().add(4 - blank)) };
    }

    unsafe { msg_outtrans(cmd.uc_name, HLF_D, false) };
    // The name column is 17 wide; a longer name pushes the rest left.
    let mut len = unsafe { ucmd_name(cmd) }.len() + 4;
    if len < 21 {
        // Field padding spaces   12345678901234567
        static SPACES: &CStr = c"                 ";
        unsafe { msg_puts(SPACES.as_ptr().add(len - 4)) };
        len = 21;
    }
    unsafe { msg_putchar(b' ' as c_int) };
    len += 1;
    let over = len as int64_t - 22;

    // The middle columns are assembled in one buffer and printed once.
    {
        let mut cols = Cols {
            buf: &mut middle[..],
            len: 0,
        };
        cols.put(nargs_str(a).to_bytes());
        cols.pad_to(5, over);

        if a.has(ExArgt::RANGE | ExArgt::COUNT) {
            if a.has(ExArgt::COUNT) {
                // -count=N
                let _ = write!(cols, "{}c", cmd.uc_def);
            } else if a.has(ExArgt::DFLALL) {
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
        cols.buf[end] = NUL as c_char;
    }
    unsafe { msg_outtrans(middle.as_mut_ptr(), 0, false) };

    if cmd.uc_luaref != LUA_NOREF {
        let text = unsafe { nlua_funcref_str(cmd.uc_luaref, ptr::null_mut()) };
        unsafe { msg_puts_hl(text, HLF_8, false) };
        unsafe { xfree(text.cast()) };
        // The definition goes on a line of its own.
        if unsafe { *cmd.uc_rep } != NUL as c_char {
            unsafe { msg_puts(c"\n                                               ".as_ptr()) };
        }
    }
    // The definition column is what is left of the line when the whole table
    // is being listed, and the whole width when one command is.
    let room = if name_len == 0 { Columns.get() - 47 } else { 0 };
    // SAFETY: module contract; `uc_rep` is the entry's own string.
    unsafe { msg_outtrans_special(cmd.uc_rep, false, room) };
    if p_verbose.get() > 0 {
        unsafe { last_set_msg(cmd.uc_script_ctx) };
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
    for (key, value) in entries {
        if let Some(value) = value {
            unsafe { dict_put(&mut dict, key, value) };
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
pub(crate) unsafe fn commands_array(buf: *mut buf_T, arena: *mut Arena) -> Dict {
    let table = if buf.is_null() {
        Table::Global
    } else {
        Table::Buffer(buf)
    };
    // SAFETY: caller contract; nothing below adds or removes a command.
    let cmds = unsafe { table.list() };
    let mut rv = arena_dict(arena, cmds.len());
    for cmd in cmds {
        // SAFETY: module contract.
        let d = unsafe { describe(cmd, arena) };
        // SAFETY: `rv` was reserved for exactly one pair per command, and
        // this is the only thing that writes to it.
        unsafe { dict_put_str(&mut rv, cstr_as_string(cmd.uc_name), Object::dict(d)) };
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
    let (name, definition) = unsafe { (cstr_as_string(cmd.uc_name), cstr_as_string(cmd.uc_rep)) };
    let complete_arg = if cmd.uc_compl_arg.is_null() {
        Object::NIL
    } else {
        // SAFETY: as above.
        Object::string(unsafe { cstr_as_string(cmd.uc_compl_arg) })
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
    let count = (a.has(ExArgt::COUNT)).then(|| {
        if cmd.uc_def >= 0 {
            // SAFETY: `arena` is the dispatcher's.
            Object::string(unsafe { arena_printf(arena, c"%ld".as_ptr(), cmd.uc_def) })
        } else {
            Object::string(static_cstring(c"0"))
        }
    });
    let range = (a.has(ExArgt::RANGE)).then(|| {
        if a.has(ExArgt::DFLALL) {
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
            (c"bang", Some(Object::boolean(a.has(ExArgt::BANG)))),
            (c"bar", Some(Object::boolean(a.has(ExArgt::TRLBAR)))),
            (c"register", Some(Object::boolean(a.has(ExArgt::REGSTR)))),
            (
                c"keepscript",
                Some(Object::boolean(a.has(ExArgt::KEEPSCRIPT))),
            ),
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
