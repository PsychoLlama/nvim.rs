//! The `Callback` value: building one from a typval, and calling it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::guard::Depth;
use core::ffi::{CStr, c_char, c_int};
use core::ptr::{null, null_mut};

use crate::ascii::ascii_isdigit;
use crate::eval::collect::set_ref_in_item;
use crate::eval::typval::{kCallbackFuncref, kCallbackLua, kCallbackNone, kCallbackPartial};
use crate::eval::userfunc::{call_func, func_ref, get_scriptlocal_funcname};
use crate::eval::vars::emsg_static;
use crate::eval::vars::get_vim_var_partial;
use crate::eval::window::cur_win;
use crate::eval::{
    ARRAY_DICT_INIT, Cb, FUNCEXE_INIT, Tv, callback_depth, check_luafunc_name, kRetNilBool,
    partial_name,
};
use crate::lua::executor::{
    nlua_call_ref_quiet, nlua_is_table_from_lua, nlua_register_table_as_callable,
};
use crate::main::{e_command_too_recursive, p_mfd};
use crate::memory::xstrdup;
use crate::types::{
    Arena, Callback, CallbackReader, FAIL, NUL, OK, OptInt, VAR_DICT, VAR_FUNC, VAR_NUMBER,
    VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, Vv, funcexe_T, ht_stack_T,
    kObjectTypeBoolean, list_stack_T, partial_T, size_t, typval_T, typval_vval_union,
};
use ::libc::{memcmp, strlen};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// Build a `Callback` out of whatever the user handed a builtin.
///
/// A String or a Funcref names a function; a partial is taken as it is; a
/// Lua table with a `__call` is registered as one. `v:null` and the Number
/// 0 mean "no callback", which is not an error. A String that *starts with
/// a digit* is refused, because that is a channel id rather than a name.
///
/// # Safety
/// `callback` and `arg` must be valid.
pub unsafe fn callback_from_typval(callback: *mut Callback, arg: *const typval_T) -> bool {
    // SAFETY: the caller's promise -- both pointees outlive the call. `arg`
    // is only ever read through, which is what makes casting its `const`
    // away sound.
    let (mut cb, tv) = unsafe { (Cb::new(callback), Tv::new(arg.cast_mut())) };
    let mut r = OK;
    // Every union read below is guarded by the `v_type` that names the live
    // member, which is the promise each SAFETY note restates.
    if tv.v_type == VAR_PARTIAL && !unsafe { tv.vval.v_partial }.is_null() {
        // SAFETY: `VAR_PARTIAL` says `v_partial` is the live member.
        let partial = unsafe { tv.vval.v_partial };
        cb.data.partial = partial;
        // SAFETY: the typval holds a live partial.
        unsafe { (*partial).pt_refcount.retain() };
        cb.type_0 = kCallbackPartial;
    } else if tv.v_type == VAR_STRING
        // SAFETY: `VAR_STRING` says `v_string` is the live member, and a
        // non-null one is NUL-terminated, so its first byte is readable.
        && !unsafe { tv.vval.v_string }.is_null()
        && ascii_isdigit(unsafe { *tv.vval.v_string } as c_int)
    {
        r = FAIL;
    } else if tv.v_type == VAR_FUNC || tv.v_type == VAR_STRING {
        // SAFETY: both types keep their name in `v_string`.
        let name = unsafe { tv.vval.v_string };
        if name.is_null() {
            r = FAIL;
        // SAFETY: a non-null name is NUL-terminated.
        } else if unsafe { *name } as c_int == NUL {
            cb.type_0 = kCallbackNone;
            cb.data.funcref = null_mut();
        } else {
            // A plain String may name a script-local function, which
            // has to be resolved against the current script now.
            cb.data.funcref = null_mut();
            if tv.v_type == VAR_STRING {
                // SAFETY: `name` is the typval's NUL-terminated string.
                cb.data.funcref = unsafe { get_scriptlocal_funcname(name) };
            }
            // SAFETY: `funcref` is the member written just above.
            if unsafe { cb.data.funcref }.is_null() {
                // SAFETY: as above -- `name` is NUL-terminated.
                cb.data.funcref = unsafe { xstrdup(name) };
            }
            // SAFETY: the name just stored is a live owned string.
            unsafe { func_ref(cb.data.funcref) };
            cb.type_0 = kCallbackFuncref;
        }
    // SAFETY: the caller's promise about `arg`.
    } else if unsafe { nlua_is_table_from_lua(arg) } {
        // SAFETY: as above; the table has a `__call`.
        let name = unsafe { nlua_register_table_as_callable(arg) };
        if name.is_null() {
            r = FAIL;
        } else {
            // SAFETY: `name` is the registered function's name.
            cb.data.funcref = unsafe { xstrdup(name) };
            cb.type_0 = kCallbackFuncref;
        }
    } else if tv.v_type == VAR_SPECIAL
        // SAFETY: `VAR_NUMBER` says `v_number` is the live member.
        || (tv.v_type == VAR_NUMBER && unsafe { tv.vval.v_number } == 0)
    {
        cb.type_0 = kCallbackNone;
        cb.data.funcref = null_mut();
    } else {
        r = FAIL;
    }

    if r == FAIL {
        // SAFETY: the message is a NUL-terminated literal.
        emsg_static(c"E921: Invalid callback argument");
        return false;
    }
    true
}

/// How deep the callback nesting currently is.
pub fn get_callback_depth() -> c_int {
    callback_depth.get()
}

/// The prefix that makes a funcref name a Lua one.
const VLUA: &CStr = c"v:lua.";

