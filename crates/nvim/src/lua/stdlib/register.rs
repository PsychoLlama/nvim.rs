//! Installing all of the above onto a `lua_State`.
//!
//! [`add_editor_stdlib`] is the registration point for everything that
//! reaches editor state, which only the main state gets: the string
//! functions, `vim.regex`, `vim.spell`, `vim.iconv`, `vim.base64` and the
//! `vim._*` internals. [`share_module`] is how a vendored library the state
//! opens becomes both a `vim.*` field and a `package.loaded` entry.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::{
    add_regex_metatable, getvar, iconv, regex, setvar, str_byteindex, str_utf_end, str_utf_pos,
    str_utf_start, str_utfindex, stricmp, with,
};
use crate::fold::fold_update;
use crate::lua::base64::push_base64;
use crate::lua::spell::push_spell;
use crate::lua::stack::{At, Raise, Stack};
use crate::lua_functions;
use crate::narrow::number_as_int;
use crate::types::LineNr;
use crate::winlayer;

lua_functions! {
    /// The string functions and `vim.regex`, in the order they are set.
    enum StringFunctions in STRING_FUNCTIONS {
        0 => c"stricmp": stricmp,
        1 => c"_str_utfindex": str_utfindex,
        2 => c"_str_byteindex": str_byteindex,
        3 => c"str_utf_pos": str_utf_pos,
        4 => c"str_utf_start": str_utf_start,
        5 => c"str_utf_end": str_utf_end,
        6 => c"regex": regex,
    }
}

lua_functions! {
    /// `vim.iconv`, which depends on 'ambiwidth' and 'emoji'.
    enum IconvFunction in ICONV_FUNCTION {
        0 => c"iconv": iconv,
    }
}

lua_functions! {
    /// Access to internal functions, for use in `runtime/`.
    enum InternalFunctions in INTERNAL_FUNCTIONS {
        0 => c"_getvar": getvar,
        1 => c"_setvar": setvar,
        2 => c"_foldupdate": foldupdate,
        3 => c"_with_c": with,
    }
}

/// An argument as the `int` a C cast narrows it to.
fn int_arg(stack: &Stack, at: c_int) -> Result<c_int, Raise> {
    let n = stack.check_integer(At::arg(at))?;
    Ok(number_as_int(i64::try_from(n).unwrap_or_default()))
}

/// `vim._foldupdate(win, top, bot)`: recompute fold levels (which can mean
/// evaluating 'foldexpr') over a zero-based end-exclusive line range, without
/// any of `zx`'s other side effects.
fn foldupdate(stack: &mut Stack) -> Result<c_int, Raise> {
    let Some(win) = winlayer::window(int_arg(stack, 1)?) else {
        return Err(Raise::message("invalid window"));
    };
    let top: LineNr = int_arg(stack, 2)? + 1;
    if top < 1 {
        return Err(Raise::message("invalid top"));
    }
    let bot: LineNr = int_arg(stack, 3)?;
    if top > bot {
        return Err(Raise::message("invalid bot"));
    }

    fold_update(win, top, bot);
    Ok(0)
}

/// Register every `vim.*` function that reaches editor state onto the `vim`
/// table on top. Only the main state gets these: a thread's state is cut
/// down to what is thread-safe.
pub fn add_editor_stdlib(stack: &mut Stack) {
    // TODO(bfredl): some of the basic string functions should already
    // be (or be easy to make) threadsafe.
    stack.register_functions(STRING_FUNCTIONS);
    add_regex_metatable(stack);

    // vim.spell
    push_spell(stack);
    stack.set_field(At::from_top(2), c"spell");

    // vim.iconv -- depends on p_ambw, p_emoji
    stack.register_functions(ICONV_FUNCTION);

    // vim.base64
    push_base64(stack);
    stack.set_field(At::from_top(2), c"base64");

    stack.register_functions(INTERNAL_FUNCTIONS);
}

/// Put the vendored library on top of the stack onto the `vim` table under
/// `name` *and* into `package.loaded`, so `require` hands back this same
/// table rather than initialising the library a second time. `depth` is how
/// far below the library's table the `vim` table has sunk -- each library
/// leaves its own table behind, so it grows.
pub(crate) fn share_module(stack: &mut Stack, name: &CStr, depth: c_int) {
    stack.push_value(At::TOP);
    stack.set_field(At::from_top(depth + 1), name);

    stack.get_global(c"package");
    stack.get_field(At::TOP, c"loaded");
    stack.push_value(At::from_top(3));
    stack.set_field(At::from_top(2), name);
    stack.pop(depth + 1);
}
