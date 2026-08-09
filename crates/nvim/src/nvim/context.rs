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

use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_string, api_set_error, arena_dict, copy_array,
    copy_object, cstr_as_string, string_to_array,
};
use crate::src::nvim::api::vimscript::exec_impl;
use crate::src::nvim::eval::encode::encode_vim_list_to_buf;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::eval::userfunc::func_tbl_get;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::getchar::VIML_INTERNAL_CALL;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::keycodes::K_SPECIAL;
use crate::src::nvim::memory::{strequal, xrealloc};
use crate::src::nvim::option::{get_option_value, optval_free, set_option_value};
use crate::src::nvim::options::kOptShada;
use crate::src::nvim::shada::{
    shada_encode_buflist, shada_encode_gvars, shada_encode_jumps, shada_encode_regs,
    shada_read_string,
};
use crate::src::nvim::types::{
    Arena, Array, Context, Dict, Error, KeyDict_exec_opts, KeyValuePair, Object, OptVal,
    OptValData, OptValType, String_0, VAR_LIST, VAR_UNKNOWN, VAR_UNLOCKED, hashitem_T,
    kErrorTypeException, kErrorTypeNone, kObjectTypeArray, kObjectTypeString, key_value_pair,
    object, object_data as C2Rust_Unnamed_0, size_t, typval_T, typval_vval_union, uint8_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};

pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;

/// The `ContextTypeFlags` a `Context` can carry, one bit per section.
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const kCtxFuncs: C2Rust_Unnamed_1 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_1 = 16;
pub const kCtxGVars: C2Rust_Unnamed_1 = 8;
pub const kCtxBufs: C2Rust_Unnamed_1 = 4;
pub const kCtxJumps: C2Rust_Unnamed_1 = 2;
pub const kCtxRegs: C2Rust_Unnamed_1 = 1;

pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const OPT_GLOBAL: C2Rust_Unnamed_2 = 1;
pub const OPT_SKIPRTP: C2Rust_Unnamed_2 = 128;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_2 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_2 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_2 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_2 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_2 = 2;

pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const kShaDaForceit: C2Rust_Unnamed_3 = 4;
pub const kShaDaWantInfo: C2Rust_Unnamed_3 = 1;

/// `shada_read_string` flags for every restore: read the info sections, and
/// overwrite what is already there.
const SHADA_RESTORE: c_int = kShaDaWantInfo as c_int | kShaDaForceit as c_int;

/// `'shada'` while a context is being restored: no history, 100 marks, and
/// the buffer list.
const SHADA_WHILE_RESTORING: &CStr = c"!,'100,%";

pub static kCtxAll: GlobalCell<c_int> = GlobalCell::new(
    kCtxRegs as c_int
        | kCtxJumps as c_int
        | kCtxBufs as c_int
        | kCtxGVars as c_int
        | kCtxSFuncs as c_int
        | kCtxFuncs as c_int,
);

const ARRAY_INIT: Array = Array {
    size: 0,
    capacity: 0,
    items: core::ptr::null_mut(),
};
const STRING_INIT: String_0 = String_0 {
    data: core::ptr::null_mut(),
    size: 0,
};
const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: ARRAY_INIT,
};

/// The `ctxpush`/`ctxpop` stack. Contexts are pushed and popped at the end,
/// but [`ctx_get`] indexes from the *top*, which is what `ctxget()` takes.
static CTX_STACK: GlobalCell<Vec<Context>> = GlobalCell::new(Vec::new());

/// How many contexts are on the stack.
///
/// # Safety
/// Main-thread editor call.
pub unsafe fn ctx_size() -> size_t {
    CTX_STACK.with(Vec::len)
}

/// The context `index` places below the top of the stack, or null when the
/// index is out of bounds.
///
/// # Safety
/// Main-thread editor call. The pointer is into the stack's storage, so it
/// is invalidated by any later push or pop — exactly as the C `kvec` one
/// was.
pub unsafe fn ctx_get(index: size_t) -> *mut Context {
    CTX_STACK.with_mut(|stack| match stack.len().checked_sub(index + 1) {
        Some(at) => &raw mut stack[at],
        None => core::ptr::null_mut(),
    })
}

