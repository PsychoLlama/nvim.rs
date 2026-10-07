//! Context: a snapshot of the whole editor state as one object.
//!
//! A [`Context`] holds four ShaDa-encoded msgpack blobs (registers, the
//! jumplist, the buffer list, global variables) plus an array of `:function`
//! definitions. `:function` bodies are captured by *executing* `func! {name}`
//! with output capture and restored by executing the text back — the same
//! round trip `nvim_get_context`/`nvim_load_context` and `ctxpush`/`ctxpop`
//! expose to scripts.
//!
//! The dict form ([`ctx_to_dict`]/[`ctx_from_dict`]) is API surface: each
//! blob appears as an array of byte-strings (`readfile()` shape), and
//! [`array_to_string`] converts one back. Any change to that shape is a
//! change to what a saved context means, so it is fixed.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::helpers::{cstr_to_string, string_to_array};
use crate::api::vimscript::exec_impl;
use crate::cstr;
use crate::eval::encode::encode_vim_list_to_buf;
use crate::eval::userfunc::all_funcs;
use crate::ex_docmd::do_cmdline_cmd;
use crate::getchar::VIML_INTERNAL_CALL;
use crate::global_cell::GlobalCell;
use crate::keycodes::K_SPECIAL;
use crate::option::{get_option_value, optval_free, set_option_value};
use crate::options::kOptShada;
use crate::shada::{
    shada_encode_buflist, shada_encode_gvars, shada_encode_jumps, shada_encode_regs,
    shada_read_string,
};
use crate::types::TypVal;
use crate::types::{
    ApiDict, Array, Context, Error, KeyDict_exec_opts, KeyValuePair, Object, OptVal,
    OptionSetFlags, String_0, VAR_LIST, size_t, uint8_t,
};
use core::ffi::{CStr, c_char, c_int};

/// The `ContextTypeFlags` a `Context` can carry, one bit per section.
pub const kCtxFuncs: ::core::ffi::c_uint = 32;
pub const kCtxSFuncs: ::core::ffi::c_uint = 16;
pub const kCtxGVars: ::core::ffi::c_uint = 8;
pub const kCtxBufs: ::core::ffi::c_uint = 4;
pub const kCtxJumps: ::core::ffi::c_uint = 2;
pub const kCtxRegs: ::core::ffi::c_uint = 1;
pub const kShaDaForceit: ::core::ffi::c_uint = 4;
pub const kShaDaWantInfo: ::core::ffi::c_uint = 1;

/// `shada_read_string` flags for every restore: read the info sections, and
/// overwrite what is already there.
const SHADA_RESTORE: c_int = kShaDaWantInfo as c_int | kShaDaForceit as c_int;

/// `'shada'` while a context is being restored: no history, 100 marks, and
/// the buffer list.
const SHADA_WHILE_RESTORING: &CStr = c"!,'100,%";

/// Every part a context can save: what `ctxpush()` with no argument takes.
pub const CTX_ALL: c_int = kCtxRegs as c_int
    | kCtxJumps as c_int
    | kCtxBufs as c_int
    | kCtxGVars as c_int
    | kCtxSFuncs as c_int
    | kCtxFuncs as c_int;

const ARRAY_INIT: Array = Array::EMPTY;
const CONTEXT_INIT: Context = Context {
    regs: String_0::NULL,
    jumps: String_0::NULL,
    bufs: String_0::NULL,
    gvars: String_0::NULL,
    funcs: ARRAY_INIT,
};

/// The `ctxpush`/`ctxpop` stack. Contexts are pushed and popped at the end,
/// but [`ctx_get`] indexes from the *top*, which is what `ctxget()` takes.
static CTX_STACK: GlobalCell<Vec<Context>> = GlobalCell::new(Vec::new());

/// How many contexts are on the stack.
pub(crate) fn ctx_size() -> size_t {
    CTX_STACK.with(Vec::len)
}

/// Run `f` on the context `index` places below the top of the stack; `None`
/// when the index is out of bounds. `f` runs under the stack's borrow, so it
/// must run no editor code.
pub(crate) fn with_ctx<R>(index: size_t, f: impl FnOnce(&mut Context) -> R) -> Option<R> {
    CTX_STACK.with_mut(|stack| {
        let at = stack.len().checked_sub(index + 1)?;
        Some(f(&mut stack[at]))
    })
}

