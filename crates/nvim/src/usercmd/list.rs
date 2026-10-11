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
//! same three tables the parser uses. Both copy the table into [`Row`]s
//! before rendering anything: printing can stop at a prompt and taking a
//! Lua reference can run Lua, and neither may happen with a table borrowed.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::attr::named_addr_type;
use super::complete::command_complete_name;
use super::{LUA_NOREF, Scope, Table};
use crate::cstr;
use crate::eval::last_set_msg;
use crate::getchar::state::got_int;
use crate::highlight_group::{HLF_8, HLF_D};
use crate::lua::executor::{api_new_luaref, funcref_description};
use crate::message::msg_display_bytes;
use crate::message::{
    message_filtered, msg, msg_display, msg_display_keys, msg_ext_set_kind, msg_putchar, msg_str,
    msg_str_hl, msg_title,
};
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::types::String_0;
use crate::types::{
    ApiDict, CmdAddr, ExArgt, ExpandContext, LuaRef, Object, ScriptCtx, UserCmd, int64_t,
};
use crate::ui::state::Columns;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use core::fmt::Write as _;
use std::rc::Rc;

/// One command, copied out of its table for rendering.
///
/// The Lua references are the table's, not taken: a row is rendered before
/// anything can delete the command, and [`describe`] takes a reference of
/// its own for the API's answer.
struct Row {
    name: Rc<CStr>,
    argt: ExArgt,
    replacement: Rc<CStr>,
    default_count: int64_t,
    complete: ExpandContext,
    addr_type: CmdAddr,
    script_ctx: ScriptCtx,
    complete_arg: Option<Rc<CStr>>,
    complete_luaref: LuaRef,
    preview_luaref: LuaRef,
    luaref: LuaRef,
}

impl Row {
    fn of(cmd: &UserCmd) -> Row {
        Row {
            name: cmd.name.clone(),
            argt: cmd.argt,
            replacement: cmd.replacement.clone(),
            default_count: cmd.default_count,
            complete: cmd.complete,
            addr_type: cmd.addr_type,
            script_ctx: cmd.script_ctx,
            complete_arg: cmd.complete_arg.clone(),
            complete_luaref: cmd.complete_luaref,
            preview_luaref: cmd.preview_luaref,
            luaref: cmd.luaref,
        }
    }
}

/// Every command of `table` whose name starts with `prefix`, copied.
fn rows(table: Table, prefix: &[u8]) -> Vec<Row> {
    table.with(|cmds| {
        cmds.iter()
            .filter(|cmd| cmd.name.to_bytes().starts_with(prefix))
            .map(Row::of)
            .collect()
    })
}

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
/// `msg_display` writes again.
///
/// `over` is how far the name column overran; every following column is
/// pulled left by it, and a column that would then start before the text
/// already written still gets its one separating space.
struct Cols {
    buf: Vec<u8>,
}

impl Cols {
    fn push(&mut self, byte: u8) {
        self.buf.push(byte);
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
            if self.buf.len() as int64_t >= col - over {
                return;
            }
        }
    }
}

impl core::fmt::Write for Cols {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        self.put(text.as_bytes());
        Ok(())
    }
}

