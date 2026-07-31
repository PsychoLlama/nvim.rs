//! The `Callback` value: building one from a typval, and calling it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr::{null, null_mut};

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::eval::collect::set_ref_in_item;
use crate::src::nvim::eval::typval::{
    kCallbackFuncref, kCallbackLua, kCallbackNone, kCallbackPartial,
};
use crate::src::nvim::eval::userfunc::{call_func, func_ref, get_scriptlocal_funcname};
use crate::src::nvim::eval::vars::get_vim_var_partial;
use crate::src::nvim::eval::{
    ARRAY_DICT_INIT, FAIL, FUNCEXE_INIT, NUL, OK, VAR_DICT, VAR_FUNC, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_LUA, callback_depth, check_luafunc_name,
    kRetNilBool, partial_name,
};
use crate::src::nvim::lua::executor::{
    nlua_call_ref, nlua_is_table_from_lua, nlua_register_table_as_callable,
};
use crate::src::nvim::main::{curwin, e_command_too_recursive, p_mfd};
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, memcmp, strlen};
use crate::src::nvim::types::{
    Arena, Callback, CallbackReader, Error, Object, OptInt, funcexe_T, ht_stack_T,
    kObjectTypeBoolean, list_stack_T, partial_T, size_t, typval_T, typval_vval_union,
};

/// A Lua reference. Not in the parent's preamble because nothing there
/// named it; the other three `kCallback*` tags are.

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
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
    unsafe {
        let mut r = OK;
        if (*arg).v_type == VAR_PARTIAL && !(*arg).vval.v_partial.is_null() {
            (*callback).data.partial = (*arg).vval.v_partial;
            (*(*callback).data.partial).pt_refcount += 1;
            (*callback).type_0 = kCallbackPartial;
        } else if (*arg).v_type == VAR_STRING
            && !(*arg).vval.v_string.is_null()
            && ascii_isdigit(*(*arg).vval.v_string as c_int)
        {
            r = FAIL;
        } else if (*arg).v_type == VAR_FUNC || (*arg).v_type == VAR_STRING {
            let name = (*arg).vval.v_string;
            if name.is_null() {
                r = FAIL;
            } else if *name as c_int == NUL {
                (*callback).type_0 = kCallbackNone;
                (*callback).data.funcref = null_mut();
            } else {
                // A plain String may name a script-local function, which
                // has to be resolved against the current script now.
                (*callback).data.funcref = null_mut();
                if (*arg).v_type == VAR_STRING {
                    (*callback).data.funcref = get_scriptlocal_funcname(name);
                }
                if (*callback).data.funcref.is_null() {
                    (*callback).data.funcref = xstrdup(name);
                }
                func_ref((*callback).data.funcref);
                (*callback).type_0 = kCallbackFuncref;
            }
        } else if nlua_is_table_from_lua(arg) {
            let name = nlua_register_table_as_callable(arg);
            if name.is_null() {
                r = FAIL;
            } else {
                (*callback).data.funcref = xstrdup(name);
                (*callback).type_0 = kCallbackFuncref;
            }
        } else if (*arg).v_type == VAR_SPECIAL
            || ((*arg).v_type == VAR_NUMBER && (*arg).vval.v_number == 0)
        {
            (*callback).type_0 = kCallbackNone;
            (*callback).data.funcref = null_mut();
        } else {
            r = FAIL;
        }

        if r == FAIL {
            emsg(gettext(c"E921: Invalid callback argument".as_ptr()));
            return false;
        }
        true
    }
}

/// How deep the callback nesting currently is.
pub fn get_callback_depth() -> c_int {
    callback_depth.get()
}

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
    unsafe {
        if callback_depth.get() as OptInt > p_mfd.get() {
            emsg(gettext(e_command_too_recursive.ptr().cast()));
            return false;
        }

        let mut partial: *mut partial_T = null_mut();
        let mut name: *mut c_char;
        match (*callback).type_0 {
            kCallbackFuncref => {
                name = (*callback).data.funcref;
                let len = strlen(name) as c_int;
                if len >= 6 && memcmp(name.cast(), c"v:lua.".as_ptr().cast(), 6 as size_t) == 0 {
                    name = name.add(6);
                    if check_luafunc_name(name, false) == 0 {
                        return false;
                    }
                    partial = get_vim_var_partial(VV_LUA);
                }
            }
            kCallbackPartial => {
                partial = (*callback).data.partial;
                name = partial_name(partial);
            }
            kCallbackLua => {
                // A Lua reference is called directly, with no arguments —
                // this is the "is it still wanted" question, not a
                // general-purpose call.
                let rv: Object = nlua_call_ref(
                    (*callback).data.luaref,
                    null::<c_char>(),
                    ARRAY_DICT_INIT,
                    kRetNilBool,
                    null_mut::<Arena>(),
                    null_mut::<Error>(),
                );
                return rv.type_0 == kObjectTypeBoolean && rv.data.boolean;
            }
            // kCallbackNone, and anything else.
            _ => return false,
        }

        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = partial;
        *callback_depth.ptr() += 1;
        let ret = call_func(name, -1, rettv, argcount_in, argvars_in, &raw mut funcexe);
        *callback_depth.ptr() -= 1;
        ret != 0
    }
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
    unsafe {
        match (*callback).type_0 {
            kCallbackPartial => {
                let mut tv = UNSET_TV;
                tv.v_type = VAR_PARTIAL;
                tv.vval.v_partial = (*callback).data.partial;
                set_ref_in_item(&raw mut tv, copy_id, ht_stack, list_stack)
            }
            // A Lua reference is the Lua garbage collector's, not this one's,
            // and nothing that reaches here should hold one.
            kCallbackLua => unreachable!("set_ref_in_callback on a Lua callback"),
            // kCallbackFuncref and kCallbackNone hold nothing collectable.
            _ => false,
        }
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
    unsafe {
        if set_ref_in_callback(&raw mut (*reader).cb, copy_id, ht_stack, list_stack) {
            return true;
        }
        if !(*reader).self_0.is_null() {
            let mut tv = UNSET_TV;
            tv.v_type = VAR_DICT;
            tv.vval.v_dict = (*reader).self_0;
            return set_ref_in_item(&raw mut tv, copy_id, ht_stack, list_stack);
        }
        false
    }
}