/// Call `callback` with `argcount_in` arguments.
///
/// # Safety
/// `callback` and `rettv` must be valid; `argvars_in` must hold
/// `argcount_in` typvals.
pub unsafe fn callback_call(
    callback: *mut Callback,
    argcount_in: c_int,
    argvars_in: *mut typval_T,
    rettv: *mut typval_T,
) -> bool {
    if callback_depth.get() as OptInt > p_mfd.get() {
        // SAFETY: the message is a NUL-terminated literal.
        emsg_static(e_command_too_recursive);
        return false;
    }

    // SAFETY: the caller's promise -- the callback outlives the call.
    let cb = unsafe { Cb::new(callback) };
    let mut partial: *mut partial_T = null_mut();
    let mut name: *mut c_char;
    match cb.type_0 {
        kCallbackFuncref => {
            // SAFETY: `kCallbackFuncref` says `funcref` is the live member,
            // and it holds a NUL-terminated name.
            name = unsafe { cb.data.funcref };
            let len = unsafe { strlen(name) } as c_int;
            let vlua = VLUA.as_ptr().cast();
            // SAFETY: `len >= 6` promises six readable bytes on both sides.
            if len >= 6 && unsafe { memcmp(name.cast(), vlua, 6 as size_t) } == 0 {
                // SAFETY: the six bytes just compared are behind us, so what
                // is left is still inside the name.
                name = unsafe { name.add(6) };
                // SAFETY: `name` is NUL-terminated.
                if unsafe { check_luafunc_name(name, false) } == 0 {
                    return false;
                }
                partial = unsafe { get_vim_var_partial(Vv::Lua) };
            }
        }
        kCallbackPartial => {
            // SAFETY: `kCallbackPartial` says `partial` is the live member.
            partial = unsafe { cb.data.partial };
            // SAFETY: the callback holds a live partial.
            name = unsafe { partial_name(partial) };
        }
        kCallbackLua => {
            // A Lua reference is called directly, with no arguments —
            // this is the "is it still wanted" question, not a
            // general-purpose call.
            let no_args = ARRAY_DICT_INIT;
            let arena = null_mut::<Arena>();
            // SAFETY: `kCallbackLua` says `luaref` is the live member.
            let luaref = unsafe { cb.data.luaref };
            // SAFETY: the reference is the one the callback owns, and the
            // call is handed no arguments, no arena and no error sink.
            let rv = unsafe { nlua_call_ref_quiet(luaref, null(), no_args, kRetNilBool, arena) };
            // SAFETY: a boolean object holds its value inline.
            return rv.type_0 == kObjectTypeBoolean && unsafe { rv.data.boolean };
        }
        // kCallbackNone, and anything else.
        _ => return false,
    }

    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = cur_win().w_cursor.lnum;
    funcexe.fe_lastline = cur_win().w_cursor.lnum;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    // The un-bump is the guard's, so that an early exit cannot skip it.
    let depth = Depth::of(&callback_depth);
    // SAFETY: `name` is a NUL-terminated function name, and the caller's
    // promise covers `argvars_in`, `argcount_in` and `rettv`.
    let ret = unsafe { call_func(name, -1, rettv, argcount_in, argvars_in, &raw mut funcexe) };
    drop(depth);
    ret.is_ok()
}

/// Mark what a callback keeps alive for the collector.
///
/// # Safety
/// `callback` must be valid; the two stacks as `set_ref_in_item`'s.
pub unsafe fn set_ref_in_callback(
    callback: *mut Callback,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    // SAFETY: the caller's promise -- the callback outlives the call.
    let cb = unsafe { Cb::new(callback) };
    match cb.type_0 {
        kCallbackPartial => {
            let mut tv = UNSET_TV;
            tv.v_type = VAR_PARTIAL;
            // SAFETY: `kCallbackPartial` says `partial` is the live member.
            tv.vval.v_partial = unsafe { cb.data.partial };
            // SAFETY: `tv` is this frame's, and the stacks are the caller's.
            unsafe { set_ref_in_item(&raw mut tv, copy_id, ht_stack, list_stack) }
        }
        // A Lua reference is the Lua garbage collector's, not this one's,
        // and nothing that reaches here should hold one.
        kCallbackLua => unreachable!("set_ref_in_callback on a Lua callback"),
        // kCallbackFuncref and kCallbackNone hold nothing collectable.
        _ => false,
    }
}

/// Mark what a callback *reader* keeps alive: its callback, and the `self`
/// dictionary it would be called with.
///
/// # Safety
/// As `set_ref_in_callback`.
pub(crate) unsafe fn set_ref_in_callback_reader(
    reader: *mut CallbackReader,
    copy_id: c_int,
    ht_stack: *mut *mut ht_stack_T,
    list_stack: *mut *mut list_stack_T,
) -> bool {
    // SAFETY: the caller's promise -- the reader outlives the call, and its
    // `cb` is the callback it owns.
    if unsafe { set_ref_in_callback(&raw mut (*reader).cb, copy_id, ht_stack, list_stack) } {
        return true;
    }
    // SAFETY: as above.
    let self_dict = unsafe { (*reader).self_0 };
    if !self_dict.is_null() {
        let mut tv = UNSET_TV;
        tv.v_type = VAR_DICT;
        tv.vval.v_dict = self_dict;
        // SAFETY: `tv` is this frame's, and the stacks are the caller's.
        return unsafe { set_ref_in_item(&raw mut tv, copy_id, ht_stack, list_stack) };
    }
    false
}