/// Free everything a context owns.
///
/// # Safety
/// `ctx` is a live context whose blobs are owned.
pub unsafe fn ctx_free(ctx: *mut Context) {
    // SAFETY: the caller's context.
    unsafe {
        let ctx = &mut *ctx;
        api_free_string(ctx.regs);
        api_free_string(ctx.jumps);
        api_free_string(ctx.bufs);
        api_free_string(ctx.gvars);
        api_free_array(ctx.funcs);
    }
}

/// Save the editor state selected by `flags` into `ctx`, or push a new
/// context on the stack when `ctx` is null.
///
/// # Safety
/// Main-thread editor call; `ctx` is null or a live, empty context.
pub unsafe fn ctx_save(ctx: *mut Context, flags: c_int) {
    let ctx = if ctx.is_null() {
        CTX_STACK.with_mut(|stack| {
            stack.push(CONTEXT_INIT);
            let at = stack.len() - 1;
            &raw mut stack[at]
        })
    } else {
        ctx
    };
    // SAFETY: either the caller's context or the one just pushed. Each
    // encoder runs editor code, so the stack is not borrowed across them.
    unsafe {
        let ctx = &mut *ctx;
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
}

/// Restore the editor state selected by `flags` from `ctx`, or pop the top
/// of the stack when `ctx` is null. False only when the stack is empty.
///
/// # Safety
/// Main-thread editor call; `ctx` is null or a live context.
pub unsafe fn ctx_restore(ctx: *mut Context, flags: c_int) -> bool {
    let mut popped = None;
    let ctx = if ctx.is_null() {
        let Some(top) = CTX_STACK.with_mut(Vec::pop) else {
            return false;
        };
        // The popped context is owned here; it is freed at the end, as
        // upstream frees the one it popped off the kvec.
        &raw mut *popped.insert(top)
    } else {
        ctx
    };

    // Reading a context's ShaDa blobs must not be filtered by whatever the
    // user's 'shada' says.
    // SAFETY: main-thread editor call; the option value is owned here.
    unsafe {
        let op_shada = get_option_value(kOptShada, OPT_GLOBAL as c_int);
        set_option_value(kOptShada, shada_while_restoring(), OPT_GLOBAL as c_int);

        if flags & kCtxRegs as c_int != 0 {
            shada_read_string((*ctx).regs, SHADA_RESTORE);
        }
        if flags & kCtxJumps as c_int != 0 {
            shada_read_string((*ctx).jumps, SHADA_RESTORE);
        }
        if flags & kCtxBufs as c_int != 0 {
            shada_read_string((*ctx).bufs, SHADA_RESTORE);
        }
        if flags & kCtxGVars as c_int != 0 {
            shada_read_string((*ctx).gvars, SHADA_RESTORE);
        }
        if flags & kCtxFuncs as c_int != 0 {
            ctx_restore_funcs(&*ctx);
        }
        if popped.is_some() {
            ctx_free(ctx);
        }

        set_option_value(kOptShada, op_shada, OPT_GLOBAL as c_int);
        optval_free(op_shada);
    }
    true
}

/// `'shada'` as the fixed string a restore runs under.
fn shada_while_restoring() -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0 {
                data: SHADA_WHILE_RESTORING.as_ptr().cast_mut(),
                size: SHADA_WHILE_RESTORING.count_bytes(),
            },
        },
    }
}