/// Free everything a context owns.
pub fn ctx_free(ctx: &mut Context) {
    // Assigning the empty context releases the five fields it replaces.
    *ctx = CONTEXT_INIT;
}

/// Save the editor state selected by `flags` into `ctx`, or push a new
/// context on the stack when there is none.
pub fn ctx_save(ctx: Option<&mut Context>, flags: c_int) {
    match ctx {
        Some(ctx) => ctx_save_into(ctx, flags),
        None => {
            // Filled before it is pushed: each encoder runs editor code, so
            // the stack is not borrowed across them.
            let mut pushed = CONTEXT_INIT;
            ctx_save_into(&mut pushed, flags);
            CTX_STACK.with_mut(|stack| stack.push(pushed));
        }
    }
}

/// [`ctx_save`] into a context of the caller's own.
fn ctx_save_into(ctx: &mut Context, flags: c_int) {
    if flags & kCtxRegs as c_int != 0 {
        ctx.regs = shada_encode_regs();
    }
    if flags & kCtxJumps as c_int != 0 {
        ctx.jumps = shada_encode_jumps();
    }
    if flags & kCtxBufs as c_int != 0 {
        ctx.bufs = shada_encode_buflist();
    }
    if flags & kCtxGVars as c_int != 0 {
        ctx.gvars = shada_encode_gvars();
    }
    if flags & kCtxFuncs as c_int != 0 {
        ctx_save_funcs(ctx, false);
    } else if flags & kCtxSFuncs as c_int != 0 {
        ctx_save_funcs(ctx, true);
    }
}

/// Restore the editor state selected by `flags` from `ctx`, or pop the top
/// of the stack when there is none. False only when the stack is empty.
pub fn ctx_restore(ctx: Option<&Context>, flags: c_int) -> bool {
    let popped;
    let ctx = match ctx {
        Some(ctx) => ctx,
        None => {
            let Some(top) = CTX_STACK.with_mut(Vec::pop) else {
                return false;
            };
            // The popped context is owned here and freed when this returns,
            // as upstream frees the one it popped off the kvec.
            popped = top;
            &popped
        }
    };

    // Reading a context's ShaDa blobs must not be filtered by whatever the
    // user's 'shada' says.
    let op_shada = get_option_value(kOptShada, OptionSetFlags::GLOBAL);
    let _ = set_option_value(kOptShada, shada_while_restoring(), OptionSetFlags::GLOBAL);

    // SAFETY: each blob is the context's own, read as a copy.
    if flags & kCtxRegs as c_int != 0 {
        unsafe { shada_read_string(ctx.regs.clone(), SHADA_RESTORE) };
    }
    if flags & kCtxJumps as c_int != 0 {
        unsafe { shada_read_string(ctx.jumps.clone(), SHADA_RESTORE) };
    }
    if flags & kCtxBufs as c_int != 0 {
        unsafe { shada_read_string(ctx.bufs.clone(), SHADA_RESTORE) };
    }
    if flags & kCtxGVars as c_int != 0 {
        unsafe { shada_read_string(ctx.gvars.clone(), SHADA_RESTORE) };
    }
    if flags & kCtxFuncs as c_int != 0 {
        // SAFETY: the captured definitions are API strings of this context.
        unsafe { ctx_restore_funcs(ctx) };
    }

    let _ = set_option_value(kOptShada, op_shada, OptionSetFlags::GLOBAL);
    optval_free(op_shada);
    true
}

/// `'shada'` as the fixed string a restore runs under. Borrowed: the caller
/// does not free what it hands `set_option_value`.
const fn shada_while_restoring() -> OptVal {
    OptVal::static_string(SHADA_WHILE_RESTORING)
}