/// `:command` -- list every user command whose name starts with `wanted`.
pub(super) fn uc_list(wanted: &[u8]) {
    msg_ext_set_kind(c"list_cmd");
    let mut found = false;

    for scope in Scope::BOTH {
        let mut interrupted = false;
        for row in rows(scope.table(), wanted) {
            if message_filtered(&row.name) {
                continue;
            }
            if !found {
                msg_title(gettext(
                    c"\n    Name              Args Address Complete    Definition",
                ));
            }
            found = true;
            msg_putchar(c_int::from(b'\n'));
            if got_int.get() {
                interrupted = true;
                break;
            }
            list_one(&row, scope, wanted.len());
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
        msg(gettext(c"No user-defined commands found"), 0);
    }
}

/// One command's row, from the flag column to the definition.
fn list_one(cmd: &Row, scope: Scope, name_len: usize) {
    let a = cmd.argt;
    // The flag column is right-aligned in four cells.
    let mut blank = 4;
    for (present, mark) in [
        (a.has(ExArgt::BANG), b'!'),
        (a.has(ExArgt::REGSTR), b'"'),
        (scope == Scope::Buffer, b'b'),
        (a.has(ExArgt::TRLBAR), b'|'),
    ] {
        if present {
            msg_putchar(c_int::from(mark));
            blank -= 1;
        }
    }
    if blank != 0 {
        msg_str(cstr::in_bytes(&b"    \0"[4 - blank..]));
    }

    msg_display(&cmd.name, HLF_D, false);
    // The name column is 17 wide; a longer name pushes the rest left.
    let mut len = cmd.name.to_bytes().len() + 4;
    if len < 21 {
        // Field padding spaces   12345678901234567
        static SPACES: &[u8] = b"                 \0";
        msg_str(cstr::in_bytes(&SPACES[len - 4..]));
        len = 21;
    }
    msg_putchar(c_int::from(b' '));
    len += 1;
    let over = len as int64_t - 22;

    // The middle columns are assembled in one buffer and printed once.
    // Upstream uses `IObuff`; the columns are a few dozen bytes.
    let middle = {
        let mut cols = Cols {
            buf: Vec::with_capacity(32),
        };
        cols.put(nargs_str(a).to_bytes());
        cols.pad_to(5, over);

        if a.has(ExArgt::RANGE | ExArgt::COUNT) {
            if a.has(ExArgt::COUNT) {
                // -count=N
                let _ = write!(cols, "{}c", cmd.default_count);
            } else if a.has(ExArgt::DFLALL) {
                cols.push(b'%');
            } else if cmd.default_count >= 0 {
                // -range=N
                let _ = write!(cols, "{}", cmd.default_count);
            } else {
                cols.push(b'.');
            }
        }
        cols.pad_to(8, over);

        if let Some(row) = named_addr_type(cmd.addr_type) {
            cols.put(row.shortname.to_bytes());
        }
        cols.pad_to(13, over);

        if let Some(name) = command_complete_name(cmd.complete) {
            cols.put(name.to_bytes());
        }
        cols.pad_to(25, over);
        cols.buf
    };
    msg_display_bytes(&middle, 0, false);

    if cmd.luaref != LUA_NOREF {
        if let Some(text) = funcref_description(cmd.luaref) {
            msg_str_hl(&text, HLF_8, false);
        }
        // The definition goes on a line of its own.
        if !cmd.replacement.is_empty() {
            msg_str(c"\n                                               ");
        }
    }
    // The definition column is what is left of the line when the whole table
    // is being listed, and the whole width when one command is.
    let room = if name_len == 0 { Columns.get() - 47 } else { 0 };
    msg_display_keys(&cmd.replacement, false, room);
    if p_verbose() > 0 {
        last_set_msg(cmd.script_ctx);
    }
}

/// Collect the entries that are present into a Dict.
///
/// `capacity` is upstream's, not `N`: sizing and filling from one array is
/// what makes the puts sound, and the two must agree, so the assertion is
/// the contract rather than a hope.
fn dict_of<const N: usize>(
    capacity: usize,
    entries: [(&'static CStr, Option<Object>); N],
) -> ApiDict {
    debug_assert!(N <= capacity, "dict_of past capacity");
    let mut dict = ApiDict::with_capacity(capacity);
    // SAFETY: the dict was reserved for `capacity` pairs, at most `N` are
    // written, and this is the only thing that writes to it.
    for (key, value) in entries {
        if let Some(value) = value {
            dict.insert(key, value);
        }
    }
    dict
}

/// `nvim_get_commands()`: every user command of `buffer`, or every global one
/// when `buffer` is `None`, as a map from name to description.
pub(crate) fn commands_array(buffer: Option<Buf>) -> ApiDict {
    let rows = rows(buffer.map_or(Table::Global, Table::Buffer), b"");
    let mut rv = ApiDict::with_capacity(rows.len());
    for row in &rows {
        rv.insert(row.name.to_bytes(), Object::dict(describe(row)));
    }
    rv
}

/// One command as the API describes it. The key order is upstream's and is
/// what the wire format carries.
fn describe(cmd: &Row) -> ApiDict {
    let a = cmd.argt;
    let luaref = |r: LuaRef| (r != LUA_NOREF).then(|| Object::luaref(api_new_luaref(r)));
    let name = String_0::from_cstr(&cmd.name);
    let definition = String_0::from_cstr(&cmd.replacement);
    let complete_arg = cmd
        .complete_arg
        .as_deref()
        .map_or(Object::Nil, |arg| Object::string(String_0::from_cstr(arg)));
    // The completion is a Lua reference when the command was given one,
    // and the `-complete=` name otherwise.
    let complete = match luaref(cmd.complete_luaref) {
        Some(callback) => callback,
        None => match command_complete_name(cmd.complete) {
            Some(text) => Object::string(String_0::from_cstr(text)),
            None => Object::Nil,
        },
    };
    let number = |n: int64_t| Object::string(String_0::from_bytes(n.to_string().as_bytes()));
    let count = (a.has(ExArgt::COUNT)).then(|| {
        if cmd.default_count >= 0 {
            number(cmd.default_count)
        } else {
            Object::string(String_0::from_cstr(c"0"))
        }
    });
    let range = (a.has(ExArgt::RANGE)).then(|| {
        if a.has(ExArgt::DFLALL) {
            Object::string(String_0::from_cstr(c"%"))
        } else if cmd.default_count >= 0 {
            number(cmd.default_count)
        } else {
            Object::string(String_0::from_cstr(c"."))
        }
    });
    let nargs = String_0::from_cstr(nargs_str(a));

    dict_of(
        16,
        [
            (c"name", Some(Object::string(name))),
            (c"definition", Some(Object::string(definition))),
            (
                c"script_id",
                Some(Object::integer(cmd.script_ctx.sc_sid.into())),
            ),
            (c"bang", Some(Object::boolean(a.has(ExArgt::BANG)))),
            (c"bar", Some(Object::boolean(a.has(ExArgt::TRLBAR)))),
            (c"register", Some(Object::boolean(a.has(ExArgt::REGSTR)))),
            (
                c"keepscript",
                Some(Object::boolean(a.has(ExArgt::KEEPSCRIPT))),
            ),
            (c"preview", luaref(cmd.preview_luaref)),
            (c"callback", luaref(cmd.luaref)),
            (c"nargs", Some(Object::string(nargs))),
            (c"complete", Some(complete)),
            (c"complete_arg", Some(complete_arg)),
            (c"count", Some(count.unwrap_or(Object::Nil))),
            (c"range", Some(range.unwrap_or(Object::Nil))),
            (c"addr", Some(addr_object(cmd))),
        ],
    )
}

/// The `addr` field: the name of the command's address type, when it has
/// one that is not the default.
fn addr_object(cmd: &Row) -> Object {
    match named_addr_type(cmd.addr_type) {
        Some(row) => Object::string(String_0::from_cstr(row.name)),
        None => Object::Nil,
    }
}