/// Every name in the global function table, in hash-table order.
///
/// Collected before any of them is executed: upstream walks the table with
/// `exec_impl` running inside the walk, which is only safe because listing a
/// function cannot define or delete one.
///
/// # Safety
/// Main-thread editor call; the function table is live.
unsafe fn func_names() -> Vec<*const c_char> {
    let mut names = Vec::new();
    // SAFETY: `ht_used` bounds how many occupied slots the walk will find,
    // and the array holds at least that many past the ones it skips.
    unsafe {
        let functbl = func_tbl_get();
        let mut todo = (*functbl).ht_used;
        let mut hi: *mut hashitem_T = (*functbl).ht_array;
        while todo != 0 {
            if !(*hi).hi_key.is_null() && (*hi).hi_key != &raw const hash_removed as *mut c_char {
                todo -= 1;
                names.push((*hi).hi_key as *const c_char);
            }
            hi = hi.offset(1);
        }
    }
    names
}

/// `kv_push` on an API `Array`: grow by doubling from 8 and append, which is
/// what the transpiled `ADD` did.
///
/// # Safety
/// `arr.items` is null or an `xmalloc`'d array of `arr.capacity` objects.
unsafe fn array_push(arr: &mut Array, value: Object) {
    // SAFETY: the caller's array.
    unsafe {
        if arr.size == arr.capacity {
            arr.capacity = if arr.capacity != 0 {
                arr.capacity << 1
            } else {
                8
            };
            arr.items = xrealloc(arr.items as *mut c_void, size_of::<Object>() * arr.capacity)
                as *mut Object;
        }
        *arr.items.add(arr.size) = value;
        arr.size += 1;
    }
}

/// Capture every function's `:function` listing into `ctx.funcs`.
///
/// Lambdas are skipped (they have no name to redefine), and with
/// `scriptonly` so is everything but the script-local (`s:`) ones, whose
/// names start with the `K_SPECIAL` byte.
///
/// # Safety
/// Main-thread editor call; the function table is live.
unsafe fn ctx_save_funcs(ctx: &mut Context, scriptonly: bool) {
    ctx.funcs = ARRAY_INIT;
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: core::ptr::null_mut(),
    };
    // SAFETY: the caller's contract; every name is NUL-terminated and alive
    // for the walk, and `cmd` is owned until `exec_impl` has copied it.
    unsafe {
        for name in func_names() {
            let bytes = CStr::from_ptr(name).to_bytes();
            let islambda = bytes.starts_with(b"<lambda>");
            let isscript = bytes.first() == Some(&(K_SPECIAL as uint8_t));
            if islambda || (scriptonly && !isscript) {
                continue;
            }
            let mut cmd = Vec::with_capacity(b"func! ".len() + bytes.len() + 1);
            cmd.extend_from_slice(b"func! ");
            cmd.extend_from_slice(bytes);
            cmd.push(0);
            let mut opts = KeyDict_exec_opts { output: true };
            let func_body = exec_impl(
                VIML_INTERNAL_CALL,
                cstr_as_string(cmd.as_ptr() as *const c_char),
                &raw mut opts,
                &raw mut err,
            );
            if err.type_0 as c_int == kErrorTypeNone as c_int {
                array_push(
                    &mut ctx.funcs,
                    object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_0 { string: func_body },
                    },
                );
            }
            api_clear_error(&raw mut err);
        }
    }
}

/// Re-execute the captured `:function` definitions.
///
/// # Safety
/// Main-thread editor call; `ctx.funcs` holds NUL-terminated strings.
unsafe fn ctx_restore_funcs(ctx: &Context) {
    // SAFETY: the caller's contract.
    unsafe {
        for i in 0..ctx.funcs.size {
            do_cmdline_cmd((*ctx.funcs.items.add(i)).data.string.data);
        }
    }
}

/// Convert a `readfile()`-style array back to the msgpack blob it encodes.
///
/// # Safety
/// Main-thread editor call; `err` is a live error object.
unsafe fn array_to_string(array: Array, err: *mut Error) -> String_0 {
    let mut sbuf = STRING_INIT;
    let mut list_tv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    // SAFETY: the caller's array and error; `list_tv` owns the conversion
    // result until `tv_clear`.
    unsafe {
        object_to_vim(
            object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_0 { array },
            },
            &raw mut list_tv,
            err,
        );
        debug_assert!(
            list_tv.v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_uint,
            "list_tv.v_type == VAR_LIST"
        );
        if !encode_vim_list_to_buf(list_tv.vval.v_list, &raw mut sbuf.size, &raw mut sbuf.data) {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                c"E474: Failed to convert list to msgpack string buffer".as_ptr(),
            );
        }
        tv_clear(&raw mut list_tv);
    }
    sbuf
}