/// Capture every function's `:function` listing into `ctx.funcs`.
///
/// Lambdas are skipped (they have no name to redefine), and with
/// `scriptonly` so is everything but the script-local (`s:`) ones, whose
/// names start with the `K_SPECIAL` byte.
fn ctx_save_funcs(ctx: &mut Context, scriptonly: bool) {
    ctx.funcs = ARRAY_INIT;
    // Collected before any of them is executed: upstream walks the table
    // with `exec_impl` running inside the walk, which is only safe because
    // listing a function cannot define or delete one.
    for func in all_funcs() {
        let bytes = func.name().as_bytes();
        let islambda = bytes.starts_with(b"<lambda>");
        let isscript = bytes.first() == Some(&(K_SPECIAL as uint8_t));
        if islambda || (scriptonly && !isscript) {
            continue;
        }
        let mut cmd = Vec::with_capacity(b"func! ".len() + bytes.len() + 1);
        cmd.extend_from_slice(b"func! ");
        cmd.extend_from_slice(bytes);
        cmd.push(0);
        let mut opts = KeyDict_exec_opts { output: Some(true) };
        let src = unsafe { cstr_to_string(cmd.as_ptr() as *const c_char) };
        if let Ok(func_body) = exec_impl(VIML_INTERNAL_CALL, src, &mut opts) {
            ctx.funcs.push(Object::string(func_body));
        }
    }
}

/// Re-execute the captured `:function` definitions.
///
/// # Safety
/// Main-thread editor call; `ctx.funcs` holds NUL-terminated strings.
unsafe fn ctx_restore_funcs(ctx: &Context) {
    for func in &ctx.funcs {
        // `funcs` is whatever array `ctx_from_dict` was handed, so an entry
        // need not be a string. The C read the string arm under any tag;
        // anything else is skipped here.
        let Some(cmd) = func.as_string() else {
            continue;
        };
        // SAFETY: the caller's contract -- the entry is NUL-terminated.
        let _ = do_cmdline_cmd(unsafe { cstr::at(cmd.data()) });
    }
}

/// Convert a `readfile()`-style array back to the msgpack blob it encodes.
fn array_to_string(array: Array) -> Result<String_0, Error> {
    let list_tv = TypVal::from(Object::array(array));
    debug_assert!(
        list_tv.v_type() as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_uint,
        "list_tv.v_type() == VAR_LIST"
    );
    match encode_vim_list_to_buf(list_tv.list_ref()) {
        // An empty list is the NULL string, as upstream's buffer was.
        Some(bytes) if bytes.is_empty() => Ok(String_0::NULL),
        Some(bytes) => Ok(String_0::from_bytes(&bytes)),
        None => Err(Error::exception(
            c"E474: Failed to convert list to msgpack string buffer",
        )),
    }
}

/// Append one `key: [bytes...]` entry to a dict.
fn put_array(rv: &mut ApiDict, key: &CStr, array: Array) {
    rv.insert(key, Object::array(array));
}

/// The dict form of a context: each blob as an array of byte-strings, plus
/// the function bodies. This shape is API surface — see the module docs.
pub fn ctx_to_dict(ctx: &Context) -> ApiDict {
    let mut rv = ApiDict::with_capacity(5);
    put_array(&mut rv, c"regs", string_to_array(&ctx.regs, false));
    put_array(&mut rv, c"jumps", string_to_array(&ctx.jumps, false));
    put_array(&mut rv, c"bufs", string_to_array(&ctx.bufs, false));
    put_array(&mut rv, c"gvars", string_to_array(&ctx.gvars, false));
    put_array(&mut rv, c"funcs", ctx.funcs.clone());
    rv
}

/// Read a context back out of its dict form, into `ctx`. Returns the
/// `kCtx*` flags for the sections the dict actually carried; entries that
/// are not arrays, and names that are not one of the five, are ignored.
///
/// The sections read before a refusal stay in `ctx`, which the caller frees
/// either way.
pub fn ctx_from_dict(dict: ApiDict, ctx: &mut Context) -> Result<c_int, Error> {
    let mut types = 0;
    for KeyValuePair { key, value } in dict {
        let Some(array) = value.into_array() else {
            continue;
        };
        match key.bytes() {
            b"regs" => {
                types |= kCtxRegs as c_int;
                ctx.regs = array_to_string(array)?;
            }
            b"jumps" => {
                types |= kCtxJumps as c_int;
                ctx.jumps = array_to_string(array)?;
            }
            b"bufs" => {
                types |= kCtxBufs as c_int;
                ctx.bufs = array_to_string(array)?;
            }
            b"gvars" => {
                types |= kCtxGVars as c_int;
                ctx.gvars = array_to_string(array)?;
            }
            b"funcs" => {
                types |= kCtxFuncs as c_int;
                ctx.funcs = array;
            }
            _ => {}
        }
    }
    Ok(types)
}