/// Append one `key: [bytes...]` entry to an arena-allocated dict.
///
/// # Safety
/// `rv` has room for another entry, and `key` is a NUL-terminated literal.
unsafe fn put_array(rv: &mut Dict, key: &CStr, array: Array) {
    // SAFETY: the caller's contract.
    unsafe {
        *rv.items.add(rv.size) = key_value_pair {
            key: cstr_as_string(key.as_ptr()),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_0 { array },
            },
        };
        rv.size += 1;
    }
}

/// The dict form of a context: each blob as an array of byte-strings, plus
/// the function bodies. This shape is API surface — see the module docs.
///
/// # Safety
/// Main-thread editor call; `ctx` is a live context and `arena` a live
/// arena.
pub unsafe fn ctx_to_dict(ctx: *mut Context, arena: *mut Arena) -> Dict {
    debug_assert!(!ctx.is_null(), "ctx != NULL");
    // SAFETY: the caller's context and arena; the dict is sized for the five
    // entries put into it.
    unsafe {
        let ctx = &*ctx;
        let mut rv = arena_dict(arena, 5);
        put_array(&mut rv, c"regs", string_to_array(ctx.regs, false, arena));
        put_array(&mut rv, c"jumps", string_to_array(ctx.jumps, false, arena));
        put_array(&mut rv, c"bufs", string_to_array(ctx.bufs, false, arena));
        put_array(&mut rv, c"gvars", string_to_array(ctx.gvars, false, arena));
        put_array(&mut rv, c"funcs", copy_array(ctx.funcs, arena));
        rv
    }
}

/// Read a context back out of its dict form, into `ctx`. Returns the
/// `kCtx*` flags for the sections the dict actually carried; entries that
/// are not arrays, and names that are not one of the five, are ignored.
///
/// # Safety
/// Main-thread editor call; `ctx` is a live context and `err` a live error.
pub unsafe fn ctx_from_dict(dict: Dict, ctx: *mut Context, err: *mut Error) -> c_int {
    debug_assert!(!ctx.is_null(), "ctx != NULL");
    let mut types = 0;
    // SAFETY: the caller's dict, context and error.
    unsafe {
        let ctx = &mut *ctx;
        for i in 0..dict.size {
            if (*err).type_0 as c_int != kErrorTypeNone as c_int {
                break;
            }
            let item: KeyValuePair = *dict.items.add(i);
            if item.value.type_0 as ::core::ffi::c_uint != kObjectTypeArray as ::core::ffi::c_uint {
                continue;
            }
            let array = item.value.data.array;
            if strequal(item.key.data, c"regs".as_ptr()) {
                types |= kCtxRegs as c_int;
                ctx.regs = array_to_string(array, err);
            } else if strequal(item.key.data, c"jumps".as_ptr()) {
                types |= kCtxJumps as c_int;
                ctx.jumps = array_to_string(array, err);
            } else if strequal(item.key.data, c"bufs".as_ptr()) {
                types |= kCtxBufs as c_int;
                ctx.bufs = array_to_string(array, err);
            } else if strequal(item.key.data, c"gvars".as_ptr()) {
                types |= kCtxGVars as c_int;
                ctx.gvars = array_to_string(array, err);
            } else if strequal(item.key.data, c"funcs".as_ptr()) {
                types |= kCtxFuncs as c_int;
                ctx.funcs = copy_object(item.value, core::ptr::null_mut::<Arena>())
                    .data
                    .array;
            }
        }
    }
    types
}
